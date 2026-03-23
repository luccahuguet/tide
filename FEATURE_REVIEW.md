# Ferox Feature Review

This document reviews the distinct goals and feature bets currently stated across `spec/VISION.md`, `spec/LANGUAGE.md`, `spec/TOOLING.md`, and `GOVERNANCE.md`.

It does **not** treat every numbered task or `br` issue as a separate feature. The task system is implementation decomposition; this file is a strategic review of the underlying ideas.

## Rating Scale

* `1` = very low
* `2` = low
* `3` = medium
* `4` = high
* `5` = very high

Definitions:

* `Cost`: design and implementation cost
* `Benefit`: direct value to users and maintainers
* `Danger`: risk of harming coherence, adoption, or implementation viability
* `Complexity`: conceptual and compiler complexity
* `Potential`: long-term upside if executed well

Recommendation labels:

* `Keep`: core to the identity and worth keeping as-is
* `Keep/Simplify`: keep the idea, but narrow the scope
* `Defer`: worthwhile, but not for early versions
* `Move Out`: should exist, but outside the language spec
* `Drop`: not worth the current cost or risk

## Strategic Summary

Ferox has one very strong core bet:

* a lower-entropy systems language for safe change centered on ownership, explicitness, long-term maintainability, and machine-usable tooling

The main risk is not lack of ambition. The main risk is **too many hard bets at once**.

The best path is:

* keep the boldness in ownership, explicitness, mechanically safer change, and machine-friendly regularity
* simplify or defer the rest until the core model proves coherent
* prefer one primary way to do something, with at most one explicit escape hatch

## Product and Language Direction

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F01 | High-trust, correctness-first language | 3 | 5 | 2 | 3 | 5 | Keep | Strong north star. It gives the language a clear market and design filter. |
| F02 | Optimized for humans and coding agents | 3 | 4 | 3 | 3 | 5 | Keep | Distinctive and worth pursuing, but it should mean mechanically safer change, not vague AI branding. |
| F03 | Target domain focus on backends, infra, CLIs, services, data systems, long-lived codebases | 2 | 4 | 1 | 1 | 4 | Keep | Good constraint. It prevents the project from drifting into general-purpose wish-list design. |
| F04 | Non-goals: not beginner-first, not C replacement, not scripting, not multi-paradigm | 1 | 4 | 1 | 1 | 3 | Keep | These exclusions are healthy. They reduce design churn. |

## Core Identity

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F05 | Compiled language | 4 | 4 | 1 | 3 | 4 | Keep | Necessary for the performance and tooling story you want. |
| F06 | Static typing | 3 | 5 | 1 | 3 | 4 | Keep | Non-negotiable for the guarantees and refactoring goals. |
| F07 | Ownership-based design | 5 | 5 | 4 | 5 | 5 | Keep | This is the strongest differentiator, but it is also the hardest subsystem. This should be the primary innovation focus. |
| F08 | Safe by default | 4 | 5 | 2 | 4 | 5 | Keep | Essential if Ferox wants to sit between Go and Rust in a meaningful way. |
| F09 | Performance-transparent model | 4 | 5 | 2 | 4 | 5 | Keep | Very good positioning. Explicit costs are a real advantage for both humans and agents. |
| F10 | Tooling-first language | 4 | 4 | 2 | 3 | 5 | Keep | Worth keeping, but implement conservatively. Tooling-first should focus on safe change, machine-readable outputs, and fast feedback rather than novelty everywhere. |

## Memory and Safety

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F11 | Ownership + simplified borrowing | 5 | 5 | 5 | 5 | 5 | Keep | This is the big bet. It can make Ferox real, or it can consume the entire project. Prioritize this above everything else. |
| F12 | Deterministic memory management without GC | 5 | 5 | 3 | 5 | 5 | Keep | Necessary if you want predictable performance and a serious systems-language identity. |
| F13 | Explicit move/copy model | 4 | 5 | 2 | 4 | 5 | Keep | Strong payoff for clarity and performance. It reinforces the ownership story. |
| F14 | First-class regions/arenas | 5 | 4 | 5 | 5 | 5 | Defer | Valuable in some designs, but too costly for the v1 core unless real pressure later proves they are necessary. |
| F15 | Explicit shared ownership | 4 | 4 | 3 | 4 | 4 | Keep/Simplify | Necessary, but keep the initial surface very small. |
| F16 | Explicit and isolated `unsafe` | 3 | 5 | 2 | 3 | 4 | Keep | Mandatory for a safe systems language. |
| F17 | Structural safety guarantees in safe code | 4 | 5 | 3 | 4 | 5 | Keep | Good goal. The wording should stay narrow and mechanical, not grandiose. |

