// Shared verbatim with the conformance fixture.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, LockResult, Mutex, MutexGuard, mpsc};
use std::thread;
use std::time::Duration;

mod scheduler {
    include!("task/scheduler.rs");
}
pub(crate) use scheduler::Trace;
use scheduler::{Operation, Pending, Scheduler, Span, TaskId};
mod limit {
    include!("task/limit.rs");
}
use limit::{Limit, Slots};
mod channel {
    include!("task/channel.rs");
}
pub(crate) use channel::Channel;
mod net {
    include!("task/net.rs");
}
pub(crate) use net::{NetContext, Network};
mod proc {
    include!("task/proc.rs");
}
pub(crate) use proc::{Ended, ModelContext, ProcContext};

#[derive(Clone)]
struct Waiter {
    thread: thread::Thread,
    event: Option<Arc<std::os::fd::OwnedFd>>,
}
impl Waiter {
    fn unpark(&self) {
        self.thread.unpark();
        if let Some(event) = &self.event {
            let _ = rustix::io::write(event, &1_u64.to_ne_bytes());
        }
    }
}

/// Authority to spawn structured tasks on the thread runtime.
#[derive(Clone)]
pub struct Tasks {
    simulation: Option<Arc<Mutex<Runtime>>>,
    limits: Vec<Arc<Limit>>,
}

// Runtime owns all lifecycle/failure transitions. Payloads below only holds typed error data.
#[derive(Default)]
struct Runtime {
    scopes: BTreeMap<usize, ScopeState>,
    next_scope: usize,
    // Real waiting threads; cancellation and resources unpark them to recheck.
    waiting: Vec<Waiter>,
    baton: Option<Baton>,
    simulation_ended: bool,
    inputs: crate::env::Inputs,
    replay_journal: Option<(Arc<Mutex<crate::sim::manifest::Journal>>, usize)>,
}

struct Baton {
    scheduler: Scheduler,
    parked: Vec<Arc<Condvar>>,
    // Where each task blocked in a cancellable wait sits, so a cancel there wakes it.
    cancellable: BTreeMap<TaskId, Location>,
    resources: BTreeMap<TaskId, Arc<Signal>>,
    // Simulated child processes' model tasks (03 §9.9), released when the simulation ends.
    models: Vec<TaskId>,
}

// Resource-owned readiness and waker, also usable across independent real runtimes.
pub(crate) struct Signal {
    ready: AtomicBool,
    thread: Mutex<thread::Thread>,
}
impl Signal {
    pub(crate) fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            thread: Mutex::new(thread::current()),
        }
    }
    pub(crate) fn wake(&self) {
        self.ready.store(true, Ordering::SeqCst);
        self.thread.lock().unwrap().unpark();
    }
    fn ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
}

/// How a wait ended. When several hold at once, cancellation is reported first (TASK-4), then
/// readiness.
enum Wake {
    Ready,
    Timeout,
    Cancelled,
}

// Resource waits can defer echo observation until they decide whether a hand-off already
// completed. Waiting itself must still wake on cancellation; only a returned error sets a bit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cancellation {
    Shielded,
    Checkpoint,
    Deferred,
}
impl Cancellation {
    fn check(self, runtime: &mut Runtime, at: Location) -> bool {
        match self {
            Self::Shielded => false,
            Self::Checkpoint => runtime.observe(at),
            Self::Deferred => runtime.cancelled(at),
        }
    }
}

impl Baton {
    fn register(&mut self, span: Span) -> TaskId {
        let task = self.scheduler.register(span);
        self.parked.push(Arc::new(Condvar::new()));
        task
    }

