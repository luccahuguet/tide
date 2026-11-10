// Shared verbatim with the conformance fixture; keep later task APIs out of this slice.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

crate::capability!(
    /// Authority to spawn structured tasks on the thread runtime.
    Tasks
);

// Runtime owns all lifecycle/failure transitions. Payloads below only holds typed error data.
#[derive(Default)]
struct Runtime {
    scopes: BTreeMap<usize, ScopeState>,
    next_scope: usize,
}

struct ScopeState {
    parent: Option<Location>,
    cause: Option<Cause>,
    children: Vec<Child>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cause {
    Failure(usize),
    Explicit(usize),
}

#[derive(Default)]
struct Child {
    echo: bool,
    outcome: Option<Outcome>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Ok,
    Error,
    Echo,
    Panic,
}

#[derive(Clone, Copy)]
struct Location {
    scope: usize,
    child: Option<usize>,
}

impl Runtime {
    fn scope(&mut self, parent: Option<Location>) -> usize {
        let cause = parent.and_then(|p| self.scopes[&p.scope].cause);
        let id = self.next_scope;
        self.next_scope += 1;
        self.scopes.insert(
            id,
            ScopeState {
                parent,
                cause,
                children: Vec::new(),
            },
        );
        id
    }

    fn spawn(&mut self, scope: usize) -> Location {
        let children = &mut self.scopes.get_mut(&scope).unwrap().children;
        let child = children.len();
        children.push(Child::default());
        Location {
            scope,
            child: Some(child),
        }
    }

    fn cancel(&mut self, scope: usize, cause: Cause) {
        if self.scopes[&scope].cause.is_some() {
            return;
        }
        self.scopes.get_mut(&scope).unwrap().cause = Some(cause);
        // Parents precede descendants; a nested scope's own first cause stops propagation.
        let descendants: Vec<_> = self.scopes.range(scope + 1..).map(|(&id, _)| id).collect();
        for id in descendants {
            let parent = self.scopes[&id].parent.unwrap().scope;
            if self.scopes[&id].cause.is_none() && self.scopes[&parent].cause == Some(cause) {
                self.scopes.get_mut(&id).unwrap().cause = Some(cause);
            }
        }
    }

    fn observe(&mut self, mut at: Location) -> bool {
        let Some(cause) = self.scopes[&at.scope].cause else {
            return false;
        };
        if let Cause::Failure(origin) = cause {
            loop {
                if self.scopes[&at.scope].cause != Some(cause) {
                    break;
                }
                if at.scope == origin {
                    if let Some(child) = at.child {
                        self.scopes.get_mut(&at.scope).unwrap().children[child].echo = true;
                    }
                    break;
                }
                let Some(parent) = self.scopes[&at.scope].parent else {
                    break;
                };
                at = parent;
            }
        }
        true
    }

    fn finish(&mut self, at: Location, mut outcome: Outcome) {
        let child = &mut self.scopes.get_mut(&at.scope).unwrap().children[at.child.unwrap()];
        if outcome == Outcome::Error && child.echo {
            outcome = Outcome::Echo;
        }
        child.outcome = Some(outcome);
        if matches!(outcome, Outcome::Error | Outcome::Panic) {
            self.cancel(at.scope, Cause::Failure(at.scope));
        }
    }

    fn finish_scope(&mut self, scope: usize) -> Option<usize> {
        let children = self.scopes.remove(&scope).unwrap().children;
        [Outcome::Panic, Outcome::Error]
            .into_iter()
            .find_map(|outcome| {
                children
                    .iter()
                    .position(|child| child.outcome == Some(outcome))
            })
    }
}

#[derive(Clone)]
struct Context {
    runtime: Arc<Mutex<Runtime>>,
    at: Location,
}

std::thread_local! {
    // Thread runtimes never migrate tasks; no user thread-local state is exposed.
    static CURRENT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

struct Enter(Option<Context>);
impl Enter {
    fn new(context: Context) -> Self {
        Self(CURRENT.with(|slot| slot.replace(Some(context))))
    }
}
impl Drop for Enter {
    fn drop(&mut self) {
        CURRENT.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}

fn observe_cancellation() -> bool {
    CURRENT.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|context| context.runtime.lock().unwrap().observe(context.at))
    })
}

