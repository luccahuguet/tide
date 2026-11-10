// Shared verbatim with the conformance fixture; keep later task APIs out of this slice.
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Mutex, mpsc};
use std::thread;

crate::capability!(
    /// Authority to spawn structured tasks on the thread runtime.
    Tasks
);

impl Tasks {
    /// Runs `body` and waits for all its children, including grandchildren and children whose
    /// handles were dropped or forgotten. Children may borrow state that outlives this call.
    /// A single child panic is re-raised after joining; a body panic continues after joining.
    ///
    /// This lifecycle slice does not implement cancellation or multi-failure selection (fx-nkv.3).
    pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
    where
        E: From<TaskError> + Send,
        F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>,
    {
        let failures = Mutex::new(Vec::<thread::Result<E>>::new());
        let result = thread::scope(|threads| {
            body(Scope {
                threads,
                failures: Failures(std::ptr::from_ref(&failures).cast()),
                cancelled: || E::from(TaskError::Cancelled),
                _marker: PhantomData,
            })
        });
        // shortcut: completion order selects errors, replace with TASK-2 arbitration in fx-nkv.3.
        let mut error = None;
        for failure in failures.into_inner().unwrap() {
            match failure {
                Err(panic) => resume_unwind(panic),
                Ok(e) if error.is_none() => error = Some(e),
                Ok(_) => {}
            }
        }
        match error {
            Some(e) => Err(e),
            None => result,
        }
    }
}

/// A copyable scope handle. Both lifetimes are invariant, as in Rust's scoped threads.
pub struct Scope<'s, 'env: 's, E> {
    threads: &'s (dyn Spawner<'s> + Sync),
    // A reference here would impose E: 's, which the public scope contract does not require.
    failures: Failures,
    cancelled: fn() -> E,
    _marker: PhantomData<(&'s mut &'s (), &'env mut &'env (), fn(E) -> E)>,
}

#[derive(Clone, Copy)]
struct Failures(*const ());

// SAFETY: only Tasks::scope constructs this pointer, with E: Send. Its mutex lives until the native
// scope has joined all children. The pointer is only dereferenced to &Mutex<Vec<thread::Result<E>>>,
// which is Sync because E: Send; the data is read and written only under that lock.
unsafe impl Send for Failures {}
// SAFETY: the same constructor and synchronization invariant permits shared access.
unsafe impl Sync for Failures {}

// Erase the native environment lifetime: local runtime state can be shorter-lived than &Tasks.
trait Spawner<'s> {
    fn spawn(&'s self, f: Box<dyn FnOnce() + Send + 's>);
}

impl<'s, 'env> Spawner<'s> for thread::Scope<'s, 'env> {
    fn spawn(&'s self, f: Box<dyn FnOnce() + Send + 's>) {
        // Native handles must not be forgettable by callers: a leaked handle stalls thread::scope.
        drop(thread::Scope::spawn(self, f));
    }
}

impl<E> Clone for Scope<'_, '_, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<E> Copy for Scope<'_, '_, E> {}

impl<'s, 'env, E> Scope<'s, 'env, E> {
    /// Starts a child that cannot outlive this scope. Dropping its handle does not detach it.
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
    where
        F: FnOnce() -> Result<U, E> + Send + 's,
    {
        let (tx, rx) = mpsc::sync_channel(1);
        // Erase E from the job's captures without adding E: 's to the public signature.
        let run_job: unsafe fn(F, mpsc::SyncSender<U>, Failures) = run::<F, U, E>;
        let failures = self.failures;
        self.threads.spawn(Box::new(move || {
            // SAFETY: Scope's invariant E matches the mutex allocated by Tasks::scope, and its
            // invariant 's keeps this job within the native scope, even on unwind.
            unsafe { run_job(f, tx, failures) };
        }));
        Task {
            result: Mutex::new(rx),
            cancelled: self.cancelled,
            _marker: PhantomData,
        }
    }
}

