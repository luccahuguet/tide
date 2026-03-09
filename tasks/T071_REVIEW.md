# T071 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first implementation-readiness verdict completed

## Verdict

Current recommendation:

* **conditional go**

Ferox is ready to start a **narrow Rust bootstrap**, but not ready to behave as if the entire semantic core is frozen and ready for deep implementation all at once.

## What Is Ready Now

The narrowed v1 direction is strong enough to start:

* workspace and crate layout
* source handling, spans, tokens, and parser infrastructure
* concrete syntax tree and normalized AST design
* formatter implementation
* diagnostics data model and rendering
* manifest and workspace parsing
* driver scaffolding such as `ferox fmt` and `ferox check`
* early checked-IR design work

This is a real implementation start, not project theater.

## What Can Start Carefully But Should Stay Incremental

The following can start, but only with a staged mindset:

* name resolution
* visibility checking
* ordinary type checking
* trait and generic validation for the narrowed v1 surface
* ownership and async analysis scaffolding

These should be built against the narrowed rules already in the spec and should avoid speculative support for deferred systems.

## What Should Still Wait

Do **not** front-load:

* a full ownership checker designed for advanced or non-lexical borrowing patterns
* arena-aware semantics as if arenas were still part of the v1 core
* a broad concurrency runtime beyond the current `async let` model
* advanced cancellation behavior
* rich effect tracking
* effect handlers or richer effect systems
* capability-heavy concurrency machinery
* advanced backend optimization work
* Perceus-style reuse work beyond later research

## Why The Verdict Is Not A Full Go

### 1. The v1 core is narrower, but still new

The important design cuts are now in place, but they are still young.

That means implementation should validate them rather than assume they are beyond revision.

### 2. Some rules are clear enough for implementation, but not yet proven in code

This is especially true for:

* ownership diagnostics quality
* interactions among traits, generics, and ownership
* async and concurrency edge cases

These are not reasons to block all implementation.

They are reasons to sequence the work honestly.

### 3. Ferox still benefits from architecture that absorbs change

The current implementation plan already points in the right direction:

* modular Rust workspace
* syntax and diagnostics as reusable libraries
* formatter before deep semantics
* checked IR before backend ambition

That is exactly the shape needed for a conditional-go start.

## Go / No-Go By Layer

### Go now

* parser
* CST / AST
* formatter
* diagnostics
* manifest and workspace parsing
* CLI and driver scaffolding

### Conditional go now

* name resolution
* type checking
* trait and generic checking
* checked IR design
* ownership and async analysis prototypes

### No-go for now

* ambitious ownership expressiveness beyond the narrowed spec
* broad async runtime design beyond the current small model
* rich cancellation semantics
* advanced optimization and backend sophistication
* any implementation work that quietly assumes deferred v2 or v3 systems are coming back into v1

## Remaining Risks

The main remaining risks are:

* semantic edge cases in ownership and async behavior
* diagnostic quality for the simplified borrowing model
* pressure to reintroduce deferred systems during implementation
* building semantic stages too aggressively before syntax, formatter, and checked-IR design are stable

## Practical Recommendation

Start implementation now, but start with the following discipline:

1. build the Rust workspace and parser / formatter / diagnostics path first
2. define the checked IR as a regularized internal form
3. add semantic stages only against the already-narrowed v1 rules
4. treat any pressure to reintroduce effects, arenas, or richer concurrency into v1 as a design-review event, not as an implementation convenience

## Current Recommendation

For Ferox:

* implementation should begin
* the start should be **narrow and architecture-first**
* the project should avoid pretending that a full deep semantic implementation is already risk-free
