# Ferox Implementation Strategy

## Implementation Strategy

Ferox will be **bootstrapped in Rust**.

Reasons:

* Rust aligns with Ferox’s safety goals
* Rust has a mature ecosystem for building compilers
* Rust provides excellent performance for language tooling
* Rust enables a fast and reliable first implementation

### Self-hosting goal

Ferox is intended to become **self-hosting**.

Plan:

1. **Phase 1 — Rust bootstrap**

   The first compiler, formatter, and toolchain are implemented in Rust.

2. **Phase 2 — Ferox transition**

   As the language stabilizes, components are gradually rewritten in Ferox:

   * formatter
   * small utilities
   * linter
   * parser
   * type checker
   * compiler frontend
   * full compiler

3. **Phase 3 — Ferox-first**

   Ferox becomes the primary implementation language for its own ecosystem.

Rust may remain in selected infrastructure layers where it continues to provide strategic value.

Principle:

> Rust brings Ferox into existence.
> Ferox proves it deserves to exist.

## Version Roadmap Alignment

Implementation phases and language versions are related, but they are not the same thing.

Core rule:

* rewriting toolchain components in Ferox is part of the implementation path around the v1 language era, not automatically a v2 language event

Current roadmap:

* v1: narrow language core plus Rust bootstrap
* later v1-era implementation maturity: selected Ferox rewrites and self-hosting transition
* v2 and v3: only for deferred language expansion after real pressure justifies it

See:

* [ROADMAP.md](../ROADMAP.md)
* [V2_BACKLOG.md](../V2_BACKLOG.md)
* [V3_BACKLOG.md](../V3_BACKLOG.md)

## Rust Bootstrap Architecture

The initial implementation should use a modular Rust workspace rather than a single monolithic binary crate.

Core direction:

* one Rust workspace contains the bootstrap toolchain components
* shared language data structures are defined in reusable library crates
* the `ferox` CLI is a thin orchestration layer over those libraries
* the architecture should support fast incremental checking and reusable compiler services from the start where practical
* the compiler should trend toward a query-oriented architecture so checking, diagnostics, refactors, and later agent tooling can share one dependency-aware core

### Initial Component Layout

Initial bootstrap components:

* `ferox_cli`: command-line entrypoint and command dispatch
* `ferox_syntax`: tokens, full-fidelity concrete syntax tree, parser, and source span handling
* `ferox_ast`: normalized language AST where needed beyond the syntax tree
* `ferox_fmt`: formatter built on syntax and source mapping
* `ferox_sem`: name resolution, type checking, ownership and async-related analysis, and other semantic analysis
* `ferox_diag`: diagnostics data model and rendering
* `ferox_driver`: build orchestration, package loading, and pipeline coordination

This structure keeps the formatter, diagnostics, parser, and semantic passes usable as libraries instead of only as subcommands inside one binary.

### Architectural Rules

Rules:

* syntax and diagnostics types should be shareable across formatter, checker, and later compiler stages
* the CLI should not contain core language logic directly
* package loading and build orchestration should remain separate from parsing and semantic analysis
* machine-readable output should be produced from structured diagnostics and tool events, not reconstructed from human terminal text
* compiler architecture should favor incremental reuse and tool-facing data rather than one-shot batch-only execution
* internal boundaries should support future structural rule engines and safe autofix tooling without requiring a second parser or ad hoc source analysis layer

### Design Intent

The Rust bootstrap architecture should make later self-hosting incremental rather than all-or-nothing.

Goals:

* keep component boundaries explicit
* allow individual tools such as the formatter to be rewritten in Ferox independently
* avoid tight coupling between the CLI and the compiler pipeline
* support testing of parser, formatter, and semantic logic as standalone libraries
* keep the path open for query-oriented compiler services instead of a CLI-only architecture
* keep the path open for first-party structural refactor and autofix tooling built on the same normalized compiler data

## Query-Oriented Compiler Architecture

Ferox should treat a query-oriented compiler architecture as an explicit direction, not just a vague preference.

Core rule:

* the compiler should be organized around reusable dependency-aware services keyed by stable identities, rather than around one-shot command pipelines only

