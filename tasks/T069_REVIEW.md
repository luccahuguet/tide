# T069 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed

## Decision

Current recommendation:

* **adapt selective concurrency lessons from Pony and Verona**

But with a strict boundary:

* **yes** to isolation, ownership transfer, and explicit sharing as design principles
* **no** to importing a full capability lattice into Ferox

## Why These Sources Matter

The primary sources converge on a serious point:

* shared mutable data is the hard part of concurrency
* immutable sharing is safe
* isolated ownership transfer is safe
* compiler-enforced discipline matters if you want safe parallelism without relying on ad hoc locking

That fits Ferox well.

Ferox already wants:

* explicit ownership
* explicit sharing
* safe concurrency by default
* a smaller mental model than Rust plus extra subsystems

## What Ferox Should Borrow

### 1. Isolation and transfer are better defaults than shared mutable concurrency

This is the strongest lesson.

Ferox should continue preferring concurrency built around:

* values moved into child work
* immutable data safely shared
* synchronized shared mutation only when explicitly requested

That is already directionally consistent with the current `async let` model and the explicit `SyncShared[T]` boundary.

### 2. Concurrency boundaries should make sharing mode obvious

Ferox should aim for a future where it is visually and semantically obvious which of these is happening:

* ownership transfer into concurrent work
* immutable sharing
* synchronized shared mutation

Those cases should not collapse into one generic "it is all just references" model.

### 3. If concurrency grows later, it should grow through ownership-aware rules first

If Ferox adds richer concurrency after v1, the first expansion should favor:

* clearer ownership-transfer rules across task boundaries
* clearer isolation guarantees for values sent into concurrent work
* explicit synchronized-sharing APIs when data truly must be shared

That is much more Ferox-like than importing a capability matrix.

## What Ferox Should Not Borrow Directly

### 1. Pony's full reference-capability system

Reject for Ferox.

Reasons:

* it is powerful but conceptually large
* it would turn concurrency into another major type-system subsystem
* it conflicts with the current Ferox direction of keeping one dominant ownership model plus explicit escape hatches

### 2. Verona's broader research agenda as a v1 source-language plan

Reject for v1.

The Verona questions around:

* no concurrent mutation
* linear regions
* compartmentalisation-oriented regions

are interesting, but they are far beyond what Ferox should absorb into its first concurrency wave.

### 3. Capability-heavy concurrency as the next obvious step

Reject.

Ferox should not assume that "better concurrency" means "more qualifiers and richer capability algebra."

The first post-v1 step should be smaller than that.

## Best Adoption Boundary

### V1 core

Keep the current small direction:

* `async let`
* explicit `await`
* explicit synchronized sharing
* no general background-task spawning

Do not widen v1 with capability machinery.

### Post-v1 concurrency growth

This is the real adoption target.

Possible later Ferox work:

* ownership-aware child-task transfer rules
* clearer isolation expectations for values crossing concurrency boundaries
* richer structured concurrency that still preserves explicit sharing modes

### Long-horizon research

Full capability-heavy concurrency models should stay on the long-horizon track unless real Ferox usage proves a much stronger need.

## Why This Helps Ferox

This keeps the strongest lessons while preserving the narrower core:

* concurrency remains explicit
* shared mutation remains second-class and visible
* ownership keeps doing the main conceptual work
* Ferox avoids importing another major semantic lattice

## Roadmap Implication

Current recommendation:

* a small ownership-aware concurrency refinement is a plausible **v2 candidate**
* full capability-heavy concurrency remains a **v3 / research-only** candidate

## Primary Sources

* https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html
* https://www.ponylang.io/learn/reference-capabilities/
* https://www.microsoft.com/en-us/research/project/project-verona/
* https://www.microsoft.com/en-us/research/publication/uniqueness-and-reference-immutability-for-safe-parallelism/

## Current Recommendation

For Ferox:

* **adapt** the isolation and ownership-transfer lessons
* reject a full capability lattice
* keep v1 small
* grow later concurrency, if needed, through explicit transfer and sharing rules instead of capability-heavy design
