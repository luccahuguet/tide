use super::SimRoot;
use std::io::{self, BufRead, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CHILD: &str = "__TIDE_SIM_CHILD";
const SEED: &str = "TIDE_SIM_SEED";
const START: &str = "TIDE_SIM_SEED_START ";
const TIMEOUT: Duration = Duration::from_secs(300);

/// Internal entry for `#[sim_test]`. `TIDE_SIM_SEED` selects one fresh decimal u64 seed;
/// it performs no source/build identity check and is not manifest replay.
#[doc(hidden)]
pub fn __run(path: &str, seeds: u64, body: fn(SimRoot)) {
    let name = path.split_once("::").map_or(path, |(_, name)| name);
    if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
        run_seeds(name, seeds, body);
        return;
    }
    let output = child(path, name, TIMEOUT).expect("could not run simulation child");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let prefix = format!("simulation {name}: ");
    let summary = stderr
        .lines()
        .find(|line| line.starts_with(&prefix) && line.ends_with(" seeds)"));
    // Use Rust's print family so libtest captures harness diagnostics under this test.
    if let Some(summary) = summary
        && !output.timed_out
        && output.status.success()
    {
        std::eprintln!("{summary}");
        return;
    }
    std::eprint!("{stderr}");
    if output.timed_out || output.status.code().is_none() || summary.is_none() {
        panic!(
            "simulation {name} failed: {} (last started seed: {}); status: {}",
            if output.timed_out {
                "per-seed wall-clock timeout"
            } else if output.status.code().is_none() {
                "child terminated by signal"
            } else {
                "missing summary"
            },
            output
                .last_seed
                .map_or_else(|| "none".into(), |seed| seed.to_string()),
            output.status,
        );
    }
    let failing = stderr
        .lines()
        .filter_map(|line| {
            line.strip_prefix("simulation failure seed=")?
                .split_whitespace()
                .next()
        })
        .collect::<Vec<_>>()
        .join(", ");
    panic!("simulation {name} failed; failing seeds: {failing}");
}

struct ChildOutput {
    status: ExitStatus,
    stderr: Vec<u8>,
    last_seed: Option<u64>,
    timed_out: bool,
}

fn child(path: &str, name: &str, timeout: Duration) -> io::Result<ChildOutput> {
    let mut process = Command::new(std::env::current_exe()?)
        .args(["--exact", name, "--nocapture", "--include-ignored"])
        .env(CHILD, path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = process.stdout.take().unwrap();
    let mut stderr = process.stderr.take().unwrap();
    let progress = Arc::new(Mutex::new((None, Instant::now())));
    let starts = progress.clone();
    // Drain both pipes while the child runs: a full trace may exceed a pipe's buffer.
    let out = std::thread::spawn(move || -> io::Result<()> {
        let mut reader = io::BufReader::new(stdout);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line)? != 0 {
            if let Ok(text) = std::str::from_utf8(&line)
                && let Some(seed) = text
                    .strip_prefix(START)
                    .and_then(|seed| seed.trim().parse().ok())
            {
                *starts.lock().unwrap() = (Some(seed), Instant::now());
            }
            line.clear();
        }
        Ok(())
    });
    let err = std::thread::spawn(move || -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        stderr.read_to_end(&mut output)?;
        Ok(output)
    });
    let mut timed_out = false;
    let status = loop {
        match process.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if progress.lock().unwrap().1.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(10))
            }
            waiting => {
                timed_out = waiting.is_ok();
                let _ = process.kill();
                let reaped = process.wait();
                break waiting.and(reaped);
            }
        }
    };
    // Reap and join even when waiting failed, so no child or reader outlives this call.
    let drained = out.join().expect("simulation stdout reader panicked");
    let stderr = err.join().expect("simulation stderr reader panicked");
    drained?;
    let last_seed = progress.lock().unwrap().0;
    Ok(ChildOutput {
        status: status?,
        stderr: stderr?,
        last_seed,
        timed_out,
    })
}

struct Run {
    trace: String,
    failure: Option<String>,
}

fn evaluate(seed: u64, body: fn(SimRoot)) -> Run {
    let sim = crate::Root::sim(seed);
    // Retain observation access after the body consumes and drops its SimRoot, even on panic.
    let tasks = sim.tasks.clone();
    let result = catch_unwind(AssertUnwindSafe(|| body(sim)));
    let failure = result.err().map(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).into()))
            .unwrap_or_else(|| "panic with a non-string payload".into())
    });
    Run {
        trace: tasks.sim_trace(),
        failure,
    }
}

fn first_difference<'a>(first: &'a str, second: &'a str) -> Option<(usize, &'a str, &'a str)> {
    let mut first = first.split_inclusive('\n');
    let mut second = second.split_inclusive('\n');
    for line in 1.. {
        match (first.next(), second.next()) {
            (None, None) => return None,
            (a, b) if a == b => {}
            (a, b) => {
                return Some((
                    line,
                    a.unwrap_or("<end of trace>"),
                    b.unwrap_or("<end of trace>"),
                ));
            }
        }
    }
    unreachable!()
}

