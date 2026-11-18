// Simulated backend: sim.world.proc's v0 lifecycle subset (03 §9.9; research/40 §§6, 7).
// Models are registered under exact spellings; nothing is resolved and no host process starts.
// Each spawn runs its model on a hidden scheduler task (`task/proc.rs`), so every model action
// is a ready action the existing scheduler interleaves with application tasks, and model delays
// are operation timers. `kill` submits termination as its own ready action. An instance runs
// `Running -> Zombie(status) -> Reaped(status)`; its endpoints are bounded pipes with seeded
// short transfers. A coverage failure (unregistered program, unsupported mode, unmatched
// command or input) stops the seed: every later process operation and the root's drop panic.
use super::{Child as PublicChild, ChildStderr, ChildStdin, ChildStdout, Command, Config, Pipe as PublicPipe, Span};
use crate::io::{self, ErrorKind};
use crate::sim::ProcessModel;
use crate::sim::process::Step;
use crate::task::{ModelContext, ProcContext, Signal, Tasks};
use std::collections::{BTreeMap, VecDeque};
use std::ffi::OsString;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Bytes each simulated pipe direction holds (provisional; recorded in the manifest).
pub(crate) const CAPACITY: usize = 65_536;
/// The process profile and model schema the manifest records (REPLAY-1/2).
pub(crate) const PROFILE: &str = "proc-lifecycle-v0 schema=1";
const SIGKILL: i32 = 9;
const SIGPIPE: i32 = 13;

