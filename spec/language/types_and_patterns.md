# Ferox Language Specification: Types and Patterns

This document defines Ferox's v1 type-system boundaries, ADTs, and destructuring rules.

## Type System

Ferox uses:

* Static typing
* Strong type inference for local variables
* Explicit types in public APIs
* Algebraic data types (ADTs)
* Tagged unions / enums
* Exhaustive pattern matching
* Traits / protocols instead of inheritance
* No implicit conversions
* No null values

Built-in primitives:

Option
Result

Goals:

* Make invalid states unrepresentable
* Increase compiler guarantees
* Improve program reasoning

## Inference Boundaries

Ferox uses strong local inference, but inference is intentionally bounded.

Design rules:

* local implementation detail may be inferred
* public API surface must remain explicit
* inference should reduce noise without hiding type structure that matters for maintenance

### Local Variable Inference

Local bindings may omit an explicit type when the compiler can infer it from the initializer.

Examples:

```ferox
let count = 42;
let name = "ana";
let pair = (1, true);
```

This is allowed because the initializer provides the type information directly.

Local bindings may also use explicit annotations:

```ferox
let retries: Int = 3;
```

Rules:

* local `let` bindings may use type inference
* inference is based on the initializer and local context
* if inference would be ambiguous, the compiler must require an explicit type
* type inference does not override the rule against implicit conversions

### Function Signatures

Function parameters must be explicitly typed.

Return types in function declarations are also explicit in the initial design.

Example:

```ferox
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

Rules:

* parameter types are always explicit
* declared function return types are explicit
* the function body is checked against the declared return type
* Ferox does not use inferred parameter types in function declarations

### Public APIs

All public APIs must use explicit types.

Examples of public API surface:

* `pub fn`
* `pub trait`
* public methods in public traits
* public type declarations and their exposed fields if field visibility is later introduced

Example:

```ferox
pub fn parse(text: String) -> Result[Config, ParseError] {
    parse_config(text)
}
```

Rules:

* public signatures must not rely on inferred parameter or return types
* public API types should remain stable and visible in source
* Ferox prefers explicitness at module boundaries over shorter declarations

### Design Intent

Inference should help inside implementation code, not erase structure at API boundaries.

Goals:

* keep local code concise
* keep public interfaces explicit
* improve diagnostics by reducing hidden type decisions
* preserve refactor safety across module boundaries

## ADTs and Enum Layout

Ferox uses `struct` and `enum` as the primary algebraic data type forms in v1.

Design rules:

* data layout must be explicit in source
* enum variants are tagged by default
* invalid enum states must be unrepresentable in safe code
* ADT forms should remain small and regular

### Structs

Structs define product types with named fields.

Example:

```ferox
struct User {
    id: Int,
    name: String,
}
```

Rules:

* struct fields are named
* field types are explicit
* field order is part of the source definition
* structs do not imply inheritance or subtyping

### Enums

Enums define tagged sum types.

Example:

```ferox
enum Token {
    number(Int),
    text(String),
    end,
}
```

Rules:

* each enum value belongs to exactly one declared variant
* the active variant tag is part of the value representation
* enum construction is explicit through variant names
* safe code cannot create a value with an invalid or unknown variant tag

### Supported Variant Forms

The initial enum model supports:

* unit-like variants
* tuple-like variants
* named-field variants

Unit-like variant:

```ferox
enum Mode {
    read,
    write,
}
```

Tuple-like variant:

```ferox
enum Message {
    text(String),
    binary(Bytes),
}
```

Named-field variant:

```ferox
enum Event {
    connected { user: User, at: Time },
    disconnected { user: User, at: Time },
}
```

Rules:

* variant shape is explicit in the enum definition
* all values of a given variant follow that variant's declared field layout
* variant layouts may differ across variants within the same enum

### Option and Result

`Option[T]` and `Result[T, E]` are standard tagged data types in the language model.

Canonical forms:

```ferox
some(value)
none
ok(value)
err(error)
```

These are ordinary tagged forms from the language user's point of view, even if the implementation later applies optimized layouts.

### Layout Policy

The language-level guarantee is semantic, not byte-level.

This means:

* the source definition determines the logical shape of the type
* the compiler may optimize physical layout
* such optimization must not change observable safe-language behavior
* the spec does not promise a stable ABI layout for all ADTs in v1

### Design Intent

Ferox's ADT model should make invalid states hard or impossible to represent while staying easy to inspect and transform.

Goals:

* make sum and product types explicit
* keep variant shape obvious in source
* support exhaustive matching cleanly
* preserve room for compiler optimization without exposing layout tricks as source-level complexity

## Pattern Destructuring

Ferox supports destructuring patterns for structs, enums, tuples, `Option`, and `Result`.

Design rules:

* destructuring follows the declared shape of the type
* bindings introduced by destructuring are explicit and local to the pattern arm
* destructuring syntax should mirror construction syntax where practical
* pattern destructuring must not bypass exhaustiveness checking

### Tuple Destructuring

Tuple values are destructured positionally.

Example:

```ferox
match pair {
    (left, right) => left + right,
}
```

Rules:

* tuple positions are matched in order
* tuple arity must match the scrutinee type
* `_` may be used to ignore elements

Example:

```ferox
match pair {
    (left, _) => left,
}
```

### Struct Destructuring

Struct values are destructured by field name.

Example:

```ferox
match user {
    User { id, name } => name,
}
```

Rules:

* struct fields are matched by name, not by declaration order
* field names may bind directly to local names
* `_` may be used to ignore a field value
* omitted fields are not implicitly ignored in the initial design

Example:

```ferox
match user {
    User { id, name: username } => username,
}
```

### Enum Destructuring

Enum values are destructured according to their variant form.

Tuple-like variant:

```ferox
match message {
    text(value) => value,
    binary(bytes) => decode(bytes),
}
```

Named-field variant:

```ferox
match event {
    connected { user, at } => user.name,
    disconnected { user, at } => user.name,
}
```

Unit-like variant:

```ferox
match mode {
    read => open_read(),
    write => open_write(),
}
```

Rules:

* tuple-like variants destructure positionally
* named-field variants destructure by field name
* unit-like variants bind no inner values
* pattern shape must match the declared variant shape

### Option and Result Destructuring

`Option` and `Result` use their canonical tagged forms in patterns.

Examples:

```ferox
match value {
    some(inner) => inner,
    none => fallback,
}
```

```ferox
match result {
    ok(value) => value,
    err(error) => handle(error),
}
```

Rules:

* `some(...)` and `none` are the canonical `Option` patterns
* `ok(...)` and `err(...)` are the canonical `Result` patterns
* these forms participate in ordinary exhaustiveness checking

### Nested Destructuring

Patterns may nest when the inner structure is explicit.

Example:

```ferox
match response {
    ok(some(User { name, id })) => name,
    ok(none) => "missing",
    err(error) => "error",
}
```

Rules:

* nested destructuring is allowed
* each nested layer must follow the declared type shape
* nested patterns do not weaken exhaustiveness requirements

### Design Intent

Destructuring should make structured data easy to unpack without introducing hidden control flow or ambiguous binding behavior.

Goals:

* keep patterns close to type shape
* support readable case handling
* improve refactor safety for ADTs
* keep compiler analysis straightforward
