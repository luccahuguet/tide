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

pub(super) fn bounded(f: impl FnOnce() + Send + 'static) {
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

struct OnDrop<F: Fn()>(F);
impl<F: Fn()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

// Effect rules map to these tests in research/34-effect-interruption-oracle.md.
#[test]
fn effect_oracle_nested_scope_close_order() {
    bounded(|| {
        for seed in 0..20 {
            for cancel in [false, true] {
                let simulation = Simulation::new(seed);
                let (tasks, clock) = (simulation.tasks(), simulation.clock());
                let events = &Mutex::new(Vec::new());
                let (ready, started) = crate::sync::mpsc::channel(tasks);
                let result = tasks.scope::<_, Ended, _>(|outer| {
                    drop(outer.spawn(|| {
                        let _parent = OnDrop(|| events.lock().unwrap().push("parent-drop"));
                        let result = tasks.scope::<_, Ended, _>(|inner| {
                            drop(inner.spawn(|| {
                                let _first = OnDrop(|| events.lock().unwrap().push("first-drop"));
                                let _second = OnDrop(|| events.lock().unwrap().push("second-drop"));
                                ready.send(()).unwrap();
                                clock.sleep(SECOND).map_err(|_| Ended::Child)
                            }));
                            Ok(())
                        });
                        events.lock().unwrap().push("inner-closed");
                        result
                    }));
                    started.recv().unwrap();
                    if cancel { Err(Ended::Body) } else { Ok(()) }
                });
                assert_eq!(result, if cancel { Err(Ended::Body) } else { Ok(()) });
                events.lock().unwrap().push("outer-closed");
                assert_eq!(
                    *events.lock().unwrap(),
                    [
                        "second-drop",
                        "first-drop",
                        "inner-closed",
                        "parent-drop",
                        "outer-closed"
                    ]
                );
                assert_eq!(
                    simulation.tasks().sim_elapsed(),
                    if cancel { Duration::ZERO } else { SECOND }
                );
            }
        }
    });
}

#[test]
fn effect_oracle_passed_scope_outlives_spawning_task_but_not_scope() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let (tasks, clock) = (simulation.tasks(), simulation.clock());
            let parent_done = &AtomicBool::new(false);
            let child_done = &AtomicBool::new(false);
            tasks
                .scope::<_, TaskError, _>(|outer| {
                    outer
                        .spawn(move || {
                            // A passed scope owns this child, even after its spawning task returns.
                            drop(outer.spawn(move || {
                                clock.sleep(SECOND).unwrap();
                                assert!(parent_done.load(Ordering::SeqCst));
                                child_done.store(true, Ordering::SeqCst);
                                Ok(())
                            }));
                            parent_done.store(true, Ordering::SeqCst);
                            Ok(())
                        })
                        .join()?;
                    assert!(!child_done.load(Ordering::SeqCst));
                    Ok(())
                })
                .unwrap();
            assert!(child_done.load(Ordering::SeqCst));
            assert_eq!(simulation.tasks().sim_elapsed(), SECOND);
        }
    });
}

#[test]
fn effect_oracle_cancel_is_level_triggered_and_join_waits_for_cleanup() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let (tasks, clock) = (simulation.tasks(), simulation.clock());
            let finished = &AtomicBool::new(false);
            let (ready, started) = crate::sync::mpsc::channel(tasks);
            let result = tasks.scope::<_, Ended, _>(|scope| {
                let child = scope.spawn(|| {
                    let _drop = OnDrop(|| finished.store(true, Ordering::SeqCst));
                    ready.send(()).unwrap();
                    assert_eq!(clock.sleep(Duration::MAX), Err(Cancelled));
                    assert_eq!(tasks.checkpoint(), Err(Cancelled));
                    assert_eq!(clock.sleep(Duration::ZERO), Err(Cancelled));
                    tasks.shield(|| clock.sleep(2 * SECOND).unwrap());
                    assert_eq!(tasks.checkpoint(), Err(Cancelled));
                    Err::<(), _>(Ended::Child)
                });
                started.recv().unwrap();
                scope.cancel();
                assert_eq!(simulation.tasks().sim_elapsed(), Duration::ZERO);
                assert!(!finished.load(Ordering::SeqCst));
                assert_eq!(child.join(), Err(Ended::Task(TaskError::Cancelled)));
                assert!(finished.load(Ordering::SeqCst));
                assert_eq!(simulation.tasks().sim_elapsed(), 2 * SECOND);
                Ok(())
            });
            assert_eq!(result, Err(Ended::Child));
        }
    });
}

