// Real Linux backend over `std::process` (research/40 §§3, 5). Std resolves the program itself:
// a bare name against the absolute elements of the start-time `PATH`, then the file, or an
// absolute program, to its canonical path in the parent, which the child execs with argv[0] as
// spelled. After stdio setup the child marks every other descriptor close-on-exec with
// `close_range(3, ~0, CLOSE_RANGE_CLOEXEC)` and fails the spawn if that fails (Linux 5.11).
// Pipe ends are nonblocking; reads, writes and waits are cancellable through the network
// adapter's descriptor wait, on an epoll set for two pipes and a pidfd for exit.
use super::{Command, Config, Pipe as PublicPipe, Span};
use crate::io;
use crate::task::{NetContext, Network};
use rustix::event::epoll;
use rustix::fd::{AsFd, FromRawFd, OwnedFd};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

const SYS_PIDFD_OPEN: std::ffi::c_long = 434; // Linux x86_64 and aarch64 numbers.
const SYS_CLOSE_RANGE: std::ffi::c_long = 436;
const CLOSE_RANGE_CLOEXEC: std::ffi::c_uint = 1 << 2;
unsafe extern "C" {
    fn syscall(number: std::ffi::c_long, ...) -> std::ffi::c_long;
}

// Injected failures for the cleanup tests: "nonblocking" fails pipe setup after the spawn,
// "drain" fails reading a hidden child's output.
#[cfg(test)]
thread_local! {
    pub(super) static FAIL: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
fn injected(point: &str) -> io::Result<()> {
    if FAIL.get() == Some(point) { Err(io::Error::new(io::ErrorKind::Other, format!("injected {point} failure"))) } else { Ok(()) }
}

fn context() -> NetContext {
    Network::new(None).context()
}

/// The program the child execs: lookup for a bare name, then the canonical path in the parent.
/// A relative path with `/` (unattenuated `Proc` only, since `only` grants none) keeps Rust's
/// meaning.
fn resolve(program: &OsStr) -> io::Result<OsString> {
    let bytes = program.as_bytes();
    let file = if !bytes.contains(&b'/') {
        lookup(program, std::env::var_os("PATH"))?
    } else if bytes[0] == b'/' {
        PathBuf::from(program)
    } else {
        return Ok(program.to_owned());
    };
    std::fs::canonicalize(file).map(PathBuf::into_os_string).map_err(io::from_std)
}

/// The first executable regular file named `name` in an absolute element of `path`. Empty and
/// relative elements are skipped, and there is no default search path.
pub(super) fn lookup(name: &OsStr, path: Option<OsString>) -> io::Result<PathBuf> {
    let path = path.unwrap_or_default();
    for dir in path.as_bytes().split(|b| *b == b':') {
        if dir.first() != Some(&b'/') {
            continue;
        }
        let file = Path::new(OsStr::from_bytes(dir)).join(name);
        let regular = std::fs::metadata(&file).is_ok_and(|m| m.is_file());
        if regular && rustix::fs::access(&file, rustix::fs::Access::EXEC_OK).is_ok() {
            return Ok(file);
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "program not found on the start-time PATH"))
}

fn stdio(config: Config, slot: usize) -> io::Result<std::process::Stdio> {
    Ok(match config {
        Config::Piped => std::process::Stdio::piped(),
        Config::Inherit => std::process::Stdio::inherit(),
        Config::Null => std::process::Stdio::null(),
        Config::Parent(1) => std::io::stdout().into(),
        Config::Parent(_) => std::io::stderr().into(),
        Config::Fd(fd) => OwnedFd::from(fd.try_clone().map_err(io::from_std)?).into(),
        Config::Unsupported(what) => panic!("simulation handle used outside its simulation: {what} on real slot {slot}"),
    })
}

pub(super) fn spawn(cmd: &Command, config: [Config; 3]) -> io::Result<super::Child> {
    crate::task::outside_simulation();
    let program = resolve(&cmd.program)?;
    let mut std_cmd = std::process::Command::new(&program);
    std_cmd.arg0(&cmd.program).args(&cmd.args);
    if cmd.env_clear {
        std_cmd.env_clear();
    }
    for (k, v) in &cmd.env {
        match v {
            Some(v) => std_cmd.env(k, v),
            None => std_cmd.env_remove(k),
        };
    }
    if let Some(dir) = &cmd.cwd {
        std_cmd.current_dir(dir);
    }
    let [i, o, e] = config;
    std_cmd.stdin(stdio(i, 0)?).stdout(stdio(o, 1)?).stderr(stdio(e, 2)?);
    // SAFETY: the closure makes one async-signal-safe system call and reads errno.
    unsafe {
        std_cmd.pre_exec(|| {
            if syscall(SYS_CLOSE_RANGE, 3 as std::ffi::c_uint, !0 as std::ffi::c_uint, CLOSE_RANGE_CLOEXEC) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    let mut child = std_cmd.spawn().map_err(io::from_std)?;
    let pipe = |fd: Option<OwnedFd>| -> io::Result<Option<PublicPipe>> {
        fd.map(|fd| {
            #[cfg(test)]
            injected("nonblocking")?;
            rustix::io::ioctl_fionbio(&fd, true).map_err(|e| io::from_std(e.into()))?;
            Ok(PublicPipe::Real(Pipe(fd)))
        })
        .transpose()
    };
    let ends = (|| -> io::Result<_> {
        let stdin = pipe(child.stdin.take().map(OwnedFd::from))?.map(super::ChildStdin);
        let stdout = pipe(child.stdout.take().map(OwnedFd::from))?.map(super::ChildStdout);
        let stderr = pipe(child.stderr.take().map(OwnedFd::from))?.map(super::ChildStderr);
        Ok((stdin, stdout, stderr))
    })();
    match ends {
        Ok((stdin, stdout, stderr)) => {
            Ok(super::Child { stdin, stdout, stderr, inner: super::Inner::Real(Child { child, status: None }) })
        }
        // The caller never gets this child. Close every pipe end first, including those not yet
        // set up, so a child that survives a failed kill cannot block writing to one; then kill
        // and reap it.
        Err(e) => {
            (child.stdin.take(), child.stdout.take(), child.stderr.take());
            let _ = child.kill();
            let _ = child.wait();
            Err(e)
        }
    }
}

pub(super) struct Child {
    child: std::process::Child,
    status: Option<std::process::ExitStatus>,
}
impl Child {
    pub(super) fn id(&self) -> u32 {
        self.child.id()
    }
    pub(super) fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        crate::task::outside_simulation();
        if self.status.is_none() {
            self.status = self.child.try_wait().map_err(io::from_std)?;
        }
        Ok(self.status)
    }
    pub(super) fn kill(&mut self) -> io::Result<()> {
        crate::task::outside_simulation();
        if self.status.is_some() {
            return Ok(());
        }
        self.child.kill().map_err(io::from_std)
    }
    // A cancellable wait polls a pidfd; a shielded one (cleanup) blocks in the kernel.
    pub(super) fn wait(&mut self, span: Span, cancellable: bool) -> io::Result<std::process::ExitStatus> {
        // A checkpoint even when the status is already known (TASK-3).
        if cancellable {
            context().check()?;
        }
        if let Some(status) = self.try_wait()? {
            return Ok(status);
        }
        if cancellable {
            // SAFETY: pidfd_open takes a PID and flags and returns a new descriptor or -1.
            let fd = unsafe { syscall(SYS_PIDFD_OPEN, self.child.id() as std::ffi::c_int, 0 as std::ffi::c_uint) };
            if fd < 0 {
                return Err(io::from_std(std::io::Error::last_os_error()));
            }
            // SAFETY: the descriptor was just returned to us and is owned by nobody else.
            let pidfd = unsafe { OwnedFd::from_raw_fd(fd as i32) };
            let c = context();
            while self.try_wait()?.is_none() {
                c.socket(pidfd.as_fd(), false, span, None)?;
            }
        } else {
            self.status = Some(self.child.wait().map_err(io::from_std)?);
        }
        Ok(self.status.unwrap())
    }
}

pub(super) struct Pipe(OwnedFd);
impl Pipe {
    pub(super) fn into_fd(self) -> OwnedFd {
        // A child given this end expects ordinary blocking I/O.
        let _ = rustix::io::ioctl_fionbio(&self.0, false);
        self.0
    }
    fn retry<T>(&self, write: bool, span: Span, mut f: impl FnMut() -> rustix::io::Result<T>) -> io::Result<T> {
        crate::task::outside_simulation();
        let c = context();
        loop {
            c.check()?;
            match f() {
                Err(rustix::io::Errno::AGAIN) => c.socket(self.0.as_fd(), write, span, None)?,
                Err(rustix::io::Errno::INTR) => {}
                result => return result.map_err(|e| io::from_std(e.into())),
            }
        }
    }
    pub(super) fn read(&self, buf: &mut [u8], span: Span) -> io::Result<usize> {
        self.retry(false, span, || rustix::io::read(&self.0, &mut *buf))
    }
    pub(super) fn write(&self, buf: &[u8], span: Span) -> io::Result<usize> {
        self.retry(true, span, || rustix::io::write(&self.0, buf))
    }
}

/// Reads both pipes to end at once, so neither can fill and stall the child. Cancellable.
pub(super) fn drain(
    out: Option<super::ChildStdout>,
    err: Option<super::ChildStderr>,
    span: Span,
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    crate::task::outside_simulation();
    let fd = |end: Option<PublicPipe>| match end {
        Some(PublicPipe::Real(p)) => Some(p.0),
        Some(PublicPipe::Sim(_)) => panic!("simulation handle used outside its simulation: a simulated pipe on a real child"),
        None => None,
    };
    let mut ends = [(fd(out.map(|o| o.0)), Vec::new()), (fd(err.map(|e| e.0)), Vec::new())];
    // A checkpoint even with no pipe to read, before any wait resource exists.
    context().check()?;
    #[cfg(test)]
    injected("drain")?;
    let set = epoll::create(epoll::CreateFlags::CLOEXEC).map_err(|e| io::from_std(e.into()))?;
    for (i, (fd, _)) in ends.iter().enumerate() {
        if let Some(fd) = fd {
            epoll::add(&set, fd, epoll::EventData::new_u64(i as u64), epoll::EventFlags::IN)
                .map_err(|e| io::from_std(e.into()))?;
        }
    }
    let c = context();
    let mut chunk = [0u8; 8192];
    while ends.iter().any(|e| e.0.is_some()) {
        c.check()?;
        let mut progressed = false;
        for (end, bytes) in &mut ends {
            let Some(fd) = end else { continue };
            match rustix::io::read(&*fd, &mut chunk) {
                Ok(0) => {
                    *end = None; // Closing the descriptor also leaves the epoll set.
                    progressed = true;
                }
                Ok(n) => {
                    bytes.extend_from_slice(&chunk[..n]);
                    progressed = true;
                }
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(e) => return Err(io::from_std(e.into())),
            }
        }
        if !progressed {
            c.socket(set.as_fd(), false, span, None)?;
        }
    }
    let [(_, out), (_, err)] = ends;
    Ok((out, err))
}
