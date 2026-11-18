// Seed witnesses for research/40 §12's table, each paired with its deliberately broken control
// (process/sim.rs `Broken`), plus the real backend's PROC-ONLY, lookup, canonical-exec and
// descriptor checks. A witness returns whether every outcome it names was observed.
use super::sim::{BROKEN, Broken};
use super::*;
use crate::io::{ErrorKind as K, Read, Write};
use crate::sim::{ProcessModel as M, SimRoot};
use crate::task::TaskError;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn under<R>(control: Option<Broken>, f: impl FnOnce() -> R) -> R {
    BROKEN.set(control);
    let r = catch_unwind(AssertUnwindSafe(f));
    BROKEN.set(None);
    r.unwrap_or_else(|p| std::panic::resume_unwind(p))
}

/// The witness holds as built and fails under its broken control.
fn witness(control: Broken, check: fn() -> bool) {
    assert!(under(None, check), "witness failed without a control");
    assert!(!under(Some(control), check), "control {control:?} did not fail the witness");
}

fn null(cmd: &mut Command) -> &mut Command {
    cmd.stdin(Stdio::null())
}

/// Makes the listed system calls fail with EPERM in this thread and the processes it forks, with
/// a seccomp filter, which an unprivileged process may install after PR_SET_NO_NEW_PRIVS.
fn deny_syscalls(numbers: &[u32]) {
    #[repr(C)]
    struct Filter(u16, u8, u8, u32);
    #[repr(C)]
    struct Program(u16, *const Filter);
    let n = numbers.len();
    let mut filter = vec![Filter(0x20, 0, 0, 0)]; // BPF_LD|W|ABS: seccomp_data.nr
    for (i, &nr) in numbers.iter().enumerate() {
        filter.push(Filter(0x15, (n - i) as u8, 0, nr)); // BPF_JMP|JEQ|K: to the EPERM return
    }
    filter.push(Filter(0x06, 0, 0, 0x7fff_0000)); // SECCOMP_RET_ALLOW
    filter.push(Filter(0x06, 0, 0, 0x0005_0000 | 1)); // SECCOMP_RET_ERRNO | EPERM
    let program = Program(filter.len() as u16, filter.as_ptr());
    // SAFETY: PR_SET_NO_NEW_PRIVS (38) and PR_SET_SECCOMP (22) with SECCOMP_MODE_FILTER (2)
    // and a filter program that outlives the call.
    unsafe {
        assert_eq!(prctl(38, 1u64, 0u64, 0u64, 0u64), 0, "{}", std::io::Error::last_os_error());
        assert_eq!(prctl(22, 2u64, &program as *const Program, 0u64, 0u64), 0, "{}", std::io::Error::last_os_error());
    }
}

unsafe extern "C" {
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    fn prctl(option: i32, ...) -> i32;
}

/// Reruns test `name` alone in a child test process, with `env` added, because it changes
/// process-wide state; true in the parent, which checks the child ran it and passed.
fn isolated(name: &str, env: &[(&str, std::ffi::OsString)]) -> bool {
    if std::env::var_os("TIDE_PROC_ISOLATED").is_some() {
        return false;
    }
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &format!("process::tests::{name}"), "--nocapture", "--test-threads=1"])
        .env("TIDE_PROC_ISOLATED", "1")
        .envs(env.iter().map(|(k, v)| (k, v)))
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success() && text.contains(&format!("ISOLATED_RAN {name}")), "{text}{}", String::from_utf8_lossy(&out.stderr));
    true
}

fn panics_with(f: impl FnOnce(), text: &str) -> bool {
    match catch_unwind(AssertUnwindSafe(f)) {
        Err(p) => p
            .downcast_ref::<String>()
            .map(String::as_str)
            .or(p.downcast_ref::<&str>().copied())
            .is_some_and(|m| m.contains(text)),
        Ok(()) => false,
    }
}

// research/30 §8: output closes while the child runs, then it exits; it exits with bytes unread.
fn eof_is_not_exit() -> bool {
    (0..8).all(|seed| {
        let sim = crate::Root::sim(seed);
        sim.world.proc("w", M::new().write("ab").close_stdout().delay(ms(5), ms(5)));
        sim.world.proc("x", M::new().write("cd").exit(2));
        let mut child = null(&mut sim.proc.command("w")).stdout(Stdio::piped()).spawn().unwrap();
        let mut text = Vec::new();
        child.stdout.take().unwrap().read_to_end(&mut text).unwrap();
        let running = child.try_wait().unwrap().is_none();
        let ok = child.wait().unwrap().code() == Some(0);
        let mut child = null(&mut sim.proc.command("x")).stdout(Stdio::piped()).spawn().unwrap();
        let code = child.wait().unwrap().code();
        let mut late = Vec::new();
        child.stdout.take().unwrap().read_to_end(&mut late).unwrap();
        text == b"ab" && running && ok && code == Some(2) && late == b"cd"
    })
}
#[test]
fn eof_does_not_reap() {
    witness(Broken::EofReaps, eof_is_not_exit);
}
#[test]
fn exit_keeps_buffered_output() {
    witness(Broken::DropOutputOnExit, eof_is_not_exit);
}