/// Deliberately broken controls for the seed-witness tests (research/40 §12).
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Broken {
    EofReaps,
    DropOutputOnExit,
    WaitIgnoresDeadline,
    UntimedDelay,
    MissingCreatesInstance,
    KillIgnored,
    DropKills,
    UnregisteredIsNotFound,
    HostClock,
    NoCache,
    SpawnNoPoint,
    KillInline,
    TerminationOverrides,
    FullTransfer,
    SigpipeCode,
    SilentWrite,
    Unbounded,
    TryWaitClosesStdin,
    OutputSkipsCheck,
    NoCleanupWait,
    RecheckAfterWait,
    CancelDropsChild,
    // Injected faults, not controls: the regressions that must surface them.
    ModelPanics,
    OmitManifest,
}
#[cfg(test)]
thread_local! {
    pub(crate) static BROKEN: std::cell::Cell<Option<Broken>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
macro_rules! broken {
    ($state:expr, $b:ident) => {
        $state == Some(Broken::$b)
    };
}
#[cfg(not(test))]
macro_rules! broken {
    ($state:expr, $b:ident) => {{
        let _ = &$state;
        false
    }};
}
#[cfg(test)]
type Control = Option<Broken>;
#[cfg(not(test))]
type Control = ();

pub(crate) struct SimProc {
    models: BTreeMap<OsString, (ProcessModel, Span)>,
    next: usize,
    failure: Arc<Mutex<Option<String>>>,
    pub(crate) control: Control,
}
impl SimProc {
    pub(crate) fn new() -> Self {
        Self {
            models: BTreeMap::new(),
            next: 0,
            failure: Arc::default(),
            #[cfg(test)]
            control: BROKEN.get(),
            #[cfg(not(test))]
            control: (),
        }
    }
    /// The first coverage failure of this simulation, if any.
    pub(crate) fn failure(&self) -> Option<String> {
        self.failure.lock().unwrap().clone()
    }
}

/// `sim.world.proc`: registers a model; takes no point, draw or time; panics on duplicates and
/// malformed models.
#[track_caller]
pub(crate) fn register(proc: &super::Proc, program: &std::ffi::OsStr, model: ProcessModel) {
    let span = std::panic::Location::caller();
    model.validate();
    let tasks = proc.tasks.as_ref().unwrap();
    tasks.check_simulation();
    let mut state = proc.sim.as_ref().unwrap().lock().unwrap();
    assert!(
        !state.models.contains_key(program),
        "sim.world.proc: {} is already registered",
        program.to_string_lossy()
    );
    let data = format!("{PROFILE} capacity={CAPACITY} {model:?}");
    state.models.insert(program.to_owned(), (model, span));
    let control = state.control;
    drop(state);
    if !broken!(control, OmitManifest) {
        tasks.replay_input("proc", program.as_encoded_bytes().to_vec(), data.into_bytes());
    }
}

#[derive(Default)]
struct Stream {
    // Mode of the child's descriptor: a pipe to the parent, null, or a simulated sink.
    sink: Option<&'static str>,
    null: bool,
    buf: VecDeque<u8>,
    writer: bool,
    reader: bool,
    // The model closed its own end; on every route, so later use is a coverage failure.
    closed: bool,
}

struct Instance {
    id: usize,
    status: Option<i32>,
    reaped: bool,
    kill: bool,
    pipes: [Stream; 3],
    parents: Vec<Arc<Signal>>,
    model: Option<Arc<Signal>>,
    failure: Arc<Mutex<Option<String>>>,
    control: Control,
}
type Shared = Arc<Mutex<Instance>>;

impl Instance {
    fn changed(&mut self) {
        for signal in self.parents.drain(..).chain(self.model.take()) {
            signal.wake();
        }
    }
    fn terminate(&mut self, raw: i32) {
        if self.status.is_none() {
            self.status = Some(raw);
            self.pipes[0].reader = false;
            self.pipes[1].writer = false;
            self.pipes[2].writer = false;
            if broken!(self.control, DropOutputOnExit) {
                self.pipes[1].buf.clear();
                self.pipes[2].buf.clear();
            }
        }
        self.changed();
    }
    fn failed(&self) -> Option<String> {
        self.failure.lock().unwrap().clone()
    }
    fn room(&self, slot: usize) -> usize {
        if broken!(self.control, Unbounded) {
            usize::MAX
        } else {
            CAPACITY - self.pipes[slot].buf.len()
        }
    }
}

fn label(id: usize) -> String {
    format!("main:0:{id}")
}

#[track_caller]
fn coverage(failure: &Mutex<Option<String>>, report: String) -> ! {
    failure.lock().unwrap().get_or_insert_with(|| report.clone());
    panic!("{report}");
}

pub(super) fn spawn(
    sim: &Arc<Mutex<SimProc>>,
    tasks: &Tasks,
    cmd: &Command,
    stdio: [Config; 3],
    span: Span,
) -> io::Result<PublicChild> {
    let ctx = tasks.proc_context();
    let (model, at, id, failure, control) = {
        let mut state = sim.lock().unwrap();
        if let Some(f) = state.failure() {
            drop(state);
            panic!("{f}");
        }
        let program = cmd.program.to_string_lossy().into_owned();
        let report = |what: String| {
            format!("unmodelled effect: proc.spawn of `{program}` at {span}: {what}; under simulation a program runs only as a sim.world.proc model")
        };
        let Some((model, at)) = state.models.get(&cmd.program).cloned() else {
            if broken!(state.control, UnregisteredIsNotFound) {
                return Err(ErrorKind::NotFound.into());
            }
            let f = state.failure.clone();
            drop(state);
            coverage(&f, report("no model is registered for this spelling".into()));
        };
        if model.missing {
            if broken!(state.control, MissingCreatesInstance) {
                state.next += 1;
            }
            drop(state);
            ctx.record("proc.spawn", 0, span, &format!("{program}: not found"));
            return Err(io::Error::new(ErrorKind::NotFound, "program not found (ProcessModel::missing)"));
        }
        for (slot, config) in stdio.iter().enumerate() {
            let what = match config {
                Config::Inherit if slot == 0 => Some("inherited stdin"),
                Config::Fd(_) => Some("a real descriptor"),
                Config::Unsupported(what) => Some(*what),
                _ => None,
            };
            if let Some(what) = what {
                let f = state.failure.clone();
                drop(state);
                coverage(&f, report(format!("{what} is not a supported mode in the v0 simulator")));
            }
        }
        let mut env = BTreeMap::new();
        if !cmd.env_clear {
            tasks.world_inputs(|inputs| env = inputs.env.clone());
        }
        for (k, v) in &cmd.env {
            match v {
                Some(v) => env.insert(k.clone(), v.clone()),
                None => env.remove(k),
            };
        }
        let cwd = cmd.cwd.clone().unwrap_or_else(|| "/".into());
        let mismatch = if model.args.as_ref().is_some_and(|a| *a != cmd.args) {
            Some(format!("arguments {:?} do not match the model's", cmd.args))
        } else if let Some((k, _)) = model.env.iter().find(|(k, v)| env.get(k) != Some(v)) {
            Some(format!("environment variable {} does not match the model's", k.to_string_lossy()))
        } else if model.cwd.as_ref().is_some_and(|d| *d != cwd) {
            Some(format!("directory {} does not match the model's", cwd.display()))
        } else {
            None
        };
        if let Some(what) = mismatch {
            let f = state.failure.clone();
            drop(state);
            coverage(&f, report(what));
        }
        let id = state.next;
        state.next += 1;
        (model, at, id, state.failure.clone(), state.control)
    };
    let mut detail = format!("{}: args={:?} cwd={}", cmd.program.to_string_lossy(), cmd.args, cmd.cwd.as_deref().unwrap_or("/".as_ref()).display());
    if broken!(control, HostClock) {
        detail.push_str(&format!(" host={:?}", std::time::SystemTime::now()));
    }
    let mut pipes: [Stream; 3] = Default::default();
    for (slot, config) in stdio.iter().enumerate() {
        let stream = &mut pipes[slot];
        match config {
            Config::Piped => {
                stream.reader = true;
                stream.writer = true;
            }
            Config::Null => stream.null = true,
            Config::Inherit => stream.sink = Some(if slot == 1 { "out" } else { "err" }),
            Config::Parent(fd) => stream.sink = Some(if *fd == 1 { "out" } else { "err" }),
            Config::Fd(_) | Config::Unsupported(_) => unreachable!("refused above"),
        }
    }
    let inst: Shared = Arc::new(Mutex::new(Instance {
        id,
        status: None,
        reaped: false,
        kill: false,
        pipes,
        parents: Vec::new(),
        model: None,
        failure,
        control,
    }));
    ctx.record("proc.spawn", id, span, &detail);
    let model_inst = inst.clone();
    let ignores = model.ignores_sigpipe;
    ctx.spawn_model(at, move |mc| {
        let flow = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if broken!(control, ModelPanics) {
                panic!("injected model panic");
            }
            run(&mc, &model_inst, &model.steps, at, ignores)
        }));
        match flow {
            Ok(Ok(Flow::Continue)) => exit(&mc, &model_inst, 0, at),
            Ok(_) => {}
            // An internal panic fails the seed like any coverage failure; never a silent success.
            Err(panic) => {
                let message = panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or(panic.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string payload");
                fail(&mc, &model_inst, at, format!("the model panicked: {message}"));
            }
        }
    });
    let end = |slot: usize| {
        matches!(stdio[slot], Config::Piped).then(|| PublicPipe::Sim(Pipe { inst: inst.clone(), tasks: tasks.clone(), slot }))
    };
    let child = PublicChild {
        stdin: end(0).map(ChildStdin),
        stdout: end(1).map(ChildStdout),
        stderr: end(2).map(ChildStderr),
        inner: super::Inner::Sim(Child { inst: inst.clone(), tasks: tasks.clone(), status: None }),
    };
    // Creation is atomic; the scheduler may run the child, even to exit, before the caller resumes.
    if !broken!(control, SpawnNoPoint) {
        ctx.point("proc.spawn", id, span);
    }
    Ok(child)
}

