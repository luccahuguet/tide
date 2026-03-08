# Orbis

Orbis is a language design project for a compiled, ownership-based programming language aimed at high-trust software.

The current direction is:

* safety first
* explicit costs and predictable performance
* simpler ownership and borrowing than Rust
* strong support for long-lived codebases
* syntax and tooling designed to work well for both humans and coding agents

## Status

This repository is currently design-first.

There is no compiler implementation yet. The current work is focused on:

* language vision
* core syntax
* memory model decisions
* design tradeoff review
* implementation planning

## Current Language Direction

The language currently targets:

* backends
* infrastructure systems
* CLIs
* services
* data systems

Current planned characteristics:

* compiled
* statically typed
* ownership-based
* safe by default
* performance-transparent
* tooling-first

The public language name is **Orbis**.

The official CLI binary is `orb`.

The official source file extension is `.orb`.

## Repository Guide

The main documents are:

* [spec.md](./spec.md): spec index and document map
* [spec/LANGUAGE.md](./spec/LANGUAGE.md): the working language specification
* [spec/VISION.md](./spec/VISION.md): project vision, priorities, and positioning
* [spec/TOOLING.md](./spec/TOOLING.md): tooling, manifests, workspaces, and package design
* [spec/IMPLEMENTATION.md](./spec/IMPLEMENTATION.md): bootstrap and self-hosting strategy
* [TASKS.md](./TASKS.md): indexed design and implementation tasks
* [PUBLIC_NAME_REVIEW.md](./PUBLIC_NAME_REVIEW.md): naming review history and final decision
* [DOMAIN_STRATEGY.md](./DOMAIN_STRATEGY.md): domain strategy review for `Orbis` and `orb`
* [FEATURE_REVIEW.md](./FEATURE_REVIEW.md): feature-by-feature strategic review
* [COMPARISON.md](./COMPARISON.md): positioning against Rust, Zig, and Go
* [MEMORY_MODEL_RESEARCH.md](./MEMORY_MODEL_RESEARCH.md): research notes for the ownership and borrowing design
* [GOVERNANCE.md](./GOVERNANCE.md): project governance rules

## Current Progress

Already specified:

* core syntax for functions, bindings, blocks, expressions, and statements
* modules, imports, exports, and visibility
* traits and implementations
* generic types and generic functions
* move semantics
* copy-safe type rules
* basic borrowing model with lexical borrow scopes
* arenas, arena escape rules, and shared ownership primitives
* effects, async, structured concurrency, and cancellation
* error handling with `Option`, `Result`, `?`, and `panic`
* package manifests, workspaces, and dependency resolution

Still in progress:

* registry and publishing strategy
* documentation generation and machine-readable tooling output
* canonical formatting details
* implementation sequencing and self-hosting criteria

## Near-Term Priorities

The next core design work is focused on:

* registry and publishing strategy
* documentation generation
* machine-readable tooling output
* formatting rules
* implementation sequencing

## Principle

Orbis is trying to be bold selectively:

* bold in ownership, explicitness, and analyzability
* conservative where extra semantic novelty would make the system harder to reason about
