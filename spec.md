# Tark Language Design Decisions (v0.3)

## Language Name

The language is called **Tark**.

Status:

* selected as the project name
* should still pass trademark and brand-conflict review before becoming final for public release

The name is treated as a standalone project name.

This reflects the philosophy of the language:

* correctness-first software
* strong logical foundations
* high-trust systems
* compiler-enforced guarantees

Example CLI:

tark build
tark run
tark fmt
tark test

Tentative file extension:

`.tark`

Status:

* **Not yet approved as a final decision**
* `.ax` was rejected as the preferred extension
* `.tark` is the current preferred extension because it is explicit and low-ambiguity
* avoid short extensions that are likely to create domain or file-format confusion

Pre-finalization checks required:

* trademark search in target jurisdictions
* product and company-name conflict review in developer tools and software markets
* domain strategy review for `tark.*`
* source file extension review, including collision risk, searchability, editor support, and confusion with internet domains

---

# Vision

Tark is a compiled programming language designed for **high-trust software** where correctness, analyzability, and predictable performance are prioritized over convenience.

The language is optimized for **both humans and coding agents** to read, reason about, and safely evolve codebases.

Core idea:

> Build fast software that both humans and agents can maintain safely over long periods.

---

# Priority Hierarchy

When tradeoffs occur, the language prioritizes:

1. **Safety > fewer lines of code**
2. **Agentic-use > human readability**
3. **Performance > elegance**
4. **Compiler guarantees > ease of learning**

Secondary rule:

> When possible, preserve both sides of the tradeoff without compromising the higher priority.

---

# Core Identity

Tark is:

* Compiled
* Statically typed
* Ownership-based
* Safe by default
* Performance-transparent
* Agent-friendly
* Tooling-first

Target domains:

* Backends
* Infrastructure systems
* CLIs
* Data systems
* Services
* Long-lived codebases
* Agent-assisted development environments

Non-goals:

* Being the simplest beginner language
* Replacing C for extremely low-level systems work
* Being a purely functional language
* Being a scripting language
* Supporting many paradigms equally

---

# Memory Model

Tark uses:

**Ownership + simplified borrowing + regions**

instead of a full Rust-style borrow checker.

Goals:

* Deterministic memory management
* No garbage collector required
* Prevent use-after-free
* Prevent double-free
* Prevent data races in safe code
* Keep the reasoning model smaller than Rust

Key characteristics:

* Every value has an **owner**
* Moves are the default for non-copy types
* Copies must be explicit or restricted to copy-safe types
* Borrowing is allowed but simplified
* Borrowing is strictly lexical in v1
* Complex long-distance borrowing patterns are discouraged
* **Regions / arenas** exist as first-class constructs for grouped allocation
* References tied to a region cannot escape it unsafely
* Shared ownership must be explicit

## Borrowing Scope Policy

Tark uses a strictly lexical borrowing model in v1.

This means:

* borrow scopes are intentionally local and predictable
* some valid-but-complex Rust borrowing patterns may be rejected
* Tark prefers simpler user reasoning over maximum borrow expressiveness

Rationale:

* keeps the memory model smaller than Rust
* makes compiler behavior easier to predict
* supports Tark's explicitness and analyzability goals

## Move Semantics

For non-copy types, assignment, argument passing, and return move ownership by default.

Examples:

```tark
let a = user;
let b = a;
```

After the move into `b`, `a` may no longer be used unless the type is copy-safe.

Argument passing also moves by default:

```tark
fn save(user: User) -> Unit {
    persist(user);
}

let user = load_user();
save(user);
```

After `save(user);`, the original binding is considered moved.

Rules:

* Non-copy values move on assignment
* Non-copy values move when passed by value to a function
* Non-copy values move when returned from a function
* Using a moved value in safe code is rejected
* Borrowing must be explicit when ownership transfer is not intended

Goals:

* make ownership transfer visible
* prevent accidental implicit copying of expensive values
* preserve predictable runtime behavior
* keep the ownership model teachable

---

# Safety Guarantees

Safe Tark code is intended to guarantee the following within the boundaries of the safe language subset:

* Use-after-free
* Double free
* Null dereference
* Data races
* Uninitialized reads
* Invalid enum states created through safe construction paths

These guarantees apply only after the ownership, borrowing, mutation, and effects rules are fully specified and implemented.

This section does not claim:

* semantic correctness of business logic
* prevention of all invalid program states
* safety for `unsafe` code

Principle:

> If safe Tark code compiles, it should satisfy a small, explicit, and mechanically enforced set of structural safety guarantees.

Unsafe code exists but must be explicit and isolated.

---

# Type System

Tark uses:

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

---

# Functional Programming Influence

Functional programming influences Tark, but it is **not a purely functional language**.

Included FP ideas:

* Immutability by default
* Pure functions encouraged
* Algebraic data types
* Pattern matching
* Expression-oriented constructs
* Exhaustive match checking
* Explicit effect boundaries

Not adopted from FP:

* Lazy evaluation by default
* Purity absolutism
* Heavy monadic ceremony
* Complex type-level programming
* Higher-kinded types (initial versions)

