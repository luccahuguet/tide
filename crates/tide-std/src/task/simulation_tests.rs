use super::*;
use crate::sync::{LockError, Shared};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

std::thread_local! {
    pub(super) static START_DELAY: Cell<Duration> = const { Cell::new(Duration::ZERO) };
}

fn bounded(f: impl FnOnce() + Send + 'static) {
    let (done, completed) = mpsc::channel();
    let worker = thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let _ = done.send(result);
    });
    match completed.recv_timeout(Duration::from_secs(60)) {
        Ok(Ok(())) => worker.join().unwrap(),
        Ok(Err(panic)) => resume_unwind(panic),
        Err(error) => panic!("simulation test failed to finish: {error}"),
    }
}

// Reruns one test of this module in a fresh process, with `variable` selecting its child role.
fn rerun(test: &str, variable: &str, value: &str) -> std::process::Output {
    let test = format!("{}::{test}", module_path!().split_once("::").unwrap().1);
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env(variable, value)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn message(panic: &(dyn std::any::Any + Send)) -> Option<&str> {
    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
}

#[derive(Debug, PartialEq, Eq)]
enum Ended {
    Body,
    Child,
    Task(TaskError),
}
impl From<TaskError> for Ended {
    fn from(error: TaskError) -> Self {
        Self::Task(error)
    }
}

fn two_tasks(seed: u64) -> (String, u32, u32) {
    let simulation = Simulation::new(seed);
    let tasks = simulation.tasks();
    let cell = Shared::new(0, tasks);
    let join_line = tasks
        .scope::<_, TaskError, _>(|scope| {
            let a = scope.spawn(|| {
                cell.with(|n| *n += 1).unwrap();
                Ok(())
            });
            let b = scope.spawn(|| {
                cell.with(|n| *n += 2).unwrap();
                Ok(())
            });
            let join_line = line!() + 1;
            a.join()?;
            drop(b);
            Ok(join_line)
        })
        .unwrap();
    let acquire_line = line!() + 1;
    assert_eq!(cell.with(|n| *n), Ok(3));
    (simulation.trace(), join_line, acquire_line)
}

#[test]
fn seeded_trace_replays_with_delayed_start_and_in_a_fresh_process() {
    const REPLAY: &str = "TIDE_SCHEDULER_TEST_REPLAY";
    const BEGIN: &str = "TIDE_SCHEDULER_TRACE_BEGIN\n";
    const END: &str = "TIDE_SCHEDULER_TRACE_END";
    bounded(|| {
        let (trace, _, _) = two_tasks(5);
        if std::env::var_os(REPLAY).is_some() {
            println!("{BEGIN}{trace}{END}");
            return;
        }
        START_DELAY.with(|delay| delay.set(Duration::from_millis(5)));
        assert_eq!(trace, two_tasks(5).0);
        START_DELAY.with(|delay| delay.set(Duration::ZERO));

        let child = rerun(
            "seeded_trace_replays_with_delayed_start_and_in_a_fresh_process",
            REPLAY,
            "1",
        );
        let output = String::from_utf8(child.stdout).unwrap();
        assert!(child.status.success(), "{output}");
        let (_, events) = output.split_once(BEGIN).expect(&output);
        assert_eq!(
            events.split_once(END).unwrap().0.as_bytes(),
            trace.as_bytes()
        );
    });
}

#[test]
fn seeds_choose_different_interleavings_and_trace_waits_at_call_sites() {
    bounded(|| {
        let mut choices = BTreeSet::new();
        let mut saw_blocked_join = false;
        let mut saw_scope_wait = false;
        for seed in 0..20 {
            let (trace, join_line, acquire_line) = two_tasks(seed);
            let at_join = format!("\"line\":{join_line},");
            let at_acquire = format!("\"line\":{acquire_line},");
            let mut saw_acquire = false;
            choices.insert(
                trace
                    .lines()
                    .filter(|line| line.contains("\"kind\":\"sched\""))
                    .map(|line| line.split_once("\"chosen\":").unwrap().1)
                    .collect::<Vec<_>>()
                    .join(","),
            );
            for line in trace.lines() {
                let has_file = line.contains(file!());
                if line.contains("\"kind\":\"join\"") && line.contains("\"wait\":\"blocked\"") {
                    assert!(has_file && line.contains(&at_join), "{line}");
                    saw_blocked_join = true;
                }
                if line.contains("\"kind\":\"scope_exit\"") && line.contains("\"wait\":\"blocked\"")
                {
                    saw_scope_wait = true;
                }
                if line.contains("\"operation\":\"cell_acquire\"") && line.contains(&at_acquire) {
                    assert!(has_file && line.contains("\"wait\":\"ready\""), "{line}");
                    saw_acquire = true;
                }
            }
            assert!(saw_acquire, "{trace}");
        }
        assert!(choices.len() > 1);
        assert!(saw_blocked_join);
        assert!(saw_scope_wait);
    });
}

fn assert_alone(inside: &AtomicBool) {
    // Use an atomic so the assertion itself stays race-free if the baton breaks.
    assert!(!inside.swap(true, Ordering::SeqCst));
    thread::yield_now();
    assert!(inside.swap(false, Ordering::SeqCst));
}

#[test]
fn baton_serializes_cells_and_joins_nested_dropped_and_forgotten_tasks() {
    bounded(|| {
        for seed in 0..200 {
            let simulation = Simulation::new(seed);
            let tasks = &simulation.tasks;
            let cell = &Shared::new(0, tasks);
            let inside = &AtomicBool::new(false);
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    let a = scope.spawn(|| {
                        assert_alone(inside);
                        tasks.scope(|nested| {
                            let grandchild = nested.spawn(|| {
                                assert_alone(inside);
                                Ok(11)
                            });
                            assert_eq!(grandchild.join()?, 11);
                            Ok(())
                        })?;
                        Ok(12)
                    });
                    let b = scope.spawn(move || {
                        assert_alone(inside);
                        assert_eq!(a.join()?, 12);
                        Ok(())
                    });
                    b.join()?;
                    for index in 0..3 {
                        let child = scope.spawn(move || {
                            for _ in 0..16 {
                                cell.with(|n| {
                                    assert_alone(inside);
                                    *n += 1;
                                })
                                .unwrap();
                                assert_alone(inside);
                                tasks.yield_now().unwrap();
                            }
                            Ok(())
                        });
                        if index == 0 {
                            std::mem::forget(child);
                        } else {
                            drop(child);
                        }
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(cell.with(|n| *n), Ok(48));
        }
    });
}

#[test]
fn a_release_can_reschedule_before_the_next_user_statement() {
    bounded(|| {
        let mut witnessed = false;
        for seed in 0..100 {
            let simulation = Simulation::new(seed);
            let tasks = &simulation.tasks;
            let cell = &Shared::new(0, tasks);
            let printed = &Mutex::new(Vec::new());
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    scope.spawn(|| {
                        cell.with(|n| *n = 1).unwrap();
                        printed.lock().unwrap().push("A");
                        Ok(())
                    });
                    scope.spawn(|| {
                        if cell.with(|n| *n).unwrap() == 1 {
                            printed.lock().unwrap().push("B");
                        }
                        Ok(())
                    });
                    Ok(())
                })
                .unwrap();
            witnessed |= *printed.lock().unwrap() == ["B", "A"];
        }
        assert!(witnessed);
    });
}

