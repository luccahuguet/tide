// Processes (03 §1.1 PROC-ONLY, §9.9; research/40-proc-command-v0.md): `Proc::command` and
// `only`, `Command`, `Child`, its pipe ends, and `Stdio`, with Rust's names and receivers
// except that there is no ambient `Command::new` and `wait_with_output` takes `&mut self`.
// The real backend (`process/real.rs`) is Linux `std::process` behind std-owned lookup,
// canonical exec and a fail-closed descriptor sweep; the simulated backend (`process/sim.rs`)
// runs `sim.world.proc` lifecycle models on the seeded scheduler. Both run one refusal order.
// `tide_std::process`'s public items are fixed by tide-admit's PROC-3 item-list test.
use crate::ffi::{OsStr, OsString};
use crate::io;
use crate::task::Tasks;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
pub use std::process::{ExitCode, ExitStatus, Output, Termination};
#[path = "process/real.rs"]
mod real;
#[path = "process/sim.rs"]
pub(crate) mod sim;
type Span = &'static std::panic::Location<'static>;

/// Authority to start processes (03 §1.1). A child is not capability-checked: it runs with at
/// least this process's operating-system authority, and keeps running after the call that
/// started it returns. `only` bounds which program image starts, not what it does.
#[derive(Clone)]
pub struct Proc {
    pub(crate) tasks: Option<Tasks>,
    only: Option<Arc<[OsString]>>,
    pub(crate) sim: Option<Arc<Mutex<sim::SimProc>>>,
}

impl Proc {
    pub(crate) fn new(tasks: Option<Tasks>) -> Self {
        Self {
            sim: tasks.as_ref().map(|_| Arc::new(Mutex::new(sim::SimProc::new()))),
            tasks,
            only: None,
        }
    }

    /// A command for `program`. Making one cannot fail; `spawn`, `output` and `status` check it.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        Command {
            proc: self.clone(),
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            env: BTreeMap::new(),
            env_clear: false,
            cwd: None,
            stdio: [None, None, None],
        }
    }

    /// PROC-ONLY: allows only programs spelled exactly as an entry (a bare name or an absolute
    /// path), byte for byte. Empty entries, NUL, and relative paths with `/` grant nothing;
    /// lists intersect. Allowing a shell, interpreter or any program that runs what it is told
    /// is full authority.
    pub fn only(&self, programs: impl IntoIterator<Item: AsRef<OsStr>>) -> Proc {
        let programs: Vec<OsString> = programs
            .into_iter()
            .map(|p| p.as_ref().to_owned())
            .filter(|p| valid_entry(p))
            .filter(|p| self.only.as_ref().is_none_or(|old| old.contains(p)))
            .collect();
        Proc {
            only: Some(programs.into()),
            ..self.clone()
        }
    }
}

fn valid_entry(p: &OsStr) -> bool {
    let b = p.as_encoded_bytes();
    !b.is_empty() && !b.contains(&0) && (!b.contains(&b'/') || b[0] == b'/')
}

// glibc 2.39 `UNSECURE_ENVVARS` (sysdeps/generic/unsecvars.h) beyond its fourteen `LD_` names,
// which the prefix rule covers.
const UNSECURE: [&str; 20] = [
    "GCONV_PATH", "GETCONF_DIR", "GLIBC_TUNABLES", "HOSTALIASES", "LOCALDOMAIN", "LOCPATH",
    "MALLOC_ARENA_MAX", "MALLOC_ARENA_TEST", "MALLOC_MMAP_MAX_", "MALLOC_MMAP_THRESHOLD_",
    "MALLOC_PERTURB_", "MALLOC_TOP_PAD_", "MALLOC_TRACE", "MALLOC_TRIM_THRESHOLD_", "NIS_PATH",
    "NLSPATH", "RESOLV_HOST_CONF", "RES_OPTIONS", "TMPDIR", "TZDIR",
];

// Refused under `only`. A key holding `=` is refused too: no variable's name holds one, and the
// child's C library would read the bytes before it as the name (`GCONV_PATH=/x` sets
// `GCONV_PATH`), so such a key could smuggle in any refused variable.
fn refused_variable(name: &OsStr) -> bool {
    let b = name.as_encoded_bytes();
    b.contains(&b'=')
        || b.starts_with(b"LD_")
        || b.starts_with(b"DYLD_")
        || UNSECURE.iter().any(|u| u.as_bytes() == b)
}

