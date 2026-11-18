Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 6308c95 (code)

## Verdict

**Changes required.** Round 1's authority refusal is fixed, and its new regressions pass in the candidate workspace log. Three lifecycle defects remain: incomplete cleanup after a pipe-setup error, simulated draining of the wrong child's pipe, and a duplicate simulated exit event after termination. Confidence is high in the source findings; I did not execute these new scenarios because this pass permits inspection and non-heavy checks only.

## Scope

Full fresh review of the implementation diff, including process API/configuration, both backends, admission and public exports, handle classification, trust diff, scheduler/task changes, replay recording and validation, conformance fixtures, tests, and changed documentation. The owning contracts are stage 1's research/40, design/03 §1.1 and §9.9, companion 20, and PROC-1–3, with research/30 for model mechanics. Net/fx-ff13 is excluded.

Observed HEAD is `6308c95e623dd25ba5de820390891194356179e9`; the tracked working tree was clean. Main was `6a656ed08e83f832188a91efe6437e3921d166dd`. SHA-256 of the literal current `git diff main..6308c95` is `d0f8d16518bfa1686ffd05b8424efefe7ce0e7c88755383c23e6112fa37eeb07`. Excluding `.beads/issues.jsonl` gives exactly the brief's `e6cbfcb862cfbb39be599673a5ff62a570c3f326f4eabbf0d667b99471efb7c7`: the identity discrepancy is ledger-only, not implementation drift. This report is the only review write; no source or ledger was edited, no commit was made, and no heavy build or compiled Tide probe was started.

## Blocking findings

### B1 — Pipe-setup failure still retains unprocessed pipe ends during cleanup

**Location:** `crates/tide-std/src/process/real.rs:123–136`.

This is an incomplete part of round 1 B2. The fallible closure takes and configures the endpoints one at a time. On an error, already-taken endpoints close, but endpoints not yet taken remain in the underlying Rust `Child`. The error arm immediately calls `kill` and `wait` without closing them. Rust's `Child::wait` closes retained stdin, but retains stdout and stderr.

Concrete failure: `output()` has piped stdout and stderr. An inherited syscall policy denies FIONBIO, so configuring stdout fails; stderr is still in `child.stderr`. If the policy also denies SIGKILL, a child writing enough stderr fills that retained pipe and blocks forever while Tide waits shielded. Closing the parent read end would instead let its write fail/terminate. Failed termination is a supported cleanup case, explicitly discussed in 03 §1.1 and companion 20. The comment claiming all pipe ends are already closed is false on this path.

**Suggested fix:** in the error arm, explicitly take/drop every remaining Rust child pipe before the shielded wait, then attempt termination and reap while preserving the original setup error. Extend `real_failures_after_spawn_reap_the_child` with a bounded isolated case where setup fails, kill is denied, and the child writes to the still-unprocessed stderr pipe. Its current injected setup-error case passes because kill succeeds, so it cannot detect this defect.

### B2 — Simulated `wait_with_output` reads the child's original buffers instead of its public pipe fields

**Location:** `crates/tide-std/src/process/sim.rs:488–513`; public fields and drain forwarding at `crates/tide-std/src/process.rs:327–330,387–401`.

`drain` retains the supplied endpoint objects only to track whether each field is present. Readiness, bytes and buffer mutation always use `self.inst.pipes[i + 1]`, ignoring the instance and slot actually owned by each supplied pipe. Public `Child` fields can legally be moved between children; the owning docs explicitly preserve Rust's public-field API and private handle ownership.

Concrete deterministic example: spawn A with piped stdout containing `a`, and B with piped stdout containing `b`; wait for both to exit while retaining their stdout fields. Then assign `a.stdout = b.stdout.take()` and call `a.wait_with_output()`. The real backend reads the moved B pipe and returns `b` with A's status. Simulation drains A's old buffer and returns `a`, discarding B's unread output when the supplied endpoint drops. Both children can be ordinary supported models, and no pipe-to-`Stdio` conversion or unsupported mode is involved.