// research/30 §8: a deadline and a process timer due at once fire deadline first.
#[test]
fn deadline_fires_before_a_process_timer_at_the_same_instant() {
    witness(Broken::WaitIgnoresDeadline, || {
        (0..8).all(|seed| {
            let sim = crate::Root::sim(seed);
            sim.world.proc("t", M::new().delay(ms(5), ms(5)).exit(0));
            let mut result = None;
            let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
                s.set_deadline(sim.clock.now() + ms(5));
                result = Some(null(&mut sim.proc.command("t")).status());
                Ok(())
            });
            result.unwrap().is_err_and(|e| e.is_cancelled())
        })
    });
}

// research/30 §8: a try_wait spin with a future model timer hits the stall guard.
#[test]
fn try_wait_spin_hits_the_stall_guard() {
    witness(Broken::UntimedDelay, || {
        let sim = crate::Root::sim(0);
        sim.tasks.set_stall_limit(50);
        sim.world.proc("slow", M::new().delay(ms(1000), ms(1000)));
        let mut child = null(&mut sim.proc.command("slow")).spawn().unwrap();
        panics_with(
            || {
                for _ in 0..200 {
                    child.try_wait().unwrap();
                }
            },
            "simulation stalled",
        )
    });
}

// research/30 §8: a failed spawn creates no instance; kill ends the child; drop neither kills
// nor reaps.
fn spawn_kill_drop() -> bool {
    let sim = crate::Root::sim(1);
    sim.world.proc("gone", M::missing());
    sim.world.proc("long", M::new().delay(ms(1000), ms(1000)));
    sim.world.proc("bg", M::new().delay(ms(1), ms(1)).write_err("done\n"));
    let missing = sim.proc.command("gone").spawn().unwrap_err().kind() == K::NotFound;
    let mut child = null(&mut sim.proc.command("long")).spawn().unwrap();
    let first = child.id() == 1;
    child.kill().unwrap();
    let status = child.wait().unwrap();
    let killed = status.code().is_none() && !status.success() && status.to_string() == "signal: 9 (SIGKILL)";
    drop(null(&mut sim.proc.command("bg")).spawn().unwrap());
    sim.clock.sleep(ms(2)).unwrap();
    let ran = sim.records().iter().any(|r| r.message == "done");
    missing && first && killed && ran
}
#[test]
fn failed_spawn_creates_no_instance() {
    witness(Broken::MissingCreatesInstance, spawn_kill_drop);
}
#[test]
fn kill_ends_the_child() {
    witness(Broken::KillIgnored, spawn_kill_drop);
}
#[test]
fn dropping_a_child_neither_kills_nor_reaps() {
    witness(Broken::DropKills, spawn_kill_drop);
}

// research/30 §8: a missing model or unsupported mode is a coverage failure; no host process.
#[test]
fn unmodelled_programs_and_modes_stop_the_seed() {
    witness(Broken::UnregisteredIsNotFound, || {
        let unregistered = panics_with(
            || {
                let sim = crate::Root::sim(0);
                let _ = null(&mut sim.proc.command("nope")).spawn();
            },
            "unmodelled effect: proc.spawn of `nope`",
        );
        let inherited = panics_with(
            || {
                let sim = crate::Root::sim(0);
                sim.world.proc("a", M::new());
                let _ = sim.proc.command("a").spawn();
            },
            "inherited stdin is not a supported mode",
        );
        // An unmatched input stops the seed even if nothing observes the child again.
        let input = panics_with(
            || {
                let sim = crate::Root::sim(0);
                sim.world.proc("r", M::new().read("yes"));
                let mut child = sim.proc.command("r").stdin(Stdio::piped()).spawn().unwrap();
                child.stdin.as_mut().unwrap().write_all(b"no!").unwrap();
                drop(child);
                sim.clock.sleep(ms(1)).unwrap();
            },
            "unmatched input",
        );
        unregistered && inherited && input
    });
}

fn scenario(seed: u64) -> String {
    let sim = crate::Root::sim(seed);
    sim.world.proc(
        "cat",
        M::new().delay(ms(0), ms(3)).copy().choose([M::new().exit(0), M::new().write_err("x").exit(1)]),
    );
    let mut child = sim.proc.command("cat").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(&[7; 3000]).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.stdout, [7; 3000]);
    sim.trace()
}

// research/30 §8: the same identity and seed give a byte-identical trace.
#[test]
fn same_seed_same_trace() {
    witness(Broken::HostClock, || (0..4).all(|seed| scenario(seed) == scenario(seed)));
    assert_ne!(scenario(0), scenario(1));
}

