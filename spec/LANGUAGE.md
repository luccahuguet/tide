# Ferox Language Specification (v0.3)

This document contains the language-level rules for Ferox.

Current positioning:

* Ferox aims at high-trust software and safe change
* v1 should prefer lower semantic entropy over extra expressiveness
* the language should stay readable, mechanically analyzable, and friendly to fast tooling feedback

Related documents:

* `spec/VISION.md` for project goals and positioning
* `spec/TOOLING.md` for formatter, CLI, manifests, and package design
* `spec/IMPLEMENTATION.md` for bootstrap and self-hosting strategy

---

## Memory Model

Ferox uses:

**Ownership + simplified borrowing**

instead of a full Rust-style borrow checker.

Goals:

* Deterministic memory management
* No garbage collector required
* Prevent use-after-free
* Prevent double-free
* Prevent data races in safe code
* Keep the reasoning model smaller than Rust
* Preserve strong safe-code guarantees through a simpler and more restrictive model rather than through broader semantic machinery

Key characteristics:

* Every value has an **owner**
* Moves are the default for non-copy types
* Copies must be explicit or restricted to copy-safe types
* Borrowing is allowed but simplified
* Borrowing is strictly lexical in v1
* Complex long-distance borrowing patterns are discouraged
* Shared ownership must be explicit and secondary to the primary ownership path

## Borrowing Scope Policy

Ferox uses a strictly lexical borrowing model in v1.

This means:

* borrow scopes are intentionally local and predictable
* some valid-but-complex Rust borrowing patterns may be rejected
* Ferox prefers simpler user reasoning over maximum borrow expressiveness

Rationale:

* keeps the memory model smaller than Rust
* makes compiler behavior easier to predict
* supports Ferox's explicitness and analyzability goals
* supports the broader safe-change goal by keeping ownership reasoning local and mechanical

## Move Semantics

For non-copy types, assignment, argument passing, and return move ownership by default.

Examples:

```ferox
let a = user;
let b = a;
```

After the move into `b`, `a` may no longer be used unless the type is copy-safe.

Argument passing also moves by default:

```ferox
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

## Copy-Safe Types

Copy-safe types may be duplicated implicitly on assignment, argument passing, and return.

The initial v1 rule is intentionally narrow.

Built-in copy-safe categories:

* booleans
* characters
* integer types
* floating-point types
* unit

User-defined types are copy-safe only when they are explicitly declared as copy-safe and all stored fields are themselves copy-safe.

Example:

```ferox
copy struct Point {
    x: Int,
    y: Int,
}
```

This means:

```ferox
let a = Point { x: 1, y: 2 };
let b = a;
```

Both `a` and `b` remain valid because `Point` is copy-safe.

Non-copy examples in the initial model:

* strings
* heap-owning containers
* shared ownership types
* types with explicit destruction behavior

Rules:

* Copy-safe types copy implicitly
* Non-copy types move by default
* A user-defined type may only be marked `copy` if every stored field is copy-safe
* Types with ownership-sensitive resources may not be marked `copy`
* Copy-safety is part of a type's definition, not inferred ad hoc at each use site

## Explicit Copy Behavior

For values that are not implicitly copy-safe, duplication must be explicit.

Initial design direction:

* implicit copies are reserved for copy-safe types
* non-copy values require an explicit duplication operation
* the exact surface form for deep duplication may later be trait-backed, but it is never implicit

Example intent:

```ferox
let b = clone(a);
```

This expresses that duplication is a deliberate operation with visible cost.

Goals:

* keep cheap value copies ergonomic
* make expensive duplication visible in source
* prevent accidental performance regressions
* preserve the move-default mental model

## Core Semantic Abilities

Ferox should remain open to a **small built-in ability model** for core semantic properties if doing so reduces trait and marker-surface complexity.

Current design direction:

* this is a design exploration, not yet a fixed v1 commitment
* the most plausible use is for core properties such as copy-safety, destruction behavior, or concurrency-relevant capabilities
* Ferox should prefer built-in semantic categories over trait sprawl only when the result is simpler and more analyzable

Non-goal:

* inventing a large second abstraction system for ordinary user-defined behavior

## Borrowing Model

Ferox defines two safe borrow forms in v1:

* shared borrow
* exclusive mutable borrow

Borrow syntax:

```ferox
let r = &value;
let w = &mut value;
```

Shared borrows allow read access without taking ownership.

Example:

```ferox
let user = load_user();
let r1 = &user;
let r2 = &user;
show(r1);
show(r2);
```

Exclusive mutable borrows allow mutation without transferring ownership.

Example:

```ferox
let mut user = load_user();
let w = &mut user;
rename(w, "new-name");
```

## Borrowing Rules

The aliasing rule is:

* many readers or one writer

Concretely:

* many shared borrows may coexist
* at most one exclusive mutable borrow may exist at a time
* no shared borrow may overlap an exclusive mutable borrow of the same value
* while a value is borrowed mutably, the original binding may not be read or written directly
* while a value is borrowed, moving that value is rejected

Examples:

Allowed:

```ferox
let r1 = &user;
let r2 = &user;
```

Rejected:

```ferox
let r = &user;
let w = &mut user;
```

Rejected:

```ferox
let w1 = &mut user;
let w2 = &mut user;
```

## Lexical Borrow Scope

Borrow scopes are lexical in v1.

Example:

```ferox
let mut user = load_user();

