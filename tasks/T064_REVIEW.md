# T064 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed and reflected into the live docs

## Decision

Current recommendation:

* **rewrite the vision and comparison language to be narrower, more conditional, and less self-scoring**

## Main Problem

The current positioning language gets ahead of the spec.

It claims or strongly implies that Ferox will be:

* simpler than Rust
* more maintainable than Zig
* better suited to agent-assisted development than Rust, Zig, and Go

At the same time, the spec still has unresolved questions in:

* effects
* ownership surface
* method receivers
* async and cancellation

That mismatch creates pressure in the wrong direction. It makes the spec sound more proven than it is.

## Recommended Direction

### 1. Replace comparative certainty with conditional positioning

Prefer language like:

* Ferox aims to be smaller and more predictable than Rust
* Ferox is trying to keep ownership-based design explicit and analyzable
* Ferox is designed for high-trust software and long-lived codebases

Avoid language like:

* Ferox is better than Rust, Zig, or Go on specific dimensions
* Ferox is already more maintainable or more agent-friendly than mature languages

### 2. Remove self-scoring tables

Current recommendation:

* remove the numeric score table from `COMPARISON.md`

Reason:

* the numbers imply confidence and measurement that the project does not actually have
* they create marketing-style pressure instead of design discipline
* they will age poorly as the spec changes

### 3. Distinguish aspiration from current confidence

The docs should clearly separate:

* current direction
* unresolved design bets
* long-term aspirations

That makes it easier to change the spec honestly when a big subsystem turns out to be too ambitious.

### 4. Make cuts easier, not harder

The vision should help the project cut scope.

If the current framing says Ferox must simultaneously be:

* safer than Go
* simpler than Rust
* more structured than Zig
* better for agents than all three

then almost every cut starts to feel like a retreat.

The better framing is:

* Ferox has a narrow core goal
* Ferox will only keep claims it can support with a coherent design

## Concrete Rewrite Targets

### `spec/VISION.md`

Recommended changes:

* soften comparative claims against Rust, Zig, and Go
* make "smaller than Rust" an aspiration guarded by actual cuts
* reduce the emphasis on being better than all peers across many axes

### `COMPARISON.md`

Recommended changes:

* remove numeric scoring
* replace score tables with qualitative tradeoff discussion
* make clear that Ferox is a design-stage project still deciding which ambitious ideas to keep

## Why This Matters

This is not just a messaging issue.

Overclaiming in the vision changes design behavior:

* it makes the team defend too many subsystems at once
* it makes simplification feel like failure
* it rewards broad ambition over tight coherence

Narrower claims will make the adversarial review easier and healthier.

## Current Recommendation

For the adversarial spec pass:

* rewrite the vision to be conditional and focused
* remove self-scoring comparison language
* treat comparative superiority as something Ferox must earn later, not something the current docs should assume now