// §6: repeated wait and try_wait return the cached status.
#[test]
fn repeated_waits_return_the_cached_status() {
    witness(Broken::NoCache, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("three", M::new().exit(3));
        let mut child = null(&mut sim.proc.command("three")).spawn().unwrap();
        let a = child.wait().ok().and_then(|s| s.code());
        let b = child.wait().ok().and_then(|s| s.code());
        let c = child.try_wait().ok().flatten().and_then(|s| s.code());
        [a, b, c] == [Some(3); 3]
    });
}

// §6: the child may exit before the caller resumes from spawn.
#[test]
fn a_child_can_exit_before_spawn_returns() {
    witness(Broken::SpawnNoPoint, || {
        (0..32).any(|seed| {
            let sim = crate::Root::sim(seed);
            sim.world.proc("quick", M::new());
            let mut child = null(&mut sim.proc.command("quick")).spawn().unwrap();
            child.try_wait().unwrap().is_some()
        })
    });
}

// §6: after kill, either the model's own exit or termination runs first.
fn kill_race() -> bool {
    let (mut pending, mut normal, mut killed) = (false, false, false);
    for seed in 0..64 {
        let sim = crate::Root::sim(seed);
        sim.world.proc("three", M::new().exit(3));
        let mut child = null(&mut sim.proc.command("three")).spawn().unwrap();
        child.kill().unwrap();
        let submitted = sim.trace().contains("SIGKILL submitted");
        pending |= child.try_wait().unwrap().is_none();
        let status = child.wait().unwrap();
        normal |= submitted && status.code() == Some(3);
        killed |= status.code().is_none() && status.to_string() == "signal: 9 (SIGKILL)";
    }
    pending && normal && killed
}
#[test]
fn kill_submits_termination_as_a_ready_action() {
    witness(Broken::KillInline, kill_race);
}
#[test]
fn a_normal_exit_can_win_against_kill() {
    witness(Broken::TerminationOverrides, kill_race);
}

// §6: short reads and writes, sizes drawn from the process sub-stream.
#[test]
fn transfers_can_be_short() {
    witness(Broken::FullTransfer, || {
        let (mut short_write, mut short_read) = (false, false);
        for seed in 0..8 {
            let sim = crate::Root::sim(seed);
            sim.world.proc("echo", M::new().read([1; 1000]).write([2; 1000]));
            let mut child = sim.proc.command("echo").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            let mut stdin = child.stdin.take().unwrap();
            let n = stdin.write(&[1; 1000]).unwrap();
            short_write |= n < 1000;
            stdin.write_all(&[1; 1000][n..]).unwrap();
            let mut buf = [0; 1000];
            let mut stdout = child.stdout.take().unwrap();
            let m = stdout.read(&mut buf).unwrap();
            short_read |= m < 1000;
            stdout.read_exact(&mut buf[m..]).unwrap();
            assert_eq!(buf, [2; 1000]);
            assert!(child.wait().unwrap().success());
        }
        short_write && short_read
    });
}

// §6: a model's write to a closed parent end dies with no code, or, if declared, continues.
#[test]
fn a_write_to_a_closed_pipe_kills_by_default_or_continues_when_declared() {
    witness(Broken::SigpipeCode, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("die", M::new().delay(ms(1), ms(1)).write("z"));
        sim.world.proc("live", M::new().ignore_sigpipe().delay(ms(1), ms(1)).write("z").exit(4));
        let mut child = null(&mut sim.proc.command("die")).stdout(Stdio::piped()).spawn().unwrap();
        drop(child.stdout.take());
        let died = child.wait().unwrap();
        let mut child = null(&mut sim.proc.command("live")).stdout(Stdio::piped()).spawn().unwrap();
        drop(child.stdout.take());
        let lived = child.wait().unwrap();
        died.code().is_none() && died.to_string() == "signal: 13 (SIGPIPE)" && lived.code() == Some(4)
    });
}

// §6: a parent write after the model closed its input returns BrokenPipe.
#[test]
fn writing_to_a_closed_input_is_broken_pipe() {
    witness(Broken::SilentWrite, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("shut", M::new().close_stdin().delay(ms(5), ms(5)));
        let mut child = sim.proc.command("shut").stdin(Stdio::piped()).spawn().unwrap();
        sim.clock.sleep(ms(1)).unwrap();
        let r = child.stdin.as_mut().unwrap().write(b"x");
        child.wait().unwrap();
        r.is_err_and(|e| e.kind() == K::BrokenPipe)
    });
}

// §6: a parent write into a full pipe blocks until the model reads.
#[test]
fn a_full_pipe_applies_backpressure() {
    witness(Broken::Unbounded, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("lazy", M::new().delay(ms(10), ms(10)).read(vec![0; 70_000]));
        let mut child = sim.proc.command("lazy").stdin(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(&[0; 70_000]).unwrap();
        let blocked = sim.elapsed() >= ms(10);
        child.wait().unwrap();
        blocked
    });
}

