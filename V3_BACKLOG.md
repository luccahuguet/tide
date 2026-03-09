# Ferox V3 Backlog

This document tracks **long-horizon candidates** for Ferox.

Important constraint:

* these are research-grade or high-risk ideas
* they should not shape the v1 core
* they should not be pulled forward without strong evidence from real Ferox usage

## Admission Rule

An item belongs in the v3 backlog when it:

* would materially widen the language core
* adds a major semantic subsystem
* depends on real-world pressure that Ferox does not yet have
* may still turn out to be unnecessary even after v2

## Current Candidates

### General effect sets

Candidate scope:

* signature-level effect tracking
* effect propagation through blocks and control flow

Why deferred this far:

* it is a major checker subsystem
* it overlaps with other explicit mechanisms already present in Ferox
* it should not be reconsidered before real Ferox usage proves that `async`, `unsafe`, `Result`, and explicit APIs are not enough

### Effect handlers or richer effect systems

Candidate scope:

* handlers
* more expressive effect composition
* broader effect abstraction mechanisms

Why deferred this far:

* this is attractive, but much too ambitious for the current language stage
* it should stay on the research track unless Ferox later demonstrates a concrete problem that only richer effect abstraction can solve

### Capability-heavy concurrency or sharing models

Candidate scope:

* ideas inspired by Pony or Verona that require a richer capability discipline

Why deferred this far:

* Ferox should first prove a smaller ownership and concurrency model

### Broader region or lifetime-management sublanguages

Candidate scope:

* richer region systems
* more abstract lifetime-management mechanisms beyond a narrow ownership core

Why deferred this far:

* Ferox should avoid multiple overlapping memory models unless later evidence justifies them

### Major type-system expansion

Candidate scope:

* blanket implementations
* specialization
* negative implementations
* const generics
* variance annotations

Why deferred this far:

* Ferox does not need another major complexity wave until the base language proves itself

### Major backend or compiler-strategy expansion

Candidate scope:

* broader backend plurality
* more aggressive whole-program reasoning machinery
* deep IR or optimization changes driven by external research ideas

Why deferred this far:

* these may be valuable, but they should be justified by real compiler maturity, not early design temptation

## Research Connection

Some v3 candidates are already being researched through:

* `T065` to `T070` in [TASKS.md](./TASKS.md)

Those research tasks should decide whether an idea eventually deserves a place in the language roadmap at all.
