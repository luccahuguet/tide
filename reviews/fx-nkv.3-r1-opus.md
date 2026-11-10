Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-nkv.3 round 1 — Opus review (report only)

## Verdict

**request_changes**, confidence 0.85.

The runtime shape is sound: one Runtime owner, no nested locking, no user code under either mutex, first-cause tracking, spawn-order selection, and level-triggered checkpoints. Three findings in v0-core behavior block landing. Each has a small fix:

- B1: the added `From<Cancelled> for TaskError` breaks a normative design example.
- B2: echo bits are set in intermediate nested scopes, which hides a nested scope's child failure.
- B3: the suppressed-outcome summary is written to stderr, but SINK-2's default discards debug records.

## Scope

Read:
- AGENTS.md, the GOVERNANCE Merge Rule (from AGENTS.md), and the bead and parent through `br --no-db`, including the held-out rule and the r1 N2 comment.
- `impl-report.md`.
- The full diff `07e104b..102a3e2`, all of `crates/tide-std/src/task.rs` after the change, and the conformance stub's `mod task`.
- The build.rs addition, the new and edited conformance cases, and the README and lib.rs docs.
- `crates/tide-std/src/io/shared.rs`, for its `From` impls and `ErrorKind::Cancelled`.
- `design/03-authority-concurrency.md`: §6.1–6.3 (TASK-1..6, JOIN-1/2) and §2.4 (SINK-1..10).
- `design/01-design.md`: the Scope-results bullet and the tier table (structured tasks and sinks are v0 core).
- `design/reference.md` §7.1/7.2 and its std error list.
- `design/02-memory.md` §10.12.
- `design/reference-companion.md` `fetch_all`.
- The implementer's `workspace-final.log`, which shows the E0283.

I did not open `trials/e1-checked/`.

Ran one single-file rustc probe on a private archive of `102a3e2`'s `task.rs`, with minimal `io`, `__sink` and `capability!` shims. Before each compile I checked the quiet-until predicate, load (2.6–3.2) and memory (14 GB available). Probe file: `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-nkv-3/1c8ddea5-05b9-43a5-ace8-e523eded8b63/scratchpad/probe/probe.rs`. Toolchain: the worktree's `rustc`, edition 2024.

- **P1:** 02-memory §10.12 `parallel_sum`, copied verbatim, fails to compile: `error[E0283]: type annotations needed for task::Scope<'_, '_, _>`. rustc points at task.rs:181 (`E: From<TaskError>`) and task.rs:401 (the new impl).
- **P2:** An outer scope is cancelled by a sibling's failure. Inside it, a nested scope's child returns `Err` after `tasks.checkpoint()?`, and the nested body returns `Ok(5)`. Output: `inner scope returned Ok(5)` and `outer returned Err(DeadlineExceeded)`. One `debug: task scope suppressed 1 …` line is printed for the inner scope.

No cargo builds or tests were run.

## Blocking

### B1 — `From<Cancelled> for TaskError` breaks `scope(...)?` inference in spec examples

**Where:** `crates/tide-std/src/task.rs:401-405`. The edits in `conformance/cases/memory/scope_borrow_accept.rs:9` and `conformance/cases/tasks/task_lifecycle.rs:26` work around it.

**Failure scenario:** 02-memory §10.12 `parallel_sum` is a normative example. It has the form `fn …() -> Result<u64, TaskError> { tasks.scope(|s| { …; Ok(()) })?; … }`. Here `E` is constrained only by `TaskError: From<E>`.

- Before this change, the reflexive impl was the only candidate, so `E = TaskError` was inferred.
- After it, both `TaskError` and `Cancelled` are candidates, and the program fails with E0283 (probe P1; also `workspace-final.log`).

`scope_borrow_accept.rs` is that example's conformance guard, and the change edited it to add `-> Result<_, TaskError>` rather than keeping the form compiling. The guard was weakened to fit the regression.

Any user program written from the design hits this: a `let x = tasks.scope(...)?;` not in tail position, inside a function returning `Result<_, TaskError>`. The fx-nkv.2 plan-review comment records that several E1 tasks import `task::{TaskError, Tasks}`, so held-out programs are exposed too. The held-out rule allows the coordinator to confirm by running an E1 reference.

The impl is not in the spec. 03 §6.1 and reference §7.1 list no conversion from `Cancelled`.

**Not affected:**
- `From<Cancelled> for io::Error` adds no new ambiguity. `io::Error` already has `From<ErrorKind>`, `From<TryReserveError>` and `From<IntoInnerError<W>>`.
- User error enums with `#[from] TaskError` were already multi-candidate.

