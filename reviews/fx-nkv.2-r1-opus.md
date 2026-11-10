Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-nkv.2 round 1: Opus review (converge round, reports only)

Candidate `f7f77a2` on base `0583ecb` (`git diff 0583ecb f7f77a2`, 7 files, +456/-69).

## Verdict

**approve_with_alternatives**, confidence 0.8.

Nothing blocks. The change meets fx-nkv.2's Acceptance. Each `unsafe` block and impl holds under its stated invariant. The tests fit the risk. One alternative is worth applying in the fix round: restore `TaskError::DeadlineExceeded` (N1, Decision 6). The other findings are text accuracy or notes for fx-nkv.3.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md (Merge Rule).
- Beads fx-nkv.2, fx-nkv, fx-nkv.3 and fx-zc1 (`--no-db`).
- The implementer's report and the full diff.
- `crates/tide-std/src/task.rs`, read whole; `crates/tide-std/src/lib.rs`; `root.rs` (`capability!`).
- The conformance stub's `task` module and crate docs, and `crates/tide-admit/tests/build.rs` (helpers and the new test).
- The conformance cases that now run against the production signatures: `scope_borrow_reject`, `scope_escape_reject`, `scope_passed_reject`, `send_table_reject` and the new `task_lifecycle.rs`.
- Spec: 03 §6.1–§6.3 (TASK-1, TASK-2, JOIN-1, JOIN-2), 02 §8.1–§8.3 (SEND table, SCOPE-1..4, child failure) and reference §12.3.
- The pinned nightly's scoped-thread implementation: `library/std/src/thread/lifecycle.rs` 55–100 and 159–200, and `scoped.rs` 135–175. I read it to check Decisions 2 and 4.

Grepped for `DeadlineExceeded` and `TaskError::` across the repository, including only a name-level grep of `trials/e1-checked`. I opened no held-out content.

Ran: no `rustc` probes. The overseer quiet window was active for the whole review (`quiet-until` 1794273300; now 1794272548). I ran no cargo, as the brief requires. Every check result in this review is static reasoning or the implementer's claimed proof, which I have not reproduced.

## Blocking

None.

## Non-blocking

**N1. `TaskError` drops a reference variant.** `crates/tide-std/src/task.rs:153-156`, and through `include!`, the conformance stub.

Reference §7.1 and 03 §6.1 define `pub enum TaskError { Cancelled, DeadlineExceeded }`. With the stub now sharing production's one-variant enum, the conformance oracle's verdicts differ from the reference in two ways:
- An exhaustive `match e { TaskError::Cancelled => .. }` is accepted, where the reference makes it E0004.
- A program that names `TaskError::DeadlineExceeded` is rejected (E0599).

No current case or E1-checked file names either variant, so nothing regresses today. But any program written against the interim std breaks when fx-zc1 adds the variant back. Keeping the variant does not advertise a deadline operation; it is a type-level contract.

Smallest fix:
- Restore `DeadlineExceeded` with a doc line saying no operation in this slice returns it.
- Give it a `Display` arm, such as "task deadline exceeded".
- Leave producing it to fx-zc1.

See Decision 6.

**N2. A claim about destructor panics is broader than the code.** The report's Decision 2 says "Discarded-result destructors run inside the child's panic boundary". The comment at `task.rs:115` is accurate for its own path; the report's claim is not, in general.

The claim holds only when the `Task` is dropped before the child calls `tx.send`. If the child finishes first, the value stays buffered in the channel, and dropping the `Task` later runs `U`'s destructor on the dropping thread. If that thread is the body, a panicking destructor becomes a body panic; if it is another child, that child's panic. Which one happens depends on timing.

The panic still propagates either way, so this slice is correct. Two consequences for fx-nkv.3 and the seeded scheduler:
- TASK-2 attributes panics by child, so this attribution is a race on the thread runtime.
- If the body is already unwinding, a second panic here aborts (PANIC-2), as with Rust's `ScopedJoinHandle`.

The race comes with forgettable handles: a rendezvous channel would make the child always drop the value, but a forgotten handle would then stall the scope. So keep the design and correct the wording in the report or bead record. Optionally, add one line to the fx-nkv.3 handoff.