This does not require a specific framework choice in v1, but it does require the architecture to behave as if:

* data is requested through well-defined services
* service results are cacheable
* dependencies between results are explicit enough for incremental invalidation
* CLI, IDE, refactor, and later verification workflows can reuse the same core computations

### Core Service Graph

The initial service graph should be close to the following shape:

```text
source text
  -> tokenization
  -> concrete syntax tree
  -> recovery-aware parse diagnostics
  -> normalized AST / lowered syntax view
  -> name resolution
  -> type + ownership analysis
  -> checked IR
  -> machine diagnostics
  -> symbol records / symbol graph
  -> formatter support
  -> driver/build/package metadata views
```

The exact crate boundaries may evolve, but these service boundaries should remain conceptually stable.

### Initial Query Families

The architecture should support queries or equivalent reusable services for at least:

* source file loading
* file hashing or version identity
* token stream generation
* full-fidelity parse tree generation
* parse diagnostics
* lowered AST generation where needed
* package and module loading
* name resolution
* type checking
* ownership and concurrency-related checking as those rules exist
* checked IR generation
* symbol record generation
* symbol graph generation
* formatter input views
* machine-readable diagnostics output

### Stable Identity Model

The query system should key work on stable identities rather than ephemeral traversal state.

Minimum identity layers:

* workspace id
* package id
* target id
* file id
* module id
* item id
* symbol id once symbol generation exists

Core rule:

* identities should remain stable across repeated queries for the same unchanged source tree and toolchain context

### Cache Boundaries

The bootstrap architecture should assume at least these cache boundaries:

* source text by file
* tokenization by file
* parse tree by file
* parse diagnostics by file
* lowered syntax by file
* name-resolution results by package or module
* semantic checking by item or item-group where practical
* checked IR by item or target
* symbol data by package or module
* package/workspace metadata by manifest graph

Core rule:

* cache boundaries should be chosen so that a small edit invalidates only the smallest trustworthy slice of downstream work

### Invalidation Model

Single-file edits should stay cheap whenever their effects are local.

Initial direction:

* editing one file should invalidate that file's source, tokens, parse tree, parse diagnostics, and directly dependent lowered views
* module/package-level name resolution should only be recomputed when an edit can change visible declarations, imports, or module structure
* item-level semantic checks should be recomputed for affected items and their direct dependents, not the whole project by default
* package metadata and dependency-resolution caches should be invalidated only by manifest or workspace changes

Non-goal:

* promising maximal incrementality before the dependency model is implemented well enough to justify it

### CLI And Tool Reuse

The same underlying services should power:

* `ferox check`
* `ferox fmt`
* machine-readable diagnostics output
* editor integrations
* future rename/refactor tooling
* future structural-rule and autofix tooling
* later verification-oriented analyses where applicable

Core rule:

* user-facing commands may orchestrate multiple services, but they should not reimplement core language analysis separately

### V1 Architectural Non-Goals

The v1 architecture does not need:

* a distributed or remote query engine
* whole-program perfect incrementality
* stable public exposure of every internal intermediate representation
* speculative background analysis of every possible future query
* plugin-driven arbitrary compiler mutation hooks

The goal is a small, explicit, reusable architecture, not an infrastructure research project.

### Validation Expectations For Query Architecture

The architecture should be validated directly.

Required validation directions:

* unit tests for identity stability and invalidation rules
* corpus tests that compare cold and incremental results for the same inputs
* integration tests for parse -> diagnostics -> formatter -> semantic service reuse
* end-to-end scripts that run repeated edit/check flows and assert stable machine outputs
* trace inspection for query execution order, cache hits, cache misses, and invalidation breadth

### Logging And Trace Requirements

The query-oriented architecture should make incremental behavior visible.

Minimum trace areas:

* service/query start and finish
* identity key used
* cache hit versus miss
* invalidation cause
* dependency fan-out where practical
* output production stage for diagnostics, symbols, and metadata

## Implementation Order

The bootstrap implementation should be staged so that each step validates language decisions before the next major subsystem is built.

Initial order:

