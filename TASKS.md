# Tide Task List

Task-list convention:

* this file archives completed work
* open tasks live in `br` and are committed through `.beads/issues.jsonl`
* put detailed task bodies in `tasks/T###.md` when a task needs more context

## Open Work

Open tasks are tracked in `br`, not in this file.

Use:

* `br ready`
* `br list --pretty`
* `br show <id>`

See `BR_WORKFLOW.md` for the workflow rules.

## Core Language Design

- [x] T001 Define the core syntax for functions, bindings, blocks, expressions, and statements
- [x] T002 Define module, import, export, and visibility syntax
- [x] T003 Define trait/protocol syntax and implementation syntax
- [x] T004 Define generic type and generic function syntax
- [x] T005 Define pattern matching syntax and exhaustiveness rules
- [x] T006 Define literal syntax for numbers, strings, booleans, collections, and enums

## Ownership and Memory Model

- [x] T007 Define move semantics for non-copy types
- [x] T008 Define which types are copy-safe and how copy behavior is expressed
- [x] T009 Define the borrowing model and its exact restrictions
- [x] T010 Define region/arena creation, ownership, and destruction rules
- [x] T011 Define region escape rules for references
- [x] T012 Define shared ownership primitives and when they are allowed

## Type System

- [x] T013 Define inference boundaries for local variables, function signatures, and public APIs
- [x] T014 Define trait bounds and constraint syntax
- [x] T015 Decide whether associated types are supported in the initial version
- [x] T016 Define enum and ADT layout rules at the language level
- [x] T017 Define pattern destructuring rules for structs, enums, tuples, and results

## Mutation and Unsafe

- [x] T018 Define how mutable bindings and mutable access are declared
- [x] T019 Define exclusivity rules for mutable access
- [x] T020 Define the model for shared mutation and synchronization
- [x] T021 Define what `unsafe` permits
- [x] T022 Define how `unsafe` blocks, functions, and modules are isolated and audited

## Effects and Concurrency

- [x] T023 Define how effects are represented in the language
- [x] T024 Decide whether effects use annotations, capabilities, effect types, or dedicated keywords
- [x] T025 Define IO boundaries and how effectful operations are marked
- [x] T026 Define the async model and async syntax
- [x] T027 Define structured concurrency primitives
- [x] T028 Define cancellation semantics
- [x] T029 Define task spawning and synchronization rules

## Error Handling

- [x] T030 Define the syntax for `Option` and `Result` usage
- [x] T031 Define error propagation syntax
- [x] T032 Define `panic` semantics and when panic is acceptable
- [x] T033 Define whether custom error traits or protocols exist in the initial version

## Toolchain and Packages

- [x] T034 Define the package manifest format
- [x] T035 Define workspace layout and multi-package project structure
- [x] T036 Define dependency declaration and dependency resolution rules
- [x] T037 Define registry and package publishing strategy
- [x] T038 Define documentation generation format and commands
- [x] T039 Define machine-readable compiler and tooling output formats

## Formatting and Style

- [x] T040 Define canonical formatting rules
- [x] T041 Define import ordering and grouping rules
- [x] T042 Define line breaking, indentation, and trailing comma rules
- [x] T043 Define formatter behavior for partially invalid files
- [x] T044 Define formatter stability guarantees across toolchain versions

## Implementation Strategy

- [x] T045 Define the Rust bootstrap architecture
- [x] T046 Define the order of implementation for parser, formatter, type checker, and compiler stages
- [x] T047 Define the self-hosting transition criteria
- [x] T048 Define which infrastructure may remain in Rust long-term

## Code Generation and Backend Strategy

- [x] T053 Define the phased code generation and backend strategy, including early bring-up path and long-term backend direction
- [x] T059 Run an adversarial spec review to pressure-test the current design before implementation starts
  See `tasks/T059.md`.
- [x] T060 Decide whether the general effect-set model belongs in Ferox v1
  See `tasks/T060.md`.
- [x] T061 Narrow the ownership model to a smaller v1 surface
  See `tasks/T061.md`.
- [x] T062 Settle trait and method receiver semantics for v1
  See `tasks/T062.md`.
- [x] T063 Ground async, structured concurrency, and cancellation in a smaller executable v1 model
  See `tasks/T063.md`.
- [x] T064 Rewrite vision and comparison claims to match the actual confidence level of the spec
  See `tasks/T064.md`.
- [x] T065 Research high-value language and compiler ideas from existing work that Ferox may be able to adopt selectively
  See `tasks/T065.md`.
- [x] T066 Analyze whether Bosque-style regularization and reasoning-oriented IR should influence Ferox
  See `tasks/T066.md`.
- [x] T067 Analyze whether Perceus-style reuse and in-place compilation ideas should influence Ferox
  See `tasks/T067.md`.
- [x] T068 Analyze whether narrow typestate or protocol-checking ideas should influence Ferox
  See `tasks/T068.md`.
- [x] T069 Analyze what Ferox should learn from Pony and Verona concurrency ideas without importing full capability machinery
  See `tasks/T069.md`.
- [x] T070 Analyze whether effect handlers or richer effect systems belong only in a much later Ferox v2+ wave
  See `tasks/T070.md`.

## Implementation Readiness

- [x] T071 Run an implementation go / no-go review against the narrowed v1 direction
  See `tasks/T071.md`.

## Workflow

- [x] T078 Set up the initial `br` pilot workflow for active Ferox work
  See `tasks/T078.md`.
- [x] T079 Make `br` the canonical tracker for open Ferox work and keep `TASKS.md` as the completed-task archive
  See `tasks/T079.md`.

## Naming and Release Readiness

- [x] T049 Run trademark review for the earlier working name
- [x] T050 Run software and developer-tool brand conflict review for the earlier working name
- [x] T054 Find and screen a replacement public name for the language
- [x] T051 Review domain strategy for `Ferox` and `ferox.*`
- [x] T052 Finalize whether `.ferox` is the official source file extension
- [x] T055 Run trademark review for `Orbis` in target jurisdictions
- [x] T056 Run software and developer-tool brand conflict review for `Orbis`
- [x] T057 Run trademark review for `Ferox` in target jurisdictions
- [x] T058 Run software and developer-tool brand conflict review for `Ferox`