**N3. The `Failures` SAFETY comment is slightly imprecise.** `task.rs:60-61` says "every access through this pointer holds the mutex". The pointer is dereferenced to `&Mutex<..>` without holding the lock (line 127); only the access to the data is under the lock.

Suggested text: "the pointer is only ever dereferenced to `&Mutex<Vec<thread::Result<E>>>`, which is `Sync` because `E: Send`, and the data is read and written only under that lock." This is a text-only fix.

**N4. Failure paths are not exercised through admission.** The admitted test (`build.rs:90-100`) runs only the success fixture. The panic, failure and destructor paths are unit-tested in `task.rs` under cargo's Rust std build, from the same source. That is enough here, because the code is identical and the driver does not change panic strategy (no `panic=abort` in `tide-admit/src`). Consider an admitted exit-code check for a child panic when fx-nkv.3 adds failure results.

**N5. A doc sentence in the conformance stub no longer fits.** `conformance/rustc/std/src/lib.rs:14-15` says "Each of its own bodies is `unimplemented!()`". "Its own" still excludes the included `task.rs`, so the text stays true. Keep it in mind if more production modules are included; nothing is needed now.

## Soundness review of the unsafe code

**`Failures(*const ())` and its `Send`/`Sync` impls** (`task.rs:57-64`).
- Only `Tasks::scope` constructs it (line 27). That function has `E: Send`, so `Mutex<Vec<thread::Result<E>>>` is `Sync`.
- The pointee is a stack local that is dropped only after `thread::scope` returns, including on unwind.
- `thread::scope` returns or resumes unwinding only after `num_running_threads` reaches 0 (`scoped.rs:153-157`).
- A thread's count is decremented in `Packet::drop` (`lifecycle.rs:171-198`), after the thread's closure, and so after `run`'s push.
- The Acquire load in the wait loop orders the push before `into_inner`.
- `Scope`'s `E` is invariant (`fn(E) -> E`), so `run::<F, U, E>` casts back to exactly the constructor's type.

Sound.