## Type System and Error Model

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F18 | Strong local inference with explicit public API types | 3 | 5 | 1 | 3 | 4 | Keep | Excellent balance. This is a practical and proven compromise. |
| F19 | ADTs / enums / exhaustive matching | 4 | 5 | 2 | 4 | 5 | Keep | High payoff for correctness and reasoning. Strong fit for Ferox. |
| F20 | Traits / protocols instead of inheritance | 4 | 4 | 2 | 4 | 4 | Keep/Simplify | Good choice, but keep the trait model small in v1. |
| F21 | No implicit conversions | 2 | 4 | 1 | 2 | 3 | Keep | Cheap and useful. Good for analyzability. |
| F22 | No null values | 3 | 5 | 1 | 3 | 4 | Keep | Worth it. The ecosystem will be cleaner for it. |
| F23 | `Option` and `Result` as core primitives | 3 | 5 | 1 | 3 | 4 | Keep | Necessary if you reject null and exceptions. |
| F24 | No exceptions for control flow | 2 | 4 | 1 | 2 | 3 | Keep | Good fit for a predictable systems language. |
| F25 | No higher-kinded types in early versions | 1 | 3 | 1 | 1 | 2 | Keep | This is a healthy constraint. It protects early coherence. |

## Functional and Mutation Model

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F26 | Immutability by default | 3 | 4 | 1 | 3 | 4 | Keep | Good default. It complements ownership well. |
| F27 | Pure functions encouraged | 2 | 3 | 2 | 2 | 3 | Keep | Fine as guidance, but it should remain cultural, not rigid. |
| F28 | Explicit mutation with exclusive mutable access | 4 | 5 | 2 | 4 | 5 | Keep | This is core to the safety story. |
| F29 | Shared mutation only through explicit synchronization or special constructs | 4 | 5 | 3 | 4 | 5 | Keep/Simplify | Right direction. Keep the initial set of “special constructs” extremely small. |

## Effects and Concurrency

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F30 | Explicit effects model | 5 | 4 | 5 | 5 | 5 | Defer | This is attractive, but it is another major semantic subsystem. Too risky to fully innovate here in v1. |
| F31 | Safe concurrency by default | 4 | 5 | 3 | 4 | 5 | Keep | Important goal, but the implementation can be conservative. |
| F32 | Structured concurrency | 4 | 4 | 3 | 4 | 4 | Keep/Simplify | Worth keeping, but start with a narrow async/task model. |
| F33 | Explicit async behavior | 4 | 4 | 2 | 4 | 4 | Keep | Good fit for your explicitness philosophy. |
| F34 | Cancellation-friendly execution model | 4 | 4 | 3 | 4 | 4 | Defer | Good long-term target, but it adds real semantic and runtime complexity. |

## Performance Model

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F35 | Visible runtime costs in source | 4 | 5 | 2 | 4 | 5 | Keep | Strong idea. It is one of the best parts of the vision. |
| F36 | No hidden allocations, copies, async costs, or global runtime | 5 | 5 | 3 | 5 | 5 | Keep/Simplify | Strong promise, but do not over-promise. Some convenience mechanisms may still need explicit lowering rules. |
| F37 | Low memory overhead and native performance | 4 | 5 | 2 | 4 | 5 | Keep | Important and aligned with the domain targets. |

