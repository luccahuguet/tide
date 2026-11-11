// Shared verbatim with the conformance fixture; keep later task APIs out of this slice.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Condvar, LockResult, Mutex, MutexGuard, mpsc};
use std::thread;
use std::time::Duration;

mod scheduler {
    include!("task/scheduler.rs");
}
use scheduler::{Operation, Pending, Scheduler, Span, TaskId};

/// Authority to spawn structured tasks on the thread runtime.
#[derive(Clone)]
pub struct Tasks {
    simulation: Option<Arc<Mutex<Runtime>>>,
}

// Runtime owns all lifecycle/failure transitions. Payloads below only holds typed error data.
#[derive(Default)]
struct Runtime {
    scopes: BTreeMap<usize, ScopeState>,
    next_scope: usize,
    // Real waiting threads; cancellation and resources unpark them to recheck.
    waiting: Vec<thread::Thread>,
    baton: Option<Baton>,
    simulation_ended: bool,
}

struct Baton {
    scheduler: Scheduler,
    parked: Vec<Arc<Condvar>>,
    // Where each task blocked in a cancellable wait sits, so a cancel there wakes it.
    cancellable: BTreeMap<TaskId, Location>,
}

/// How a wait ended. When several hold at once, cancellation is reported first (TASK-4), then
/// readiness.
enum Wake {
    Ready,
    Timeout,
    Cancelled,
}

impl Baton {
    fn register(&mut self, span: Span) -> TaskId {
        let task = self.scheduler.register(span);
        self.parked.push(Arc::new(Condvar::new()));
        task
    }

    fn pass(&mut self, task: TaskId, pending: Pending) {
        match self.scheduler.pick(task, pending) {
            Some(next) if next != task => self.parked[next].notify_one(),
            Some(_) => {}
            // Every task is a parked thread with a live stack, so none can report by unwinding.
            None => {
                let report = self.scheduler.deadlock(task, pending);
                fatal_simulation(&report, &self.scheduler.trace);
            }
        }
    }