**Suggested fix:** drain the actual supplied pipes, using each endpoint's instance, slot and simulation identity for readiness, consumption and wake registration. Preserve concurrent draining so large stdout/stderr streams cannot deadlock. Add this small field-transfer scenario to the admitted dual-backend contract test; returning another child's bytes is an observable PROC-1/ownership failure.

### B3 — A killed model can subsequently emit a second, normal exit event

**Location:** `crates/tide-std/src/process/sim.rs:308–309,648–655,837–840`.

After its final non-exit step, `run` hands off at `mc.point` and then returns `Flow::Continue` without checking whether termination ran during that handoff. The caller unconditionally invokes the implicit `exit(0)`. `Instance::terminate` correctly preserves an existing status, but `exit` still records `proc.exit: code 0` even when no normal exit occurred.

Concrete schedule: register `ProcessModel::new().write("x")`, spawn it with null stdin and inherited stdout, then `kill` and `wait`. The model writes and yields after its final step; the caller submits kill; the termination task stores SIGKILL and records `proc.terminate: signal 9`; the original model resumes and records `proc.exit: code 0` for that same instance. Wait returns SIGKILL, so the trace contradicts the collected status and reports two terminal events. Research/30 §§5–6 excludes duplicated exit and requires exit to discard unperformed actions. Replay being byte-identical does not make that event sequence valid.

A plain Python calculation using the candidate scheduler's exact RNG/index function gives seed **17** for the required picks `[1, 0, 2, 1]` at bounds `[2, 2, 3, 2]` in this minimal scenario. This is source-derived reachability evidence, not a claimed executed Tide seed.

**Suggested fix:** under the instance lock, emit a normal-exit event only when that call actually performs the running-to-exited transition. Add a kill/default-exit race witness asserting one terminal event per instance, consistent with the returned status. Existing kill-race tests check status outcomes and miss the extra event.

## Non-blocking findings

None. No API expansion, dependency, signal/PTY implementation, or Net change is needed.

## Reasoning

### Round 1 findings and their regressions

- **B1: fixed.** The shared gate refuses equals-bearing environment keys under `only` before NUL validation or either backend lookup. `simulated_refusals_precede_model_lookup` uses an allowed unregistered spelling plus a NUL argument, so its equals-key assertions distinguish both ordering mistakes. `real_lookup_and_canonical_exec` covers real refusal. Both pass in the candidate log.
- **B2: partially fixed; round 2 B1 above remains.** Convenience-call cleanup now handles every returned error, closes its public endpoints and performs a shielded wait. Spawn's post-OS-spawn setup failure now attempts kill/reap, and `real_failures_after_spawn_reap_the_child` checks ECHILD after injected setup/drain failures. That test passes, but does not cover failed kill with an unprocessed endpoint still retained.
- **B3: fixed.** Real cancellable wait and drain check cancellation at entry, including cached-status and empty-pipe cases. `a_wait_in_a_cancelled_scope_is_cancelled_even_when_cached` checks both backends and preservation of cached status; it passes.
- **B4: fixed.** The independent closed-end flag is enforced before inherited/null/piped routing, and by stdin read/copy paths. `a_model_cannot_use_a_descriptor_it_closed` exercises the relevant routes and passes.
- **B5: the silent-seed-success defect is fixed.** Model execution catches a panic into the stored, source-located coverage failure and wakes waiters; delay overflow uses checked arithmetic. `a_model_panic_fails_the_seed` checks injected panic and accepted-model overflow and passes. The builder's additional claim that `task/proc.rs::spawn_model` now finishes panicked tasks with `Outcome::Panic` is not present in this candidate: that adapter remains unchanged and finishes `Outcome::Ok`. The observed seed-failure fix is the new process callback/failure state, not that claimed adapter change.
- **B6: the four requested proof gaps are addressed.** The driver records and replays a Proc manifest, compares traces and roots, then re-seals altered model data and checks refusal. The tide-std manifest witness has an omission control. The seccomp sweep test denies close_range and checks that the target never ran. The canonical-exec test actually spawns the bare name through the symlinked start-time PATH with alternate child cwd. The cancellation control omits only cleanup wait, checks termination before return, and cancelled `wait_with_output` retains a usable child. These tests pass in the candidate workspace log. The `replay.rs` addition of `proc` to accepted root inputs is exercised by the driver replay test.
- **B7: fixed.** The companion records 24,979 tokens dated 2026-11-18. My `scripts/count-tokens.sh design/reference.md` invocation independently returned **24979**, within D14's 25,000 limit.

