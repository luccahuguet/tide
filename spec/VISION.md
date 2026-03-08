# Tark Vision

## Language Name

The language is called **Tark**.

Status:

* selected as the project name
* should still pass trademark and brand-conflict review before becoming final for public release

The name is treated as a standalone project name.

This reflects the philosophy of the language:

* correctness-first software
* strong logical foundations
* high-trust systems
* compiler-enforced guarantees

Example CLI:

tark build
tark run
tark fmt
tark test

Tentative file extension:

`.tark`

Status:

* **Not yet approved as a final decision**
* `.ax` was rejected as the preferred extension
* `.tark` is the current preferred extension because it is explicit and low-ambiguity
* avoid short extensions that are likely to create domain or file-format confusion

Pre-finalization checks required:

* trademark search in target jurisdictions
* product and company-name conflict review in developer tools and software markets
* domain strategy review for `tark.*`
* source file extension review, including collision risk, searchability, editor support, and confusion with internet domains

---

## Vision

Tark is a compiled programming language designed for **high-trust software** where correctness, analyzability, and predictable performance are prioritized over convenience.

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

## Core Identity

Tark is:

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

Functional programming influences Tark, but it is **not a purely functional language**.

Included FP ideas:

* Immutability by default
* Pure functions encouraged
* Algebraic data types
* Pattern matching
* Expression-oriented constructs
* Exhaustive match checking
* Explicit effect boundaries

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

Tark favors **predictable performance**.

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

Tark is designed to be **machine-friendly**.

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

---

## Strategic Positioning

Tark aims to occupy a space between:

**Go and Rust**

and between:

**Zig and Rust**

Positioning:

* Safer and more correct than Go
* Simpler and more coherent than Rust
* More maintainable and structured than Zig
* Better suited for agent-assisted development than all three

Core promise:

> Fast software with strong correctness guarantees that both humans and agents can evolve safely.
