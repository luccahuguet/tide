# T068 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed

## Decision

Current recommendation:

* **adapt typestate ideas only in a narrow form**

Recommended Ferox boundary:

* **no** broad typestate subsystem in the v1 core
* **yes** to library-encoded protocol states where ownership already makes them natural
* **maybe later** to opt-in analysis or lint support for well-scoped protocol checking

## Why Typestate Matters

The typestate work addresses a real problem:

* some APIs are only valid in certain states
* ordinary type signatures often do not express those state transitions clearly
* protocol misuse can survive ordinary type checking but still be logically wrong

This is valuable for Ferox because Ferox cares about:

* high-trust software
* explicit state transitions
* analyzability

## Why Ferox Should Be Careful

The full typestate vision is not a small add-on.

The primary source explicitly centers difficult problems such as:

* pre- and postconditions on methods
* invariants on object representations
* sharing relationships among objects
* modular static checking of those conditions

That is serious machinery.

Ferox should not add another broad semantic checker on top of:

* ownership
* borrowing
* explicit mutation
* traits and generics
* async and concurrency rules

## What Ferox Should Borrow

### 1. Protocol-state thinking

Ferox should absolutely borrow the design habit of asking:

* what states can this resource be in?
* which operations are legal in each state?
* should a transition consume the old state and produce a new one?

This mindset fits Ferox very well.

### 2. Library-encoded state transitions

The smallest sensible Ferox form is not a new language subsystem.

It is:

* distinct types for distinct protocol states, or
* generic state parameters on a resource type, or
* consuming methods that move from one state-bearing value to another

Ferox's ownership model already makes this style plausible.

Example shapes later could look like:

* `Socket[Connected]`
* `Socket[Closed]`
* `Builder[Incomplete] -> Builder[Ready]`

without requiring a general typestate language.

### 3. Optional analysis later, if real APIs justify it

After v1, Ferox could consider a narrow analysis or lint layer for protocol-like APIs when the patterns are common enough to justify tool support.

The right target would be:

* opt-in
* narrow
* driven by real libraries
* built on top of ordinary types and ownership rules

not a universal new language discipline.

## What Ferox Should Not Borrow Directly

### 1. A full typestate checker in v1

Reject.

This would expand the core semantic surface too much and too early.

### 2. Typestate as a second ownership system

Reject.

Ferox should not let protocol-state tracking turn into another broad model for aliasing, lifetime, or mutation.

Ownership already carries the main resource-discipline story.

### 3. Automatic inference-heavy protocol machinery

Reject by default.

If typestate support ever appears, it should stay explicit enough that users and tools can reason about it directly.

## Best Adoption Boundary

### Source language

No general typestate subsystem.

Keep the source language small.

### Library and API design

This is the strongest current fit.

Use ordinary:

* types
* generics
* move semantics
* consuming methods

to encode protocol states where the payoff is high.

### Later analysis and lints

Possible later fit.

Only consider this after:

* the v1 trait and generic model is stable
* Ferox has real libraries with repeated state-machine patterns
* there is evidence that ordinary library encoding is useful but insufficient

## Why This Helps Ferox

This keeps the value while avoiding a large new subsystem:

* protocol correctness remains possible
* ownership and moves do most of the work
* v1 stays smaller
* later tooling can still grow if justified

## Roadmap Implication

Current recommendation:

* typestate does **not** belong in the v1 core
* a narrow protocol-state analysis or lint layer is a plausible **v2 candidate**
* it should enter only if real Ferox APIs create sustained pressure for it

## Primary Source

* https://www.microsoft.com/en-us/research/publication/typestates-for-objects/

## Current Recommendation

For Ferox:

* **adapt** the protocol-state idea narrowly
* use library encoding first
* consider opt-in lints or analysis later
* reject a broad built-in typestate subsystem for v1
