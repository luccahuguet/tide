# Ferox Language Specification: Error Handling

This document defines Ferox's v1 error-handling model.

## Error Handling

Errors are handled through types.

Recoverable errors:

Result

Absence of value:

Option

Unexpected fatal conditions:

panic / crash

## Option and Result Usage

Ferox uses `Option[T]` for presence or absence and `Result[T, E]` for recoverable success or failure.

Canonical construction forms:

```ferox
let user = some(current_user);
let missing = none;

let parsed = ok(config);
let failed = err(ParseError::new("invalid"));
```

Rules:

* `some(value)` and `none` are the canonical `Option` construction forms
* `ok(value)` and `err(error)` are the canonical `Result` construction forms
* these forms are ordinary value expressions and may appear anywhere an expression is allowed

### Match-Based Usage

The primary control-flow form for `Option` and `Result` is `match`.

Examples:

```ferox
match user {
    some(value) => render(value),
    none => render_missing(),
}
```

```ferox
match parsed {
    ok(value) => use_config(value),
    err(error) => report(error),
}
```

Rules:

* `Option` and `Result` participate in normal exhaustiveness checking
* handling code should make success and failure paths explicit
* nested `Option` and `Result` values may be matched using ordinary nested patterns

### Conditional Pattern Forms

Ferox supports `if let` and `while let` as focused pattern-driven control flow for common `Option` and `Result` usage.

Examples:

```ferox
if let some(value) = maybe_user {
    show(value);
} else {
    show_missing();
}
```

```ferox
while let some(item) = next_item() {
    process(item);
}
```

```ferox
if let ok(config) = parse(text) {
    use_config(config);
}
```

Rules:

* `if let pattern = expr { ... }` is shorthand for a single-success-case match with an optional `else`
* `while let pattern = expr { ... }` repeats while the pattern continues to match
* these forms do not replace full `match` when multiple distinct cases need explicit handling

### Design Intent

Option and Result usage should remain explicit without becoming ceremonious.

Goals:

* keep absence and recoverable failure in the type system
* make common success-path handling concise
* preserve `match` as the general control-flow form
* avoid exception-like hidden control flow

Public API guidance:

* use `Option[T]` when absence is part of the honest domain model
* use `Result[T, E]` when recoverable failure is part of the honest operation model
* do not encode absence or recoverable failure through sentinel primitives or loosely documented flag combinations when a typed representation is clearer

## Error Propagation

Ferox uses postfix `?` for concise propagation of `Result` and `Option`.

Examples:

```ferox
fn load_config(path: String) -> Result[Config, ConfigError] {
    let text = read_file(path)?;
    let config = parse_config(text)?;
    ok(config)
}
```

```ferox
fn current_name(user: Option[User]) -> Option[String] {
    let value = user?;
    some(value.name)
}
```

Rules:

* `expr?` may be used only when `expr` has type `Result[T, E]` or `Option[T]`
* for `Result[T, E]`, `expr?` yields the inner `T` on `ok(...)` and returns early with `err(...)` on failure
* for `Option[T]`, `expr?` yields the inner `T` on `some(...)` and returns early with `none` on absence
* the enclosing function or block must return a compatible `Result` or `Option`
* `?` is shorthand for ordinary early-return control flow, not an exception mechanism

### Propagation Boundaries

Rules:

* `?` does not cross incompatible return types automatically
* conversions between error types are not implicit in v1
* if an error must be transformed, the program should use `match` or an explicit conversion operation before propagation
* `?` propagates typed failure, not async behavior or any other separate semantic system

### Design Intent

The propagation operator should remove boilerplate while preserving the visibility of typed failure paths.

Goals:

* keep `Result` and `Option` ergonomic in straight-line code
* preserve explicit typed error boundaries
* avoid hidden stack unwinding semantics
* keep propagation easy for humans and tools to recognize mechanically

## Panic Semantics

Ferox provides `panic(...)` for unexpected fatal conditions that should immediately abandon normal local error handling.

Example:

```ferox
fn expect_header(bytes: Bytes) -> Header {
    if bytes.len() < HEADER_SIZE {
        panic("header too short");
    }

    parse_header(bytes)
}
```

Rules:

* `panic(message)` is an explicit fatal operation
* panic is not a recoverable `Result` path and is not represented by `Option`
* panic should be rare in ordinary application logic
* safe Ferox code may still panic when an explicit panic path is executed or when the standard library detects a fatal violated precondition

### Acceptable Uses

Panic is acceptable in v1 for:

* violated internal invariants that indicate a bug
* impossible states that should be unreachable in correct code
* runtime contract failures in APIs that are explicitly documented as panicking on misuse
* fatal initialization or runtime conditions where continuing would leave the process in an unusable state

### Non-Acceptable Uses

Panic should not be the default tool for:

* expected IO failures
* normal parse or validation errors
* routine absence of data
* ordinary control flow

These should use `Result` or `Option` instead.

### Design Intent

The language should have one clear typed path for recoverable failure and one clear explicit path for fatal failure.

Goals:

* keep `Result` as the primary recoverable error mechanism
* make panic sites easy to search and audit
* avoid exception-style hidden control flow
* allow invariant enforcement without forcing every impossible case into public error types

## Error Traits and Protocols

Ferox v1 does not require a built-in universal error trait or protocol for all recoverable errors.

Decision:

* `Result[T, E]` accepts any explicit error type `E`
* user code may define domain-specific error traits if useful, but the language does not depend on a mandatory built-in `Error` trait in v1
* standard-library conventions for formatting or reporting errors may use ordinary traits such as `Display` or later dedicated library traits, but this is not a core language requirement

Rationale:

* keeps the error model small while the language and standard library are still settling
* avoids forcing every error type into a shared abstraction before the real reporting and interoperability needs are clear
* preserves the usefulness of simple enum-based error types

### Initial Non-Goals

The initial error model does not include:

* a required universal `Error` supertrait for all `Result` error values
* automatic error boxing or erased dynamic error types as the default pattern
* implicit conversion between unrelated error types through a global error protocol

### Design Intent

Error typing should remain explicit and practical instead of prematurely abstract.

Goals:

* keep `Result[T, E]` flexible for concrete domain errors
* avoid unnecessary trait machinery in the first version
* leave room for later library-level conventions once real usage patterns exist

Exceptions are **not used** for standard control flow.

---