impl Tasks {
    /// Runs `body` and joins every child, even with dropped or forgotten handles.
    /// Body panics continue after joining. Otherwise TASK-2 selects the earliest-spawned
    /// child panic, then independent child error, then the body's result. Checkpoints
    /// observe cancellation persistently; limits, deadlines, shields and isolation are absent.
    pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
    where
        E: From<TaskError> + Send,
        F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>,
    {
        let parent = CURRENT.with(|slot| slot.borrow().clone());
        let runtime = parent.as_ref().map_or_else(
            || Arc::new(Mutex::new(Runtime::default())),
            |p| p.runtime.clone(),
        );
        let id = runtime.lock().unwrap().scope(parent.map(|p| p.at));
        let payloads = Mutex::new(Vec::<(usize, thread::Result<E>)>::new());
        let result = thread::scope(|threads| {
            let _enter = Enter::new(Context {
                runtime: runtime.clone(),
                at: Location {
                    scope: id,
                    child: None,
                },
            });
            let result = catch_unwind(AssertUnwindSafe(|| {
                body(Scope {
                    threads,
                    payloads: Payloads(std::ptr::from_ref(&payloads).cast()),
                    runtime: &runtime,
                    id,
                    cancelled: || E::from(TaskError::Cancelled),
                    _marker: PhantomData,
                })
            }));
            if !matches!(result, Ok(Ok(_))) {
                runtime.lock().unwrap().cancel(id, Cause::Failure(id));
            }
            result
        });
        let winner = runtime.lock().unwrap().finish_scope(id);
        let mut payloads = payloads.into_inner().unwrap();
        let selected = if result.is_err() {
            None
        } else {
            winner.map(|id| {
                payloads
                    .swap_remove(payloads.iter().position(|(child, _)| *child == id).unwrap())
                    .1
            })
        };
        // shortcut: suppressed outcomes become one `debug!` record when the log sink exists.
        let _suppressed =
            payloads.len() + usize::from(matches!(result, Ok(Err(_))) && selected.is_some());
        match result {
            Err(panic) => resume_unwind(panic),
            Ok(body) => match selected {
                Some(Err(panic)) => resume_unwind(panic),
                Some(Ok(error)) => Err(error),
                None => body,
            },
        }
    }

    /// An explicit checkpoint. Outside a scope it succeeds; inside a cancelled scope every
    /// call returns `Cancelled`, including calls made through another clone of `Tasks`.
    pub fn checkpoint(&self) -> Result<(), Cancelled> {
        if observe_cancellation() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }

    /// Yields the thread, checking cancellation before yielding and again before returning.
    pub fn yield_now(&self) -> Result<(), Cancelled> {
        self.checkpoint()?;
        thread::yield_now();
        self.checkpoint()
    }
}

/// A copyable scope handle. Both lifetimes are invariant, as in Rust's scoped threads.
pub struct Scope<'s, 'env: 's, E> {
    threads: &'s (dyn Spawner<'s> + Sync),
    // A reference here would impose E: 's, which the public scope contract does not require.
    payloads: Payloads,
    runtime: &'s Arc<Mutex<Runtime>>,
    id: usize,
    cancelled: fn() -> E,
    _marker: PhantomData<(&'s mut &'s (), &'env mut &'env (), fn(E) -> E)>,
}

#[derive(Clone, Copy)]
struct Payloads(*const ());