**Erasing `E` with a function pointer** (`task.rs:93-98`). The boxed job captures `F`, a `SyncSender<U>`, `Failures`, and an `unsafe fn` whose type does not mention `E`. So the job's `'s` bound needs only `F: 's` and `U: 's`, which the public signature already has.

`E` gets no proven `E: 's` bound, but it holds semantically. `E` is a type parameter of the `Tasks::scope` call, so its lifetimes outlive that call. The native `'s` ends inside the call. A child can only produce an `E` from data that lives at least that long; `task_error_can_borrow_local_state` exercises the case that matters.

Sound.

**`dyn Spawner<'s>` erasing the native `'env`** (`task.rs:66-76`). This is safe code that the compiler checks. The public `'env: 's` bound it gives the user is true: the `&Tasks` borrow outlives the call, and the native `'s` ends inside it. Both `'s` and the trait object are invariant, so a `Scope` cannot shrink or escape. The SCOPE-2 escape cases in `scope_escape_reject` now run against these exact types.

**Panics and unwinding.** All user code in a child runs inside `run`'s `catch_unwind`:
- the call to `f` and the drop of its captures;
- the drop of a discarded value when `send` fails.

Outside it, only the `SyncSender` drop and the push under an unpoisonable lock run, and neither can panic. So the native `a_thread_panicked` never fires, and the original payload is re-raised with `resume_unwind`.

A body panic goes through native `scope`'s `catch_unwind`, the join and `resume_unwind`. The failures queued by then are dropped during unwinding, as TASK-1 says.

If a native spawn fails, `Scope::spawn` panics before it builds a `Task`. The boxed job, with its `tx`, is dropped, and std's packet drop keeps the running-thread count correct.

**Forgotten and dropped handles.** Native `ScopedJoinHandle`s are dropped immediately (line 74), so a forgotten Tide `Task` leaks only its receiver and buffered value. This matters because in std, forgetting a `ScopedJoinHandle` keeps `Arc<Packet>` alive, so `decrement_num_running_threads` never runs and the scope waits forever (`lifecycle.rs:171-198`). Tests cover drop and forget for both success and panic.

**Ordering of a failure and `join`.** `tx` is dropped when `run` returns, after the failure is pushed. So a sibling that does `x.join()?` always records its echo after `x`'s own failure. Even with the completion-order shortcut, JOIN-2's example ("rule 2 reports `x`'s error") therefore holds for a direct join chain.

## Acceptance check

- Children join before borrowed state drops: native scope, plus the drop-asserting `Borrowed` tests.
- No child outlives its scope, including grandchildren and dropped or forgotten handles: tests at `task.rs:206-256` and the conformance fixture.
- A single panicking child propagates with its original payload after all children finish: `task.rs:230-256`.
- A panicking body joins its children and keeps unwinding: `task.rs:258-280`.
- TASK-1's "cancels the scope" has no observable effect yet, since this std exposes no checkpoint. The bead assigns observable cancellation to fx-nkv.3.
- Production/conformance equivalence: the stub `include!`s `task.rs`, so there is one definition. The stub-only `spawn_isolated`, `limit`, `Panicked` and `Cancelled` are items production does not expose.
- No doc overclaims. The README, the crate docs and `Tasks::scope`'s doc each say that cancellation and multi-failure selection are unavailable. The shortcut is marked at line 32, and fx-nkv.3's Acceptance owns TASK-2 precedence.

## Decisions

1. **Use `std::thread::scope`: agree.** 02 §8.2 prescribes it, and it owns JOIN-1, including on body unwind.
2. **Drop native handles and use a one-slot channel per `Task`: agree.** I confirmed the stall on a forgotten `ScopedJoinHandle` in the pinned source, as described under Soundness. Correct the report's destructor-boundary claim (N2).
3. **Keep the exact generic signatures and the erased failure pointer: agree.** I checked the safety argument above.
   - Alternative considered: declaring `E: 's` on `Scope` as an implied bound would make `Scope::spawn` capture-safe. It still would not let `Tasks::scope` prove `E: 'scope` inside the native closure, so it would not remove the unsafe code and would change the struct's declared shape. Rejected.
4. **`Task` invariant in `U` and `E`: agree.** It matches Rust's own `ScopedJoinHandle`, which is invariant in `T` through `Packet`'s `UnsafeCell<Option<Result<T>>>` (`lifecycle.rs:159-161`). No design text fixes `Task`'s variance; the old stub marked covariance as a GUESS.
   - Cross-child joins need `E: Send + 's`. The old stub needed the same: `PhantomData<fn() -> (U, E)>` imposed `E: 's`, and its `Send` impl required `E: Send`. So nothing regresses.
5. **Keep single child errors and select by completion order for now: agree.** It is marked and owned by fx-nkv.3, and a panic still beats an error (rule 1).
6. **Expose only `TaskError::Cancelled`: disagree, non-blocking.** Restore `DeadlineExceeded` without anything that produces it, as in N1. The plan comment assigns producing deadlines to fx-zc1; it does not require the enum to diverge from reference §7.1 in the meantime.
7. **No allowlist widening: agree.** These are Tide-owned items, no Rust thread or sync API is re-exported, and the admitted debug and release runs are the evidence (the implementer's claim).

## Reasoning

The design uses the native scoped-thread primitive for the memory-safety rule (JOIN-1). Tide adds a small amount on top: result delivery and a failure list. The two lifetime erasures exist only to keep the specified public bounds (no `E: 's` on `spawn`), and each is justified by a fact the compiler cannot see but the call structure guarantees. That fact is that the type parameters of the `Tasks::scope` call outlive the native scope.

The tests target the risky edges:
- forgotten handles;
- grandchildren;
- body and child panics;
- destructor panics;
- an `E` that borrows local state;
- the auto-trait table.

The existing compile-fail corpus now checks the production signatures, not a separate stub. The remaining gaps belong to fx-nkv.3 and fx-zc1, which own them explicitly, and the docs label the slice as partial. The one material alternative, the missing reference variant, does not affect any current case or program, so it does not block.

DONE