{
    let w = &mut user;
    rename(w, "new-name");
}

save(user);
```

Here the mutable borrow ends when the inner block ends.

Rules:

* a borrow begins where the borrow expression is created
* a borrow ends at the end of its enclosing lexical scope
* Ferox does not attempt to accept every valid non-lexical borrowing pattern in v1
* shorter explicit scopes are the primary way to end borrows earlier

## Design Intent

This borrowing model is deliberately more restrictive than the most advanced borrow systems.

Goals:

* keep the core rule teachable
* make compiler behavior predictable
* reduce hidden lifetime reasoning
* support safe mutation without full Rust-level borrow complexity

## Arenas And Regions

Language-level arenas and broader region constructs are **deferred from the v1 core**.

Current v1 direction:

* Ferox should rely on ordinary ownership and lexical borrowing as the primary lifetime-management model
* Ferox should avoid adding a second first-class lifetime-management sublanguage unless later evidence proves it is necessary

Reason:

* arenas are useful, but they are not a small add-on
* once they exist, many later features must answer how they interact with arena-backed values and arena lifetimes
* deferring them keeps the v1 ownership story smaller and easier to reason about

This does not rule arenas out forever. It only removes them from the current v1 semantic core.

## Shared Ownership

Ferox allows shared ownership only through explicit standard-library primitives.

Shared ownership is not ordinary borrowing and is not the default ownership mode.

In v1, shared ownership should be treated as an explicit library-level escape hatch rather than as a co-equal primary ownership path.

Initial shared ownership primitives:

* `Shared[T]` for single-threaded shared ownership
* `SyncShared[T]` for synchronized shared ownership across concurrent contexts

Design rules:

* shared ownership must be visible in the type
* shared ownership carries explicit cost
* single-threaded sharing and synchronized sharing are distinct
* shared ownership does not weaken the borrowing rules for ordinary owned values

Example:

```ferox
let a = Shared::new(Config::load());
let b = a.share();
let c = a.share();
```

Here `a`, `b`, and `c` refer to the same underlying value through explicit shared ownership.

## Single-Threaded Shared Ownership

`Shared[T]` is intended for cases where multiple parts of a program need access to the same value within a single-threaded context.

Rules:

* `Shared[T]` permits multiple owners
* cloning or sharing a `Shared[T]` value is explicit
* `Shared[T]` does not imply mutable shared access
* mutable access through shared ownership requires additional explicit mechanisms, not ordinary borrowing

## Synchronized Shared Ownership

`SyncShared[T]` is intended for ownership shared across concurrent execution contexts.

Rules:

* synchronized shared ownership must be explicit in the type
* synchronization cost must not be hidden behind ordinary ownership or borrowing syntax
* concurrent mutable access requires explicit synchronization constructs
* Ferox does not treat synchronized sharing as equivalent to ordinary local ownership

Example intent:

```ferox
let state = SyncShared::new(State::new());
```

## Shared Ownership and Borrowing

Borrowing and shared ownership are separate mechanisms.

Rules:

* borrowing creates temporary access without changing ownership count
* shared ownership creates multiple owners explicitly
* ordinary `&value` and `&mut value` do not create shared ownership
* converting owned values into shared ownership must be explicit

## Initial Non-Goals for Shared Ownership

The initial shared ownership model does not include:

* implicit promotion from owned values to shared ownership
* automatic synchronized sharing
* interior mutability as a default property of shared ownership
* conflating borrowing, sharing, and synchronization into one mechanism

This keeps shared ownership explicit, visible, and distinct from the normal ownership fast path.

---

## Safety Guarantees

Safe Ferox code is intended to guarantee the following within the boundaries of the safe language subset:

* Use-after-free
* Double free
* Null dereference
* Data races
* Uninitialized reads
* Invalid enum states created through safe construction paths

These guarantees apply only after the ownership, borrowing, mutation, and concurrency rules are fully specified and implemented.

This section does not claim:

* semantic correctness of business logic
* prevention of all invalid program states
* safety for `unsafe` code

Principle:

> If safe Ferox code compiles, it should satisfy a small, explicit, and mechanically enforced set of structural safety guarantees.

Unsafe code exists but must be explicit and isolated.

---

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

---

## Mutation Model

* Immutable bindings by default
* Mutation must be explicit
* Mutable access requires exclusivity
* Shared mutation requires explicit synchronization or special constructs
* Mutation should remain local whenever possible

## Mutable Bindings

Local mutation is declared with `mut`.

Example:

```ferox
let mut total = 0;
total = total + 1;
```

Rules:

* bindings are immutable by default
* a binding must be declared with `mut` before it may be reassigned
* reassignment to an immutable binding is rejected
* `mut` belongs to the binding, not to the value's type

### Mutable Reassignment

Assignment is the basic mutation form for mutable bindings.

Example:

```ferox
let mut retries = 3;
retries = retries - 1;
```

This mutates the binding by assigning a new value.

### Mutable Access Through Borrowing

Mutation of an existing owned value should normally happen through exclusive mutable access.

Example:

```ferox
let mut user = load_user();
let writer = &mut user;
rename(writer, "new-name");
```

Rules:

* direct reassignment uses a `mut` binding
* in-place mutation of an existing value uses exclusive mutable access
* `&mut value` is the standard safe form for mutable access

### Parameters and Public APIs

Public APIs remain explicit about mutation.

Initial design direction:

* parameter types are explicit
* mutation through parameters should be visible through mutable access forms, not hidden implicit aliasing
* ordinary by-value parameters do not imply caller-visible mutation of the original binding

## Mutable Access Exclusivity

Exclusive mutable access is enforced conservatively in v1.

Core rule:

* at most one active mutable access path may target a given root value at a time

Concretely:

* creating `&mut value` blocks any overlapping shared or mutable borrow of that same value
* while an exclusive mutable access path is active, the original binding may not be read, written, or moved directly
* exclusivity is tracked lexically and ends when the mutable borrow's scope ends
* Ferox v1 does not attempt fine-grained safe-code borrow splitting for separate fields or elements of the same root value

Rejected intent:

```ferox
let mut pair = Pair { left: 1, right: 2 };
let a = &mut pair.left;
let b = &mut pair.right;
```

In v1 this is rejected, even though the fields are intuitively distinct, because both access paths originate from the same root value.

Allowed intent:

```ferox
let mut left = 1;
let mut right = 2;
let a = &mut left;
let b = &mut right;
```

This is allowed because the mutable borrows target different root bindings.

### Design Intent

The goal is to make mutable access rules obvious from source without requiring deep alias analysis.

Goals:

* keep exclusivity simple to teach
* reject overlap conservatively in safe code
* avoid hidden borrow-splitting rules in v1
* leave room for future refinement only if it clearly improves the language

## Shared Mutation and Synchronization

Shared mutation is not the default consequence of shared ownership.

Core rule:

* a shared owner may mutate underlying state only through an explicit mutation or synchronization wrapper

Initial standard-library direction:

* `Cell[T]` for small single-threaded replace-style shared mutation
* `RefCell[T]` for checked single-threaded interior mutation with dynamic exclusivity enforcement
* `Mutex[T]` for synchronized exclusive shared mutation across concurrent contexts
* `RwLock[T]` for synchronized shared-read / exclusive-write access across concurrent contexts

Examples of intended use:

```ferox
let counter = Shared::new(Cell::new(0));
counter.set(counter.get() + 1);
```

```ferox
let state = SyncShared::new(Mutex::new(State::new()));

