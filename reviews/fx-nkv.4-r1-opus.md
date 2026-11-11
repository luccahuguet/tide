Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-10

# fx-nkv.4 round-1 Opus review

Candidate: `0fef60d` on `8b0e134`. I recomputed the diff SHA-256 as `5f24c85bd0c92d943c9900ed61c18e983a019ea472769717fc0e70fb7e4ea178`, which matches.

## Verdict

approve_with_alternatives

None of the findings blocks. The acquisition and poisoning mechanism is correct, and it matches 03 §6.5 (LOCK-1, -3 for one cell, -4, -5, -6) and reference §7.4 in its signatures, bounds, variance and auto traits. Production and conformance share one definition through `include!`. The held-out rule holds. I grepped `trials/e1-checked` for the new fixtures' shapes and found no overlap.

## What I checked

- **Ticket FIFO and missed wakeups.** A caller reserves a ticket (`sync.rs:48`) before taking the mutex. `serving` changes only under the mutex. The predicate is re-checked under the mutex (`wait_while`, `:51`), and every advance is followed by `notify_all` (`:63`). A waiter whose predicate check came before an advance either sees the new value or gets woken by that notify. No wakeup is lost, and spurious wakeups are handled. `Relaxed` on the single ticket counter gives a total order. Payload synchronization goes through the mutex. Wrapping is harmless while fewer than `usize::MAX` tickets are outstanding.
- **Poisoning and unwinds.**
  - The callback's unwind is caught inside the guard. It never crosses the mutex, so the host mutex is never poisoned.
  - On a panic, the code sets `poisoned`, advances `serving`, releases the guard, notifies, and then calls `resume_unwind` with the original payload.
  - The panic hook does not run twice. The test checks that the payload survives through `Scope`.
  - `AssertUnwindSafe` is justified: poison keeps the torn payload unreachable through any safe API (LOCK-6).
- **Deadlock.** A callback cannot re-enter `with`, because the `Pure` bounds plus FX0603 reject a captured handle and FX0606 rejects a handle in the payload. One cell has one lock, so acquisition order is not an issue.
- **Cancelled scopes.** `with` never consults cancellation, which is what LOCK-3 requires ("not a checkpoint"). The runnable counter exercises this.
- **Type shape.** `Shared<T>` stays invariant: the `UnsafeCell` inside `Mutex<State<T>>` makes it so. It is `Send + Sync` exactly when `T: Send`, the same as the old `PhantomData<Mutex<T>>` stub. `Clone` is unbounded. `LockError` derives `Debug, Clone, Copy, PartialEq, Eq` and implements `Display` and `Error`.
- **Admission keys.** `tide_std::sync::Shared` (`pure.rs:97`, SYNC) and `tide_std::sync::Shared*` (`pure.rs:232`) resolve to the real production path. The walk stops at the classified handle, so the new private fields (`Arc`, `Mutex`, `Condvar`, `AtomicUsize`) never reach user-facing purity verdicts. Nothing in admission had to change.
- **Equivalence (fx-nkv acceptance).** Every production-exposed item comes from the shared file: `Shared`, `new`, `clone`, `with` and `LockError`. The `with2`, `same` and `with_all` items that remain in the stub are compile-time placeholders with no production counterpart. The parent acceptance allows that, and the lib.rs header labels them.
- **Focused probe.** The worktree had no local changes. The load gate was open (1-minute load 7.5, 16 GB available, no quiet window). I ran `nice -n 19 CARGO_BUILD_JOBS=4 cargo test -p tide_std --lib sync` with a scratch `CARGO_TARGET_DIR`. It passed 2/2, and 30 repeated runs passed 30/30 with no flake or hang.

## Findings