    fn wait(mut runtime: MutexGuard<'_, Runtime>, task: TaskId) -> MutexGuard<'_, Runtime> {
        let parked = runtime.baton.as_ref().unwrap().parked[task].clone();
        while runtime.baton.as_ref().unwrap().scheduler.current != Some(task) {
            runtime = sim_lock(parked.wait(runtime));
        }
        runtime
    }
}

fn fatal_simulation(message: &str, trace: &str) -> ! {
    use std::io::Write;
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{message}");
    let _ = stderr.write_all(trace.as_bytes());
    let _ = stderr.flush();
    std::process::abort();
}

fn sim_lock<T>(result: LockResult<T>) -> T {
    result.unwrap_or_else(|_| fatal_simulation("simulator runtime lock poisoned", ""))
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

    fn cancel(&mut self, scope: usize, cause: Cause) -> bool {
        if self.scopes[&scope].cause.is_some() {
            return false;
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
        for thread in &self.waiting {
            thread.unpark();
        }
        true
    }

    fn cancelled(&self, at: Location) -> bool {
        self.scopes[&at.scope].cause.is_some()
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

    fn finish(&mut self, at: Location, mut outcome: Outcome) -> (Outcome, bool) {
        let child = &mut self.scopes.get_mut(&at.scope).unwrap().children[at.child.unwrap()];
        if outcome == Outcome::Error && child.echo {
            outcome = Outcome::Echo;
        }
        child.outcome = Some(outcome);
        let cancelled = matches!(outcome, Outcome::Error | Outcome::Panic)
            && self.cancel(at.scope, Cause::Failure(at.scope));
        (outcome, cancelled)
    }

    fn wait_done(&self, operation: Operation) -> bool {
        match operation {
            Operation::Join(child) => self.baton.as_ref().unwrap().scheduler.finished(child),
            Operation::ScopeExit(scope) => self.scopes[&scope]
                .children
                .iter()
                .all(|child| child.outcome.is_some()),
            _ => false,
        }
    }

    // Simulated tasks whose wait is over: its condition holds, or it is cancellable and its
    // scope is cancelled. Run it after every change to either.
    fn wake_waiters(&mut self) {
        let baton = self.baton.as_ref().unwrap();
        let ready: Vec<_> = baton
            .scheduler
            .blocked()
            .filter(|&(task, pending)| {
                self.wait_done(pending.operation)
                    || baton
                        .cancellable
                        .get(&task)
                        .is_some_and(|&at| self.cancelled(at))
            })
            .collect();
        let scheduler = &mut self.baton.as_mut().unwrap().scheduler;
        for (task, pending) in ready {
            scheduler.wake(task, pending);
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
    at: Option<Location>,
    task: Option<TaskId>,
}

std::thread_local! {
    // Thread runtimes never migrate tasks; no user thread-local state is exposed.
    static CURRENT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

struct Enter(Option<Context>, PhantomData<std::rc::Rc<()>>);
impl Enter {
    fn new(context: Context) -> Self {
        Self(
            CURRENT.with(|slot| slot.replace(Some(context))),
            PhantomData,
        )
    }
}
impl Drop for Enter {
    fn drop(&mut self) {
        // SimRoot may have been dropped out of order inside this scope. Never restore its
        // dead main context after the live-task assertion unwinds through the scope.
        let previous = self.0.take().filter(|context| {
            context.task.is_none() || !sim_lock(context.runtime.lock()).simulation_ended
        });
        CURRENT.with(|slot| {
            slot.replace(previous);
        });
    }
}

impl Context {
    fn current() -> Option<Self> {
        CURRENT.with(|slot| slot.borrow().clone())
    }

    // The one check that real authority stays outside simulations. Inside one, its
    // operations would drop out of the schedule.
    fn outside_simulation() {
        CURRENT.with(|slot| {
            assert!(
                slot.borrow()
                    .as_ref()
                    .is_none_or(|context| context.task.is_none()),
                "real capability used inside a simulation",
            );
        });
    }

    // A real thread outside every scope: nothing can cancel it.
    fn detached() -> Self {
        Self {
            runtime: Arc::new(Mutex::new(Runtime::default())),
            at: None,
            task: None,
        }
    }

    // The caller's context for a handle with this simulation identity.
    fn of(simulation: &Option<Arc<Mutex<Runtime>>>) -> Option<Self> {
        match simulation {
            Some(runtime) => Some(Self::simulated(runtime)),
            None => {
                Self::outside_simulation();
                Self::current()
            }
        }
    }

    fn simulated(runtime: &Arc<Mutex<Runtime>>) -> Self {
        Self::current()
            .filter(|context| context.task.is_some() && Arc::ptr_eq(runtime, &context.runtime))
            .expect("simulation handle used outside its simulation")
    }

    fn lock(&self) -> MutexGuard<'_, Runtime> {
        let result = self.runtime.lock();
        if self.task.is_some() {
            sim_lock(result)
        } else {
            result.unwrap()
        }
    }

    fn point(&self, operation: Operation, span: Span) {
        let task = self.task.unwrap();
        let pending = Pending { operation, span };
        let mut runtime = self.lock();
        let baton = runtime.baton.as_mut().unwrap();
        if let Err(stalled) = baton.scheduler.point(task, pending) {
            // The caller keeps the baton. Unlock first: a panic must not poison the runtime.
            drop(runtime);
            panic!("{stalled}");
        }
        baton.pass(task, pending);
        drop(Baton::wait(runtime, task));
    }

    // The one owner of blocking (AP-OWNERSHIP-001). Waits until `operation` is done
    // (`Runtime::wait_done`), the instant `until` (an offset on the clock in use), or, when
    // `cancellable`, the cancellation of the caller's scope, which is checked first on entry and
    // on every wake and marks the echo as a checkpoint does. Later wait kinds need a
    // caller-supplied readiness check or resource-owned waiter handles; fx-hxw also needs a
    // Thread-or-eventfd waker. Lock order is runtime, then resource. A resource keeps its own
    // waiter Thread handles and never takes the runtime lock while holding its own lock.
    // Release both locks before parking.
    fn wait(
        &self,
        operation: Operation,
        span: Span,
        until: Option<Duration>,
        cancellable: bool,
    ) -> Wake {
        let at = self.at.filter(|_| cancellable);
        let mut runtime = self.lock();
        let Some(task) = self.task else {
            // Callers must keep real waits outside simulations: this branch uses host time and parking.
            let me = thread::current();
            runtime.waiting.push(me.clone());
            let wake = loop {
                if at.is_some_and(|at| runtime.observe(at)) {
                    break Wake::Cancelled;
                }
                if runtime.wait_done(operation) {
                    break Wake::Ready;
                }
                let left = until.map(|until| until.saturating_sub(crate::time::monotonic()));
                if left == Some(Duration::ZERO) {
                    break Wake::Timeout;
                }
                drop(runtime);
                match left {
                    Some(left) => thread::park_timeout(left),
                    None => thread::park(),
                }
                runtime = self.lock();
            };
            runtime.waiting.retain(|thread| thread.id() != me.id());
            return wake;
        };
        // Under the simulator no path below reads real time or parks on it.
        if at.is_some_and(|at| runtime.observe(at)) {
            return Wake::Cancelled;
        }
        if runtime.wait_done(operation) {
            return Wake::Ready;
        }
        let baton = runtime.baton.as_mut().unwrap();
        // A timer never lies in the past: virtual time is set only by a jump to one.
        if until.is_some_and(|until| until <= baton.scheduler.now) {
            return Wake::Timeout;
        }
        let pending = Pending { operation, span };
        baton.scheduler.block(task, pending, until);
        if let Some(at) = at {
            baton.cancellable.insert(task, at);
        }
        baton.pass(task, pending);
        let mut runtime = Baton::wait(runtime, task);
        runtime.baton.as_mut().unwrap().cancellable.remove(&task);
        if at.is_some_and(|at| runtime.observe(at)) {
            Wake::Cancelled
        } else if runtime.wait_done(operation) {
            Wake::Ready
        } else {
            Wake::Timeout
        }
    }

    // Records a first-cause cancellation made by this simulated task and wakes its waiters.
    fn record_cancel(&self, runtime: &mut Runtime, scope: usize, span: Span) {
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        scheduler.cancel(self.task.unwrap(), scope, span);
        runtime.wake_waiters();
    }
}

// SimRoot keeps this guard on the thread that holds the main task baton.
pub(crate) struct Simulation {
    tasks: Tasks,
    clock: crate::time::Clock,
    _main: Enter,
}

impl Simulation {
    #[track_caller]
    pub(crate) fn new(seed: u64) -> Self {
        assert!(
            Context::current().is_none(),
            "a task runtime is already active"
        );
        let runtime = Arc::new(Mutex::new(Runtime {
            baton: Some(Baton {
                scheduler: Scheduler::new(seed, std::panic::Location::caller()),
                parked: vec![Arc::new(Condvar::new())],
                cancellable: BTreeMap::new(),
            }),
            ..Runtime::default()
        }));
        let main = Enter::new(Context {
            runtime: runtime.clone(),
            at: None,
            task: Some(0),
        });
        Self {
            clock: crate::time::Clock::new(Time {
                simulation: Some(runtime.clone()),
            }),
            tasks: Tasks {
                simulation: Some(runtime),
            },
            _main: main,
        }
    }

    pub(crate) fn tasks(&self) -> &Tasks {
        &self.tasks
    }

    pub(crate) fn clock(&self) -> &crate::time::Clock {
        &self.clock
    }
}

impl Drop for Simulation {
    fn drop(&mut self) {
        let mut runtime = sim_lock(self.tasks.simulation.as_ref().unwrap().lock());
        let ended = runtime.baton.as_ref().unwrap().scheduler.current == Some(0)
            && runtime.scopes.is_empty();
        if ended {
            runtime.baton.as_mut().unwrap().scheduler.finish(
                0,
                if thread::panicking() {
                    Outcome::Panic
                } else {
                    Outcome::Ok
                },
            );
        }
        runtime.simulation_ended = true;
        // Assert unlocked: a failure must not poison the lock or abort an unwind in progress.
        drop(runtime);
        assert!(
            ended || thread::panicking(),
            "simulation ended with live tasks"
        );
    }
}

// Sinks never hand off the baton. Format before entering here, outside the runtime lock.
pub(crate) fn record_sink(stream: &str, message: &str, span: Span) -> bool {
    let Some(context) = Context::current().filter(|context| context.task.is_some()) else {
        return false;
    };
    context.lock().baton.as_mut().unwrap().scheduler.sink(
        context.task.unwrap(),
        stream,
        message,
        span,
    );
    true
}

/// Rejects real authority used inside a simulation.
pub(crate) fn outside_simulation() {
    Context::outside_simulation();
}

pub(crate) struct SimCell {
    runtime: Arc<Mutex<Runtime>>,
    id: usize,
}

impl SimCell {
    pub(crate) fn acquire(&self, span: Span) {
        Context::simulated(&self.runtime).point(Operation::CellAcquire(self.id), span);
    }

    pub(crate) fn release(&self, span: Span) {
        Context::simulated(&self.runtime).point(Operation::CellRelease(self.id), span);
    }
}

pub(crate) fn observe_cancellation() -> bool {
    CURRENT.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|context| context.at.is_some_and(|at| context.lock().observe(at)))
    })
}

impl Tasks {
    // File calls check cancellation once before drawing; their timers are shielded.
    pub(crate) fn file_op<T>(
        &self,
        name: &'static str,
        path: &std::path::Path,
        shielded: bool,
        span: Span,
        f: impl FnOnce() -> crate::io::Result<T>,
    ) -> crate::io::Result<T> {
        let context = self.context().expect("file operation needs a simulation");
        let operation = Operation::File(name);
        let cancelled = !shielded && observe_cancellation();
        let latency = if cancelled {
            Duration::ZERO
        } else {
            context
                .lock()
                .baton
                .as_mut()
                .unwrap()
                .scheduler
                .file_latency()
        };
        let result = if cancelled {
            Err(crate::io::ErrorKind::Cancelled.into())
        } else {
            let now = context.lock().baton.as_ref().unwrap().scheduler.now;
            if latency.is_zero() {
                context.point(operation, span);
            } else {
                let until = now
                    .checked_add(latency)
                    .expect("file latency exceeds virtual time range");
                context.wait(operation, span, Some(until), false);
            }
            f()
        };
        let outcome = match &result {
            Ok(_) => "ok".into(),
            Err(e) => format!("{:?}", e.kind()),
        };
        context.lock().baton.as_mut().unwrap().scheduler.file_event(
            context.task.unwrap(),
            operation,
            span,
            path,
            latency,
            &outcome,
        );
        result
    }

    pub(crate) fn world_file(&self, path: &std::path::Path, span: Span, result: &str) {
        let context = self.context().expect("world setup needs a simulation");
        context.lock().baton.as_mut().unwrap().scheduler.file_event(
            context.task.unwrap(),
            Operation::WorldFile,
            span,
            path,
            Duration::ZERO,
            result,
        );
    }

    pub(crate) fn check_simulation(&self) {
        self.context().expect("simulated file needs a simulation");
    }

    pub(crate) fn sim_elapsed(&self) -> Duration {
        sim_lock(self.simulation.as_ref().unwrap().lock())
            .baton
            .as_ref()
            .unwrap()
            .scheduler
            .now
    }

    pub(crate) fn sim_trace(&self) -> String {
        sim_lock(self.simulation.as_ref().unwrap().lock())
            .baton
            .as_ref()
            .unwrap()
            .scheduler
            .trace
            .clone()
    }

    pub(crate) fn sim_records(&self) -> Vec<crate::sim::Record> {
        sim_lock(self.simulation.as_ref().unwrap().lock())
            .baton
            .as_ref()
            .unwrap()
            .scheduler
            .records
            .clone()
    }

    pub(crate) fn unmodeled(&self, capability: &str) -> ! {
        self.context(); // Reject handles from another or an ended simulation first.
        panic!("simulated {capability} operations are not available yet")
    }

    pub(crate) fn real() -> Self {
        Self { simulation: None }
    }

    pub(crate) fn sim_cell(&self, span: Span) -> Option<SimCell> {
        let Some(runtime) = &self.simulation else {
            Context::outside_simulation();
            return None;
        };
        let context = Context::simulated(runtime);
        let id = context
            .lock()
            .baton
            .as_mut()
            .unwrap()
            .scheduler
            .allocate_cell();
        context.point(Operation::CellNew(id), span);
        Some(SimCell {
            runtime: runtime.clone(),
            id,
        })
    }

    fn context(&self) -> Option<Context> {
        Context::of(&self.simulation)
    }

    /// Runs `body` and joins every child, even with dropped or forgotten handles.
    /// Body panics continue after joining. Otherwise TASK-2 selects the earliest-spawned
    /// child panic, then independent child error, then the body's result. Checkpoints
    /// observe cancellation persistently; limits, deadlines, shields and isolation are absent.
    #[track_caller]
    pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
    where
        E: From<TaskError> + Send,
        F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>,
    {
        let span = std::panic::Location::caller();
        let mut context = self.context().unwrap_or_else(Context::detached);
        let id = context.lock().scope(context.at);
        context.at = Some(Location {
            scope: id,
            child: None,
        });
        let simulated = context.task.is_some();
        if simulated {
            context.point(Operation::Scope(id), span);
        }
        let payloads = Mutex::new(Vec::<(usize, thread::Result<E>)>::new());
        let result = thread::scope(|threads| {
            let _enter = Enter::new(context.clone());
            let result = catch_unwind(AssertUnwindSafe(|| {
                body(Scope {
                    threads,
                    payloads: Payloads(std::ptr::from_ref(&payloads).cast()),
                    runtime: &context.runtime,
                    id,
                    simulated,
                    cancelled: || E::from(TaskError::Cancelled),
                    _marker: PhantomData,
                })
            }));
            if !matches!(result, Ok(Ok(_))) {
                let mut runtime = context.lock();
                if runtime.cancel(id, Cause::Failure(id)) && simulated {
                    context.record_cancel(&mut runtime, id, span);
                }
            }
            if simulated {
                context.wait(Operation::ScopeExit(id), span, None, false);
            }
            result
        });
        let winner = context.lock().finish_scope(id);
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
    #[track_caller]
    pub fn checkpoint(&self) -> Result<(), Cancelled> {
        match &self.simulation {
            Some(runtime) => Context::simulated(runtime)
                .point(Operation::Checkpoint, std::panic::Location::caller()),
            None => Context::outside_simulation(),
        }
        check_cancellation()
    }

    /// Yields the thread, checking cancellation before yielding and again before returning.
    #[track_caller]
    pub fn yield_now(&self) -> Result<(), Cancelled> {
        let simulation = self.simulation.as_ref().map(Context::simulated);
        if simulation.is_none() {
            Context::outside_simulation();
        }
        check_cancellation()?;
        if let Some(context) = simulation {
            context.point(Operation::Yield, std::panic::Location::caller());
        } else {
            thread::yield_now();
        }
        check_cancellation()
    }
}

fn check_cancellation() -> Result<(), Cancelled> {
    if observe_cancellation() {
        Err(Cancelled)
    } else {
        Ok(())
    }
}

/// A copyable scope handle. Both lifetimes are invariant, as in Rust's scoped threads.
pub struct Scope<'s, 'env: 's, E> {
    threads: &'s (dyn Spawner<'s> + Sync),
    // A reference here would impose E: 's, which the public scope contract does not require.
    payloads: Payloads,
    runtime: &'s Arc<Mutex<Runtime>>,
    id: usize,
    simulated: bool,
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
    #[track_caller]
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
    where
        F: FnOnce() -> Result<U, E> + Send + 's,
    {
        let span = std::panic::Location::caller();
        let caller = self.simulated.then(|| Context::simulated(self.runtime));
        let (tx, rx) = mpsc::sync_channel(1);
        let (at, task) = {
            let mut runtime = if let Some(caller) = &caller {
                caller.lock()
            } else {
                self.runtime.lock().unwrap()
            };
            let at = runtime.spawn(self.id);
            let task = runtime.baton.as_mut().map(|baton| baton.register(span));
            (at, task)
        };
        let context = Context {
            runtime: self.runtime.clone(),
            at: Some(at),
            task,
        };
        // Erase E from the job's captures without adding E: 's to the public signature.
        let run_job: unsafe fn(F, mpsc::SyncSender<U>, Payloads, Context) = run::<F, U, E>;
        let payloads = self.payloads;
        #[cfg(test)]
        let delay = simulation_tests::START_DELAY.with(std::cell::Cell::get);
        let job = Box::new(move || {
            #[cfg(test)]
            if !delay.is_zero() {
                thread::sleep(delay);
            }
            // SAFETY: Scope's invariant E matches the mutex allocated by Tasks::scope, and its
            // invariant 's keeps this job within the native scope, even on unwind.
            unsafe { run_job(f, tx, payloads, context) };
        });
        let simulation = if let Some(caller) = caller {
            let task = task.unwrap();
            // Native spawn can unwind before the thread starts. A starting task cannot win
            // the baton, even if dropping the rejected job reaches a scheduling point.
            let started = catch_unwind(AssertUnwindSafe(|| self.threads.spawn(job)));
            let mut runtime = caller.lock();
            let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
            match started {
                Ok(()) => scheduler.start(task),
                Err(panic) => {
                    scheduler.abandon(task);
                    runtime.finish(at, Outcome::Ok);
                    runtime.wake_waiters();
                    drop(runtime);
                    resume_unwind(panic);
                }
            }
            drop(runtime);
            caller.point(Operation::Spawn(task), span);
            Some((self.runtime.clone(), task))
        } else {
            self.threads.spawn(job);
            None
        };
        Task {
            result: Mutex::new(rx),
            cancelled: self.cancelled,
            simulation,
            _marker: PhantomData,
        }
    }

    /// Cancels this scope and its descendants. Cancellation alone is not failure, and a child
    /// returning Err after explicit cancellation remains an independent failure.
    /// Cancel is not a scheduling point: the caller's next checkpoint observes it.
    #[track_caller]
    pub fn cancel(self) {
        if self.simulated {
            let span = std::panic::Location::caller();
            let caller = Context::simulated(self.runtime);
            let mut runtime = caller.lock();
            if runtime.cancel(self.id, Cause::Explicit(self.id)) {
                caller.record_cancel(&mut runtime, self.id, span);
            }
        } else {
            self.runtime
                .lock()
                .unwrap()
                .cancel(self.id, Cause::Explicit(self.id));
        }
    }
}

// Caller must supply the live scope mutex for this E; no reference to E escapes in the job type.
unsafe fn run<F, U, E>(f: F, tx: mpsc::SyncSender<U>, payloads: Payloads, context: Context)
where
    F: FnOnce() -> Result<U, E>,
{
    if let Some(task) = context.task {
        drop(Baton::wait(context.lock(), task));
    }
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
    let at = context.at.unwrap();
    if let Some(failure) = failure {
        // SAFETY: the caller supplied matching E; scope exit joins before dropping this mutex.
        let payloads = unsafe { &*payloads.0.cast::<Mutex<Vec<(usize, thread::Result<E>)>>>() };
        payloads.lock().unwrap().push((at.child.unwrap(), failure));
    }
    let mut runtime = context.lock();
    let (kind, cancelled) = runtime.finish(at, kind);
    if let Some(task) = context.task {
        // Publish cancellation before closing the result channel, and close it before handoff.
        drop(tx);
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        let pending = scheduler.finish(task, kind);
        if cancelled {
            scheduler.cancel(task, at.scope, pending.span);
        }
        runtime.wake_waiters();
        runtime.baton.as_mut().unwrap().pass(task, pending);
    }
    // In the real runtime tx drops after cancellation is recorded, before join returns.
}

/// A child's join handle; `Send + Sync` when `U, E: Send` (02 §8.1).
pub struct Task<'s, U, E> {
    result: Mutex<mpsc::Receiver<U>>,
    cancelled: fn() -> E,
    simulation: Option<(Arc<Mutex<Runtime>>, TaskId)>,
    _marker: PhantomData<(&'s mut &'s (), Mutex<E>)>,
}

