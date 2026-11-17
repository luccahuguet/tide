//! The workspace suite runs the demo, and two runs with one revision swapped show that it fails
//! when the fix is missing or the patch under review no longer widens the interface. Each run
//! works in its own directory under Cargo's test scratch directory, unique to the test process,
//! and removes it when the case ends, whether it passes, fails, or panics. Four more cases check
//! step 5's bounded debugger run, and two the toolchain a run without rustup uses, without
//! building anything.
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tide_demo::{Demo, End, Failure, PINNED, Revision, run_bounded};

/// A case's work directory; dropping it removes the directory.
struct Work(PathBuf);

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn demo(case: &str) -> (Demo, Work) {
    // Cargo's scratch directory for integration tests is `<target-dir>/tmp`, however the target
    // dir was set (a relative `CARGO_TARGET_DIR`, `--target-dir`, `build.target-dir`): the demo
    // builds into the suite's own target dir, so its builds reuse the suite's.
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let work = scratch.join(format!("tide-demo-{case}-{}", std::process::id()));
    let demo = Demo {
        work: work.clone(),
        target: scratch.parent().map(PathBuf::from),
        ..Demo::default()
    };
    (demo, Work(work))
}

/// The step a run failed at, or a panic with what else it did.
fn failed_step(demo: Demo) -> u8 {
    match demo.run() {
        Err(Failure::Step { step, .. }) => step,
        other => panic!("expected a failed step, got {other:?}"),
    }
}

#[test]
fn the_demo_passes() {
    let (demo, _work) = demo("pass");
    if let Err(failure) = demo.run() {
        failure.report();
        panic!("{failure}");
    }
}

/// The buggy patch applied as the fixed revision: step 3 runs the bug over the seed set.
#[test]
fn a_buggy_fixed_revision_fails_step_3() {
    let (demo, _work) = demo("buggy-fixed");
    let demo = Demo {
        fixed: Revision::Patched("lost-update.patch"),
        ..demo
    };
    assert_eq!(failed_step(demo), 3);
}

/// The widening patch dropped: the trust diff no longer shows `hits::serve` widened.
#[test]
fn a_dropped_widening_fails_step_4() {
    let (demo, _work) = demo("no-widening");
    let demo = Demo {
        widened: Revision::Service,
        ..demo
    };
    assert_eq!(failed_step(demo), 4);
}