1. source handling, tokens, spans, and parser infrastructure
2. formatter on top of parsed syntax and source mapping
3. name resolution and semantic checking
4. package loading and build/check driver integration
5. compiler lowering and backend-facing compilation stages

### Stage Rationale

#### 1. Syntax First

The parser and source-model layer comes first.

Reasons:

* every later tool depends on stable syntax and source spans
* formatter and diagnostics both need a trustworthy parsed structure
* syntax work exposes grammar problems early while the language is still changing quickly
* full-fidelity syntax and local recovery are prerequisites for trustworthy formatting, refactors, and agent workflows

#### 2. Formatter Second

The formatter should come before the full type checker.

Reasons:

* it validates that the syntax tree is rich enough for real tooling
* it produces immediate value for examples, tests, and generated code
* it exercises source mapping and error recovery without requiring the full semantic model

#### 3. Semantics Third

Semantic analysis follows once syntax and formatting are stable enough to support serious language checks.

Initial scope:

* name resolution
* visibility checking
* type checking
* trait and generic validation
* ownership-, mutation-, and async-related analysis as those rules become implementable

#### 4. Driver Integration Fourth

Package loading and orchestration should be integrated once parsing, formatting, and semantic checking already exist as libraries.

This stage turns the individual components into coherent `ferox check`, `ferox fmt`, and build-oriented workflows.

It should also establish the architecture for fast incremental checking and structured machine-facing outputs.

#### 5. Compiler Stages After Frontend Maturity

Lowering, code generation, and backend integration come after the frontend pipeline is credible.

Rationale:

* frontend correctness is the larger risk for Ferox right now
* code generation strategy is a separate decision area and should not force premature architecture choices
* a stable checked intermediate form should exist before backend work becomes the main focus

### Checked IR Direction

The checked intermediate form should not be a thin dump of surface syntax.

Current direction:

* Ferox should prefer a **regularized checked IR** for semantic reasoning, diagnostics, later optimization work, and backend lowering

This means:

* lower multiple surface forms into a smaller set of semantic forms where practical
* make ownership-relevant operations explicit in the checked IR
* make control-flow and async structure more explicit than they are in the parsed syntax
* keep the IR predictable enough that later tooling and static analyses do not need to reverse-engineer meaning from raw source syntax

Non-goal:

* redesign the source language around Bosque-style radical source restrictions just to simplify the IR

The point is to make compiler reasoning smaller and more regular without widening the user-facing language core.

### Design Intent

The order should optimize for fast feedback on language design rather than rushing to native code generation.

Goals:

* validate syntax and tooling ergonomics early
* avoid backend complexity before the frontend is trustworthy
* create useful standalone tools before the full compiler exists
* keep the self-hosting path incremental

## Semantic Corpus And Golden Tests

Ferox should maintain a semantic corpus early in the bootstrap.

Purpose:

* lock down syntax expectations
* lock down formatter stability
* lock down diagnostic shape and wording quality where appropriate
* provide regression coverage for ownership and async edge cases
* create a trustworthy basis for later structural rules and autofix behavior

Suggested early corpus areas:

* parser acceptance and rejection cases
* formatter input/output pairs
* structured diagnostic snapshots
* ownership and borrow-check examples
* machine-readable output snapshots
* later rule-and-autofix before/after cases

## Shared Validation And Observability Harness

Ferox should use one shared validation and observability harness across bootstrap compiler and tooling work.

Core rule:

* parser, formatter, diagnostics, semantic analysis, and CLI work should plug into one validation system instead of inventing separate test and logging conventions per subsystem

The harness should define:

* test layers
* fixture and snapshot layout
* e2e script conventions
* logging and tracing conventions
* failure artifact capture
* local and CI invocation paths

### Test Layers

The harness should use four primary layers.

#### 1. Unit Tests

Purpose:

* fast local feedback for small components
* direct coverage of low-level logic without file-based fixtures when that would add noise

Typical scope:

* span math
* token behavior
* syntax helpers
* diagnostic construction
* formatter primitives
* small semantic utilities

#### 2. Corpus And Golden Tests

Purpose:

* lock down externally visible behavior through small source fixtures and expected outputs

Typical scope:

* parser acceptance and rejection
* formatter input/output pairs
* structured diagnostics snapshots
* ownership pass/fail examples
* machine-readable output snapshots

Core rule:

* whenever output shape matters to users or tools, prefer fixture-driven golden tests over ad hoc inline assertions

#### 3. Integration Tests

Purpose:

* validate interactions between crates or pipeline stages without requiring full CLI orchestration

Typical scope:

* syntax plus diagnostics integration
* formatter over parsed syntax
* driver plus package-loading interactions
* metadata and machine-readable output generation

#### 4. End-To-End Scripts

Purpose:

* exercise the same command surface that users, CI, and agents will actually run

Typical scope:

* `ferox check` success and failure cases
* `ferox fmt` formatting and idempotence checks
* workspace/package-loading scenarios
* machine-readable output smoke tests

Core rule:

* e2e scripts should be explicit, debuggable shell entrypoints rather than opaque CI-only glue

### Recommended Layout

The initial shared layout should look like:

```text
crates/
  ferox_syntax/
    src/
      ...
    tests/
      ...
  ferox_fmt/
    src/
      ...
    tests/
      ...
  ...
tests/
  integration/
  corpus/
    parser/
      ok/
      err/
    fmt/
      in/
      out/
    diag/
      cases/
      expected/
    ownership/
      pass/
      fail/
    machine/
      diagnostics/
      metadata/
  e2e/
    check/
    fmt/
    workspace/
  artifacts/
    .gitkeep
scripts/
  test/
    e2e_check.sh
    e2e_fmt.sh
    update_snapshots.sh
```

Rules:

* unit tests should live with the crate code they exercise, using normal Rust crate-local test conventions
* top-level `tests/` should be reserved for integration, corpus/golden, and end-to-end material that spans crate boundaries or validates externally visible behavior

This layout is a direction, not a frozen path contract, but the categories should remain stable even if exact file placement evolves.

### Fixture And Snapshot Naming

Naming should be mechanical and predictable.

Rules:

* use short descriptive fixture names based on the behavior under test rather than task ids
* pair source fixtures and expected outputs by basename where possible
* use `.expected.json` for structured output snapshots
* use `.expected.ferox` for formatter or canonical-source snapshots
* prefer one behavior family per fixture rather than giant kitchen-sink examples

Examples:

```text
tests/corpus/ownership/fail/use_after_move.ferox
tests/corpus/diag/expected/use_after_move.expected.json
tests/corpus/fmt/in/multiline_match.ferox
tests/corpus/fmt/out/multiline_match.expected.ferox
```

### Snapshot Update Workflow

Snapshot updates should be explicit.

Rules:

* failing golden tests should never silently rewrite expected outputs
* snapshot regeneration should happen through an explicit update command or script
* the update workflow should make it obvious which snapshots changed and why

Initial direction:

* provide one official snapshot-update script rather than ad hoc per-test update flags scattered across crates

### Logging And Tracing Model

Ferox should use structured logging and tracing from the beginning where practical.

Core rule:

* every major pipeline stage should emit structured events that are useful in debugging but suppressible in normal runs

Minimum event categories:

* package and workspace loading
* file discovery and source loading
* tokenization
* parse decisions and recovery
* formatter passes and idempotence failures
* semantic phase entry/exit
* ownership and concurrency checking decisions where relevant
* machine-output emission
* CLI command dispatch and exit status

Minimum common fields where applicable:

* command or phase name
* file or package identifier
* target identifier
* symbol or item identifier when available
* elapsed time or step timing where practical
* outcome status

### Trace Controls

Initial direction:

* normal developer commands should stay concise by default
* verbose tracing should be enabled explicitly through flags, environment variables, or both

Suggested controls:

* `--verbose` for human-readable additional detail
* an environment variable such as `FEROX_TRACE` for structured tracing selection during local debugging and CI artifact capture

The exact mechanism may evolve, but there should be one clear official path rather than crate-specific hidden toggles.

### Failure Artifact Capture

The harness should preserve enough failure context that regressions are easy to debug.

Expected artifacts where relevant:

* structured diagnostics output
* actual-versus-expected snapshot diffs
* captured stdout/stderr from e2e commands
* trace logs for failing runs when verbose mode is enabled

Core rule:

* failure artifacts should be stored in stable, easy-to-inspect locations rather than buried inside transient CI output only

### Local And CI Invocation Model

The same harness should support:

* focused local iteration
* broader pre-commit validation
* CI execution
* agent-driven targeted runs

Initial direction:

* offer a small set of stable entrypoints for running unit, corpus, integration, and e2e layers
* keep full-suite and focused-suite commands obvious from repository scripts or cargo aliases
* do not require contributors to memorize crate-specific test incantations for common workflows

Example workflow categories:

* fast unit run
* focused corpus run by area
* snapshot update run
* full bootstrap validation run
* e2e smoke run

### Phased Adoption Across T072-T076

The bootstrap tasks should adopt the harness in order.

#### T072 Workspace Skeleton

Must establish:

* top-level test and script locations
* baseline cargo/test wiring
* a place for failure artifacts and future corpora

#### T073 Source, Spans, Tokens, Diagnostics, Syntax Scaffolding

Must add:

* unit tests for low-level source and token behavior
* initial corpus fixture structure
* structured trace hooks for source loading, tokenization, and diagnostic construction

#### T074 Parser And Structured Diagnostics

Must add:

* parser ok/err corpus fixtures
* structured diagnostic snapshots
* parser recovery and diagnostic e2e smoke coverage

#### T075 Formatter

Must add:

* formatter in/out golden fixtures
* idempotence checks
* formatter trace points and mismatch artifacts

#### T076 CLI Commands

Must add:

* command-level e2e scripts
* stdout/stderr capture conventions
* machine-readable output smoke tests through the actual CLI

### Forward-Compatibility Rule

The harness should be able to absorb later:

* richer ownership cases
* concurrency validation
* contracts and verification examples
* structural-rule and autofix scenarios

without changing its basic layers or naming philosophy.

## Safety Invariant Validation

The ownership model should not be treated as "specified" merely because the prose sounds plausible.

The bootstrap must validate the safe-code guarantee boundary and core ownership invariants directly.

Early implementation obligations:

* represent ownership-, borrow-, and sharing-relevant states explicitly enough that checker behavior is inspectable
* produce diagnostics that point to both the violating access and the earlier move, borrow, or conflicting access that caused the violation
* keep ownership failures easy to regress through pass/fail corpus cases rather than relying only on ad hoc examples

Suggested early validation buckets:

* moved-value use rejection
* copy-eligibility checking for user-defined types
* rejection of `copy` on types that contain non-copy fields or ownership-sensitive resources
* overlapping shared and mutable borrow rejection
* overlapping mutable borrow rejection
* lexical borrow-end behavior
* deliberate rejection of disjoint-field simultaneous mutable borrows in the initial checker if that remains the chosen v1 rule
* partial-move rejection for ordinary aggregates in the initial checker if that remains the chosen v1 rule
* explicit shared-ownership boundary behavior
* cross-context ownership transfer and synchronized-sharing restrictions
* safe construction and invalid-state rejection for enums where applicable

### Concurrency-Focused Validation

The bootstrap should also validate the explicit concurrency access modes directly.

Suggested concurrency validation buckets:

* move-based task capture
* copy-safe task capture
* rejection of ordinary borrow capture across task boundaries
* explicit `SyncShared[...]` sharing across child tasks
* explicit synchronized shared mutation through `Mutex[...]` or `RwLock[...]`
* structured completion and unfinished-child-task rejection
* rejection of pseudo-ability-style concurrency declarations if the language intentionally does not support them

Expected test layers:

* unit tests for concurrency-checking rules where that logic is separable
* scenario tests for allowed and rejected task-capture patterns
* end-to-end scripts that exercise `async let`, `await`, and structured early-exit behavior

Logging expectation:

* task creation, synchronization, rejection of invalid capture modes, and structured cleanup should all be traceable enough that concurrency failures are debuggable without reverse-engineering the scheduler model
* copy-eligibility resolution and gated-operation failures should be traceable enough that users can see why a type was treated as move-only rather than copy-safe