fn counter(seed: u64, split: bool) -> u32 {
    let simulation = Simulation::new(seed);
    let tasks = &simulation.tasks;
    let cell = &Shared::new(0, tasks);
    tasks
        .scope::<_, TaskError, _>(|scope| {
            for _ in 0..2 {
                scope.spawn(|| {
                    for _ in 0..4 {
                        if split {
                            let n = cell.with(|n| *n).unwrap();
                            cell.with(|value| *value = n + 1).unwrap();
                        } else {
                            cell.with(|n| *n += 1).unwrap();
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        })
        .unwrap();
    cell.with(|n| *n).unwrap()
}

#[test]
fn random_scheduling_exposes_split_updates_but_serializes_atomic_updates() {
    bounded(|| {
        let lost = (0..100).filter(|&seed| counter(seed, true) < 8).count();
        assert!(lost > 50, "lost updates in only {lost}/100 seeds");
        for seed in 0..100 {
            assert_eq!(counter(seed, false), 8);
        }
    });
}

#[test]
fn first_scheduled_failure_sets_the_cancellation_cause() {
    #[derive(Debug, PartialEq, Eq)]
    enum Failure {
        First,
        Second,
        Task(TaskError),
    }
    impl From<TaskError> for Failure {
        fn from(error: TaskError) -> Self {
            Self::Task(error)
        }
    }
    bounded(|| {
        let mut winners = Vec::new();
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = &simulation.tasks.clone();
            let result = tasks.scope::<(), Failure, _>(|scope| {
                scope.spawn(|| {
                    let failure = if tasks.checkpoint().is_err() {
                        Failure::Second
                    } else {
                        Failure::First
                    };
                    Err::<(), _>(failure)
                });
                scope.spawn(|| Err::<(), _>(Failure::Second));
                Ok(())
            });
            let winner = result.unwrap_err();
            assert_eq!(
                simulation.trace().contains("\"outcome\":\"echo\""),
                winner == Failure::Second
            );
            winners.push(winner);
        }
        assert!(winners.contains(&Failure::First) && winners.contains(&Failure::Second));
    });
}

#[test]
fn callback_panic_releases_the_baton_poisons_the_cell_and_reaches_scope() {
    bounded(|| {
        let simulation = Simulation::new(5);
        let tasks = &simulation.tasks;
        let cell = &Shared::new(0, tasks);
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), TaskError> = tasks.scope(|scope| {
                let child = scope.spawn(|| {
                    cell.with(|_| std::panic::panic_any(73u32)).unwrap();
                    Ok(())
                });
                assert_eq!(child.join(), Err(TaskError::Cancelled));
                assert!(tasks.checkpoint().is_err());
                assert_eq!(
                    cell.with(|_| unreachable!()),
                    Err::<(), _>(LockError::Poisoned)
                );
                Ok(())
            });
        }))
        .unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 73);
        assert!(simulation.trace().contains("\"outcome\":\"panic\""));
    });
}

