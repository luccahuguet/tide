# Ferox vs Rust, Zig, and Go

This document explains Ferox's intended position relative to Rust, Zig, and Go.

Important constraint:

* Ferox is still a design-stage language
* these comparisons are about intended direction, not implemented reality
* Ferox should not claim superiority where the design has not yet earned it

How to use this document:

* this is a reasoning tool for Ferox's design, not marketing copy
* the point is to keep Ferox's identity narrow
* if a proposed Ferox feature makes these comparisons fuzzier instead of sharper, that is usually a warning sign

## Short Version

Ferox is trying to combine:

* Rust's seriousness about ownership and safety
* Zig's explicitness about cost
* Go's practical focus on backends, CLIs, and operationally maintainable software
* a stronger safe-change and tooling-first design bias than any of those is currently optimized for

The hard constraint is:

* Ferox only makes sense if it stays smaller and more coherent than the sum of those influences
* Ferox should be judged as a language for safer long-term evolution of codebases, not merely as a language for safe execution

## Internal Scorecard

These scores are for internal design reasoning only.

Rules:

* `1` = extremely low
* `5` to `6` = middle range
* `10` = extremely high
* these are not objective rankings
* higher is **not** always better across every column
* the Ferox row is a **target profile**, not implemented reality

Dimensions:

* `Safety`: strength of safety and correctness discipline in the language model
* `Cost Visibility`: how visible runtime cost and control behavior are in source
* `Domain Pragmatism`: fit for services, infrastructure, CLIs, and long-lived operational software
* `Tooling Regularity`: how friendly the surface is to parsing, formatting, refactoring, and machine reasoning
* `Semantic Breadth`: how much semantic machinery and expressiveness the language carries
* `Raw Avg`: simple mean of the five raw columns
* `Ferox Fit`: weighted closeness to the Ferox target profile

Interpretation:

* high `Semantic Breadth` is not automatically good for Ferox
* Ferox is intentionally trying to keep that column below Rust while staying high on safety, cost visibility, and tooling regularity
* `Raw Avg` is included only as a quick mechanical summary and should not drive decisions by itself
* `Ferox Fit` is the more meaningful summary number here

`Ferox Fit` method:

* per dimension, closeness score = `10 - abs(value - target)`
* weights:
  * `Safety` = `3`
  * `Cost Visibility` = `2`
  * `Domain Pragmatism` = `2`
  * `Tooling Regularity` = `3`
  * `Semantic Breadth` = `3`
* final score = weighted average of those closeness scores

Reason for the weights:

* Ferox is especially sensitive to safety, tooling regularity, and not exploding semantic breadth
* cost visibility and domain pragmatism are still important, but slightly less central to the core differentiator

| Language | Safety | Cost Visibility | Domain Pragmatism | Tooling Regularity | Semantic Breadth | Raw Avg | Ferox Fit |
|---|---:|---:|---:|---:|---:|---:|---:|
| Rust | 10 | 8 | 8 | 6 | 10 | 8.4 | 7.8 |
| Zig | 3 | 10 | 6 | 6 | 6 | 6.2 | 7.2 |
| Go | 5 | 4 | 10 | 8 | 4 | 6.2 | 6.7 |
| Ferox target | 10 | 10 | 8 | 10 | 6 | 8.8 | 10.0 |

What this scorecard is trying to enforce:

* Ferox should not chase Rust's semantic breadth
* Ferox should not give up Zig's cost visibility
* Ferox should not give up Go's domain pragmatism
* Ferox should push harder than all three on regularity for tooling, mechanically safer change, and agent-assisted work

## Ferox vs Rust

Rust is the closest reference point.

What Ferox wants from Rust:

* ownership
* deterministic memory management
* strong compile-time safety
* explicit runtime cost

What Ferox is trying to change:

* a smaller and more predictable ownership surface
* fewer advanced systems in the v1 core
* more restrictions rather than more overlapping escape hatches
* a surface that is easier for tools and agents to parse, transform, and reason about mechanically
* a language and toolchain better suited to safe automated change in long-lived codebases

Current honest tradeoff:

* Rust is more mature, more expressive, and more proven
* Ferox is only interesting if it becomes easier to reason about and safer to evolve without giving up too much of the value of ownership-based safety
* if Ferox drifts toward Rust-level semantic breadth, the project loses most of its reason to exist

If Ferox fails that test, users should just use Rust.

## Ferox vs Zig

Zig is the closest reference for explicitness and performance transparency.

What Ferox wants from Zig:

* visible costs
* low magic
* explicit behavior
* systems-language seriousness

What Ferox is trying to change:

* stronger safe-code guarantees
* more structured state and ownership reasoning
* a more analyzable and tooling-friendly surface
* less tolerance for ad hoc patterns that are hard to normalize for tools and refactors

Current honest tradeoff:

* Zig remains closer to the metal
* Ferox is only justified if it can provide stronger correctness and long-term maintenance structure without becoming bloated

Ferox should not try to beat Zig at raw low-level freedom.

## Ferox vs Go

Go is the main reference for simplicity, operational clarity, and backend pragmatism.

What Ferox wants from Go:

* practical focus
* strong standard tooling
* a small language core relative to its domain
* suitability for services, infrastructure, and CLIs

What Ferox is trying to change:

* no GC as a default requirement
* stronger compile-time guarantees
* more explicit performance behavior
* richer invalid-state modeling
* more compiler- and tooling-visible program structure
* stronger guarantees around safe change in high-trust codebases

Current honest tradeoff:

* Go is easier to teach and faster for broad team adoption
* Ferox is only justified when correctness pressure, performance transparency, and long-term codebase discipline matter enough to pay for ownership-based design

If the problem is ordinary low-risk service work, Go may remain the better tool.

## Where Ferox Should Be Careful

Ferox fails if it becomes:

* Rust-level complex without Rust-level payoff
* Zig-level manual without Zig-level control benefits
* Go-like simple while losing the ownership-based differentiation

That means the language needs a narrow identity:

* ownership and safety matter
* explicitness matters
* analyzability matters
* tooling and agent-friendliness matter
* safe change matters
* coherence matters more than feature breadth

## Current V1 Shape In Comparative Terms

The current narrowed v1 direction is roughly:

* Rust-like in taking ownership seriously
* deliberately less expressive than Rust in borrowing and trait-system ambition
* Zig-like in wanting visible costs and low magic
* Go-like in target-domain pragmatism rather than language-theory maximalism
* stricter than all three about wanting a regular, tool-friendly, machine-usable surface

This implies:

* Ferox should accept more restrictions if those restrictions buy better local reasoning
* Ferox should prefer one primary model plus explicit escape hatches
* Ferox should not add advanced systems just because Rust, Zig, or research languages can support them
* Ferox should treat fast feedback and machine-usable tooling as core parts of the language promise, not mere implementation details

If a feature proposal weakens those statements, it should face a very high bar.

## Practical Summary

Use Rust if you want:

* the strongest existing ownership ecosystem
* mature systems-language safety machinery
* maximum proven expressiveness in this space

Use Zig if you want:

* direct low-level control
* allocator-centric explicitness
* minimal language mediation

Use Go if you want:

* fast team adoption
* simple backend development
* a mature operational ecosystem

Use Ferox, if it succeeds, when you want:

* ownership-based safety
* explicit runtime behavior
* a smaller and more regular model than Rust
* stronger long-term analyzability than ad hoc systems code
* a language surface shaped for strong tooling and agent-assisted evolution
* safer automated and human-guided change in long-lived, high-trust systems