// Caller must supply the live scope mutex for this E; no reference to E escapes in the job type.
unsafe fn run<F, U, E>(f: F, tx: mpsc::SyncSender<U>, failures: Failures)
where
    F: FnOnce() -> Result<U, E>,
{
    let outcome = catch_unwind(AssertUnwindSafe(|| match f() {
        Ok(value) => {
            // A discarded result's destructor is still part of the child and may panic.
            let _ = tx.send(value);
            None
        }
        Err(error) => Some(error),
    }));
    let failure = match outcome {
        Ok(Some(error)) => Ok(error),
        Err(panic) => Err(panic),
        Ok(None) => return,
    };
    // SAFETY: the caller supplied the matching E, and scope exit joins before dropping this mutex.
    let failures = unsafe { &*failures.0.cast::<Mutex<Vec<thread::Result<E>>>>() };
    failures.lock().unwrap().push(failure);
}

/// A child's join handle; `Send + Sync` when `U, E: Send` (02 §8.1).
pub struct Task<'s, U, E> {
    result: Mutex<mpsc::Receiver<U>>,
    cancelled: fn() -> E,
    // Keep the scope origin invariant and E's auto traits equivalent to a synchronized result.
    _marker: PhantomData<(&'s mut &'s (), Mutex<E>)>,
}

impl<U, E> Task<'_, U, E> {
    /// Waits for the child. A failed or panicking child returns `Cancelled`; the scope retains
    /// the original cause even after this handle is consumed.
    pub fn join(self) -> Result<U, E> {
        self.result
            .into_inner()
            .unwrap()
            .recv()
            .map_err(|_| (self.cancelled)())
    }
}