1. **Non-blocking: `LockError` loses `Aliased` relative to reference §7.4.**
   - Evidence:
     - `crates/tide-std/src/sync.rs:73-76` defines only `Poisoned`.
     - `design/reference.md:717` gives `enum LockError { Aliased, Poisoned }`.
     - Before this change the conformance stub carried both variants (removed lines in `conformance/rustc/std/src/lib.rs`).
     - The stub still exposes `with2` and `with_all` returning `LockError` (`conformance/rustc/std/src/lib.rs:250-260`).
   - Effect: a reference-faithful program that names `LockError::Aliased` now gets E0599 from the conformance oracle as well as production. The type gets a breaking variant addition when fx-ugm lands. The deferral is sanctioned: `trials/e1-checked/protocol.md:270` assigns `Aliased` to fx-ugm, and 03 §6.5's new note says it is planned. So this is a choice, not a defect.
   - Fix I'd want (pick one):
     - (a) Keep `Aliased` in the shared `sync.rs` now, with a doc line saying only the planned multi-cell operations return it. The cost is one variant and one `Display` arm, and the reference type shape stays intact on both surfaces.
     - (b) Keep the omission, and name the missing variant in `README.md:191` and the unsupported list in `crates/tide-std/src/lib.rs` (about line 43), which today list multi-cell ops, `same` and `reset` but not `Aliased`.

   I prefer (a).

2. **Non-blocking: downstream constraint the ticket scheme imposes, unrecorded.** `with` reserves its ticket before it can block (`sync.rs:48-52`). That carries two obligations for later beads:
   - **fx-ugm.** An implementation that reserves tickets on all of a `with2`/`with_all` operand set up front and then waits in id order can deadlock. Example: X gets A#0 and Y gets A#1, then Y gets B#0 before X gets B#1. X holds A and waits on B#1, while B is serving Y's unused #0 and Y waits on A. Tickets must be taken lazily: reserve on each cell only after acquiring the previous cell in id order.
   - **fx-nkv.5.** Under a single-runnable baton scheduler, the scheduling point has to come before `fetch_add`, or the wait has to hand off the baton. A task descheduled between reserving and acquiring would otherwise park every later contender in an OS condvar wait that the baton holder can never satisfy.

   Neither obligation needs rework here. Fix: one comment line at `sync.rs:47` stating both constraints, or a note on fx-ugm and fx-nkv.5 when the coordinator records this review.

3. **Non-blocking: wake-on-poison for an already queued waiter is untested.** `shared_propagates_panics_and_poison_is_visible_through_clones` (`sync.rs:137`) calls the clone only after the panicking scope has finished. Nothing proves that a waiter blocked *during* the panicking callback gets woken and receives `Err(Poisoned)`. That path depends on `serving` advancing on the panic arm. Today the code shares that step with the normal arm, so the risk is low. If a regression broke it, the symptom would be a hang, not a failure.
   - Fix: extend the FIFO test's shape. Hold the cell in a callback that panics after the channel release, queue one clone waiter (wait until `next >= 2`), then assert that the waiter returns `Err(LockError::Poisoned)` and the scope re-raises the original payload.

4. **Nit: a panic between ticket reservation and service would strand the queue.** `cell.state.lock().unwrap()` and `wait_while(..).unwrap()` (`sync.rs:51-52`) run after `fetch_add`. If either ever panicked, that ticket would never be served, and every later caller on the cell would hang instead of seeing `Poisoned`. This is unreachable today because every guard-held step is either panic-free or inside `catch_unwind`, so the host mutex never poisons. That invariant is implicit, though.
   - Fix: use `unwrap_or_else(PoisonError::into_inner)` on both, or add a one-line comment stating the invariant.

5. **Nit: the counter fixture's rule tags omit LOCK-3.** `conformance/cases/tasks/shared_counter.rs:1` tags LOCK-1, -4, -5 and SEND-3. The case's distinctive check is LOCK-3's cancelled-scope acquisition (`scope.cancel()` before spawning), and the driver test in `crates/tide-admit/tests/build.rs` asserts it at runtime. Leaving the tag off may be deliberate, since the stub oracle only type-checks.
   - Fix: add LOCK-3 if `//@ rules` means "rules this case's executed form exercises". Otherwise leave it.

## Downstream fit

- **fx-ugm:** it adds a cell id in `new` (deterministic under the simulator, LOCK-3), `same` via `Arc::ptr_eq`, `reset` by acquiring and clearing `poisoned`, and multi-cell acquisition in id order with lazy tickets (finding 2). It needs no rework of this code.
- **fx-nkv.5:** acquisition is a clear hook point, subject to finding 2's ordering constraint.

DONE