/// `PINNED`, the identity a run without rustup checks the toolchain on `PATH` against, is the
/// toolchain rust-toolchain.toml pins, as rustup runs it.
#[test]
fn the_recorded_identity_is_the_pinned_toolchain() {
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/../../rust-toolchain.toml");
    let text = std::fs::read_to_string(file).unwrap();
    let channel = format!("channel = \"{}\"", PINNED.channel);
    assert!(text.lines().any(|line| line.trim() == channel), "{text}");
    for (tool, fields) in [("rustc", PINNED.rustc), ("cargo", PINNED.cargo)] {
        let out = Command::new("rustup")
            .env_remove("RUSTUP_TOOLCHAIN")
            .args(["run", PINNED.channel, tool, "-vV"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let reported = String::from_utf8(out.stdout).unwrap();
        for (key, value) in fields {
            let line = format!("{key}: {value}");
            assert!(
                reported.lines().any(|l| l == line),
                "{tool}: {line}\n{reported}"
            );
        }
    }
}

/// Without rustup on `PATH`, as in the flake's dev shell, the demo builds only with the rustc and
/// cargo whose `-vV` match `PINNED` exactly, and every cargo it starts compiles with the rustc it
/// checked (`$RUSTC`, else `rustc` on `PATH`), even when Cargo's own settings name another
/// (`CARGO_BUILD_RUSTC`). The fake cargo
/// records each build it is asked for and fails it, which ends the run with a setup error.
#[test]
fn without_rustup_the_demo_builds_only_with_the_checked_pinned_toolchain() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("tide-demo-no-rustup-{}", std::process::id()));
    let _work = Work(dir.clone());
    // A child shell writes each script, so no thread of this process holds it open for writing
    // while another runs it (ETXTBSY).
    let script = |path: &std::path::Path, body: &str| {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let status = Command::new("sh")
            .args([
                "-c",
                "printf '%s' \"$1\" > \"$2\" && chmod +x \"$2\"",
                "sh",
                body,
            ])
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success());
    };
    let identity = |fields: &[(&str, &str)], swap: Option<(&str, &str)>| {
        let lines = fields.iter().map(|&(key, value)| match swap {
            Some((swapped, other)) if swapped == key => format!("{key}: {other}"),
            _ => format!("{key}: {value}"),
        });
        lines.collect::<Vec<_>>().join("\n")
    };
    let log = dir.join("builds.log");
    // `case` names the directory of one run's PATH, which holds only a fake rustc and cargo.
    // `rustc_env`, when given, is the run's `RUSTC`.
    let run = |case: &str,
               rustc: Option<(&str, &str)>,
               cargo: Option<(&str, &str)>,
               rustc_env: Option<&std::path::Path>| {
        let bin = dir.join(case);
        let rustc_vv = identity(PINNED.rustc, rustc);
        script(
            &bin.join("rustc"),
            &format!("#!/bin/sh\necho 'rustc 1.99.0-nightly (fake)'\necho '{rustc_vv}'\n"),
        );
        let cargo_vv = identity(PINNED.cargo, cargo);
        script(
            &bin.join("cargo"),
            &format!(
                "#!/bin/sh\nif [ \"$1\" = -vV ]; then echo 'cargo (fake)'; echo '{cargo_vv}'; \
                 exit 0; fi\necho \"{case} RUSTC=$RUSTC $*\" >> '{}'\nexit 1\n",
                log.display()
            ),
        );
        let mut demo = Command::new(env!("CARGO_BIN_EXE_tide-demo"));
        demo.env_clear();
        if let Some(rustc) = rustc_env {
            demo.env("RUSTC", rustc);
        }
        let out = demo
            .env("PATH", &bin)
            .env("CARGO_TARGET_DIR", dir.join("target"))
            .env("CARGO_BUILD_RUSTC", "/nonexistent/other/rustc")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        assert_eq!(out.status.code(), Some(2), "{case}: {stderr}");
        (bin, stderr)
    };
    let builds = || std::fs::read_to_string(&log).unwrap_or_default();

    // The pinned pair: the build runs, with `RUSTC` the checked rustc, over Cargo's own setting.
    let (bin, stderr) = run("pinned", None, None, None);
    assert!(stderr.contains("cannot build tide-admit"), "{stderr}");
    let build = format!("pinned RUSTC={} build ", bin.join("rustc").display());
    assert!(builds().starts_with(&build), "{}", builds());
    // Another rustc or cargo of the same release: refused before any build.
    let other = "0123456789abcdef0123456789abcdef01234567";
    for (case, rustc, cargo, tool) in [
        ("rustc-hash", Some(("commit-hash", other)), None, "rustc"),
        ("cargo-hash", None, Some(("commit-hash", other)), "cargo"),
    ] {
        let (_, stderr) = run(case, rustc, cargo, None);
        let refusal = format!("is not the {tool} of the pinned {}", PINNED.channel);
        assert!(stderr.contains(&refusal), "{case}: {stderr}");
        assert!(
            stderr.contains(&format!("commit-hash: {other}")),
            "{case}: {stderr}"
        );
        assert!(!builds().contains(case), "{case}: {}", builds());
    }
    // `$RUSTC` over the rustc on `PATH`: the pinned rustc of the first run, while `PATH` holds
    // another; the check passes on `$RUSTC`, and the build is bound to it.
    let (_, stderr) = run(
        "rustc-env",
        Some(("commit-hash", other)),
        None,
        Some(&bin.join("rustc")),
    );
    assert!(stderr.contains("cannot build tide-admit"), "{stderr}");
    let build = format!("rustc-env RUSTC={} build ", bin.join("rustc").display());
    assert!(
        builds().lines().any(|line| line.starts_with(&build)),
        "{}",
        builds()
    );
}

