# Tark Task List

## Core Language Design

- [x] T001 Define the core syntax for functions, bindings, blocks, expressions, and statements
- [x] T002 Define module, import, export, and visibility syntax
- [x] T003 Define trait/protocol syntax and implementation syntax
- [x] T004 Define generic type and generic function syntax
- [ ] T005 Define pattern matching syntax and exhaustiveness rules
- [ ] T006 Define literal syntax for numbers, strings, booleans, collections, and enums

## Ownership and Memory Model

- [x] T007 Define move semantics for non-copy types
- [ ] T008 Define which types are copy-safe and how copy behavior is expressed
- [ ] T009 Define the borrowing model and its exact restrictions
- [ ] T010 Define region/arena creation, ownership, and destruction rules
- [ ] T011 Define region escape rules for references
- [ ] T012 Define shared ownership primitives and when they are allowed

## Type System

- [ ] T013 Define inference boundaries for local variables, function signatures, and public APIs
- [ ] T014 Define trait bounds and constraint syntax
- [ ] T015 Decide whether associated types are supported in the initial version
- [ ] T016 Define enum and ADT layout rules at the language level
- [ ] T017 Define pattern destructuring rules for structs, enums, tuples, and results

## Mutation and Unsafe

- [ ] T018 Define how mutable bindings and mutable access are declared
- [ ] T019 Define exclusivity rules for mutable access
- [ ] T020 Define the model for shared mutation and synchronization
- [ ] T021 Define what `unsafe` permits
- [ ] T022 Define how `unsafe` blocks, functions, and modules are isolated and audited

## Effects and Concurrency

- [ ] T023 Define how effects are represented in the language
- [ ] T024 Decide whether effects use annotations, capabilities, effect types, or dedicated keywords
- [ ] T025 Define IO boundaries and how effectful operations are marked
- [ ] T026 Define the async model and async syntax
- [ ] T027 Define structured concurrency primitives
- [ ] T028 Define cancellation semantics
- [ ] T029 Define task spawning and synchronization rules

## Error Handling

- [ ] T030 Define the syntax for `Option` and `Result` usage
- [ ] T031 Define error propagation syntax
- [ ] T032 Define `panic` semantics and when panic is acceptable
- [ ] T033 Define whether custom error traits or protocols exist in the initial version

## Toolchain and Packages

- [ ] T034 Define the package manifest format
- [ ] T035 Define workspace layout and multi-package project structure
- [ ] T036 Define dependency declaration and dependency resolution rules
- [ ] T037 Define registry and package publishing strategy
- [ ] T038 Define documentation generation format and commands
- [ ] T039 Define machine-readable compiler and tooling output formats

## Formatting and Style

- [ ] T040 Define canonical formatting rules
- [ ] T041 Define import ordering and grouping rules
- [ ] T042 Define line breaking, indentation, and trailing comma rules
- [ ] T043 Define formatter behavior for partially invalid files
- [ ] T044 Define formatter stability guarantees across toolchain versions

## Implementation Strategy

- [ ] T045 Define the Rust bootstrap architecture
- [ ] T046 Define the order of implementation for parser, formatter, type checker, and compiler stages
- [ ] T047 Define the self-hosting transition criteria
- [ ] T048 Define which infrastructure may remain in Rust long-term

## Naming and Release Readiness

- [ ] T049 Run trademark review for `Tark` in target jurisdictions
- [ ] T050 Run software and developer-tool brand conflict review for `Tark`
- [ ] T051 Review domain strategy for `tark.*`
- [ ] T052 Finalize whether `.tark` is the official source file extension
