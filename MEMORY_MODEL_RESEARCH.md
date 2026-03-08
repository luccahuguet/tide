# Tark Memory Model Research

This document captures research and design guidance for the upcoming memory-model tasks:

* `T007` move semantics
* `T008` copy-safe types
* `T009` borrowing restrictions
* `T010` region/arena creation and destruction
* `T011` region escape rules
* `T012` shared ownership primitives

The goal is not to copy Rust or Cyclone. The goal is to identify what they got right, where they became costly, and what Tark should do differently.

## Sources

Primary sources used:

* Rust ownership: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
* Rust references: https://doc.rust-lang.org/stable/core/primitive.reference.html
* Rust mutable borrow rule: https://doc.rust-lang.org/error_codes/E0499.html
* Rust borrowing limitations: https://doc.rust-lang.org/nomicon/borrow-splitting.html
* Rust lifetime mismatch example: https://doc.rust-lang.org/nomicon/lifetime-mismatch.html
* Rust shared ownership: https://doc.rust-lang.org/book/ch15-04-rc.html
* Rust unsafe/raw pointers: https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html
* Rust Send/Sync discussion: https://doc.rust-lang.org/nomicon/send-and-sync.html
* Cyclone regions overview: https://www.cs.cornell.edu/projects/cyclone/online-manual/main-screen008.html
* Cyclone for C programmers: https://www.cs.cornell.edu/Projects/cyclone/online-manual/main-screen002.html
* Linear regions research summary: https://www.cs.cornell.edu/people/fluet/research/substruct-regions/

## Main Findings

### 1. Ownership works best when the default rule is simple

Rust's strongest rule is simple:

* assignment of non-copy values moves ownership
* references borrow rather than own
* shared borrows can coexist
* mutable borrows must be exclusive

That rule is powerful because users can usually predict it before they read the compiler error.

Recommendation for Tark:

* keep the default ownership rule extremely small
* make moves the default for non-copy values
* make borrowing a separate, explicit operation
* do not start with many special borrow categories

### 2. Mutable aliasing control is the real core, not lifetime syntax

The most important practical lesson from Rust is not lifetime syntax. It is the aliasing rule:

* many readers or one writer

That is the rule surfaced by the Rust error documentation and reference docs. The complexity comes later when the compiler must prove when borrows start and end.

Recommendation for Tark:

* specify the aliasing rule first
* specify borrow duration second
* defer any explicit lifetime parameter syntax unless it becomes unavoidable

### 3. Rust shows both the value and the cost of precise borrowing

Rust's borrow checker prevents real bugs, but the official examples also show where it becomes conservative or hard to reason about:

* borrow splitting is partial
* container APIs often expose lifetime friction
* lifetime relationships leak into user-visible signatures

Recommendation for Tark:

* accept less expressiveness in exchange for a smaller mental model
* prefer shorter, more local borrows
* reject complicated long-distance borrowing patterns instead of trying to prove all of them

This supports the original Tark goal of being smaller than Rust.

### 4. Cyclone shows that regions are powerful but dangerous to overexpose

Cyclone demonstrates that regions can give explicit control over allocation and deallocation without full garbage collection. It also shows the main trap:

* too many region kinds
* too much annotation surface
* region inference that can surprise users

Cyclone exposes stack regions, lexical regions, heap, unique regions, and reference-counted regions. That is powerful, but it is a lot.

Recommendation for Tark:

* do not expose many region classes in v1
* start with one explicit growable lexical arena construct
* keep ordinary stack locals implicit
* treat heap allocation and shared ownership as separate mechanisms, not additional user-facing region kinds

### 5. Shared ownership must stay explicit and narrow

Rust's `Rc<T>` makes the tradeoff clear:

* multiple ownership is possible
* it is explicit
* it incurs bookkeeping cost

The Send/Sync material also shows that shared ownership and concurrency interact dangerously when synchronization is not explicit.

Recommendation for Tark:

* shared ownership should never be the default
* v1 should use explicit library or standard types for shared ownership
* single-threaded shared ownership and synchronized multi-threaded shared ownership should not be conflated

