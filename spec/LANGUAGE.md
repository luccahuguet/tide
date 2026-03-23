# Ferox Language Specification (v0.3)

This document is the canonical entrypoint for the Ferox language reference.

Ferox language design currently prioritizes:

* high-trust software and safe change
* Rust-grade safe-code guarantees for memory safety and data-race safety through a simpler, more restrictive model
* lower semantic entropy over extra expressiveness in v1
* readable, mechanically analyzable source code and fast tooling feedback

The detailed normative language rules now live in topic files under `spec/language/`.

## Reference Map

* [language/ownership.md](./language/ownership.md): ownership, borrowing, move semantics, copy-safe types, shared ownership, safety guarantees, and ownership invariants
* [language/types_and_patterns.md](./language/types_and_patterns.md): type-system boundaries, inference rules, ADTs, enum layout, and pattern destructuring
* [language/mutation_and_effects.md](./language/mutation_and_effects.md): mutation rules, shared mutation, `unsafe`, operational explicitness, and external interaction
* [language/async_and_concurrency.md](./language/async_and_concurrency.md): async model, structured concurrency, task capture, and explicit cross-task access modes
* [language/errors.md](./language/errors.md): `Option`, `Result`, propagation, panic semantics, and error-trait boundaries
* [language/syntax.md](./language/syntax.md): syntax philosophy, functions, bindings, blocks, expressions, `match`, literals, and semicolon rules
* [language/modules.md](./language/modules.md): module/file model, imports, exports, and visibility
* [language/traits_and_generics.md](./language/traits_and_generics.md): trait surface, receiver forms, bounds, and generic syntax

## Reading Order

Suggested order for first-time readers:

1. [language/ownership.md](./language/ownership.md)
2. [language/async_and_concurrency.md](./language/async_and_concurrency.md)
3. [language/types_and_patterns.md](./language/types_and_patterns.md)
4. [language/errors.md](./language/errors.md)
5. [language/syntax.md](./language/syntax.md)
6. [language/modules.md](./language/modules.md)
7. [language/traits_and_generics.md](./language/traits_and_generics.md)
8. [language/mutation_and_effects.md](./language/mutation_and_effects.md)

## Related Documents

* [VISION.md](./VISION.md): project goals and positioning
* [TOOLING.md](./TOOLING.md): formatter, CLI, machine contracts, manifests, and package design
* [IMPLEMENTATION.md](./IMPLEMENTATION.md): bootstrap strategy, compiler architecture, validation harness, and self-hosting plan
