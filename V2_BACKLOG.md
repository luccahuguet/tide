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

## Current Candidates

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

Those belong in [V3_BACKLOG.md](./V3_BACKLOG.md) unless later evidence strongly pulls them forward.