impl<U, E> Task<'_, U, E> {
    /// Waits shielded, returning the child's value or `Cancelled` on failure/panic. A failed
    /// join marks the caller's echo bit exactly as a checkpoint there would (TASK-2).
    #[track_caller]
    pub fn join(self) -> Result<U, E> {
        if let Some((runtime, child)) = &self.simulation {
            let span = std::panic::Location::caller();
            let caller = Context::simulated(runtime);
            caller.point(Operation::Join(*child), span);
            caller.wait(Operation::Join(*child), span, None, false);
        }
        self.result.into_inner().unwrap().recv().map_err(|_| {
            observe_cancellation();
            (self.cancelled)()
        })
    }
}

/// Where a `Clock` reads time and sleeps: the real clocks, or its simulation's virtual time.
#[derive(Clone)]
pub(crate) struct Time {
    simulation: Option<Arc<Mutex<Runtime>>>,
}

impl Time {
    pub(crate) fn real() -> Self {
        Self { simulation: None }
    }

    /// A simulated clock's reading at its scheduling point: virtual time, or with `wall`
    /// the simulated wall time since the Unix epoch. `None` on the real runtime, where the
    /// caller reads its own clock.
    pub(crate) fn read(&self, wall: bool, span: Span) -> Option<Duration> {
        let Some(runtime) = &self.simulation else {
            Context::outside_simulation();
            return None;
        };
        let context = Context::simulated(runtime);
        let reading = {
            let runtime = context.lock();
            let scheduler = &runtime.baton.as_ref().unwrap().scheduler;
            if wall {
                scheduler.wall()
            } else {
                Some(scheduler.now)
            }
        }
        .expect("`system_time()` exceeds the representable wall-time range");
        let operation = if wall {
            Operation::ClockSystemTime(reading)
        } else {
            Operation::ClockNow
        };
        context.point(operation, span);
        // This task stays runnable at the point, so virtual time cannot advance during it.
        Some(reading)
    }