let guard = state.lock();
guard.update();
```

Rules:

* `Shared[T]` alone does not permit shared mutation
* `SyncShared[T]` alone does not permit unsynchronized shared mutation
* mutable access through `Cell[T]`, `RefCell[T]`, `Mutex[T]`, or `RwLock[T]` is explicit in the type
* synchronization boundaries must be visible in source through wrapper types and guard-producing operations
* lock or guard lifetimes are lexical in v1 and end at scope exit
* ordinary `&mut value` borrowing remains the preferred fast path for local non-shared mutation

### Single-Threaded Shared Mutation

Single-threaded shared mutation is allowed only through explicit interior-mutation wrappers.

Initial direction:

* `Cell[T]` is for simple copy-like replacement and retrieval operations
* `RefCell[T]` is for cases that need temporary mutable access within one thread
* runtime exclusivity violations in `RefCell[T]` are treated as checked failures, not as permission to weaken safe borrowing everywhere else

### Concurrent Shared Mutation

Concurrent shared mutation requires explicit synchronization wrappers.

Initial direction:

* `Mutex[T]` provides one writer at a time
* `RwLock[T]` provides many readers or one writer under explicit synchronization
* synchronized access produces scoped guard values rather than raw unsynchronized aliases
* safe Ferox does not permit concurrent mutable access to ordinary owned values or plain shared owners without synchronization

### Initial Non-Goals

The initial shared-mutation model does not include:

* implicit interior mutability
* automatic locking
* lock-free shared mutation as the default model
* treating synchronized shared mutation as equivalent to ordinary local mutation

## Unsafe Capabilities

`unsafe` exists for operations that cannot be verified by the safe Ferox rules but are still necessary for systems programming and runtime implementation work.

Core rule:

* `unsafe` may bypass specific structural safety checks, but it does not disable parsing, type checking, or visibility rules

Initial operations permitted only in `unsafe` code:

* creating and dereferencing raw pointers
* pointer arithmetic
* calling functions explicitly marked `unsafe`
* calling foreign functions or interoperating with foreign memory representations
* unchecked casts or reinterpretation between layouts that the compiler cannot prove safe
* manual allocation, deallocation, and low-level memory initialization primitives
* constructing references or views from raw memory when the programmer is asserting validity manually

Example intent:

```ferox
unsafe {
    let p = raw_ptr(buffer);
    let q = p.offset(1);
    write_raw(q, 42);
}
```

Rules:

* entering `unsafe` does not make ordinary ownership and borrowing rules irrelevant; it only permits a defined escape hatch set
* values produced through `unsafe` may be used by safe code only if the unsafe code upholds Ferox's safety invariants
* `unsafe` is for low-level capability, not for bypassing ordinary API design constraints
* code that can be expressed with ordinary ownership, borrowing, or synchronized wrappers should stay in safe Ferox

### Initial Non-Goals

`unsafe` is not intended to permit:

* arbitrary suspension of the type system
* implicit weakening of visibility or module boundaries
* unchecked mutation through ordinary shared borrows
* using `unsafe` as a substitute for missing high-level language design

## Unsafe Isolation and Auditability

Unsafe code must remain easy to locate, review, and contain.

Initial forms:

* `unsafe { ... }` for a local unsafe block
* `unsafe fn` for a function with an explicit caller-side safety contract
* modules may contain unsafe code, but Ferox v1 does not provide a module-wide "everything here is unsafe" mode

### Unsafe Blocks

Unsafe blocks are the primary form for isolating unsafe operations.

Rules:

* the preferred unit of unsafe code is the smallest practical block
* only operations that require unsafe capability should be placed inside the block
* entering an unsafe block does not implicitly mark nested helper functions or surrounding scopes as unsafe

Example intent:

```ferox
fn read_header(bytes: &[Byte]) -> Header {
    unsafe {
        parse_header_unchecked(bytes)
    }
}
```

### Unsafe Functions

Unsafe functions expose a safety contract to their callers.

Rules:

* `unsafe fn` means the caller must satisfy explicit preconditions before calling the function
* calling an `unsafe fn` requires an `unsafe` context
* the body of an `unsafe fn` does not become implicitly unsafe; unsafe operations inside it should still use explicit `unsafe` blocks
* every `unsafe fn` should document the invariants the caller must uphold

This keeps the call boundary and the implementation boundary separately visible.

### Unsafe in Modules

Unsafe code may appear inside ordinary modules, but module boundaries do not suppress unsafe markers.

Rules:

* a module containing unsafe code remains an ordinary module for parsing, visibility, and import purposes
* unsafe code inside a module must still appear in explicit unsafe blocks or unsafe function signatures
* Ferox v1 does not allow a whole file or module to become implicitly unsafe
* review tooling should be able to identify unsafe blocks and unsafe functions by direct syntax search

### Audit Expectations

Unsafe code should be written so that audits can focus on a small number of explicit trust boundaries.

Initial expectations:

* every unsafe block should be narrow in scope
* every unsafe function should state its safety requirements in prose
* safe wrappers around unsafe internals should expose ordinary safe APIs whenever possible
* the standard library and core runtime should concentrate unsafe code in well-known low-level modules rather than spreading it through surface APIs

### Design Intent

Mutation syntax should make state change easy to see in source.

Goals:

* keep the default case immutable
* make local mutation straightforward
* keep state changes visible for humans and tools
* align mutation syntax with the ownership and borrowing model

---

## Operational Explicitness

Ferox still wants operations with real behavioral cost to be **visible and explicit** in v1.

However, Ferox v1 does **not** use a general effect-set system.

Instead, explicitness comes from ordinary language constructs:

* `async fn` and `await` for suspension
* `unsafe` blocks and `unsafe fn` for unsafe capability
* `Result` and `Option` for recoverable failure and absence
* `mut`, `&mut`, and synchronization wrappers for mutation and sharing boundaries
* standard-library API boundaries for external interaction

Rules:

* v1 does not attach general effect sets to expressions, blocks, or callable signatures
* v1 does not use a separate effect-propagation checker as part of the core language model
* external interaction should remain obvious in APIs even without a dedicated `io` effect annotation
* richer effect systems remain a post-v1 consideration

### Design Intent

The v1 language should preserve explicitness without adding a second major semantic subsystem on top of ownership.

Goals:

* keep behavioral boundaries visible in source
* keep the semantic model smaller than a general effect calculus
* avoid duplicating explicitness that Ferox already gets through other constructs
* leave room for later effect-system research without making it part of the v1 core

## External Interaction

Ferox does not treat IO as a checked language-level effect in v1.

Instead, external interaction remains explicit through library APIs, module boundaries, naming, and return types.

Examples of external interaction:

* filesystem reads and writes
* network communication
* terminal and console input/output
* process creation and process control
* environment-variable access
* clock, time, and timer access
* randomness sourced from the operating system or runtime environment
* foreign calls whose observable behavior depends on external state

Rules:

* standard-library modules that touch external state should expose that fact clearly in their APIs
* importing an IO-capable module does not itself perform IO
* pure data transformation on already-available values remains ordinary non-async code
* v1 does not require a dedicated signature annotation for IO

### Initial Non-Goals

The initial external-interaction model does not include:

* checked `io` annotations in every signature
* checked distinctions between subcategories such as `fs`, `net`, or `clock`
* capability-token plumbing for ordinary IO in v1

## Async Model

Ferox uses explicit async syntax in v1.

Core rule:

* `async fn` marks a function whose execution may suspend and later resume
* async does not imply automatic parallel execution

Initial direction:

* async functions use `async fn`
* the result type in the signature remains the logical result type of the computation, not a user-written future wrapper

Examples:

```ferox
async fn fetch(url: String) -> Response {
    ...
}

