// Process adapter (03 §9.9): cancellation, scheduling points, waits and the process sub-stream
// for simulated `Proc` operations, and the hidden tasks that run simulated children's models.
// A model task belongs to no scope, so nothing cancels it, and the existing scheduler alone
// interleaves its actions with application tasks; its delays are operation timers, which count
// for the stall guard. When the simulation ends, a model task still waiting is released and
// stops without touching the scheduler again.
use super::*;

/// The simulation ended while a model task waited.
pub(crate) struct Ended;

#[derive(Clone)]
pub(crate) struct ProcContext(Context);
pub(crate) struct ModelContext(Context);

impl Tasks {
    pub(crate) fn proc_context(&self) -> ProcContext {
        ProcContext(self.context().expect("a simulated Proc needs its simulation"))
    }
    // For drops: the caller's context only inside this live simulation, else nothing to wake.
    pub(crate) fn proc_closing(&self) -> Option<ProcContext> {
        let runtime = self.simulation.as_ref()?;
        let context = Context::current()?;
        if !Arc::ptr_eq(&context.runtime, runtime) || context.lock().simulation_ended {
            return None;
        }
        Some(ProcContext(context))
    }
}

fn draw(context: &Context, bound: u64) -> u64 {
    context.lock().baton.as_mut().unwrap().scheduler.proc_draw(bound)
}

fn record(context: &Context, name: &'static str, id: usize, span: Span, detail: &str) {
    if let Some(task) = context.task {
        context.lock().baton.as_mut().unwrap().scheduler.proc_event(task, name, id, span, detail);
    }
}

impl ProcContext {
    pub(crate) fn check(&self) -> crate::io::Result<()> {
        let mut runtime = self.0.lock();
        if self.0.at.is_some_and(|at| runtime.observe(at)) {
            Err(crate::io::ErrorKind::Cancelled.into())
        } else {
            Ok(())
        }
    }
    /// A scheduling point that observes no cancellation.
    pub(crate) fn point(&self, name: &'static str, id: usize, span: Span) {
        self.0.point(Operation::Proc(name, id), span);
    }
    /// A checkpoint: cancellation before and after one scheduling point.
    pub(crate) fn begin(&self, name: &'static str, id: usize, span: Span) -> crate::io::Result<()> {
        self.check()?;
        self.point(name, id, span);
        self.check()
    }
    pub(crate) fn wait(
        &self,
        name: &'static str,
        id: usize,
        span: Span,
        signal: Arc<Signal>,
        cancellable: bool,
    ) -> crate::io::Result<()> {
        let cancellation = if cancellable { Cancellation::Checkpoint } else { Cancellation::Shielded };
        match self.0.wait_for(Operation::Proc(name, id), span, None, cancellation, Some(signal)) {
            Wake::Cancelled => Err(crate::io::ErrorKind::Cancelled.into()),
            _ => Ok(()),
        }
    }
    pub(crate) fn draw(&self, bound: u64) -> u64 {
        draw(&self.0, bound)
    }
    pub(crate) fn record(&self, name: &'static str, id: usize, span: Span, detail: &str) {
        record(&self.0, name, id, span, detail);
    }
    pub(crate) fn wake(&self) {
        self.0.lock().wake_waiters();
    }

    /// Registers a model task and starts its thread; the task first runs when the scheduler
    /// picks it. Takes no scheduling point itself.
    pub(crate) fn spawn_model(&self, span: Span, model: impl FnOnce(ModelContext) + Send + 'static) {
        let task = {
            let mut runtime = self.0.lock();
            let baton = runtime.baton.as_mut().unwrap();
            let task = baton.register(span);
            baton.models.push(task);
            task
        };
        let context = Context { runtime: self.0.runtime.clone(), at: None, task: Some(task) };
        let started = thread::Builder::new().name("tide-sim-proc".into()).spawn(move || {
            if Baton::wait(context.lock(), task).simulation_ended {
                return;
            }
            let enter = Enter::new(context.clone());
            let _ = catch_unwind(AssertUnwindSafe(|| model(ModelContext(context.clone()))));
            drop(enter);
            let mut runtime = context.lock();
            if runtime.simulation_ended {
                return;
            }
            let pending = runtime.baton.as_mut().unwrap().scheduler.finish(task, Outcome::Ok);
            runtime.wake_waiters();
            runtime.pass(task, pending);
        });
        let mut runtime = self.0.lock();
        let scheduler = &mut runtime.baton.as_mut().unwrap().scheduler;
        match started {
            Ok(_) => scheduler.start(task),
            Err(error) => {
                scheduler.abandon(task);
                drop(runtime);
                panic!("could not start a simulated process: {error}");
            }
        }
    }
}

impl ModelContext {
    fn ended(&self) -> Result<(), Ended> {
        if self.0.lock().simulation_ended { Err(Ended) } else { Ok(()) }
    }
    pub(crate) fn point(&self, name: &'static str, id: usize, span: Span) -> Result<(), Ended> {
        self.ended()?;
        self.0.point(Operation::Proc(name, id), span);
        self.ended()
    }
    /// Waits for `signal` or, with `until`, an operation timer at that virtual instant.
    pub(crate) fn wait(
        &self,
        name: &'static str,
        id: usize,
        span: Span,
        until: Option<Duration>,
        signal: Arc<Signal>,
    ) -> Result<(), Ended> {
        self.ended()?;
        self.0.wait_for(Operation::Proc(name, id), span, until, Cancellation::Shielded, Some(signal));
        self.ended()
    }
    pub(crate) fn now(&self) -> Duration {
        self.0.lock().baton.as_ref().unwrap().scheduler.now
    }
    pub(crate) fn draw(&self, bound: u64) -> u64 {
        draw(&self.0, bound)
    }
    pub(crate) fn record(&self, name: &'static str, id: usize, span: Span, detail: &str) {
        record(&self.0, name, id, span, detail);
    }
    pub(crate) fn wake(&self) {
        self.0.lock().wake_waiters();
    }
}

#[cfg(test)]
impl Tasks {
    /// Lowers §9.3's stall limit so tests can reach the guard quickly.
    pub(crate) fn set_stall_limit(&self, limit: usize) {
        self.context().unwrap().lock().baton.as_mut().unwrap().scheduler.stall_limit = limit;
    }
}