The semantic corpus should therefore include:

* compile-pass examples that witness each intended invariant
* compile-fail examples that violate each invariant in one clear way
* structured diagnostic snapshots for each failure family
* end-to-end scripts that exercise representative edit, check, and failure-reporting flows

## FP Profile Validation

The Ferox FP profile also needs direct validation rather than being left as aesthetic preference.

The bootstrap should validate accepted FP-shaped forms and rejected FP-heavy anti-pattern directions through parser, lowering, formatting, diagnostics, and CLI workflows.

Suggested accepted-form buckets:

* ADT-heavy examples with exhaustive `match`
* expression-oriented blocks that return values without obscuring control flow
* small helper-function transformations written as ordinary nested calls or explicit named steps
* explicit `Option` / `Result` transformation flows using `match`, `if let`, `while let`, and `?`
* nested destructuring that stays within the explicitly supported pattern forms
* public API examples that use explicit enums, `Option`, and `Result` rather than sentinel or flag-driven state encoding

Suggested rejection or non-goal buckets:

* operator-heavy or ambiguous pipeline-style experiments that the language intentionally does not support
* syntax or diagnostics that would imply multiple equivalent FP composition styles for ordinary code
* unsupported advanced type-level abstractions that Ferox deliberately keeps out of scope
* unsupported pattern-surface growth such as guards, or-patterns, alias patterns, ranges, or collection-spread patterns
* public-modeling examples that hide meaningful states behind primitive flags or sentinel values when typed alternatives are clearer

Expected validation layers:

* unit tests for parsing and lowering of accepted expression-oriented and pattern-heavy forms
* formatter tests that lock down canonical layout for multiline `match` and nested destructuring
* diagnostic tests for malformed or deliberately unsupported FP-adjacent constructs
* diagnostic tests for deliberately unsupported pattern forms
* corpus fixtures for canonical accepted examples and representative rejected anti-patterns
* end-to-end scripts that exercise transformation-heavy workflows through `ferox check` and `ferox fmt`

Suggested early corpus examples:

* ADT-based state classification
* `Result` propagation with local transformation steps
* nested `Option` / `Result` destructuring
* multiline expression-oriented block formatting
* local block-scoped mutation followed by a returned immutable value
* rejected guard, or-pattern, and range-pattern examples with explicit diagnostics
* public API examples contrasting explicit enum modeling against rejected or discouraged flag/sentinel encoding
* module-scale examples that keep one domain concept, public typed entrypoints, and private helper transformations in a regular order

Logging and tracing expectation:

* parse, lower, and format stages should emit enough structured trace data that failures involving pattern-heavy syntax and expression blocks can be diagnosed without guessing which stage misinterpreted the source
* formatter traces should make block-layout and multiline `match` decisions easy to inspect on the canonical source-personality examples
* diagnostics for discouraged or unsupported source forms should point to the specific style or syntax boundary Ferox is protecting rather than failing as generic parse noise

## Verification-Layer Validation

If Ferox later adds an optional contract/spec layer, that layer should be validated as an extension of the safe-change toolchain rather than as a separate proof world.

First-layer validation goals:

* contracts should attach to ordinary executable items without changing their base ownership semantics
* ghost or spec constructs should be clearly erased from runtime lowering
* verification failures should produce structured diagnostics through the same machine-facing contract used for ordinary checking
* verified modules and ordinary modules should coexist without forcing proof-oriented ceremony into unrelated code

Suggested first-layer validation buckets:

* satisfied `requires` / `ensures` examples on ordinary executable functions
* rejected contract cases with precise failing-condition diagnostics
* spec-function purity and non-executable-use restrictions
* ghost-code erasure and rejection of ghost values leaking into runtime state
* ownership-sensitive contract examples involving borrowing, mutation, and state transitions
* mixed projects where only selected modules opt into contracts or spec code

Expected validation layers:

* unit tests for contract attachment, spec-function checking, ghost erasure, and verifier-facing item classification
* pass/fail corpus cases for accepted contract style and deliberately rejected proof-heavy or runtime-leaking forms
* machine-readable diagnostic snapshots for contract failures and spec-boundary errors
* end-to-end scripts that run ordinary `ferox check` and contract-enabled verification flows side by side