/// The authority the process may start with and how it is configured. A `Command` carries the
/// authority of the `Proc` that made it.
pub struct Command {
    proc: Proc,
    program: OsString,
    args: Vec<OsString>,
    // Rust's CommandEnv: explicit sets (`Some`) and removals (`None`) after an optional clear.
    env: BTreeMap<OsString, Option<OsString>>,
    env_clear: bool,
    cwd: Option<PathBuf>,
    stdio: [Option<Stdio>; 3],
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("program", &self.program)
            .field("args", &self.args)
            .finish_non_exhaustive()
    }
}

impl Command {
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Command {
        self.args.push(arg.as_ref().to_owned());
        self
    }
    pub fn args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Command {
        self.args.extend(args.into_iter().map(|a| a.as_ref().to_owned()));
        self
    }
    pub fn env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> &mut Command {
        self.env.insert(key.as_ref().to_owned(), Some(val.as_ref().to_owned()));
        self
    }
    pub fn envs(
        &mut self,
        vars: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
    ) -> &mut Command {
        for (k, v) in vars {
            self.env(k, v);
        }
        self
    }
    pub fn env_remove(&mut self, key: impl AsRef<OsStr>) -> &mut Command {
        if self.env_clear {
            self.env.remove(key.as_ref());
        } else {
            self.env.insert(key.as_ref().to_owned(), None);
        }
        self
    }
    pub fn env_clear(&mut self) -> &mut Command {
        self.env.clear();
        self.env_clear = true;
        self
    }
    /// As in Rust; it needs no `Fs` and cannot change which program starts, since lookup and
    /// canonical resolution happen in the parent first.
    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Command {
        self.cwd = Some(dir.as_ref().to_owned());
        self
    }
    pub fn stdin(&mut self, cfg: impl Into<Stdio>) -> &mut Command {
        self.stdio[0] = Some(cfg.into());
        self
    }
    pub fn stdout(&mut self, cfg: impl Into<Stdio>) -> &mut Command {
        self.stdio[1] = Some(cfg.into());
        self
    }
    pub fn stderr(&mut self, cfg: impl Into<Stdio>) -> &mut Command {
        self.stdio[2] = Some(cfg.into());
        self
    }

    /// Starts the process. Not a checkpoint: a permitted spawn in a cancelled scope starts it.
    /// Unset stdio is inherited.
    #[track_caller]
    pub fn spawn(&mut self) -> io::Result<Child> {
        self.start([Fallback::Inherit; 3], std::panic::Location::caller())
    }

    /// Runs to exit, capturing stdout and stderr (unset: piped) with stdin null by default.
    /// A checkpoint; a cancelled or otherwise failed call kills its child, waits for it
    /// shielded, and returns the error (`Cancelled` when cancelled).
    #[track_caller]
    pub fn output(&mut self) -> io::Result<Output> {
        let span = std::panic::Location::caller();
        self.checkpoint()?;
        let mut child = self.start([Fallback::Null, Fallback::Piped, Fallback::Piped], span)?;
        child.finish(span, true)
    }

    /// Runs to exit with stdio inherited by default. A checkpoint, cancelled as `output` is.
    #[track_caller]
    pub fn status(&mut self) -> io::Result<ExitStatus> {
        let span = std::panic::Location::caller();
        self.checkpoint()?;
        let mut child = self.start([Fallback::Inherit; 3], span)?;
        child.finish(span, false).map(|o| o.status)
    }

    fn checkpoint(&self) -> io::Result<()> {
        #[cfg(test)]
        if self.proc.sim.as_ref().is_some_and(|s| s.lock().unwrap().control == Some(sim::Broken::OutputSkipsCheck)) {
            return Ok(());
        }
        match &self.proc.tasks {
            Some(tasks) => tasks.proc_context().check(),
            None => {
                crate::task::outside_simulation();
                if crate::task::observe_cancellation() {
                    Err(io::ErrorKind::Cancelled.into())
                } else {
                    Ok(())
                }
            }
        }
    }