// SAFETY: only Tasks::scope constructs this pointer, with E: Send. Its mutex lives until the
// native scope joins all children. It is dereferenced only to &Mutex<Vec<(usize,
// thread::Result<E>)>>, which is Sync because E: Send. Payload access holds that mutex.
unsafe impl Send for Payloads {}
// SAFETY: the same constructor and synchronization invariant permits shared access.
unsafe impl Sync for Payloads {}

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
    /// Unlimited spawn is not a checkpoint and still starts a child in a cancelled scope.
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
    where
        F: FnOnce() -> Result<U, E> + Send + 's,
    {
        let (tx, rx) = mpsc::sync_channel(1);
        let at = self.runtime.lock().unwrap().spawn(self.id);
        let context = Context {
            runtime: self.runtime.clone(),
            at,
        };
        // Erase E from the job's captures without adding E: 's to the public signature.
        let run_job: unsafe fn(F, mpsc::SyncSender<U>, Payloads, Context) = run::<F, U, E>;
        let payloads = self.payloads;
        self.threads.spawn(Box::new(move || {
            // SAFETY: Scope's invariant E matches the mutex allocated by Tasks::scope, and its
            // invariant 's keeps this job within the native scope, even on unwind.
            unsafe { run_job(f, tx, payloads, context) };
        }));
        Task {
            result: Mutex::new(rx),
            cancelled: self.cancelled,
            _marker: PhantomData,
        }
    }

    /// Cancels this scope and its descendants. Cancellation alone is not failure, and a child
    /// returning Err after explicit cancellation remains an independent failure.
    pub fn cancel(self) {
        self.runtime
            .lock()
            .unwrap()
            .cancel(self.id, Cause::Explicit(self.id));
    }
}

// Caller must supply the live scope mutex for this E; no reference to E escapes in the job type.
unsafe fn run<F, U, E>(f: F, tx: mpsc::SyncSender<U>, payloads: Payloads, context: Context)
where
    F: FnOnce() -> Result<U, E>,
{
    let _enter = Enter::new(context.clone());
    let outcome = catch_unwind(AssertUnwindSafe(|| match f() {
        Ok(value) => {
            // A send to an already-dropped handle destroys U here. Buffered U drops on its
            // receiver's thread instead; panic attribution then depends on timing (r1 N2).
            let _ = tx.send(value);
            None
        }
        Err(error) => Some(error),
    }));
    let (kind, failure) = match outcome {
        Ok(Some(error)) => (Outcome::Error, Some(Ok(error))),
        Err(panic) => (Outcome::Panic, Some(Err(panic))),
        Ok(None) => (Outcome::Ok, None),
    };
    if let Some(failure) = failure {
        // SAFETY: the caller supplied matching E; scope exit joins before dropping this mutex.
        let payloads = unsafe { &*payloads.0.cast::<Mutex<Vec<(usize, thread::Result<E>)>>>() };
        payloads
            .lock()
            .unwrap()
            .push((context.at.child.unwrap(), failure));
    }
    context.runtime.lock().unwrap().finish(context.at, kind);
    // tx drops only after the failure and its cancellation are recorded, before join returns.
}

/// A child's join handle; `Send + Sync` when `U, E: Send` (02 §8.1).
pub struct Task<'s, U, E> {
    result: Mutex<mpsc::Receiver<U>>,
    cancelled: fn() -> E,
    _marker: PhantomData<(&'s mut &'s (), Mutex<E>)>,
}

impl<U, E> Task<'_, U, E> {
    /// Waits shielded, returning the child's value or `Cancelled` on failure/panic. A failed
    /// join marks the caller's echo bit exactly as a checkpoint there would (TASK-2).
    pub fn join(self) -> Result<U, E> {
        self.result.into_inner().unwrap().recv().map_err(|_| {
            observe_cancellation();
            (self.cancelled)()
        })
    }
}

/// Cancellation returned by an explicit checkpoint or yield.
#[derive(Debug)]
pub struct Cancelled;
impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("operation cancelled")
    }
}
impl std::error::Error for Cancelled {}
impl From<Cancelled> for crate::io::Error {
    fn from(_: Cancelled) -> Self {
        crate::io::ErrorKind::Cancelled.into()
    }
}

