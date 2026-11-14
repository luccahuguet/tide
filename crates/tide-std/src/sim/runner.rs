use crate::test::{Config, Gen};
use std::io::{self, BufRead, Read, Write};
#[cfg(test)]
use std::panic::catch_unwind;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CHILD: &str = "__TIDE_SIM_CHILD";
const START: &str = "TIDE_SIM_SEED_START ";
const TIMEOUT: Duration = Duration::from_secs(300);

/// Internal entry for `#[sim_test]`. `TIDE_SIM_SEED` selects one fresh decimal u64 seed;
/// it performs no source/build identity check and is not manifest replay.
#[doc(hidden)]
pub fn __run(path: &str, config: Config, body: fn(&mut Gen) -> Result<(), String>) {
    let name = path.split_once("::").map_or(path, |(_, name)| name);
    if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
        crate::test::run_config(path, config, body);
        return;
    }
    verdict(name, config.should_panic, child(path, name, TIMEOUT));
}

// The parent's verdict on its re-executed child, panicking exactly when the test should fail
// (or, under `should_panic`, exactly when it should pass).
fn verdict(name: &str, should_panic: bool, launched: io::Result<ChildOutput>) {
    let output = match launched {
        Ok(output) => output,
        // A child that never ran is no seed's panic.
        Err(error) if should_panic => {
            std::eprintln!(
                "simulation {name}: should_panic not satisfied; could not run simulation child: {error:?}"
            );
            return;
        }
        Err(error) => panic!("could not run simulation child: {error:?}"),
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    let prefix = format!("simulation {name}: ");
    let summary = stderr
        .lines()
        .find(|line| line.starts_with(&prefix) && line.contains(" seeds)"));
    if should_panic {
        // The child's libtest applies should_panic (and `expected`) to the failing seed's own
        // payload, which the child also prints as its cause, so the parent mirrors that verdict:
        // it panics with the child's report on a pass and returns, failing as "did not panic",
        // otherwise. A timeout, signal, missing summary or vacuous run never counts as the panic,
        // nor does a seed that returned an error or failed the determinism self-check (the child
        // returns instead of re-raising); a test's own requires failing is a failing seed here:
        // should_panic never discards.
        let seed_failed = summary
            .is_some_and(|line| line.contains(" failed (") && !line.contains("; 0 failed ("));
        if seed_failed && !output.timed_out && output.status.success() {
            panic!("{stderr}");
        }
        std::eprint!("{stderr}");
        std::eprintln!(
            "simulation {name}: should_panic not satisfied (last started seed: {}); status: {}{}",
            output
                .last_seed
                .map_or_else(|| "none".into(), |seed| seed.to_string()),
            output.status,
            if output.timed_out {
                "; per-evaluation wall-clock timeout"
            } else {
                ""
            },
        );
        return;
    }
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
                "per-evaluation wall-clock timeout"
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
    if summary.is_some_and(|line| line.starts_with(&format!("{prefix}vacuous "))) {
        panic!("simulation {name} failed as vacuous");
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

pub(crate) fn start(seed: u64) {
    println!("\n{START}{seed}");
    io::stdout()
        .flush()
        .expect("cannot report simulation seed start");
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let panic = catch_unwind(|| {
            __run(
                path,
                Config {
                    cases: 1,
                    simulation: true,
                    ..Config::default()
                },
                |_| Ok(()),
            )
        })
        .unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("last started seed: 47"), "{message}");
        assert!(message.contains("missing summary"), "{message}");
    }

    #[test]
    fn should_panic_never_accepts_a_child_that_did_not_run() {
        let launch = || Err(io::Error::from(io::ErrorKind::Other));
        verdict("quiet", true, launch());
        let panic = catch_unwind(|| verdict("quiet", false, launch())).unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(
            message.contains("could not run simulation child"),
            "{message}"
        );
        let exited = |code| <ExitStatus as std::os::unix::process::ExitStatusExt>::from_raw(code);
        let output = |code| {
            Ok(ChildOutput {
                status: exited(code),
                stderr: b"simulation quiet: 0 passed; 1 failed (1 seeds)\n".to_vec(),
                last_seed: Some(0),
                timed_out: false,
            })
        };
        assert!(catch_unwind(|| verdict("quiet", true, output(0))).is_err());
        verdict("quiet", true, output(101 << 8));
    }

    fn should_panic_run(
        path: &str,
        seeds: u64,
        body: fn(&mut Gen) -> Result<(), String>,
    ) -> Option<String> {
        let config = Config {
            cases: seeds,
            simulation: true,
            should_panic: true,
            ..Config::default()
        };
        // `None`: run_config returned, so libtest fails the test as "did not panic".
        catch_unwind(|| crate::test::run_config(path, config, body))
            .err()
            .map(|panic| panic.downcast_ref::<&str>().unwrap().to_string())
    }

    #[test]
    fn should_panic_never_accepts_a_failure_that_did_not_panic() {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        let path = concat!(
            module_path!(),
            "::should_panic_never_accepts_a_failure_that_did_not_panic"
        );
        // Returns normally, but its sink differs on the self-check's second run.
        fn diverging(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            static RUNS: AtomicUsize = AtomicUsize::new(0);
            crate::__sink::out(format_args!("{}", RUNS.fetch_add(1, Relaxed)), false);
            Ok(())
        }
        // Panics as expected on every run; seed 0's two runs agree, later runs diverge.
        fn masked(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            static RUNS: AtomicUsize = AtomicUsize::new(0);
            let run = RUNS.fetch_add(1, Relaxed);
            if run >= 2 {
                crate::__sink::out(format_args!("{run}"), false);
            }
            panic!("expected");
        }
        fn returns_err(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            Err("no panic here".into())
        }
        // Seed 0 (runs 0 and 1) returns an error; later seeds panic as expected.
        fn err_then_panic(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            static RUNS: AtomicUsize = AtomicUsize::new(0);
            if RUNS.fetch_add(1, Relaxed) < 2 {
                return Err("no panic here".into());
            }
            panic!("expected");
        }
        assert_eq!(should_panic_run(path, 1, diverging), None);
        assert_eq!(should_panic_run(path, 2, masked), None);
        assert_eq!(should_panic_run(path, 2, returns_err), None);
        assert_eq!(should_panic_run(path, 2, err_then_panic), None);
        // The property path, which shares the runner, holds the same rule.
        let property = |body: fn(&mut Gen) -> Result<(), String>| {
            let config = Config {
                should_panic: true,
                ..Config::default()
            };
            catch_unwind(|| crate::test::run_config(path, config, body)).is_err()
        };
        assert!(!property(|_| Err("no panic here".into())));
        assert!(property(|_| panic!("expected")));
    }

    #[test]
    fn should_panic_reraises_the_first_failing_seeds_panic() {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        let path = concat!(
            module_path!(),
            "::should_panic_reraises_the_first_failing_seeds_panic"
        );
        fn steady(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            panic!("expected");
        }
        // Seed 0 (runs 0 and 1) passes; later seeds panic: not every seed has to panic.
        fn quiet_then_expected(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            static RUNS: AtomicUsize = AtomicUsize::new(0);
            if RUNS.fetch_add(1, Relaxed) >= 2 {
                panic!("expected");
            }
            Ok(())
        }
        // Seed 1 (runs 2 and 3) panics "wrong" before later seeds panic "expected".
        fn wrong_then_expected(g: &mut Gen) -> Result<(), String> {
            let _sim = g.simulation();
            static RUNS: AtomicUsize = AtomicUsize::new(0);
            match RUNS.fetch_add(1, Relaxed) {
                0 | 1 => Ok(()),
                2 | 3 => panic!("wrong"),
                _ => panic!("expected"),
            }
        }
        assert_eq!(
            should_panic_run(path, 2, steady).as_deref(),
            Some("expected")
        );
        assert_eq!(
            should_panic_run(path, 3, quiet_then_expected).as_deref(),
            Some("expected")
        );
        assert_eq!(
            should_panic_run(path, 3, wrong_then_expected).as_deref(),
            Some("wrong")
        );
    }

    #[test]
    fn self_check_reports_sink_divergence_and_fails_the_child() {
        let path = concat!(
            module_path!(),
            "::self_check_reports_sink_divergence_and_fails_the_child"
        );
        let name = path.split_once("::").unwrap().1;
        if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(path)) {
            fn uncontrolled(g: &mut Gen) -> Result<(), String> {
                let _sim = g.simulation();
                static NEXT: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(0);
                let value = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                crate::__sink::out(format_args!("{value}"), false);
                Ok(())
            }
            crate::test::run_config(
                path,
                Config {
                    cases: 1,
                    simulation: true,
                    ..Config::default()
                },
                uncontrolled,
            );
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
