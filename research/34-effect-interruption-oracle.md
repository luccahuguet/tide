# Effect interruption as a Tide test oracle

Bead: `fx-woj8`. Verified 2026-11-16 UTC. The requested unversioned Effect URLs
redirected to **v3**, whose API includes `forkDaemon`; the links below retain
that version (the API pages report 3.22.4). This is a list of
documented behavioral rules relevant to scope interruption, not an exhaustive
Effect API inventory or a claim to have executed Effect.

Tide's oracle remains [D8 and D18](../design/01-design.md),
[TASK-1–6 and JOIN-1–2](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines),
and [reference §7](../design/reference.md#7-concurrency). D18 preserves Rust's
ordinary returns and destructors; Effect's resource `Scope` is not itself a
Tide task scope. “Agree” below means the stated observable property agrees,
not that the APIs or all their semantics are interchangeable. “Not covered”
means Tide has no corresponding contract to test, not a failing implementation.

## Numbered rules and mapping

| Rule | Effect rule and source | Mapping to Tide and simulator check |
| --- | --- | --- |
| 1 | Ordinary `fork` children end with their supervising parent, including normal completion. [Fibers: supervision](https://effect.website/docs/v3/concurrency/fibers#fork-with-automatic-supervision). | **Differ by design.** D8/JOIN-1 wait for children on normal scope exit; they do not cancel them just because the body returned `Ok`. Failure cancels nested scopes too (TASK-2). **T1** checks both normal exit and body failure through two child generations. |
| 2 | `forkScoped`/`forkIn` bind lifetime to a chosen scope; with `forkScoped`, a child can outlive its spawning fiber. [Fibers: lifetimes](https://effect.website/docs/v3/concurrency/fibers#lifetime-of-child-fibers). | **Agree** on scope ownership. A passed `Scope` owns its children even when another task spawns them (§6.1); **T2** observes the spawning task finish before its child, then the enclosing scope wait for that child. Tide joins instead of interrupting at normal scope exit (rule 1). |
| 3 | `forkDaemon` children survive parent termination in a global scope. [Fibers: daemon](https://effect.website/docs/v3/concurrency/fibers#fork-in-global-scope-daemon). | **Differ by design.** D8 rejects unstructured spawn. Dropping a `Task` does not detach it (JOIN-1/§6.3). **T1/T2** drop handles and observe completion before the owning scope returns. These are positive lifetime tests, not admission tests for nonexistent daemon APIs. |
| 4 | Effect uses asynchronous interruption; interruption stops the computation and runs finalizers. [Fibers: interruption model](https://effect.website/docs/v3/concurrency/fibers#interruption-model), [interrupting fibers](https://effect.website/docs/v3/concurrency/fibers#interrupting-fibers). | **Differ by design.** TASK-3 returns `Err(Cancelled)` at checkpoints, repeatedly. Ordinary code chooses whether to propagate it; it may still return `Ok`. **T3** checks repeated failed operations around successful shielded cleanup; **T4** checks that cancellation cannot skip destructors. |
| 5 | `Fiber.interrupt` normally waits for termination; background interruption is separate. [Fibers: interrupting](https://effect.website/docs/v3/concurrency/fibers#interrupting-fibers). | **Differ by design.** `Scope::cancel` requests cancellation without waiting or scheduling (TASK-4); `join` and scope exit separately wait, shielded (JOIN-1/2). **T3** observes immediate cancel, delayed cleanup, then completed destruction before join returns; the no-scheduling-point half of TASK-4 is checked by the existing `cancel_takes_no_scheduling_point_and_the_next_checkpoint_observes_it`. **T6** checks scope exit drains all children despite panics during cancellation. |
| 6 | Interrupting an already-exited fiber resumes immediately. [Fiber API: interrupt](https://effect.website/docs/v3/api/effect/Fiber#interrupt). | **Agree** on completed work staying complete. **T9** observes actual task completion, cancels its still-open scope twice, and checks the buffered `Ok` value, one destructor call, and no virtual-time advance (TASK-3/JOIN-2). Tide cancels scopes, not individual handles. |
| 7 | Finalizers run on success, failure, and interruption. [Scope: addFinalizer](https://effect.website/docs/v3/resource-management/scope#addfinalizer). | **Agree** on releasing live locals through ordinary exit/unwinding (D8, D18, TASK-1). **T1** records destructors on success and propagated cancellation; **T6** records them during child and body panic unwinding and checks TASK-2's panic precedence. Aborts/crashes are excluded. |
| 8 | Default scope finalizers run in reverse registration order. [Scope](https://effect.website/docs/v3/resource-management/scope). | **Agree** for the corresponding lexical-local case: Rust locals drop in reverse declaration order, and nested task scopes join before their caller continues (D18/JOIN-1). **T1** checks inner child locals, inner closure, parent local, then outer closure. This imposes no LIFO order on sibling tasks, struct fields, or returned values and creates no Tide finalizer registry. |
| 9 | `onExit`/`onError` cleanup is uninterruptible, including cleanup after interruption. [Resource management](https://effect.website/docs/v3/resource-management/introduction#onexit). | **Differ by design.** TASK-6 requires explicit `shield` for cleanup I/O; std `Drop` does no I/O beyond closing descriptors. **T3/T5** exercise cleanup after cancellation and cancellation arriving during cleanup. **T4** ensures an in-progress destructor and later destructors finish. Cancellation remains pending after a shield. |
| 10 | An explicitly `uninterruptible` effect disables interruption within that region. [Effect API](https://effect.website/docs/v3/api/effect/Effect#uninterruptible). | **Agree** on hiding enclosing cancellation with `Tasks::shield`. **T5** also tests Tide's specific boundary: an inner cleanup scope's own deadline still cancels its checkpoints, while the surrounding shield remains usable (TASK-6). |
| 11 | An interruptible timeout terminates the operation with `TimeoutException`. [Timing out](https://effect.website/docs/v3/error-management/timing-out#handling-timeouts). | **Differ by design.** TASK-4 scope expiry cancels but does not itself fail the scope: all-`Ok` partial results survive; a failing child after its own scope deadline yields `DeadlineExceeded`. **T7** checks both outcomes after cleanup crosses the deadline. Operation-specific timeouts remain distinct. |
| 12 | A timeout waits for uninterruptible work before reporting its result. [Timing out](https://effect.website/docs/v3/error-management/timing-out#handling-timeouts). | **Agree** on soft deadlines and waiting for cleanup (TASK-5/JOIN-1). **T7** checks a one-second scope deadline while shielded cleanup lasts three seconds; **T8** forces expiry during Rust panic unwinding, observes cancellation inside the unwinding destructor (the deadline's, since this runtime counts the child's panic only after unwinding; TASK-1/2 leave that timing open), and checks the panic survives, after destruction. Tide's outcome selection remains TASK-2, not `TimeoutException`. |
| 13 | `disconnect` lets timed-out uninterruptible work continue in the background. [Timing out: disconnection](https://effect.website/docs/v3/error-management/timing-out#disconnection-on-timeout). | **Differ by design.** D8/JOIN-1 forbid leaving task work behind at scope return. **T7/T8** check that scope return/panic propagation waits for cleanup beyond the deadline. |
| 14 | `acquireRelease` acquisition is uninterruptible and successful acquisition registers release. [Scope: acquireRelease](https://effect.website/docs/v3/resource-management/scope#acquirerelease). | **Not covered.** Tide has no generic `acquireRelease` combinator. Individual capability operations and descriptor close have their own TASK-3/6 rules; no generic atomic-acquisition guarantee is inferred. |
| 15 | A registered finalizer receives the scope's `Exit` status. [Scope: addFinalizer](https://effect.website/docs/v3/resource-management/scope#addfinalizer). | **Not covered.** Tide has no `Exit`-receiving finalizer API. Rust `Drop` is not such a callback. |
| 16 | `Scope.extend` may execute a workflow against an already-closed scope. [Scope: extending](https://effect.website/docs/v3/resource-management/scope#manually-create-and-close-scopes). | **Not covered.** Tide has no resource-scope extension API; a task `Scope` cannot escape its lexical lifetime. No simulator behavior is invented for an unusable handle. |

Counts: **6 agree, 7 differ by design, 3 not covered**. Every covered rule
points to at least one test of Tide's specified behavior.

## Tests and proof boundary

All tests are in [task/simulation_tests.rs](../crates/tide-std/src/task/simulation_tests.rs),
run over seeds `0..20`, inside the existing bounded harness. Mutexes/atomics
only record observations; waits use simulated capabilities. T9 inspects
the task's completion signal because a destructor flag precedes task completion and
joining would consume the handle before cancellation could be tested.

| ID | Test name (prefix `effect_oracle_`) |
| --- | --- |
| T1 | `nested_scope_close_order` |
| T2 | `passed_scope_outlives_spawning_task_but_not_scope` |
| T3 | `cancel_is_level_triggered_and_join_waits_for_cleanup` |
| T4 | `cancel_during_drop_runs_remaining_destructors` |
| T5 | `cancel_during_shielded_cleanup_keeps_inner_deadline` |
| T6 | `child_panics_during_parent_cancellation_drain_all_children` |
| T7 | `deadline_expires_while_children_clean_up` |
| T8 | `deadline_during_panic_unwind_preserves_panic` |
| T9 | `cancel_after_task_completion_preserves_value_and_drops_once` |

T4's destructor waits on a `Shared` handle, and T8's destructor sleeps under
`shield` during panic unwinding, to force these interleavings. No Tide rule
forbids such destructors: a user `Drop` may hold a `Shared` handle or a
`&Clock` and call `with` or `sleep` (TASK-6 limits only std's own `Drop`), so
admitted code can reach both interleavings. T7 separately covers explicit I/O
cleanup on ordinary `Err` propagation. Tests cover observable order only
where Tide specifies it, not a global sibling-finalization order. Suite
results and candidate identity are recorded on bead fx-woj8 and in the
commit's Proof trailer.

## Paragraph for the paper

Effect offers a deliberate escape from parent supervision: `forkDaemon`
attaches work to the global scope, so it survives its spawning parent's exit.
Tide's D8 instead makes the owning task scope a lifetime boundary: even a
dropped handle leaves work that scope exit must join. Background work can use
an enclosing scope passed explicitly, but it cannot escape that scope. This
trades Effect's daemon flexibility for a local argument about borrowed data,
cleanup, and when outstanding work has ended. [Effect's daemon documentation](https://effect.website/docs/v3/concurrency/fibers#fork-in-global-scope-daemon).