**Smallest fix:**
- Delete `impl From<Cancelled> for TaskError`.
- Revert the two conformance annotations, so the §10.12 form is guarded again.
- Write E = `TaskError` checkpoints in tests and the fixture as `tasks.checkpoint().map_err(|_| TaskError::Cancelled)?`, or use an enum with `#[from] Cancelled`.
- Keep the `io::Error` conversion.
- If `checkpoint()?` straight into `TaskError` is wanted as a language convenience, that is a design change for `design/01-design.md` and the reference first, together with an answer to the inference cost. It is not something this slice should add on its own.

### B2 — Echo bits are set in intermediate nested scopes, hiding a nested scope's child failure

**Where:** `crates/tide-std/src/task.rs:94-116` (`Runtime::observe`).

**Failure scenario (probe P2):**
1. Outer scope O is cancelled by sibling failure `Failure(O)`.
2. Child B of O opens nested scope I, which inherits cause `Failure(O)`.
3. I's child calls `tasks.checkpoint()?` and returns `Err`.
4. `observe` walks from `{I, child}` up to O. It marks I's child as an echo (line 104, first iteration), then B's bit in O.
5. I therefore returns its body's `Ok(5)`, and B continues as if I's work had completed.

**What the spec requires:** TASK-2 (03 §6.2) gives every scope one cause, and "an enclosing scope's cancellation" is a cause of its own. A child's bit is set only for a cancellation "that the body's `Err` or a sibling's failure caused". It adds that "a checkpoint in a nested scope sets the bit only for a cancellation that reached the nested scope from this one". I's cause is O's cancellation, so I's child is an independent failure in I, and I must return `Err(Cancelled)`. B's bit in O is set, so O still reports the root cause. The final outer result is the same; the nested scope's own result is wrong.

**Inconsistent with inherited explicit cancellation:** The code already treats an inherited `Explicit` cause correctly: no mark, and the child's error is reported. An inherited `Failure` gives the opposite result in the nested scope, although the spec treats both as "an enclosing scope's cancellation". This is the acceptance's negative case, an independent child failure hidden by echo handling, one level down.

**Why tests missed it:** `nested_cancellation_preserves_first_cause_and_echo_ancestry` asserts only the outer result. Case 0 is exactly this scenario, and the inner scope's `Ok(())` is never checked.

**Smallest fix:** In `observe`, mark a bit only at the origin scope. Walk up while `scopes[at.scope].cause == Some(cause)` until `at.scope == origin`, then set `children[child].echo` if `at.child` is `Some`. Intermediate scopes, and the nested scope itself, get no mark. Add one assertion to the nested test that the inner `scope` returns `Err(Failure::Child)` in case 0. The rest of the cause machinery stays as is.

### B3 — Suppressed-outcome summary written to stderr, contradicting SINK-2's defaults

**Where:**
- `crates/tide-std/src/task.rs:225-235`
- `crates/tide-admit/tests/build.rs:110-113`
- `README.md` ("produce one runtime debug summary on stderr")

**Failure scenario:** TASK-2 sends unreported failures, echoes and panics to "one `debug!` record". SINK-2 sets the default: "`log` at `info`, to stderr". A `debug!` record is therefore discarded unless `main` configures `root.log`, and `root.log` does not exist in this slice.

The candidate instead prints `debug: task scope suppressed N …` to the `err` sink unconditionally, in debug and release. Every Tide program with a multi-failure scope gains stderr output that a spec-conforming runtime would not produce. Examples are the fixture's second scope and any `x.join()?` body whose child failed. The line also uses a record format the spec does not define. This matters to:
- any test or trial oracle that compares stderr;
- the coming event traces (fx-nkv.6), where SINK-6 puts every record into the trace;
- the reported-effects work (SINK-9), since the program appears to use `err`.

The build test asserts the line, so it locks in behavior that goes against the spec.

**Smallest fix:**
- Emit nothing. That matches the spec's default configuration exactly.
- Keep a `shortcut:` comment saying the summary becomes a `debug!` record when the log sink lands.
- Change the build test to assert empty stderr.
- Update the README sentence.

If the record must be observable before `root.log` exists, use an env-gated or test-only hook. Do not use the user-visible `err` sink.

## Non-blocking

**N1 — Suppressed payloads are dropped outside any panic boundary.**
- When the scope resumes a panic, any `E` or panic payload whose `Drop` panics turns into a double panic and aborts (`task.rs:215-243`: the local `payloads` vector is dropped while unwinding). That matches PANIC-2.
- On the non-panic path, the panic appears in the scope's caller.

This is the same class as r1 N2. Mention it next to the N2 sentence in README and lib.rs, so the seeded scheduler (fx-nkv.5) accounts for it.

**N2 — Record `yield_now`'s public signature in its owner.**
- The capability tables (03 §1.1 line 94, reference §6.1/line 340) list `yield_now` as a checkpoint.
- Neither 03 §6.1's API block nor reference §7.1's code block gives a signature.
- This slice fixes `fn yield_now(&self) -> Result<(), Cancelled>`.