    // PROC-ONLY's order, shared by both backends: program, refused variables, NUL bytes.
    fn admit(&self) -> io::Result<()> {
        if let Some(only) = &self.proc.only {
            if !only.iter().any(|p| *p == self.program) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "program not allowed by Proc::only",
                ));
            }
            if let Some(k) = self.env.iter().find_map(|(k, v)| v.as_ref().and(Some(k)).filter(|k| refused_variable(k))) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("{} is refused under Proc::only", k.to_string_lossy()),
                ));
            }
        }
        let nul = |s: &OsStr| s.as_encoded_bytes().contains(&0);
        if nul(&self.program)
            || self.args.iter().any(|a| nul(a))
            || self.env.iter().any(|(k, v)| nul(k) || v.as_deref().is_some_and(nul))
            || self.cwd.as_ref().is_some_and(|d| nul(d.as_os_str()))
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "nul byte found in provided data"));
        }
        Ok(())
    }

    fn start(&mut self, defaults: [Fallback; 3], span: Span) -> io::Result<Child> {
        self.admit()?;
        let stdio = [0, 1, 2].map(|i| match &self.stdio[i] {
            Some(s) => s.config(),
            None => defaults[i].config(),
        });
        match &self.proc.sim {
            Some(sim) => sim::spawn(sim, self.proc.tasks.as_ref().unwrap(), self, stdio, span),
            None => real::spawn(self, stdio),
        }
    }
}

#[derive(Clone, Copy)]
enum Fallback {
    Inherit,
    Null,
    Piped,
}
impl Fallback {
    fn config(self) -> Config {
        match self {
            Fallback::Inherit => Config::Inherit,
            Fallback::Null => Config::Null,
            Fallback::Piped => Config::Piped,
        }
    }
}

/// How a child's standard descriptor is set up. Making one takes no capability, but it can hold
/// a handle.
pub struct Stdio(Config);
#[derive(Clone)]
enum Config {
    Piped,
    Inherit,
    Null,
    // `io::stdout()` / `io::stderr()`: the process's own descriptor 1 or 2.
    Parent(u8),
    // A file or another child's pipe end (real backend only).
    Fd(Arc<std::os::fd::OwnedFd>),
    // A conversion the v0 simulator does not model (a coverage failure at spawn).
    Unsupported(&'static str),
}
impl Stdio {
    fn config(&self) -> Config {
        self.0.clone()
    }
    pub fn piped() -> Stdio {
        Stdio(Config::Piped)
    }
    pub fn inherit() -> Stdio {
        Stdio(Config::Inherit)
    }
    pub fn null() -> Stdio {
        Stdio(Config::Null)
    }
}
impl std::fmt::Debug for Stdio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Stdio { .. }")
    }
}
impl From<crate::fs::File> for Stdio {
    fn from(file: crate::fs::File) -> Stdio {
        Stdio(match file.into_fd() {
            Some(fd) => Config::Fd(Arc::new(fd)),
            None => Config::Unsupported("Stdio::from(File)"),
        })
    }
}
impl From<crate::io::Stdout> for Stdio {
    fn from(_: crate::io::Stdout) -> Stdio {
        Stdio(Config::Parent(1))
    }
}
impl From<crate::io::Stderr> for Stdio {
    fn from(_: crate::io::Stderr) -> Stdio {
        Stdio(Config::Parent(2))
    }
}

/// A started process. Dropping it neither kills nor reaps; dropping a pipe field closes it.
pub struct Child {
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
    inner: Inner,
}
enum Inner {
    Real(real::Child),
    Sim(sim::Child),
}
impl std::fmt::Debug for Child {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child")
            .field("stdin", &self.stdin)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .finish_non_exhaustive()
    }
}

