# Ferox Tooling Reference: Toolchain Scope

This document defines the overall Ferox toolchain direction and bootstrap scope.

## Tooling Philosophy

A single official toolchain exists.

The toolchain should be treated as part of the language contract for safe change, not as a thin convenience wrapper around unrelated tools.

Core direction:

* structured diagnostics and machine-readable outputs are first-class
* incremental `ferox check` should be a product requirement
* formatter and parser architecture should support editor and agent workflows on partially complete code
* tooling interfaces should grow from the same compiler data model rather than from scraped terminal text

The official machine-facing surface should eventually include more than diagnostics alone.

Long-term direction:

* stable symbol identity and symbol graphs
* project and package metadata suitable for tools and agents
* refactor-oriented compiler services
* machine-readable build and analysis events with controlled evolution

CLI:

ferox

Long-term responsibilities:

* build
* run
* test
* format
* lint
* benchmark
* documentation generation
* dependency management
* package publishing

Initial implementation scope should stay narrower.

The first bootstrap wave should prioritize:

* `ferox check`
* `ferox fmt`
* diagnostics
* package loading
* basic build-oriented workflows

Commands such as documentation generation, richer dependency workflows, and package publishing remain part of the official long-term toolchain direction, but they are not required to prove the initial safe-core bootstrap.

Goals:

* Excellent diagnostics
* Machine-readable compiler output
* Deterministic builds
* First-class testing support
