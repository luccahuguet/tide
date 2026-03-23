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

Ferox is a compiled programming language for **high-trust software** and **safe change**.

The project should aim for:

* preserving Rust-grade safe-code guarantees for memory safety and data-race safety through a simpler and more restrictive model
* lower semantic entropy than Rust
* explicit performance behavior
* fast incremental feedback
* tooling that helps both humans and agents evolve codebases without semantic drift

Core idea:

> Build software that is hard to break, easy to reason about, and safer to change at scale.

Ferox should not try to win by being more expressive than Rust or more magical than modern FP languages.

Ferox should try to win by making large codebases:

* easier to read
* easier to reason about locally
* safer to refactor
* harder to accidentally degrade
* easier to validate mechanically

---

## Priority Hierarchy

When tradeoffs occur, the language prioritizes:

1. **Safe change > local convenience**
2. **Safety > fewer lines of code**
3. **Low entropy and analyzability > feature breadth**
4. **Fast feedback > semantic ambition**
5. **Explicit cost > hidden convenience**

Secondary rule:

> Ferox should preserve power only when doing so does not weaken local reasoning, regularity, or feedback speed.

---

## Low-Entropy Rule

Ferox should aggressively resist semantic sprawl.

Primary rule:

> Prefer one dominant way to solve a problem, with at most one explicit escape hatch.

This matters especially for:

* ownership and borrowing
* mutation
* concurrency
* error handling
* abstraction mechanisms
* package and tooling workflows

Design guidance:

* overlapping mechanisms are a cost, not a free feature
* restrictions are often better than widening the model early
* if two features solve mostly the same problem, v1 should usually pick one
* every new subsystem should justify its entropy cost directly

Reason:

* this keeps the language easier to learn
* this keeps the compiler and diagnostics smaller
* this improves formatter and refactor determinism
* this makes "smaller than Rust" a real engineering constraint instead of a slogan

---

## Core Identity

Ferox is:

* Compiled
* Statically typed
* Ownership-based
* Safe by default
* Performance-transparent
* Low-entropy
* Tooling-first

In strategic terms, Ferox should keep the **language rigid** while making the **tooling extensible**.

That means:

* avoid syntax-extension culture
* avoid multiple competing user-defined abstraction layers for the same job
* prefer stronger compiler services, rule engines, diagnostics, and refactor tooling over language-level extensibility

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

Functional programming influences Ferox, but Ferox should borrow FP mostly as a **code-shaping discipline**, not as a second semantic center.

Included FP ideas:

* Immutability by default
* Algebraic data types
* Pattern matching
* Exhaustive match checking
* Expression-oriented constructs
* Small pure helper functions
* Narrow pipeline-friendly data transformation where it improves readability
* At most one official pipe operator if it proves readable, regular, and low-ambiguity in practice

Not adopted from FP:

* Lazy evaluation by default
* Purity absolutism
* Heavy monadic ceremony
* Higher-kinded types in early versions
* Type-level complexity that weakens local reasoning
* Broad operator culture

Philosophy:

> Prefer functional shape where it improves readability, regularity, and safe transformation.

---

## Performance And Feedback Philosophy

Ferox favors both **predictable performance** and **fast feedback**.

Design principles:

* runtime costs should be visible in the source
* allocations should not be hidden
* copies and moves should be understandable
* async behavior should be explicit
* there should be no hidden global runtime
* incremental `ferox check` should be treated as a first-class product requirement

Performance and tooling goals:

* low memory overhead
* native performance
* deterministic destruction
* efficient concurrency
* fast incremental checking
* reusable compiler services

## Concurrency Direction

Ferox should treat ownership-aware concurrency as part of its identity, not just as a deferred implementation detail.

Core direction:

* concurrency should make ownership transfer, immutable sharing, and synchronized sharing visually distinct
* shared mutable concurrency should remain explicit and second-class relative to ownership transfer and immutable sharing
* Ferox should prefer clearer concurrency boundaries over richer qualifier lattices

This is one of the main ways Ferox should stay safer to evolve than ad hoc concurrent systems code.

---

## Tooling And Agent Design Principles

Ferox should be designed for **mechanically safer change**.

"Better for agents" should mean:

* easier to parse
* easier to transform
* easier to refactor
* easier to validate mechanically
* easier to repair without semantic drift

Key design goals:

* small grammar
* low syntax ambiguity
* canonical formatting
* highly regular syntax
* explicit imports and exports
* full-fidelity syntax trees
* error-tolerant parsing
* predictable AST structure
* normalized checked IR
* machine-readable diagnostics and symbol data

Ferox should therefore prefer:

* fewer overlapping abstractions
* more explicit default paths
* smaller semantic surface area
* one dominant model where possible instead of several partially competing ones
* query-oriented compiler services over tooling built around one-shot batch-only execution
* official structural rules and autofix capabilities over ad hoc source rewriting

Strategic differentiator:

* Ferox should treat first-party structural rules and safe autofix as one of its strongest long-term advantages for large-scale, mechanically validated change

## Safe-Change Tooling Contract

Ferox should treat the following as part of the long-term safe-change contract:

* canonical formatting
* structured diagnostics
* stable machine-readable outputs
* stable symbol and project metadata
* deterministic builds and dependency resolution
* refactor-oriented compiler services

These are not incidental tooling niceties.

They are part of the language's strategy for mechanically safer change.

---

## Verification Direction

Ferox should be **verification-ready**, not proof-first.

Near-term rule:

* do not turn the core language into a theorem prover or dependent-type system

Longer-term direction:

* contracts
* invariants
* ghost or spec code
* proof-oriented tooling for critical modules when justified by real usage

The goal is to support deeper assurance where it matters without forcing the whole language into proof-oriented everyday programming.

---

## Strategic Positioning

Ferox is closest to Rust, but the project should not position itself as "Rust, but easier because it does less safety."

The intended position is:

* Rust-grade safe-code guarantees for memory safety and data-race safety through stricter and more regular rules
* stricter and more regular than Rust
* more explicit about code shape and cost
* more disciplined about tooling and machine-usable outputs
* better suited to safe automated change in long-lived codebases

Current caution:

* Ferox has not earned claims of superiority over mature languages
* the language is still in design and early bootstrap planning
* comparative claims should remain conditional until the implementation and tooling prove the thesis

Core promise:

> Build a lower-entropy ownership-based language for high-trust software and safe change.