// Short transfers: a size in 1..=n from the process sub-stream; no draw when n is 1.
fn short(draw: impl FnOnce(u64) -> u64, n: usize, control: Control) -> usize {
    if n <= 1 || broken!(control, FullTransfer) { n } else { 1 + draw(n as u64) as usize }
}

/// Waits until `ready` holds for the instance, checking its coverage failure first.
fn wait_on<R>(
    inst: &Shared,
    ctx: &ProcContext,
    name: &'static str,
    span: Span,
    cancellable: bool,
    mut ready: impl FnMut(&mut Instance) -> Option<R>,
) -> io::Result<R> {
    loop {
        let mut st = inst.lock().unwrap();
        if let Some(f) = st.failed() {
            drop(st);
            panic!("{f}");
        }
        if let Some(r) = ready(&mut st) {
            return Ok(r);
        }
        let signal = Arc::new(Signal::new());
        st.parents.push(signal.clone());
        let id = st.id;
        drop(st);
        ctx.wait(name, id, span, signal, cancellable)?;
    }
}

// Handles keep their simulation, not a context: each operation runs in its caller's scope.
pub(super) struct Child {
    inst: Shared,
    tasks: Tasks,
    status: Option<ExitStatus>,
}
impl Child {
    fn ctx(&self) -> ProcContext {
        self.tasks.proc_context()
    }
    pub(super) fn id(&self) -> u32 {
        // Simulated data: the spawn ordinal plus one, never a host PID.
        self.inst.lock().unwrap().id as u32 + 1
    }
    // Zombie -> Reaped once; later calls return the cached status.
    fn reap(&mut self, st: &mut Instance) -> Option<io::Result<ExitStatus>> {
        if let Some(status) = self.status.filter(|_| !broken!(st.control, NoCache)) {
            return Some(Ok(status));
        }
        let raw = st.status?;
        if st.reaped {
            return Some(Err(io::Error::new(ErrorKind::Other, "no child process (already reaped)")));
        }
        st.reaped = true;
        let status = ExitStatus::from_raw(raw);
        self.status = Some(status);
        Some(Ok(status))
    }
    pub(super) fn try_wait(&mut self, span: Span) -> io::Result<Option<ExitStatus>> {
        self.ctx();
        let inst = self.inst.clone();
        let mut st = inst.lock().unwrap();
        if let Some(f) = st.failed() {
            drop(st);
            panic!("{f}");
        }
        if broken!(st.control, TryWaitClosesStdin) {
            st.pipes[0].writer = false;
            st.changed();
        }
        if let Some(result) = self.reap(&mut st) {
            return result.map(Some);
        }
        let id = st.id;
        drop(st);
        self.ctx().wake();
        // Polling is a scheduling point, so a try_wait spin counts for the stall guard.
        self.ctx().point("proc.try_wait", id, span);
        Ok(None)
    }
    pub(super) fn wait(&mut self, span: Span, cancellable: bool) -> io::Result<ExitStatus> {
        let (id, control) = {
            let st = self.inst.lock().unwrap();
            (st.id, st.control)
        };
        let cancellable = cancellable && !broken!(control, WaitIgnoresDeadline);
        if cancellable {
            self.ctx().begin("proc.wait", id, span)?;
        }
        let inst = self.inst.clone();
        let ctx = self.ctx();
        let result = wait_on(&inst, &ctx, "proc.wait", span, cancellable, |st| self.reap(st));
        match result {
            Ok(result) => {
                if cancellable && broken!(control, RecheckAfterWait) {
                    ctx.begin("proc.wait", id, span)?;
                }
                result
            }
            Err(e) => {
                if broken!(control, CancelDropsChild) {
                    inst.lock().unwrap().reaped = true;
                }
                Err(e)
            }
        }
    }
    #[cfg(test)]
    pub(super) fn control(&self) -> Control {
        self.inst.lock().unwrap().control
    }
    pub(super) fn kill(&mut self, span: Span) -> io::Result<()> {
        self.ctx();
        let mut st = self.inst.lock().unwrap();
        if st.status.is_some() || st.kill || broken!(st.control, KillIgnored) {
            return Ok(());
        }
        st.kill = true;
        let id = st.id;
        if broken!(st.control, KillInline) {
            st.terminate(SIGKILL);
            drop(st);
            self.ctx().wake();
            return Ok(());
        }
        drop(st);
        self.ctx().record("proc.kill", id, span, "SIGKILL submitted");
        // Termination is a ready action of its own, on the existing scheduler.
        let inst = self.inst.clone();
        self.ctx().spawn_model(span, move |mc| {
            let mut st = inst.lock().unwrap();
            if st.status.is_none() {
                st.terminate(SIGKILL);
                drop(st);
                mc.record("proc.terminate", id, span, "signal 9");
            }
            mc.wake();
        });
        Ok(())
    }
    pub(super) fn drain(
        &mut self,
        out: Option<ChildStdout>,
        err: Option<ChildStderr>,
        span: Span,
    ) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let id = self.inst.lock().unwrap().id;
        self.ctx().begin("proc.read", id, span)?;
        // Read the supplied ends, which may have moved between children: each reads its own
        // instance and slot, both at once, so neither can fill and stall its writer.
        // Each end must belong to this live simulation, as its own reads require; this panics
        // otherwise, before the end is inspected, registered or changed.
        let pipe = |p: PublicPipe| match p {
            PublicPipe::Sim(p) => {
                p.ctx();
                p
            }
            PublicPipe::Real(_) => panic!("simulation handle used outside its simulation: a real pipe on a simulated child"),
        };
        let mut ends = [(out.map(|o| pipe(o.0)), Vec::new()), (err.map(|e| pipe(e.0)), Vec::new())];
        let control = self.inst.lock().unwrap().control;
        let ctx = self.ctx();
        loop {
            let open: Vec<usize> = ends.iter().enumerate().filter(|(_, e)| e.0.is_some()).map(|(i, _)| i).collect();
            if open.is_empty() {
                break;
            }
            let chunks = loop {
                let signal = Arc::new(Signal::new());
                let mut chunks = Vec::new();
                for &i in &open {
                    let p = ends[i].0.as_ref().unwrap();
                    let mut st = p.inst.lock().unwrap();
                    if let Some(f) = st.failed() {
                        drop(st);
                        panic!("{f}");
                    }
                    let s = &st.pipes[p.slot];
                    if !s.buf.is_empty() || !s.writer {
                        chunks.push((i, s.buf.len()));
                    } else {
                        st.parents.push(signal.clone());
                    }
                }
                if !chunks.is_empty() {
                    break chunks;
                }
                ctx.wait("proc.read", id, span, signal, true)?;
            };
            for (i, available) in chunks {
                if available == 0 {
                    ends[i].0 = None; // EOF: dropping the end closes it.
                    continue;
                }
                let n = short(|b| ctx.draw(b), available, control);
                let p = ends[i].0.as_ref().unwrap();
                let mut st = p.inst.lock().unwrap();
                ends[i].1.extend(st.pipes[p.slot].buf.drain(..n));
                st.changed();
            }
            ctx.wake();
        }
        let [(_, out), (_, err)] = ends;
        Ok((out, err))
    }
}

