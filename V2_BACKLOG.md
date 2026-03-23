# Ferox V2 Backlog

This document tracks current **post-v1 candidates**.

Important constraint:

* these are candidates, not commitments
* nothing here is guaranteed to land in v2
* if v1 does not demonstrate clear need, an item should stay deferred or be dropped

## Admission Rule

An item belongs in the v2 backlog only if:

* it is plausibly useful after a stable v1 exists
* it does not obviously explode the language core
* it solves a real problem rather than adding a parallel model for the same job
* it strengthens Ferox's safe-change story more than it increases entropy

## Current Candidates

These candidates are the main ways Ferox may strengthen **safe change** after proving the v1 safe core.

They should still be pulled in only when real usage creates pressure for them.

### Structural rule and autofix system

Candidate scope:

* first-party structural lint rules
* machine-safe autofix and codemod support
* policy enforcement and large-scale migration tooling

Why deferred:

* the parser, formatter, diagnostics, and normalized compiler representations need to exist first
* the language must prove its syntax and semantic regularity before a rule system can be trusted

### Contract or spec layer

Candidate scope:

* preconditions
* postconditions
* invariants
* ghost or spec code in a narrow form

Why deferred:

* v1 should prove the safe core before adding a second assurance layer
* Ferox should stay verification-ready without becoming proof-first too early

### Structured concurrency beyond `async let`

Candidate scope:

* dynamic task groups or similar fan-out constructs

Why deferred:

* the current v1 async/concurrency model needs to stay much smaller

### Richer but still bounded cancellation semantics

Candidate scope:

* clearer parent/child cancellation behavior
* more concrete cancellation-completion rules

Why deferred:

* current cancellation design is too broad for v1

### Ownership-aware concurrency refinement

Candidate scope:

* clearer ownership-transfer rules across task boundaries
* clearer distinction between moved values, immutable sharing, and synchronized sharing in concurrent code

Why deferred:

* v1 should first prove the much smaller `async let` model
* Ferox should not jump from a tiny concurrency core straight into capability-heavy design
* the first concurrency expansion should stay ownership-centered and explicit

### Arenas or regions, if real v1 pressure justifies them

Candidate scope:

* explicit grouped-lifetime constructs

Why deferred:

* they create a second lifetime-management model and should not be part of v1 unless proven essential

### More granular visibility

Candidate scope:

* package-scoped visibility
* module-subtree visibility

Why deferred:

* `pub` and private are enough for the initial model

### Narrow public-type/private-construction mechanism

Candidate scope:

* one explicit way to expose a nominal type publicly while restricting external construction from outside the defining module
* smart-constructor-friendly APIs for invariant-sensitive modules
* ordinary public functions or methods for validated creation and observation

Preferred shape:

* external code cannot directly construct the hidden representation
* external code should also not rely on a special match-only/private-constructor middle ground in the initial design of this feature
* the feature should behave like full external opacity with explicit API access, not like a broad constructor-visibility lattice

Why deferred:

* v1 should keep visibility binary and avoid a broader constructor/export lattice
* if Ferox adopts this later, it should be one narrow invariant-preserving tool rather than a large new access-control system
* the language should first prove that ordinary module boundaries and explicit constructor functions are insufficient in practice

Why this still looks promising:

* it can make invalid-state prevention materially stronger at module boundaries
* it fits Ferox's explicit API and low-entropy goals better than a richer privacy lattice
* it supports smart-constructor patterns without requiring proof-oriented language features

### Trait-system growth, if v1 proves the need

Candidate scope:

* associated types
* carefully chosen trait-surface refinements

Why deferred:

* the initial trait model should stay small and analyzable

### Narrow protocol-state checking, if real APIs justify it

Candidate scope:

* library-encoded state transitions
* opt-in protocol-state lints or analysis for well-scoped APIs

Why deferred:

* v1 should rely on ordinary ownership, types, and consuming methods first
* this is only worth adding if real Ferox libraries show repeated protocol-state pressure
* Ferox should avoid a broad built-in typestate subsystem unless the smaller approach clearly fails

### Richer machine-readable compiler metadata

Candidate scope:

* expanded symbol graphs
* richer refactor and analysis hooks
* more stable machine-facing compiler services

Why deferred:

* v1 should first prove the basic diagnostics and tooling contract
* wider machine interfaces should grow from real tooling pressure, not from speculation

### Generic-surface growth, if v1 proves the need

Candidate scope:

* `where` clauses
* default type parameters
* other moderate generic refinements

Why deferred:

* the v1 generic system should remain tight and local

### Package-system growth

Candidate scope:

* target-specific dependency sections
* feature flags
* build scripts
* alternate registries
* per-dependency source overrides

Why deferred:

* the v1 package model should stay stable and small before it becomes flexible

## Not Automatically V2

The following are **not** assumed to be v2 items by default:

* general effect sets
* effect handlers
* capability-heavy concurrency models
* major semantic expansions that would widen the language core substantially
* proof-first or theorem-prover-style language features

Those belong in [V3_BACKLOG.md](./V3_BACKLOG.md) unless later evidence strongly pulls them forward.
