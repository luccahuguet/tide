use super::simulation_tests::bounded;
use super::*;
use crate::time::{Clock, Duration};
use std::sync::atomic::AtomicUsize;

fn clock() -> Clock {
    Clock::new(Time::real())
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    Child,
    Body,
    Task(TaskError),
}
impl From<TaskError> for Error {
    fn from(error: TaskError) -> Self {
        Self::Task(error)
    }
}

#[test]
fn scope_deadlines_preserve_partial_ok_and_rewrite_only_own_child_errors() {
    bounded(|| {
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            let (tasks, clock) = (simulation.tasks(), simulation.clock());
            for partial in [false, true] {
                let at = clock.now() + Duration::from_secs(5);
                let result = tasks.scope::<_, Error, _>(|s| {
                    s.set_deadline(at);
                    s.set_deadline(at + Duration::from_secs(9));
                    let child = s.spawn(|| {
                        assert_eq!(clock.sleep_until(at), Err(Cancelled));
                        assert_eq!(clock.now(), at);
                        assert_eq!(tasks.checkpoint(), Err(Cancelled));
                        if partial { Ok(17) } else { Err(Error::Child) }
                    });
                    if partial { Ok(child.join()?) } else { Ok(0) }
                });
                assert_eq!(
                    result,
                    if partial {
                        Ok(17)
                    } else {
                        Err(Error::Task(TaskError::DeadlineExceeded))
                    }
                );
                assert_eq!(clock.now(), at);
            }
            let at = clock.now() + Duration::from_secs(1);
            let result = tasks.scope::<(), Error, _>(|outer| {
                outer.set_deadline(at);
                outer.spawn(|| {
                    let nested = tasks.scope::<(), Error, _>(|inner| {
                        inner.set_deadline(at);
                        inner.spawn(|| {
                            assert_eq!(clock.sleep_until(at), Err(Cancelled));
                            Err::<(), _>(Error::Child)
                        });
                        Ok(())
                    });
                    // The ancestor's tied deadline fired first, so the inner error is intact.
                    assert_eq!(nested, Err(Error::Child));
                    nested
                });
                Ok(())
            });
            assert_eq!(result, Err(Error::Task(TaskError::DeadlineExceeded)));
            let result = tasks.scope::<(), Error, _>(|s| {
                s.set_deadline(clock.now());
                assert_eq!(tasks.checkpoint(), Err(Cancelled));
                Err(Error::Body)
            });
            assert_eq!(result, Err(Error::Body));
            tasks.checkpoint().unwrap();
        }
    });
}

#[test]
fn deadline_lowering_scope_exit_and_earlier_failure_remove_stale_timers() {
    bounded(|| {
        let simulation = Simulation::new(0);
        let (tasks, clock) = (simulation.tasks(), simulation.clock());
        let start = clock.now();
        tasks
            .scope::<_, TaskError, _>(|s| {
                s.set_deadline(start + Duration::from_secs(5));
                s.set_deadline(start + Duration::from_secs(2));
                s.spawn(|| {
                    assert!(clock.sleep(Duration::MAX).is_err());
                    Ok(())
                });
                Ok(())
            })
            .unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), Duration::from_secs(2));
        tasks
            .scope::<_, TaskError, _>(|s| {
                s.set_deadline(start + Duration::from_secs(5));
                Ok(())
            })
            .unwrap();
        let failed = tasks.scope::<(), Error, _>(|s| {
            s.set_deadline(start + Duration::from_secs(5));
            s.spawn(|| Err::<(), _>(Error::Child));
            Ok(())
        });
        assert_eq!(failed, Err(Error::Child));
        clock.sleep(Duration::from_secs(4)).unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), Duration::from_secs(6));
        let trace = simulation.tasks().sim_trace();
        assert!(!trace.contains("\"to\":5000000000"), "{trace}");
        assert!(trace.contains("\"operation\":\"set_deadline\""));
        assert!(trace.contains("\"operation\":\"deadline\""));
    });
}

