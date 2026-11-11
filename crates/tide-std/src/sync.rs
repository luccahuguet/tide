use crate::marker::Pure;
use crate::task::{SimCell, Tasks};
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc as RustArc, Condvar, Mutex, PoisonError, TryLockError};

/// A shared cell. Cloning the handle keeps the same cell.
pub struct Shared<T>(RustArc<SharedCell<T>>);

struct SharedCell<T> {
    simulation: Option<SimCell>,
    next: AtomicUsize,
    state: Mutex<State<T>>,
    ready: Condvar,
}

struct State<T> {
    serving: usize,
    poisoned: bool,
    value: T,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Send + Pure> Shared<T> {
    /// Creates a cell using the caller's task authority.
    #[track_caller]
    pub fn new(v: T, tasks: &Tasks) -> Shared<T> {
        Self(RustArc::new(SharedCell {
            simulation: tasks.sim_cell(std::panic::Location::caller()),
            next: AtomicUsize::new(0),
            state: Mutex::new(State {
                serving: 0,
                poisoned: false,
                value: v,
            }),
            ready: Condvar::new(),
        }))
    }

    /// Runs `f` with exclusive access, in FIFO acquisition order, even in a cancelled scope.
    /// A panic propagates and poisons the cell. Later calls skip `f` and return `Poisoned`.
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError> {
        let cell = &self.0;
        let span = std::panic::Location::caller();
        let mut state = if let Some(simulation) = &cell.simulation {
            // A Pure callback cannot suspend. Keep the whole ticket lifetime under one baton.
            simulation.acquire(span);
            let state = match cell.state.try_lock() {
                Ok(state) => state,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => panic!("contended cell under the simulator"),
            };
            let ticket = cell.next.fetch_add(1, Ordering::Relaxed);
            assert_eq!(state.serving, ticket, "contended cell under the simulator");
            state
        } else {
            crate::task::outside_simulation();
            // Take a ticket before waiting for the mutex so new arrivals cannot overtake waiters.
            // Multi-cell operations (fx-ugm) reserve each ticket after acquiring the previous
            // cell in ID order.
            let ticket = cell.next.fetch_add(1, Ordering::Relaxed);
            let mut state = cell.state.lock().unwrap_or_else(PoisonError::into_inner);
            while state.serving != ticket {
                state = cell
                    .ready
                    .wait(state)
                    .unwrap_or_else(PoisonError::into_inner);
            }
            state
        };
        let outcome = if state.poisoned {
            Ok(Err(LockError::Poisoned))
        } else {
            let outcome = catch_unwind(AssertUnwindSafe(|| f(&mut state.value)));
            state.poisoned = outcome.is_err();
            outcome.map(Ok)
        };
        state.serving = state.serving.wrapping_add(1);
        drop(state);
        // Any waiter can wake first, but only the next ticket can proceed.
        cell.ready.notify_all();
        if let Some(simulation) = &cell.simulation {
            simulation.release(span);
        }
        match outcome {
            Ok(result) => result,
            Err(panic) => resume_unwind(panic),
        }
    }
}

/// Failure to access a shared cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockError {
    /// Multiple operands name the same cell. Only the planned multi-cell operations return
    /// this error (fx-ugm).
    Aliased,
    /// A previous callback panicked while holding the cell.
    Poisoned,
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aliased => f.write_str("shared cell appears more than once"),
            Self::Poisoned => f.write_str("shared cell poisoned"),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskError;
    use std::panic::panic_any;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    const TIMEOUT: Duration = Duration::from_secs(20);

    fn wait_until(ready: impl Fn() -> bool, failure: &str) {
        let deadline = Instant::now() + TIMEOUT;
        while !ready() {
            assert!(Instant::now() < deadline, "{failure}");
            thread::yield_now();
        }
    }

    #[test]
    fn shared_serializes_cloned_handles_in_fifo_order() {
        let tasks = Tasks::real();
        let order = Shared::new(Vec::new(), &tasks);
        let result: Result<(), TaskError> = tasks.scope(|scope| {
            let (entered, in_cell) = mpsc::channel();
            let (release, held) = mpsc::channel();
            drop(scope.spawn(|| {
                order
                    .with(move |xs| {
                        xs.push(0);
                        entered.send(()).unwrap();
                        let _ = held.recv();
                    })
                    .unwrap();
                Ok(())
            }));
            in_cell.recv_timeout(TIMEOUT).unwrap();
            for n in 1..=3 {
                let handle = order.clone();
                drop(scope.spawn(move || {
                    handle.with(|xs| xs.push(n)).unwrap();
                    Ok(())
                }));
                // Establish arrival order while the first callback still holds the cell.
                wait_until(
                    || order.0.next.load(Ordering::Relaxed) > n,
                    "waiter did not take a ticket",
                );
            }
            release.send(()).unwrap();
            Ok(())
        });
        result.unwrap();
        assert_eq!(order.with(|xs| xs.clone()), Ok(vec![0, 1, 2, 3]));
    }

    #[test]
    fn shared_propagates_panics_and_wakes_poisoned_waiters() {
        const WAITERS: usize = 3;
        let tasks = Tasks::real();
        let cell = Shared::new(0, &tasks);
        let (release, held) = mpsc::channel();
        let (completed, results) = mpsc::channel();
        let scoped_cell = cell.clone();
        // Keep the test's deadline outside the scope, which must join even stuck children.
        let run = thread::spawn(move || {
            let _: Result<(), TaskError> = tasks.scope(move |scope| {
                let (entered, in_cell) = mpsc::channel();
                let holder = scoped_cell.clone();
                drop(scope.spawn(move || {
                    holder
                        .with(move |value| {
                            *value = 1;
                            entered.send(()).unwrap();
                            held.recv().unwrap();
                            panic_any(73u32);
                        })
                        .unwrap();
                    Ok(())
                }));
                in_cell.recv_timeout(TIMEOUT).unwrap();
                for _ in 0..WAITERS {
                    let handle = scoped_cell.clone();
                    let completed = completed.clone();
                    drop(scope.spawn(move || {
                        let mut ran = false;
                        let result = handle.with(|_| ran = true);
                        completed.send((result, ran)).unwrap();
                        Ok(())
                    }));
                }
                Ok(())
            });
        });
        wait_until(
            || cell.0.next.load(Ordering::Relaxed) == WAITERS + 1,
            "queued callers did not reserve their tickets",
        );
        release.send(()).unwrap();
        for _ in 0..WAITERS {
            assert_eq!(
                results
                    .recv_timeout(TIMEOUT)
                    .expect("queued caller did not finish"),
                (Err(LockError::Poisoned), false)
            );
        }
        wait_until(|| run.is_finished(), "panicking scope did not finish");
        let panic = run.join().unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 73);
    }
}