// §6: wait closes the retained stdin, so the model sees input closure; try_wait does not.
#[test]
fn wait_closes_stdin_and_try_wait_does_not() {
    witness(Broken::TryWaitClosesStdin, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("eof", M::new().read_to_end());
        let mut child = sim.proc.command("eof").stdin(Stdio::piped()).spawn().unwrap();
        let polled = (0..20).all(|_| child.try_wait().unwrap().is_none());
        polled && child.wait().unwrap().success()
    });
}

// 03 §1.1: output or status cancelled before the spawn starts nothing.
#[test]
fn a_cancelled_call_before_spawn_starts_nothing() {
    witness(Broken::OutputSkipsCheck, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("p", M::new());
        let mut cancelled = false;
        let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
            s.cancel();
            cancelled = null(&mut sim.proc.command("p")).status().is_err_and(|e| e.is_cancelled());
            Ok(())
        });
        let next = null(&mut sim.proc.command("p")).spawn().unwrap().id();
        cancelled && next == 1
    });
}

// 03 §1.1: cancelled after the spawn, the call kills, waits shielded, then returns Cancelled.
#[test]
fn a_cancelled_call_after_spawn_kills_and_waits() {
    witness(Broken::NoCleanupWait, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("late", M::new().delay(ms(10), ms(10)).write_err("late\n"));
        let (mut cancelled, mut reaped) = (false, false);
        let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
            s.set_deadline(sim.clock.now() + ms(1));
            cancelled = null(&mut sim.proc.command("late")).stderr(Stdio::inherit()).output().is_err_and(|e| e.is_cancelled());
            // Before the call returned, the termination ran and the shielded wait collected it.
            reaped = sim.trace().contains("signal 9");
            Ok(())
        });
        sim.clock.sleep(ms(20)).unwrap();
        cancelled && reaped && sim.records().iter().all(|r| r.message != "late")
    });
}

// 03 §1.1, TASK-3: a wait that completes before the cancellation takes effect returns its own
// result. Witness: no seed returns Cancelled when the waiting task acted between its wake and
// the cancellation; and some seed returns the result although the cancel follows at once.
#[test]
fn a_completed_wait_keeps_its_result() {
    witness(Broken::RecheckAfterWait, || {
        let (mut kept, mut sound) = (false, true);
        for seed in 0..64 {
            let sim = crate::Root::sim(seed);
            sim.world.proc("five", M::new().delay(ms(5), ms(5)).exit(7));
            let mut result = None;
            let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
                let proc = &sim.proc;
                let r = &mut result;
                s.spawn(move || {
                    *r = Some(null(&mut proc.command("five")).status());
                    Ok(())
                });
                sim.clock.sleep_until(sim.clock.now() + ms(5)).unwrap();
                s.cancel();
                Ok(())
            });
            match result.unwrap() {
                Ok(status) => kept |= status.code() == Some(7),
                Err(_) => {
                    // The waiter (task 1) must not have run past its wake before the cancel.
                    let trace = sim.trace();
                    let lines: Vec<&str> = trace.lines().collect();
                    let wake = lines.iter().position(|l| l.contains("\"task\":1,") && l.contains("proc.wait") && l.contains("\"action\":\"wake\""));
                    let cancel = lines.iter().position(|l| l.contains("\"action\":\"cancelled\""));
                    if let (Some(w), Some(c)) = (wake, cancel)
                        && w < c
                    {
                        sound &= !lines[w + 1..c].iter().any(|l| l.contains("\"task\":1,"));
                    }
                }
            }
        }
        kept && sound
    });
}

// 03 §1.1: a cancelled wait leaves the Child with the caller, which can still kill and wait it.
#[test]
fn a_cancelled_wait_leaves_the_child_with_the_caller() {
    witness(Broken::CancelDropsChild, || {
        let sim = crate::Root::sim(0);
        sim.world.proc("long", M::new().delay(ms(1000), ms(1000)));
        let mut child = null(&mut sim.proc.command("long")).spawn().unwrap();
        let mut cancelled = false;
        let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
            s.set_deadline(sim.clock.now() + ms(1));
            cancelled = child.wait().is_err_and(|e| e.is_cancelled());
            Ok(())
        });
        child.kill().unwrap();
        // A cancelled wait_with_output leaves its child with the caller too.
        sim.world.proc("piped", M::new().delay(ms(1000), ms(1000)));
        let mut piped = null(&mut sim.proc.command("piped")).stdout(Stdio::piped()).spawn().unwrap();
        let mut output_cancelled = false;
        let _ = sim.tasks.scope::<_, TaskError, _>(|s| {
            s.cancel();
            output_cancelled = piped.wait_with_output().is_err_and(|e| e.is_cancelled());
            Ok(())
        });
        piped.kill().unwrap();
        cancelled
            && child.wait().is_ok_and(|s| s.code().is_none())
            && output_cancelled
            && piped.stdout.is_none()
            && piped.wait().is_ok_and(|s| s.code().is_none())
    });
}