## Agentic and Tooling Design

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F38 | Small grammar / low ambiguity / regular syntax / predictable AST | 4 | 5 | 2 | 4 | 5 | Keep | This is one of Ferox's strongest and most realistic differentiators. |
| F39 | Explicit imports and exports | 2 | 4 | 1 | 2 | 4 | Keep | Cheap win for tooling and readability. |
| F40 | Machine-readable diagnostics | 4 | 5 | 1 | 3 | 5 | Keep | High leverage. Very strong fit for the safe-change and machine-usable tooling vision. |
| F41 | Single canonical formatter | 3 | 4 | 2 | 3 | 4 | Keep | Worth keeping, especially for machine-assisted work. |
| F42 | Enforce formatting through formatter and CI, not build/run/test failure | 2 | 4 | 1 | 2 | 3 | Keep | This is the correct enforcement model. |
| F43 | One official toolchain for build/run/test/fmt/lint/bench/docs/deps/publish | 5 | 5 | 3 | 4 | 5 | Keep/Simplify | Good strategic idea, but the initial toolchain should stay narrow. |

## Error Model and Syntax Surface

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F44 | Expression-oriented syntax | 3 | 4 | 1 | 3 | 4 | Keep | Strong fit for composability and analyzability. |
| F45 | Blocks as expressions with trailing-expression semantics | 3 | 4 | 1 | 3 | 4 | Keep | Good design. Predictable and expressive. |
| F46 | Assignment as a statement, not an expression | 2 | 4 | 1 | 2 | 3 | Keep | Sensible guardrail. It reduces ambiguity. |
| F47 | `if` as an expression | 2 | 4 | 1 | 2 | 3 | Keep | Standard and useful in an expression-oriented language. |
| F48 | No arbitrary syntax macros | 2 | 4 | 1 | 2 | 4 | Keep | This protects long-term coherence. |
| F49 | File-system-based module system with `use` and `pub` | 3 | 4 | 2 | 3 | 4 | Keep | Good early choice. Simple, explicit, and tool-friendly. |

## Implementation Strategy and Governance

| ID | Item | Cost | Benefit | Danger | Complexity | Potential | Recommendation | Notes |
|---|---|---:|---:|---:|---:|---:|---|---|
| F50 | Rust bootstrap implementation | 4 | 5 | 1 | 3 | 4 | Keep | The obvious correct move. It lowers risk. |
| F51 | Self-hosting goal | 5 | 4 | 3 | 5 | 5 | Keep/Defer | Worth keeping as a long-term goal, but it should not distort v1 design choices. |
| F52 | Agent-produced-code governance rule | 2 | 3 | 4 | 1 | 3 | Move Out | Fine as project governance. Wrong place if treated as language semantics. |

## Highest-Value Areas to Double Down On

These are the ideas with the best benefit/potential relative to danger:

* F07 Ownership-based design
* F09 Performance-transparent model
* fast incremental feedback as a product requirement
* query-oriented compiler services and reusable compiler data
* F18 Strong local inference with explicit public API types
* F19 ADTs / enums / exhaustive matching
* F38 Small grammar / predictable AST
* F40 Machine-readable diagnostics
* F49 File-system-based explicit module system
* F50 Rust bootstrap

## Highest-Risk Areas to Control Tightly

These are the ideas most likely to blow up scope or coherence if handled aggressively:

* F11 Ownership + simplified borrowing
* F14 First-class regions / arenas
* F30 Explicit effects model
* F32 Structured concurrency
* F34 Cancellation-friendly execution model
* F36 No hidden runtime costs of any kind
* F43 One official toolchain covering everything from day one

Coherence reminder:

* the risk is not only that each subsystem is hard
* the risk is also that Ferox accumulates multiple overlapping ways to solve the same problem
* the right v1 bias is one dominant model plus a small explicit escape hatch, not parallel first-class approaches

## Recommended v1 Shape

For a coherent first version, the recommended profile is:

* Keep: ownership, explicitness, ADTs, exhaustive matching, `Option`/`Result`, regular syntax, machine-readable diagnostics, canonical formatting, explicit modules, fast-feedback tooling goals, Rust bootstrap
* Simplify: shared ownership, traits, structured concurrency, full toolchain scope, "no hidden runtime" promises, pipe syntax if adopted
* Defer: structural rule/autofix systems until the frontend is credible, contracts/spec layers until the safe core is proven, arenas/regions, rich effects system, advanced cancellation semantics, broad self-hosting milestones, built-in ability experiments until they clearly reduce complexity
* Move Out: agent-production rule as governance rather than language design
