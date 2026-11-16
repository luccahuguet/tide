Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 898c8ab

# fx-ip4x — Opus review round (GOVERNANCE.md Merge Rule, report only)

## Verdict

request_changes: 1 blocking finding, 9 non-blocking.

## Scope

- Diff `git diff 4cb53db...898c8ab`. I checked the SHA-256 myself: 5d79167c54e172e44a02680432b441d165e151878cca8f119fa7a496330b36cf. Head is 898c8ab.
- I read all of `crates/tide-admit/src/invocation/sandbox.rs`. I also read the changed `run_test_process`, `supervise` and `report_bounded` in `invocation/replay.rs`, the flag parsing and help in `invocation.rs`, the docs in `main.rs`, the four new tests in `tests/build.rs`, and the tide-std changes (`sim/runner.rs`, `sim/manifest.rs`, `replay.rs`).
- Context I read but the diff does not change: `destination()` (it canonicalizes OUT), the build-cache code around `.tide-cache`, `test/runner.rs` (the vacuous and should_panic verdicts and the fresh-rerun printing), the `tide-std` crate docs, and `conformance/README.md`.
- Spec: RUN-1 to RUN-6 in design/03, lines 730–935. I also read the README and design/03 diffs, GOVERNANCE.md's Merge Rule, and the bead's Scope, Acceptance and comments.
- I checked the Landlock constants and ABI versions against the host's `/usr/include/linux/landlock.h`. FS bits 0–16 match, `RESOLVE_UNIX` is ABI 9, `NET_*_UDP` is ABI 10, and the scopes are ABI 6. I checked every BPF jump offset by hand: the clone, prlimit64 and ioctl/fcntl/prctl `one_of` blocks, and the arch check.
- I ran no builds or suites, as the brief says. The coordinator's logs show `cargo test --workspace` exit 0 on c1–c4. I did not check whether conformance passed on this candidate. The bead comments say "conformance stopped as stale" on earlier candidates, and Acceptance requires the full suites to pass.

## Blocking findings

### B1. The sandboxed child outlives the driver: Ctrl-C, `timeout`, or killing the driver leaves it running, with no deadline

- Where: `crates/tide-admit/src/invocation/sandbox.rs:117`. `install` calls `setsid()` and sets no parent-death signal. Supervision is in `crates/tide-admit/src/invocation/replay.rs:320-392`, and only `supervise` ever kills the child (`:388`).
- Mechanism: RUN-3 correctly requires a new session. But a new session takes the child out of the caller's process group:
  - terminal SIGINT/SIGQUIT goes to the foreground process group, which the child is no longer in;
  - GNU `timeout`, without `--foreground`, signals its whole process group (`kill(0, sig)`), which no longer contains the child;
  - SIGKILL from a hang guard or `heavy.sh` reaches only the driver.

  Nothing ties the child's lifetime to the driver's. There is no `PR_SET_PDEATHSIG` and no signal forwarding. Before this change the child was a plain `Command` in the caller's process group, so all of these reached it.
- Failure scenario 1: `heavy.sh ... timeout 600 cargo test --workspace` hits its limit while a `#[sim_test]` that spins (`loop {}`) or a plain test with an infinite loop is running under `--tide-run-test`. `timeout` kills cargo's group, which includes the test process and the driver. The sandboxed child is in its own session, so it survives, reparented to init or a subreaper, and keeps spinning at 100% CPU. The 300 s per-evaluation backstop died with the driver.
- Failure scenario 2: the new tests' own hang guard (`bounded()` and the inline loop in `run3_stalled_stderr_does_not_hold_the_limit`) kills only the driver. If it ever fires, the `flood` child, which sleeps forever, stays alive.
- Failure scenario 3: an operator's Ctrl-C on a hung `tide-admit --tide-run-test` run returns the prompt and leaves the child running. A child that writes stdout eventually dies of EPIPE; a quiet spinner never does.
- Why it matters: RUN-4 hole 6 accepts that a candidate "can spin ... until a limit the operator sets stops it". After this change the operator's limits (Ctrl-C, `timeout`, killing the driver) no longer stop it. On this fleet, where load caps decide what may run, orphaned spinners that no one is accounting for are a real cost.
- Why it blocks under the tier rule: the fix changes v0 code the change edits (`sandbox.rs` `install`, possibly `supervise`). It is a regression the change introduced, not later-tier text.
- Suggested fix: in `install`, before the seccomp filter, call `prctl(PR_SET_PDEATHSIG, SIGKILL)`. It is async-signal-safe and survives exec of a non-setuid binary. The parent is alive during `pre_exec`, because `spawn` blocks on the exec pipe, but a `getppid()` check against the parent's pid, passed in, closes the race fully.

  `PR_SET_PDEATHSIG` fires when the *thread* that forked exits. `supervise` spawns and waits on the same thread, so this holds.

  Add a test: kill the driver while its child spins, then check that the child pid is gone. The child can print its pid on stdout, which the test reads.

## Non-blocking findings

### N1. The driver buffers child output without bound