// TASK-3: a wait entered in a cancelled scope is cancelled, even with the status cached or no
// pipe left to read; the child and its status stay with the caller. Both backends.
fn cancelled_waits(tasks: &crate::task::Tasks, child: &mut Child) {
    let status = child.wait().unwrap();
    let mut cancelled = (false, false);
    let _ = tasks.scope::<_, TaskError, _>(|s| {
        s.cancel();
        cancelled = (child.wait().is_err_and(|e| e.is_cancelled()), child.wait_with_output().is_err_and(|e| e.is_cancelled()));
        Ok(())
    });
    assert_eq!(cancelled, (true, true));
    assert_eq!(child.wait().unwrap(), status);
}
#[test]
fn a_wait_in_a_cancelled_scope_is_cancelled_even_when_cached() {
    {
        let sim = crate::Root::sim(0);
        sim.world.proc("p", M::new());
        cancelled_waits(&sim.tasks, &mut null(&mut sim.proc.command("p")).spawn().unwrap());
    }
    let tasks = crate::task::Tasks::real();
    cancelled_waits(&tasks, &mut Proc::new(None).command("true").stdin(Stdio::null()).spawn().unwrap());
}

// §6: a descriptor the model closed stays closed whatever it is routed to.
#[test]
fn a_model_cannot_use_a_descriptor_it_closed() {
    for route in [Stdio::inherit, Stdio::null, Stdio::piped] {
        let write = || {
            let sim = crate::Root::sim(0);
            sim.world.proc("w", M::new().close_stdout().write("after close\n"));
            let _ = null(&mut sim.proc.command("w")).stdout(route()).spawn().unwrap().wait();
        };
        assert!(panics_with(write, "write after the model closed descriptor 1"));
    }
    for route in [Stdio::null, Stdio::piped] {
        let read = || {
            let sim = crate::Root::sim(0);
            sim.world.proc("r", M::new().close_stdin().read_to_end());
            let _ = sim.proc.command("r").stdin(route()).spawn().unwrap().wait();
        };
        assert!(panics_with(read, "read after the model closed descriptor 0"));
    }
}

// A panic inside the simulator's model task fails the seed; it is never a silent success.
#[test]
fn a_model_panic_fails_the_seed() {
    let run = || {
        let sim = crate::Root::sim(0);
        sim.world.proc("p", M::new());
        let _ = null(&mut sim.proc.command("p")).spawn().unwrap().wait();
    };
    assert!(under(Some(Broken::ModelPanics), || panics_with(run, "the model panicked: injected model panic")));
    // An accepted model whose delay passes the end of virtual time fails the seed.
    assert!(panics_with(
        || {
            let sim = crate::Root::sim(0);
            sim.clock.sleep(Duration::from_nanos(1)).unwrap();
            sim.world.proc("p", M::new().delay(Duration::MAX, Duration::MAX));
            let _ = null(&mut sim.proc.command("p")).spawn().unwrap().wait();
        },
        "passes the end of virtual time"
    ));
}

// research/40 §7, §12: a registered model's profile and data are root inputs of the replay
// manifest, which replay compares, so replay refuses changed model data.
#[test]
fn model_data_enters_the_replay_manifest() {
    witness(Broken::OmitManifest, || {
        let inputs = |model: M| {
            crate::sim::manifest::begin("", 3, 0, crate::test::Config::default(), false);
            let sim = crate::Root::sim(3);
            sim.world.proc("cat", model);
            let inputs = crate::sim::manifest::recorded_inputs();
            drop(sim);
            crate::sim::manifest::finish(&[], "returned", false);
            inputs
        };
        let copy = inputs(M::new().copy());
        let recorded = copy.iter().any(|(kind, key, value)| {
            kind == "proc" && key == b"cat" && value.starts_with(b"proc-lifecycle-v0 schema=1 capacity=65536 ")
        });
        recorded && copy != inputs(M::new().write("x"))
    });
}

