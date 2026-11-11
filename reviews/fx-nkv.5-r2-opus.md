Model: claude-opus-5-5 · Effort: low · Date: 2026-11-11

# fx-nkv.5 round-2 Opus re-check

Candidate: `c1c1061` on `d98ae1a`. The full diff's SHA-256 is `b89a7f4dac00e57cf9e4ae9a504594eb31ccd7daa2196c6d9b77f56215480f2e` (verified). I re-checked only my own findings 1–5 against `git diff e436752 c1c1061`. Finding 6 is docs-only, so I did not re-check it. I ran no build or probe: the 1-minute load was 22.3, above the gate of 12. This re-check comes from reading the code. It relies on the author's proof on `c1c1061` (workspace 114 passed; conformance 215 PASS / 24 AWAITS / 0 FAIL).

## Verdict

approve

## Findings

1. **Resolved.**
   - `Context::outside_simulation` (task.rs:263) is now the single owner of the "real task authority used inside a simulation" assert. It reads the thread-local without cloning it, and `task::outside_simulation` (task.rs:378) exposes it to sync.rs.
   - Every path my finding named calls it:
     - `Tasks::context`'s real branch
     - `checkpoint`'s `None` arm
     - `yield_now` when unsimulated, before `check_cancellation`
     - `sim_cell`'s `None` branch, which covers `Shared::new`
     - `Shared::with`'s real-ticket branch
   - The old inline copy in `Tasks::context` is deleted.
   - Real code inside a real scope is still allowed. The real scope context has `task: None` (task.rs:454), and the predicate `is_none_or(task.is_none())` accepts it.
   - `simulated_handles_require_their_own_live_simulation` now checks all five misuses for the exact message. It also checks that the real cell works again after the simulation drops.

2. **Resolved.**
   - `body_error_with_unfinished_children_cancels_once_and_blocks_at_scope_exit` runs 20 seeds, with and without two explicit cancels. It asserts:
     - the TASK-2 winner (the child's error after an explicit cancel, else the body's error);
     - exactly one `cancelled` record;
     - exactly one blocked `scope_exit` record;
     - the child's `error` or `echo` outcome.
   - The assertions are deterministic across seeds. The child can finish only after it observes cancellation, and the body reaches no point between the cancel and the scope exit, so the child is always unfinished at `wait_for(ScopeExit)`.
   - My round-1 fix asked for "one `cancel` point per call". That no longer applies, because Codex finding 1 removed the cancel point. The new `cancel_takes_no_scheduling_point_...` test covers that change. The real-in-sim assertion is covered under finding 1.

3. **Resolved.**
   - `Simulation::drop` now computes `ended` under the lock, drops the guard, and then asserts `ended || thread::panicking()`. A failing assert therefore no longer poisons the runtime lock, and it no longer aborts an unwind already in progress.
   - The lock acquisition still goes through `sim_lock`, so an already poisoned lock still aborts. That behaviour is unchanged and intended.

4. **Resolved.**
   - Before aborting, `Baton::pass`'s `None` arm now lists each blocked task from `Scheduler::blocked()`, with its operation (Debug) and its span.
   - `poisoned_lock_and_empty_runnable_set_abort_the_process` checks two things in a subprocess: SIGABRT and the line `task 0 blocked in Join(0) at`.

5. **Resolved.**
   - `Scheduler::event` now always writes `wait`, as `"wait":null` when an event has none. This applies to `exit`, `cancel` and a finishing task's `sched`.
   - `runnable` is written as an explicit comma-separated JSON list, no longer through `Debug`.
   - Nothing in `design/`, `conformance/` or `research/` pins the old byte shape. The format stays provisional until fx-nkv.7.

## New findings from the fix diff

None.
- `Operation::Cancel` is still built, for the first-cause trace record (scheduler.rs:118), so it is not dead code.
- `Scope::cancel` still checks the simulation identity through `Context::simulated`, even without the point.
- The design/03 change removes only the status paragraph. The file now matches `d98ae1a`.

DONE