#[test]
fn real_deadlines_bound_waits_and_win_over_late_independent_errors() {
    bounded(|| {
        let (tasks, clock) = (Tasks::real(), clock());
        let start = std::time::Instant::now();
        let result = tasks.scope::<_, Error, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_millis(20));
            s.spawn(|| {
                assert_eq!(clock.sleep(Duration::from_secs(30)), Err(Cancelled));
                Err::<(), _>(Error::Child)
            });
            Ok(())
        });
        assert_eq!(result, Err(Error::Task(TaskError::DeadlineExceeded)));
        assert!(start.elapsed() < Duration::from_secs(5));
        let result = tasks.scope::<(), Error, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_millis(5));
            s.spawn(|| {
                thread::sleep(Duration::from_millis(20));
                Err::<(), _>(Error::Child)
            });
            Ok(())
        });
        assert_eq!(result, Err(Error::Task(TaskError::DeadlineExceeded)));
    });
}

fn isolated_borrowed_type<'a>(tasks: &Tasks, _: &'a str) -> PhantomData<&'a str> {
    tasks.spawn_isolated(|| PhantomData).unwrap()
}

fn shields(tasks: &Tasks, clock: &Clock) {
    tasks
        .scope::<_, TaskError, _>(|outer| {
            outer.cancel();
            assert_eq!(tasks.checkpoint(), Err(Cancelled));
            tasks.shield(|| {
                tasks.checkpoint().unwrap();
                let at = clock.now() + Duration::from_millis(1);
                tasks
                    .scope::<_, TaskError, _>(|inner| {
                        inner.set_deadline(at);
                        assert_eq!(clock.sleep_until(at), Err(Cancelled));
                        Ok(())
                    })
                    .unwrap();
                tasks.checkpoint().unwrap();
            });
            assert_eq!(tasks.checkpoint(), Err(Cancelled));
            Ok(())
        })
        .unwrap();
    let result = tasks.scope::<(), Error, _>(|outer| {
        let first = outer.spawn(move || {
            tasks.shield(|| {
                // A failed join inside a shield still observes the outer failure for echoes.
                let failing = outer.spawn(|| Err::<(), _>(Error::Child));
                assert!(failing.join().is_err());
                tasks.checkpoint().unwrap();
            });
            Err::<(), _>(Error::Body)
        });
        assert!(first.join().is_err());
        Ok(())
    });
    assert_eq!(result, Err(Error::Child));
}

#[test]
fn shields_hide_outer_causes_keep_inner_deadlines_and_failed_join_echoes() {
    bounded(|| {
        shields(&Tasks::real(), &clock());
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            shields(simulation.tasks(), simulation.clock());
        }
    });
}

fn limits(tasks: &Tasks, clock: &Clock) {
    let limited = tasks.limit(2);
    let live = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    limited
        .scope::<_, TaskError, _>(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    let n = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(n, Ordering::SeqCst);
                    assert!(n <= 2);
                    clock.sleep(Duration::from_millis(1)).unwrap();
                    live.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                });
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert!(peak.load(Ordering::SeqCst) >= 1);
    let nested = limited.limit(1);
    nested
        .scope::<_, TaskError, _>(|s| {
            for _ in 0..5 {
                s.spawn(|| {
                    assert_eq!(live.fetch_add(1, Ordering::SeqCst), 0);
                    clock.sleep(Duration::from_millis(1)).unwrap();
                    live.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                });
            }
            Ok(())
        })
        .unwrap();
    let never = &AtomicBool::new(false);
    let one = tasks.limit(1);
    let result = one.scope::<(), TaskError, _>(|s| {
        s.spawn(move || {
            // Holding the only slot, this child tries to spawn again. The deadline frees it.
            s.set_deadline(clock.now() + Duration::from_millis(1));
            let not_started = s.spawn(|| {
                never.store(true, Ordering::SeqCst);
                Ok(())
            });
            assert_eq!(not_started.join(), Err(TaskError::Cancelled));
            assert_eq!(tasks.checkpoint(), Err(Cancelled));
            Ok(())
        });
        Ok(())
    });
    assert_eq!(result, Err(TaskError::DeadlineExceeded));
    assert!(!never.load(Ordering::SeqCst));
    one.scope::<_, TaskError, _>(|s| {
        assert_eq!(s.spawn(|| Ok(27)).join()?, 27);
        Ok(())
    })
    .unwrap();
    one.scope::<_, TaskError, _>(|s| {
        s.cancel();
        assert_eq!(s.spawn(|| Ok(())).join(), Err(TaskError::Cancelled));
        Ok(())
    })
    .unwrap_err();
    tasks.limit(0).scope::<_, TaskError, _>(|_| Ok(())).unwrap();
}