Philosophy:

> Functional defaults with practical systems programming capabilities.

---

# Mutation Model

* Immutable bindings by default
* Mutation must be explicit
* Mutable access requires exclusivity
* Shared mutation requires explicit synchronization or special constructs
* Mutation should remain local whenever possible

---

# Effects Model

Side effects should be **visible and explicit**.

Examples of effects:

* IO
* Async operations
* Unsafe operations
* Global/state mutation

Goals:

* Improve reasoning
* Improve testing
* Improve compiler analysis
* Improve agentic refactoring

---

# Concurrency

Goals:

* Safe concurrency by default
* Structured concurrency
* No data races in safe code
* Explicit async behavior
* Cancellation-friendly execution model

---

# Performance Philosophy

Tark favors **predictable performance**.

Design principles:

* Runtime costs should be visible in the source
* Allocations should not be hidden
* Copies and moves should be understandable
* Async behavior should be explicit
* No hidden global runtime

Performance model goals:

* Low memory overhead
* Native performance
* Deterministic destruction
* Efficient concurrency

---

# Agentic Design Principles

Tark is designed to be **machine-friendly**.

Key design goals:

* Small grammar
* Low syntax ambiguity
* Canonical formatting
* Highly regular syntax
* Explicit imports and exports
* Predictable AST structure
* Machine-readable diagnostics

The language should be easy for agents to:

* Parse
* Transform
* Refactor
* Verify
* Repair

---

# Formatting Policy

Tark defines a **single canonical source format**.

Canonical formatting is part of the standard workflow, but formatting is not a prerequisite for executing otherwise valid code.

If a file is syntactically valid but **not canonically formatted**:

* `tark build`
* `tark run`
* `tark test`

must still operate on the program normally.

Formatting compliance is enforced by:

* `tark fmt`
* `tark fmt --check`
* lint or CI policy

Formatting is enforced by the official formatter, not by making ordinary execution commands reject valid code.

Example command:

tark fmt

Goals:

* eliminate style ambiguity
* improve agentic reliability
* reduce review noise
* stabilize refactoring and code generation
* ensure uniform codebases

---

# Tooling Philosophy

A single official toolchain exists.

CLI:

tark

Responsibilities:

* build
* run
* test
* format
* lint
* benchmark
* documentation generation
* dependency management
* package publishing

Goals:

* Excellent diagnostics
* Machine-readable compiler output
* Deterministic builds
* First-class testing support

---

# Error Handling

Errors are handled through types.

Recoverable errors:

Result

Absence of value:

Option

Unexpected fatal conditions:

panic / crash

Exceptions are **not used** for standard control flow.

---

# Syntax Philosophy

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

Arbitrary syntax macros are discouraged or forbidden.

---

# Core Syntax

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

Syntax shape:

```tark
fn name(param: Type, param: Type) -> Type {
    expression
}
```

Examples:

```tark
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn is_even(value: Int) -> Bool {
    value % 2 == 0
}
```

Rules:

* Parameters are separated by commas
* Each parameter has an explicit type
* Return types use `->`
* A function body is always a block
* The final expression is the return value unless `return` is used explicitly

## Bindings

Bindings use `let`.

Immutable binding:

```tark
let port = 8080;
```

Mutable binding:

```tark
let mut total = 0;
```

Rules:

* Bindings are immutable by default
* Mutation requires `mut`
* Binding statements end with `;`
* Type inference is allowed for local bindings
* Explicit local type annotations are allowed

Typed local binding:

```tark
let retries: Int = 3;
```

## Blocks

Blocks use braces and may contain zero or more statements followed by an optional final expression.

Example:

```tark
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

```tark
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

```tark
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

```tark
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

## Semicolon Rule

Semicolons determine whether a construct contributes a value.

```tark
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

This rule is part of Tark's core readability and analyzability model.

## Modules, Imports, Exports, and Visibility

Tark uses a file-and-module model with explicit imports and explicit public exports.

Design rules:

* Each source file defines one module
* Module boundaries follow the file system layout
* Imports are always explicit
* Visibility is private by default
* Public API surface must be marked explicitly

### Modules

A source file corresponds to a module named from its file.

Examples:

* `math.tark` defines module `math`
* `net/http.tark` defines module `net.http`
* `main.tark` may define the entry module for an executable package

Tark does not require a `module` declaration in the file in the initial design.

Rationale:

* reduces boilerplate
* keeps the AST predictable
* makes the module graph follow the repository layout directly

### Imports

Imports use `use`.

Import a single item:

```tark
use net.http.Client;
```

Import a module namespace:

```tark
use net.http;
```

Import with alias:

```tark
use net.http.Client as HttpClient;
```

Grouped import:

```tark
use net.http.{Client, Request, Response};
```

Rules:

* Import paths are dot-separated
* Imports end with `;`
* Aliases use `as`
* Grouped imports are allowed for multiple items from the same module
* Wildcard imports are forbidden in the initial design

### Exports

Public symbols are exported by marking their declarations with `pub`.

Examples:

```tark
pub fn parse(input: String) -> Result<Value, ParseError> {
    parse_impl(input)
}

