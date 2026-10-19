# Ferox V1 Direction

This document summarizes the current narrowed Ferox v1 direction after the adversarial spec review and first external-ideas research pass.

Use this as the shortest design summary before reading the full spec and review notes.

Important constraint:

* this is the current direction, not an unchangeable promise
* v1 coherence is more important than preserving any specific deferred idea

## Current Rule

Ferox v1 should stay small enough that implementation does not immediately force a redesign.

That means:

* one primary model where possible
* explicit escape hatches instead of overlapping subsystems
* defer ambitious ideas unless real pressure justifies them
* treat lower semantic entropy and fast feedback as product constraints, not afterthoughts

## Current V1 Core

Ferox v1 currently means:

* ownership-based by default
* simplified lexical borrowing
* explicit mutation
* moves by default for non-copy values
* narrow copy-safe model
* shared ownership only as an explicit secondary mechanism
* ADTs and exhaustive matching
* expression-oriented, FP-shaped readability without FP-heavy machinery
* narrow traits and generics
* `Option`, `Result`, and `?`
* explicit `async fn`, `await`, and a small `async let` concurrency model
* explicit `unsafe`
* regular syntax, canonical formatting, and machine-usable tooling
* fast incremental `check` and credible machine-readable diagnostics as explicit implementation goals
* cross-task data flow by move, copy-safe duplication, or explicit `SyncShared[...]`, not by ordinary borrow capture

## FP Profile

Ferox should be FP-shaped in source form, not FP-heavy in semantic machinery.

Core FP choices for v1:

* immutable-by-default bindings and values unless mutation is made explicit
* ADTs and exhaustive `match` as the normal invalid-state and branching tools
* expression-oriented code with small transformation-focused helper functions where that improves readability

Public-data modeling preference:

* prefer explicit ADTs, `Option`, and `Result` over flag combinations, sentinel values, or loosely encoded state conventions in public APIs

Clarification:

* Ferox v1 does not treat purity here as a checked effect contract; this is a style and API-shaping preference, not a second semantic system

Secondary FP choices:

* focused pattern-control forms are useful when they remain obvious shorthands rather than a second control-flow family

Rejected FP directions for v1:

* lazy defaults
* purity/effect machinery as a parallel semantic system
* monadic ceremony as the expected everyday style
* higher-kinded or research-heavy type abstraction
* broad operator culture

The target feel is:

* transformation-friendly
* explicit
* regular
* easy to lower, format, and rewrite
* strongly typed at API boundaries in ways that expose meaningful state directly

not:

* abstraction-dense
* operator-driven
* type-level
* semantically clever

## Concrete Source Personality

Everyday Ferox should feel like straightforward, strongly typed systems code with a regular expression-oriented shape.

Core style rules:

* use `if` for ordinary boolean conditions and `match` for typed state or mode changes
* prefer named intermediate values once a flow changes representation, can fail, or would become awkward as nested calls
* keep helper functions small when they clarify a transformation boundary, not because tiny combinators are fashionable
* keep mutation local and visually obvious; return to immutable values at API boundaries
* keep public APIs honest with explicit enums, `Option`, and `Result`
* favor modules centered on one domain concept or capability rather than wide grab-bags of unrelated helpers

Canonical examples:

```ferox
fn classify(user: User) -> Status {
    match user.role {
        admin => active,
        guest => limited,
        suspended => blocked,
    }
}

fn load_profile(id: UserId) -> Result[Profile, LoadError] {
    let raw = read_profile(id)?;
    let parsed = parse_profile(raw)?;
    ok(parsed)
}

fn display_name(user: Option[User]) -> String {
    match user {
        some(value) => value.name,
        none => "guest",
    }
}

fn increment(current: Int) -> Int {
    let next = {
        let mut value = current;
        value = value + 1;
        value
    };
    next
}
```

Discouraged everyday moves:

* deep nesting of small calls when named steps would make representation changes clearer
* public APIs that encode state with boolean flags, sentinel integers, or loosely related primitive parameters
* broad mutation-oriented call chains when an owned value can be transformed and returned more directly
* control flow that depends on clever pattern features, operator tricks, or abstraction stacks to express ordinary branching
* modules that mix unrelated domain concepts simply because the code is short

## Decisions Already Made

The current v1 direction includes these explicit cuts:

* no general effect-set system in v1
* no effect handlers or richer effect systems in v1
* no first-class arenas or broader region language in the v1 core
* no broad typestate subsystem in v1
* no capability-heavy concurrency system in v1
* no large structured-concurrency or cancellation model in v1
* no broad constructor-visibility or export-control lattice in v1