#[test]
fn limits_bound_children_and_cancel_slot_waits_on_both_runtimes() {
    bounded(|| {
        limits(&Tasks::real(), &clock());
        for seed in 0..20 {
            let simulation = Simulation::new(seed);
            limits(simulation.tasks(), simulation.clock());
        }
    });
}

#[test]
fn zero_limit_spawn_panics_even_when_cancelled_or_shielded() {
    bounded(|| {
        for simulated in [false, true] {
            let simulation = simulated.then(|| Simulation::new(0));
            let tasks = simulation
                .as_ref()
                .map_or_else(Tasks::real, |s| s.tasks().clone())
                .limit(0);
            for mode in 0..3 {
                let panic = catch_unwind(AssertUnwindSafe(|| {
                    let _ = tasks.scope::<_, TaskError, _>(|s| {
                        if mode == 1 {
                            s.cancel();
                        }
                        if mode == 2 {
                            tasks.shield(|| s.spawn(|| Ok(())));
                        } else {
                            s.spawn(|| Ok(()));
                        }
                        Ok(())
                    });
                }));
                assert!(panic.is_err());
            }
        }
    });
}

#[test]
fn count_tickets_are_fifo_reserve_slots_and_take_all_counts_or_none() {
    let outer = Limit::new(0, 2);
    let inner = Limit::new(1, 1);
    let first = Slots::queue(&[outer.clone(), inner.clone()]);
    let waiting = Slots::queue(&[outer.clone(), inner.clone()]);
    assert!(!waiting.signal().ready());
    assert_eq!((outer.live(), inner.live()), (1, 1));
    // Waiting has not taken the otherwise free outer slot; fairness keeps its ticket first.
    let outer_only = Slots::queue(&[outer.clone()]);
    assert!(!outer_only.signal().ready());
    drop(first);
    assert!(waiting.signal().ready());
    assert!(outer_only.signal().ready());
    let third = Slots::queue(&[outer.clone()]);
    let fourth = Slots::queue(&[outer]);
    drop(waiting);
    assert!(third.signal().ready());
    assert!(!fourth.signal().ready());
    // A new arrival cannot steal third's reserved slot before it runs.
    drop(third);
    assert!(fourth.signal().ready());
    drop(outer_only);

    let count = Limit::new(2, 1);
    let owner = Slots::queue(&[count.clone()]);
    let cancelled = Slots::queue(&[count.clone()]);
    let next = Slots::queue(&[count.clone()]);
    drop(cancelled);
    assert!(!next.signal().ready());
    drop(owner);
    assert!(next.signal().ready());
    let handed_off = Slots::queue(&[count.clone()]);
    let after = Slots::queue(&[count]);
    drop(next);
    assert!(handed_off.signal().ready() && !after.signal().ready());
    drop(handed_off);
    assert!(after.signal().ready());
}

#[test]
fn limits_wake_waiters_in_other_real_runtimes_and_cancelled_tickets_pass_on() {
    bounded(|| {
        let tasks = Tasks::real().limit(1);
        let (started, running) = mpsc::channel();
        let (release, finish) = mpsc::channel();
        let first_tasks = tasks.clone();
        let first = thread::spawn(move || {
            first_tasks.scope::<_, TaskError, _>(|s| {
                s.spawn(move || {
                    started.send(()).unwrap();
                    finish.recv().unwrap();
                    Ok(())
                });
                Ok(())
            })
        });
        running.recv_timeout(Duration::from_secs(2)).unwrap();
        let other_tasks = tasks.clone();
        let second = thread::spawn(move || {
            other_tasks.scope::<_, TaskError, _>(|s| s.spawn(|| Ok(91)).join())
        });
        // Wait for the actual ticket rather than assuming thread startup ordering.
        let until = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            // The independent runtime is blocked while only the first child owns the count.
            if tasks.limits[0].queued() > 0 {
                break;
            }
            assert!(std::time::Instant::now() < until);
            thread::yield_now();
        }
        release.send(()).unwrap();
        assert_eq!(first.join().unwrap(), Ok(()));
        assert_eq!(second.join().unwrap(), Ok(91));
    });
}