#[test]
fn effect_oracle_cancel_during_drop_runs_remaining_destructors() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = simulation.tasks();
            let stage = &Shared::new(0, tasks);
            let events = &Mutex::new(Vec::new());
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    scope.spawn(|| {
                        let _last = OnDrop(|| events.lock().unwrap().push("last-drop"));
                        let _first = OnDrop(|| {
                            events.lock().unwrap().push("drop-start");
                            // Drop waits on a Shared handle to force the overlap.
                            stage.with(|n| *n = 1).unwrap();
                            while stage.with(|n| *n).unwrap() != 2 {}
                            events.lock().unwrap().push("drop-end");
                        });
                        Ok(())
                    });
                    scope.spawn(move || {
                        while stage.with(|n| *n).unwrap() != 1 {}
                        scope.cancel();
                        events.lock().unwrap().push("cancel");
                        stage.with(|n| *n = 2).unwrap();
                        assert_eq!(tasks.checkpoint(), Err(Cancelled));
                        Ok(())
                    });
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                *events.lock().unwrap(),
                ["drop-start", "cancel", "drop-end", "last-drop"]
            );
        }
    });
}

#[test]
fn effect_oracle_cancel_during_shielded_cleanup_keeps_inner_deadline() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let (tasks, clock) = (simulation.tasks(), simulation.clock());
            let start = clock.now();
            let (ready, started) = crate::sync::mpsc::channel(tasks);
            tasks
                .scope::<_, TaskError, _>(|outer| {
                    let child = outer.spawn(|| {
                        tasks.shield(|| {
                            tasks
                                .scope::<_, TaskError, _>(|cleanup| {
                                    cleanup.set_deadline(start + 2 * SECOND);
                                    ready.send(()).unwrap();
                                    assert_eq!(clock.sleep(3 * SECOND), Err(Cancelled));
                                    assert_eq!(clock.now(), start + 2 * SECOND);
                                    assert_eq!(tasks.checkpoint(), Err(Cancelled));
                                    Ok(())
                                })
                                .unwrap();
                            tasks.checkpoint().unwrap();
                        });
                        assert_eq!(tasks.checkpoint(), Err(Cancelled));
                        Ok(19)
                    });
                    started.recv().unwrap();
                    clock.sleep(SECOND).unwrap();
                    outer.cancel();
                    assert_eq!(child.join(), Ok(19));
                    Ok(())
                })
                .unwrap();
            assert_eq!(clock.now(), start + 2 * SECOND);
        }
    });
}

#[test]
fn effect_oracle_child_panics_during_parent_cancellation_drain_all_children() {
    bounded(|| {
        for seed in 0..20 {
            for body_panics in [false, true] {
                let simulation = Simulation::new(seed);
                let (tasks, clock) = (simulation.tasks(), simulation.clock());
                let dropped = &Mutex::new(Vec::new());
                let (ready, started) = crate::sync::mpsc::channel(tasks);
                let panic = catch_unwind(AssertUnwindSafe(|| {
                    let _: Result<(), Ended> = tasks.scope(|scope| {
                        for (name, delay) in
                            [("first-child", 2), ("second-child", 1), ("survivor", 3)]
                        {
                            let ready = ready.clone();
                            scope.spawn(move || {
                                let _drop = OnDrop(|| dropped.lock().unwrap().push(name));
                                ready.send(()).unwrap();
                                assert_eq!(clock.sleep(Duration::MAX), Err(Cancelled));
                                tasks.shield(|| clock.sleep(delay * SECOND).unwrap());
                                if name != "survivor" {
                                    panic!("{name}");
                                }
                                Err::<(), _>(Ended::Child)
                            });
                        }
                        for _ in 0..3 {
                            started.recv().unwrap();
                        }
                        if body_panics {
                            panic!("body-panic");
                        }
                        Err(Ended::Body)
                    });
                }))
                .unwrap_err();
                assert_eq!(
                    message(&*panic),
                    Some(if body_panics {
                        "body-panic"
                    } else {
                        "first-child"
                    })
                );
                assert_eq!(
                    *dropped.lock().unwrap(),
                    ["second-child", "first-child", "survivor"]
                );
                assert_eq!(simulation.tasks().sim_elapsed(), 3 * SECOND);
                tasks.checkpoint().unwrap();
            }
        }
    });
}

