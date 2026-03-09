# T065 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first research pass completed

## Goal

Find genuinely strong ideas from existing language and compiler work that Ferox may be able to adopt selectively without widening the v1 core casually.

## Research Filter Used

* prefer primary sources
* prefer ideas with real technical substance, not novelty for novelty's sake
* prefer ideas that improve compiler/runtime/tooling architecture without forcing a broader source-language surface
* treat new semantic subsystems as high-risk by default

## Current Shortlist

### 1. Bosque-style regularization and reasoning-oriented IR

Why it made the shortlist:

* strong fit with Ferox's machine-friendly and analyzable ambitions
* most promising as IR and tooling architecture, not as a direct source-language copy

Primary sources:

* https://www.microsoft.com/en-us/research/project/bosque-programming-language/
* https://www.microsoft.com/en-us/research/publication/regularized-programming-with-the-bosque-language/

### 2. Perceus-style reuse and in-place compilation ideas

Why it made the shortlist:

* potentially high leverage for Ferox performance and memory behavior
* promising because it may improve implementation quality without widening the language surface

Primary sources:

* https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse
* https://www.microsoft.com/en-us/research/publication/reference-counting-with-frame-limited-reuse/

### 3. Narrow typestate or protocol-state checking

Why it made the shortlist:

* useful for protocol correctness and stateful API safety
* promising only if kept narrow and not turned into a large language subsystem

Primary source:

* https://www.microsoft.com/en-us/research/publication/typestates-for-objects/

### 4. Concurrency lessons from Pony and Verona without full capability machinery

Why it made the shortlist:

* there are valuable ideas around isolation, local reasoning, and safe parallelism
* the likely value is in selective lessons, not wholesale adoption

Primary sources:

* https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html
* https://www.ponylang.io/learn/reference-capabilities/
* https://www.microsoft.com/en-us/research/project/project-verona/
* https://www.microsoft.com/en-us/research/publication/uniqueness-and-reference-immutability-for-safe-parallelism/

### 5. Effect handlers and richer effect systems as later research only

Why it made the shortlist:

* the ideas are clearly powerful and technically serious
* they are still too expensive for the Ferox v1 core
* they belong on the long-horizon research track unless later evidence changes the tradeoff

Primary sources:

* https://www.microsoft.com/en-us/research/project/koka/
* https://www.microsoft.com/en-us/research/publication/koka-programming-with-row-polymorphic-effect-types/
* https://www.eff-lang.org/
* https://www.microsoft.com/en-us/research/publication/qualified-effect-types/

## Current Recommendation

For Ferox, the best external-ideas bias is:

* borrow implementation and tooling ideas aggressively when they strengthen the core without widening it
* be much more conservative about new source-language semantic subsystems

Current ranking:

1. Bosque-style regularization and reasoning-oriented IR
2. Perceus-style reuse and in-place compilation ideas
3. Narrow typestate or protocol checking
4. Concurrency lessons from Pony and Verona without their full capability systems
5. Richer effect systems only as much later research

## Follow-Up

The next tasks should analyze each shortlisted idea in Ferox-specific terms:

* `T066`
* `T067`
* `T068`
* `T069`
* `T070`