- Where: `replay.rs:332` uses an unbounded `mpsc::channel`, and `:352` uses `read_until(b'\n')`. Nothing bounds the line length.
- Scenario: a test that prints progress without newlines (`print!(".")` in a loop) has its whole output held in the driver until a newline or EOF. Output no longer streams, and a long run grows the driver's heap without limit. When the consumer is slower than the child (a paused `| less`, or an agent harness that reads lazily), every forwarded chunk queues in the driver.
- This trades the old pipe backpressure for driver memory. RUN-4 hole 6 covers resource use, so I'm not blocking on it.
- Suggestion: forward partial lines once the buffer passes a small cap (seed records are short), and use a bounded queue that drops or coalesces only on the failure path.

### N2. `publish` deletes a user's `OUT/target` or `OUT/target/sim` when either is a link or a regular file

- Where: `sandbox.rs:373-389` (`subdirectory(..., replace = true)` calls `unlinkat`), called from `publish` (`:310`).
- At level `full`, the child cannot create links (seccomp denies `symlink*`/`link*`, and Landlock denies `MAKE_SYM`), and it cannot write under `OUT/target` at all. So replacing defends only against things the operator did. Examples:
  - a `target -> /bigdisk/...` symlink is silently replaced by an empty directory;
  - a `--crate-type=bin --crate-name=target` build in the same OUT writes the executable `OUT/target`, and the next `--tide-run-test` deletes it.
- RUN-2 says to replace "at each copy's path", not at the parent directories.
- Suggestion: fail with a clear error instead of unlinking, or follow an operator-owned `target`, since publishing renames only from a private directory the driver created.

### N3. Stale doc left over in tide-std

- Where: `crates/tide-std/src/lib.rs:248-250` still says only `#[sim_test]` entries isolate their run in a child process, "without RUN-3's OS sandbox". Under the driver's run modes, a `#[sim_test]` now runs in process, inside the RUN-3 child.
- `conformance/README.md:464` still calls RUN-3 "planned". That sentence is about the conformance harness, so it is still true, but it reads as stale.
- These are text-only fixes and need no re-check.

### N4. tide-admit no longer compiles outside x86_64 Linux

- Where: `invocation.rs:33` declares `mod sandbox;` with no `cfg`. Uses such as `libc::SYS_open`, `SYS_stat`, `prctl` and the seccomp constants do not exist on aarch64 Linux or macOS.
- Before this change, admission without linking needed no x86_64 host; only `--emit=link` was gated on one (`invocation.rs:238`). The README names x86_64-linux-gnu as the supported host, so this is not material today.
- Suggestion: `cfg(all(target_os = "linux", target_arch = "x86_64"))` on the module, with a `none` level elsewhere, which RUN-3 already specifies.

### N5. A rare hang path remains after a failed run

- After a timeout or signal, the stderr forwarder may still hold std's stderr lock while blocked on a stalled consumer. If `publish` then fails (`replay.rs:262`), or `supervise` returns a read error, the `Err` reaches `main`'s `eprintln!` (`main.rs:110`), which waits on that lock forever.
- Suggestion: send these errors through `report_bounded` too, or return the failure code first.

### N6. Error message wording

- `replay.rs:257` labels every `supervise` error "cannot start the test child in its RUN-3 sandbox", including a forwarding `EPIPE` after a child that ran normally. That error also discards the child's exit status.
- Suggestion: keep the "cannot start" wording for spawn errors only.

### N7. Gaps in test strength (Acceptance is met)

The denial test does prove the kernel denies. The program never passed admission, a direct control run reaches every target, and under the driver it gets EACCES, EPERM and EINVAL. It is hang-guarded. Without Landlock, `read=Ok`; without seccomp, the socket gets EACCES instead of EPERM; without pipes, `kept` is lost. So it fails without each fix.

What nothing proves:

- Process creation is denied. README and RUN-3 claim it, and the hand-written `clone` BPF is the subtlest part of the filter. I checked it by hand and it is correct. A `Command::new("/bin/true").status()` expecting EPERM in the unadmitted program would pin it.
- Signals to the driver are scoped. `kill(getppid(), 0)` is enough.
- An inherited non-CLOEXEC descriptor is closed. This was round-1 finding 3; the fix is in place but has no regression test.
- The run directory under `.tide-cache` is removed afterwards.

Smaller points:

- The `claimed` run's exit status is never checked. Its assertion fails only if both the token filter and `RENAME_NOREPLACE` are removed.
- The denial test fails, rather than skips, on hosts below `full`. That is acceptable, since it says so, but CI hosts without Landlock will fail it.

### N8. Small output and behaviour changes

- `start()` prints `"\n{START}{seed}"`. The driver drops the record but forwards the blank line before it, so a default 100-seed `#[sim_test]` under the driver writes about 100 blank lines to stdout. That contradicts README's "seed-start records stay internal" in spirit.
- The printed `fresh rerun` and `replay` commands leave out `--tide-eval-timeout`.
- A child can leave an empty directory with mode 000 in `sim/` (`mkdir(path, 0)` is allowed). `RunDir::remove` then fails silently, leaving `.tide-cache/run-*` behind; or the directory is published and blocks `rm -rf target`. This is minor, and hole 6 covers it.

