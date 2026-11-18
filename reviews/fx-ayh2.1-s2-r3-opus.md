Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 4c88de9 (code)

# fx-ayh2.1 stage 2, Opus converge round s2-r3 (reports, does not fix)

## Verdict

**Changes required.** There are two blocking findings, and both have one-line fixes. One is a
stale "(planned)" label in the companion. The other is a simulation-identity check that two
simulated `Child` operations skip, the same class of defect as the s2-r2 re-check's B2. I found no
authority hole, no unsafe-code defect and no lifecycle-model defect. The earlier rounds' fixes hold.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-ayh2.1`, branch `fx-ayh2.1`, HEAD
  `4c88de909aa2bff103160d75e3a2f3b99194e5f4`. Tree clean.
- Base: `git merge-base HEAD main` = `a4696340e8135938444f381a94dec8bf0e4f6447`.
- SHA-256 of `git diff <base>..HEAD`: `e6869a434f09527ae6a4607812077b03ba56d0239fcd9c5a8283fb93c58d774d`.
  The diff has 27 files, +3247/−60.
- Read in full: `process.rs`, `process/real.rs`, `process/sim.rs`, `sim/process.rs`,
  `task/proc.rs` and `process/tests.rs`; the diffs to `sim.rs`, `task.rs`, `scheduler.rs`,
  `manifest.rs`, `replay.rs`, `root.rs`, `fs.rs`, `lib.rs`, the stub std, `diff.rs`, `policy.rs`
  and `build.rs`; `conformance/cases/authority/proc.rs` and the two edited reject cases; and the
  diffs to rules.md, 03, the companion, reference.md, README and research/40. I also checked
  research/40 §§2, 5–8 and 12, research/30 §6, 03 §4.3, `walk.rs` `admit_def`, `pure.rs`
  `NOT_PURE` and `NetContext::socket`.
- Read but not repeated: s2-r1, s2-r2 and both re-check reports, report.md, and the bead with its
  comments.
