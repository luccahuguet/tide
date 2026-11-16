Model: claude-opus-5-5 · Date: 2026-11-16

Verdict: request_changes

## Scope

Candidate 1b05613 on fx-woj8, `git diff main...HEAD` (sha256 66c1d4aa…e6e9, verified), touching
crates/tide-std/src/task/simulation_tests.rs (+361) and research/34-effect-interruption-oracle.md (new).
Read: bead fx-woj8 (`br --no-db show`), AGENTS.md, design/03 §6.1–6.3 (TASK-1..6, JOIN-1..2) and §9.3,
design/reference.md §7, design/01 D5, D7, D8, D18, design/02 §1.3 (drop order), conformance/rules.md
(Drop/TASK rows). Runtime: crates/tide-std/src/task.rs (scope, shield, checkpoint, yield_now,
Scope::spawn/set_deadline/cancel, run, Task::join), existing simulation_tests.rs and control_tests.rs.
Checked Effect v3 pages for fibers, scope, timing-out, and resource-management intro against the
note's citations. No build or test was run, per the brief.

## Blocking findings

B1. The note and two test comments say the spec forbids effectful destructors, but no Tide rule does.
- simulation_tests.rs:196-197, the T4 comment: "admitted Tide destructors cannot use Shared handles (D5/D7)".
- simulation_tests.rs:362-363, the T8 comment: "effectful destructors are not admitted Tide code".
- research/34 "Tests and proof boundary": "These internal harness hooks force otherwise inaccessible
  runtime interleavings. They neither admit effectful Tide destructors … D5/D7 and TASK-6 still
  constrain admitted code."
- D5 says only that abandoned values are dropped and that `Drop` impls are verified without assuming
  the invariant. D7 restricts `Pure` and authority-free code, and nothing in it covers `impl Drop`.
  TASK-6 constrains std's own `Drop` ("`Drop` in std does no I/O beyond closing descriptors"), not
  user destructors. design/02 §1.3/§6 and conformance/rules.md (DROP-1/2) cover drop order and drop
  check only. Under D7/D8, a user struct may hold a `Shared<T>` handle or a `&Clock`, and its
  `Drop` may call `with` or `sleep`. Only the closure passed to `with` must be `Pure`.
- Failure scenario: the paper, or a later design decision, relies on note 34 as evidence that
  destructors cannot reach a checkpoint. It would then treat T4/T8's interleavings as unreachable
  in admitted code, when admitted code can reach them. The note's "otherwise inaccessible" also
  undersells what T4 and T8 cover.
- The citations do not support the claim, which the brief asks this review to catch. The note is
  part of this change, so the fix falls in an edited tier.
- Fix (text only): say these destructors use handles that admitted user `Drop` impls may also hold,
  and that TASK-6 limits only std's `Drop`. If a restriction on user destructors is wanted, report
  it as a design gap; this bead should not assert one.

## Non-blocking findings

N1. T8 does not isolate the deadline. simulation_tests.rs:365-367 says "Nothing requested
cancellation, so only the deadline … can make this fail." That depends on when the runtime counts
the child's panic. `run` records the panic only after `catch_unwind` returns, after unwinding.
TASK-1/TASK-2 say a child's panic cancels the scope, "in scheduler order", without fixing whether
that happens at the raise or at completion. Under a conforming runtime that cancels at the raise,
`checkpoint()` returns `Err(Cancelled)` with no deadline at all, so the test would not show that the
deadline fired during unwinding. The assertions that matter still hold: the panic payload survives
and scope exit waits until 3 s. Reword the comment and the note's "observes the deadline's
cancellation inside the unwinding destructor" (rule 12), or drop that claim.

N2. A failure in T8 aborts the whole test binary. Its `assert!`, `unwrap` and `assert_eq!` run
inside a destructor during unwinding (simulation_tests.rs:362-368). If any fails, it panics while
already panicking, which aborts the process. That takes down every other tide-std unit test in the
run with poor diagnostics, and `bounded`'s timeout cannot catch it. The module already has `rerun`
for tests that may abort; consider running T8 through it.

N3. Two citations are imprecise (nit).
- Rule 4 cites #interruption-model for "interruption stops the computation and runs finalizers". That
  section says only that interruption is fully asynchronous; the finalizer sentence is under
  #interrupting-fibers.
- Rule 2 says `forkScoped`/`forkIn` let a child outlive its spawner. The docs say that only for
  forkScoped ("Such fibers can outlive their parent fiber"); forkIn is described only as choosing the
  scope.
- Every other cited rule matches the v3 pages: supervision, daemon, Fiber.interrupt waiting and
  interruptFork, exited-fiber interrupt, reverse finalizer order, Exit-receiving finalizers,
  uninterruptible acquire, onExit/onError uninterruptible cleanup, TimeoutException,
  uninterruptible wait, disconnect, and Scope.extend on a closed scope.

