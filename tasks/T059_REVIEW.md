# T059 Review Notes

Review date:

* March 8, 2026

Status:

* in progress
* this is the first adversarial pass
* focus so far: vision, irreversible semantics, effects, concurrency, and bootstrap realism

## Current Recommendation

Do **not** start full language implementation yet.

A narrow parser-and-formatter bootstrap could still be defensible, but the language-level semantics are not yet stable enough for confident type-checker or compiler work.

Current recommendation:

* keep reviewing before implementation of core semantics
* narrow or defer the highest-risk subsystems first
* only start deeper implementation once the semantic surface is smaller and more coherent

## Blocking Concerns

### 1. The effects model is too large and too central for v1

Files:

* `../spec/LANGUAGE.md` effects model, async model
* `../FEATURE_REVIEW.md` effects and cancellation recommendations

Problem:

* Ferox is already taking a hard language-design risk on ownership, borrowing, arenas, and explicit mutation
* the explicit effect-set model adds another major semantic subsystem on top of that
* `io`, `async`, cancellation, and structured concurrency are all defined in terms that depend on the effects model being coherent

Why this is dangerous:

* it creates a second "big bet" instead of one focused language core
* it increases checker complexity, signature complexity, and future feature interaction cost
* it weakens the claim that Ferox is meaningfully smaller than Rust

Recommended direction:

* defer the general effect-set system from v1
* keep `async` and recoverable error handling explicit through narrower dedicated mechanisms
* treat richer effect tracking as a later language evolution, not an initial semantic pillar

### 2. The ownership story is trying to do too many things at once

Files:

* `../spec/LANGUAGE.md` memory model, arenas, shared ownership, shared mutation

Problem:

Ferox currently combines:

* lexical borrowing
* explicit arenas as a language construct
* explicit shared ownership
* interior-mutation wrappers
* synchronized shared ownership

Why this is dangerous:

* this is already a sizable ownership sublanguage
* the language risks being "smaller than Rust" only in one axis while still being broad in total semantic surface
* users may end up learning multiple overlapping resource-management styles in v1

Recommended direction:

* choose one primary ownership path for early Ferox and keep the alternatives narrow
* if arenas remain in v1, keep them extremely local and minimal
* avoid expanding shared-ownership and interior-mutation ergonomics until the base ownership model proves usable

### 3. Trait receiver semantics are still under-specified

Files:

* `../spec/LANGUAGE.md` receiver syntax, borrowing model, mutation model

Problem:

* the spec defines traits, impls, and `self`
* but it explicitly defers receiver ownership and borrowing variants
* that leaves an important part of method semantics unresolved

Why this is dangerous:

* method dispatch and API design depend on whether `self` means move, shared borrow, mutable borrow, or multiple forms
* this interacts directly with ownership, mutation, trait ergonomics, and public API style
* implementing traits without a settled receiver model risks rework in parsing, AST shape, checking, and diagnostics

Recommended direction:

* settle the receiver model before implementation
* keep the initial receiver surface small, even if that means fewer method forms in v1

### 4. Async, structured concurrency, and cancellation are specified more as intention than as an executable model

Files:

* `../spec/LANGUAGE.md` async model, structured concurrency, cancellation semantics
* `../spec/IMPLEMENTATION.md` implementation order and bootstrap scope

Problem:

* the spec says there is no hidden global runtime
* async functions do not expose a future-like type in signatures
* task groups, child tasks, cancellation observation points, and parent-scope obligations are defined at a high level, but not as a fully grounded execution model

Why this is dangerous:

* runtime and compiler architecture decisions are downstream of these semantics
* cancellation and task completion rules become hard to test or implement if the execution model is still abstract
* this area can easily become a compiler-plus-runtime research project on its own

Recommended direction:

* narrow v1 concurrency heavily
* either define a much more concrete execution model or defer part of the structured concurrency and cancellation design
* do not let concurrency semantics remain aspirational if they are meant to guide implementation soon

### 5. The vision overstates the language's comparative confidence before the hard parts are resolved

Files:

* `../spec/VISION.md` strategic positioning and priority hierarchy
* `../COMPARISON.md` comparative claims against Rust, Zig, and Go

Problem:

* the spec claims Ferox will be simpler than Rust, more maintainable than Zig, and better suited for agent-assisted development than all three comparison languages
* at the same time, Ferox still has unresolved hard questions in ownership, effects, traits, async, and cancellation

Why this is dangerous:

* it encourages the spec to optimize for self-justifying ambition instead of hard tradeoff control
* it makes it harder to cut features, because the positioning language already assumes broad wins
* it blurs the line between intended direction and proven design

Recommended direction:

* make the vision more conditional and less comparative until the semantic core is smaller
* keep the public promise narrow: ownership-based, explicit, analyzable, and tooling-friendly

## Simplifications Worth Serious Consideration

* defer the general effect-set system from v1
* narrow structured concurrency to the smallest model that can be implemented coherently
* defer advanced cancellation semantics until the async/runtime model is more concrete
* settle method receiver forms before deeper trait work
* keep arenas only if they remain genuinely small and local
* treat "smaller than Rust" as a constraint to enforce by cuts, not a slogan

## Current Go / No-Go Recommendation

Current recommendation:

* do not start full semantic implementation yet
* acceptable early work:
  * syntax
  * parsing
  * formatting
  * diagnostics infrastructure
* not yet recommended:
  * full type checker
  * ownership checker
  * effect checker
  * async runtime or concurrency implementation

## Next Review Pass

Next pass should focus on:

* method receiver model
* exact trait/generic complexity budget
* whether arenas belong in v1 at all
* whether effect sets should be removed from the initial language plan