#[test]
fn effect_oracle_deadline_expires_while_children_clean_up() {
    bounded(|| {
        for seed in 0..20 {
            for fail in [false, true] {
                let simulation = Simulation::new(seed);
                let (tasks, clock) = (simulation.tasks(), simulation.clock());
                let start = clock.now();
                let dropped = &AtomicBool::new(false);
                let result = tasks.scope::<_, Ended, _>(|scope| {
                    scope.set_deadline(start + SECOND);
                    scope.spawn(|| {
                        let _drop = OnDrop(|| dropped.store(true, Ordering::SeqCst));
                        // Ordinary Err propagation performs explicit I/O cleanup before return.
                        let work = if fail { Err(Ended::Child) } else { Ok(23) };
                        tasks.shield(|| clock.sleep(3 * SECOND).unwrap());
                        assert_eq!(tasks.checkpoint(), Err(Cancelled));
                        work
                    });
                    Ok(7)
                });
                assert_eq!(
                    result,
                    if fail {
                        Err(Ended::Task(TaskError::DeadlineExceeded))
                    } else {
                        Ok(7)
                    }
                );
                assert!(dropped.load(Ordering::SeqCst));
                assert_eq!(clock.now(), start + 3 * SECOND);
            }
        }
    });
}

#[test]
fn effect_oracle_deadline_during_panic_unwind_preserves_panic() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let (tasks, clock) = (simulation.tasks(), simulation.clock());
            let start = clock.now();
            let dropped = &AtomicBool::new(false);
            let panic = catch_unwind(AssertUnwindSafe(|| {
                let _: Result<(), TaskError> = tasks.scope(|scope| {
                    scope.set_deadline(start + SECOND);
                    scope.spawn(|| -> Result<(), TaskError> {
                        let _drop = OnDrop(|| {
                            // A shielded delay in Drop forces expiry while Rust is unwinding.
                            assert!(thread::panicking());
                            tasks.shield(|| clock.sleep(3 * SECOND).unwrap());
                            // Nothing requested cancellation. This runtime counts the child's
                            // panic only after unwinding (TASK-1/2 leave that timing open),
                            // so here the cancellation seen is the deadline's.
                            assert_eq!(tasks.checkpoint(), Err(Cancelled));
                            dropped.store(true, Ordering::SeqCst);
                        });
                        panic!("child-unwinding");
                    });
                    Ok(())
                });
            }))
            .unwrap_err();
            assert_eq!(message(&*panic), Some("child-unwinding"));
            assert!(dropped.load(Ordering::SeqCst));
            assert_eq!(clock.now(), start + 3 * SECOND);
            tasks.checkpoint().unwrap();
        }
    });
}