#[cfg(test)]
impl Drop for Child {
    fn drop(&mut self) {
        let mut st = self.inst.lock().unwrap_or_else(|e| e.into_inner());
        if broken!(st.control, DropKills) {
            st.terminate(SIGKILL);
            drop(st);
            if let Some(c) = self.tasks.proc_closing() {
                c.wake();
            }
        }
    }
}

pub(super) struct Pipe {
    inst: Shared,
    tasks: Tasks,
    slot: usize,
}
impl Pipe {
    fn ctx(&self) -> ProcContext {
        self.tasks.proc_context()
    }
    pub(super) fn read(&self, buf: &mut [u8], span: Span) -> io::Result<usize> {
        let (id, control) = {
            let st = self.inst.lock().unwrap();
            (st.id, st.control)
        };
        self.ctx().begin("proc.read", id, span)?;
        if buf.is_empty() {
            return Ok(0);
        }
        let slot = self.slot;
        let available = wait_on(&self.inst, &self.ctx(), "proc.read", span, true, |st| {
            let s = &st.pipes[slot];
            (!s.buf.is_empty() || !s.writer).then_some(s.buf.len())
        })?;
        if available == 0 {
            if broken!(control, EofReaps) {
                self.inst.lock().unwrap().status.get_or_insert(0);
            }
            return Ok(0);
        }
        let n = short(|b| self.ctx().draw(b), available.min(buf.len()), control);
        let mut st = self.inst.lock().unwrap();
        for (dst, src) in buf.iter_mut().zip(st.pipes[slot].buf.drain(..n)) {
            *dst = src;
        }
        st.changed();
        drop(st);
        self.ctx().wake();
        Ok(n)
    }
    pub(super) fn write(&self, buf: &[u8], span: Span) -> io::Result<usize> {
        let (id, control) = {
            let st = self.inst.lock().unwrap();
            (st.id, st.control)
        };
        self.ctx().begin("proc.write", id, span)?;
        let room = wait_on(&self.inst, &self.ctx(), "proc.write", span, true, |st| {
            if !st.pipes[0].reader {
                return Some(None);
            }
            let room = st.room(0);
            (room > 0 || buf.is_empty()).then_some(Some(room))
        })?;
        let Some(room) = room else {
            if broken!(control, SilentWrite) {
                return Ok(buf.len());
            }
            return Err(io::Error::from(ErrorKind::BrokenPipe));
        };
        if buf.is_empty() {
            return Ok(0);
        }
        let n = short(|b| self.ctx().draw(b), buf.len().min(room), control);
        let mut st = self.inst.lock().unwrap();
        st.pipes[0].buf.extend(&buf[..n]);
        st.changed();
        drop(st);
        self.ctx().wake();
        Ok(n)
    }
}
impl Drop for Pipe {
    fn drop(&mut self) {
        let mut st = self.inst.lock().unwrap_or_else(|e| e.into_inner());
        if self.slot == 0 {
            st.pipes[0].writer = false;
        } else {
            st.pipes[self.slot].reader = false;
        }
        st.changed();
        drop(st);
        // A blocked model may now run: tell the scheduler, when this drop runs in the simulation.
        if let Some(c) = self.tasks.proc_closing() {
            c.wake();
        }
    }
}

