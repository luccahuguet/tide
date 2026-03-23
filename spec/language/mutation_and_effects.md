# Ferox Language Specification: Mutation, Unsafe, and Operational Explicitness

This document defines Ferox's mutation model, explicit unsafe boundaries, and v1 operational explicitness rules.

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