/// Step 5's debugger run waits for a process the command left behind, as gdb leaves the driver it
/// detaches, and keeps what that process writes after the command exits.
#[test]
fn a_bounded_run_waits_for_a_left_behind_writer() {
    let mut command = Command::new("sh");
    command.args(["-c", "(sleep 1; echo late) & echo early"]);
    let run = run_bounded(command, Duration::from_secs(30)).unwrap();
    assert!(
        matches!(run.end, End::Exited(status) if status.success()),
        "{:?}",
        run.end
    );
    assert_eq!(String::from_utf8_lossy(&run.output), "early\nlate\n");
}

/// The pids a command printed, one or more per line.
fn pids(output: &[u8]) -> Vec<u32> {
    String::from_utf8_lossy(output)
        .split_whitespace()
        .map(|pid| pid.parse().expect("a pid"))
        .collect()
}

/// Whether `pid` is still running: present in `/proc` and not a zombie its new parent has yet
/// to reap, which is `run_bounded`'s contract (a killed orphan is init's or the subreaper's to
/// reap). Waits up to 5 s for it to stop.
fn still_running(pid: u32) -> bool {
    let running = || {
        std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
            !stat
                .rsplit_once(") ")
                .is_some_and(|(_, rest)| rest.starts_with('Z'))
        })
    };
    let wait = Instant::now();
    while running() && wait.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(50));
    }
    running()
}

/// A stalled debugger fails step 5 at the deadline, and every process of its group is killed,
/// including a child that holds the output pipe.
#[test]
fn a_bounded_run_kills_a_stalled_command_at_its_deadline() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 600 & echo $!; sleep 600"]);
    let start = Instant::now();
    let run = run_bounded(command, Duration::from_secs(1)).unwrap();
    assert!(matches!(run.end, End::Killed), "{:?}", run.end);
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{:?}",
        start.elapsed()
    );
    for pid in pids(&run.output) {
        assert!(!still_running(pid), "{pid} is still running");
    }
}

/// The deadline holds after the command itself exits, as gdb does once its inferior ends: a
/// writer the command left behind that outlives the deadline fails the run and is killed.
#[test]
fn a_bounded_run_kills_a_left_behind_writer_at_its_deadline() {
    let mut command = Command::new("sh");
    // The subshell prints its sleep's pid, and the command its subshell's, then exits at once.
    command.args(["-c", "(sleep 600 & echo $!; wait; echo late) & echo $!"]);
    let start = Instant::now();
    let run = run_bounded(command, Duration::from_secs(1)).unwrap();
    assert!(matches!(run.end, End::Killed), "{:?}", run.end);
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{:?}",
        start.elapsed()
    );
    let pids = pids(&run.output);
    assert_eq!(pids.len(), 2, "{:?}", String::from_utf8_lossy(&run.output));
    for pid in pids {
        assert!(!still_running(pid), "{pid} is still running");
    }
}

/// A writer that left the group with `setsid` and keeps writing does not hold the run past its
/// bound: after the deadline the run kills the group and drains what is left for at most 2 s.
#[test]
fn a_bounded_run_returns_while_an_escaped_writer_keeps_writing() {
    /// The escaped writer's own group, which this test kills however it ends.
    struct Escaped(i32);
    impl Drop for Escaped {
        fn drop(&mut self) {
            // SAFETY: `kill` takes plain integers; the group is the writer `setsid` made, whose
            // leader keeps writing until this kill.
            unsafe { libc::kill(-self.0, libc::SIGKILL) };
        }
    }
    let mut command = Command::new("sh");
    // The writer prints its pid, which `setsid` made its group's id, then writes without end;
    // the command exits after 0.2 s.
    command.args([
        "-c",
        "setsid sh -c 'echo $$; while :; do echo escaped; done' & sleep 0.2",
    ]);
    let start = Instant::now();
    let run = run_bounded(command, Duration::from_secs(1)).unwrap();
    let elapsed = start.elapsed();
    let output = String::from_utf8_lossy(&run.output);
    let pid: i32 = output
        .lines()
        .next()
        .and_then(|line| line.parse().ok())
        .expect("the writer's pid");
    let _escaped = Escaped(pid);
    assert!(matches!(run.end, End::Killed), "{:?}", run.end);
    // The 1 s deadline, the 2 s drain, and slack for a loaded host.
    assert!(elapsed < Duration::from_secs(8), "{elapsed:?}");
    assert!(output.lines().nth(1) == Some("escaped"));
}