// PROC-2 under simulation: only, then refused variables, then NUL, all before model lookup.
#[test]
fn simulated_refusals_precede_model_lookup() {
    let sim = crate::Root::sim(0);
    sim.world.proc("git", M::new());
    let only = sim.proc.only(["git", "./rel", "", "/usr/bin/git"]).only(["git", "/usr/bin/git", "sh"]);
    let kind = |c: &mut Command| null(c).spawn().unwrap_err().kind();
    assert_eq!(kind(&mut only.command("sh")), K::PermissionDenied);
    assert_eq!(kind(&mut only.command("./rel")), K::PermissionDenied);
    assert_eq!(kind(&mut only.command("nope")), K::PermissionDenied); // never reaches coverage
    for var in ["LD_PRELOAD", "DYLD_INSERT_LIBRARIES", "GCONV_PATH", "GLIBC_TUNABLES", "MALLOC_TOP_PAD_", "TMPDIR"] {
        assert_eq!(kind(only.command("git").env(var, "x")), K::PermissionDenied, "{var}");
    }
    // Under unattenuated Proc a loader variable passes; the model accepts it.
    assert!(null(sim.proc.command("git").env("LD_PRELOAD", "x")).spawn().is_ok());
    // Removed or cleared variables are not set.
    assert!(null(only.command("git").env("LD_PRELOAD", "x").env_remove("LD_PRELOAD")).spawn().is_ok());
    assert_eq!(kind(only.command("git").arg("a\0b")), K::InvalidInput);
    assert_eq!(kind(only.command("git").env("A", "b\0")), K::InvalidInput);
    assert_eq!(kind(only.command("git").current_dir("/a\0")), K::InvalidInput);
    // A refused variable precedes NUL.
    assert_eq!(kind(only.command("git").env("LD_X", "1").arg("\0")), K::PermissionDenied);
    // A key holding `=` is refused, since the C library would read `GCONV_PATH` from
    // `GCONV_PATH=/x`: before model lookup (`unmodelled` has no model) and before NUL.
    let eq = sim.proc.only(["unmodelled"]);
    for key in ["GCONV_PATH=/tmp/injected", "GLIBC_TUNABLES=glibc.malloc.check", "A=B"] {
        assert_eq!(kind(eq.command("unmodelled").env(key, "x").arg("\0")), K::PermissionDenied, "{key}");
    }
    let mut child = null(&mut only.command("git")).spawn().unwrap();
    assert_eq!(child.id(), 3);
    assert!(child.wait().unwrap().success());
}

// PROC-2 on the real backend: lookup, canonical exec through /proc/self/cwd, argv[0], refusals.
// The process starts with `links/bin`, a symlink to `/proc/self/cwd/bin`, first on its PATH.
#[test]
fn real_lookup_and_canonical_exec() {
    let dir = std::env::temp_dir().join(format!("tide-proc-{}", std::process::id()));
    let path = std::env::join_paths(
        std::iter::once(dir.join("links/bin")).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    if isolated("real_lookup_and_canonical_exec", &[("PATH", path), ("TIDE_PROC_DIR", dir.into())]) {
        return;
    }
    let dir = std::path::PathBuf::from(std::env::var_os("TIDE_PROC_DIR").unwrap());
    let _ = std::fs::remove_dir_all(&dir);
    for sub in ["bin", "other/bin", "links"] {
        std::fs::create_dir_all(dir.join(sub)).unwrap();
    }
    let script = |path: &std::path::Path, text: &str| {
        std::fs::write(path, format!("#!/bin/sh\necho {text} \"$0\"\n")).unwrap();
        std::process::Command::new("chmod").arg("+x").arg(path).status().unwrap();
    };
    script(&dir.join("bin/tool"), "parent");
    script(&dir.join("other/bin/tool"), "cwd");
    std::os::unix::fs::symlink("/proc/self/cwd/bin", dir.join("links/bin")).unwrap();
    // The parent's cwd is `dir`, so the link names dir/bin; the child's is `other`, whose
    // bin/tool differs.
    std::env::set_current_dir(&dir).unwrap();
    let target = std::fs::canonicalize(dir.join("bin/tool")).unwrap();
    let parent = format!("parent {}\n", target.display());
    let lookup_path = format!("{}:rel:", dir.join("links/bin").display());
    assert_eq!(real::lookup("tool".as_ref(), Some(lookup_path.into())).unwrap(), dir.join("links/bin/tool"));
    assert_eq!(real::lookup("tool".as_ref(), Some("rel::".into())).unwrap_err().kind(), K::NotFound);
    assert_eq!(real::lookup("tool".as_ref(), None).unwrap_err().kind(), K::NotFound);
    let proc = Proc::new(None);
    // An absolute allowed entry through the link, and a bare name through the start-time PATH:
    // each runs the parent's file; a script's $0 is the canonical path (research/40 §3).
    let entry = dir.join("links/bin/tool");
    let out = proc.only([&entry]).command(&entry).current_dir(dir.join("other")).output().unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), parent);
    let out = proc.only(["tool"]).command("tool").current_dir(dir.join("other")).output().unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), parent);
    // A command's own PATH reaches only the child.
    let out = proc.command("tool").env("PATH", dir.join("other/bin")).current_dir(dir.join("other")).output().unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), parent);
    let only_sh = proc.only(["sh"]);
    for key in ["LD_PRELOAD", "GCONV_PATH=/tmp/injected", "GLIBC_TUNABLES=glibc.malloc.check"] {
        assert_eq!(only_sh.command("sh").env(key, "x").output().unwrap_err().kind(), K::PermissionDenied, "{key}");
    }
    assert_eq!(proc.command("sh").arg("\0").output().unwrap_err().kind(), K::InvalidInput);
    // argv[0] as spelled for a bare name.
    let out = proc.command("sh").args(["-c", "echo $0"]).output().unwrap();
    assert_eq!(out.stdout, b"sh\n");
    std::env::set_current_dir("/").unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    println!("ISOLATED_RAN real_lookup_and_canonical_exec");
}