impl Child {
    /// The host process ID; simulated data under simulation, never a host PID.
    pub fn id(&self) -> u32 {
        match &self.inner {
            Inner::Real(c) => c.id(),
            Inner::Sim(c) => c.id(),
        }
    }
    /// Closes the retained stdin, then waits for exit. A checkpoint: a cancelled wait returns
    /// `Cancelled` and leaves the child with the caller. Repeated calls return the cached status.
    #[track_caller]
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let span = std::panic::Location::caller();
        drop(self.stdin.take());
        match &mut self.inner {
            Inner::Real(c) => c.wait(span, true),
            Inner::Sim(c) => c.wait(span, true),
        }
    }
    /// `None` while running; otherwise reaps once and caches. Does not close stdin.
    #[track_caller]
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match &mut self.inner {
            Inner::Real(c) => c.try_wait(),
            Inner::Sim(c) => c.try_wait(std::panic::Location::caller()),
        }
    }
    /// Submits SIGKILL to this child and returns without waiting; `Ok` once it has exited.
    #[track_caller]
    pub fn kill(&mut self) -> io::Result<()> {
        match &mut self.inner {
            Inner::Real(c) => c.kill(),
            Inner::Sim(c) => c.kill(std::panic::Location::caller()),
        }
    }
    /// Closes stdin, takes stdout and stderr, reads them to end, and waits. Takes `&mut self`
    /// (03 §1.1): a cancelled call returns `Cancelled` and leaves the child with the caller,
    /// with those fields `None` and the output read so far lost.
    #[track_caller]
    pub fn wait_with_output(&mut self) -> io::Result<Output> {
        let span = std::panic::Location::caller();
        let (out, err) = self.drain(span)?;
        let status = self.wait()?;
        Ok(Output { status, stdout: out, stderr: err })
    }

    fn drain(&mut self, span: Span) -> io::Result<(Vec<u8>, Vec<u8>)> {
        drop(self.stdin.take());
        let (out, err) = (self.stdout.take(), self.stderr.take());
        match &mut self.inner {
            Inner::Real(_) => real::drain(out, err, span),
            Inner::Sim(c) => c.drain(out, err, span),
        }
    }

    // `output` and `status` after their spawn. On any error, cancellation included: kill, close
    // the pipe ends, wait shielded, then return the error, so a child the caller never sees is
    // never left running or unreaped (03 §1.1).
    fn finish(&mut self, span: Span, capture: bool) -> io::Result<Output> {
        let result = (|| -> io::Result<Output> {
            let (out, err) = if capture { self.drain(span)? } else { (vec![], vec![]) };
            drop(self.stdin.take());
            let status = match &mut self.inner {
                Inner::Real(c) => c.wait(span, true)?,
                Inner::Sim(c) => c.wait(span, true)?,
            };
            Ok(Output { status, stdout: out, stderr: err })
        })();
        let Err(e) = result else { return result };
        if let Err(k) = self.kill() {
            crate::__sink::log(
                crate::log::Level::Warn,
                format_args!("a failed process call could not kill its child: {k}"),
            );
        }
        (self.stdin.take(), self.stdout.take(), self.stderr.take());
        #[cfg(test)]
        if let Inner::Sim(c) = &self.inner
            && c.control() == Some(sim::Broken::NoCleanupWait)
        {
            return Err(e);
        }
        let reaped = match &mut self.inner {
            Inner::Real(c) => c.wait(span, false),
            Inner::Sim(c) => c.wait(span, false),
        };
        if let Err(w) = reaped {
            crate::__sink::log(
                crate::log::Level::Warn,
                format_args!("a failed process call could not wait for its child: {w}"),
            );
        }
        Err(e)
    }
}

/// The parent's end of a child's stdin pipe. Writes are checkpoints.
pub struct ChildStdin(Pipe);
/// The parent's end of a child's stdout pipe. Reads are checkpoints; EOF is not an exit.
pub struct ChildStdout(Pipe);
/// The parent's end of a child's stderr pipe. Reads are checkpoints.
pub struct ChildStderr(Pipe);
enum Pipe {
    Real(real::Pipe),
    Sim(sim::Pipe),
}
macro_rules! pipe_debug {
    ($($t:ident)*) => {$(
        impl std::fmt::Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(concat!(stringify!($t), " { .. }"))
            }
        }
    )*};
}
pipe_debug!(ChildStdin ChildStdout ChildStderr);

impl Pipe {
    #[track_caller]
    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Pipe::Real(p) => p.read(buf, std::panic::Location::caller()),
            Pipe::Sim(p) => p.read(buf, std::panic::Location::caller()),
        }
    }
    #[track_caller]
    fn write(&self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Pipe::Real(p) => p.write(buf, std::panic::Location::caller()),
            Pipe::Sim(p) => p.write(buf, std::panic::Location::caller()),
        }
    }
    fn into_stdio(self) -> Stdio {
        Stdio(match self {
            Pipe::Real(p) => Config::Fd(Arc::new(p.into_fd())),
            Pipe::Sim(_) => Config::Unsupported("piping one child into another"),
        })
    }
}

macro_rules! pipe_io {
    ($($t:ident: $trait:ident $method:ident $buf:ty;)*) => {$(
        impl io::$trait for &$t {
            #[track_caller]
            fn $method(&mut self, buf: $buf) -> io::Result<usize> {
                self.0.$method(buf)
            }
            pipe_io!(@flush $trait);
        }
        impl io::$trait for $t {
            #[track_caller]
            fn $method(&mut self, buf: $buf) -> io::Result<usize> {
                self.0.$method(buf)
            }
            pipe_io!(@flush $trait);
        }
        impl From<$t> for Stdio {
            fn from(end: $t) -> Stdio {
                end.0.into_stdio()
            }
        }
    )*};
    (@flush Write) => {
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    };
    (@flush Read) => {};
}
pipe_io! {
    ChildStdin: Write write &[u8];
    ChildStdout: Read read &mut [u8];
    ChildStderr: Read read &mut [u8];
}

#[cfg(test)]
#[path = "process/tests.rs"]
mod tests;
