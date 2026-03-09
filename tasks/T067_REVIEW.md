# T067 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed

## Decision

Current recommendation:

* **adapt Perceus-style reuse ideas only as later backend and optimization research**

Do **not** make them part of:

* the Ferox source-language surface
* the core v1 semantic model
* the early Rust bootstrap obligations

## Why Perceus Matters

Perceus made the shortlist because it aims at a strong result:

* improve runtime behavior
* eliminate unnecessary allocation churn
* recover in-place update opportunities

And it tries to do that without forcing a bigger surface language.

That makes it attractive for Ferox in principle.

## Why The Fit Is Partial

Perceus comes from a different baseline than Ferox.

It is especially compelling in settings where:

* functional update style is dominant
* reference counting is a central implementation technique
* reuse analysis can recover in-place behavior that the source language does not expose directly

Ferox is different:

* Ferox already has explicit ownership and moves
* Ferox already plans explicit mutable access through `&mut`
* Ferox does not currently define itself around reference counting as the primary runtime model

So the right lesson is not "make Ferox into a Perceus-style language runtime."

The right lesson is:

* use reuse and uniqueness analysis selectively where it improves backend quality without changing the language contract

## What Ferox Should Borrow

### 1. Reuse as an optimization target, not a language feature

Ferox should be willing to recover in-place updates or storage reuse when the compiler can prove that doing so preserves the safe-language semantics.

Good targets later:

* short-lived immutable-looking reconstruction patterns
* ADT transformation pipelines
* temporary aggregates that are proven unique

This belongs in backend and lowering strategy, not in user-facing syntax.

### 2. Ownership-aware optimization opportunities

Ferox's ownership model may actually make some reuse opportunities easier to justify than in a less ownership-aware language.

In particular:

* moves
* uniqueness after consumption
* explicit non-aliasing paths

may provide strong signals for backend reuse analysis.

That is the most promising Ferox-specific angle.

### 3. Keep the source contract simple while allowing stronger internals later

Perceus is a good reminder that implementation sophistication does not always need a matching surface-language feature.

For Ferox, this means:

* keep v1 semantics simple
* leave room for backend reuse wins later
* do not promise source-level constructs just to expose those wins early

## What Ferox Should Not Borrow Directly

### 1. Reference counting as Ferox's defining runtime story

Perceus is tightly connected to precise reference counting and reuse.

Ferox should not adopt "reference-counting-centric runtime" as a language-level commitment from this research pass.

That would create new questions around:

* runtime cost predictability
* interoperability with the current ownership story
* whether deterministic destruction remains conceptually simple

### 2. Early compiler complexity

Ferox is not ready to make advanced reuse analysis part of the bootstrap critical path.

Reasons:

* the language core has only recently been narrowed
* the checked IR is not implemented yet
* the first backend is intentionally supposed to favor simplicity and bring-up speed

Perceus-style optimization is exactly the kind of thing that should come after the compiler is already credible.

### 3. Source-level semantics that depend on backend cleverness

Ferox should not promise users that certain updates are cheap because the compiler "will probably reuse memory."

That would cut directly against the project's explicitness goals.

If reuse happens, it should be:

* a backend optimization
* observationally invisible
* never required for ordinary source-level reasoning

## Best Adoption Boundary

### Source language

Reject as a direct feature source.

Ferox does not need new syntax, types, or user-visible memory modes for this idea.

### V1 bootstrap implementation

Defer.

The first Rust implementation should prioritize:

* correct parsing
* formatting
* diagnostics
* semantic checking
* a working native backend path

It should not depend on advanced reuse analysis to validate the language.

### Later backend work

This is the correct home.

Once Ferox has:

* a stable checked IR
* working ownership and async analysis
* a credible first native backend
* profiling data showing real allocation pressure

then Perceus-style reuse becomes a strong candidate for backend experimentation.

## Why This Helps Ferox

This recommendation preserves the upside without breaking the roadmap:

* possible long-term runtime wins
* no widening of the language core
* no early bootstrap complexity spike
* strong alignment with the "implementation sophistication without source bloat" rule

## Consequences For Ferox

Current recommendation:

* treat Perceus-style reuse as a **later backend optimization research track**
* do **not** treat it as a v2 language-feature commitment

Practical implication:

* `spec/IMPLEMENTATION.md` should mention reuse analysis only as a later optimization possibility after the first backend is already credible

## Primary Sources

* https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/
* https://www.microsoft.com/en-us/research/publication/reference-counting-with-frame-limited-reuse/

## Current Recommendation

For Ferox:

* **adapt** the idea only as later backend research
* reject source-level adoption
* defer implementation work until after the checked IR and first native backend are stable
