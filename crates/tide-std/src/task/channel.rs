// Channel adapter to the one runtime wait/trace owner. Resource locks are acquired only
// inside `with`/`close`, after the runtime lock, and released before any point or wait.
use super::*;
use scheduler::ChannelEvent;

pub(crate) struct Channel {
    simulation: Option<Arc<Mutex<Runtime>>>,
    id: usize,
}

impl Channel {
    pub(crate) fn new(tasks: &Tasks, bound: Option<usize>, span: Span) -> Self {
        let context = tasks.context();
        let id = context
            .as_ref()
            .filter(|c| c.task.is_some())
            .map_or(0, |c| {
                let mut runtime = c.lock();
                let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
                let id = scheduler.allocate_channel();
                scheduler.channel(c.task.unwrap(), id, span, ChannelEvent::New(bound));
                id
            });
        Self {
            simulation: tasks.simulation.clone(),
            id,
        }
    }

    pub(crate) fn context(&self) -> Context {
        Context::of(&self.simulation).unwrap_or_else(Context::detached)
    }

    pub(crate) fn begin(&self, name: &'static str, span: Span) -> Context {
        let context = self.context();
        if context.task.is_some() {
            context.point(Operation::Channel(name, self.id), span);
        }
        context
    }

    pub(crate) fn with<T>(
        &self,
        context: &Context,
        f: impl FnOnce(&mut dyn FnMut() -> bool) -> T,
    ) -> T {
        let mut runtime = context.lock();
        let result = f(&mut || context.at.is_some_and(|at| runtime.observe(at)));
        if context.task.is_some() {
            runtime.wake_waiters();
        }
        result
    }

    pub(crate) fn wait(
        &self,
        context: &Context,
        send: bool,
        span: Span,
        until: Option<Duration>,
        signal: Arc<Signal>,
    ) -> bool {
        matches!(
            context.wait_for(
                Operation::Channel(if send { "send" } else { "recv" }, self.id),
                span,
                until,
                Cancellation::Deferred,
                Some(signal),
            ),
            Wake::Timeout
        )
    }

    pub(crate) fn record(
        &self,
        context: &Context,
        name: &'static str,
        span: Span,
        result: &'static str,
        until: Option<Duration>,
    ) {
        if let Some(task) = context.task {
            let mut runtime = context.lock();
            runtime.baton.as_mut().unwrap().scheduler.channel(
                task,
                self.id,
                span,
                ChannelEvent::Result {
                    name,
                    result,
                    until,
                },
            );
        }
    }

    // Destructors do not validate a current simulation or take a point. Handles may outlive
    // their simulation, or drop during unwinding. Signals also wake independent real runtimes.
    pub(crate) fn close<T>(
        &self,
        end: &'static str,
        span: Span,
        f: impl FnOnce() -> (T, Option<usize>),
    ) -> T {
        let Some(runtime) = &self.simulation else {
            return f().0;
        };
        let mut runtime = sim_lock(runtime.lock());
        let (result, senders) = f();
        let task = Context::current()
            .filter(|c| Arc::ptr_eq(&c.runtime, self.simulation.as_ref().unwrap()))
            .and_then(|c| c.task)
            .unwrap_or(0);
        runtime.baton.as_mut().unwrap().scheduler.channel(
            task,
            self.id,
            span,
            ChannelEvent::Close { end, senders },
        );
        runtime.wake_waiters();
        result
    }
}
