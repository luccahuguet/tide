# T066 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed

## Decision

Current recommendation:

* **adapt Bosque-style regularization for Ferox**

But the adoption boundary matters:

* **yes** for compiler-internal IR and tooling architecture
* **yes** for source-language regularity as a design discipline
* **no** for importing Bosque's more radical source-language restrictions into Ferox v1

## Why Bosque Matters

Bosque made the shortlist because it treats regularization and machine reasoning as first-class design goals instead of accidental side effects.

That aligns with several Ferox goals that already exist:

* smaller semantic surface
* regular syntax and predictable ASTs
* machine-friendly tooling
* long-term analyzability

The strongest Bosque lesson is not "copy this language."

It is:

* aggressively reduce irregular internal forms
* make the compiler reason over a smaller set of normalized semantic shapes
* treat analysis and tooling as primary consumers of the language design

## What Ferox Should Borrow

### 1. A regularized checked IR

Ferox should adopt the idea that serious semantic analysis and later backend work should operate on a normalized intermediate representation rather than directly on every surface-language shape.

Recommended direction:

* lower parsed Ferox syntax into a checked IR with fewer semantic forms
* make ownership-relevant operations explicit in that IR
* make async boundaries and structured control-flow lowering explicit in that IR
* keep the IR stable enough to support diagnostics, refactors, and later optimization work

This is the strongest Bosque fit for Ferox.

### 2. Regularization as a language-design filter

Bosque is a useful reminder that regularity is a real design asset, not merely an aesthetic preference.

For Ferox this should mean:

* avoid multiple near-equivalent surface constructs
* prefer one dominant form for each core operation
* prefer lowering-friendly source constructs
* keep AST shape predictable enough for machine tooling and later automated reasoning

This reinforces Ferox's existing coherence rule.

### 3. Tooling-first semantics

Bosque treats automated reasoning as a serious use case.

Ferox should keep that mindset, but apply it pragmatically:

* diagnostics should be derived from structured semantic data
* refactoring and repair tools should depend on normalized compiler representations
* later static analyses should build on explicit intermediate forms rather than ad hoc AST walks

## What Ferox Should Not Borrow Directly

### 1. Bosque's more radical source-language restrictions

Bosque's loop-free and more strongly regularized source-level direction is interesting, but it is not a good fit for Ferox's intended identity.

Ferox is still trying to be:

* ownership-based
* systems-aware
* useful for backends, CLIs, infrastructure, and long-lived services

That means Ferox should not distort its source language just to imitate Bosque's strongest restrictions.

### 2. Bosque as a direct semantic template

Ferox should not try to become "Bosque with ownership."

Reasons:

* Bosque is optimizing for a different language personality
* Ferox already has a harder semantic bet in ownership and borrowing
* importing Bosque-style source constraints directly would increase the chance of building a language that is principled but impractical

### 3. Large v1 source-language expansion in the name of reasoning

If a Bosque-inspired idea requires a broader v1 surface or a second major checker subsystem, it should be rejected or deferred.

Regularization should simplify Ferox, not widen it.

## Best Adoption Boundary

### Source language

Adopt only as a discipline:

* small grammar
* predictable AST
* regular constructs
* fewer overlapping features

Do **not** adopt Bosque-style radical source constraints as part of Ferox v1.

### IR and compiler internals

This is the main adoption target.

Ferox should strongly prefer:

* a checked IR after parsing and semantic analysis
* explicit lowered forms for ownership, control flow, and async structure
* normalized representations that are easier to analyze than raw syntax

### Tooling

Adopt the general lesson:

* build diagnostics, refactors, and later reasoning tools on normalized compiler data rather than on loosely interpreted syntax trees

## Why This Helps Ferox

This direction strengthens Ferox without widening the language core:

* improves analyzability
* improves compiler architecture
* improves future machine tooling
* avoids adding another user-facing semantic subsystem

That is exactly the kind of external idea Ferox should prefer.

## Consequences For Ferox

Current recommendation:

* treat Bosque influence as an **implementation and tooling architecture decision**, not as a v2 or v3 language-feature commitment

Practical implication:

* `spec/IMPLEMENTATION.md` should explicitly prefer a regularized checked IR
* `spec/LANGUAGE.md` should continue using regularity and coherence as design filters, but should not import Bosque-like source restrictions casually

## Primary Sources

* https://www.microsoft.com/en-us/research/project/bosque-programming-language/
* https://www.microsoft.com/en-us/research/publication/regularized-programming-with-the-bosque-language/

## Current Recommendation

For Ferox:

* **adapt** Bosque-style regularization
* adopt it primarily in **IR and tooling**
* keep only the lighter regularity discipline at the **source-language** level
* reject direct import of Bosque's more radical source-language constraints into Ferox v1
