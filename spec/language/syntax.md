# Ferox Language Specification: Core Syntax

This document defines Ferox's v1 syntax philosophy and initial core surface syntax.

## Syntax Philosophy

Syntax should prioritize:

* Regularity
* Explicitness
* Readability
* Predictability

Expected characteristics:

* Small grammar
* Few keywords
* Expression-oriented constructs
* Minimal magic
* Consistent formatting

Ferox v1 does not include user-defined syntax macros.

---

## Core Syntax

This section defines the initial surface syntax for:

* functions
* bindings
* blocks
* expressions
* statements

Design rules:

* Curly braces delimit blocks
* Semicolons terminate statements
* Blocks are expressions
* The final expression in a block becomes the block value if it has no semicolon
* Public APIs use explicit parameter and return types

## Functions

Function declarations use `fn`.

Suspendable functions use `async fn`.

Syntax shape:

```ferox
fn name(param: Type, param: Type) -> Type {
    expression
}
```

Examples:

```ferox
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn is_even(value: Int) -> Bool {
    value % 2 == 0
}

async fn fetch(url: String) -> Response {
    ...
}
```

Rules:

* Parameters are separated by commas
* Each parameter has an explicit type
* Return types use `->`
* A function body is always a block
* The final expression is the return value unless `return` is used explicitly
* `async fn` uses the same basic declaration shape as `fn`, but marks a suspendable function

## Bindings

Bindings use `let`.

Immutable binding:

```ferox
let port = 8080;
```

Mutable binding:

```ferox
let mut total = 0;
```

Rules:

* Bindings are immutable by default
* Mutation requires `mut`
* Binding statements end with `;`
* Type inference is allowed for local bindings
* Explicit local type annotations are allowed

Typed local binding:

```ferox
let retries: Int = 3;
```

## Blocks

Blocks use braces and may contain zero or more statements followed by an optional final expression.

Example:

```ferox
let result = {
    let base = 40;
    let extra = 2;
    base + extra
};
```

Rules:

* A block creates a new scope
* Names declared inside a block do not escape it unless returned through the block value
* A trailing expression without `;` becomes the value of the block
* Adding `;` discards the value and makes the block evaluate to unit

## Expressions

Initial core expression forms:

* literals
* name references
* grouped expressions
* unary expressions
* binary expressions
* function calls
* block expressions
* `if` expressions

Examples:

```ferox
42
ready
(a + b)
-count
left + right
hash(value)
if valid { 1 } else { 0 }
```

Operator expressions are expression-oriented and produce values.

## Statements

Initial core statement forms:

* `let` binding statements
* assignment statements
* `return` statements
* expression statements

Examples:

```ferox
let value = 1;
total = total + value;
log(total);
return total;
```

Rules:

* Statements inside a block end with `;`
* Expression statements are allowed when the value is intentionally discarded
* `return` exits the current function immediately
* Assignment is a statement, not an expression

## If Expressions

`if` is an expression, not only a statement.

Example:

```ferox
let label = if score >= 90 {
    "great"
} else {
    "ok"
};
```

Rules:

* Conditions must evaluate to `Bool`
* Both branches must produce compatible types when the `if` expression is used as a value
* `else if` chains are allowed as nested `if` expressions

## Match Expressions

Ferox uses `match` for pattern matching.

`match` is an expression.

Basic shape:

```ferox
match value {
    pattern => expression,
    pattern => expression,
}
```

Block arms are also allowed:

```ferox
match value {
    pattern => {
        statement;
        expression
    },
    _ => fallback,
}
```

Design rules:

* `match` must be exhaustive
* each arm uses `=>`
* arm separators use `,`
* arms may evaluate to expressions or blocks
* when used as a value, all arms must produce compatible types

### Supported Initial Pattern Forms

The initial pattern set includes:

* wildcard pattern: `_`
* name binding pattern: `name`
* literal pattern
* enum/variant pattern
* tuple pattern

Examples:

```ferox
match result {
    ok(value) => value,
    err(error) => handle(error),
}
```

```ferox
match option {
    some(value) => value,
    none => default_value,
}
```

```ferox
match pair {
    (left, right) => left + right,
}
```

### Binding Behavior

Patterns may introduce new bindings.

Example:

```ferox
let message = match event {
    connected(user) => user.name,
    disconnected(user) => user.name,
};
```

Bindings introduced by a pattern are scoped to that arm only.

### Exhaustiveness

Match expressions must cover all possible cases.

Examples:

```ferox
match option {
    some(value) => value,
    none => fallback,
}
```

This is exhaustive.

```ferox
match option {
    some(value) => value,
}
```

This is rejected because `none` is not covered.

Wildcard arms may be used to make a match exhaustive:

```ferox
match token {
    number(value) => value,
    _ => 0,
}
```

### Ordering

Arms are checked in source order.

Rules:

* the first matching arm is selected
* unreachable arms should be diagnosed by the compiler
* wildcard arms should normally appear last

### Design Intent

Pattern matching is part of Ferox's invalid-state modeling story.

Goals:

* encourage explicit handling of all cases
* make enum and result handling mechanical
* improve refactor safety
* keep control flow analyzable

## Literal Syntax

Ferox defines a small and regular set of literal forms in v1.

Initial literal categories:

* integer literals
* floating-point literals
* boolean literals
* string literals
* tuple literals
* array literals
* map literals
* enum variant literals

### Integer Literals

Basic decimal integers:

```ferox
0
1
42
1000000
```

Readable separators are allowed:

```ferox
1_000_000
```

### Floating-Point Literals

Examples:

```ferox
0.0
3.14
10.5
1_000.25
```

### Boolean Literals

Boolean literals are:

```ferox
true
false
```

### String Literals

Double-quoted strings:

```ferox
"hello"
"config.toml"
"line\nbreak"
```

Initial rules:

* string literals use double quotes
* common escape sequences are supported
* single-quoted strings are reserved for character literals if introduced later

### Tuple Literals

Examples:

```ferox
(1, 2)
("x", 10, true)
```

Rules:

* tuple elements are comma-separated
* parentheses distinguish tuples from other collection forms

### Array Literals

Examples:

```ferox
[]
[1, 2, 3]
["a", "b", "c"]
```

Rules:

* array literals use square brackets
* elements are comma-separated
* element types must be compatible

### Map Literals

Examples:

```ferox
{}
{"host": "localhost", "port": 5432}
```

Rules:

* map literals use braces
* key-value pairs use `:`
* pairs are comma-separated
* key and value types must be compatible within the map

### Enum Variant Literals

Enum variants are constructed by naming the variant directly.

Examples:

```ferox
some(42)
none
ok(value)
err("failed")
```

Rules:

* unit-like variants use the variant name alone
* value-carrying variants use function-like construction syntax
* variant construction is explicit and regular

### Design Intent

Literal syntax should stay easy to parse, easy to generate, and easy to read.

Goals:

* keep the literal surface small
* avoid many special-case forms
* support predictable parsing
* make generated code stable and uniform

## Semicolon Rule

Semicolons determine whether a construct contributes a value.

```ferox
fn answer() -> Int {
    42
}

fn ignore() -> Unit {
    42;
}
```

Interpretation:

* `42` without `;` is the return value of the block
* `42;` is a statement whose value is discarded

This rule is part of Ferox's core readability and analyzability model.