#[test]
fn effect_oracle_cancel_after_task_completion_preserves_value_and_drops_once() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let tasks = simulation.tasks();
            let drops = &Mutex::new(0);
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    let child = scope.spawn(|| {
                        let _drop = OnDrop(|| *drops.lock().unwrap() += 1);
                        Ok(31)
                    });
                    // Observe actual completion, without consuming the handle in join.
                    // The destructor alone precedes completion publication.
                    while !child.done.ready() {
                        tasks.yield_now().unwrap();
                    }
                    scope.cancel();
                    scope.cancel();
                    assert_eq!(child.join(), Ok(31));
                    assert_eq!(*drops.lock().unwrap(), 1);
                    assert_eq!(tasks.checkpoint(), Err(Cancelled));
                    Ok(())
                })
                .unwrap();
            assert_eq!(*drops.lock().unwrap(), 1);
            assert_eq!(simulation.tasks().sim_elapsed(), Duration::ZERO);
        }
    });
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
    (simulation.tasks().sim_trace(), join_line, acquire_line)
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
                if line.contains("\"operation\":\"cells_acquire\"") && line.contains(&at_acquire) {
                    assert!(
                        has_file
                            && line.contains("\"wait\":\"ready\"")
                            && line.contains("\"targets\":[0]"),
                        "{line}"
                    );
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
                        tasks.scope::<_, TaskError, _>(|nested| {
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
                simulation.tasks().sim_trace().contains("\"outcome\":\"echo\""),
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
        assert!(simulation.tasks().sim_trace().contains("\"outcome\":\"panic\""));
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
    let real_clock = crate::time::Clock::new(Time::real());
    let fs = crate::fs::Fs::root();
    let file = fs.open("Cargo.toml").unwrap();
    let (tasks, cell, clock) = {
        let simulation = Simulation::new(0);
        let tasks = simulation.tasks.clone();
        let cell = Shared::new(5, &tasks);
        (tasks, cell, simulation.clock().clone())
    };
    let fails = || {
        let panic = catch_unwind(|| tasks.checkpoint()).unwrap_err();
        assert_eq!(
            message(&*panic),
            Some("simulation handle used outside its simulation")
        );
        assert!(catch_unwind(|| cell.with(|_| ())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| clock.now())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| clock.system_time())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| clock.sleep(Duration::ZERO))).is_err());
    };
    fails();
    let simulation = Simulation::new(0);
    fails();
    assert_eq!(Shared::new(7, &simulation.tasks).with(|n| *n), Ok(7));
    let misuses: [&dyn Fn(); 12] = [
        &|| _ = real_clock.now(),
        &|| _ = real_clock.system_time(),
        &|| _ = real_clock.sleep(Duration::ZERO),
        &|| _ = real_clock.sleep_until(real_clock.monotonic_only().now()),
        &|| _ = fs.exists("Cargo.toml"),
        &|| _ = crate::io::Read::read(&mut &file, &mut [0]),
        &|| _ = fs.open("Cargo.toml").map(crate::fs::File::close),
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
            Some("real capability used inside a simulation")
        );
    }
    let close = catch_unwind(AssertUnwindSafe(|| file.close())).unwrap_err();
    assert_eq!(
        message(&*close),
        Some("real capability used inside a simulation")
    );
    drop(simulation);
    assert_eq!(real_cell.with(|n| *n), Ok(1));
    assert!(fs.exists("Cargo.toml").unwrap());
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
                let before = simulation.tasks().sim_trace();
                scope.cancel();
                scope.cancel();
                let trace = simulation.tasks().sim_trace();
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
                let trace = simulation.tasks().sim_trace();
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
            let trace = simulation.tasks().sim_trace();
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
            let clock = simulation.clock().clone();
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
            } else if mode == "forever" {
                // A sleep past the last instant never ends, and no other timer is pending.
                clock.sleep(Duration::from_nanos(1)).unwrap();
                let _ = clock.sleep(Duration::MAX);
            } else {
                // Task 0 joining itself leaves no runnable task.
                let main = Context::current().unwrap();
                main.wait(
                    Operation::Join(0),
                    std::panic::Location::caller(),
                    None,
                    false,
                );
            }
        });
        return;
    }
    for (mode, report) in [
        ("poison", "simulator runtime lock poisoned"),
        (
            "stuck",
            "deadlock: no runnable task in the simulator and no timer pending (virtual time \
             0ns)\n  task 0 blocked in join(0) at ",
        ),
        (
            "forever",
            "deadlock: no runnable task in the simulator and no timer pending (virtual time \
             1ns)\n  task 0 blocked in clock.sleep forever at ",
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
        if mode != "poison" {
            let events: Vec<_> = stderr
                .lines()
                .filter(|line| line.starts_with('{'))
                .collect();
            assert!(!events.is_empty(), "{stderr}");
            for (step, event) in events.iter().enumerate() {
                assert!(
                    event.starts_with(&format!("{{\"step\":{step},")),
                    "{stderr}"
                );
                assert!(event.ends_with('}'), "{stderr}");
            }
            let last = events.last().unwrap();
            assert_eq!(stderr.lines().last(), Some(*last), "{stderr}");
            assert!(
                last.contains("\"kind\":\"sched\",\"operation\":\"deadlock\""),
                "{stderr}"
            );
            assert!(
                last.contains("\"blocked\":[{\"task\":0,\"operation\":"),
                "{stderr}"
            );
        }
    }
}

