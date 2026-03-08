# Tark vs Rust, Zig, and Go

This document explains Tark's intended position relative to Rust, Zig, and Go.

Important constraint:

* Tark is still a design-stage language
* some comparisons are about intended direction, not implemented reality
* if the spec changes, this document should change with it

## Short Version

Tark is aiming for this space:

* safer than Go
* simpler than Rust
* more structured and analyzable than Zig
* more machine-friendly than all three

## Comparison Table

| Dimension | Tark | Rust | Zig | Go |
|---|---|---|---|---|
| Safety model | Safe by default with ownership and simplified borrowing | Safe by default with full borrow checker | Lower-level manual control with fewer default safety guarantees | GC-based safety with fewer compile-time correctness guarantees |
| Memory management | Deterministic, no GC, ownership plus lexical borrowing plus arenas/regions | Deterministic, no GC, ownership plus borrowing | Manual allocators and explicit memory management | Garbage collection |
| Borrowing complexity | Intentionally smaller than Rust | High expressiveness, higher complexity | No borrow checker | No borrow checker |
| Performance transparency | Core design goal | Strong | Strong | More hidden runtime costs than Tark intends |
| Language regularity | High priority | High, but with more feature interaction | High at low level, but rougher edges | Simple surface, less expressive |
| Tooling philosophy | One official toolchain and canonical formatting | Strong tooling ecosystem | Strong but leaner tooling model | Strong official toolchain |
| Agent-friendliness | Explicit design goal | Good in many areas, but more semantic complexity | Good for explicitness, weaker for structural guarantees | Good simplicity, weaker safety/model precision |
| Target feeling | High-trust systems with smaller mental model than Rust | Maximal safety and control with higher complexity | Explicit systems programming with manual sharp edges | Productive backend engineering with weaker low-level guarantees |

## Tark vs Rust

Rust is the closest reference point for Tark.

What Tark wants from Rust:

* ownership
* deterministic memory management
* explicit costs
* strong compile-time safety
* good tooling

What Tark is trying to change:

* smaller borrowing model
* fewer advanced type-system features early on
* more restrictions against long-distance or subtle borrowing patterns
* more emphasis on regularity and machine-oriented transformation

Current intended tradeoff:

* Rust is likely to remain more expressive
* Tark aims to be easier to predict and easier to maintain

Tark should win over Rust only if it actually achieves:

* a meaningfully smaller mental model
* strong enough safety without collapsing into manual memory pain

If it fails there, users should just use Rust.

## Tark vs Zig

Zig is the closest reference for explicitness and performance transparency.

What Tark wants from Zig:

* visible costs
* low magic
* explicit behavior
* systems-language seriousness

What Tark is trying to change:

* stronger compile-time safety in safe code
* stronger invalid-state prevention
* more structured ownership story
* more constrained and analyzable language surface

Current intended tradeoff:

* Zig gives more direct low-level control
* Tark aims to give stronger correctness guarantees and more structured long-term maintenance

Tark should not try to beat Zig at "closest to the metal." That is not the right fight.

## Tark vs Go

Go is the main reference for simplicity, operational clarity, and backend adoption.

What Tark wants from Go:

* a small language
* practical focus
* strong standard tooling story
* suitability for backends and services

What Tark is trying to change:

* eliminate garbage collection as a default requirement
* provide stronger compile-time guarantees
* use richer type modeling
* make performance costs more visible

Current intended tradeoff:

* Go remains easier for broad teams and faster to teach
* Tark aims to support higher-trust and more performance-sensitive systems with stronger compiler help

Tark should win over Go only in cases where:

* correctness pressure is high
* memory/runtime behavior matters
* long-term maintenance cost matters

If the project target is simple CRUD or low-risk services, Go may still be the better choice.

## Where Tark Should Be Careful

The positioning only works if Tark avoids these traps:

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

Use Tark, if it succeeds, when you want:

* Rust-like seriousness about safety
* Zig-like explicitness about cost
* Go-like focus on practical backend and infrastructure domains
* a smaller and more regular reasoning model than Rust
