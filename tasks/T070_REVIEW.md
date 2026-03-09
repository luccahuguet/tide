# T070 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed

## Decision

Current recommendation:

* **keep effect handlers and richer effect systems on the long-horizon track only**

That means:

* not part of Ferox v1
* not an automatic v2 commitment
* keep as research-grade candidates unless later evidence is unusually strong

## Why These Ideas Still Matter

The primary sources show that richer effect systems are not toy ideas.

They offer real value:

* precise tracking of effects
* principled effect composition
* user-definable control abstractions through handlers
* stronger reasoning about control flow in advanced cases

So the right answer is not "these are bad ideas."

The right answer is:

* they are powerful
* they are expensive
* they are misaligned with Ferox's current stage

## Why Ferox Should Keep Them Far Away For Now

### 1. Ferox already rejected a smaller effect system for v1

Ferox has already concluded that even a general effect-set model is too much for the v1 core.

Effect handlers and richer effect typing are a larger move, not a smaller one.

So they are not a near-term comeback candidate.

### 2. Ferox already has simpler explicit mechanisms for most current needs

Today Ferox gets explicitness from:

* `async fn` and `await`
* `unsafe`
* `Result` and `Option`
* explicit synchronization and sharing wrappers

That does not solve every advanced control-abstraction problem, but it does cover the current language direction without adding a second major semantic subsystem.

### 3. Richer effect systems would compete with the core simplification effort

Ferox is still trying to prove:

* a smaller ownership model
* a smaller concurrency model
* a coherent implementation path

This is exactly the wrong phase to add:

* effect rows
* effect inference and propagation
* handler semantics
* control-flow linearity qualifiers

### 4. The most interesting effect work solves problems Ferox does not yet know it has

The Koka and qualified-effect work shines when a language already needs:

* composable user-defined effects
* advanced control abstractions
* principled interaction between control effects and side effects

Ferox has not earned that pressure yet.

## What Ferox Should Learn Without Adopting The System

### 1. Keep explicitness through ordinary constructs as long as possible

The existence of strong effect research does not mean every language should adopt it early.

For Ferox, the main lesson is:

* do not add a full effect system just because it is elegant

### 2. Be precise later about the problem before reviving the solution

If effect systems come back later, the trigger should be a concrete recurring problem such as:

* higher-order libraries that need effect abstraction
* repeated control-flow patterns that are awkward with plain async and `Result`
* a strong need for user-defined control abstractions that ordinary language constructs cannot express cleanly

### 3. Separate effect research from v1 and v2 language promises

Ferox should not let the long-horizon elegance of effect research distort the current roadmap.

## Required Evidence Before Reopening This

Ferox should revisit richer effect systems only if most of the following become true:

* v1 is already stable and implemented
* real Ferox code shows repeated pressure that `async`, `unsafe`, `Result`, and explicit APIs do not handle well
* there is a clear candidate problem statement, not just admiration for Koka or Eff
* the team can explain why a richer effect system would simplify the language overall instead of merely adding power
* compiler and tooling maturity are high enough to absorb another major checker subsystem

If those conditions are not met, the default answer should remain no.

## Best Roadmap Placement

### V1

Reject completely.

### V2

Not automatic.

Only reconsider if v1 and early real-world usage produce unusually strong evidence.

### V3 or research track

This is the correct home today.

That matches both the technical ambition of the idea and the current immaturity of the Ferox codebase.

## Why This Helps Ferox

This protects the language from one of the easiest forms of self-sabotage:

* importing a beautiful semantic system before the core language is even proven

It preserves:

* focus
* roadmap honesty
* a smaller v1
* room for later research without current roadmap pollution

## Primary Sources

* https://www.microsoft.com/en-us/research/project/koka/
* https://www.microsoft.com/en-us/research/publication/koka-programming-with-row-polymorphic-effect-types/
* https://www.eff-lang.org/
* https://www.microsoft.com/en-us/research/publication/qualified-effect-types/

## Current Recommendation

For Ferox:

* keep richer effect systems as **study-only / long-horizon** work
* do not treat them as an assumed v2 feature
* require strong evidence from real Ferox usage before reopening them