fn parse_impl(input: String) -> Result<Value, ParseError> {
    todo();
}
```

Rules:

* `pub` exports a declaration from its defining module
* Unmarked declarations remain private to the module
* Export lists are not used in the initial design
* Public API is declaration-based, not file-footer-based

### Visibility

Initial visibility levels:

* private
* public

Private declaration:

```tark
fn normalize(value: String) -> String {
    value
}
```

Public declaration:

```tark
pub fn parse_config(text: String) -> Result<Config, ConfigError> {
    build_config(text)
}
```

Rules:

* Visibility is private by default
* `pub` is the only visibility modifier in the initial design
* More granular visibility such as package-scoped or module-subtree-scoped access is deferred

### Imported Name Usage

Imported names are referenced directly by their local imported name.

Examples:

```tark
use core.io.Writer;
use core.hash.HashMap as Map;

fn write_all(writer: Writer, values: Map<String, Int>) -> Unit {
    writer.flush();
}
```

Module-qualified references are also allowed when importing a namespace:

```tark
use net.http;

fn new_client() -> http.Client {
    http.Client::new()
}
```

### Initial Non-Goals

The initial module system does not include:

* wildcard imports
* implicit re-exports
* package-private visibility
* cyclic import support as a design goal
* runtime module loading

## Traits and Implementations

Tark uses `trait` as the initial surface syntax for protocol-like behavior.

Design rules:

* Traits define required behavior
* Types implement traits explicitly
* Inheritance is not part of the model
* Trait conformance is declaration-based, not structural by default
* Visibility follows the same `pub` rules as other declarations

### Trait Declarations

Traits use the `trait` keyword.

Example:

```tark
pub trait Display {
    fn render(self) -> String;
}
```

Traits may also provide default method bodies.

```tark
pub trait Named {
    fn name(self) -> String;

    fn is_empty(self) -> Bool {
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

```tark
impl Display for User {
    fn render(self) -> String {
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

The initial syntax reserves `self` as the receiver parameter in trait and impl methods.

Example:

```tark
trait Len {
    fn len(self) -> Int;
}
```

Rules:

* `self` may appear as the first parameter of a trait method or trait implementation method
* Receiver ownership and borrowing variants will be refined in the memory model sections
* The surface syntax keeps method declarations compact while deferring exact receiver semantics

### Associated Functions

Traits may declare functions that do not take `self`.

Example:

```tark
trait Parser {
    fn parse(input: String) -> Result<Value, ParseError>;
}
```

These are associated functions, not instance methods.

### Visibility

Traits and impl methods follow normal declaration visibility rules.

Examples:

```tark
pub trait Write {
    fn write(self, bytes: Bytes) -> Result<Unit, IoError>;
}

impl Write for File {
    fn write(self, bytes: Bytes) -> Result<Unit, IoError> {
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

These may be considered later, but they are intentionally excluded from the initial design to keep the trait model small and analyzable.

## Generic Types and Generic Functions

Tark uses square brackets for generic parameter lists and generic type arguments.

Design rules:

* Generic syntax must remain visually distinct from function call syntax
* Generic parameter and argument syntax should be uniform across types, functions, and impl blocks
* Generic use sites may omit explicit type arguments when inference is sufficient
* Advanced generic features are deferred until the core type system is stable

### Generic Functions

Generic functions place type parameters after the function name.

Example:

```tark
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

```tark
let a = identity(1);
let b = identity[Int](1);
```

### Generic Types

Generic type declarations place type parameters after the type name using the same bracket syntax.

Example:

```tark
struct Pair[A, B] {
    left: A,
    right: B,
}
```

Type application uses the same form:

```tark
let item: Pair[String, Int] = Pair::new("x", 1);
```

Rules:

* Generic type parameters are ordered and positional
* Type arguments use the same `[]` syntax as type parameters
* Nested type arguments are allowed

Example:

```tark
let result: Result[Option[String], ParseError];
```

### Generic Implementations

Generic `impl` blocks place type parameters after `impl`.

Example:

```tark
impl[T] Display for Box[T] {
    fn render(self) -> String {
        format_box(self)
    }
}
```

Rules:

* Generic parameters introduced by an `impl` are in scope for the entire impl block
* The target type may use those parameters directly
* Constraint syntax for generic implementations is deferred

### Generic Argument Inference

Tark allows generic argument inference when the compiler can determine the type arguments from:

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
* inline trait-bound syntax
* `where` clauses

These may be added later, but they are intentionally excluded from the initial generic surface.

---

# Implementation Strategy

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

---

# Strategic Positioning

Tark aims to occupy a space between:

**Go and Rust**

and between:

**Zig and Rust**

Positioning:

* Safer and more correct than Go
* Simpler and more coherent than Rust
* More maintainable and structured than Zig
* Better suited for agent-assisted development than all three

Core promise:

> Fast software with strong correctness guarantees that both humans and agents can evolve safely.
