# T063 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed and reflected into the live spec

## Decision

Current recommendation:

* **narrow Ferox v1 concurrency to a small async core**

Recommended v1 concurrency surface:

* `async fn`
* explicit `await`
* a small structured child-task form, preferably `async let`

Recommended deferrals:

* general task groups
* rich cancellation semantics
* any concurrency feature that depends on a more ambitious runtime model

## Why

### 1. The current model is broader than it first looks

The current spec includes:

* async suspension semantics
* structured child-task semantics
* task groups for dynamic fan-out
* cancellation propagation
* cancellation observation points
* parent/child completion obligations

That is already close to a runtime model, not just a syntax choice.

### 2. Ferox does not need all of that in v1

Ferox needs:

* explicit async behavior
* predictable suspension points
* no hidden fire-and-forget default

Ferox does **not** need:

* a fully developed structured-concurrency framework in the first version
* a rich cancellation theory before the base async model is even proven

### 3. Dynamic task groups are a scope multiplier

`task group` is attractive, but it introduces:

* group lifecycle rules
* group result collection rules
* group cancellation rules
* interaction between parent scope, child tasks, and group state

That is too much for a first concurrency model.

## Recommended V1 Concurrency Shape

### 1. Use dedicated async syntax

Recommended direction:

* use `async fn` for suspendable functions
* keep `await expr` explicit

This is simpler and more legible than expressing async through a general effect annotation.

### 2. Keep exactly one structured child-task primitive in v1

Recommended direction:

* keep `async let` as the only built-in child-task form in v1

Reason:

* it covers the small fixed-fan-out case
* it keeps concurrency lexical and visible
* it avoids introducing a second abstraction such as `task group` before the first one is proven

### 3. Defer `task group`

Current recommendation:

* remove `task group` from v1

Reason:

* it is the point where structured concurrency starts becoming a broader framework instead of a small language feature
* it can return later once the runtime and async semantics are proven

### 4. Narrow cancellation

Current recommendation:

* defer rich cancellation semantics from v1

Keep only a minimal structured rule:

* if a parent async scope exits early, unfinished child tasks created by that scope do not silently detach

Do **not** promise more until the execution model is clearer.

This means v1 should avoid detailed guarantees about:

* cancellation observation points beyond `await`
* long-running explicit cancellation checks
* sibling-cancellation policy in complex group scenarios
* cancellation-completion contracts for advanced task structures

## Proposed V1 Model

Recommended v1 async/concurrency model:

* `async fn` defines a suspendable function
* `await expr` marks suspension points explicitly
* `async let` introduces a child task inside an async scope
* child tasks are lexical and do not become detached background tasks by default
* the language does not yet include dynamic task groups
* the language does not yet specify a rich general cancellation system

## Why This Is Better

This narrower model:

* keeps async explicit
* stays compatible with Ferox's explicitness goals
* gives implementation a realistic target
* avoids turning v1 into a language-plus-runtime research program

## Consequences For The Spec

If adopted, this decision implies follow-up edits to:

* `spec/LANGUAGE.md`
  * replace effect-based async description with dedicated async syntax
  * keep `await`
  * cut or defer `task group`
  * cut or heavily narrow the cancellation section
* `spec/IMPLEMENTATION.md`
  * stop assuming a broader effect-and-concurrency checker for early semantic work
  * align bootstrap expectations with a smaller async model

## Open Questions

This review does **not** yet settle:

* the exact static rules around unfinished `async let` bindings on all control-flow paths
* whether v1 should permit implicit join, explicit join only, or a restricted structured fallback on scope exit
* what minimal runtime model Ferox should describe publicly

Those questions are smaller and more manageable than the current full design.

## Current Recommendation

For the adversarial spec pass:

* keep explicit async
* keep `await`
* keep at most one structured child-task primitive
* defer `task group` and rich cancellation semantics from v1
