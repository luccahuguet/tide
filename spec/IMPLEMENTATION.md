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

### Initial Component Layout

Initial bootstrap components:

* `ferox_cli`: command-line entrypoint and command dispatch
* `ferox_syntax`: tokens, concrete syntax tree, parser, and source span handling
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

### Design Intent

The Rust bootstrap architecture should make later self-hosting incremental rather than all-or-nothing.

Goals:

* keep component boundaries explicit
* allow individual tools such as the formatter to be rewritten in Ferox independently
* avoid tight coupling between the CLI and the compiler pipeline
* support testing of parser, formatter, and semantic logic as standalone libraries

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
* reuse and in-place-update analysis inspired by work such as Perceus may be worth exploring later, but only after the checked IR and first backend are already credible

### Design Intent

The backend strategy should keep Ferox from front-loading complexity that does not yet help the language prove itself.

Goals:

* get to a working compiler sooner
* keep backend complexity proportional to project maturity
* separate frontend language risk from backend optimization work
* leave room for a stronger long-term performance story without making it a blocker for bring-up