Per AGENTS.md Decisions, record it in its owning document: add the line to 03 §6.1 and reference §7.1 in this change or in an fx-djp follow-up. Text-only; it does not block.

**N3 — Design note for fx-djp: explicit cancel followed by a failed join.**
- Once `cancel()` is the first cause, a sibling's later failure leaves that cause unchanged.
- A child that then does `x.join()?` gets no bit, as the spec requires: a checkpoint there would return a cancellation caused by `cancel()`.
- Its `TaskError::Cancelled` is therefore an independent failure. If the joiner was spawned before `x`, the scope reports `Cancelled` rather than `x`'s error.

The implementation follows TASK-2 and TASK-4 ("`cancel()` … a failing child's own error is reported") literally. Whether the spec meant this is a design question, not a code defect.

**N4 — Test style is fine.** No sleeps. The `while tasks.checkpoint().is_ok() { yield_now() }` spins end deterministically, and channels order every cross-task step. Once B2's assertion and B3's empty-stderr assertion are added, coverage fits the risk.

**N5 — Contention is acceptable for v0.** Every checkpoint takes the tree-wide Runtime mutex, even when nothing is cancelled. That is fine on the thread runtime. If it ever shows up, a per-scope atomic "cancelled" fast path is the obvious fix. Nothing to do now.

## Decisions

1. **Expose `Scope::cancel` — agree.** It is in 03 §6.1 and reference §7.1. It tests TASK-3 directly, and the code keeps TASK-4's rule that explicit cancellation is no failure and marks no echo.
2. **`checkpoint` and `yield_now` as the only checkpoints; yield returns `Result<(), Cancelled>` — agree.**
   - These are exactly the `Tasks` checkpoints in the capability table that this slice can expose. Unlimited spawn, joins and in-memory I/O are correctly not checkpoints (TASK-5, JOIN-2, §6.1).
   - Checking before and after the native yield is harmless and keeps the operation level-triggered.
   - Record the signature (N2).
   - Keep `From<Cancelled> for io::Error`. Drop `From<Cancelled> for TaskError` (B1).
3. **One Runtime owner and one mutex — agree.**
   - Lock order is trivial: the payload and runtime mutexes are never held together; `run` releases the payloads guard before `finish`.
   - No user code runs under either mutex, so poisoning can only follow an internal indexing bug.
   - Scope IDs are monotonic, so the `range(scope+1..)` propagation visits parents before descendants. Removing completed scopes is safe, because every context that names a scope is gone once its native scope has joined.
   - The TLS context is restored by `Enter` on return and on unwind.
   - This is the right single owner for fx-nkv.5 to reuse.
4. **Causal echo propagation — agree with the mechanism, disagree with the marking (B2).**
   - Correct: first-cause-per-scope; stopping propagation at a nested scope's own earlier cause; never inspecting `E`; failed joins observing at the caller; successful joins not observing.
   - Wrong: marking every scope on the path. Only the origin scope's child bit should be set.
5. **stderr suppressed-outcome summary — disagree (B3).** Discard it for now; SINK-2 defaults make that the spec-conforming output. Replace the shortcut with a `debug!` record when the log sink exists.
6. **Keep the bounded channel; attribute a destructor panic to the task that runs it — agree.** This is what r1 N2 requires, and a rendezvous channel would stall on a forgotten handle. The documentation says so and makes no replay claim. Extend the same note to suppressed payloads (N1).

## Reasoning

Acceptance has three parts:
- Independent failure not hidden: met at one level, broken one level down (B2).
- TASK-2 precedence: met. Body panic wins, then earliest-spawned child panic, then earliest-spawned non-echo error, then the body. Selection is by spawn index under the lock, so it does not depend on interleaving; only echo classification depends on timing, which is inherent.
- Level-triggered cancellation at the exposed operations: met, including inheritance into scopes created after cancellation and propagation into scopes that already exist.

No deadline, limit, shield or isolation expansion. Production/conformance equivalence holds: the stub includes `task.rs` verbatim and drops its duplicate `Cancelled`.

The three blockers are in v0-core behavior (structured tasks; sinks):
- B1 changes the public API so that a normative design example no longer compiles, and the conformance guard for that example was edited instead.
- B2 departs from TASK-2's per-scope cause rule.
- B3 adds output that the default sink configuration forbids.

Each fix is local: delete one impl and revert two annotations; narrow one marking loop and add one assertion; remove one emission and change one assertion. None touches the unsafe sites or the lifetime-erasure design, whose SAFETY arguments still hold after the added `runtime` and `Context` fields. Those fields are `Send + Sync` and do not depend on `E`, and the auto-trait test is unchanged.

After the fixes, the author re-runs the task unit tests, the two task build tests and the three affected conformance cases. Only B2 changes runtime behavior, and it does not cross a trust boundary, so a targeted re-check suffices rather than a full re-review.

DONE
