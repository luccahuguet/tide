//! Wall time and peak resident memory of one child process (fx-6u8m.6), written so the
//! build-time benchmark of fx-ayxr can include this file as it is (`#[path]`; it needs only
//! `libc`). Linux only. `wait4` reports `ru_maxrss` for the child and the descendants it waited
//! for (the kernel's `RUSAGE_BOTH`), as the largest of their peaks, never their sum: a driver
//! that starts a compile child, which starts the linker, reports its largest process.
use std::io::{self, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// What one measured process did.
pub struct Measured {
    pub status: ExitStatus,
    /// From just before the spawn until `wait4` returned.
    pub wall: Duration,
    /// The largest peak resident set of the process and its waited-for descendants, in KiB.
    pub peak_rss_kib: u64,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Runs `command` with standard input closed and its output captured, and reaps it with `wait4`.
pub fn measure(command: &mut Command) -> io::Result<Measured> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let start = Instant::now();
    let mut child = command.spawn()?;
    let pid = child.id() as libc::pid_t;
    let mut out = child.stdout.take().expect("stdout is piped");
    let mut err = child.stderr.take().expect("stderr is piped");
    let errors = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        err.read_to_end(&mut bytes).map(|_| bytes)
    });
    let mut stdout = Vec::new();
    let read = out.read_to_end(&mut stdout);
    let stderr = errors.join().expect("the stderr reader panicked");
    let mut status = 0;
    // SAFETY: `rusage` is plain old data, for which all zero bytes are a valid value.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: both pointers are to live locals; `pid` is our unreaped child. `Child` never
        // waits for it afterwards, and dropping a `Child` neither waits nor kills.
        let reaped = unsafe { libc::wait4(pid, &mut status, 0, &mut usage) };
        if reaped == pid {
            break;
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    let wall = start.elapsed();
    read?;
    Ok(Measured {
        status: ExitStatus::from_raw(status),
        wall,
        peak_rss_kib: u64::try_from(usage.ru_maxrss).unwrap_or(0),
        stdout,
        stderr: stderr?,
    })
}
