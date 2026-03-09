# T061 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed and reflected into the live spec

## Decision

Current recommendation:

* **narrow the v1 ownership model**

The current ownership story is too wide for an initial language that also wants to be smaller and more predictable than Rust.

## What Ferox Should Keep As Core

The strongest v1 core is:

* owned values by default
* explicit move semantics for non-copy types
* a narrow copy-safe model
* explicit shared borrows with `&`
* explicit exclusive mutable borrows with `&mut`
* lexical borrow scopes in v1

That is already a real language with a strong identity.

## What Is Making The Current Surface Too Broad

The current spec layers on:

* language-level arenas as a first-class construct
* explicit shared ownership types
* explicit synchronized shared ownership types
* interior-mutation wrappers
* synchronized shared-mutation wrappers

Some of these are reasonable tools. The problem is taking all of them as first-wave language surface.

## Recommended Narrowing

Guiding rule:

* one primary ownership path
* at most one explicit escape hatch
* no parallel first-class lifetime-management models unless they solve genuinely different problems

This is the practical meaning of keeping Ferox smaller and easier to reason about.

### 1. Keep ownership plus lexical borrowing as the main v1 story

Reason:

* this is the cleanest differentiator
* it is the part most directly tied to Ferox's core promise
* it is already difficult enough to specify and implement well

### 2. Defer language-level arenas from the v1 semantic core

Current recommendation:

* do **not** make `arena` a core v1 language construct unless a later pass proves it is essential

Reason:

* arenas are not a small add-on; they are a second lifetime-management model
* once arenas exist in the language, many later features must answer how they interact with arena-backed values
* they may be useful, but usefulness is not enough to justify adding another major semantic subsystem to v1

If Ferox eventually wants arenas, they should come after the base ownership model proves coherent.

### 3. Treat shared ownership and synchronization primarily as library-level escape hatches

Current recommendation:

* keep shared ownership small and explicit
* avoid making `Shared`, `SyncShared`, `Cell`, `RefCell`, `Mutex`, and `RwLock` feel like equally central ownership modes in the v1 story

Reason:

* these constructs are useful, but they should not compete with the primary ownership model for conceptual center stage
* they belong in the language ecosystem, but not in the core pitch of how Ferox code is meant to work

### 4. Resist compensating for lexical borrowing with more ownership machinery

Reason:

* lexical borrow scopes are already a deliberate simplification
* if Ferox immediately adds arenas and multiple shared-ownership variants to soften every restriction, the result is not a smaller model
* it is better to accept real restrictions in v1 than to patch them with overlapping systems

## Proposed V1 Ownership Shape

Recommended v1 ownership surface:

* moves by default for non-copy types
* explicit `copy` for narrow copy-safe types
* `&` for shared borrow
* `&mut` for exclusive mutable borrow
* lexical borrow scopes
* no fine-grained borrow splitting in v1
* shared ownership available only through explicit library types when truly needed

Recommended v1 cuts or deferrals:

* language-level `arena` and `alloc arena` constructs
* broad first-class region language
* making shared ownership and interior mutability a large part of the introductory ownership story

## Why This Is Better

This narrower model:

* better matches the "smaller than Rust" claim
* keeps the hard part concentrated in one place
* makes the checker and diagnostics easier to reason about
* avoids teaching multiple overlapping resource-management styles in the first version

## Consequences For The Spec

If adopted, this decision implies follow-up edits to:

* `spec/LANGUAGE.md`
  * narrow the ownership story to owned values, moves, borrowing, and mutation
  * either delete or explicitly defer the arena sections
  * reduce the prominence of shared-ownership and shared-mutation wrappers in the language core
* `spec/VISION.md`
  * make the ownership promise tighter and less feature-broad
* `COMPARISON.md`
  * stop assuming Ferox already has a mature ownership-plus-arenas story

## Open Questions

This review does **not** yet settle:

* whether arenas should be a post-v1 language feature or a library/tooling pattern
* the smallest shared-ownership surface Ferox should acknowledge in its core docs
* whether the current lexical-borrow restrictions are acceptable enough without arena support

Those questions should continue in later passes, but the default should now be to cut rather than add.

## Current Recommendation

For the adversarial spec pass:

* treat the ownership core as **keep**
* treat arenas as **defer unless proven essential**
* treat shared ownership and synchronization as **keep, but demote from core identity**
