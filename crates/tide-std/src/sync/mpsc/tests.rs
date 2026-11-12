use super::*;
use crate::task::{Simulation, TaskError};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant as HostInstant};

fn await_state<T>(channel: &Chan<T>, ready: impl Fn(&State<T>) -> bool) {
    let end = HostInstant::now() + Duration::from_secs(5);
    while !ready(&channel.state.lock().unwrap()) {
        assert!(HostInstant::now() < end, "channel waiter did not register");
        thread::yield_now();
    }
}

#[test]
fn blocked_senders_are_fifo_and_cannot_be_barged_on_either_capacity() {
    for bound in [0, 1] {
        let tasks = Tasks::real();
        let (tx, rx) = sync_channel(bound, &tasks);
        if bound == 1 {
            tx.send(0).unwrap();
        }
        let completed = AtomicUsize::new(0);
        thread::scope(|scope| {
            for value in 1..=3 {
                let tx = &tx;
                let completed = &completed;
                scope.spawn(move || {
                    tx.send(value).unwrap();
                    completed.fetch_add(1, Ordering::SeqCst);
                });
                await_state(&tx.0, |state| state.pending.len() == value);
            }
            assert_eq!(completed.load(Ordering::SeqCst), 0);
            assert_eq!(tx.try_send(4), Err(TrySendError::Full(4)));
            if bound == 1 {
                assert_eq!(rx.recv(), Ok(0));
            }
            for value in 1..=3 {
                assert_eq!(rx.recv(), Ok(value));
            }
        });
        assert_eq!(completed.load(Ordering::SeqCst), 3);
    }
}

#[test]
fn rendezvous_try_send_needs_one_waiting_receiver() {
    let tasks = Tasks::real();
    let (tx, rx) = sync_channel(0, &tasks);
    thread::scope(|scope| {
        let receiving = scope.spawn(move || rx.recv());
        await_state(&tx.0, |state| state.receiver.is_some());
        assert_eq!(tx.try_send(9), Ok(()));
        assert_eq!(receiving.join().unwrap(), Ok(9));
    });
    assert_eq!(tx.try_send(10), Err(TrySendError::Disconnected(10)));
}

#[test]
fn receiver_drop_returns_each_blocked_senders_value_across_real_runtimes() {
    let tasks = Tasks::real();
    let (tx, rx) = sync_channel(1, &tasks);
    tx.send(0).unwrap();
    thread::scope(|scope| {
        let first = scope.spawn(|| tx.send(1));
        await_state(&tx.0, |state| state.pending.len() == 1);
        let second = scope.spawn(|| tx.send(2));
        await_state(&tx.0, |state| state.pending.len() == 2);
        drop(rx);
        assert_eq!(first.join().unwrap(), Err(SendError::Disconnected(1)));
        assert_eq!(second.join().unwrap(), Err(SendError::Disconnected(2)));
    });
}

#[test]
fn cancellation_wakes_a_receiver_and_leaves_a_later_buffered_value() {
    let tasks = Tasks::real();
    let (tx, rx) = sync_channel(1, &tasks);
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.spawn(move || {
                await_state(&tx.0, |state| state.receiver.is_some());
                s.cancel();
                tx.send(7).unwrap_err(); // Bounded sends are cancelled even with room.
                tx.try_send(8).unwrap();
                Ok(())
            });
            assert_eq!(rx.recv(), Err(RecvError::Cancelled));
            Ok(())
        })
        .unwrap();
    assert_eq!(rx.try_recv(), Ok(8));
}

fn cancellation_and_handoff(tasks: &Tasks, clock: &crate::time::Clock) {
    let (tx, rx) = sync_channel(1, tasks);
    tx.send(0).unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.spawn(|| {
                tasks.scope::<_, TaskError, _>(|inner| {
                    inner.set_deadline(clock.now() + Duration::from_millis(1));
                    assert_eq!(tx.send(1), Err(SendError::Cancelled(1)));
                    Ok(())
                })
            });
            s.spawn(|| {
                tx.send(2).unwrap();
                Ok(())
            });
            clock.sleep(Duration::from_millis(2)).unwrap();
            assert_eq!(rx.recv(), Ok(0));
            assert_eq!(rx.recv(), Ok(2));
            Ok(())
        })
        .unwrap();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn cancelling_a_queued_sender_removes_only_its_value() {
    let tasks = Tasks::real();
    let (tx, rx) = sync_channel(1, &tasks);
    tx.send(0).unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.spawn(|| {
                assert_eq!(tx.send(1), Err(SendError::Cancelled(1)));
                Ok(())
            });
            await_state(&tx.0, |state| state.pending.len() == 1);
            s.cancel();
            Ok(())
        })
        .unwrap();
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    for seed in 0..8 {
        let sim = Simulation::new(seed);
        cancellation_and_handoff(sim.tasks(), sim.clock());
    }
}

