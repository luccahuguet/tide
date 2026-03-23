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
* a likely bias toward one narrow pipe operator if it improves readability without adding operator sprawl

## Decisions Already Made

The current v1 direction includes these explicit cuts:

* no general effect-set system in v1
* no effect handlers or richer effect systems in v1
* no first-class arenas or broader region language in the v1 core
* no broad typestate subsystem in v1
* no capability-heavy concurrency system in v1
* no large structured-concurrency or cancellation model in v1

The current v1 direction includes these explicit simplifications:

* borrowing is lexical and intentionally more restrictive than Rust
* method receivers are only `self`, `&self`, and `&mut self`
* traits stay small
* shared ownership and synchronization are explicit and secondary
* concurrency stays bounded and visible

## What Ferox Should Borrow From Research

Current external-ideas conclusions:

* Bosque-style regularization is a strong fit for compiler IR and tooling architecture
* Perceus-style reuse is promising only as later backend optimization work
* typestate ideas are promising only in narrow library-encoded forms, with possible later opt-in analysis
* Pony and Verona are useful mainly for isolation and ownership-transfer lessons, not for a capability lattice
* Koka / Eff-style richer effect systems remain long-horizon research only
* contracts and proof-oriented tooling are promising only as later optional assurance layers, not as the core v1 language identity
* a small built-in ability model may be worth exploring if it reduces trait and marker complexity without widening the language

## V2 Candidates

The most plausible post-v1 growth areas are:

* structural rule and autofix tooling
* contracts or a narrow spec layer
* structured concurrency beyond `async let`
* richer but still bounded cancellation semantics
* ownership-aware concurrency refinement
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
