# Orbis vs Rust, Zig, and Go

This document explains Orbis's intended position relative to Rust, Zig, and Go.

Important constraint:

* Orbis is still a design-stage language
* some comparisons are about intended direction, not implemented reality
* if the spec changes, this document should change with it

## Short Version

Orbis is aiming for this space:

* safer than Go
* simpler than Rust
* more structured and analyzable than Zig
* more machine-friendly than all three

## Comparison Table

Score meaning:

* `1` = very weak on that dimension
* `10` = very strong on that dimension

These scores are directional, not scientific.

| Dimension | Orbis | Rust | Zig | Go |
|---|---|---|---|---|
| Safety model | `9/10` Safe by default with ownership and simplified borrowing | `10/10` Safe by default with full borrow checker | `5/10` Lower-level manual control with fewer default safety guarantees | `6/10` GC-based safety with fewer compile-time correctness guarantees |
| Memory management | `9/10` Deterministic, no GC, ownership plus lexical borrowing plus arenas/regions | `10/10` Deterministic, no GC, ownership plus borrowing | `9/10` Manual allocators and explicit memory management | `4/10` Garbage collection |
| Borrowing power | `7/10` Intentionally smaller than Rust, but still ownership-based | `10/10` Highest expressiveness and maturity in this group | `1/10` No borrow checker | `1/10` No borrow checker |
| Raw runtime performance | `9/10` Intended to be near-native and systems-grade | `10/10` Mature, high-performance systems language | `10/10` Extremely strong low-level performance profile | `7/10` Good, but GC and runtime tradeoffs remain |
| Performance transparency | `9/10` Core design goal | `9/10` Strong | `10/10` Strong | `5/10` More hidden runtime costs than Orbis intends |
| Predictability of runtime cost | `9/10` Explicit-cost design is a core goal | `9/10` Strong and mature | `10/10` Best-in-class explicitness | `5/10` Runtime behavior is less explicit |
| Allocation visibility | `9/10` Intended to keep allocation decisions visible | `8/10` Strong, though abstractions can hide details | `10/10` Very explicit allocation model | `4/10` Allocation costs are less visible by design |
| Low-level control | `8/10` Strong, but not trying to beat Zig at raw control | `9/10` Strong with safety machinery | `10/10` Primary strength | `4/10` Lower-level control is not a main design goal |
| Backend productivity per unit performance | `8/10` Intended balance between rigor and practicality | `6/10` Powerful, but heavier for many backend teams | `5/10` Strong control, lower productivity for typical backend work | `9/10` One of Go's strongest tradeoffs |
| Mental-model simplicity | `9/10` Explicit goal: smaller than Rust | `6/10` Powerful but demanding | `7/10` Simple in some areas, sharp in others | `9/10` Small and teachable |
| Language regularity | `9/10` High priority | `8/10` High, but with more feature interaction | `7/10` High at low level, but rougher edges | `8/10` Simple surface, less expressive |
| Syntax analyzability | `10/10` Explicit design goal | `8/10` Strong, but more semantic interaction | `7/10` Explicit, but lower-level and rougher | `8/10` Very parseable, but less expressive structure |
| Refactorability and transformability | `10/10` Explicitly optimized for machine-assisted maintenance | `8/10` Strong, but borrow/type complexity raises cost | `6/10` Explicit, but more manual edge handling | `7/10` Simpler than Rust, but weaker type precision |
| Tooling philosophy | `8/10` One official toolchain and canonical formatting | `9/10` Strong tooling ecosystem | `7/10` Strong but leaner tooling model | `9/10` Strong official toolchain |
| Formatting uniformity | `10/10` Canonical formatting is a core policy | `6/10` Strong formatter, but less central as a language-level rule | `9/10` Strong formatter culture | `10/10` gofmt-level uniformity is one of Go's strongest properties |
| Agent-friendliness | `10/10` Explicit design goal | `7/10` Good in many areas, but more semantic complexity | `6/10` Good for explicitness, weaker for structural guarantees | `7/10` Good simplicity, weaker safety/model precision |
| Invalid-state modeling | `9/10` Core goal through ADTs, matching, and explicit types | `9/10` Very strong | `5/10` Weaker as a language-level priority | `6/10` Better than dynamic languages, but less expressive here |
| Deterministic destruction and lifetime control | `9/10` Ownership plus arenas | `10/10` Mature ownership and destruction model | `9/10` Strong manual control | `3/10` GC-oriented runtime model |
| Long-term maintainability structure | `9/10` Strong bias toward explicitness, analyzability, and stable tooling | `8/10` Strong, but complexity raises maintenance cost | `6/10` Powerful, but rougher long-term ergonomics | `8/10` Operationally maintainable, but with weaker semantic guarantees |
| Target feeling | `9/10` High-trust systems with smaller mental model than Rust | `10/10` Maximal safety and control with higher complexity | `8/10` Explicit systems programming with manual sharp edges | `8/10` Productive backend engineering with weaker low-level guarantees |
| Average score | `8.9/10` | `8.5/10` | `7.0/10` | `6.4/10` |