N4. The note's proof sentence points to a temporary location (nit). "Workspace and conformance
results, candidate identity, and logs are recorded in the implementer's report" refers to a file
under ~/.cache that does not land. AP-PROOF-001 says a temporary path is not proof. Point to the
bead or the commit's Proof trailer, or drop the sentence.

N5. Naming (nit).
- In T1 the loop variable `cancel` means "the body returns Err", not `Scope::cancel`. Calling it
  `body_fails` would match the note ("body failure").
- T9's "drops_once" (`*drops == 1`) asserts Rust's semantics that a closure's locals drop once.
  That is trivia under AP-TEST-001; the real checks are `join() == Ok(31)` and the scope staying `Ok`.

N6. T9 couples to private internals. It polls `child.done.ready()`, a private field and a private
fn. The note justifies this, but a runtime refactor could break the test without any change in
behavior. Acceptable as is.

N7. Overlap with existing tests, for information only. T5 overlaps `shields()` in control_tests.rs,
T7 overlaps `scope_deadlines_preserve_partial_ok_and_rewrite_only_own_child_errors`, and T6 overlaps
`body_error_cancels_children_and_body_panic_wins_after_cleanup` and
`earliest_spawned_panic_beats_other_panics_and_errors`. Each new test adds a distinct interleaving:
- T5: a cancel arriving during a shielded wait rather than before it;
- T7: cleanup under shield crossing the scope deadline;
- T6: panic time order opposite to spawn order, during the body's cancellation.

None of them is a pure duplicate.

## Reasoning

Coverage of the five required cases is complete:
- nested scope close order: T1;
- cancel during cleanup or Drop: T4 and T5;
- a child panic during parent cancellation: T6;
- a deadline expiring while children unwind: T7 (Err-propagation cleanup) and T8 (panic unwinding);
- cancel of an already-finished task: T9.

I traced each test against the spec and runtime and found it checks Tide's rule, not Effect's or an
accident, and would fail if the cited rule broke:
- T1: if JOIN-1 broke, "inner-closed"/"outer-closed" would precede the drops. If TASK-2 stopped
  cancelling nested scopes, `sim_elapsed` would be 1 s instead of 0. The `Err(Body)` result
  exercises echo ancestry through a nested scope correctly: C's Err is independent inside the inner
  scope, but P's bit is set because the cancellation came from the outer body.
- T2: if dropping a handle detached it, `child_done` would be false after scope exit.
- T3: it checks repeated `Err(Cancelled)` before and after a successful shield; `!finished` holds
  because `cancel` is not a scheduling point (TASK-4); join waits for destructors (JOIN-2); and
  `Err(Child)` is reported after an explicit cancel, as TASK-4 and reference §7.2 require.
- T4: the stage handshake through `Shared::with` forces the order deterministically, and `with`
  keeps succeeding after cancellation (TASK-5).
- T5: a broken shield would end the sleep at 1 s, not 2 s.
- T6: the reported panic is "first-child", by spawn order, although "second-child" panics first in
  virtual time. That is TASK-2 rule 1, and the body panic wins (TASK-1).
- T7: expiry alone does not fail the scope, `Err` after the own deadline becomes `DeadlineExceeded`,
  and scope exit waits past the deadline.
- T9: join returns the buffered `Ok` and the scope stays `Ok` after a cancel that arrives once the
  child has completed.

Determinism: every ordering asserted is forced by joins, the Shared handshake, or distinct virtual
instants. Each seed builds a fresh Simulation and fresh recorders. No test has an operation timer
pending while it spins on `yield_now` or `with`, so the stall guard cannot fire. Wall-clock time
enters only through `bounded`'s 60 s cap. I found no flakiness.

The note's mapping is sound. The counts are right (6 agree, 7 differ, 3 not covered), every covered
rule names a test, and the bead's two required differences are recorded as "differ by design", with
Tide's rule deciding: no detached fibers, and level-triggered `Err(Cancelled)`. The paper paragraph
is accurate. No runtime bug was claimed and I found none.

The one blocking item is B1, a factual claim about admission that the cited decisions do not
contain. The fix is text-only, so it needs no test rerun, and the finder's re-check covers just that
wording.

## Author response (tide-lead, 2026-11-16, commit d82695e)

- B1: fixed. The T4 and T8 comments and research/34's harness paragraph no longer claim a rule against effectful destructors; the note says admitted user destructors can reach both interleavings (TASK-6 limits only std's `Drop`, checked in design/03). Text-only, so no re-check (AP-PROOF-001).
- N1: fixed. The T8 comment and the rule-12 mapping say the deadline attribution rests on this runtime counting the child's panic after unwinding, which TASK-1/2 leave open.
- N3: fixed. Rule 4 also cites #interrupting-fibers, and rule 2 attributes outliving the spawner to `forkScoped`.
- N4: fixed. The note points to the bead and the commit's Proof trailer.
- N2: no change. A failing assertion that aborts the test binary still fails the suite loudly, and the abort was confirmed by a mutant with the deadline removed.
- N5, N6, N7: no change (nits, or no action asked).
