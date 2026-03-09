# Ferox Vision

## Language Name

The language is called **Ferox**.

Status:

* selected as the public language name
* official CLI binary is `ferox`
* official source file extension is `.ferox`
* first-pass domain, trademark, and brand review are complete
* legal follow-up still remains advisable before public launch

The name is treated as a standalone project name.

This reflects the philosophy of the language:

* correctness-first software
* strong logical foundations
* high-trust systems
* compiler-enforced guarantees

Example CLI:

ferox build
ferox run
ferox fmt
ferox test

Official file extension:

`.ferox`

Status:

* selected as the current official source-file extension
* `.ax` was rejected earlier in the naming process
* short extensions still need practical review for searchability, editor support, and ecosystem collisions

Follow-up checks still recommended:

* legal clearance review in target jurisdictions for `Ferox` and `ferox`
* deeper product and company-name conflict review if the project approaches public launch
* registrar and social-handle checkout once the project is ready to claim public assets
* source file extension follow-up, including editor support, registry conventions, and practical ecosystem friction

---

## Vision

Ferox is a compiled programming language designed for **high-trust software** where correctness, analyzability, and predictable performance are prioritized over convenience.

The language is optimized for **both humans and coding agents** to read, reason about, and safely evolve codebases.

Core idea:

> Build fast software that both humans and agents can maintain safely over long periods.

---

## Priority Hierarchy

When tradeoffs occur, the language prioritizes:

1. **Safety > fewer lines of code**
2. **Agentic-use > human readability**
3. **Performance > elegance**
4. **Compiler guarantees > ease of learning**

Secondary rule:

> When possible, preserve both sides of the tradeoff without compromising the higher priority.

---

## Coherence Rule

Ferox should avoid multiple overlapping ways to do the same job.

Primary rule:

> Prefer one primary way to do something, with at most one explicit escape hatch.

This is especially important for:

* memory handling
* mutation
* concurrency
* error handling
* package and tooling workflows

Design guidance:

* multiple mechanisms are acceptable when they solve genuinely different problems
* multiple mechanisms are bad when they are mostly different ways to reach the same result
* if two features overlap heavily, v1 should usually pick one and defer the other
* restrictions are often better than overlapping escape hatches in an early language

Reason:

* this keeps the language easier to learn
* this keeps the compiler and diagnostics smaller
* this keeps the mental model more coherent for both humans and coding agents
* this makes "smaller than Rust" a real constraint instead of a slogan

---

## Core Identity

Ferox is:

* Compiled
* Statically typed
* Ownership-based
* Safe by default
* Performance-transparent
* Agent-friendly
* Tooling-first

Target domains:

* Backends
* Infrastructure systems
* CLIs
* Data systems
* Services
* Long-lived codebases
* Agent-assisted development environments

Non-goals:

* Being the simplest beginner language
* Replacing C for extremely low-level systems work
* Being a purely functional language
* Being a scripting language
* Supporting many paradigms equally

---

## Functional Programming Influence

Functional programming influences Ferox, but it is **not a purely functional language**.

Included FP ideas:

* Immutability by default
* Pure functions encouraged
* Algebraic data types
* Pattern matching
* Expression-oriented constructs
* Exhaustive match checking
* Explicit async and unsafe boundaries

Not adopted from FP:

* Lazy evaluation by default
* Purity absolutism
* Heavy monadic ceremony
* Complex type-level programming
* Higher-kinded types (initial versions)

Philosophy:

> Functional defaults with practical systems programming capabilities.

---

## Performance Philosophy

Ferox favors **predictable performance**.

Design principles:

* Runtime costs should be visible in the source
* Allocations should not be hidden
* Copies and moves should be understandable
* Async behavior should be explicit
* No hidden global runtime

Performance model goals:

* Low memory overhead
* Native performance
* Deterministic destruction
* Efficient concurrency

---

## Agentic Design Principles

Ferox is designed to be **machine-friendly**.

Key design goals:

* Small grammar
* Low syntax ambiguity
* Canonical formatting
* Highly regular syntax
* Explicit imports and exports
* Predictable AST structure
* Machine-readable diagnostics

The language should be easy for agents to:

* Parse
* Transform
* Refactor
* Verify
* Repair

Ferox should therefore prefer:

* fewer overlapping abstractions
* more explicit default paths
* smaller semantic surface area
* one dominant model where possible instead of several partially competing ones

---

## Strategic Positioning

Ferox is trying to occupy a narrow space between Rust, Zig, and Go without copying any of them directly.

Current aspiration:

* ownership-based and safety-oriented like Rust
* explicit about runtime cost like Zig
* practical about tooling and backend use like Go

Current caution:

* Ferox has not earned broad claims of being better than mature languages
* the language is still narrowing its v1 core
* comparative claims should stay conditional until the design proves coherent

Core promise:

> Build a smaller, more analyzable ownership-based language for high-trust software.