fn run_seeds(name: &str, seeds: u64, body: fn(SimRoot)) {
    let selected = std::env::var_os(SEED).map(|seed| {
        seed.to_str()
            .and_then(|seed| seed.parse::<u64>().ok())
            .expect("TIDE_SIM_SEED must be a decimal u64")
    });
    let count = if selected.is_some() { 1 } else { seeds };
    let mut failures = 0;
    for index in 0..count {
        let seed = selected.unwrap_or(index);
        // Begin on a new line even when libtest has printed its `test name ... ` prefix.
        println!("\n{START}{seed}");
        io::stdout()
            .flush()
            .expect("cannot report simulation seed start");
        let run = evaluate(seed, body);
        let mut failed = run.failure.is_some();
        if index == 0 || failed {
            let again = evaluate(seed, body);
            if let Some((line, a, b)) = first_difference(&run.trace, &again.trace) {
                eprintln!(
                    "simulation nondeterminism seed={seed}: first differing trace line {line}\n  first: {a:?}\n  second: {b:?}"
                );
                failed = true;
            }
            if run.failure != again.failure {
                eprintln!(
                    "simulation nondeterminism seed={seed}: outcomes differ\n  first: {:?}\n  second: {:?}",
                    run.failure, again.failure
                );
                failed = true;
            }
        }
        if failed {
            failures += 1;
            eprintln!("simulation failure seed={seed} test={name}");
            eprintln!(
                "  cause: {}",
                run.failure
                    .as_deref()
                    .unwrap_or("determinism self-check failed")
            );
            eprintln!(
                "  fresh seed: TIDE_SIM_SEED={seed} <test-binary> --exact {name} --nocapture"
            );
            // Includes the ordered sink output; success seeds keep their output in memory only.
            eprint!("{}", run.trace);
        }
    }
    eprintln!(
        "simulation {name}: {} passed; {failures} failed ({count} seeds)",
        count - failures
    );
    assert_eq!(failures, 0, "simulation {name} failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_difference_names_the_first_changed_or_missing_line() {
        assert_eq!(
            first_difference("a\nb\n", "a\nc\n"),
            Some((2, "b\n", "c\n"))
        );
        assert_eq!(
            first_difference("a\n", "a\nb\n"),
            Some((2, "<end of trace>", "b\n"))
        );
        assert_eq!(
            first_difference("a\nb\n", "a\n"),
            Some((2, "b\n", "<end of trace>"))
        );
        assert_eq!(first_difference("a", "a\n"), Some((1, "a", "a\n")));
        assert_eq!(first_difference("", ""), None);
    }

    #[test]
    fn process_failure_and_timeout_keep_the_last_seed() {
        let path = concat!(
            module_path!(),
            "::process_failure_and_timeout_keep_the_last_seed"
        );
        let name = path.split_once("::").unwrap().1;
        if let Ok(mode) = std::env::var(CHILD) {
            println!("\n{START}47");
            io::stdout().flush().unwrap();
            if mode == "timeout" {
                std::thread::sleep(Duration::from_secs(60));
            } else {
                std::process::abort();
            }
            return;
        }
        let aborted = child("abort", name, Duration::from_secs(10)).unwrap();
        assert!(!aborted.status.success());
        assert!(!aborted.timed_out);
        assert_eq!(aborted.last_seed, Some(47));
        let timed = child("timeout", name, Duration::from_secs(5)).unwrap();
        assert!(!timed.status.success());
        assert!(timed.timed_out);
        assert_eq!(timed.last_seed, Some(47));
    }
    #[test]
    fn each_seed_gets_a_fresh_timeout_budget() {
        let path = concat!(module_path!(), "::each_seed_gets_a_fresh_timeout_budget");
        let name = path.split_once("::").unwrap().1;
        if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
            for seed in 0..6 {
                println!("\n{START}{seed}");
                io::stdout().flush().unwrap();
                std::thread::sleep(Duration::from_secs(1));
            }
            return;
        }
        let output = child(path, name, Duration::from_secs(5)).unwrap();
        assert!(output.status.success());
        assert!(!output.timed_out);
        assert_eq!(output.last_seed, Some(5));
    }

    #[test]
    fn missing_summary_reports_the_last_started_seed() {
        let path = concat!(
            module_path!(),
            "::missing_summary_reports_the_last_started_seed"
        );
        if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
            println!("\n{START}47");
            io::stdout().flush().unwrap();
            return;
        }
        let panic = catch_unwind(|| __run(path, 1, |_| {})).unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("last started seed: 47"), "{message}");
        assert!(message.contains("missing summary"), "{message}");
    }

    #[test]
    fn self_check_reports_sink_divergence_and_fails_the_child() {
        let path = concat!(
            module_path!(),
            "::self_check_reports_sink_divergence_and_fails_the_child"
        );
        let name = path.split_once("::").unwrap().1;
        if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
            fn uncontrolled(_sim: SimRoot) {
                static NEXT: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(0);
                let value = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                crate::__sink::out(format_args!("{value}"), false);
            }
            run_seeds(name, 1, uncontrolled);
            return;
        }
        let output = child(path, name, Duration::from_secs(10)).unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("simulation nondeterminism seed=0: first differing trace line 1"),
            "{stderr}"
        );
        assert!(
            stderr.contains("first:") && stderr.contains("second:") && stderr.contains("runner.rs"),
            "{stderr}"
        );
        assert!(stderr.contains("simulation failure seed=0"), "{stderr}");
    }
}
