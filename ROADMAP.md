# Ferox Roadmap

This document tracks the intended Ferox version path.

Important constraint:

* this is a planning document, not a promise
* items may move between versions or be dropped entirely
* v1 coherence is a higher priority than preserving any particular deferred feature

## Current Position

Ferox is still in the design stage.

Current work:

* first adversarial narrowing pass completed
* version backlog and external-ideas research completed for the first pass
* preparing the architecture-first Rust bootstrap against the narrowed v1

Current rule:

* do not start full semantic implementation until the v1 core is smaller and more coherent

## Version Rule

Ferox should separate:

* language versions
* implementation phases

Important clarification:

* rewriting parts of the toolchain in Ferox is **not by itself** a v2 language event
* the Rust bootstrap, later Ferox rewrites, and self-hosting transition are part of the implementation path around the v1 language era

## V1: Safe Core

Goal:

* prove a coherent, narrow Ferox safe core

Target shape:

* ownership-based
* lexical borrowing
* explicit mutation
* lower semantic entropy than Rust
* narrow trait and generic surface
* ADTs and exhaustive matching
* explicit `Result` / `Option`
* expression-oriented, FP-shaped readability without FP-heavy machinery
* explicit async, but with a small concurrency model
* fast incremental `check` and machine-readable tooling as first-class implementation goals
* no general effect-set system in the v1 core

Implementation path:

1. freeze the narrow v1 spec
2. build the first toolchain in Rust
3. bring up parser, full-fidelity syntax, formatter, diagnostics, semantic checking, and compiler stages incrementally
4. begin rewriting selected components in Ferox only after the Rust bootstrap is stable enough

V1 success means:

* the language core is coherent
* the Rust implementation is usable
* `ferox check` proves fast and architecture-friendly enough for day-to-day feedback
* diagnostics and machine-readable outputs are credible enough to support editors, CI, and agent workflows
* Ferox can begin carrying selected parts of its own toolchain without destabilizing the project

## V2: Safe Change

Goal:

* strengthen safe change only after v1 proves real pressure for it

V2 is the first place where Ferox should consider carefully expanding the language and tooling again.

Default rule:

* no v2 feature should enter just because it is elegant or interesting
* v2 additions should be justified by real implementation pain, user ergonomics, or clear safe-change leverage

Current likely direction:

* stronger structural tooling and refactor support
* ownership-aware concurrency refinement
* protocol-state assistance where justified by real APIs
* an initial contract or spec layer if real Ferox usage proves the need

Current backlog:

* [V2_BACKLOG.md](./V2_BACKLOG.md)

## V3: Deep Assurance

Goal:

* consider deeper assurance and other high-risk ideas only after Ferox has real language and toolchain maturity

V3 is the place for ideas that are:

* powerful but expensive
* attractive but still speculative for Ferox
* likely to widen the semantic surface substantially

Current likely direction:

* richer verification
* deeper analysis machinery
* higher-cost concurrency research
* other major semantic expansions only if real usage proves they are necessary

Current backlog:

* [V3_BACKLOG.md](./V3_BACKLOG.md)

## Research Track

Some ideas may influence Ferox without becoming language-version features directly.

Examples:

* IR design
* optimization strategy
* diagnostic architecture
* compiler-internal reasoning models

These should be researched separately before they are added to any version backlog.

Completed research tasks:

* the completed `T065` to `T070` analysis set in `tasks/`