- Proof I observed (tide-lead's runs on this HEAD, `target/work/suites/`): `candidate.txt` names
  4c88de9 and `head-check` says HEAD unchanged. `cargo test --workspace` exit 0: all 16
  `test result` lines report 0 failed. `tide-conformance` exit 0: 390 cases, 387 PASS, 3 AWAITS
  CHECKER, 0 FAIL.
- `scripts/count-tokens.sh design/reference.md` gives **24,979**, which is ≤ 25,000 and matches the
  companion's line.
- I started no heavy run and need none for this verdict. Each fix below has a cheap regression
  test, and the next candidate's suites cover it.

## Blocking findings

### B1. The companion still labels `proc.command` as planned

`design/reference-companion.md:86` (§14.2, ambient std calls), a v0 file this change edits:

    | `Command::new("git")` | `proc.command("git")` (planned) |

- **Failure:** an agent porting `Command::new` reads that `proc.command` is not built, while
  reference §6.1, companion §20 and the code say it is. This breaks AP-CONTRACT-001's
  current-state alignment, and it is exactly what review point 4 asks about. research/40 §12's
  flip list missed this line.
- **Minimum fix:** drop " (planned)" from that cell. Keep the `proc.exit(1) (planned)` on line
  88, which is correct. This is text only and needs no re-check.

### B2. Simulated `Child::try_wait` and `Child::kill` skip the simulation-identity check

`crates/tide-std/src/process/sim.rs:397–410` (`try_wait`) and `:449–463` (`kill`).

- **What happens:**
  - `try_wait` locks the instance and returns `self.reap(..)`'s result before it ever calls
    `self.ctx()`. `reap` returns the `Child`'s cached `status` first.
  - `kill` returns `Ok` when a status exists. On a running child it sets `st.kill = true`
    before `self.ctx()` (inside `record`) can panic.
  - `wait` and both pipe operations do check (`self.ctx().begin`, or `ctx()` before
    `wait_on`), and since 4c88de9 so does `drain`.
- **Failure:** in simulation S1, register `M::new().exit(3)`, spawn it with null stdin, call
  `child.wait()`, and keep the `Child` after S1 ends.
  - In S2, or outside any simulation, `child.try_wait()` returns `Ok(Some(code 3))`, and
    `child.kill()` returns `Ok(())`.
  - `child.wait()` on the same handle panics with "simulation handle used outside its
    simulation".
  - So one handle is refused by one operation and accepted by another. A result in S2 then
    depends on S1's history, which replaying S2's seed alone does not reproduce. That breaks
    the runtime invariant that `simulated_handles_require_their_own_live_simulation` records,
    and it is the defect the Codex finder rated blocking for `drain` in the s2-r2 re-check.
  - The `kill` variant is worse: on a child that is still running, it sets `kill` before
    panicking. Any later `kill()` in the owning simulation then returns `Ok` without submitting
    termination (`if st.status.is_some() || st.kill`). If that later kill is `finish`'s cleanup,
    the shielded wait waits for the model's own end.
- **Minimum fix:** call `self.ctx()` (that is, `Tasks::proc_context`, the check `Pipe::read`
  uses) first in sim `try_wait` and `kill`, before the instance is locked, read or changed.
  - `id()` can stay unchecked, since it is plain data.
  - Extend `a_pipe_from_another_simulation_is_refused` (`process/tests.rs:715`) with the kept
    `Child`: in S2, `try_wait` and `kill` must both panic with that message.
  - The fix crosses no trust boundary, so this finder re-checks only it.

## Non-blocking findings

1. **Mixing backends hits `unreachable!` with a misleading message.**
   - `real.rs:80` (`Config::Unsupported(what) => unreachable!("{what} is a real handle …")`) is
     reachable. Take a simulated `File` or `ChildStdout`, keep it after its simulation ends, then
     pass it as `Stdio` to a real `Command`. `Root::sim` is not test-only in `policy.rs`, so a
     `main` can do this.
   - `real.rs:231` and `sim.rs:497` behave the same for a pipe end from the other backend
     assigned to a child's public field.
   - Each is a panic, not an authority or memory issue, but the message says the opposite of
     what happened. Prefer `InvalidInput`, or the existing "simulation handle used outside its
     simulation" panic.
2. **A dead parameter.** The `cancellable` parameter of `process.rs:393` `Child::drain` and
   `sim.rs:482` `sim::Child::drain` is `true` at both call sites, so it can be dropped
   (AP-MINIMAL-001, simplification).
3. **Text nits on syscalls.**
   - research/40 §5 says "if it fails (as on a kernel before 5.11, `EINVAL`)". Before 5.9 there
     is no `close_range` and the error is `ENOSYS`; 5.9 and 5.10 give `EINVAL` for the flag.
   - `real.rs:18`'s comment "Linux generic syscall table, all architectures" overclaims, since
     alpha numbers these differently. Tide targets only x86_64 and aarch64, so nothing breaks.
4. **Acceptance wording on the symlink case.** The bead's Acceptance wants the one admitted
   dual-backend suite to include `only`'s PATH and symlink cases.
   - `authority/proc.rs` has the PATH case (`only(["true", "sh"])` refusing `/usr/bin/true`, and
     a command's own `PATH`).
   - The symlink and `/proc/self/cwd` cases are only in the real-only tide-std test
     `real_lookup_and_canonical_exec`, which rules.md PROC-2 names. The simulator has no
     symlinks, so the split is defensible.
   - tide-lead should confirm that it meets Acceptance, or add one line to the case, for example
     `proc.only(["/bin/true"]).command("/usr/bin/true")` returning `PermissionDenied` on both
     backends.
5. **A stale `NOT_PURE` entry.** `pure.rs:98` still lists `tide_std::process::Signals`, a type
   that research/40 §2 moves to Later and that does not exist. The entry predates this change
   and is harmless; it could be removed now or when signals land.

## Reasoning

### Authority (review point 1)

`Command::start` runs `admit()` before either backend, in PROC-ONLY's order:

1. the program, matched byte for byte against entries that `valid_entry` filters and that
   `only` intersects;
2. refused variables, checked on explicit sets only, including any key holding `=`
   (`process.rs:88–94`);
3. NUL bytes.

Then:

- **Real backend.** `real::resolve` looks a bare name up only on the parent's `PATH`, keeping
  absolute elements and skipping empty and relative ones. It canonicalizes in the parent, and
  `CommandExt::arg0` keeps argv[0] as spelled, so `current_dir` and a command's own `PATH` cannot
  change which program starts.
- **Simulator.** It matches exact spellings after the same `admit()`. A refusal reaches no model.
- **`Proc` itself.** It has no public constructor (`Proc::new` is `pub(crate)`), and
  `tide_std::__rt::*` is denied (`policy.rs:94`).
- **Admission.** `std_process_public_items_are_fixed` pins the public items of `process.rs`,
  `pub use` included, and the backends expose none. The re-export carries no process-starting
  item. `ExitStatus` adds only `code` and `success` to the allowlist.
- **`NOT_PURE`.** It already lists `Proc`, `Command`, `Child`, the three pipe ends and `Stdio`.
- **Ambient calls.** `std::process::Command::new` still yields `TD0501` (now with `E0599`).

I found no route to start a program outside the grant, to pass a refused variable, or to reach
a process without a capability.

### Real backend (review point 2)

- **Unsafe code.** The `pre_exec` closure makes one async-signal-safe raw syscall and reads
  errno. Rust's fork path runs it after the stdio `dup2`s and `chdir`. On failure Rust reports
  through its CLOEXEC error pipe and reaps the child itself, so the sweep fails closed with no
  zombie. The kernel sets `O_CLOEXEC` on the pidfd itself.
- **Failure after spawn.** Every remaining Rust pipe end is dropped before kill and the blocking
  wait (s2-r2 B1 holds). `finish` kills, drops the ends and waits shielded on every error.
- **Reaping once.** Std's internal status cache and `real::Child.status` cover it. Kill after
  exit is `Ok`, and a zombie that has not been reaped pins the PID, so kill cannot hit a reused
  PID.
- **Cancellable waits.** These go through `NetContext::socket` on the pipe, the epoll set or
  the pidfd. A cancelled wait leaves the child with the caller.
- **The 5.11 floor.** It is stated in 03, the companion and research/40 §9, and the code has no
  fallback, which is consistent.

### Simulator and replay (review point 3)

- **Lifecycle.** It is one immutable status per instance. `terminate` sets it once and closes the
  child's endpoints together. `exit()` returns early when a kill already won (s2-r2 B3 holds), so
  each instance has exactly one terminal event: `code N`, `signal 13` or `signal 9`. Kill is a
  separate ready action.
- **Pipes.** Buffers are bounded at 65,536 and apply backpressure in both directions. Transfers
  are short, drawn from `proc_rng`, a separate sub-stream, so a run that spawns nothing makes
  today's draws. Closing writes and reads is enforced on every route (s2-r1 B4 holds). A model
  panic becomes a coverage failure (s2-r1 B5 holds). Model timers are operation timers.
- **Seed witnesses.** Every row of research/40 §12's table has a witness paired with a `Broken`
  control, plus `ModelPanics`/`OmitManifest` fault checks and the one-terminal-event test.
- **Replay.** Model data is recorded inline, as profile, schema, capacity and the model's
  `Debug`, which is stable under the pinned toolchain. `a_proc_seed_replays_and_refuses_changed_model_data`
  shows a byte-identical replay and the refusal of changed model data through the driver.

### Contract and docs (review point 4)

- The code, rules.md PROC-1 to 3, 03 §1.1 and §9.9, companion §20, reference §6.1 and 6.8, the
  README and the research/40 status agree, apart from B1.
- `Proc::exit` stays labeled planned everywhere, correctly.
- The trust-diff note matches 03 §4.3's example, and new `proc` paths sort first. The test in
  `diff.rs` and the build test cover both.

### Leftovers and tests (review point 5)

- The tests target observable outcomes, each with a failing control. The real-backend tests that
  change process-wide state run isolated in a child test process.
- Only non-blocking items 2 and 5 remain as leftovers.

DONE