### N9. Spec and documentation wording

- Wording: RUN-2 says publishing replaces whatever is at each copy's path, while the code never replaces (`RENAME_NOREPLACE`) and README documents it that way. The code is the stricter choice and fits the design. Align RUN-2's sentence when design/03 is next edited (v0 text, text-only).
- Missing feature, not a regression: under the driver, an early `exit(0)` in a `#[sim_test]` now passes. Before, the old parent's "missing summary" rule failed it. That rule now exists only outside the driver. Only code that reaches a RUN-4 hole can do this, and RUN-6, which README lists as missing, is the real fix.

## Reasoning

**What the child can reach is sound for a `full` host.** I checked:

- Landlock handles every FS right up to ABI 9, and TCP/UDP up to ABI 10, with scopes from ABI 6. The grants are:
  - the binary copy: EXECUTE and READ, by file;
  - the loader and the libraries resolved from the binary's ELF: EXECUTE and READ;
  - `context.json`: READ;
  - `sim/`: read, write, make-reg, make-dir, remove-file, truncate. It gets no REFER, no MAKE_SYM, no REMOVE_DIR and no EXECUTE.
- The seccomp allowlist has no `socket*`, `fork`/`vfork`/`execveat`, `kill`/`tkill`/`rt_sigqueueinfo`, `ptrace`/`process_vm_*`, `io_uring_*`, `bpf`/`perf_event_open`, `keyctl`, SysV/POSIX IPC, `*xattr`, `readlink*`, `chmod`/`chown`/`utimensat`, `link`/`symlink`/`mknod`, `memfd_create`, `unshare`/`setns`/`mount`, `setrlimit`, or `pidfd_*`.
- `clone` is admitted only with `CLONE_THREAD` and no namespace flag, and `clone3` gets ENOSYS. `prlimit64` passes only with a NULL new limit. `ioctl` is limited to `TCGETS`/`FIONBIO`/`FIOCLEX`/`FIONCLEX`, so `TIOCSTI` is excluded. `fcntl` excludes `F_SETOWN`, so no SIGIO can be aimed elsewhere. `prctl` is limited to setting and getting the name.
- x32 calls fall to DENY, and a foreign arch is killed.
- `/proc/self/*` is denied by Landlock. Reopening a pipe through `/proc/self/fd` gives only the child's own pipes.
- Descriptors: `close_range(3, ~0, CLOEXEC)` is required, stdin is `/dev/null` opened read-only, and stdout and stderr are pipes.
- Core dumps: the 1-byte core limit cannot be raised, because `setrlimit` is denied and `prlimit64` only reads.
- Publishing reads only token-prefixed directories from a private 0700 run directory after the child is reaped. With links, FIFOs and sockets uncreatable, those trees hold only regular files and directories. They are renamed with `RENAME_NOREPLACE` through descriptors opened with `O_NOFOLLOW`.
- `OUT` is canonicalized by `destination()`, so an operator's symlinked out-dir is not hit by the `O_NOFOLLOW` opens. I withdrew that concern.

**The code between fork and exec is async-signal-safe.** The `pre_exec` closure owns a ruleset descriptor and a filter `Vec`, both allocated before fork. `install` makes only raw syscalls and builds `io::Error::Os` from errno, which does not allocate. The order is right: `setsid`, the core limit, `close_range`, `NO_NEW_PRIVS`, `landlock_restrict_self`, then seccomp. After that, Rust std's `execvp` runs on an absolute (canonicalized) path, and the exec-error pipe write and `_exit` both appear on the allowlist.

**The reported level is honest.**

- `full` needs Landlock ABI 6 or later plus seccomp's `RET_ERRNO`. Missing Landlock UDP or `RESOLVE_UNIX` below ABI 9 or 10 opens no gap, because seccomp already refuses `socket`.
- `partial` names what is missing and runs under what remains, which RUN-3 permits.
- If Landlock is present but the ruleset or `restrict_self` fails, or if loading the filter fails, the run fails. That matches "cannot install the level it reports".
- The run also fails before starting on a hard core limit of 0 or a kernel without `close_range`, as RUN-3 requires.

**Supervision meets the round-1 and round-2 findings.**

- Readers only drain the pipes and observe seed starts; separate forwarders do the writing. The deadline loop never blocks on a sink.
- After a timeout or signal, the driver waits at most `FORWARD_GRACE` for forwarding, and `report_bounded` writes to fd 2 without std's lock.
- At exit, std flushes stdout with `try_lock`, so a stuck forwarder cannot hang it.
- The three supervision tests are hang-guarded, and each would hang or fail without its fix.

The remaining gap is lifetime in the other direction: nothing ends the child when the driver dies. That is B1, and it is the only finding I block on.

**The tide-std change is right.** The sandbox forbids processes, so `#[sim_test]` under the driver's context must run in process. `run_config` keeps libtest's should_panic, vacuous and self-check verdicts. The driver takes over the backstop and the last-seed report, and the README states that.

**Leftovers.** The README, design/03 §9.2, the `main.rs` docs and help, and the help line for `--tide-eval-timeout` are aligned. N3 lists the two remaining stale sentences.

DONE