const SECOND: Duration = Duration::from_secs(1);

#[test]
fn a_real_wait_checks_cancellation_then_readiness_before_timeout() {
    bounded(|| {
        let mut context = Context::detached();
        let scope = context.lock().scope(None);
        context.at = Some(Location { scope, child: None });
        let operation = Operation::ScopeExit(scope);
        let span = std::panic::Location::caller();
        assert!(matches!(
            context.wait(operation, span, Some(Duration::ZERO), true),
            Wake::Ready
        ));
        context.lock().cancel(scope, Cause::Explicit(scope));
        assert!(matches!(
            context.wait(operation, span, Some(Duration::ZERO), true),
            Wake::Cancelled
        ));
        assert!(context.lock().waiting.is_empty());
    });
}

#[test]
fn a_real_wait_rechecks_readiness_after_spurious_and_resource_wakes() {
    bounded(|| {
        for cancellable in [false, true] {
            let mut context = Context::detached();
            let scope = context.lock().scope(None);
            let child = context.lock().spawn(scope);
            context.at = Some(Location { scope, child: None });
            let waiting = context.clone();
            let (done, completed) = mpsc::channel();
            let until = crate::time::monotonic() + 5 * SECOND;
            let worker = thread::spawn(move || {
                let wake = waiting.wait(
                    Operation::ScopeExit(scope),
                    std::panic::Location::caller(),
                    Some(until),
                    cancellable,
                );
                done.send(matches!(wake, Wake::Ready)).unwrap();
            });
            let waiter = loop {
                if let Some(waiter) = context.lock().waiting.first().cloned() {
                    break waiter;
                }
                assert!(crate::time::monotonic() < until, "waiter never registered");
                thread::yield_now();
            };
            waiter.unpark();
            assert!(completed.recv_timeout(Duration::from_millis(20)).is_err());
            {
                let mut runtime = context.lock();
                runtime.finish(child, Outcome::Ok);
                // The resource changes its condition and notifies under the registration lock.
                waiter.unpark();
            }
            assert!(completed.recv_timeout(SECOND).unwrap());
            worker.join().unwrap();
            assert!(context.lock().waiting.is_empty());
        }
    });
}

#[test]
fn the_last_representable_virtual_instant_is_a_finite_sleep_target() {
    bounded(|| {
        let simulation = Simulation::new(0);
        let clock = simulation.clock();
        let start = clock.now();
        clock.sleep(Duration::MAX).unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), Duration::MAX);
        assert_eq!(clock.now() - start, Duration::MAX);
        let trace = simulation.tasks().sim_trace();
        let panic = catch_unwind(AssertUnwindSafe(|| clock.system_time())).unwrap_err();
        assert_eq!(
            message(&*panic),
            Some("`system_time()` exceeds the representable wall-time range")
        );
        assert_eq!(simulation.tasks().sim_trace(), trace);
        // The range failure did not poison the runtime or consume a trace step.
        assert_eq!(clock.now() - start, Duration::MAX);
        simulation.tasks().checkpoint().unwrap();
        clock.sleep(Duration::ZERO).unwrap();
        let after = simulation.tasks().sim_trace();
        assert!(after.starts_with(&trace), "{after}");
        for (step, line) in after.lines().enumerate() {
            assert!(line.starts_with(&format!("{{\"step\":{step},")), "{line}");
            assert!(line.ends_with('}'), "{line}");
        }
    });
}