Suggested early examples:

* a sorted-input precondition plus search postcondition
* a state-transition contract on a small mutable domain type
* a rejected ghost-to-runtime leak
* a rejected proof-heavy construct that the first layer intentionally keeps out of scope

Logging and tracing expectation:

* verification traces should identify contract attachment points, erased ghost/spec items, and the specific failing obligation when verification does not succeed
* ordinary compiler traces should remain understandable even when verification is disabled for most of the project

Related ownership-checking expectation:

* the bootstrap checker should make ownership-state transitions and conflict detection debuggable enough that a failing corpus test can be diagnosed without reverse-engineering the entire checker

## Self-Hosting Transition Criteria

Self-hosting is a gated transition, not just a symbolic rewrite milestone.

Core rule:

* a component should move from Rust to Ferox only when the Ferox toolchain can build, check, and validate that component with enough reliability to improve the project rather than destabilize it

### Phase 2 Entry Criteria

The project may begin rewriting selected tools in Ferox when the following are true:

* the Rust bootstrap parser, formatter, and semantic checker are stable enough to support day-to-day development
* `ferox check` and `ferox fmt` are usable on the project's own codebase
* diagnostics are good enough that language-tooling bugs are debuggable without extraordinary effort
* package loading, workspace handling, and deterministic builds are in place for the project's own repository

At that point, the first Ferox rewrites should target lower-risk components such as:

* formatter
* small utilities
* linter

### Phase 3 Entry Criteria

Ferox should be considered meaningfully self-hosting only when the following are true:

* the Ferox implementation can build the primary Ferox compiler or compiler frontend from source
* the resulting artifacts can rebuild themselves reproducibly or with only narrowly understood bootstrap steps
* the Ferox-based implementation passes the same language and tooling test suites expected of the Rust bootstrap
* the Ferox-based toolchain is usable for normal project development without falling back to Rust for routine iteration

### Practical Definition

Initial practical interpretation:

* rewriting one tool in Ferox does not by itself make the project self-hosting
* the project becomes self-hosting in stages as more of the trusted development path is handled by Ferox
* the milestone that matters is whether Ferox is the primary language used to build and evolve Ferox itself

### Design Intent

The self-hosting milestone should mark real operational capability rather than project theater.

Goals:

* prevent premature rewrites that slow the project down
* move components in an order that matches toolchain maturity
* make the self-hosting claim technically meaningful
* preserve the Rust bootstrap as a reliability baseline until Ferox genuinely earns the handoff

## Long-Term Rust Boundary

Ferox should become the primary implementation language for language-facing tooling, but not every supporting layer must be rewritten for ideological purity.

Core rule:

* Rust may remain long-term for infrastructure that is strategically low-level, heavily platform-bound, or not central to Ferox's language-design value

### Ferox-First Targets

The following should be preferred candidates for eventual Ferox ownership:

* formatter
* linter
* parser
* semantic checker
* compiler frontend
* core developer-facing command logic where Ferox is exercising its own language strengths

These are the parts most directly tied to proving that Ferox can build and maintain real language tooling.

### Rust-Permitted Long-Term Layers

The following may reasonably remain in Rust if doing so continues to provide clear engineering leverage:

* low-level runtime bootstrap support
* platform-specific integration layers
* deeply optimized backend or systems-integration shims
* infrastructure wrappers around external toolchains or host APIs
* narrowly scoped bootstrap utilities that do not materially affect day-to-day Ferox development

### Decision Standard

Rust retention should be judged pragmatically.

Questions to ask:

* does rewriting this component in Ferox materially strengthen the language or its ecosystem?
* is the component part of the trusted developer-facing path that Ferox should own?
* would a Ferox rewrite reduce complexity, or merely duplicate stable low-level infrastructure?
* does the component depend heavily on platform APIs or mature Rust ecosystem libraries that are not worth reimplementing early?

### Design Intent

The project should pursue self-hosting where it proves Ferox's value, not where it creates ceremonial work.

Goals:

