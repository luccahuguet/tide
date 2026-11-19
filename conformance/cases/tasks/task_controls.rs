//@ rules: SCOPE-1, SCOPE-3, SEND-3, TASK-6
// Production and conformance share these APIs; the driver executes both runtimes.
use std::sync::Shared;
use std::task::{Cancelled, Panicked, TaskError, Tasks};
use std::time::{Clock, Duration};

fn isolated_result<'a>(tasks: &Tasks, _: &'a str) -> std::marker::PhantomData<&'a str> {
    tasks.spawn_isolated(|| std::marker::PhantomData).unwrap()
}

fn exercise(tasks: &Tasks, clock: &Clock) -> Result<(), TaskError> {
    let counts = Shared::new((0usize, 0usize), tasks);
    let limited = tasks.limit(2);
    limited.scope::<_, TaskError, _>(|s| {
        for _ in 0..6 {
            s.spawn(|| {
                counts
                    .with(|c| {
                        c.0 += 1;
                        c.1 = c.1.max(c.0);
                        assert!(c.0 <= 2);
                    })
                    .unwrap();
                clock.sleep(Duration::from_millis(1))?;
                counts.with(|c| c.0 -= 1).unwrap();
                Ok(())
            });
        }
        Ok(())
    })?;
    counts
        .with(|c| {
            assert_eq!(c.0, 0);
            assert!(c.1 > 0 && c.1 <= 2);
        })
        .unwrap();
    tasks.limit(0).scope::<_, TaskError, _>(|_| Ok(()))?;

    let one = tasks.limit(1);
    let result: Result<(), TaskError> = one.scope(|s| {
        s.spawn(move || {
            // A task holding the only slot cannot start another before this deadline.
            s.set_deadline(clock.now() + Duration::from_millis(2));
            let never = s.spawn(|| -> Result<(), TaskError> { panic!("cancelled closure ran") });
            assert_eq!(never.join(), Err(TaskError::Cancelled));
            Ok(())
        });
        Ok(())
    });
    assert_eq!(result, Err(TaskError::DeadlineExceeded));
    assert_eq!(one.scope::<_, TaskError, _>(|s| s.spawn(|| Ok(19)).join())?, 19);

    let at = clock.now() + Duration::from_millis(2);
    let result: Result<(), TaskError> = tasks.scope(|s| {
        s.set_deadline(at);
        s.set_deadline(at + Duration::from_secs(20));
        // `Ok` from the sleep would leave the scope `Ok`: the cancellation is observed.
        s.spawn(|| Ok(clock.sleep_until(at)?));
        Ok(())
    });
    assert_eq!(result, Err(TaskError::DeadlineExceeded));
    let partial = tasks.scope::<_, TaskError, _>(|s| {
        s.set_deadline(clock.now() + Duration::from_millis(2));
        s.spawn(|| {
            assert_eq!(clock.sleep(Duration::from_secs(20)), Err(Cancelled));
            Ok(41)
        })
        .join()
    })?;
    assert_eq!(partial, 41);

    tasks.scope::<_, TaskError, _>(|s| {
        s.cancel();
        for _ in 0..3 {
            assert_eq!(tasks.checkpoint(), Err(Cancelled));
        }
        tasks.shield(|| {
            tasks.checkpoint().unwrap();
            tasks.scope::<_, TaskError, _>(|inner| {
                inner.set_deadline(clock.now());
                assert_eq!(tasks.checkpoint(), Err(Cancelled));
                Ok(())
            })
        })?;
        assert_eq!(tasks.checkpoint(), Err(Cancelled));
        let observer = tasks.clone();
        assert_eq!(
            tasks.spawn_isolated(move || observer.checkpoint()).unwrap(),
            Err(Cancelled)
        );
        Ok(())
    })?;
    let local = String::from("borrowed type");
    let _ = isolated_result(tasks, &local);
    let panic: Panicked = tasks.scope::<_, TaskError, _>(|_| {
        let panic = tasks
            .spawn_isolated(|| -> u32 { panic!("contained task panic") })
            .unwrap_err();
        tasks.checkpoint().unwrap();
        Ok(panic)
    })?;
    assert_eq!(panic.message, "contained task panic");
    assert_eq!(panic.to_string(), "contained task panic");
    one.scope::<_, TaskError, _>(|s| {
        s.spawn(|| {
            assert_eq!(tasks.spawn_isolated(|| 73).unwrap(), 73);
            Ok(())
        })
        .join()
    })?;
    Ok(tasks.checkpoint()?)
}

#[tide::main]
fn main(root: Root) -> Result<(), TaskError> {
    exercise(&root.tasks, &root.clock)?;
    println!("TASK_CONTROLS_RAN");
    Ok(())
}

#[test]
fn simulated_controls() {
    for seed in 0..12 {
        let sim = Root::sim(seed);
        exercise(&sim.tasks, &sim.clock).unwrap();
        assert!(sim.trace().contains("\"operation\":\"deadline\""));
        assert!(sim.trace().contains("\"result\":\"not_started\""));
        assert!(sim.trace().contains("\"isolated\":true"));
    }
}