async fn render_page(url: String) -> Html {
    let response = await fetch(url);
    render(response)
}
```

Here `fetch` may suspend, and `await` marks the suspension point at the call site.

### Await Syntax

Ferox uses `await expr` as the suspension-point syntax.

Rules:

* `await` may be used only inside an async context
* awaiting an async computation yields its logical result value
* calling an async function in a value position requires `await` unless an explicit concurrency construct captures the computation as child work
* `await` marks a possible suspension point and is therefore always explicit in source

### Model Boundaries

The initial async model does not assume that every async computation is running concurrently.

Rules:

* async controls suspension and resumption semantics
* concurrency primitives that create independent tasks are specified separately
* pure synchronous functions may be called freely from async code
* v1 does not express async through a general effect-set system

### Initial Non-Goals

The initial async model does not include:

* explicit future or promise types in ordinary user-facing function signatures
* implicit async contagion without visible `await`
* a hidden global runtime model baked into the language specification

---

## Concurrency

Goals:

* Safe concurrency by default
* Structured concurrency
* No data races in safe code
* Explicit async behavior

## Structured Concurrency

Ferox uses structured concurrency in v1.

Core rule:

* every concurrently running child task must belong to a lexical parent scope

This means child tasks are created inside an enclosing async computation and do not outlive that computation unless a later explicit detached model is added.

### Initial Primitive

Ferox v1 uses one built-in structured-concurrency form:

* `async let` for a small fixed set of child computations

Example intent:

```ferox
async fn load_page(url: String) -> Page {
    async let profile = fetch_profile(url);
    async let posts = fetch_posts(url);

    Page {
        profile: await profile,
        posts: await posts,
    }
}
```

### Parent-Child Rules

Structured concurrency is governed by parent-child lifetime rules.

Rules:

* child tasks begin within a lexical parent scope
* the parent scope is responsible for awaiting or otherwise concluding all child tasks before ordinary successful exit
* leaving a scope with unfinished child tasks is not allowed to silently detach them
* child tasks inherit the enclosing async context rather than creating an unrelated global execution context

### Design Boundaries

The initial structured-concurrency model does not include:

* detached tasks as a default primitive
* unscoped background task spawning
* implicit task creation from every async call
* dynamic task-group constructs in v1

This keeps concurrency visible, bounded, and compatible with Ferox's long-term maintainability goals.

## Early Scope Exit And Unfinished Child Tasks

Ferox v1 does not define a rich general cancellation system.

Core rule:

* unfinished child tasks must not silently detach from their lexical parent scope

Initial direction:

* successful scope completion should not leave unfinished `async let` bindings behind
* early scope exit through `return`, `?`, or other control flow must still preserve structured cleanup rather than detaching child work
* the exact broader cancellation protocol is deferred

### Initial Non-Goals

Ferox v1 does not specify:

* a general cancellation API
* rich cancellation observation rules beyond explicit async suspension points
* sibling-cancellation policy for more advanced task structures
* cancellation-completion protocols for dynamic task groups

## Task Spawning and Synchronization

Task creation and task completion are both explicit in v1.

Core rule:

* spawning child work creates an obligation for the parent scope to synchronize with that work before normal scope completion

### `async let`

`async let` creates a child task immediately when the declaration is evaluated.

Rules:

* `async let name = expr;` starts `expr` as a child task of the current async scope
* the bound name represents the eventual result of that child task
* `await name` synchronizes with the child task and yields its result
* using the bound value without `await` is rejected
* normal successful scope completion with an unfinished `async let` binding is rejected
* early scope exit must still preserve structured cleanup rather than silently detaching the child task

Example:

```ferox
async let profile = fetch_profile(user_id);
let data = await profile;
```

### Synchronization Rules

Rules:

* every `async let` child task must be explicitly awaited or otherwise structurally concluded before ordinary successful scope completion
* synchronization points are explicit: `await name` and structured early-scope cleanup
* the language does not implicitly detach unfinished child tasks on successful scope exit
* child-task results become available only through explicit synchronization operations

### Initial Non-Goals

The initial task-spawning model does not include:

* fire-and-forget task spawning in ordinary safe code
* implicit join on successful scope exit without visible synchronization in source
* exposing raw scheduler handles as the default user model
* dynamic task-group constructs in v1

This keeps concurrency mechanics explicit and makes spawned work visible to both humans and tools.

---

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

Arbitrary syntax macros are discouraged or forbidden.

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

## Modules, Imports, Exports, and Visibility

Ferox uses a file-and-module model with explicit imports and explicit public exports.

Design rules:

* Each source file defines one module
* Module boundaries follow the file system layout
* Imports are always explicit
* Visibility is private by default
* Public API surface must be marked explicitly

### Modules

A source file corresponds to a module named from its file.

Examples:

* `math.ferox` defines module `math`
* `net/http.ferox` defines module `net.http`
* `main.ferox` may define the entry module for an executable package

Ferox does not require a `module` declaration in the file in the initial design.

Rationale:

* reduces boilerplate
* keeps the AST predictable
* makes the module graph follow the repository layout directly

### Imports

Imports use `use`.

Import a single item:

```ferox
use net.http.Client;
```

Import a module namespace:

```ferox
use net.http;
```

Import with alias:

```ferox
use net.http.Client as HttpClient;
```

Grouped import:

```ferox
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

```ferox
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

```ferox
fn normalize(value: String) -> String {
    value
}
```

Public declaration:

```ferox
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

```ferox
use core.io.Writer;
use core.hash.HashMap as Map;

fn write_all(writer: Writer, values: Map<String, Int>) -> Unit {
    writer.flush();
}
```

Module-qualified references are also allowed when importing a namespace:

```ferox
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