#[test]
fn a_sleeper_wakes_at_its_exact_virtual_instant_and_only_blocked_tasks_move_time() {
    bounded(|| {
        let started = std::time::Instant::now();
        let simulation = Simulation::new(3);
        let (tasks, clock) = (simulation.tasks(), simulation.clock());
        let wall = |clock: &crate::time::Clock| {
            let since = clock.system_time().duration_since(crate::time::UNIX_EPOCH);
            since.unwrap()
        };
        let start = clock.now();
        // 2000-01-01T00:00:00Z.
        assert_eq!(wall(clock), Duration::from_secs(946_684_800));
        // Checkpoints, clock readings and sleeps that do not block take no virtual time.
        for _ in 0..10_000 {
            tasks.checkpoint().unwrap();
        }
        clock.sleep(Duration::ZERO).unwrap();
        clock.sleep_until(start).unwrap();
        assert_eq!((clock.now(), simulation.tasks().sim_elapsed()), (start, Duration::ZERO));
        tasks
            .scope::<_, TaskError, _>(|scope| {
                scope.spawn(|| {
                    clock.sleep(5 * SECOND).unwrap();
                    assert_eq!(clock.now(), start + 5 * SECOND);
                    Ok(())
                });
                Ok(())
            })
            .unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), 5 * SECOND);
        clock
            .monotonic_only()
            .sleep_until(start + 7 * SECOND)
            .unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), 7 * SECOND);
        assert_eq!(clock.now() - start, 7 * SECOND);
        assert_eq!(wall(clock), Duration::from_secs(946_684_807));
        // Seven virtual seconds pass without waiting for them.
        assert!(started.elapsed() < 5 * SECOND);

        let trace = simulation.tasks().sim_trace();
        let lines = |a: &str, b: &str| -> Vec<&str> {
            let matching = trace
                .lines()
                .filter(|line| line.contains(a) && line.ends_with(b));
            matching.collect()
        };
        let advances = lines("\"kind\":\"sched\",\"operation\":\"advance\"", "}");
        assert_eq!(advances.len(), 2, "{trace}");
        assert!(advances[0].contains("\"time\":0,"), "{trace}");
        assert!(
            advances[0].ends_with(",\"from\":0,\"to\":5000000000}"),
            "{trace}"
        );
        assert!(advances[1].contains("\"time\":5000000000,"), "{trace}");
        assert!(
            advances[1].ends_with(",\"from\":5000000000,\"to\":7000000000}"),
            "{trace}"
        );
        let sleep = "\"kind\":\"cap\",\"operation\":\"clock.sleep\"";
        for (operation, end, count) in [
            (sleep, "\"wait\":\"blocked\",\"until\":5000000000}", 1),
            (
                sleep,
                "\"wait\":\"ready\",\"until\":5000000000,\"action\":\"wake\"}",
                1,
            ),
            (
                sleep,
                "\"wait\":null,\"until\":5000000000,\"result\":\"ok\"}",
                1,
            ),
            // The zero sleep is a point, not a wait.
            (sleep, "\"wait\":\"ready\",\"until\":0}", 1),
            (
                "\"operation\":\"clock.sleep_until\"",
                ",\"result\":\"ok\"}",
                2,
            ),
            (
                "\"kind\":\"cap\",\"operation\":\"clock.now\"",
                "\"wait\":\"ready\",\"result\":0}",
                2,
            ),
            ("\"operation\":\"clock.now\"", ",\"result\":7000000000}", 1),
            (
                "\"kind\":\"cap\",\"operation\":\"clock.system_time\"",
                ",\"result\":946684800000000000}",
                1,
            ),
        ] {
            assert_eq!(
                lines(operation, end).len(),
                count,
                "{operation} {end}\n{trace}"
            );
        }
        let panic = catch_unwind(AssertUnwindSafe(|| clock.monotonic_only().system_time()));
        assert_eq!(
            message(&*panic.unwrap_err()),
            Some(
                "`system_time()` called on a `Clock` from `monotonic_only()`, which cannot read \
                 wall time"
            )
        );
    });
}

