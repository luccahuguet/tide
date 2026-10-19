# Ferox Language Specification: Traits and Generics

This document defines Ferox's v1 trait and generic-programming surface.

## Traits and Implementations

Ferox uses `trait` as the initial surface syntax for protocol-like behavior.

Design rules:

* Traits define required behavior
* Types implement traits explicitly
* Inheritance is not part of the model
* Trait conformance is declaration-based, not structural by default
* Visibility follows the same `pub` rules as other declarations

### Trait Declarations

Traits use the `trait` keyword.

Example:

```ferox
pub trait Display {
    fn render(&self) -> String;
}
```

Traits may also provide default method bodies.

```ferox
pub trait Named {
    fn name(&self) -> String;

    fn is_empty(&self) -> Bool {
        self.name() == ""
    }
}
```

Rules:

* A trait body contains method declarations
* Required methods end with `;`
* Default methods provide a block body
* Traits do not declare stored fields
* Trait items are ordered explicitly in source and preserved by formatting

### Implementations

Trait implementations use `impl Trait for Type`.

Example:

```ferox
impl Display for User {
    fn render(&self) -> String {
        self.username
    }
}
```

Rules:

* An implementation must name both the trait and the target type explicitly
* All required trait methods must be implemented unless a default body exists in the trait
* Implementations may override default trait methods
* Methods defined in an `impl` block must match the trait signature

### Receiver Syntax

Ferox v1 supports exactly three method receiver forms.

Example:

```ferox
trait Len {
    fn len(&self) -> Int;
}
```

Rules:

* the first parameter of a trait or impl method may be `self`, `&self`, or `&mut self`
* `self` consumes ownership of the receiver
* `&self` provides shared borrowed access to the receiver
* `&mut self` provides exclusive mutable access to the receiver
* trait methods and impl methods must match receiver form exactly
* v1 does not support additional receiver forms or receiver polymorphism

### Associated Functions

Traits may declare functions that do not take `self`.

Example:

```ferox
trait Parser {
    fn parse(input: String) -> Result<Value, ParseError>;
}
```

These are associated functions, not instance methods.

### Associated Types

Associated types are **not supported** in the initial version of Ferox.

Decision:

* initial Ferox traits do not declare associated types
* initial Ferox impls do not define associated type bindings
* generic parameters should be used instead where a relationship must be expressed in v1

Example of the preferred v1 style:

```ferox
trait Parser[T] {
    fn parse(input: String) -> Result[T, ParseError>;
}
```

instead of an associated-type style such as:

```ferox
trait Parser {
    type Output;
    fn parse(input: String) -> Result[Output, ParseError>;
}
```

Rationale:

* keeps the trait model smaller
* avoids adding another layer of generic reasoning in v1
* reduces interaction complexity between traits, bounds, and inference
* fits Ferox's goal of staying smaller and more predictable than Rust

### Visibility

Traits and impl methods follow normal declaration visibility rules.

Examples:

```ferox
pub trait Write {
    fn write(&mut self, bytes: Bytes) -> Result<Unit, IoError>;
}

impl Write for File {
    fn write(&mut self, bytes: Bytes) -> Result<Unit, IoError> {
        write_file(self, bytes)
    }
}
```

Rules:

* `pub trait` exports the trait from its module
* Trait methods inside a public trait are part of the public interface of that trait
* `impl` blocks do not use a separate visibility modifier in the initial design

### Initial Non-Goals

The initial trait system does not include:

* trait inheritance
* specialization
* blanket implementations
* negative implementations
* associated types
* operator overloading via traits as a design priority
* structural typing as the default conformance model
* receiver forms beyond `self`, `&self`, and `&mut self`

These may be considered later, but they are intentionally excluded from the initial design to keep the trait model small and analyzable.

## Generic Types and Generic Functions

Ferox uses square brackets for generic parameter lists and generic type arguments.

Design rules:

* Generic syntax must remain visually distinct from function call syntax
* Generic parameter and argument syntax should be uniform across types, functions, and impl blocks
* Generic use sites may omit explicit type arguments when inference is sufficient
* Advanced generic features are deferred until the core type system is stable

### Generic Functions

Generic functions place type parameters after the function name.

Example:

```ferox
fn identity[T](value: T) -> T {
    value
}

fn pair[A, B](left: A, right: B) -> Pair[A, B] {
    Pair::new(left, right)
}
```

Rules:

* Generic parameters use `[]`
* Parameters are comma-separated
* Generic parameters are part of the function signature
* Generic functions may be called with inferred or explicit type arguments

Examples:

```ferox
let a = identity(1);
let b = identity[Int](1);
```

### Generic Types

Generic type declarations place type parameters after the type name using the same bracket syntax.

Example:

```ferox
struct Pair[A, B] {
    left: A,
    right: B,
}
```

Type application uses the same form:

```ferox
let item: Pair[String, Int] = Pair::new("x", 1);
```

Rules:

* Generic type parameters are ordered and positional
* Type arguments use the same `[]` syntax as type parameters
* Nested type arguments are allowed

Example:

```ferox
let result: Result[Option[String], ParseError];
```

### Generic Implementations

Generic `impl` blocks place type parameters after `impl`.

Example:

```ferox
impl[T] Display for Box[T] {
    fn render(self) -> String {
        format_box(self)
    }
}
```

Rules:

* Generic parameters introduced by an `impl` are in scope for the entire impl block
* The target type may use those parameters directly

### Trait Bounds and Constraints

Ferox uses inline trait bounds in generic parameter lists.

Basic shape:

```ferox
fn name[T: Trait](value: T) -> T {
    value
}
```

Multiple bounds use `+`:

```ferox
fn render[T: Display + Debug](value: T) -> String {
    value.render()
}
```

Design rules:

* bounds are written next to the constrained type parameter
* bounds use `:`
* multiple bounds use `+`
* the same surface form is used for functions, types, traits, and impl blocks where applicable

### Generic Function Bounds

Example:

```ferox
fn max[T: Ord](left: T, right: T) -> T {
    if left >= right { left } else { right }
}
```

Rules:

* constrained parameters remain part of the generic parameter list
* bounds restrict which types may satisfy the parameter
* function bodies may rely on the operations guaranteed by the stated bounds

### Generic Type Bounds

Type declarations may also constrain their parameters inline.

Example:

```ferox
struct SortedSet[T: Ord] {
    values: List[T],
}
```

Rules:

* type-level bounds constrain valid instantiations of the generic type
* bounds on type declarations are part of the public type definition

### Generic Impl Bounds

Bounds may appear on generic `impl` blocks.

Example:

```ferox
impl[T: Display] Display for Box[T] {
    fn render(self) -> String {
        self.value.render()
    }
}
```

Rules:

* impl bounds restrict when the implementation applies
* bounds must be explicit in the impl header
* Ferox does not use implicit blanket applicability without stated constraints

### Design Intent

Constraint syntax should stay small and local.

Goals:

* keep constraints near the parameter they affect
* avoid introducing multiple equivalent constraint syntaxes in v1
* make generic requirements visible in public declarations

### Generic Argument Inference

Ferox allows generic argument inference when the compiler can determine the type arguments from:

* function arguments
* expected result types
* local type context

Explicit type arguments remain available when inference would be ambiguous or when the programmer wants to pin a type.

### Initial Non-Goals

The initial generic syntax does not include:

* higher-kinded types
* default type parameters
* const generics
* variance annotations
* `where` clauses

These may be added later, but they are intentionally excluded from the initial generic surface.

---
