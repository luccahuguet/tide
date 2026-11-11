Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-10

# fx-nkv.5 round-1 Opus review

Candidate: `e436752` on `d98ae1a`. The diff's SHA-256 is `cd059b256d0bebeebfa86a86b5bd242577b3f6e8c45c81c44ed4fccbe280b25c` (verified). Its stable patch-id `ce575a5b…` equals that of the author's tested `0e2457a..7ae0542`, so the rebase changed only the base. The suites still need the coordinator's rerun on the rebased tree, as planned. I ran no builds; this review comes from reading the code.

## Verdict

approve

The baton hand-off is correct for every path this slice supports. Determinism holds. The design matches the 2026-11-10 decisions 1–7 and 03 §9.3's point list. The real runtime is unchanged when no simulation is active. The findings below are non-blocking or nits; the author fixes or rebuts each one.

## What I checked and found sound

- **Baton.** All scheduler state, `current` included, lives under one runtime mutex. `pass` notifies the chosen task's condvar while holding that mutex. `Baton::wait` rechecks `current` under the lock before parking and after every wakeup. A notify sent before the child thread parks, as in the delayed start, is therefore not lost; the child's first check sees `current == me` (task.rs:49-58).
- **Spawn.** A spawned child is Starting until the native spawn returns, and pick ignores Starting tasks. If the rejected job's destructor reaches a point, the phantom child cannot win. The abandon path records the child and wakes waiters before `resume_unwind` (task.rs:606-625).
- **Task finish.** Task finish matches decision 7. The closure is consumed inside `catch_unwind`, and a failed `send` drops `U` there. `tx` drops before the hand-off. No user `Drop` runs on a child thread after `pass` (task.rs:672-704). A buffered `U` drops on the receiver's thread, which holds the baton (tested).
- **Join and scope exit.** Both waits use one condition owner, `Runtime::wait_done`, re-evaluated in task-id order after every finish (task.rs:180-204). After `wait_for(ScopeExit)`, the native `thread::scope` join waits only for threads that have already handed off.
- **Shared.** The acquire point comes before ticket reservation and the release point after unlock and `notify_all`. A `try_lock` that returns `WouldBlock` panics (sync.rs:52-88). The cell mutex is never held while the runtime lock is taken. A Pure callback cannot reach `Shared` or `Tasks`, because pure.rs lists both as not Pure, so LOCK-4's never-contended reading holds.
- **No reachable all-blocked state.** `Task` and `Scope` are not Pure (pure.rs:108-109), so this slice cannot pass a handle back to form a join cycle. The `None => abort` in `pass` is unreachable from supported Tide code.
- **Determinism.**
  - IDs are per simulation: tasks, scopes (`Runtime.next_scope`) and cells.
  - State lives only in Vec and BTreeMap.
  - Spans come from `#[track_caller]`.
  - A single runnable task makes no draw and emits no `sched` event.
  - The draw rejects `draw < 2^64 mod bound` from a u64, which is unbiased.
  - SplitMix64 is seeded from the seed XOR a domain constant.
- **Real runtime.** With `simulation: None`, every sim branch is skipped. `run`'s real path still releases the lock before `tx` drops. `Shared::with`'s real path is byte-for-byte the old body.
- **Equivalence.** Production and conformance include the same `task.rs`, `task/scheduler.rs` (through `include!`) and `sync.rs`. No `Tasks(())` construction remains anywhere in `crates/` or `conformance/`. `Tasks` is still Clone + Send + Sync with a private field (ROOT-4).
- **Downstream beads.** Each can build on this without rework:
  - fx-o8j: timers and time advance belong in `Scheduler::pick`'s `[] => None` arm.
  - fx-zc1 and fx-o4h: the shared wait primitive generalizes `wait_for`, `wait_done` and `wake_waiters`.
  - fx-iob: the mode switch can read `CURRENT.task.is_some()`.
  - fx-nkv.6: SimRoot can own the `Simulation` guard, which is !Send.
- **Held-out rule.** Nothing in the diff references `trials/e1-checked/`.

## Findings