    /// Sleeps until the instant `until` gives for this clock's now, or without end for `None`.
    /// A target that is not after now is a checkpoint that does not block.
    pub(crate) fn sleep(
        &self,
        name: &'static str,
        until: impl FnOnce(Duration) -> Option<Duration>,
        span: Span,
    ) -> Result<(), Cancelled> {
        let context = Context::of(&self.simulation).unwrap_or_else(Context::detached);
        let Some(task) = context.task else {
            let until = until(crate::time::monotonic());
            let operation = Operation::Sleep { name, until };
            return match context.wait(operation, span, until, true) {
                Wake::Cancelled => Err(Cancelled),
                Wake::Ready | Wake::Timeout => Ok(()),
            };
        };
        let now = context.lock().baton.as_ref().unwrap().scheduler.now;
        let until = until(now);
        let operation = Operation::Sleep { name, until };
        let result = if until.is_some_and(|until| until <= now) {
            check_cancellation().and_then(|()| {
                context.point(operation, span);
                check_cancellation()
            })
        } else {
            match context.wait(operation, span, until, true) {
                Wake::Cancelled => Err(Cancelled),
                Wake::Ready | Wake::Timeout => Ok(()),
            }
        };
        let pending = Pending { operation, span };
        let mut runtime = context.lock();
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        scheduler.result(task, pending, result.is_ok());
        result
    }
}

/// Cancellation returned by a checkpoint: `Tasks::checkpoint`, `Tasks::yield_now`,
/// `Clock::sleep` and `Clock::sleep_until`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
        let message = String::from("local error");
        let result: Result<(), _> = tasks.scope(|s| {
            s.spawn(|| Err::<(), _>(BorrowedError::Message(&message)));
            Ok(())
        });
        assert_eq!(result, Err(BorrowedError::Message(&message)));
    }

    #[test]
    fn cancellation_is_level_triggered_and_does_not_cancel_joins_or_spawn() {
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let authority = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let tasks = Tasks::real();
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
        let authority = Tasks::real();
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
        let tasks = Tasks::real();
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

#[cfg(test)]
mod simulation_tests {
    include!("task/simulation_tests.rs");
}
