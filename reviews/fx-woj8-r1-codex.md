Model: gpt-6.1-sol (high) · Date: 2026-11-16
Verdict: request_changes

## Scope

Read-only GOVERNANCE.md Codex supervisor pass for fx-woj8. Reviewed commit `236392934fb892c0cedfc709743a8bb8b5392692` against main `18b82c785b7cef07d9c145ed74317f9ffbc31377`; `git diff main...HEAD` SHA-256 is `e9d3be4fb474846bcc8c930c0b3ed929e1631dee997f03248dd91ef983f8e23b`. The tree was clean before and after inspection. The pre-rebase author commit and frozen candidate have identical stable patch-id `b262a0a12e96e608336f6df9e0fe263b3c28ea4e`.

Read AGENTS.md, GOVERNANCE.md's Merge Rule, `br --no-db show fx-woj8` (Scope and Acceptance), the author's report, both changed files, D8/D18, design/03 §§6 and 9.3, reference §7, and the surrounding task/scheduler implementation and existing cancellation/deadline tests. Checked the linked official Effect documentation. No repository files changed. Only this requested external report was written.

## Blocking findings

1. **P2 — T8 never observes deadline cancellation during unwinding.** `crates/tide-std/src/task/simulation_tests.rs:359–376`; corresponding claim at `research/34-effect-interruption-oracle.md:32`.

   `effect_oracle_deadline_during_panic_unwind_preserves_panic` installs a one-second deadline, then sleeps for three seconds inside a shielded destructor. Its assertions observe `thread::panicking()`, the original panic payload, destructor completion, and three seconds elapsed. All those assertions also pass if the scope deadline never cancels this scope during the unwind. For example, a regression that loses the deadline timer in this unwind path leaves exactly the same asserted outcome. The checkpoint after `catch_unwind` is outside the closed scope and only establishes that cancellation did not leak outward.

   This matters because “a deadline expiring while children unwind” is an explicit required case. T7 observes deadline cancellation during explicit cleanup, but does not panic or unwind; T8 is the case intended to establish the unwind interleaving. A configured deadline and elapsed time alone do not establish that cancellation occurred before the panic was published.

   Smallest fix: immediately after the shielded sleep returns, while still inside the destructor, assert `tasks.checkpoint() == Err(Cancelled)`. At that point the body returned `Ok`, the child has not published its panic, and no explicit cancellation was requested, so cancellation must come from the deadline. Keep the panic, drop, and time assertions. This strengthens the existing harness-only test without changing runtime behavior or expanding scope. No production runtime defect is asserted by this finding.

## Non-blocking findings

2. **P3 — T3 does not independently prove that cancel takes no scheduling point.** `crates/tide-std/src/task/simulation_tests.rs:165–167`; mapping at `research/34-effect-interruption-oracle.md:25`.

   Zero elapsed time and an unfinished child establish that cancellation does not wait for cleanup. They do not establish the stronger no-scheduling-point rule: an erroneous yield inside `cancel()` could let the child run up to its shielded sleep, then return to the still-runnable parent with time still zero and `finished == false`. This is already covered by `cancel_takes_no_scheduling_point_and_the_next_checkpoint_observes_it` at line 798, which checks the trace delta. No duplicate test is needed; the mapping can point to that existing check when discussing this stronger property. T3's name and its immediate-cancel/join-wait claim are otherwise supported.

## Reasoning

The direction is correct and the remaining change for approval is small. Confidence is high in the source-level findings; test execution is outside this pass.

The Effect rules have sources and the classifications are sound at the explicitly stated observable-property boundary: six agree, seven differ by design, three have no corresponding Tide contract. Rules 1–5 correctly distinguish parent supervision, chosen-scope ownership, daemon lifetimes, interruption, and interruption waiting. The [official Fibers documentation](https://effect.website/docs/v3/concurrency/fibers) supports those distinctions. Rule 6 is supported by the [Fiber interrupt API](https://effect.website/docs/v3/api/effect/Fiber#interrupt), including immediate resumption for an exited fiber. The API page reports 3.22.4.

Rules 7–8 and 14–16 match the [Scope documentation](https://effect.website/docs/v3/resource-management/scope), including reverse finalization, acquisition protection, Exit callbacks, and the closed-scope extension example. The lexical-local qualification in rule 8 correctly avoids inventing sibling LIFO order or a Tide finalizer registry. Rule 9 matches [onExit/onError cleanup](https://effect.website/docs/v3/resource-management/introduction#onexit), and rule 10 matches [uninterruptible](https://effect.website/docs/v3/api/effect/Effect#uninterruptible). Rules 11–13 accurately distinguish timeout results, waiting, and disconnection in the [timeout documentation](https://effect.website/docs/v3/error-management/timing-out#handling-timeouts). Tide's TASK/JOIN rules, rather than Effect's outcome selection, supply the test assertions. The forkDaemon/D8 paper paragraph is accurate: a passed enclosing scope permits longer-lived work without permitting escape from its owning scope.

The five required cases are represented:

- Nested scope close order: T1, on normal completion and body failure, including reverse lexical-local destruction and both scope boundaries.
- Cancel during cleanup or Drop: T3/T5 cover explicit shielded cleanup; T4 forces cancellation between destructor start and completion and checks remaining destruction.
- Child panic during parent cancellation: T6 checks all-child draining, earliest-spawned child-panic selection despite opposite completion order, and body-panic precedence.
- Deadline during unwinding: T8 sets up the intended case, but needs finding 1's observation. T7 separately checks soft-deadline cleanup, successful partial results, and DeadlineExceeded for child failure.
- Cancel after task completion: T9 waits for the completion signal before cancelling twice, then checks the buffered value and single destruction. It avoids confusing a destructor flag with completion publication.

T4 and T8 clearly label their synthetic destructor scheduling/delay as internal harness instrumentation. They do not claim admission of effectful destructors or arbitrary destructor I/O. No additional scope or admission coverage is needed here.

No scheduler-order flakiness was found in the assertions. Seeds are fixed at `0..20`; waits use simulated channels, Shared operations, or virtual sleeps. T1/T2 keep a task runnable until the necessary observations, so virtual time cannot advance early under §9.3. T6's sibling destruction order is forced by distinct virtual wake times after a readiness barrier, rather than by an unspecified sibling-finalization rule. T4 orders events through Shared state, and T9's polling waits for runnable completion without waiting for virtual time. The existing 60-second wall-clock bound remains the harness backstop; these 20 seeds are sampled schedules, not exhaustive interleaving proof.

`git diff --check main...HEAD` passed. Cargo and conformance were not run, as instructed. The author's suite results are pre-rebase evidence; identical patch-id does not establish a passing suite on the rebased tree. The coordinator's rebased-tree proof and subsequent Opus review remain required before landing.

DONE
