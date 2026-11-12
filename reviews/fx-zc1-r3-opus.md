Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-12

Verdict: approve. Landing still needs both rebased suites on 5e94fa3 to exit 0. They were running when I wrote this: `workspace-rebased.log` was empty and `rebased.status` did not exist. I did not read their results.

This was a narrow re-check (AP-PROOF-001) of round-2 findings 1 and 2, and of the coordinator's rebase.
- Read-only: no edits, staging, commits or `br` writes.
- No builds or probes.
- `trials/e1-checked/` was not opened.

Identities checked:
- `git diff 25c6255 f7f4e67 | sha256sum` = `57a6de0a…ececf`.
- `git diff c8f3c79 5e94fa3 | sha256sum` = `d7cdd7a5689109f0a1d9d78c30f8aa5185a247d461e91f82c9f5b8a02cd17cb8`.
- The worktree was clean at HEAD `5e94fa3`.

## Finding 1: fixed

`design/03-authority-concurrency.md:1324-1329` now states the rule as implemented. A release grants slots only to a ticket that heads every count it queued in, and reserves them before that ticket's task becomes runnable. A ticket blocked on a full inner count holds back later tickets of the outer count. This matches `task/limit.rs:97-124` and decision 3. The FIFO-per-count and all-or-none sentences are unchanged.

## Finding 2: fixed

- `crates/tide-std/src/task.rs:1483-1485`: `if let Err(second) = catch_unwind(..) { std::mem::forget(second); fatal_simulation(..) }`. The second payload is moved into the binding and forgotten, so its destructor never runs, and the abort follows.
- `crates/tide-std/src/lib.rs:144` documents the abort under PANIC-2.
- The new regression test `control_tests.rs:581-628` (`isolated_payload_drop_panic_aborts_without_dropping_the_second_payload`) runs a subprocess on both runtimes with a 10 s timeout and closed stdin.
  - The second payload exits 99 if it is ever dropped.
  - The test asserts SIGABRT and the diagnostic.
  - The author reports it failed before the fix (`fix-r2-regression-before.log`) and passed after (`fix-r2-task.log`, 56 passed). I read the test but did not rerun it.

## Nits 3 and 4: fixed. Nothing new from the fixes

- **3:** `task.rs:500-508` now gives the `Signal` contract fx-o4h needs: one fresh one-shot `Signal` per waiting task, one per waiter for multi-waiter resources, and lock order runtime before resource.
- **4:** simulated `Task::join` (`task.rs:1515`) now calls `wait(..)`, which passes no `Signal`. Completion therefore has one readiness owner, `Runtime::wait_done` → `scheduler.finished(child)`. That stays correct on every path that ends a child:
  - `run` calls `scheduler.finish` and then `wake_waiters`;
  - native-spawn failure calls `abandon` and then `wake_waiters`;
  - the not-started path calls `register` and then `abandon`, so the child is already `Finished` before any join.
- The real-side join keeps its `Signal`, and the author reports a byte-identical seed-5 trace before and after.
- The fix delta (`git diff c9a5f58 f7f4e67`) contains only these four changes and the test. It changes no other behaviour.

## Rebase: correct

- **Automatic merge, code:** for each of `task.rs`, `scheduler.rs`, `limit.rs`, `control_tests.rs`, `simulation_tests.rs`, `time.rs`, `tide-admit/tests/build.rs`, `design/03` and `conformance/cases`, I compared the `+/-` lines of `git diff 25c6255 f7f4e67` with those of `git diff c8f3c79 5e94fa3`. They are identical, so fx-zc1's patch was replayed unchanged.
- **Merged scheduler:**
  - fx-ugm's `operation(task, pending)` signature and fx-zc1's `deadlock` loop agree: `deadlock` calls `self.operation(id, wait)`.
  - `event` emits fx-zc1's `until`/`isolated` fields after fx-ugm's `targets`.
  - `kind()` maps the cell variants to `cell` and fx-zc1's new variants to `cancel` and `spawn`.
- **Semantic interaction with fx-ugm's set waits:**
  - Cell sets take scheduling points only at `begin_set` (`CellsAcquire`) and `end_set` (`CellsRelease`). Under the simulator there is no point between them, so the task holding the baton acquires and releases without yielding. `SharedCell::acquire` (`sync.rs:45-58`) asserts that a cell is never contended. A simulated cell wait therefore never blocks the scheduler, and the deadlock report has nothing to report for cells; `blocked()` only lists `wait_for` waits.
  - On the thread runtime, a cell waits on its own `Condvar` (`sync.rs:61-66`) outside `wait_for`. That predates this change: base `25c6255` already had it for single-cell `with`, and fx-ugm kept it. It is correct by TASK-5, since `Shared::with` acquisition is not cancellable and so needs neither deadline nor cancellation wakes.
  - `SimCell::context` reads `CURRENT` with `task.is_some()`, so cell calls inside a simulated `spawn_isolated` task or a `shield` resolve to the right runtime.
  - I found no lock nesting between cell mutexes and the runtime or count locks. The runtime lock is never held across a cell `Condvar` wait.
- **Conflict resolutions, docs:**
  - In `conformance/README.md`, fx-ugm's `shared_sets` paragraph is intact and fx-zc1's `task_controls`/`limit_deadlock` paragraph follows it.
  - In `crates/tide-std/src/lib.rs`, fx-zc1's limit/deadline/shield/isolation bullet sits before fx-ugm's `Shared` bullet, the PANIC-2 line is present, and "Not provided yet" has dropped both the task-control and the multi-cell items.
  - The `README.md` and `conformance/rules.md` hunks are unchanged from the candidate. The new `PAR-1` row from main is kept.

## New findings

None.

One observation, outside fx-zc1's scope and needing no action here: on the thread runtime, `Shared` waits are a second blocking mechanism beside `wait_for`. Decision 2 listed `Shared::with` as a non-cancellable user of the one primitive. The cell `Condvar` predates this change and fx-ugm kept it, so moving it is for the cell code's owner, if anyone wants to.

DONE
