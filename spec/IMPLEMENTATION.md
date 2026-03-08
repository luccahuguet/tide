# Tark Implementation Strategy

## Implementation Strategy

Tark will be **bootstrapped in Rust**.

Reasons:

* Rust aligns with Tark’s safety goals
* Rust has a mature ecosystem for building compilers
* Rust provides excellent performance for language tooling
* Rust enables a fast and reliable first implementation

### Self-hosting goal

Tark is intended to become **self-hosting**.

Plan:

1. **Phase 1 — Rust bootstrap**

   The first compiler, formatter, and toolchain are implemented in Rust.

2. **Phase 2 — Tark transition**

   As the language stabilizes, components are gradually rewritten in Tark:

   * formatter
   * small utilities
   * linter
   * parser
   * type checker
   * compiler frontend
   * full compiler

3. **Phase 3 — Tark-first**

   Tark becomes the primary implementation language for its own ecosystem.

Rust may remain in selected infrastructure layers where it continues to provide strategic value.

Principle:

> Rust brings Tark into existence.
> Tark proves it deserves to exist.
