# Ferox Language Specification: Ownership, Borrowing, and Safety

This document defines Ferox's v1 ownership, borrowing, shared-ownership, and safety-boundary rules.

## Memory Model

Ferox uses:

**Ownership + simplified borrowing**

through a smaller and more restrictive model than Rust's full borrow-checker surface.

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
* Complex long-distance borrowing patterns are intentionally out of scope in v1
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

## Concrete Ownership Rules

The v1 ownership model should follow these concrete rules.

### Whole-Value Ownership

Ownership moves and borrows are defined around ordinary values and explicit access paths, but v1 should stay conservative about fine-grained path reasoning.

Rules:

* the default unit of ownership reasoning is the value bound to a local binding, parameter, or named field access path written directly in source
* moving a non-copy value invalidates the moved-from path immediately
* a moved-from binding or path may not be read, written, borrowed, or moved again until it is reinitialized
* moving a value while any active borrow derived from that value exists is rejected

### Mutation Rules

Rules:

* mutation requires a mutable binding or an active exclusive mutable borrow
* mutation through shared borrows is rejected
* mutation through shared ownership requires explicit shared-ownership mechanisms rather than ordinary `&mut` borrowing
* ordinary safe mutation may not hide synchronization or ownership promotion

### Borrow Origin Rules

Rules:

* safe borrows must be explicit in source
* a borrow must originate from a currently valid owner path
* a borrow does not create a new owner
* borrows may be passed to functions, but the borrow itself remains governed by the enclosing lexical scope in v1

## Conservative Precision Rules

Ferox v1 should choose conservative checker behavior when more precision would require substantially more semantic machinery.

### No Guaranteed Disjoint-Field Borrow Precision

Ferox v1 does not need to guarantee that the checker accepts simultaneous borrows of disjoint fields from the same aggregate.

This means code such as:

```ferox
let mut pair = Pair { left: 1, right: 2 };
let l = &mut pair.left;
let r = &mut pair.right;
```

may be rejected in v1 even though the fields are disjoint.

Reason:

* this keeps the checker smaller and more predictable
* it avoids turning field-sensitive alias analysis into a hidden source of complexity early

### No Partial Moves from Ordinary Aggregates in V1

Ferox v1 should reject partial moves that leave the remainder of an aggregate in a partly-moved state.

Example:

```ferox
let user = User { id: 1, name: "ana" };
let name = user.name;
show(user.id);
```

This should be rejected in v1 rather than requiring the language and checker to reason about partially moved aggregate states.

Reason:

* keeps the value-state model smaller
* avoids partially valid aggregates as a user-visible concept in the initial language

### Lexical Endpoints Over Flow-Sensitive Borrow Endings

Ferox v1 should require explicit scope boundaries to end borrows early.

Example:

```ferox
let mut user = load_user();
let w = &mut user;
rename(w, "new-name");
save(user);
```

This may be rejected in v1 if `w` remains in scope, even if a more flow-sensitive system could prove it is no longer used.

Preferred v1 style:

```ferox
let mut user = load_user();
{
    let w = &mut user;
    rename(w, "new-name");
}
save(user);
```

Reason:

* explicit scopes are easier for both humans and tools to reason about
* this avoids non-local borrow-ending analysis in the initial checker

## Deliberate Exclusions Relative To Rust

Ferox v1 should reject some useful Rust patterns on purpose.

Current deliberate exclusions:

* no non-lexical lifetime-style borrow endings
* no requirement to accept disjoint sibling-field mutable borrows
* no partial moves from ordinary aggregates
* no implicit promotion from ordinary ownership into shared ownership
* no mutation through ordinary shared borrows
* no hidden interior-mutability semantics through ordinary references
* no borrowing model that depends on users understanding a large lifetime sublanguage

Current conservative rule:

* borrows should not cross async or task boundaries by default unless a later concurrency rule explicitly allows a narrowly defined case

Example intent:

```ferox
let user = load_user();
async let handle = process(&user);
```

This should be rejected in v1 unless the concurrency design later introduces an explicit narrowly scoped exception and updates this rule accordingly.

## Accepted And Rejected Ownership Examples

Accepted:

```ferox
let user = load_user();
let r1 = &user;
let r2 = &user;
show(r1);
show(r2);
```

Accepted:

```ferox
let mut user = load_user();
{
    let w = &mut user;
    rename(w, "new-name");
}
save(user);
```

Rejected:

```ferox
let user = load_user();
let moved = user;
show(user);
```

Rejected:

```ferox
let mut user = load_user();
let r = &user;
let w = &mut user;
```

Rejected:

```ferox
let mut pair = Pair { left: 1, right: 2 };
let l = &mut pair.left;
let r = &mut pair.right;
```

Rejected:

```ferox
let user = User { id: 1, name: "ana" };
let name = user.name;
show(user.id);
```

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

* no use-after-free
* no double free or double drop
* no dangling borrow or other access path to already-destroyed storage
* no uninitialized read through safe code
* no null dereference through ordinary safe Ferox references and other non-nullable safe access paths
* no data races in safe concurrent code
* no invalid enum or tagged-union discriminant created through safe construction paths

These guarantees apply only when all of the following are true:

* the code stays within the safe Ferox subset
* `unsafe` code, foreign code, and runtime escape hatches uphold their documented contracts
* core standard-library ownership and synchronization primitives uphold the language invariants they expose
* the compiler and runtime implementation correctly enforce the specified rules

This section does not claim:

* semantic correctness of business logic
* protocol correctness beyond what the type system explicitly encodes
* freedom from deadlock, starvation, livelock, or cancellation bugs
* prevention of every invalid application state
* safety for `unsafe` code or foreign code by default

Principle:

> If safe Ferox code compiles, it should satisfy a small, explicit, and mechanically enforced set of structural safety guarantees.

Unsafe code exists but must be explicit and isolated.

## Core Ownership Invariants

The ownership model should preserve a small invariant set that remains true independently of surface syntax details.

### 1. Live Owner Invariant

Every non-shared value has exactly one live owning path at a time.

Implications:

* moving a value transfers that unique owning path
* using the old path after a move is rejected
* multiple owners are only possible through explicit shared-ownership primitives

### 2. Derived Access Invariant

Every safe read, write, move, or drop must occur through:

* the current live owner, or
* a borrow derived from that live owner under the borrowing rules, or
* an explicit shared-ownership primitive with its own rules

There is no hidden aliasing path outside those mechanisms in safe code.

### 3. Unique Mutation Invariant

Mutation requires unique active access to the relevant value.

Implications:

* mutable access may not overlap other mutable access to the same value
* mutable access may not overlap shared read access to the same value
* safe mutation and aliasing must obey the same rule in local and concurrent settings

### 4. Lexical Borrow Invariant

In v1, safe borrows are local, explicit, and lexical.

Implications:

* borrow lifetimes do not depend on non-local compiler reasoning in v1
* ending a borrow early should usually require ending or narrowing an explicit scope
* Ferox may reject valid Rust programs if accepting them would weaken predictability or locality

### 5. Drop-Exactly-Once Invariant

Owned values are destroyed at most once, and only after their final owning path becomes unreachable.

Implications:

* safe code cannot double-drop a value
* safe code cannot observe a value after destruction
* explicit shared-ownership primitives must preserve this invariant through their own accounting

### 6. Explicit Sharing Invariant

Multiple-owner access is never implicit in safe code.

Implications:

* ordinary borrowing does not create multiple owners
* conversion from ordinary ownership to shared ownership must be explicit
* single-threaded shared ownership and synchronized shared ownership remain distinct

### 7. Concurrency Boundary Invariant

Cross-context access in safe code must happen through one of a small number of explicit paths:

* ownership transfer
* explicit immutable sharing where the language and type permit it
* explicit synchronized shared ownership

Ferox safe code should not admit hidden shared mutable state across concurrent contexts.

## Safety Boundary Non-Goals

Ferox's safety claim should remain deliberately narrow.

Ferox does not claim, merely from being safe code, that a program is:

* deadlock-free
* panic-free
* logically correct
* free of resource leaks caused by explicit shared ownership cycles if such cycles are introduced by user-visible shared abstractions
* free of application-level race conditions that do not violate the language's structural data-race rules

The language should be honest about this boundary rather than pretending memory safety implies full correctness.

## Safety Boundary Examples

Examples of failures Ferox safe code should rule out mechanically:

```ferox
let user = load_user();
let moved = user;
show(user);
```

This should be rejected because `user` is used after ownership moved into `moved`.

```ferox
let mut user = load_user();
let r = &user;
let w = &mut user;
```

This should be rejected because shared and mutable access overlap.

Examples of bugs Ferox does not claim to prevent merely by being safe code:

```ferox
fn can_login(user: User) -> Bool {
    if user.disabled {
        true
    } else {
        false
    }
}
```

This can be logically wrong while still being memory-safe.

Likewise, if later synchronized shared primitives permit blocking operations, safe code may still be able to deadlock or starve unless a separate protocol or verification layer rules that out.

## Ownership Model Consequences

These invariants impose real language-design constraints.

Current consequences:

* shared ownership must stay explicit and secondary
* borrowing must stay smaller and more local than Rust in v1
* ordinary safe references may not hide ownership promotion or synchronization
* concurrency design must preserve the same mutation and aliasing invariants rather than introducing a separate weaker model
* any `unsafe` escape hatch must document which invariant it temporarily suspends and what contract it must re-establish before returning to safe code