### Full fresh review observations

The shared refusal gate implements exact OS-string matching, invalid-entry filtering and intersection. It checks program, explicitly set refused variables and NUL before lookup, model lookup or spawn. Removed/cleared variables are not treated as final explicit sets. `output`/`status` observe entry cancellation; `spawn` does not.

Real lookup ignores command-local PATH, skips inherited empty/relative PATH elements, has no default search path, and canonicalizes bare-name results and absolute paths in the parent before applying child cwd. Relative program paths remain limited to unrestricted Proc. `arg0` preserves the requested spelling. Admitted safe code has no parent-environment setter; the backend's ambient PATH read follows the same parent environment that real `Env::var` reads. Writable executables/directories, program-specific configuration, scripts and interpreters retain the explicitly documented authority limits.

I found no new public backend constructor or ambient spawn path through process re-exports or `__rt`. PROC-3's guard includes public-use lines and separately pins the four Rust re-export names. Proc, Command, Child, all three pipe ends and Stdio are in NOT_PURE. The ambient Command::new case still expects TD0501, and the capability trust-diff test passes with `proc` and the child-as-ambient note. The sort change preserves item order within its priority groups.

For the unsafe spawn path, I inspected the local source of the pinned nightly: a pre-exec hook prevents the posix_spawn fast path; Rust sets up stdio before the hook and reports/reaps pre-exec failure through its CLOEXEC error channel. Tide's hook performs the raw close_range syscall and captures errno, with no allocation or fcntl fallback. The Linux 5.11 floor is documented. Owned descriptors close on ordinary error unwinding; real drain services both output pipes, pipe I/O retries EINTR, and shielded Rust wait retries interrupted waitpid. Successful waits cache status and protect kill after reap. Round 2 B1 is the remaining cleanup exception identified here.

Simulation reuses the existing scheduler, operation timers and cancellation machinery, with a separate `tideproc` sub-stream. Registration records exact program spelling, `proc-lifecycle-v0 schema=1`, capacity 65,536 and model data inline; root recording retains the seed. Unregistered programs and unsupported modeled modes fail coverage, and ordinary no-process runs introduce no process draws. The research/40 §12 witness/control pairs are present, including the new manifest omission control. The two new simulator findings concern endpoint ownership and truthful terminal events, beyond the earlier witnesses.

Verification observed on candidate 6308c95, without starting heavy work:

- `git diff --check main..6308c95`: pass.
- Reference token count: 24,979 / 25,000; companion matches.
- `target/work/suites/candidate.txt`: exact candidate HEAD. The coordinator's workspace log completed from 2026-11-18 17:33:25Z to 17:37:23Z, with exit **0**, **424 passed / 0 failed** across 16 result groups. The new replay, refusal, cleanup, closure, panic and descriptor-sweep regressions are explicitly present as passing results.
- Conformance had only an empty `conformance.log` and no completion/exit file at the last inspection. I do not claim a completed conformance run on this revision. The prior 412fc87 results are prior-revision evidence only.

The green workspace suite does not exercise the three concrete failures above. No new executable reproductions, heavy runs, implementation changes or commits were made.

DONE
