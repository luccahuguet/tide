Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-12

Verdict: approve

Candidate: worktree `.worktrees/fx-zc1`, HEAD `c9a5f587c491f71d969549b0bf5a090e5bfead19`, base `25c6255`. `git diff 25c6255 c9a5f58 | sha256sum` = `7fcc292b…e625`, matching the frozen candidate. The worktree was clean at the start and end of the review. Nothing was edited, staged or committed, and nothing was written through `br`. Held-out `trials/e1-checked/` was not opened.

Probes: none ran. At review time (00:05 local) the quiet window in `eon-overseer/quiet-until` was active (1794456000 = 01:00 local), so the brief barred builds. The findings below come from reading the code. Each claim about runtime behaviour is reasoned from Rust semantics, and the reasoning is cited. No test run backs them. The suite results are the author's (workspace 198 passed; conformance 286 PASS / 12 AWAITS / 0 FAIL). I did not rerun them.

## What I verified (no finding)

- **Limits** (`task/limit.rs`):
  - A request is pushed into every count's queue while all of the chain's count locks are held, so any two requests keep the same relative order in every queue they share. `grant` (limit.rs:97) succeeds only for a request that is the head of every count and has capacity in each. The oldest waiting request is therefore never waiting on another queued request. Queued tickets cannot deadlock each other, and overlapping chains cannot starve or deadlock each other.
  - Chains only ever append fresh `Limit`s (`Tasks::limit`), so shared counts always appear in the same order and the lock order stays consistent.
  - `Slots::drop` reads `ready` under the count locks that `grant` holds while setting it. The decision "release a granted slot or withdraw the ticket" therefore cannot race a concurrent grant.
  - A cancelled waiter passes its slot on, whether it was cancelled before or after the grant (task.rs `spawn`, `Wake::Cancelled`/re-check path). Slots are released:
    - in `run` after `finish`, so a sibling woken by the release sees a failure cause first;
    - on a caught panic;
    - when the native spawn fails (the job and its `Slots` are dropped);
    - on the not-started path.
  - No task holds a count while waiting for another.
  - The zero-count panic comes before the cancellation checks, as decided.
- **Deadlines:**
  - `set_deadline` stores `min(old, t)`, and a deadline already past cancels immediately.
  - `Runtime::cancel` keeps the first cause and stops at shields.
  - `finish` rewrites only `Error` under `Deadline(own scope)`, so nested scopes keep their children's own errors. The tie test covers this.
  - In simulation, `Runtime::pass` → `advance` → `fire_timer` cancels through `expire` and runs `wake_waiters` before `pick` returns any task.
  - Class-0 keys sort before class-1 keys at the same instant, and ancestors sort first by scope id.
  - `cancel` and `finish_scope` remove stale class-0 timers.
  - The stall guard counts only `OPERATION_TIMER` keys.
  - On the real side, deadlines are refreshed lazily in `Context::lock` and bound `park_timeout`. `set_deadline` unparks waiters so they recompute their timeouts. A late wake reports `Cancelled`, and design/03 TASK-4 states exactly this.
- **Wait primitive:**
  - `wait_for` is the only blocking owner for joins, scope exit, slots, isolation, sleeps and file latency. The native joins that remain are reclamation after completion.
  - Real side: the waiter is registered in `runtime.waiting` and in `Signal.thread` under the runtime lock before it checks readiness. Wakers either see the registration or the waiter sees `ready` (both are SeqCst; the thread mutex orders them), and park/unpark tokens cover the remaining window, so no wakeup is lost.
  - The lock order is runtime → count → signal-thread. No path takes the runtime lock while holding a count or signal lock.
  - The shape is sound for fx-o4h and fx-hxw to extend. A blocking operation passes an `Arc<Signal>` that the resource wakes, and the simulator reads the same readiness in `wake_waiters` (but see finding 3).
- **`spawn_isolated`:**
  - After `pass`, the isolated thread runs no user code and does not block. The `_enter` restore on a fresh native thread is `None`, so it takes no lock. `handle.join()` therefore cannot strand the baton.
  - The cloned `Context` keeps `at`, so checkpoints and echoes reach the caller's scopes. The caller's wait is `cancellable: false`.
  - The message rule follows decision 4, and the panic hook is untouched.
  - `spawn_scoped` with `f: 'static` and `T: Send` is sound inside the native scope.
  - The entry-stall drain path is correct.