### 6. Unsafe should exist to support abstraction boundaries, not bypass the model casually

Rust's unsafe/raw-pointer split is useful:

* safe references carry guarantees
* raw pointers and unchecked aliasing are moved into `unsafe`

Recommendation for Tark:

* keep safe references strong
* move raw pointer-style operations into explicit `unsafe`
* use `unsafe` to build safe abstractions, not to create an alternate programming style

## Recommendations For Tark

## Recommended v1 memory model shape

Tark v1 should aim for the following:

* one owner for non-copy values
* move by default
* explicit copy only for copy-safe types
* two borrow kinds:
  * shared read borrow
  * exclusive mutable borrow
* one explicit arena/region construct for grouped allocation
* explicit shared ownership through standard library types
* raw pointers and unchecked aliasing only in `unsafe`

This is enough to be meaningful without trying to beat Rust on expressiveness in v1.

## Specific recommendations by task

### T007: Move semantics for non-copy types

Recommendation:

* adopt Rust-like move semantics for non-copy values
* assignment, argument passing, and return should move unless the type is copy-safe or an explicit borrow is taken

Why:

* it is teachable
* it makes ownership visible
* it aligns with Tark's performance transparency goals

### T008: Copy-safe types

Recommendation:

* restrict implicit copying to a small set of plainly copy-safe value types in v1
* examples: integers, booleans, chars, small fixed-size plain-value aggregates if all fields are copy-safe
* require explicit clone/copy operation for everything else

Why:

* cheap copies stay cheap
* expensive copies never hide in ordinary assignment

### T009: Borrowing restrictions

Recommendation:

* define only two safe borrow forms in v1:
  * shared borrow
  * exclusive mutable borrow
* enforce:
  * many shared borrows allowed
  * one exclusive mutable borrow allowed
  * no shared borrows overlapping an exclusive mutable borrow
* keep borrows lexically local where possible

Why:

* this is the real safety rule people need to remember
* it gives strong value without the full complexity of Rust's more advanced cases

### T010: Region/arena creation and destruction

Recommendation:

* use one explicit lexical arena construct in v1
* arena lifetime should be structured and block-bounded
* deallocation should happen en masse at scope exit

Why:

* this captures most of the value of arenas
* it avoids Cyclone-style proliferation of region classes
* it keeps implementation and user reasoning smaller

### T011: Region escape rules

Recommendation:

* references into an arena may not escape the arena's lexical lifetime
* returning such references or storing them into longer-lived objects should be rejected
* do not attempt complex region polymorphism in v1

Why:

* this keeps the rule mechanical
* it avoids complex inference and surprising escapes

### T012: Shared ownership primitives

Recommendation:

* make shared ownership explicit through dedicated standard types
* split single-threaded and synchronized shared ownership
* do not model shared ownership as ordinary borrowing

Why:

* it keeps the fast path simple
* it makes the cost and synchronization story visible
* it avoids unsound or confusing overlap between aliasing and ownership

## Design Choices Tark Should Probably Avoid

These are the main traps suggested by the research:

* do not create many first-class region kinds in v1
* do not require explicit lifetime annotations in ordinary user code unless absolutely necessary
* do not support complex long-distance borrowing just because Rust can express some of it
* do not mix shared ownership, interior mutability, and concurrency into one blurred abstraction
* do not let `unsafe` become the easy escape hatch for routine programming

## Proposed Draft Policy

If Tark wants to stay bold but coherent, the draft policy should be:

* Be bold in ownership and explicitness
* Be conservative in borrow expressiveness
* Be minimal in region design
* Be explicit in shared ownership
* Be strict about `unsafe` boundaries

## Immediate Design Consequence

Before writing the actual `T007` to `T012` sections, the most important design decision to lock is:

* whether Tark wants a strictly lexical borrowing model in v1

My recommendation is yes.

That would mean:

* borrow scopes are intentionally local and predictable
* some valid-but-complex Rust patterns are rejected in Tark
* arenas remain structured and block-scoped
* the language stays closer to its "smaller than Rust" promise