fn two_sleepers(seed: u64) -> (Vec<u8>, String) {
    let simulation = Simulation::new(seed);
    let (tasks, clock) = (simulation.tasks(), simulation.clock());
    let order = &Mutex::new(Vec::new());
    let until = clock.now() + SECOND;
    tasks
        .scope::<_, TaskError, _>(|scope| {
            for id in 0..2 {
                scope.spawn(move || {
                    clock.sleep_until(until).unwrap();
                    order.lock().unwrap().push(id);
                    Ok(())
                });
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(simulation.tasks().sim_elapsed(), SECOND);
    (order.lock().unwrap().clone(), simulation.tasks().sim_trace())
}

#[test]
fn sleepers_of_one_instant_wake_in_seeded_order_and_replay() {
    bounded(|| {
        let orders: BTreeSet<_> = (0..20).map(|seed| two_sleepers(seed).0).collect();
        assert_eq!(orders.len(), 2, "{orders:?}");
        let (order, trace) = two_sleepers(7);
        assert_eq!((order, trace.clone()), two_sleepers(7));
        // Both timers fire in the one jump, before either sleeper runs.
        let advance = trace
            .lines()
            .position(|line| line.contains("\"operation\":\"advance\""));
        let after: Vec<_> = trace.lines().skip(advance.unwrap() + 1).take(2).collect();
        assert!(
            after
                .iter()
                .all(|line| line.contains("\"action\":\"wake\"")),
            "{trace}"
        );
        assert_eq!(
            trace.matches("\"operation\":\"advance\"").count(),
            1,
            "{trace}"
        );
    });
}

#[test]
fn cancellation_ends_a_sleep_at_once_and_takes_its_timer() {
    bounded(|| {
        for seed in 0..20 {
            for mode in ["sibling", "explicit", "body"] {
                let simulation = Simulation::new(seed);
                let (tasks, clock) = (simulation.tasks(), simulation.clock());
                let start = clock.now();
                let result = tasks.scope::<(), Ended, _>(|scope| {
                    scope.spawn(|| {
                        // Woken by the cancel at one second, not by its own timer at five.
                        assert_eq!(clock.sleep(5 * SECOND), Err(Cancelled));
                        assert_eq!(clock.now(), start + SECOND);
                        // Level-triggered: later sleeps fail at once, whatever their target.
                        assert_eq!(clock.sleep(Duration::MAX), Err(Cancelled));
                        assert_eq!(clock.sleep(Duration::ZERO), Err(Cancelled));
                        assert_eq!(clock.sleep_until(start), Err(Cancelled));
                        Ok(())
                    });
                    if mode == "sibling" {
                        scope.spawn(|| {
                            clock.sleep(SECOND).unwrap();
                            Err::<(), _>(Ended::Child)
                        });
                        return Ok(());
                    }
                    clock.sleep(SECOND).unwrap();
                    if mode == "body" {
                        return Err(Ended::Body);
                    }
                    scope.cancel();
                    assert_eq!(clock.sleep(5 * SECOND), Err(Cancelled));
                    Ok(())
                });
                let expected = match mode {
                    "sibling" => Err(Ended::Child),
                    "body" => Err(Ended::Body),
                    _ => Ok(()),
                };
                assert_eq!(result, expected);
                assert_eq!(simulation.tasks().sim_elapsed(), SECOND);
                // The cancelled sleeper's timer is gone: the next jump is this sleep's own.
                clock.sleep(2 * SECOND).unwrap();
                assert_eq!(simulation.tasks().sim_elapsed(), 3 * SECOND);
                let trace = simulation.tasks().sim_trace();
                assert!(!trace.contains("\"to\":5000000000"), "{trace}");
                assert!(trace.contains(",\"result\":\"cancelled\"}"), "{trace}");
            }
        }
    });
}

#[test]
fn polling_while_a_timer_waits_fails_the_seed_with_the_stall_report() {
    bounded(|| {
        let simulation = Simulation::new(0);
        let (tasks, clock) = (simulation.tasks(), simulation.clock());
        let runtime = tasks.simulation.as_ref().unwrap();
        runtime
            .lock()
            .unwrap()
            .baton
            .as_mut()
            .unwrap()
            .scheduler
            .stall_limit = 1000;
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), TaskError> = tasks.scope(|scope| {
                let deadline = clock.now() + SECOND;
                scope.spawn(move || Ok(clock.sleep_until(deadline)?));
                // Polling keeps this task runnable, so the deadline never comes.
                while clock.now() < deadline {
                    tasks.yield_now()?;
                }
                Ok(())
            });
        }))
        .unwrap_err();
        let report = message(&*panic).unwrap();
        assert!(
            report.starts_with(
                "simulation stalled at virtual time 0ns: 1000 scheduling points passed while a \
                 timer waited."
            ),
            "{report}"
        );
        assert!(
            report.contains("\n  task 0 runnable in ") && report.contains(file!()),
            "{report}"
        );
        assert_eq!(simulation.tasks().sim_elapsed(), Duration::ZERO);
        // Without a pending timer the same loop is no stall.
        for _ in 0..3000 {
            tasks.yield_now().unwrap();
        }
    });
}