#[derive(Debug, PartialEq)]
enum Failure {
    Origin,
    Independent,
    Task(TaskError),
}
impl From<TaskError> for Failure {
    fn from(e: TaskError) -> Self {
        Self::Task(e)
    }
}

#[test]
fn accepted_send_beats_cancellation_without_marking_an_echo() {
    for seed in 0..32 {
        let sim = Simulation::new(seed);
        let tasks = sim.tasks();
        let clock = sim.clock();
        let (tx, rx) = sync_channel(0, tasks);
        let result = tasks.scope::<_, Failure, _>(|s| {
            s.spawn(move || {
                // The other child consumes then cancels before this blocked send resumes.
                tx.send(1).unwrap();
                Err::<(), _>(Failure::Independent)
            });
            s.spawn(move || {
                clock.sleep(Duration::from_millis(1)).unwrap();
                assert_eq!(rx.try_recv(), Ok(1)); // No point after acceptance before failure.
                Err::<(), _>(Failure::Origin)
            });
            Ok(())
        });
        assert_eq!(result, Err(Failure::Independent));
    }
}

struct Reenter {
    sender: Option<Sender<Reenter>>,
    dropped: Arc<AtomicUsize>,
}
impl Drop for Reenter {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
        if let Some(tx) = &self.sender {
            assert!(matches!(
                tx.send(Self {
                    sender: None,
                    dropped: self.dropped.clone()
                }),
                Err(SendError::Disconnected(_))
            ));
        }
    }
}

#[test]
fn receiver_drops_payloads_after_both_locks_are_released_even_while_unwinding() {
    fn exercise(tasks: &Tasks) {
        let (tx, rx) = channel(tasks);
        let dropped = Arc::new(AtomicUsize::new(0));
        assert!(
            tx.send(Reenter {
                sender: Some(tx.clone()),
                dropped: dropped.clone()
            })
            .is_ok()
        );
        assert!(
            catch_unwind(AssertUnwindSafe(move || {
                let _rx = rx;
                panic!("drop while unwinding");
            }))
            .is_err()
        );
        assert_eq!(dropped.load(Ordering::SeqCst), 2);
    }
    exercise(&Tasks::real());
    let sim = Simulation::new(0);
    exercise(sim.tasks());
}

#[test]
fn channel_domain_checks_and_payload_free_replay() {
    let (real, _) = channel(&Tasks::real());
    let simulation = Simulation::new(6);
    assert!(catch_unwind(AssertUnwindSafe(|| real.send(0))).is_err());
    let (tx, rx) = channel(simulation.tasks());
    tx.send("payload must never appear").unwrap();
    assert_eq!(rx.recv(), Ok("payload must never appear"));
    let before = simulation.tasks().sim_trace();
    let copy = tx.clone();
    assert_eq!(simulation.tasks().sim_trace(), before); // Clone takes no point/event.
    drop(copy);
    let trace = simulation.tasks().sim_trace();
    assert!(!trace.contains("payload must never appear"));
    assert!(trace.contains("\"operation\":\"close\""));
    assert!(trace.contains("\"end\":\"sender\",\"senders\":1"));
    drop(simulation);
    let other = Simulation::new(6);
    assert!(catch_unwind(AssertUnwindSafe(|| tx.send("wrong domain"))).is_err());
    drop(other);
    // A destructor after the creating simulation ended is still safe.
    drop((tx, rx));
}

#[test]
fn zero_capacity_send_does_not_complete_on_registration_alone() {
    let tasks = Tasks::real();
    let (tx, rx) = sync_channel(0, &tasks);
    let completed = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            tx.send(7).unwrap();
            completed.store(true, Ordering::SeqCst);
        });
        await_state(&tx.0, |state| state.pending.len() == 1);
        assert!(!completed.load(Ordering::SeqCst));
        assert_eq!(rx.recv(), Ok(7));
    });
    assert!(completed.load(Ordering::SeqCst));
}