// A failure after the OS spawn, in spawn's pipe setup or in a hidden child's output, kills and
// reaps the child: none is left running or waitable.
#[test]
fn real_failures_after_spawn_reap_the_child() {
    if isolated("real_failures_after_spawn_reap_the_child", &[]) {
        return;
    }
    for point in ["nonblocking", "drain"] {
        real::FAIL.set(Some(point));
        let err = Proc::new(None).command("sleep").arg("30").output().unwrap_err();
        real::FAIL.set(None);
        assert!(err.to_string().contains(point), "{err}");
        // SAFETY: waitpid with a null status pointer; WNOHANG (1) never blocks.
        let none = unsafe { waitpid(-1, std::ptr::null_mut(), 1) };
        assert_eq!((none, std::io::Error::last_os_error().raw_os_error()), (-1, Some(10)), "{point}: a child remains"); // ECHILD
    }
    println!("ISOLATED_RAN real_failures_after_spawn_reap_the_child");
}

// A failed pipe setup closes every pipe end, including those it had not reached, before the
// shielded wait: with kill denied, a child writing to stderr then fails its write and exits
// instead of blocking that wait forever.
#[test]
fn real_a_failed_setup_closes_every_pipe_before_reaping() {
    if isolated("real_a_failed_setup_closes_every_pipe_before_reaping", &[]) {
        return;
    }
    #[cfg(target_arch = "x86_64")]
    const KILL: u32 = 62;
    #[cfg(target_arch = "aarch64")]
    const KILL: u32 = 129;
    deny_syscalls(&[KILL, 424]); // kill, pidfd_send_signal
    let (done, result) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // Setup fails at stdout, leaving stderr not yet set up.
        real::FAIL.set(Some("nonblocking"));
        let r = Proc::new(None).command("sh").args(["-c", "head -c 1048576 /dev/zero >&2"]).output();
        done.send(r.map(|_| ()).map_err(|e| e.to_string())).unwrap();
    });
    let Ok(r) = result.recv_timeout(Duration::from_secs(20)) else {
        println!("the shielded wait blocked on a child holding a retained pipe");
        std::process::exit(1);
    };
    assert!(r.unwrap_err().contains("nonblocking"));
    // SAFETY: waitpid with a null status pointer; WNOHANG (1) never blocks.
    let none = unsafe { waitpid(-1, std::ptr::null_mut(), 1) };
    assert_eq!((none, std::io::Error::last_os_error().raw_os_error()), (-1, Some(10))); // ECHILD
    println!("ISOLATED_RAN real_a_failed_setup_closes_every_pipe_before_reaping");
}

// Handles kept from an ended simulation are refused in the next, even with cached status.
#[test]
fn a_pipe_from_another_simulation_is_refused() {
    let kept = || {
        let sim = crate::Root::sim(0);
        sim.world.proc("w", M::new().write("old"));
        let mut child = null(&mut sim.proc.command("w")).stdout(Stdio::piped()).spawn().unwrap();
        child.wait().unwrap();
        child.stdout.take().unwrap()
    };
    let (mut read, drained) = (kept(), kept());
    let refused = "simulation handle used outside its simulation";
    assert!(panics_with(
        || {
            let _sim = crate::Root::sim(1);
            let _ = read.read(&mut [0; 8]);
        },
        refused
    ));
    assert!(panics_with(
        || {
            let sim = crate::Root::sim(1);
            sim.world.proc("v", M::new());
            let mut child = null(&mut sim.proc.command("v")).spawn().unwrap();
            child.wait().unwrap();
            child.stdout = Some(drained);
            let _ = child.wait_with_output();
        },
        refused
    ));
    let mut child = {
        let sim = crate::Root::sim(0);
        sim.world.proc("w", M::new().exit(3));
        let mut child = null(&mut sim.proc.command("w")).spawn().unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(3));
        child
    };
    let try_wait_refused = panics_with(
        || {
            let _sim = crate::Root::sim(1);
            let _ = child.try_wait();
        },
        refused
    );
    let kill_refused = panics_with(
        || {
            let _sim = crate::Root::sim(1);
            let _ = child.kill();
        },
        refused
    );
    assert_eq!((try_wait_refused, kill_refused), (true, true), "try_wait and kill must both refuse");
}