1. **non-blocking: real task authority inside a simulation is rejected only by `scope`.**
   - Decision 2 says a handle used outside its own simulation panics. `Tasks::context` enforces this for `scope` with the message "real task authority used inside a simulation" (task.rs:398-412). The other entry points skip the check:
     - `Tasks::real().checkpoint()` on a sim task takes no point and still observes the simulation's cancellation (task.rs:495-501).
     - `yield_now` does an OS `thread::yield_now` with no point (task.rs:505-514).
     - `Shared::new(v, &Tasks::real())` returns an unsimulated cell (`sim_cell` returns `None` at task.rs:382; sync.rs:35).
   - The same gap applies to a real `Shared` used inside a simulation: `with` takes the real ticket path with no acquire or release points (sync.rs:63-73). The run stays deterministic, but those operations drop out of the schedule silently, so the run explores fewer interleavings.
   - It is unreachable today, because `Simulation` is crate-private. It becomes reachable once fx-nkv.6 exposes `Root::sim`, while FX0504's test-only check is still a later bead.
   - Fix: give one owner the check that the current context is not a simulation when the authority is real. Call it from `checkpoint`, `yield_now`, `sim_cell`'s `None` branch, and `Shared::with` when `cell.simulation` is `None`. Extend `simulated_handles_require_their_own_live_simulation` with the real-in-sim cases.

2. **non-blocking: test gaps on simulation paths the change adds.**
   - Three simulated paths have no test:
     - `Scope::cancel` under simulation: its `Cancel` point, and a `cancelled` record only on the first cause transition (task.rs:641-653).
     - The scope-body `Err` or panic path while children are still unfinished: the scope's cancel record, then `wait_for(ScopeExit)` (task.rs:453-465).
     - The "real task authority used inside a simulation" assertion.
   - Fix: add one seeded test over a few seeds. The body spawns a child that loops on `checkpoint`, calls `scope.cancel()` twice, then returns `Err`. Assert:
     - the scope result;
     - one `cancel` point per call;
     - exactly one `"action":"cancelled"` record;
     - a `scope_exit` blocked record;
     - termination under `bounded`.

3. **nit: `Simulation::drop` asserts while holding the runtime lock (task.rs:341-350).**
   - A failing assert poisons the lock. Any later sim lock on that thread then aborts with "simulator runtime lock poisoned" instead of the assert's clearer message. One example is a scope still unwinding after the guard was moved into its body and dropped there. If the drop runs during unwinding, the assert also aborts.
   - Fix: read the condition, drop the guard, then assert, skipping the assert when `std::thread::panicking()`.

4. **nit: the abort in `pass` reports nothing about the stuck state (task.rs:45).**
   - `Scheduler::blocked()` already lists each blocked task with its operation and span. The deadlock report itself belongs to a later slice, but writing that list to stderr before `abort` costs three lines. It would also make fx-nkv.6's abort attribution informative if an internal invariant ever breaks.

5. **nit: trace shape against fx-nkv.6 decision 1.**
   - Decision 1 says each event carries a wait state, but `exit` and `cancel` events, and `sched` events from a finishing task, have no `wait` field (scheduler.rs:116-140, 158-162).
   - `runnable` is printed with Rust's `Debug` format (`[0, 1]`, scheduler.rs:166). That is valid JSON, but it ties the format to `Debug`.
   - Both are provisional until fx-nkv.7. Either record them there, or emit `"wait":null` and write the list explicitly.

6. **nit: docs.**
   - lib.rs:11-12 still lists `Tasks` among the capability types Root carries "as identities". `Tasks` now carries an optional simulation and is no longer `capability!`-declared (lib.rs:53-55 says so). Reword line 11.
   - 03 §9.1 gains the chapter's first implementation-status paragraph (03:1710-1713). The same status already lives in README.md:204 and lib.rs:19. A third copy in the design chapter will need editing when fx-nkv.6 lands. Keep the status in README.md and lib.rs and drop it from 03, unless the coordinator wants 03 to carry availability.
   - sync.rs lost the comment that fx-ugm reserves each ticket after acquiring the previous cell. Research §1.4 keeps that constraint, so this one is optional.

DONE