The current v1 direction includes these explicit simplifications:

* borrowing is lexical and intentionally more restrictive than Rust
* method receivers are only `self`, `&self`, and `&mut self`
* traits stay small
* patterns stay small, explicit, and shape-driven
* shared ownership and synchronization are explicit and secondary
* concurrency stays bounded and visible
* ordinary borrows do not cross task boundaries by default
* module visibility stays binary in v1 unless a later narrow invariant-preserving mechanism clearly earns its keep

## What Ferox Should Borrow From Research

Current external-ideas conclusions:

* Bosque-style regularization is a strong fit for compiler IR and tooling architecture
* Perceus-style storage reuse is promising only as a later backend optimization direction; it should not shape the v1 language surface or become a semantic promise to users
* typestate ideas are promising only in narrow library-encoded forms, with possible later opt-in analysis
* Pony and Verona are useful mainly for isolation and ownership-transfer lessons, not for a capability lattice
* Koka / Eff-style richer effect systems remain long-horizon research only
* contracts and proof-oriented tooling are promising only as later optional assurance layers, not as the core v1 language identity
* Ferox should not adopt a general built-in ability system in v1; `copy` remains special, while `drop`, `send`, and `share` stay out unless later language pressure proves they are truly simpler than the existing ownership and concurrency rules
* opaque public types with private constructors and explicit smart constructors look promising if they strengthen invariants without adding major semantic machinery, but they should be treated as a narrow post-v1 feature rather than part of the v1 core

## V2 Candidates

The most plausible post-v1 growth areas are:

* structural rule and autofix tooling
* contracts or a narrow spec layer
* structured concurrency beyond `async let`
* richer but still bounded cancellation semantics
* ownership-aware concurrency refinement
* one narrow fully opaque public-type/private-construction mechanism if invariant pressure proves it worthwhile
* arenas or regions only if real pressure justifies them
* narrow protocol-state checking only if real APIs justify it
* carefully limited trait and generic growth

See:

* [V2_BACKLOG.md](./V2_BACKLOG.md)

## V3 Or Research-Only Candidates

These stay out of the near-term plan:

* general effect sets
* effect handlers and richer effect systems
* capability-heavy concurrency or sharing models
* broader region or lifetime-management sublanguages
* major type-system expansion
* major backend or compiler-strategy expansion driven by speculative research

See:

* [V3_BACKLOG.md](./V3_BACKLOG.md)

## Implementation Consequences

The implementation direction now assumes:

* Rust bootstrap first
* frontend and tooling credibility before backend ambition
* fast feedback as a first-class implementation goal
* a regularized checked IR
* later optimization work only after the checked IR and first backend are credible
* tool-facing compiler data should be treated as part of the architecture, not as an afterthought
* the bootstrap should accumulate a semantic corpus and golden tests early enough to stabilize parser, formatter, diagnostics, and ownership behavior

This means Ferox should not start implementation as if:

* effects are still part of the v1 checker
* arenas are still core
* advanced concurrency machinery is still expected in the first compiler

## Current Implementation Verdict

Current recommendation:

* start implementation now
* keep the start narrow
* treat this as a Rust bootstrap and architecture-validation phase, not as permission to front-load every semantic subsystem at once
* validate the safe-core and safe-change thesis through real tooling, not just through spec wording

The current implementation-readiness verdict is recorded in:

* [tasks/T071_REVIEW.md](./tasks/T071_REVIEW.md)

## Review Trail

The main review notes behind this summary are:

* [tasks/T059_REVIEW.md](./tasks/T059_REVIEW.md)
* [tasks/T060_REVIEW.md](./tasks/T060_REVIEW.md)
* [tasks/T061_REVIEW.md](./tasks/T061_REVIEW.md)
* [tasks/T062_REVIEW.md](./tasks/T062_REVIEW.md)
* [tasks/T063_REVIEW.md](./tasks/T063_REVIEW.md)
* [tasks/T064_REVIEW.md](./tasks/T064_REVIEW.md)
* [tasks/T065_REVIEW.md](./tasks/T065_REVIEW.md)
* [tasks/T066_REVIEW.md](./tasks/T066_REVIEW.md)
* [tasks/T067_REVIEW.md](./tasks/T067_REVIEW.md)
* [tasks/T068_REVIEW.md](./tasks/T068_REVIEW.md)
* [tasks/T069_REVIEW.md](./tasks/T069_REVIEW.md)
* [tasks/T070_REVIEW.md](./tasks/T070_REVIEW.md)