* keep Ferox as the primary language for the language's own core tooling
* avoid performative rewrites of low-level support code
* preserve the option to use Rust where it remains the sharper engineering choice
* keep the self-hosting claim meaningful without demanding total infrastructure purity

## Code Generation And Backend Strategy

Ferox should use a phased backend strategy rather than committing immediately to one permanent backend for all stages of the project.

Core rule:

* the backend plan should optimize first for language bring-up and compiler progress, then later for peak optimization where justified

### Phase 1: Bring-Up Path

The earliest executable path should prioritize simplicity and feedback speed.

Initial direction:

* early compiler work may use an interpreter, evaluator, or otherwise simple execution path for semantic validation before full native-code maturity
* this stage exists to validate language behavior, not to establish the final performance story

### Phase 2: First Native Backend

The first serious native backend should favor implementation simplicity and fast iteration.

Decision:

* `Cranelift` is the preferred first native backend direction

Rationale:

* simpler integration than LLVM
* faster bring-up for a new language
* good fit for early compiler iteration and reasonable native performance
* aligns with Ferox's current need to validate frontend and compiler architecture before chasing maximum optimization

### Phase 3: Long-Term Optimization Options

Higher-complexity backend work should come only after the frontend and first native backend are credible.

Initial direction:

* `LLVM` remains a possible later backend for stronger optimization or broader low-level targeting needs
* the project may ultimately support more than one backend if the engineering cost is justified
* backend plurality is a later optimization decision, not a requirement for the first working compiler
* uniqueness- or ownership-driven storage reuse inspired by work such as Perceus may be worth exploring later, but only as an implementation optimization after the checked IR and first backend are already credible

### Functional Source, Reuse-Oriented Backend

Ferox should keep this idea only as a long-term implementation direction, not as a language promise.

What the idea means here:

* source code may remain expression-oriented and transformation-friendly
* the compiler may later prove that some temporary values are unique, dead after use, or otherwise safely reusable
* when that proof holds, lowering and backend passes may reuse storage or emit in-place update sequences without changing Ferox source semantics

What Ferox must not promise:

* users must not be required to write purely functional code to get good performance
* users must not depend on reuse happening for correctness
* optimized reuse must not change observable ownership, aliasing, drop timing, or synchronization rules at the language level
* the language should not grow surface mutation shortcuts merely to make this optimization easier

Why this is only a later direction:

* Ferox already has explicit mutation for code that genuinely wants in-place behavior
* the optimization only matters after the checked IR, ownership tracking, and backend pipeline are mature enough to prove reuse safely
* pushing this idea too early would distort language design around speculative backend wins

Prerequisites:

* a checked IR that makes ownership transfer, copy-safety, destruction points, and shared-ownership boundaries explicit
* stable value identity through lowering so reuse decisions can be explained and audited
* backend passes that can distinguish unique owned values from borrowed, shared, synchronized, or otherwise non-reusable values
* debug and non-optimized modes that preserve a credible explanation path for why reuse did or did not occur

Non-goals:

* introducing a garbage-collection or reference-counting mandate as part of the language identity
* replacing explicit mutable APIs with hidden optimizer magic as the primary performance story
* promising whole-program reuse analysis in the first native backend

Suggested validation and benchmark plan:

* benchmark transformation-heavy but ownership-friendly examples against explicit mutable equivalents
* validate that optimized reuse preserves diagnostics, value semantics, and destruction behavior
* include IR snapshots or debug traces for representative reuse wins and deliberate non-reuse cases
* require pass/fail regression tests for cases where reuse must be rejected because a value is borrowed, shared, synchronized, or observed after the candidate update point

Logging and observability expectation:

* optimized builds should be able to emit reuse decisions for selected functions
* traces should identify why a candidate value was reusable or why reuse was rejected
* debugging optimized code should not require guessing whether hidden in-place lowering happened

### Design Intent

The backend strategy should keep Ferox from front-loading complexity that does not yet help the language prove itself.

Goals:

* get to a working compiler sooner
* keep backend complexity proportional to project maturity
* separate frontend language risk from backend optimization work
* leave room for a stronger long-term performance story without making it a blocker for bring-up
