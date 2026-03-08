# Orbis Implementation Strategy

## Implementation Strategy

Orbis will be **bootstrapped in Rust**.

Reasons:

* Rust aligns with Orbis’s safety goals
* Rust has a mature ecosystem for building compilers
* Rust provides excellent performance for language tooling
* Rust enables a fast and reliable first implementation

### Self-hosting goal

Orbis is intended to become **self-hosting**.

Plan:

1. **Phase 1 — Rust bootstrap**

   The first compiler, formatter, and toolchain are implemented in Rust.

2. **Phase 2 — Orbis transition**

   As the language stabilizes, components are gradually rewritten in Orbis:

   * formatter
   * small utilities
   * linter
   * parser
   * type checker
   * compiler frontend
   * full compiler

3. **Phase 3 — Orbis-first**

   Orbis becomes the primary implementation language for its own ecosystem.

Rust may remain in selected infrastructure layers where it continues to provide strategic value.

Principle:

> Rust brings Orbis into existence.
> Orbis proves it deserves to exist.

## Rust Bootstrap Architecture

The initial implementation should use a modular Rust workspace rather than a single monolithic binary crate.

Core direction:

* one Rust workspace contains the bootstrap toolchain components
* shared language data structures are defined in reusable library crates
* the `orb` CLI is a thin orchestration layer over those libraries

### Initial Component Layout

Initial bootstrap components:

* `orbis_cli`: command-line entrypoint and command dispatch
* `orbis_syntax`: tokens, concrete syntax tree, parser, and source span handling
* `orbis_ast`: normalized language AST where needed beyond the syntax tree
* `orbis_fmt`: formatter built on syntax and source mapping
* `orbis_sem`: name resolution, type checking, effect checking, and other semantic analysis
* `orbis_diag`: diagnostics data model and rendering
* `orbis_driver`: build orchestration, package loading, and pipeline coordination

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
* allow individual tools such as the formatter to be rewritten in Orbis independently
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
* ownership-, mutation-, and effect-related analysis as those rules become implementable

#### 4. Driver Integration Fourth

Package loading and orchestration should be integrated once parsing, formatting, and semantic checking already exist as libraries.

This stage turns the individual components into coherent `orb check`, `orb fmt`, and build-oriented workflows.

#### 5. Compiler Stages After Frontend Maturity

Lowering, code generation, and backend integration come after the frontend pipeline is credible.

Rationale:

* frontend correctness is the larger risk for Orbis right now
* code generation strategy is a separate decision area and should not force premature architecture choices
* a stable checked intermediate form should exist before backend work becomes the main focus

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

* a component should move from Rust to Orbis only when the Orbis toolchain can build, check, and validate that component with enough reliability to improve the project rather than destabilize it

### Phase 2 Entry Criteria

The project may begin rewriting selected tools in Orbis when the following are true:

* the Rust bootstrap parser, formatter, and semantic checker are stable enough to support day-to-day development
* `orb check` and `orb fmt` are usable on the project's own codebase
* diagnostics are good enough that language-tooling bugs are debuggable without extraordinary effort
* package loading, workspace handling, and deterministic builds are in place for the project's own repository

At that point, the first Orbis rewrites should target lower-risk components such as:

* formatter
* small utilities
* linter

### Phase 3 Entry Criteria

Orbis should be considered meaningfully self-hosting only when the following are true:

* the Orbis implementation can build the primary Orbis compiler or compiler frontend from source
* the resulting artifacts can rebuild themselves reproducibly or with only narrowly understood bootstrap steps
* the Orbis-based implementation passes the same language and tooling test suites expected of the Rust bootstrap
* the Orbis-based toolchain is usable for normal project development without falling back to Rust for routine iteration

### Practical Definition

Initial practical interpretation:

* rewriting one tool in Orbis does not by itself make the project self-hosting
* the project becomes self-hosting in stages as more of the trusted development path is handled by Orbis
* the milestone that matters is whether Orbis is the primary language used to build and evolve Orbis itself

### Design Intent

The self-hosting milestone should mark real operational capability rather than project theater.

Goals:

* prevent premature rewrites that slow the project down
* move components in an order that matches toolchain maturity
* make the self-hosting claim technically meaningful
* preserve the Rust bootstrap as a reliability baseline until Orbis genuinely earns the handoff

## Long-Term Rust Boundary

Orbis should become the primary implementation language for language-facing tooling, but not every supporting layer must be rewritten for ideological purity.

Core rule:

* Rust may remain long-term for infrastructure that is strategically low-level, heavily platform-bound, or not central to Orbis's language-design value

### Orbis-First Targets

The following should be preferred candidates for eventual Orbis ownership:

* formatter
* linter
* parser
* semantic checker
* compiler frontend
* core developer-facing command logic where Orbis is exercising its own language strengths

These are the parts most directly tied to proving that Orbis can build and maintain real language tooling.

### Rust-Permitted Long-Term Layers

The following may reasonably remain in Rust if doing so continues to provide clear engineering leverage:

* low-level runtime bootstrap support
* platform-specific integration layers
* deeply optimized backend or systems-integration shims
* infrastructure wrappers around external toolchains or host APIs
* narrowly scoped bootstrap utilities that do not materially affect day-to-day Orbis development

### Decision Standard

Rust retention should be judged pragmatically.

Questions to ask:

* does rewriting this component in Orbis materially strengthen the language or its ecosystem?
* is the component part of the trusted developer-facing path that Orbis should own?
* would a Orbis rewrite reduce complexity, or merely duplicate stable low-level infrastructure?
* does the component depend heavily on platform APIs or mature Rust ecosystem libraries that are not worth reimplementing early?

### Design Intent

The project should pursue self-hosting where it proves Orbis's value, not where it creates ceremonial work.

Goals:

* keep Orbis as the primary language for the language's own core tooling
* avoid performative rewrites of low-level support code
* preserve the option to use Rust where it remains the sharper engineering choice
* keep the self-hosting claim meaningful without demanding total infrastructure purity

## Code Generation And Backend Strategy

Orbis should use a phased backend strategy rather than committing immediately to one permanent backend for all stages of the project.

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
* aligns with Orbis's current need to validate frontend and compiler architecture before chasing maximum optimization

### Phase 3: Long-Term Optimization Options

Higher-complexity backend work should come only after the frontend and first native backend are credible.

Initial direction:

* `LLVM` remains a possible later backend for stronger optimization or broader low-level targeting needs
* the project may ultimately support more than one backend if the engineering cost is justified
* backend plurality is a later optimization decision, not a requirement for the first working compiler

### Design Intent

The backend strategy should keep Orbis from front-loading complexity that does not yet help the language prove itself.

Goals:

* get to a working compiler sooner
* keep backend complexity proportional to project maturity
* separate frontend language risk from backend optimization work
* leave room for a stronger long-term performance story without making it a blocker for bring-up