#[test]
fn mixed_process_backends_are_refused() {
    let kept = || {
        let sim = crate::Root::sim(0);
        sim.world.proc("w", M::new());
        let mut child = null(&mut sim.proc.command("w")).stdout(Stdio::piped()).spawn().unwrap();
        child.wait().unwrap();
        child.stdout.take().unwrap()
    };
    let (stdio, drained) = (kept(), kept());
    let refused = "simulation handle used outside its simulation";
    let stdio_refused = panics_with(
        || {
            let _ = Proc::new(None).command("true").stdin(Stdio::from(stdio)).spawn();
        },
        refused
    );
    let mut real = Proc::new(None).command("true").stdin(Stdio::null()).stdout(Stdio::piped()).spawn().unwrap();
    real.wait().unwrap();
    let real_pipe = real.stdout.take().unwrap();
    real.stdout = Some(drained);
    let real_drain_refused = panics_with(|| { let _ = real.wait_with_output(); }, refused);
    let sim_drain_refused = panics_with(
        || {
            let sim = crate::Root::sim(1);
            sim.world.proc("w", M::new());
            let mut child = null(&mut sim.proc.command("w")).spawn().unwrap();
            child.wait().unwrap();
            child.stdout = Some(real_pipe);
            let _ = child.wait_with_output();
        },
        refused
    );
    assert_eq!((stdio_refused, real_drain_refused, sim_drain_refused), (true, true, true));
}

// research/30 §§5-6: one terminal event per instance, matching the status a wait collects, even
// when a kill's termination runs while the model is handed off after its last step.
#[test]
fn a_killed_instance_has_one_terminal_event() {
    let (mut bad, mut killed, mut exited) = (Vec::new(), false, false);
    for seed in 0..64 {
        let sim = crate::Root::sim(seed);
        sim.world.proc("w", M::new().write("x"));
        let mut child = null(&mut sim.proc.command("w")).spawn().unwrap();
        child.kill().unwrap();
        let status = child.wait().unwrap();
        let trace = sim.trace();
        let terminal: Vec<&str> = trace.lines().filter(|l| l.contains("\"proc\":\"code ") || l.contains("\"proc\":\"signal ")).collect();
        let expected = if status.code().is_none() { "\"proc\":\"signal 9\"" } else { "\"proc\":\"code 0\"" };
        killed |= status.code().is_none();
        exited |= status.code() == Some(0);
        if terminal.len() != 1 || !terminal[0].contains(expected) {
            bad.push(seed);
        }
    }
    assert!(bad.is_empty(), "seeds without exactly one matching terminal event: {bad:?}");
    assert!(killed && exited, "both outcomes of the race");
}

// PROC-1: when the descriptor sweep fails, the spawn fails and the program never runs.
#[test]
fn real_a_failed_descriptor_sweep_fails_the_spawn() {
    if isolated("real_a_failed_descriptor_sweep_fails_the_spawn", &[]) {
        return;
    }
    deny_syscalls(&[436]); // close_range
    let marker = std::env::temp_dir().join(format!("tide-proc-sweep-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let err = Proc::new(None).command("touch").arg(&marker).stdin(Stdio::null()).status().unwrap_err();
    assert_eq!(err.kind(), K::PermissionDenied, "{err}");
    assert!(!marker.exists(), "the program ran after a failed sweep");
    println!("ISOLATED_RAN real_a_failed_descriptor_sweep_fails_the_spawn");
}

// PROC-1 on the real backend: the child sees exactly 0, 1 and 2 while the parent holds more.
#[test]
fn real_child_sees_exactly_three_descriptors() {
    let file = std::fs::File::open("/dev/null").unwrap();
    // A high descriptor without close-on-exec, as an inherited or ffi-opened one would be.
    let extra = rustix::io::fcntl_dupfd_cloexec(&file, 200).unwrap();
    rustix::io::fcntl_setfd(&extra, rustix::io::FdFlags::empty()).unwrap();
    let out = Proc::new(None).command("ls").arg("/proc/self/fd").output().unwrap();
    let mut fds: Vec<u32> = String::from_utf8(out.stdout).unwrap().split_whitespace().map(|s| s.parse().unwrap()).collect();
    fds.sort();
    // ls itself opens descriptor 3 to list the directory.
    assert_eq!(fds, [0, 1, 2, 3]);
    drop(extra);
}

// The real backend's lifecycle: status, captured output, piped I/O, kill and a missing program.
#[test]
fn real_lifecycle() {
    let proc = Proc::new(None);
    assert_eq!(proc.command("sh").args(["-c", "exit 3"]).stdin(Stdio::null()).status().unwrap().code(), Some(3));
    assert_eq!(proc.command("no-such-tide-program").output().unwrap_err().kind(), K::NotFound);
    let mut cat = proc.command("cat").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    cat.stdin.as_mut().unwrap().write_all(b"hello").unwrap();
    let out = cat.wait_with_output().unwrap();
    assert_eq!((out.stdout.as_slice(), out.status.success()), (&b"hello"[..], true));
    let mut sleeper = proc.command("sleep").arg("30").stdin(Stdio::null()).spawn().unwrap();
    assert!(sleeper.try_wait().unwrap().is_none());
    sleeper.kill().unwrap();
    let status = sleeper.wait().unwrap();
    assert_eq!((status.code(), status.to_string()), (None, "signal: 9 (SIGKILL)".to_owned()));
    sleeper.kill().unwrap();
    assert_eq!(sleeper.wait().unwrap(), status);
}

#[allow(dead_code)]
fn _sim_root_fields_move(sim: SimRoot) -> Proc {
    sim.proc
}