## Orbis vs Rust

Rust is the closest reference point for Orbis.

What Orbis wants from Rust:

* ownership
* deterministic memory management
* explicit costs
* strong compile-time safety
* good tooling

What Orbis is trying to change:

* smaller borrowing model
* fewer advanced type-system features early on
* more restrictions against long-distance or subtle borrowing patterns
* more emphasis on regularity and machine-oriented transformation

Current intended tradeoff:

* Rust is likely to remain more expressive
* Orbis aims to be easier to predict and easier to maintain

Orbis should win over Rust only if it actually achieves:

* a meaningfully smaller mental model
* strong enough safety without collapsing into manual memory pain

If it fails there, users should just use Rust.

## Orbis vs Zig

Zig is the closest reference for explicitness and performance transparency.

What Orbis wants from Zig:

* visible costs
* low magic
* explicit behavior
* systems-language seriousness

What Orbis is trying to change:

* stronger compile-time safety in safe code
* stronger invalid-state prevention
* more structured ownership story
* more constrained and analyzable language surface

Current intended tradeoff:

* Zig gives more direct low-level control
* Orbis aims to give stronger correctness guarantees and more structured long-term maintenance

Orbis should not try to beat Zig at "closest to the metal." That is not the right fight.

## Orbis vs Go

Go is the main reference for simplicity, operational clarity, and backend adoption.

What Orbis wants from Go:

* a small language
* practical focus
* strong standard tooling story
* suitability for backends and services

What Orbis is trying to change:

* eliminate garbage collection as a default requirement
* provide stronger compile-time guarantees
* use richer type modeling
* make performance costs more visible

Current intended tradeoff:

* Go remains easier for broad teams and faster to teach
* Orbis aims to support higher-trust and more performance-sensitive systems with stronger compiler help

Orbis should win over Go only in cases where:

* correctness pressure is high
* memory/runtime behavior matters
* long-term maintenance cost matters

If the project target is simple CRUD or low-risk services, Go may still be the better choice.

## Where Orbis Should Be Careful

The positioning only works if Orbis avoids these traps:

* becoming Rust-level complex without Rust-level payoff
* becoming Zig-level manual without Zig-level control benefits
* becoming Go-like simple while losing the ownership-based differentiation

That means the language needs a narrow identity:

* ownership and safety matter
* explicitness matters
* analyzability matters
* the mental model must stay smaller than Rust

## Practical Summary

Use Rust if you want:

* the strongest existing ownership ecosystem
* maximum low-level control with mature safety machinery
* a proven systems-language toolchain today

Use Zig if you want:

* direct systems programming control
* allocator-centric design
* explicit low-level behavior with fewer language constraints

Use Go if you want:

* fast team adoption
* simple backend development
* a mature productivity-oriented server language

Use Orbis, if it succeeds, when you want:

* Rust-like seriousness about safety
* Zig-like explicitness about cost
* Go-like focus on practical backend and infrastructure domains
* a smaller and more regular reasoning model than Rust