    fn wait(mut runtime: MutexGuard<'_, Runtime>, task: TaskId) -> MutexGuard<'_, Runtime> {
        let parked = runtime.baton.as_ref().unwrap().parked[task].clone();
        while runtime.baton.as_ref().unwrap().scheduler.current != Some(task) {
            if runtime.simulation_ended && runtime.baton.as_ref().unwrap().models.contains(&task) {
                break;
            }
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
    deadline: Option<crate::time::Instant>,
    shielded: bool,
    children: Vec<Child>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cause {
    Failure(usize),
    Explicit(usize),
    Deadline(usize),
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
    Deadline,
    Panic,
}

#[derive(Clone, Copy)]
struct Location {
    scope: usize,
    child: Option<usize>,
}

impl Runtime {
    fn pass(&mut self, task: TaskId, pending: Pending) {
        loop {
            let baton = self.baton.as_mut().unwrap();
            if let Some(next) = baton.scheduler.pick(task, pending) {
                if next != task {
                    baton.parked[next].notify_one();
                }
                return;
            }
            if !baton.scheduler.advance(task, pending) {
                let report = baton.scheduler.deadlock(task, pending);
                fatal_simulation(&report, &baton.scheduler.trace());
            }
            while let Some((class, id)) = self.baton.as_mut().unwrap().scheduler.fire_timer() {
                if class == 0 {
                    self.expire(id, task, pending.span);
                }
            }
            self.wake_waiters();
        }
    }

    fn expire(&mut self, scope: usize, task: TaskId, span: Span) {
        if self.cancel(scope, Cause::Deadline(scope)) {
            let at = self.scopes[&scope].deadline.unwrap().0;
            if let Some(baton) = &mut self.baton {
                baton.scheduler.event_result(
                    task,
                    Pending {
                        operation: Operation::Deadline(scope, at),
                        span,
                    },
                    "cancelled",
                );
            }
        }
    }

    fn refresh_deadlines(&mut self) {
        if self.baton.is_some() {
            return;
        }
        let now = crate::time::monotonic();
        let mut expired: Vec<_> = self
            .scopes
            .iter()
            .filter_map(|(&id, scope)| {
                scope
                    .deadline
                    .filter(|at| at.0 <= now && scope.cause.is_none())
                    .map(|at| (at, id))
            })
            .collect();
        expired.sort_unstable();
        for (_, scope) in expired {
            self.cancel(scope, Cause::Deadline(scope));
        }
    }

    fn effective_deadline(&self, mut at: Location) -> Option<Duration> {
        let mut deadline = None;
        loop {
            let scope = &self.scopes[&at.scope];
            if scope.shielded {
                break;
            }
            if let Some(t) = scope.deadline {
                deadline = Some(deadline.map_or(t.0, |d: Duration| d.min(t.0)));
            }
            let Some(parent) = scope.parent else {
                break;
            };
            at = parent;
        }
        deadline
    }

    fn observe_unshielded(&mut self, mut at: Location) -> bool {
        // A nested scope's own first cause wins even when a failed join ignores shields.
        while self.scopes[&at.scope].cause.is_none() {
            let Some(parent) = self.scopes[&at.scope].parent else {
                return false;
            };
            at = parent;
        }
        self.observe(at)
    }

    fn scope(&mut self, parent: Option<Location>) -> usize {
        let cause = parent.and_then(|p| self.scopes[&p.scope].cause);
        let id = self.next_scope;
        self.next_scope += 1;
        self.scopes.insert(
            id,
            ScopeState {
                parent,
                cause,
                deadline: None,
                shielded: false,
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
            if !self.scopes[&id].shielded
                && self.scopes[&id].cause.is_none()
                && self.scopes[&parent].cause == Some(cause)
            {
                self.scopes.get_mut(&id).unwrap().cause = Some(cause);
            }
        }
        if let Some(baton) = &mut self.baton {
            for (&id, scope) in &self.scopes {
                if scope.cause.is_some() {
                    baton
                        .scheduler
                        .deadline(id, scope.deadline.map(|t| t.0), None);
                }
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
        if outcome == Outcome::Error
            && self.scopes[&at.scope].cause == Some(Cause::Deadline(at.scope))
        {
            outcome = Outcome::Deadline;
        }
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
            Operation::Join(child) => self
                .baton
                .as_ref()
                .is_some_and(|baton| baton.scheduler.finished(child)),
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
        for thread in &self.waiting {
            thread.unpark();
        }
        let Some(baton) = self.baton.as_ref() else {
            return;
        };
        let ready: Vec<_> = baton
            .scheduler
            .blocked()
            .filter(|&(task, pending)| {
                baton
                    .resources
                    .get(&task)
                    .is_some_and(|signal| signal.ready())
                    || self.wait_done(pending.operation)
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

    fn finish_scope(&mut self, scope: usize) -> Option<(usize, Outcome)> {
        if let Some(baton) = &mut self.baton {
            baton
                .scheduler
                .deadline(scope, self.scopes[&scope].deadline.map(|t| t.0), None);
        }
        let children = self.scopes.remove(&scope).unwrap().children;
        children
            .iter()
            .position(|child| child.outcome == Some(Outcome::Panic))
            .map(|id| (id, Outcome::Panic))
            .or_else(|| {
                children.iter().enumerate().find_map(|(id, child)| {
                    child
                        .outcome
                        .filter(|outcome| matches!(outcome, Outcome::Error | Outcome::Deadline))
                        .map(|outcome| (id, outcome))
                })
            })
    }
}

#[derive(Clone)]
pub(crate) struct Context {
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
            let mut runtime = result.unwrap();
            runtime.refresh_deadlines();
            runtime
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
        runtime.pass(task, pending);
        drop(Baton::wait(runtime, task));
    }

    // The one owner of blocking (AP-OWNERSHIP-001). Waits until `operation` is done
    // (`Runtime::wait_done`), the instant `until` (an offset on the clock in use), or, when
    // `cancellable`, the cancellation of the caller's scope, which is checked first on entry and
    // on every wake. Checkpoint waits mark the echo there; a Deferred resource wait observes
    // only when the resource returns cancellation, after checking a completed hand-off.
    // A blocking resource passes one fresh Signal per waiting task to wait_for and wakes it
    // on hand-off. A resource with
    // several waiters, such as a channel, keeps one Signal per waiter; Signals are one-shot
    // and hold one Thread handle. Socket waits use a per-thread eventfd waker. Lock order is
    // runtime, then resource; a resource never takes the runtime lock while holding its own
    // lock. Release both locks before parking.
    fn wait(
        &self,
        operation: Operation,
        span: Span,
        until: Option<Duration>,
        cancellable: bool,
    ) -> Wake {
        self.wait_for(
            operation,
            span,
            until,
            if cancellable {
                Cancellation::Checkpoint
            } else {
                Cancellation::Shielded
            },
            None,
        )
    }

    fn wait_for(
        &self,
        operation: Operation,
        span: Span,
        until: Option<Duration>,
        cancellation: Cancellation,
        resource: Option<Arc<Signal>>,
    ) -> Wake {
        self.wait_with_socket(operation, span, until, cancellation, resource, None)
            .expect("a wait without a socket cannot return an I/O error")
    }

    fn wait_with_socket(
        &self,
        operation: Operation,
        span: Span,
        until: Option<Duration>,
        cancellation: Cancellation,
        resource: Option<Arc<Signal>>,
        socket: Option<(
            std::os::fd::BorrowedFd<'_>,
            rustix::event::PollFlags,
            Arc<std::os::fd::OwnedFd>,
        )>,
    ) -> crate::io::Result<Wake> {
        let at = self.at.filter(|_| cancellation != Cancellation::Shielded);
        let mut runtime = self.lock();
        if let Some(resource) = &resource {
            *resource.thread.lock().unwrap() = thread::current();
        }
        let Some(task) = self.task else {
            // Callers must keep real waits outside simulations: this branch uses host time and parking.
            let event = socket.as_ref().map(|(_, _, event)| event.clone());
            let me = thread::current();
            runtime.waiting.push(Waiter {
                thread: me.clone(),
                event: event.clone(),
            });
            let wake = loop {
                if at.is_some_and(|at| cancellation.check(&mut runtime, at)) {
                    break Ok(Wake::Cancelled);
                }
                if resource.as_ref().is_some_and(|signal| signal.ready())
                    || runtime.wait_done(operation)
                {
                    break Ok(Wake::Ready);
                }
                let deadline = at.and_then(|at| runtime.effective_deadline(at));
                let target = match (until, deadline) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
                let left = target.map(|until| until.saturating_sub(crate::time::monotonic()));
                if left == Some(Duration::ZERO) {
                    runtime.refresh_deadlines();
                    if at.is_some_and(|at| cancellation.check(&mut runtime, at)) {
                        break Ok(Wake::Cancelled);
                    }
                    break Ok(Wake::Timeout);
                }
                drop(runtime);
                if let Some((fd, flags, event)) = &socket {
                    let ready = net::poll(*fd, *flags, event, left);
                    runtime = self.lock();
                    let ready = match ready {
                        Ok(ready) => ready,
                        Err(error) => break Err(error),
                    };
                    if ready && !at.is_some_and(|at| cancellation.check(&mut runtime, at)) {
                        break Ok(Wake::Ready);
                    }
                    continue;
                }
                match left {
                    Some(left) => thread::park_timeout(left),
                    None => thread::park(),
                }
                runtime = self.lock();
            };
            runtime.waiting.retain(|thread| thread.thread.id() != me.id());
            return wake;
        };
        // Under the simulator no path below reads real time or parks on it.
        if at.is_some_and(|at| cancellation.check(&mut runtime, at)) {
            return Ok(Wake::Cancelled);
        }
        if resource.as_ref().is_some_and(|signal| signal.ready()) || runtime.wait_done(operation) {
            return Ok(Wake::Ready);
        }
        let baton = runtime.baton.as_mut().unwrap();
        // A timer never lies in the past: virtual time is set only by a jump to one.
        if until.is_some_and(|until| until <= baton.scheduler.now) {
            return Ok(Wake::Timeout);
        }
        let pending = Pending { operation, span };
        baton.scheduler.block(task, pending, until);
        if let Some(resource) = &resource {
            baton.resources.insert(task, resource.clone());
        }
        if let Some(at) = at {
            baton.cancellable.insert(task, at);
        }
        runtime.pass(task, pending);
        let mut runtime = Baton::wait(runtime, task);
        runtime.baton.as_mut().unwrap().cancellable.remove(&task);
        runtime.baton.as_mut().unwrap().resources.remove(&task);
        Ok(if at.is_some_and(|at| cancellation.check(&mut runtime, at)) {
            Wake::Cancelled
        } else if resource.as_ref().is_some_and(|signal| signal.ready())
            || runtime.wait_done(operation)
        {
            Wake::Ready
        } else {
            Wake::Timeout
        })
    }

    // Records a first-cause cancellation made by this simulated task and wakes its waiters.
    fn record_cancel(&self, runtime: &mut Runtime, scope: usize, span: Span) {
        let cause = runtime.scopes[&scope].cause.unwrap();
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        scheduler.cancel(self.task.unwrap(), scope, span, cause);
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
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn new(seed: u64) -> Self {
        Self::new_at(seed, None)
    }

    #[track_caller]
    pub(crate) fn new_at(seed: u64, origin: Option<crate::test::Source>) -> Self {
        assert!(
            Context::current().is_none(),
            "a task runtime is already active"
        );
        let runtime = Arc::new(Mutex::new(Runtime {
            baton: Some(Baton {
                scheduler: Scheduler::new(seed, std::panic::Location::caller()),
                parked: vec![Arc::new(Condvar::new())],
                cancellable: BTreeMap::new(),
                resources: BTreeMap::new(),
                models: Vec::new(),
            }),
            ..Runtime::default()
        }));
        runtime.lock().unwrap().baton.as_mut().unwrap().scheduler.main_origin = origin;
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
                limits: Vec::new(),
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
        let baton = runtime.baton.as_ref().unwrap();
        for &model in &baton.models {
            baton.parked[model].notify_one();
        }
        // Assert unlocked: a failure must not poison the lock or abort an unwind in progress.
        drop(runtime);
        assert!(
            ended || thread::panicking(),
            "simulation ended with live tasks"
        );
    }
}

// CURRENT's simulation task identity is the sole owner of parallel-region mode.
pub(crate) fn is_simulating() -> bool {
    CURRENT.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|context| context.task.is_some())
    })
}

// Sinks never hand off the baton. Format before entering here, outside the runtime lock.
pub(crate) fn record_sink(stream: &'static str, message: &[u8], span: Span) -> bool {
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

/// Simulation identity and trace family for the public Arc/Weak wrappers.
#[derive(Clone)]
pub(crate) struct ArcHandle {
    simulation: Option<Arc<Mutex<Runtime>>>,
    id: usize,
}

impl ArcHandle {
    pub(crate) fn new(tasks: &Tasks, span: Span) -> Self {
        let context = tasks.context();
        let id = context
            .as_ref()
            .filter(|c| c.task.is_some())
            .map_or(0, |c| {
                c.lock().baton.as_mut().unwrap().scheduler.allocate_arc()
            });
        let handle = Self {
            simulation: tasks.simulation.clone(),
            id,
        };
        handle.begin("arc.new", span);
        handle
    }

    pub(crate) fn begin(&self, name: &'static str, span: Span) {
        if let Some(context) = Context::of(&self.simulation).filter(|c| c.task.is_some()) {
            context.point(Operation::Arc(name, self.id), span);
        }
    }

    pub(crate) fn downgrade(&self, tasks: &Tasks, span: Span) -> Self {
        // Both validations use the current context, so neither real nor foreign Tasks can mix.
        tasks.context();
        self.begin("arc.downgrade", span);
        self.clone()
    }
}

pub(crate) struct SimCell {
    runtime: Arc<Mutex<Runtime>>,
    id: usize,
}

impl SimCell {
    pub(crate) fn id(&self) -> u64 {
        self.id as u64
    }

    pub(crate) fn context<'a>(
        cells: impl IntoIterator<Item = Option<&'a Self>>,
    ) -> Option<Context> {
        let context =
            CURRENT.with(|slot| slot.borrow().as_ref().filter(|c| c.task.is_some()).cloned());
        for cell in cells {
            match cell {
                Some(cell) => assert!(
                    context
                        .as_ref()
                        .is_some_and(|c| Arc::ptr_eq(&cell.runtime, &c.runtime)),
                    "simulation handle used outside its simulation",
                ),
                None => assert!(
                    context.is_none(),
                    "real capability used inside a simulation"
                ),
            }
        }
        context
    }

    pub(crate) fn begin_set(
        context: Option<&Context>,
        ids: impl IntoIterator<Item = usize>,
        span: Span,
    ) {
        if let Some(context) = context {
            let ids: Vec<_> = ids.into_iter().collect();
            context
                .lock()
                .baton
                .as_mut()
                .unwrap()
                .scheduler
                .cell_set(context.task.unwrap(), &ids);
            context.point(Operation::CellsAcquire, span);
        }
    }

    pub(crate) fn record_set(
        context: Option<&Context>,
        ids: impl IntoIterator<Item = usize>,
        acquire: bool,
        span: Span,
    ) {
        if let Some(context) = context {
            context.lock().baton.as_mut().unwrap().scheduler.cells(
                context.task.unwrap(),
                ids,
                acquire,
                span,
            );
        }
    }

    pub(crate) fn end_set(context: Option<&Context>, span: Span) {
        if let Some(context) = context {
            context.point(Operation::CellsRelease, span);
        }
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
    pub(crate) fn sim_recording(&self, journal: Arc<Mutex<crate::sim::manifest::Journal>>, ordinal: usize) {
        sim_lock(self.simulation.as_ref().unwrap().lock()).replay_journal = Some((journal, ordinal));
    }

    pub(crate) fn replay_input(&self, kind: &str, key: Vec<u8>, value: Vec<u8>) {
        let journal = sim_lock(self.simulation.as_ref().unwrap().lock()).replay_journal.clone();
        if let Some((journal, ordinal)) = journal { journal.lock().unwrap().input(ordinal, kind, key, value); }
    }

    /// Process inputs take one scheduling point, no time and no random draw.
    #[track_caller]
    pub(crate) fn input_op<T>(&self, name: &'static str, f: impl FnOnce(&mut crate::env::Inputs) -> T) -> T {
        let context = self.context().expect("input operation needs a simulation");
        context.point(Operation::Input(name), std::panic::Location::caller());
        f(&mut context.lock().inputs)
    }

    pub(crate) fn world_inputs(&self, f: impl FnOnce(&mut crate::env::Inputs)) {
        let context = self.context().expect("input setup needs a simulation");
        f(&mut context.lock().inputs);
    }

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
            .trace()
    }

    /// Moves the trace out as data, for the runner's determinism check and failure report.
    pub(crate) fn sim_take_trace(&self) -> Trace {
        sim_lock(self.simulation.as_ref().unwrap().lock())
            .baton
            .as_mut()
            .unwrap()
            .scheduler
            .take_trace()
    }

    pub(crate) fn sim_records(&self) -> Vec<crate::sim::Record> {
        sim_lock(self.simulation.as_ref().unwrap().lock())
            .baton
            .as_ref()
            .unwrap()
            .scheduler
            .records()
    }

    pub(crate) fn unmodeled(&self, capability: &str) -> ! {
        self.context(); // Reject handles from another or an ended simulation first.
        panic!("simulated {capability} operations are not available yet")
    }

    pub(crate) fn real() -> Self {
        Self {
            simulation: None,
            limits: Vec::new(),
        }
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

    /// Shares an additional concurrency bound across this handle's scopes and clones.
    /// Only spawning under a zero count panics; creating it is valid for empty work.
    pub fn limit(&self, n: usize) -> Tasks {
        let context = self.context();
        let id = context
            .as_ref()
            .filter(|c| c.task.is_some())
            .map_or(0, |c| {
                c.lock().baton.as_mut().unwrap().scheduler.allocate_limit()
            });
        let mut tasks = self.clone();
        tasks.limits.push(Limit::new(id, n));
        tasks
    }

    /// Runs `body` and joins every child, even with dropped or forgotten handles.
    /// Body panics continue after joining. Otherwise TASK-2 selects the earliest-spawned
    /// child panic, then independent child error, then the body's result. Checkpoints
    /// observe cancellation persistently; deadline expiry alone preserves successful results.
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
                    limits: &self.limits,
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
            context.wait(Operation::ScopeExit(id), span, None, false);
            result
        });
        let winner = context.lock().finish_scope(id);
        let mut payloads = payloads.into_inner().unwrap();
        let selected = if result.is_err() {
            None
        } else {
            winner.map(|(id, outcome)| {
                let failure = payloads
                    .swap_remove(payloads.iter().position(|(child, _)| *child == id).unwrap())
                    .1;
                if outcome == Outcome::Deadline {
                    Ok(E::from(TaskError::DeadlineExceeded))
                } else {
                    failure
                }
            })
        };
        let suppressed =
            payloads.len() + usize::from(matches!(result, Ok(Err(_))) && selected.is_some());
        if suppressed != 0 {
            crate::__sink::log(crate::log::Level::Debug, format_args!("{suppressed} suppressed task outcomes"));
        }
        match result {
            Err(panic) => resume_unwind(panic),
            Ok(body) => match selected {
                Some(Err(panic)) => resume_unwind(panic),
                Some(Ok(error)) => Err(error),
                None => body,
            },
        }
    }

    /// Runs an owned closure in a new task and waits without interruption. Only unwinding
    /// panics are contained; aborts and stack overflows retain process-wide behavior.
    #[track_caller]
    pub fn spawn_isolated<T: Send>(
        &self,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, Panicked> {
        let span = std::panic::Location::caller();
        let caller = self.context().unwrap_or_else(Context::detached);
        let task = caller
            .lock()
            .baton
            .as_mut()
            .map(|baton| baton.register(span));
        let mut context = caller.clone();
        context.task = task;
        let done = Arc::new(Signal::new());
        let finished = done.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        thread::scope(|threads| {
            let started = catch_unwind(AssertUnwindSafe(|| {
                thread::Builder::new().spawn_scoped(threads, move || {
                    let (result, _enter) = run_caught(&context, f);
                    let result = result.map_err(Panicked::from_payload);
                    let outcome = if result.is_ok() {
                        Outcome::Ok
                    } else {
                        Outcome::Panic
                    };
                    tx.send(result).ok();
                    let mut runtime = context.lock();
                    finished.wake();
                    if let Some(task) = task {
                        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
                        let pending = scheduler.finish_isolated(task, outcome);
                        runtime.wake_waiters();
                        runtime.pass(task, pending);
                    }
                })
            }));
            let handle = match started {
                Ok(Ok(handle)) => handle,
                other => {
                    if let Some(task) = task {
                        caller
                            .lock()
                            .baton
                            .as_mut()
                            .unwrap()
                            .scheduler
                            .abandon(task);
                    }
                    match other {
                        Err(panic) => resume_unwind(panic),
                        Ok(Err(error)) => panic!("could not start isolated task: {error}"),
                        _ => unreachable!(),
                    }
                }
            };
            if let Some(task) = task {
                caller.lock().baton.as_mut().unwrap().scheduler.start(task);
            }
            // Even a stall panic at the entry point must wait for the owned task before the
            // native scope joins; joining while holding the baton would strand the new task.
            let point = catch_unwind(AssertUnwindSafe(|| {
                if let Some(task) = task {
                    caller.point(Operation::Isolated(task), span);
                }
            }));
            caller.wait_for(
                Operation::Isolated(task.unwrap_or(0)),
                span,
                None,
                Cancellation::Shielded,
                Some(done),
            );
            handle.join().expect("isolated task runtime panicked");
            if let Err(panic) = point {
                resume_unwind(panic);
            }
            rx.recv().expect("isolated task did not publish its result")
        })
    }

    /// Runs cleanup with enclosing cancellation hidden. Inner scopes keep their own causes.
    pub fn shield<T>(&self, body: impl FnOnce() -> T) -> T {
        let mut context = self.context().unwrap_or_else(Context::detached);
        let id = {
            let mut runtime = context.lock();
            let id = runtime.scope(context.at);
            let state = runtime.scopes.get_mut(&id).unwrap();
            state.shielded = true;
            state.cause = None;
            id
        };
        context.at = Some(Location {
            scope: id,
            child: None,
        });
        let result = {
            let _enter = Enter::new(context.clone());
            catch_unwind(AssertUnwindSafe(body))
        };
        context.lock().finish_scope(id);
        match result {
            Ok(value) => value,
            Err(panic) => resume_unwind(panic),
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
    limits: &'s [Arc<Limit>],
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
    // The scope owns its state even when its handle is called from another real runtime.
    fn owner(self) -> Context {
        if self.simulated {
            Context::simulated(self.runtime)
        } else {
            Context {
                runtime: self.runtime.clone(),
                at: None,
                task: None,
            }
        }
    }

    /// Starts a child that cannot outlive this scope. Dropping its handle does not detach it.
    /// Unlimited spawn is not a checkpoint and still starts a child in a cancelled scope.
    /// A limited spawn waits FIFO for every count together; cancellation drops `f` unrun
    /// and returns a handle whose join fails. A zero count panics before cancellation checks.
    #[track_caller]
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
    where
        F: FnOnce() -> Result<U, E> + Send + 's,
    {
        let span = std::panic::Location::caller();
        assert!(
            !self.limits.iter().any(|limit| limit.is_zero()),
            "spawn under Tasks::limit(0)"
        );
        let current = Context::current().unwrap_or_else(|| self.owner());
        if self.simulated {
            Context::simulated(self.runtime);
        }
        let mut limited_operation = None;
        let slots = if self.limits.is_empty() {
            None
        } else {
            if check_cancellation().is_err() {
                return self.not_started(f, &current, span);
            }
            let slots = {
                let _runtime = current.lock();
                Slots::queue(self.limits)
            };
            let operation = Operation::Limit(slots.blocked_at());
            let cancelled = matches!(
                current.wait_for(
                    operation,
                    span,
                    None,
                    Cancellation::Checkpoint,
                    Some(slots.signal())
                ),
                Wake::Cancelled
            );
            if cancelled || check_cancellation().is_err() {
                let mut runtime = current.lock();
                drop(slots);
                if self.simulated {
                    runtime.wake_waiters();
                }
                drop(runtime);
                return self.not_started(f, &current, span);
            }
            limited_operation = Some(operation);
            Some(slots)
        };
        let caller = self.simulated.then_some(current);
        let (tx, rx) = mpsc::sync_channel(1);
        let done = Arc::new(Signal::new());
        let completion = done.clone();
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
        let run_job: unsafe fn(
            F,
            mpsc::SyncSender<U>,
            Payloads,
            Context,
            Option<Slots>,
            Arc<Signal>,
        ) = run::<F, U, E>;
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
            unsafe { run_job(f, tx, payloads, context, slots, completion) };
        });
        let simulation = if let Some(caller) = caller {
            let task = task.unwrap();
            // Native spawn can unwind before the thread starts. A starting task cannot win
            // the baton, even if dropping the rejected job reaches a scheduling point.
            let started = catch_unwind(AssertUnwindSafe(|| self.threads.spawn(job)));
            let mut runtime = caller.lock();
            let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
            match started {
                Ok(()) => {
                    scheduler.start(task);
                    if let Some(operation) = limited_operation {
                        scheduler.event_result(
                            caller.task.unwrap(),
                            Pending { operation, span },
                            "started",
                        );
                    }
                }
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
            if let Err(panic) = catch_unwind(AssertUnwindSafe(|| self.threads.spawn(job))) {
                let mut runtime = self.runtime.lock().unwrap();
                runtime.finish(at, Outcome::Ok);
                runtime.wake_waiters();
                drop(runtime);
                resume_unwind(panic);
            }
            None
        };
        Task {
            result: Mutex::new(rx),
            cancelled: self.cancelled,
            simulation,
            done,
            _marker: PhantomData,
        }
    }

    fn not_started<U, F>(self, f: F, caller: &Context, span: Span) -> Task<'s, U, E> {
        drop(f);
        let (tx, rx) = mpsc::sync_channel(1);
        drop(tx);
        if let Some(at) = caller.at {
            caller.lock().observe(at);
        }
        let owner = self.owner();
        let mut runtime = owner.lock();
        let error = if runtime.scopes[&self.id].cause.is_some() {
            // Converting E runs user code. Keep it outside locks and before registration.
            drop(runtime);
            let error = (self.cancelled)();
            runtime = owner.lock();
            Some(error)
        } else {
            None
        };
        let at = runtime.spawn(self.id);
        if let Some(error) = error {
            runtime.observe(at);
            runtime.finish(at, Outcome::Error);
            // SAFETY: this Scope carries the matching E and its payload mutex outlives it.
            let payloads = unsafe {
                &*self
                    .payloads
                    .0
                    .cast::<Mutex<Vec<(usize, thread::Result<E>)>>>()
            };
            payloads
                .lock()
                .unwrap()
                .push((at.child.unwrap(), Ok(error)));
        } else {
            runtime.finish(at, Outcome::Echo);
        }
        let simulation = runtime.baton.as_mut().map(|baton| {
            let task = baton.register(span);
            baton.scheduler.abandon(task);
            (self.runtime.clone(), task)
        });
        if let Some(task) = caller.task {
            runtime.baton.as_mut().unwrap().scheduler.event_result(
                task,
                Pending {
                    operation: Operation::Limit(self.limits[0].id),
                    span,
                },
                "not_started",
            );
            runtime.wake_waiters();
        }
        let done = Arc::new(Signal::new());
        done.wake();
        Task {
            result: Mutex::new(rx),
            cancelled: self.cancelled,
            simulation,
            done,
            _marker: PhantomData,
        }
    }

    /// Lowers this scope's deadline. A past deadline cancels immediately; expiry alone is
    /// not failure, but subsequent child errors are reported as `DeadlineExceeded`.
    #[track_caller]
    pub fn set_deadline(self, at: crate::time::Instant) {
        let span = std::panic::Location::caller();
        let caller = self.owner();
        let mut runtime = caller.lock();
        let old = runtime.scopes[&self.id].deadline;
        let at = old.map_or(at, |old| old.min(at));
        runtime.scopes.get_mut(&self.id).unwrap().deadline = Some(at);
        let now = runtime
            .baton
            .as_ref()
            .map_or_else(crate::time::monotonic, |b| b.scheduler.now);
        let active = runtime.scopes[&self.id].cause.is_none();
        let effective = runtime
            .effective_deadline(Location {
                scope: self.id,
                child: None,
            })
            .unwrap();
        if let Some(baton) = &mut runtime.baton {
            baton.scheduler.deadline(
                self.id,
                old.map(|t| t.0),
                (active && at.0 > now).then_some(at.0),
            );
            baton.scheduler.event_result(
                caller.task.unwrap(),
                Pending {
                    operation: Operation::SetDeadline(self.id, effective),
                    span,
                },
                "set",
            );
        }
        if active && at.0 <= now {
            runtime.expire(self.id, caller.task.unwrap_or(0), span);
        }
        if self.simulated {
            runtime.wake_waiters();
        }
        for thread in &runtime.waiting {
            thread.unpark();
        }
    }

    /// Cancels this scope and its descendants. Cancellation alone is not failure, and a child
    /// returning Err after explicit cancellation remains an independent failure.
    /// Cancel is not a scheduling point: the caller's next checkpoint observes it.
    #[track_caller]
    pub fn cancel(self) {
        let caller = self.owner();
        let mut runtime = caller.lock();
        if runtime.cancel(self.id, Cause::Explicit(self.id)) && self.simulated {
            caller.record_cancel(&mut runtime, self.id, std::panic::Location::caller());
        }
    }
}

// Caller must supply the live scope mutex for this E; no reference to E escapes in the job type.
unsafe fn run<F, U, E>(
    f: F,
    tx: mpsc::SyncSender<U>,
    payloads: Payloads,
    context: Context,
    slots: Option<Slots>,
    done: Arc<Signal>,
) where
    F: FnOnce() -> Result<U, E>,
{
    let (outcome, _enter) = run_caught(&context, || match f() {
        Ok(value) => {
            // Buffered values drop on the receiver's thread if it outlives this send.
            let _ = tx.send(value);
            None
        }
        Err(error) => Some(error),
    });
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
    drop(slots);
    drop(tx);
    done.wake();
    if let Some(task) = context.task {
        // Publish cancellation before closing the result channel, and close it before handoff.
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        let pending = scheduler.finish(task, kind);
        if cancelled {
            scheduler.cancel(task, at.scope, pending.span, Cause::Failure(at.scope));
        }
        runtime.wake_waiters();
        runtime.pass(task, pending);
    } else {
        runtime.wake_waiters();
    }
    // Completion is published after cancellation and before a waiter can return.
}

// Shared panic boundary for structured and isolated tasks; handoff stays in their explicit
// completion paths, never in a destructor that could run during user unwinding.
fn run_caught<T>(context: &Context, f: impl FnOnce() -> T) -> (thread::Result<T>, Enter) {
    if let Some(task) = context.task {
        drop(Baton::wait(context.lock(), task));
    }
    let enter = Enter::new(context.clone());
    (catch_unwind(AssertUnwindSafe(f)), enter)
}

/// An isolated task's unwinding panic. The process panic hook still runs normally.
#[derive(Debug)]
pub struct Panicked {
    pub message: String,
}
impl Panicked {
    fn from_payload(payload: Box<dyn std::any::Any + Send>) -> Self {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Box<dyn Any>".to_owned());
        if let Err(second) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            std::mem::forget(second);
            fatal_simulation("panic while dropping an isolated panic payload", "");
        }
        Self { message }
    }
}
impl std::fmt::Display for Panicked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Panicked {}

/// A child's join handle; `Send + Sync` when `U, E: Send` (02 §8.1).
pub struct Task<'s, U, E> {
    result: Mutex<mpsc::Receiver<U>>,
    cancelled: fn() -> E,
    simulation: Option<(Arc<Mutex<Runtime>>, TaskId)>,
    done: Arc<Signal>,
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
        } else {
            let caller = Context::current().unwrap_or_else(Context::detached);
            caller.wait_for(
                Operation::Join(0),
                std::panic::Location::caller(),
                None,
                Cancellation::Shielded,
                Some(self.done.clone()),
            );
        }
        self.result.into_inner().unwrap().recv().map_err(|_| {
            if let Some(context) = Context::current() {
                if let Some(at) = context.at {
                    context.lock().observe_unshielded(at);
                }
            }
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
/// `Clock::sleep` and `Clock::sleep_until`. `?` converts it into `TaskError::Cancelled` and
/// into an `io::Error` of kind `Cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;
impl Cancelled {
    /// Always true: every capability error type answers `is_cancelled()` (reference 6.1).
    pub fn is_cancelled(&self) -> bool {
        true
    }
}
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
// The cancellation a checkpoint observed, returned as the task's error (03 §6.1).
impl From<Cancelled> for TaskError {
    fn from(_: Cancelled) -> Self {
        TaskError::Cancelled
    }
}

/// Task lifecycle errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskError {
    Cancelled,
    /// A child failed after its own scope deadline cancelled it.
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
                assert!(tasks.checkpoint().unwrap_err().is_cancelled());
                assert_eq!(
                    tasks.yield_now().map_err(TaskError::from),
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

#[cfg(test)]
mod control_tests {
    include!("task/control_tests.rs");
}
