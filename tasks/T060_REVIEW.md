# T060 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed and reflected into the live spec

## Decision

Current recommendation:

* **defer the general effect-set model from Ferox v1**

This means Ferox v1 should **not** use a general signature form like `![io, async]` as a core language contract.

## Why

### 1. The effect model is not just syntax sugar

The current design makes effects a core semantic layer:

* every expression, block, and callable body has an effect set
* function and method signatures carry effect annotations
* async behavior is expressed through the effect system
* IO behavior is expressed through the effect system
* parts of the concurrency and cancellation story depend on those effects being coherent

That is not a small surface refinement. It is a major checker subsystem.

### 2. Ferox already has a harder primary bet

Ferox is already betting on:

* ownership
* simplified borrowing
* explicit mutation
* arenas
* explicit shared ownership

That is enough semantic risk for an initial language.

Adding a general effect system makes Ferox harder to reason about precisely where the project says it wants to be smaller and more predictable than Rust.

### 3. The current effect system overlaps with other explicit mechanisms

Ferox already has or plans:

* `unsafe` for unsafe capability
* `Result` and `Option` for recoverable failure and absence
* explicit mutation and synchronization wrappers
* explicit `await`

That means a general effect-set layer is partly duplicating explicitness that the language already gets through other mechanisms.

### 4. The implementation cost is real

If effect sets stay in v1, Ferox needs:

* effect-aware signature checking
* effect propagation through blocks and control flow
* effect-aware diagnostics
* effect interaction rules with async and concurrency
* effect interaction rules with future language features

That cost is too high for the value it adds in v1.

## Recommended Replacement Direction

### Keep explicit async, but not through general effect sets

Recommended v1 direction:

* use dedicated async surface syntax rather than `![async]`
* keep `await` explicit
* keep concurrency rules separate from a general effect calculus

This likely means Ferox should prefer a simpler dedicated async form such as `async fn` or an equally narrow alternative, rather than making async one member of a general effect-set notation.

### Do not make IO a language-level checked effect in v1

Recommended v1 direction:

* IO remains explicit through APIs, modules, naming, and return types
* IO does not need a checked effect annotation in every signature

Reason:

* the value of checked `io` is lower than the cost of introducing a general effect system
* most of the practical explicitness gain can be preserved without turning IO into a core type-checking concern in v1

### Keep `unsafe` explicit through existing unsafe syntax

Recommended v1 direction:

* `unsafe` remains an explicit language boundary
* it should not also be modeled as a general effect kind in v1

### Do not model ordinary mutation as part of a v1 effect system

Recommended v1 direction:

* mutation remains visible through binding forms, borrowing forms, shared-mutation wrappers, and synchronization constructs
* do not add another layer of checked effect machinery on top of that

## Practical V1 Shape

If this recommendation is accepted, Ferox v1 should prefer:

* explicit `unsafe`
* explicit `Result` / `Option`
* explicit `await`
* explicit mutation syntax
* explicit synchronization wrappers

Ferox v1 should avoid:

* general effect sets on signatures
* effect propagation as a separate first-class checker subsystem
* presenting `io`, `async`, `unsafe`, and mutation as one unified effect language

## Consequences For The Spec

If adopted, this decision implies follow-up edits to:

* `spec/LANGUAGE.md`
  * remove or rewrite the `Effects Model` section
  * replace `![...]`-based examples
  * narrow the async section so it no longer depends on a general effect-set system
* `spec/IMPLEMENTATION.md`
  * remove effect checking as an early core checker obligation for v1
* `spec/VISION.md`
  * stop implying that explicit effect typing is part of the core promise

## Open Questions

This review does **not** yet settle:

* the exact async syntax to use if `![async]` is removed
* whether any minimal non-general effect marker should remain for a tiny subset of behaviors
* how much of the current concurrency design should survive once the effect system is removed

Those questions should be handled alongside `T063`.

## Current Recommendation

For the adversarial spec pass:

* treat the general effect-set model as a **defer** unless a much stronger argument appears
* simplify the language by removing this subsystem from the v1 semantic core