#[test]
fn isolation_is_a_new_task_contains_payloads_and_inherits_echo_state() {
    bounded(|| {
        for simulated in [false, true] {
            for seed in 0..10 {
                let simulation = simulated.then(|| Simulation::new(seed));
                let tasks = simulation
                    .as_ref()
                    .map_or_else(Tasks::real, |s| s.tasks().clone());
                let local = String::from("local lifetime");
                let _ = isolated_borrowed_type(&tasks, &local);
                assert_ne!(
                    tasks.spawn_isolated(|| thread::current().id()).unwrap(),
                    thread::current().id()
                );
                for mode in 0..3 {
                    let error = tasks
                        .spawn_isolated(move || match mode {
                            0 => panic!("isolated str"),
                            1 => std::panic::panic_any(String::from("isolated String")),
                            _ => std::panic::panic_any(13u32),
                        })
                        .unwrap_err();
                    assert_eq!(
                        error.message,
                        ["isolated str", "isolated String", "Box<dyn Any>"][mode]
                    );
                }
                let cell = crate::sync::Shared::new(0, &tasks);
                let held = cell.clone();
                tasks
                    .scope::<_, TaskError, _>(|_| {
                        assert_eq!(
                            tasks
                                .spawn_isolated(move || {
                                    held.with(|_| panic!("isolated cell panic")).unwrap();
                                })
                                .unwrap_err()
                                .message,
                            "isolated cell panic"
                        );
                        tasks.checkpoint().unwrap();
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(cell.with(|_| ()), Err(crate::sync::LockError::Poisoned));
                struct Payload(Tasks, Arc<AtomicBool>);
                impl Drop for Payload {
                    fn drop(&mut self) {
                        self.0.checkpoint().unwrap();
                        self.1.store(true, Ordering::SeqCst);
                    }
                }
                let destroyed = Arc::new(AtomicBool::new(false));
                let payload = Payload(tasks.clone(), destroyed.clone());
                assert_eq!(
                    tasks
                        .spawn_isolated(move || std::panic::panic_any(payload))
                        .unwrap_err()
                        .message,
                    "Box<dyn Any>"
                );
                assert!(destroyed.load(Ordering::SeqCst));
                let limited = tasks.limit(1);
                limited
                    .scope::<_, TaskError, _>(|s| {
                        s.spawn(|| {
                            assert_eq!(tasks.spawn_isolated(|| 8).unwrap(), 8);
                            Ok(())
                        })
                        .join()
                    })
                    .unwrap();
                tasks
                    .scope::<_, TaskError, _>(|s| {
                        s.cancel();
                        let observer = tasks.clone();
                        assert!(
                            tasks
                                .spawn_isolated(move || observer.checkpoint().is_err())
                                .unwrap()
                        );
                        Ok(())
                    })
                    .unwrap();
                let tasks = &tasks;
                let result = tasks.scope::<(), Error, _>(|s| {
                    s.spawn(move || {
                        let failing = s.spawn(|| Err::<(), _>(Error::Child));
                        assert!(failing.join().is_err());
                        let observer = tasks.clone();
                        tasks
                            .spawn_isolated(move || {
                                assert_eq!(observer.checkpoint(), Err(Cancelled));
                                let nested = observer.clone();
                                observer
                                    .spawn_isolated(move || assert!(nested.checkpoint().is_err()))
                                    .unwrap();
                            })
                            .unwrap();
                        Err::<(), _>(Error::Body)
                    });
                    Ok(())
                });
                assert_eq!(result, Err(Error::Child));
                tasks.checkpoint().unwrap();
                if let Some(simulation) = &simulation {
                    let trace = simulation.tasks().sim_trace();
                    assert!(trace.contains("\"isolated\":true,\"result\":\"panicked\""));
                }
            }
        }
    });
}

#[test]
fn scope_deadlines_alone_do_not_trigger_the_operation_timer_stall_guard() {
    bounded(|| {
        let simulation = Simulation::new(0);
        let (tasks, clock) = (simulation.tasks(), simulation.clock());
        tasks
            .simulation
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .baton
            .as_mut()
            .unwrap()
            .scheduler
            .stall_limit = 5;
        tasks
            .scope::<_, TaskError, _>(|s| {
                s.set_deadline(clock.now() + Duration::from_secs(1));
                for _ in 0..20 {
                    tasks.checkpoint().unwrap();
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(simulation.tasks().sim_elapsed(), Duration::ZERO);
    });
}

#[cfg(unix)]
#[test]
fn recursive_limit_deadlock_reports_all_blocked_tasks_and_spans() {
    const ROLE: &str = "TIDE_LIMIT_DEADLOCK_TEST";
    const NAME: &str = "recursive_limit_deadlock_reports_all_blocked_tasks_and_spans";
    if std::env::var_os(ROLE).is_some() {
        let simulation = Simulation::new(0);
        let tasks = simulation.tasks().limit(1);
        let _ = tasks.scope::<_, TaskError, _>(|s| {
            s.spawn(move || {
                s.spawn(|| Ok(()));
                Ok(())
            });
            // Both tasks wait on the same count; no join/scope-exit obscures the report.
            s.spawn(|| Ok(()));
            Ok(())
        });
        unreachable!();
    }
    let test = format!("task::control_tests::{NAME}");
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env(ROLE, "1")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&child.stderr);
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&child.status),
        Some(6),
        "{report}"
    );
    assert!(
        report.contains("task 0 blocked in limit(0) at "),
        "{report}"
    );
    assert!(
        report.contains("task 1 blocked in limit(0) at "),
        "{report}"
    );
    assert!(report.contains(file!()), "{report}");
    let last = report.lines().last().unwrap();
    assert!(last.contains("\"operation\":\"deadlock\""), "{report}");
    assert!(
        last.contains("\"blocked\":[{\"task\":0,\"operation\":\"limit\""),
        "{report}"
    );
    assert!(
        last.contains("{\"task\":1,\"operation\":\"limit\""),
        "{report}"
    );
}

#[cfg(unix)]
#[test]
fn isolated_payload_drop_panic_aborts_without_dropping_the_second_payload() {
    const ROLE: &str = "TIDE_ISOLATED_PAYLOAD_DROP_TEST";
    const NAME: &str = "isolated_payload_drop_panic_aborts_without_dropping_the_second_payload";
    if let Ok(mode) = std::env::var(ROLE) {
        struct First;
        struct Second;
        impl Drop for First {
            fn drop(&mut self) {
                std::panic::panic_any(Second);
            }
        }
        impl Drop for Second {
            fn drop(&mut self) {
                // Distinguish dropping this payload from the required abort without hanging.
                std::process::exit(99);
            }
        }
        let simulation = (mode == "simulated").then(|| Simulation::new(0));
        let tasks = simulation
            .as_ref()
            .map_or_else(Tasks::real, |s| s.tasks().clone());
        let _ = tasks.spawn_isolated(|| std::panic::panic_any(First));
        unreachable!();
    }
    let test = format!("task::control_tests::{NAME}");
    for mode in ["real", "simulated"] {
        let child = std::process::Command::new("timeout")
            .arg("10")
            .arg(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--nocapture"])
            .env(ROLE, mode)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let report = String::from_utf8_lossy(&child.stderr);
        assert_eq!(
            std::os::unix::process::ExitStatusExt::signal(&child.status),
            Some(6),
            "{mode}: {report}"
        );
        assert!(
            report.contains("panic while dropping an isolated panic payload"),
            "{report}"
        );
    }
}

#[test]
fn isolated_entry_stall_drains_its_child_before_resuming_the_panic() {
    bounded(|| {
        let simulation = Simulation::new(0);
        let (tasks, clock) = (simulation.tasks(), simulation.clock());
        tasks
            .scope::<_, TaskError, _>(|s| {
                s.spawn(|| {
                    clock.sleep(Duration::from_secs(1)).unwrap();
                    Ok(())
                });
                let runtime = tasks.simulation.as_ref().unwrap();
                runtime
                    .lock()
                    .unwrap()
                    .baton
                    .as_mut()
                    .unwrap()
                    .scheduler
                    .stall_limit = 0;
                // Ensure an operation timer exists; this point does not check cancellation.
                let context = Context::current().unwrap();
                while !context
                    .lock()
                    .baton
                    .as_ref()
                    .unwrap()
                    .scheduler
                    .has_operation_timer()
                {
                    let _ = catch_unwind(AssertUnwindSafe(|| tasks.yield_now()));
                }
                let finished = Arc::new(AtomicBool::new(false));
                let flag = finished.clone();
                assert!(
                    catch_unwind(AssertUnwindSafe(|| tasks.spawn_isolated(move || {
                        flag.store(true, Ordering::SeqCst);
                    })))
                    .is_err()
                );
                assert!(finished.load(Ordering::SeqCst));
                runtime
                    .lock()
                    .unwrap()
                    .baton
                    .as_mut()
                    .unwrap()
                    .scheduler
                    .stall_limit = 1_000_000;
                Ok(())
            })
            .unwrap();
        tasks.checkpoint().unwrap();
    });
}

#[test]
fn isolated_checkpoints_alone_mark_the_waiting_callers_echo() {
    bounded(|| {
        for simulated in [false, true] {
            let simulation = simulated.then(|| Simulation::new(7));
            let tasks = &simulation
                .as_ref()
                .map_or_else(Tasks::real, |s| s.tasks().clone());
            let result = tasks.scope::<(), Error, _>(|s| {
                s.spawn(move || {
                    s.spawn(|| Err::<(), _>(Error::Child));
                    let context = Context::current().unwrap();
                    let at = context.at.unwrap();
                    // Let the failure finish without observing cancellation or a failed join.
                    while !context.lock().cancelled(at) {
                        if context.task.is_some() {
                            context.point(Operation::Yield, std::panic::Location::caller());
                        } else {
                            thread::yield_now();
                        }
                    }
                    assert!(!context.lock().scopes[&at.scope].children[at.child.unwrap()].echo);
                    let observer = tasks.clone();
                    tasks
                        .spawn_isolated(move || {
                            assert_eq!(observer.checkpoint(), Err(Cancelled));
                        })
                        .unwrap();
                    assert!(context.lock().scopes[&at.scope].children[at.child.unwrap()].echo);
                    Err::<(), _>(Error::Body)
                });
                Ok(())
            });
            assert_eq!(result, Err(Error::Child));
        }
    });
}

#[test]
fn cancelled_nested_callers_do_not_fail_the_uncancelled_scope_they_spawn_into() {
    bounded(|| {
        for simulated in [false, true] {
            let simulation = simulated.then(|| Simulation::new(5));
            let tasks = &simulation
                .as_ref()
                .map_or_else(Tasks::real, |s| s.tasks().clone())
                .limit(1);
            tasks
                .scope::<_, TaskError, _>(|outer| {
                    tasks.scope::<_, TaskError, _>(|inner| {
                        inner.cancel();
                        assert_eq!(
                            outer
                                .spawn(|| -> Result<(), TaskError> { panic!("must not start") })
                                .join(),
                            Err(TaskError::Cancelled)
                        );
                        Ok(())
                    })?;
                    tasks.checkpoint().unwrap();
                    outer.spawn(|| Ok(())).join()
                })
                .unwrap();
        }
    });
}

#[test]
fn scope_state_uses_its_owner_when_called_from_other_real_contexts() {
    bounded(|| {
        let (tasks, clock) = (&Tasks::real().limit(1), &clock());
        tasks
            .scope::<_, TaskError, _>(|outer| {
                thread::scope(|native| {
                    native
                        .spawn(move || {
                            // A native caller has no Tide context; it can still use a scoped handle.
                            assert_eq!(outer.spawn(|| Ok(3)).join(), Ok(3));
                            tasks
                                .scope::<_, TaskError, _>(|inner| {
                                    inner.cancel();
                                    // Caller cancellation must not register the synthetic child in inner.
                                    assert_eq!(
                                        outer
                                            .spawn(|| -> Result<(), TaskError> {
                                                panic!("must not start")
                                            })
                                            .join(),
                                        Err(TaskError::Cancelled)
                                    );
                                    outer.set_deadline(clock.now());
                                    Ok(())
                                })
                                .unwrap();
                        })
                        .join()
                        .unwrap();
                });
                assert_eq!(tasks.checkpoint(), Err(Cancelled));
                Ok(())
            })
            .unwrap();
        tasks
            .scope::<_, TaskError, _>(|s| {
                thread::scope(|native| native.spawn(move || s.cancel()).join().unwrap());
                assert_eq!(tasks.checkpoint(), Err(Cancelled));
                Ok(())
            })
            .unwrap();
    });
}