enum Flow {
    Continue,
    Stop,
}

// The model's next action may run only while its instance runs.
fn stopped(inst: &Shared) -> bool {
    let st = inst.lock().unwrap();
    st.status.is_some() || st.failed().is_some()
}

fn fail(mc: &ModelContext, inst: &Shared, at: Span, what: String) -> Flow {
    let mut st = inst.lock().unwrap_or_else(|e| e.into_inner());
    let report = format!(
        "unmodelled effect: instance {} of the model registered at {at}: {what}",
        label(st.id)
    );
    st.failure.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert(report);
    st.changed();
    drop(st);
    mc.wake();
    Flow::Stop
}

// A normal exit only from running: a kill that terminated the instance while the model was
// handed off leaves one terminal event, its own, matching the status a wait collects.
fn exit(mc: &ModelContext, inst: &Shared, code: i32, at: Span) {
    let mut st = inst.lock().unwrap();
    if st.status.is_some() {
        return;
    }
    let raw = if st.kill && broken!(st.control, TerminationOverrides) { SIGKILL } else { code << 8 };
    let id = st.id;
    st.terminate(raw);
    drop(st);
    mc.record("proc.exit", id, at, &format!("code {code}"));
    mc.wake();
}

// Blocks the model until the instance changes, then takes no extra point.
fn block(mc: &ModelContext, inst: &Shared, name: &'static str, at: Span, until: Option<Duration>) -> Result<(), crate::task::Ended> {
    let signal = Arc::new(Signal::new());
    let id = {
        let mut st = inst.lock().unwrap();
        st.model = Some(signal.clone());
        st.id
    };
    mc.wait(name, id, at, until, signal)
}

