Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-nkv.3 round 2: Opus re-check of my round-1 findings

## Verdict

**approve.** All three blockers (B1, B2, B3) and both non-blocking notes (N1, N2) are resolved. The changed lines contain no new defect.

## Scope

- Read `git diff 102a3e2 fx-nkv.3`, which is commit `a29875a`: 8 files, +34/−38.
- Read `fix1-brief.md` and `fix1-report.md`.
- Read `Runtime::observe`, `cancel` and `finish` in the fixed `task.rs`, and the nested test.
- Grepped `crates/`, `conformance/` and `README.md` for any `From<Cancelled>` or suppressed-summary code left behind. Only the intended `From<Cancelled> for io::Error` remains, at `task.rs:393`.

I reran both round-1 probes on a private copy of `task.rs` taken with `git archive fx-nkv.3`. The probes use the worktree's pinned rustc (`1.99.0-nightly 771916f90`), edition 2024, `nice -n 19`, and the round-1 shims. They ran one at a time, after the quiet-window, load (<12) and memory (≥6 GB) checks passed. Probe file: `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-nkv-3/dcd4b6b2-5978-4a56-928c-3a472c53b177/scratchpad/probe/probe.rs`. I ran no cargo builds. For the workspace suite and conformance results I rely on the author's report: 83 passed, 0 failed; and 198 PASS, 25 AWAITS CHECKER, 0 FAIL.

## Per-finding status

**B1: resolved.**
- The `From<Cancelled> for TaskError` impl is gone (`crates/tide-std/src/task.rs:392`). `From<Cancelled> for io::Error` is kept (`:393`).
- `conformance/cases/memory/scope_borrow_accept.rs:9` and `conformance/cases/tasks/task_lifecycle.rs:26` are back to the unannotated `tasks.scope(|s| {` form. The author's report shows both files match `07e104b` exactly.
- Code that needs a `TaskError` now uses `.map_err(|_| TaskError::Cancelled)?`: `conformance/cases/tasks/task_cancellation.rs:7,8,30,38,46` and the unit test at `task.rs:646`.
- **Probe P1** compiles cleanly (`rustc --cfg b1`, exit 0, no E0283). P1 is 02-memory §10.12 `parallel_sum`, copied verbatim, with `scope(...)?` in a function returning `Result<u64, TaskError>`.

**B2: resolved.**
- `Runtime::observe` (`task.rs:94-116`) still walks up only while the scope's cause equals the observed cause. It now sets `children[child].echo` only when `at.scope == origin`.
- An inherited `Failure(O)` cause therefore marks only the bit of O's child, and leaves the nested scope and any intermediate scope unmarked. That matches TASK-2's per-scope cause rule.
- Walking a three-level case (O → J → I) by hand gives the right results: J and I each report their child's error as independent, and O treats B's error as an echo of its first cause.
- The nested test now asserts `Err(Failure::Child)` from the inner scope in all three cases (`task.rs:816`). It then returns that result, so the outer ancestry assertion is still exercised. The author reports this assertion failed before the fix (`left: Ok(())`) and passed after.
- **Probe P2** now prints `inner scope returned Err(Cancelled)` and `outer returned Err(DeadlineExceeded)`. In round 1 it printed `Ok(5)`. The probe's child now maps its checkpoint error explicitly, because the `TaskError` conversion is gone.

**B3: resolved.**
- The `err`-sink emission is removed. `task.rs:225-226` keeps the outcome count as `_suppressed`, with a `shortcut:` comment that defers it to one `debug!` record once the log sink exists.
- `crates/tide-admit/tests/build.rs:110` asserts empty stderr in both profiles and prints the stderr on failure.
- `README.md:170` and `crates/tide-std/src/lib.rs:22` describe the silent default.
- P2 prints no `debug:` line even though the probe still links an `err` shim. Nothing in `crates/`, `conformance/` or `README.md` refers to the old summary any more.

**N1: resolved.** `README.md:173-175` and `crates/tide-std/src/lib.rs:25-26` each add the sentence about suppressed-payload drops: on the panic path they follow PANIC-2 and can abort; otherwise the panic surfaces in the scope's caller. This is accurate for the drop paths at `task.rs:215-235`. `lib.rs` also mirrors the README's existing note on discarded-value attribution. That is consistent with the README and within scope.

**N2: resolved.** `design/03-authority-concurrency.md:1208` adds `pub fn yield_now(&self) -> Result<(), Cancelled>;` to the §6.1 API block, and it matches `Tasks::yield_now`. `design/reference.md` is unchanged, as decided; recording it in the reference is the coordinator's job on fx-djp.

## New findings

None that block.

- Nit: `_suppressed` at `task.rs:226` is computed and never used. The brief asked to keep the accounting and the shortcut marker, so this is intended. The compiler optimizes it away.

DONE