#[test]
fn dropping_unjoined_values_keeps_the_baton_until_their_destructor_finishes() {
    struct Value<'a>(&'a Shared<u32>);
    impl Drop for Value<'_> {
        fn drop(&mut self) {
            self.0.with(|n| *n += 1).unwrap();
        }
    }

    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = simulation.tasks();
            let cell = &Shared::new(0, tasks);
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    drop(scope.spawn(|| Ok(Value(cell))));
                    Ok(())
                })
                .unwrap();
            assert_eq!(cell.with(|n| *n), Ok(1));
        }
    });
}

#[test]
fn simulated_handles_require_their_own_live_simulation() {
    let real = Tasks::real();
    let real_cell = Shared::new(1, &real);
    let (tasks, cell) = {
        let simulation = Simulation::new(0);
        let tasks = simulation.tasks.clone();
        let cell = Shared::new(5, &tasks);
        (tasks, cell)
    };
    let fails = || {
        let panic = catch_unwind(|| tasks.checkpoint()).unwrap_err();
        assert_eq!(
            message(&*panic),
            Some("simulation handle used outside its simulation")
        );
        assert!(catch_unwind(|| cell.with(|_| ())).is_err());
    };
    fails();
    let simulation = Simulation::new(0);
    fails();
    assert_eq!(Shared::new(7, &simulation.tasks).with(|n| *n), Ok(7));
    let misuses: [&dyn Fn(); 5] = [
        &|| _ = real.checkpoint(),
        &|| _ = real.yield_now(),
        &|| _ = real.scope::<(), TaskError, _>(|_| Ok(())),
        &|| _ = Shared::new(2, &real),
        &|| _ = real_cell.with(|n| *n),
    ];
    for misuse in misuses {
        let panic = catch_unwind(AssertUnwindSafe(misuse)).unwrap_err();
        assert_eq!(
            message(&*panic),
            Some("real task authority used inside a simulation")
        );
    }
    drop(simulation);
    assert_eq!(real_cell.with(|n| *n), Ok(1));
}

#[test]
fn cancel_takes_no_scheduling_point_and_the_next_checkpoint_observes_it() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = simulation.tasks();
            let result = tasks.scope::<(), Ended, _>(|scope| {
                // This child stays runnable until it observes the cancellation.
                scope.spawn(|| {
                    while tasks.checkpoint().is_ok() {}
                    Err::<(), _>(Ended::Child)
                });
                let before = simulation.trace();
                scope.cancel();
                scope.cancel();
                let trace = simulation.trace();
                // A point here would add its own record and, with two runnable tasks, a
                // `sched` record with a draw. Only the first-cause transition is recorded.
                let added = trace.strip_prefix(before.as_str()).unwrap();
                assert_eq!(added.lines().count(), 1, "{added}");
                assert!(
                    added.contains("\"kind\":\"cancel\"")
                        && added.contains("\"action\":\"cancelled\""),
                    "{added}"
                );
                assert!(tasks.checkpoint().is_err());
                Ok(())
            });
            assert_eq!(result, Err(Ended::Child));
        }
    });
}