/// Task lifecycle errors. Observable cancellation belongs to fx-nkv.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskError {
    Cancelled,
    /// No operation in this lifecycle slice returns this; deadline support belongs to fx-zc1.
    DeadlineExceeded,
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => f.write_str("task cancelled"),
            Self::DeadlineExceeded => f.write_str("task deadline exceeded"),
        }
    }
}
impl std::error::Error for TaskError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;

    #[test]
    fn task_errors_have_distinct_display_messages() {
        assert_eq!(TaskError::Cancelled.to_string(), "task cancelled");
        assert_eq!(
            TaskError::DeadlineExceeded.to_string(),
            "task deadline exceeded"
        );
    }

    #[test]
    fn task_borrows_results_and_non_send_local_values() {
        let tasks = Tasks(());
        let mut count = 0;
        let text = String::from("borrowed");
        let result: Result<_, TaskError> = tasks.scope(|s| {
            let child = s.spawn(|| {
                count += 1;
                let local = std::rc::Rc::new(7);
                assert_eq!(*local, 7);
                Ok(text.as_str())
            });
            Ok((child.join()?, std::rc::Rc::new(8)))
        });
        let (borrowed, local) = result.unwrap();
        assert_eq!(borrowed, "borrowed");
        assert_eq!(*local, 8);
        assert_eq!(count, 1);
    }

    struct Borrowed {
        finished: AtomicBool,
    }

    impl Drop for Borrowed {
        fn drop(&mut self) {
            assert!(
                *self.finished.get_mut(),
                "borrowed state dropped before child"
            );
        }
    }

    #[test]
    fn task_scope_joins_dropped_and_forgotten_handles_and_grandchildren() {
        let tasks = Tasks(());
        let borrowed = Borrowed {
            finished: AtomicBool::new(false),
        };
        let state = &borrowed;
        let (tx, rx) = mpsc::channel::<()>();
        let result: Result<_, TaskError> = tasks.scope(|s| {
            drop(s.spawn(move || {
                std::mem::forget(s.spawn(move || {
                    assert!(rx.recv().is_err());
                    state.finished.store(true, Ordering::SeqCst);
                    Ok(())
                }));
                Ok(())
            }));
            drop(tx);
            Ok(())
        });
        result.unwrap();
        assert!(borrowed.finished.load(Ordering::SeqCst));
    }

    #[test]
    fn task_child_panic_survives_join_drop_and_forget() {
        let tasks = Tasks(());
        for mode in 0..3 {
            let borrowed = Borrowed {
                finished: AtomicBool::new(false),
            };
            let panic = catch_unwind(AssertUnwindSafe(|| {
                let _: Result<(), TaskError> = tasks.scope(|s| {
                    let child = s.spawn(|| -> Result<(), TaskError> { panic_any(73u32) });
                    match mode {
                        0 => assert_eq!(child.join(), Err(TaskError::Cancelled)),
                        1 => drop(child),
                        _ => std::mem::forget(child),
                    }
                    s.spawn(|| {
                        borrowed.finished.store(true, Ordering::SeqCst);
                        Ok(())
                    });
                    Ok(())
                });
            }))
            .unwrap_err();
            assert_eq!(*panic.downcast::<u32>().unwrap(), 73);
            assert!(borrowed.finished.load(Ordering::SeqCst));
        }
    }

    #[test]
    fn task_body_panic_joins_children_before_resuming() {
        let tasks = Tasks(());
        let borrowed = Borrowed {
            finished: AtomicBool::new(false),
        };
        let state = &borrowed;
        let (tx, rx) = mpsc::channel::<()>();
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), TaskError> = tasks.scope(move |s| {
                let _until_body_unwinds = tx;
                s.spawn(move || {
                    assert!(rx.recv().is_err());
                    state.finished.store(true, Ordering::SeqCst);
                    Ok(())
                });
                panic_any(91u32)
            });
        }))
        .unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 91);
        assert!(borrowed.finished.load(Ordering::SeqCst));
    }

    #[test]
    fn task_discarded_result_destructor_panics_reach_the_scope() {
        struct Bomb;
        impl Drop for Bomb {
            fn drop(&mut self) {
                panic_any(29u32);
            }
        }
        let tasks = Tasks(());
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), TaskError> = tasks.scope(|s| {
                let (tx, rx) = mpsc::channel();
                let child = s.spawn(move || {
                    rx.recv().unwrap();
                    Ok(Bomb)
                });
                drop(child);
                tx.send(()).unwrap();
                Ok(())
            });
        }))
        .unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 29);
    }

    #[derive(Debug, PartialEq)]
    enum Failure {
        Child,
        Task(TaskError),
    }

    impl From<TaskError> for Failure {
        fn from(e: TaskError) -> Self {
            Self::Task(e)
        }
    }

    #[test]
    fn task_join_reports_cancelled_but_scope_keeps_the_single_child_error() {
        let tasks = Tasks(());
        for joined in [false, true] {
            let result: Result<(), Failure> = tasks.scope(|s| {
                let child = s.spawn(|| -> Result<(), Failure> { Err(Failure::Child) });
                if joined {
                    child.join()?;
                }
                Ok(())
            });
            assert_eq!(result, Err(Failure::Child));
        }
        assert_eq!(
            tasks.scope(|_| Err::<(), _>(Failure::Child)),
            Err(Failure::Child)
        );
    }

    #[test]
    fn task_scope_and_handle_auto_traits_match_the_memory_table() {
        fn both<T: Send + Sync>() {}
        fn scope<'s, 'env: 's, E>() {
            both::<Scope<'s, 'env, E>>();
        }
        fn task<'s>() {
            both::<Task<'s, Cell<u8>, Cell<u8>>>();
        }
        scope::<std::rc::Rc<()>>();
        task();
    }

    #[test]
    fn task_error_can_borrow_local_state() {
        #[derive(Debug, PartialEq)]
        enum BorrowedError<'a> {
            Task(TaskError),
            Message(&'a str),
        }
        impl From<TaskError> for BorrowedError<'_> {
            fn from(e: TaskError) -> Self {
                Self::Task(e)
            }
        }
        let tasks = Tasks(());
        let message = String::from("local error");
        let result: Result<(), _> = tasks.scope(|s| {
            s.spawn(|| Err::<(), _>(BorrowedError::Message(&message)));
            Ok(())
        });
        assert_eq!(result, Err(BorrowedError::Message(&message)));
    }
}
