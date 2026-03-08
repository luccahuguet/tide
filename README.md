# Tark

Tark is a language design project for a compiled, ownership-based programming language aimed at high-trust software.

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

The current preferred source file extension is `.tark`.

## Repository Guide

The main documents are:

* [spec.md](./spec.md): the working language specification
* [TASKS.md](./TASKS.md): indexed design and implementation tasks
* [FEATURE_REVIEW.md](./FEATURE_REVIEW.md): feature-by-feature strategic review
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

Still in progress:

* regions and arenas
* region escape rules
* shared ownership
* pattern matching details
* literal syntax
* effects and concurrency
* package and toolchain details

## Near-Term Priorities

The next core design work is focused on the remaining memory-model tasks:

* region and arena creation
* region escape rules
* shared ownership primitives

After that, the likely next areas are:

* pattern matching
* literal syntax
* type-system constraints
* package/tooling structure

## Principle

Tark is trying to be bold selectively:

* bold in ownership, explicitness, and analyzability
* conservative where extra semantic novelty would make the system harder to reason about