#[test]
fn body_error_with_unfinished_children_cancels_once_and_blocks_at_scope_exit() {
    bounded(|| {
        for seed in 0..20 {
            for explicit in [false, true] {
                let simulation = Simulation::new(seed);
                let tasks = simulation.tasks();
                let result = tasks.scope::<(), Ended, _>(|scope| {
                    scope.spawn(|| {
                        while tasks.checkpoint().is_ok() {}
                        Err::<(), _>(Ended::Child)
                    });
                    if explicit {
                        scope.cancel();
                        scope.cancel();
                    }
                    Err(Ended::Body)
                });
                // The child's error is independent after an explicit cancel, and an echo of
                // the body's failure otherwise.
                let (winner, outcome) = if explicit {
                    (Ended::Child, "\"outcome\":\"error\"")
                } else {
                    (Ended::Body, "\"outcome\":\"echo\"")
                };
                assert_eq!(result, Err(winner));
                let trace = simulation.trace();
                let count = |a: &str, b: &str| {
                    trace
                        .lines()
                        .filter(|line| line.contains(a) && line.contains(b))
                        .count()
                };
                assert_eq!(count("\"action\":\"cancelled\"", ""), 1, "{trace}");
                assert_eq!(
                    count("\"kind\":\"scope_exit\"", "\"wait\":\"blocked\""),
                    1,
                    "{trace}"
                );
                assert_eq!(count(outcome, ""), 1, "{trace}");
            }
        }
    });
}

#[test]
fn rejected_native_spawn_keeps_its_panic_and_leaves_no_phantom_task() {
    struct Reject;
    impl<'s> Spawner<'s> for Reject {
        fn spawn(&'s self, job: Box<dyn FnOnce() + Send + 's>) {
            // The rejected job's captured value reaches scheduling points as it drops.
            drop(job);
            std::panic::panic_any(41u32);
        }
    }
    struct Value<'a>(&'a Shared<u32>);
    impl Drop for Value<'_> {
        fn drop(&mut self) {
            self.0.with(|n| *n += 1).unwrap();
        }
    }

    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = simulation.tasks();
            let cell = &Shared::new(0, tasks);
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    // Task 1 gives those points a second runnable task to choose from.
                    scope.spawn(|| {
                        let _ = tasks.yield_now();
                        Ok(())
                    });
                    let value = Value(cell);
                    let rejecting = Scope {
                        threads: &Reject,
                        ..scope
                    };
                    let panic = catch_unwind(AssertUnwindSafe(|| {
                        rejecting.spawn(move || {
                            let _value = value;
                            Ok(())
                        });
                    }))
                    .unwrap_err();
                    assert_eq!(*panic.downcast::<u32>().unwrap(), 41);
                    assert_eq!(cell.with(|n| *n), Ok(1));
                    Ok(())
                })
                .unwrap();
            // The abandoned registration, task 2, never ran and was never runnable.
            let trace = simulation.trace();
            for line in trace.lines() {
                assert!(!line.contains("\"task\":2,"), "{line}");
                if let Some((_, runnable)) = line.split_once("\"runnable\":[") {
                    let runnable = runnable.split_once(']').unwrap().0;
                    assert!(runnable.split(',').all(|id| id != "2"), "{line}");
                }
            }
        }
    });
}

#[cfg(unix)]
#[test]
fn poisoned_lock_and_empty_runnable_set_abort_the_process() {
    const FATAL: &str = "TIDE_SCHEDULER_TEST_FATAL";
    const NAME: &str = "poisoned_lock_and_empty_runnable_set_abort_the_process";
    if let Ok(mode) = std::env::var(FATAL) {
        bounded(move || {
            let simulation = Simulation::new(0);
            let tasks = simulation.tasks.clone();
            // Without the guard's own lock, only the adapter path under test can abort.
            std::mem::forget(simulation);
            if mode == "poison" {
                let runtime = tasks.simulation.clone().unwrap();
                let _ = thread::spawn(move || {
                    let _held = runtime.lock().unwrap();
                    panic!("poisoning the simulator lock");
                })
                .join();
                let _ = tasks.checkpoint();
            } else {
                // Task 0 joining itself leaves no runnable task.
                let main = Context::current().unwrap();
                main.wait_for(Operation::Join(0), std::panic::Location::caller());
            }
        });
        return;
    }
    for (mode, report) in [
        ("poison", "simulator runtime lock poisoned"),
        (
            "stuck",
            "no runnable task in the simulator\n  task 0 blocked in Join(0) at ",
        ),
    ] {
        let child = rerun(NAME, FATAL, mode);
        let stderr = String::from_utf8_lossy(&child.stderr);
        // SIGABRT: an unwind would exit with the harness's failure code instead.
        assert_eq!(
            std::os::unix::process::ExitStatusExt::signal(&child.status),
            Some(6),
            "{stderr}"
        );
        assert!(stderr.contains(report), "{stderr}");
    }
}
