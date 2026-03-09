# Ferox

Ferox is a language design project for a compiled, ownership-based programming language aimed at high-trust software.

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
* narrowing the v1 scope
* preparing the narrow Rust bootstrap

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

The public language name is **Ferox**.

The official CLI binary is `ferox`.

The official source file extension is `.ferox`.

The v1 semantic core has recently been narrowed through an adversarial review pass.

Current implementation verdict:

* start a narrow Rust bootstrap now
* keep deeper semantic ambition staged and architecture-first

## Repository Guide

The main documents are:

* [spec.md](./spec.md): spec index and document map
* [spec/LANGUAGE.md](./spec/LANGUAGE.md): the working language specification
* [spec/VISION.md](./spec/VISION.md): project vision, priorities, and positioning
* [spec/TOOLING.md](./spec/TOOLING.md): tooling, manifests, workspaces, and package design
* [spec/IMPLEMENTATION.md](./spec/IMPLEMENTATION.md): bootstrap and self-hosting strategy
* [TASKS.md](./TASKS.md): completed-task archive
* [BR_WORKFLOW.md](./BR_WORKFLOW.md): `br` workflow and open-task tracking rules
* [V1_DIRECTION.md](./V1_DIRECTION.md): current narrowed v1 direction after the adversarial review
* [ROADMAP.md](./ROADMAP.md): version roadmap and implementation-path summary
* [V2_BACKLOG.md](./V2_BACKLOG.md): post-v1 candidate backlog
* [V3_BACKLOG.md](./V3_BACKLOG.md): long-horizon backlog
* [PUBLIC_NAME_REVIEW.md](./PUBLIC_NAME_REVIEW.md): naming review history and final decision
* [FEROX_DOMAIN_STRATEGY.md](./FEROX_DOMAIN_STRATEGY.md): first-pass domain strategy for `Ferox`
* [FEROX_TRADEMARK_REVIEW.md](./FEROX_TRADEMARK_REVIEW.md): first-pass trademark review for `Ferox`
* [FEROX_SOFTWARE_BRAND_REVIEW.md](./FEROX_SOFTWARE_BRAND_REVIEW.md): first-pass software and developer-brand review for `Ferox`
* [AI_SUPERVISOR.md](./AI_SUPERVISOR.md): mandatory AI supervisor review policy and prompt template
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
* shared ownership primitives as explicit escape hatches
* explicit async syntax and a first-pass concurrency model
* error handling with `Option`, `Result`, `?`, and `panic`
* package manifests, workspaces, and dependency resolution
* registry and publishing strategy
* documentation generation and machine-readable tooling output
* canonical formatting rules
* implementation sequencing and self-hosting criteria

Under active v1 review and still subject to cuts or deferral:

* semantic edge cases that may still surface once ownership and async checks are implemented

Still in progress:

* architecture-first Rust bootstrap work against the narrowed v1
* `br`-tracked open work for the first bootstrap queue

## Near-Term Priorities

The next core design work is focused on:

* keeping the v1 core narrow and coherent
* starting implementation without reintroducing deferred systems
* later-version backlog discipline
* validating the narrowed spec through a real bootstrap path

## Principle

Ferox is trying to be bold selectively:

* bold in ownership, explicitness, and analyzability
* conservative where extra semantic novelty would make the system harder to reason about
