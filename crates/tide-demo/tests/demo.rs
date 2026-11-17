//! The workspace suite runs the demo, and two runs with one revision swapped show that it fails
//! when the fix is missing or the patch under review no longer widens the interface. Each run
//! works in its own directory under Cargo's test scratch directory, unique to the test process,
//! and removes it when the case ends, whether it passes, fails, or panics. Four more cases check
//! step 5's bounded debugger run without building anything.
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tide_demo::{Demo, End, Failure, Revision, run_bounded};

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