- **Deadlock report:** it reuses fx-o8j's `Scheduler::deadlock`. Every blocked task's `Operation` (`limit(id)`, `isolated(id)`, …) and span is listed. The terminal `sched`/`deadlock` event carries the blocked list. The kinds and fields follow research §1.9 (`explicit`/`failure`/`deadline`+`until`, `set_deadline`+effective `until`, `limit`→started/not_started, `isolated:true` with exit result).
- **Ownership and equivalence:**
  - The duplicate stub items are removed, and the conformance build includes production `task.rs` verbatim.
  - README, crate docs, conformance README and rules match the code.
  - No dependency was added. No dead code found beyond the test-only helpers marked `#[cfg(test)]`.
  - A cancelled limited spawn returns without a scheduling point. That matches the existing `yield_now` and past-`sleep` precedent, so it is not a finding.

## Findings

1. **Non-blocking.** design/03 §6.1 misdescribes multi-count fairness.
   - Evidence: `design/03-authority-concurrency.md:1324-1327` says "A release reserves capacity directly for the oldest eligible ticket". The implementation (`crates/tide-std/src/task/limit.rs:97-124`) grants only to a ticket at the head of every count it queued in. Take outer n=2 and inner n=1, with ticket R1=[outer, inner] waiting on a full inner count, followed by R2=[outer]. R2 waits even though outer has room: this is strict per-count FIFO with head-of-line blocking, as the author's report states. A reader takes "oldest eligible" to mean that R2 is granted.
   - Why non-blocking: the first sentence ("FIFO per count") is true, and the behaviour honours decision 3 (FIFO with direct hand-off, no barging).
   - Fix: replace that sentence with: "A release grants slots only to a ticket that heads every count it queued in, reserving them before its task becomes runnable, so later spawns cannot take them; a ticket waiting on a full inner count also holds back later tickets of the outer count." This states the rule, so it is not text-only, and it needs my narrow re-check.

2. **Non-blocking.** The second panic payload escapes the abort meant to stop it.
   - Evidence: `crates/tide-std/src/task.rs:1482`, `if catch_unwind(AssertUnwindSafe(|| drop(payload))).is_err() { fatal_simulation(..) }`. The `Result` holding the second payload is a temporary of the `if` condition, so Rust drops it before the block runs. If that payload's `Drop` also panics, the isolated thread unwinds out of `from_payload` before `tx.send`, `finished.wake()` and `pass`. On the real runtime the caller then parks forever. Under the simulator the baton is never passed. That is the stranded-caller outcome the abort exists to prevent.
   - Also: the abort itself is documented only in the author's report. `crates/tide-std/src/lib.rs:142` and the new paragraph in design/03 §6.1 cover suppressed scope payloads, not the isolated payload.
   - Fix: `if let Err(second) = catch_unwind(..) { std::mem::forget(second); fatal_simulation(..) }`. Then extend the lib.rs:142 sentence, or the design/03 §6.1 paragraph, to say that a panic while dropping an isolated task's panic payload aborts (PANIC-2).

3. **Nit.** The `Context::wait` comment is stale, and the `Signal` contract is unstated for fx-o4h.
   - Evidence: `crates/tide-std/src/task.rs:500-506` still says later wait kinds "need a caller-supplied readiness check or resource-owned waiter handles". `Signal` is now that handle. It records exactly one waiting thread (`Signal.thread`), which `wait_for` overwrites, and it is one-shot.
   - Fix: say that a blocking resource passes one fresh `Signal` per waiting task and wakes it on hand-off. A multi-waiter resource (channels) keeps a queue of `Signal`s, one per waiter.

4. **Nit.** Simulated joins have two readiness owners.
   - Evidence: `crates/tide-std/src/task.rs:1513-1519`. The simulated join passes `Some(self.done.clone())` as well as `Operation::Join(child)`, which `Runtime::wait_done` already resolves through `scheduler.finished(child)`. Both mean the same thing, but there are two owners of "child finished" in the simulator.
   - Fix (optional): pass `None` on the simulated path, or drop the `Join` arm in `wait_done` and rely on `done`.

5. **Out of scope, pre-existing; report only.** `conformance/rules.md` keeps stale line anchors in neighbouring rows: TASK-2 `03:1135`, TASK-5 `03:1223`, JOIN-1 `03:1240`. Those rules are now at design/03 lines 1347, 1459 and 1481. This change converted only the rows it touched to `03 §6.2`, so the list now mixes both styles. No action is needed in fx-zc1.

Findings 1 and 2 touch text and code that this change edits. I rate them non-blocking because neither changes a recorded decision or the v0 behaviour on any input that is not pathological. The author may fix them before landing. If they do, I re-check only those fixes.

DONE
