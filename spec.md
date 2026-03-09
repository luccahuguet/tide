# Ferox Spec Index

This repository now splits the specification into focused documents instead of keeping all language, tooling, and strategy material in one file.

## Current Status

The language is called **Ferox**.

Current status:

* public language name selected as **Ferox**
* official CLI binary selected as `ferox`
* official source file extension selected as `.ferox`
* domain and legal follow-up review remain advisable before public launch
* adversarial v1 scope-narrowing first pass is complete
* first implementation-readiness review is complete
* current recommendation is to start a narrow Rust bootstrap, not a maximal semantic implementation

## Main Documents

* [spec/VISION.md](./spec/VISION.md): project vision, priorities, positioning, and high-level design principles
* [spec/LANGUAGE.md](./spec/LANGUAGE.md): language rules, semantics, syntax, ownership, async, concurrency, and error handling
* [spec/TOOLING.md](./spec/TOOLING.md): formatter, CLI, manifests, workspaces, and package-system design
* [spec/IMPLEMENTATION.md](./spec/IMPLEMENTATION.md): bootstrap and self-hosting strategy

## Supporting Documents

* [TASKS.md](./TASKS.md): completed-task archive
* [BR_WORKFLOW.md](./BR_WORKFLOW.md): open-task tracking rules and `br` workflow
* [V1_DIRECTION.md](./V1_DIRECTION.md): current narrowed v1 direction after the review passes
* [ROADMAP.md](./ROADMAP.md): version roadmap and implementation-path summary
* [V2_BACKLOG.md](./V2_BACKLOG.md): post-v1 candidate backlog
* [V3_BACKLOG.md](./V3_BACKLOG.md): long-horizon backlog
* [tasks/T071_REVIEW.md](./tasks/T071_REVIEW.md): current implementation go / no-go verdict
* [PUBLIC_NAME_REVIEW.md](./PUBLIC_NAME_REVIEW.md): naming review history and final decision
* [FEROX_DOMAIN_STRATEGY.md](./FEROX_DOMAIN_STRATEGY.md): first-pass domain strategy for `Ferox`
* [FEROX_TRADEMARK_REVIEW.md](./FEROX_TRADEMARK_REVIEW.md): first-pass trademark review for `Ferox`
* [FEROX_SOFTWARE_BRAND_REVIEW.md](./FEROX_SOFTWARE_BRAND_REVIEW.md): first-pass software and developer-brand review for `Ferox`
* [AI_SUPERVISOR.md](./AI_SUPERVISOR.md): mandatory AI supervisor review policy and prompt template
* [FEATURE_REVIEW.md](./FEATURE_REVIEW.md): feature-by-feature strategic review
* [COMPARISON.md](./COMPARISON.md): positioning against Rust, Zig, and Go
* [MEMORY_MODEL_RESEARCH.md](./MEMORY_MODEL_RESEARCH.md): ownership and borrowing research notes
* [GOVERNANCE.md](./GOVERNANCE.md): project governance rules