/// Task lifecycle errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskError {
    Cancelled,
    /// No operation in this slice returns this; deadline support belongs to fx-zc1.
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

    #[test]
    fn cancellation_is_level_triggered_and_does_not_cancel_joins_or_spawn() {
        let tasks = Tasks(());
        tasks.checkpoint().unwrap();
        tasks.yield_now().unwrap();
        let result: Result<_, TaskError> = tasks.scope(|s| {
            let ready = s.spawn(|| Ok(17));
            s.cancel();
            s.cancel();
            for _ in 0..3 {
                let error: crate::io::Error = tasks.checkpoint().unwrap_err().into();
                assert!(error.is_cancelled());
                assert_eq!(error.kind(), crate::io::ErrorKind::Cancelled);
                assert_eq!(
                    tasks.yield_now().map_err(|_| TaskError::Cancelled),
                    Err(TaskError::Cancelled)
                );
            }
            assert_eq!(ready.join()?, 17);
            // Unlimited spawn still starts, and cancellation alone does not turn Ok into Err.
            s.spawn(|| {
                assert!(tasks.checkpoint().is_err());
                assert!(tasks.yield_now().is_err());
                Ok(())
            })
            .join()?;
            Ok(23)
        });
        assert_eq!(result, Ok(23));
        tasks.checkpoint().unwrap();
        tasks.yield_now().unwrap();
    }

    #[test]
    fn earliest_spawned_independent_error_beats_completion_order_and_body() {
        let tasks = Tasks(());
        let result: Result<(), Failure> = tasks.scope(|s| {
            let (tx, rx) = mpsc::channel();
            let first = s.spawn(move || {
                rx.recv().unwrap();
                Err::<(), _>(Failure::Child)
            });
            let later = s.spawn(|| Err::<(), _>(Failure::Task(TaskError::DeadlineExceeded)));
            assert!(later.join().is_err());
            tx.send(()).unwrap();
            assert!(first.join().is_err());
            Err(Failure::Task(TaskError::Cancelled))
        });
        assert_eq!(result, Err(Failure::Child));
    }

    #[test]
    fn independent_failure_survives_earlier_checkpoint_and_join_echoes() {
        let authority = Tasks(());
        let tasks = &authority;
        for checkpoint in [false, true] {
            let result: Result<(), Failure> = tasks.scope(|s| {
                let (tx, rx) = mpsc::channel();
                let first = s.spawn(move || {
                    let failing = rx.recv().unwrap();
                    if checkpoint {
                        assert!(tasks.checkpoint().is_err());
                    } else {
                        assert!(Task::join(failing).is_err());
                    }
                    // Echo classification never inspects this unrelated-looking error.
                    Err::<(), _>(Failure::Task(TaskError::DeadlineExceeded))
                });
                let cause = s.spawn(|| Err::<(), _>(Failure::Child));
                // Sending the failed handle after its child finishes also tests join's bit.
                while tasks.checkpoint().is_ok() {
                    thread::yield_now();
                }
                tx.send(cause).unwrap();
                assert!(first.join().is_err());
                Ok(())
            });
            assert_eq!(result, Err(Failure::Child));
        }
    }

    #[test]
    fn successful_join_under_failure_cancellation_does_not_make_an_echo() {
        let tasks = Tasks(());
        let result: Result<(), Failure> = tasks.scope(|s| {
            let ready = s.spawn(|| Ok(7));
            let (tx, rx) = mpsc::channel();
            s.spawn(move || {
                rx.recv().unwrap();
                assert_eq!(ready.join()?, 7);
                Err::<(), _>(Failure::Child)
            });
            let cause = s.spawn(|| Err::<(), _>(Failure::Task(TaskError::DeadlineExceeded)));
            assert!(cause.join().is_err());
            tx.send(()).unwrap();
            Ok(())
        });
        assert_eq!(result, Err(Failure::Child));
    }

    #[test]
    fn body_error_cancels_children_and_body_panic_wins_after_cleanup() {
        let tasks = Tasks(());
        let result: Result<(), Failure> = tasks.scope(|s| {
            s.spawn(|| {
                while tasks.checkpoint().is_ok() {
                    thread::yield_now();
                }
                Err::<(), _>(Failure::Child)
            });
            Err(Failure::Task(TaskError::DeadlineExceeded))
        });
        assert_eq!(result, Err(Failure::Task(TaskError::DeadlineExceeded)));
        let finished = AtomicBool::new(false);
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), TaskError> = tasks.scope(|s| {
                s.spawn(|| -> Result<(), TaskError> {
                    while tasks.checkpoint().is_ok() {
                        thread::yield_now();
                    }
                    finished.store(true, Ordering::SeqCst);
                    panic_any(12u32)
                });
                panic_any(34u32)
            });
        }))
        .unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 34);
        assert!(finished.load(Ordering::SeqCst));
        tasks.checkpoint().unwrap();
    }

    #[test]
    fn earliest_spawned_panic_beats_other_panics_and_errors() {
        let tasks = Tasks(());
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<(), Failure> = tasks.scope(|s| {
                let (tx, rx) = mpsc::channel();
                let observer = &tasks;
                let first = s.spawn(move || -> Result<(), Failure> {
                    rx.recv().unwrap();
                    assert!(observer.checkpoint().is_err());
                    panic_any(11u32)
                });
                let later = s.spawn(|| -> Result<(), Failure> { panic_any(22u32) });
                assert!(later.join().is_err());
                tx.send(()).unwrap();
                assert!(first.join().is_err());
                s.spawn(|| Err::<(), _>(Failure::Child));
                Err(Failure::Task(TaskError::Cancelled))
            });
        }))
        .unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 11);
    }

    #[test]
    fn nested_cancellation_preserves_first_cause_and_echo_ancestry() {
        let authority = Tasks(());
        let tasks = &authority;
        for inner_cancels_first in 0..3 {
            let result: Result<(), Failure> = tasks.scope(|outer| {
                let (ready_tx, ready_rx) = mpsc::channel();
                let (go_tx, go_rx) = mpsc::channel();
                outer.spawn(move || {
                    let result = tasks.scope(|inner| {
                        match inner_cancels_first {
                            1 => inner.cancel(),
                            2 => {
                                assert!(
                                    inner.spawn(|| Err::<(), _>(Failure::Child)).join().is_err()
                                );
                            }
                            _ => {}
                        }
                        ready_tx.send(()).unwrap();
                        go_rx.recv().unwrap();
                        inner.spawn(|| {
                            assert!(tasks.checkpoint().is_err());
                            assert!(tasks.yield_now().is_err());
                            Err::<(), _>(Failure::Child)
                        });
                        Ok(())
                    });
                    assert_eq!(result, Err(Failure::Child));
                    result
                });
                ready_rx.recv().unwrap();
                let cause =
                    outer.spawn(|| Err::<(), _>(Failure::Task(TaskError::DeadlineExceeded)));
                assert!(cause.join().is_err());
                go_tx.send(()).unwrap();
                Ok(())
            });
            assert_eq!(
                result,
                Err(if inner_cancels_first != 0 {
                    Failure::Child
                } else {
                    Failure::Task(TaskError::DeadlineExceeded)
                })
            );
        }
    }

    #[test]
    fn explicit_cancel_does_not_mark_errors_as_echoes() {
        let tasks = Tasks(());
        let result: Result<(), Failure> = tasks.scope(|s| {
            s.cancel();
            s.spawn(|| {
                assert!(tasks.checkpoint().is_err());
                Err::<(), _>(Failure::Child)
            });
            Ok(())
        });
        assert_eq!(result, Err(Failure::Child));
    }
}