fn run(mc: &ModelContext, inst: &Shared, steps: &[Step], at: Span, ignores: bool) -> Result<Flow, crate::task::Ended> {
    let (id, control) = {
        let st = inst.lock().unwrap();
        (st.id, st.control)
    };
    for step in steps {
        if stopped(inst) {
            return Ok(Flow::Stop);
        }
        match step {
            Step::Delay(min, max) => {
                let span = (*max - *min).as_nanos().min(u64::MAX as u128 - 1) as u64;
                let d = *min + Duration::from_nanos(if span == 0 { 0 } else { mc.draw(span + 1) });
                let Some(until) = mc.now().checked_add(d) else {
                    return Ok(fail(mc, inst, at, format!("a delay of {d:?} passes the end of virtual time")));
                };
                while mc.now() < until {
                    if stopped(inst) {
                        return Ok(Flow::Stop);
                    }
                    let timer = (!broken!(control, UntimedDelay)).then_some(until);
                    block(mc, inst, "proc.delay", at, timer)?;
                }
            }
            Step::Write(slot, bytes) => {
                let slot = *slot as usize;
                let mut rest = &bytes[..];
                while !rest.is_empty() {
                    let mut st = inst.lock().unwrap();
                    if st.status.is_some() {
                        return Ok(Flow::Stop);
                    }
                    let s = &st.pipes[slot];
                    if s.closed {
                        drop(st);
                        return Ok(fail(mc, inst, at, format!("write after the model closed descriptor {slot}")));
                    }
                    if let Some(stream) = s.sink {
                        drop(st);
                        crate::__sink::bytes(stream, rest);
                        break;
                    }
                    if s.null {
                        break;
                    }
                    if !s.reader {
                        if ignores {
                            break; // BrokenPipe; the model continues.
                        }
                        let raw = if broken!(control, SigpipeCode) { (128 + SIGPIPE) << 8 } else { SIGPIPE };
                        st.terminate(raw);
                        drop(st);
                        mc.record("proc.exit", id, at, "signal 13");
                        mc.wake();
                        return Ok(Flow::Stop);
                    }
                    let room = st.room(slot);
                    if room == 0 {
                        drop(st);
                        block(mc, inst, "proc.write", at, None)?;
                        continue;
                    }
                    drop(st);
                    let n = short(|b| mc.draw(b), rest.len().min(room), control);
                    let mut st = inst.lock().unwrap();
                    st.pipes[slot].buf.extend(&rest[..n]);
                    st.changed();
                    drop(st);
                    mc.wake();
                    rest = &rest[n..];
                    if !rest.is_empty() {
                        mc.point("proc.write", id, at)?;
                    }
                }
            }
            Step::Read(prefix) => {
                let mut rest = &prefix[..];
                while !rest.is_empty() {
                    let st = inst.lock().unwrap();
                    if st.status.is_some() {
                        return Ok(Flow::Stop);
                    }
                    let s = &st.pipes[0];
                    if s.closed {
                        drop(st);
                        return Ok(fail(mc, inst, at, "read after the model closed descriptor 0".into()));
                    }
                    if s.buf.is_empty() {
                        if !s.writer || s.null {
                            drop(st);
                            return Ok(fail(mc, inst, at, "stdin closed before the model's expected input".into()));
                        }
                        drop(st);
                        block(mc, inst, "proc.read", at, None)?;
                        continue;
                    }
                    let available = s.buf.len().min(rest.len());
                    drop(st);
                    let n = short(|b| mc.draw(b), available, control);
                    let mut st = inst.lock().unwrap();
                    let got: Vec<u8> = st.pipes[0].buf.drain(..n).collect();
                    st.changed();
                    drop(st);
                    mc.wake();
                    if got != rest[..n] {
                        return Ok(fail(mc, inst, at, format!("unmatched input {:?}", String::from_utf8_lossy(&got))));
                    }
                    rest = &rest[n..];
                }
            }
            Step::ReadEof | Step::Copy => loop {
                let st = inst.lock().unwrap();
                if st.status.is_some() {
                    return Ok(Flow::Stop);
                }
                let s = &st.pipes[0];
                if s.closed {
                    drop(st);
                    return Ok(fail(mc, inst, at, "read after the model closed descriptor 0".into()));
                }
                if s.buf.is_empty() {
                    if !s.writer || s.null {
                        break;
                    }
                    drop(st);
                    block(mc, inst, "proc.read", at, None)?;
                    continue;
                }
                if matches!(step, Step::ReadEof) {
                    drop(st);
                    return Ok(fail(mc, inst, at, "unexpected input before end of stdin".into()));
                }
                let available = s.buf.len();
                drop(st);
                let n = short(|b| mc.draw(b), available, control);
                let mut st = inst.lock().unwrap();
                let chunk: Vec<u8> = st.pipes[0].buf.drain(..n).collect();
                st.changed();
                drop(st);
                mc.wake();
                if let Flow::Stop = run(mc, inst, &[Step::Write(1, chunk)], at, ignores)? {
                    return Ok(Flow::Stop);
                }
            },
            Step::Close(slot) => {
                let mut st = inst.lock().unwrap();
                st.pipes[*slot as usize].closed = true;
                match slot {
                    0 => st.pipes[0].reader = false,
                    s => st.pipes[*s as usize].writer = false,
                }
                st.changed();
                drop(st);
                mc.wake();
            }
            Step::Choose(alternatives) => {
                let pick = if alternatives.len() == 1 { 0 } else { mc.draw(alternatives.len() as u64) as usize };
                mc.record("proc.choose", id, at, &format!("alternative {pick}"));
                if let Flow::Stop = run(mc, inst, &alternatives[pick], at, ignores)? {
                    return Ok(Flow::Stop);
                }
                continue;
            }
            Step::Exit(code) => {
                exit(mc, inst, *code, at);
                return Ok(Flow::Stop);
            }
        }
        // Each step is one ready action: the scheduler may run others before the next.
        mc.point("proc.step", id, at)?;
    }
    Ok(Flow::Continue)
}