#[test]
fn send_errors_need_no_payload_debug_bound() {
    struct Opaque;
    fn error<E: std::error::Error>(e: E) {
        assert!(!e.to_string().is_empty());
    }
    error(SendError::Cancelled(Opaque));
    assert_eq!(
        format!("{:?}", SendError::Disconnected(Opaque)),
        "Disconnected(..)"
    );
}

#[test]
fn simulated_rendezvous_try_send_and_exact_timeout() {
    for seed in 0..8 {
        let sim = Simulation::new(seed);
        let tasks = sim.tasks();
        let clock = sim.clock();
        let (tx, rx) = sync_channel(0, tasks);
        tasks
            .scope::<_, TaskError, _>(|s| {
                let receiving = s.spawn(move || {
                    assert_eq!(rx.recv(), Ok(6));
                    Ok(())
                });
                clock.sleep(Duration::from_millis(1)).unwrap();
                tx.try_send(6).unwrap();
                receiving.join()
            })
            .unwrap();
        let (_tx, rx) = sync_channel::<u8>(1, tasks);
        let until = clock.now() + Duration::from_millis(3);
        assert_eq!(rx.recv_deadline(until), Err(RecvTimeoutError::Timeout));
        assert_eq!(clock.now(), until);
    }
}

#[test]
fn cancelled_waiting_receive_keeps_the_value_accepted_before_it_resumes() {
    for seed in 0..8 {
        let sim = Simulation::new(seed);
        let tasks = sim.tasks();
        let (tx, rx) = sync_channel(1, tasks);
        tasks
            .scope::<_, TaskError, _>(|s| {
                s.spawn(move || {
                    assert_eq!(rx.recv(), Err(RecvError::Cancelled));
                    assert_eq!(rx.try_recv(), Ok(7));
                    Ok(())
                });
                sim.clock().sleep(Duration::from_millis(1)).unwrap();
                tx.try_send(7).unwrap();
                s.cancel(); // No point between accepting the value and cancellation.
                Ok(())
            })
            .unwrap();
    }
}

fn cancelled_rendezvous_try_send(tasks: &Tasks, registered: impl FnOnce(&SyncSender<u8>)) -> bool {
    tasks
        .scope::<_, TaskError, _>(|s| {
            let (tx, rx) = sync_channel(0, tasks);
            let receiving = s.spawn(move || {
                let result = rx.recv();
                drop(rx);
                Ok(result)
            });
            registered(&tx);
            s.cancel();
            let sent = tx.try_send(7);
            let received = receiving.join()?;
            match (sent, received) {
                (Ok(()), Ok(7)) => Ok(true),
                (
                    Err(TrySendError::Full(7) | TrySendError::Disconnected(7)),
                    Err(RecvError::Cancelled),
                ) => Ok(false),
                outcomes => panic!("lost rendezvous value: {outcomes:?}"),
            }
        })
        .unwrap()
}

#[test]
fn cancelled_rendezvous_try_send_is_delivered_or_returns_its_value() {
    let mut accepted = 0;
    for seed in 0..32 {
        let sim = Simulation::new(seed);
        accepted += usize::from(cancelled_rendezvous_try_send(sim.tasks(), |_| {
            sim.clock().sleep(Duration::from_millis(1)).unwrap();
        }));
    }
    assert!(accepted > 0, "no cancelled receiver accepted a hand-off");
    for _ in 0..32 {
        cancelled_rendezvous_try_send(&Tasks::real(), |tx| {
            await_state(&tx.0, |state| state.receiver.is_some());
        });
    }
}

#[test]
fn accepted_rendezvous_receive_beats_cancellation_without_marking_an_echo() {
    for timed in [false, true] {
        for seed in 0..8 {
            let sim = Simulation::new(seed);
            let tasks = sim.tasks();
            let clock = sim.clock();
            let (tx, rx) = sync_channel(0, tasks);
            let result = tasks.scope::<_, Failure, _>(|s| {
                s.spawn(move || {
                    let received = if timed {
                        rx.recv_deadline(clock.now() + Duration::from_secs(1)).ok()
                    } else {
                        rx.recv().ok()
                    };
                    assert_eq!(received, Some(7));
                    drop(rx);
                    Err::<(), _>(Failure::Independent)
                });
                clock.sleep(Duration::from_millis(1)).unwrap();
                tx.try_send(7).unwrap();
                Err::<(), _>(Failure::Origin) // Cancels before the receiver resumes.
            });
            assert_eq!(result, Err(Failure::Independent));
        }
    }
}
