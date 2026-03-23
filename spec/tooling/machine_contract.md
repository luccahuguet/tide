# Ferox Tooling Reference: Machine Contracts

This document defines Ferox's machine-readable tooling outputs and safe automated mutation contract.

## Machine-Readable Tooling Output

Machine-readable output is an official part of the toolchain contract.

Primary direction:

* human-oriented output remains the default for terminal use
* commands that support structured output should also support an explicit machine-readable mode
* JSON is the initial official machine-readable format

### Diagnostics Output

Compiler and checker diagnostics should be available as structured data.

Initial direction:

* commands such as `ferox build`, `ferox check`, and `ferox test` may support a flag such as `--output json`
* structured diagnostics include at least severity, message, file path, span information, and diagnostic code when available
* related notes and secondary spans should be represented explicitly rather than embedded only in plain text

### Build And Tool Events

Structured output is not limited to errors.

Initial direction:

* machine-readable mode may report build status, test events, and other tool results in JSON records or JSON documents
* structured output should distinguish diagnostics from ordinary tool progress
* file paths and target identifiers should be emitted in stable machine-usable form

### Stability Expectations

Rules:

* the JSON schema should be versioned or otherwise evolve in a controlled compatible way
* human-readable terminal formatting may change more freely than machine-readable output
* machine-readable output should not require scraping styled terminal text
* stable machine-facing outputs should grow beyond diagnostics when the compiler architecture is ready, especially for symbols, project metadata, and refactor workflows

### Design Intent

The official toolchain should be scriptable and integration-friendly by design.

Goals:

* support editors, CI systems, and agent workflows directly
* eliminate brittle terminal-text scraping as the primary automation path
* keep structured output explicit rather than always-on
* preserve good human CLI output while still offering a stable automation surface

### Initial Non-Goals

The initial machine-readable output model does not include:

* multiple official structured formats beyond JSON
* exposing every internal compiler data structure as public output
* guaranteeing that ordinary human terminal output is stable for parsers

## Read-Side Machine Contract

Ferox should define an explicit read-side machine contract for tools, CI systems, editors, and agents.

Core rule:

* tools should depend on official structured outputs, not scraped terminal text or unstable internal compiler structures

The read-side contract should be divided into four output categories:

* diagnostics
* symbol and semantic identity data
* project and package metadata
* deterministic tool outputs and build-resolution facts

### Stability Tiers

Each output category should declare a stability tier.

Initial tiers:

* `stable`: intended for long-term external tooling use; changes require controlled evolution
* `provisional`: machine-readable and documented, but expected to evolve during bootstrap
* `internal`: not part of the public tooling contract

Core rule:

* ordinary bootstrap implementation details remain internal unless explicitly promoted into the read-side contract

## Diagnostics Contract

Diagnostics are the first required read-side output.

Every structured diagnostic record should eventually include at least:

* schema version
* tool or command name
* severity
* diagnostic code
* primary message
* primary file path
* primary span with start and end locations
* zero or more related spans or notes
* zero or more suggested fixes when available

Initial location shape:

* file path
* byte range
* line and column data when available

Suggested stability tier:

* `stable` once exposed through official JSON output flags for `ferox check`, `ferox build`, and related commands

Example shape:

```json
{
  "schema_version": "1",
  "tool": "ferox check",
  "diagnostics": [
    {
      "severity": "error",
      "code": "ownership.use_after_move",
      "message": "value used after move",
      "primary": {
        "file": "src/main.ferox",
        "span": { "start_byte": 42, "end_byte": 46, "start_line": 4, "start_column": 5, "end_line": 4, "end_column": 9 }
      },
      "related": [
        {
          "message": "value moved here",
          "file": "src/main.ferox",
          "span": { "start_byte": 20, "end_byte": 24, "start_line": 3, "start_column": 9, "end_line": 3, "end_column": 13 }
        }
      ],
      "suggested_fixes": []
    }
  ]
}
```

### Diagnostic Evolution Rules

Rules:

* new optional fields may be added in a controlled way
* existing stable fields must not silently change meaning
* diagnostic codes should stay stable enough to be used in tooling, tests, and documentation
* human-readable wording may improve over time, but code, span structure, and related-information shape should evolve carefully once marked stable

## Symbol And Semantic Identity Contract

Ferox should expose stable symbol identity and symbol graph data once the compiler architecture is ready.

Initial direction:

* every externally visible declaration should have a stable symbol identifier within the scope of a given package graph and toolchain contract version
* symbol records should identify declaration kind, name, defining package, defining module, visibility, and source span
* symbol graph data should expose relations such as declaration ownership, containment, and reference edges where the compiler can do so reliably

Minimum symbol record fields:

* schema version
* symbol id
* package id
* module path
* declaration kind
* local name
* fully qualified name or canonical path
* defining span
* visibility

Suggested stability tier:

* `provisional` in early bootstrap
* `stable` once symbol-based editor and refactor tooling depend on it

Non-goal:

* exposing every internal compiler node or temporary analysis artifact as a public symbol

## Project And Package Metadata Contract

Ferox should expose project and package metadata in structured form rather than requiring tools to reverse-engineer manifests and workspace structure manually.

Minimum metadata areas:

* workspace root identity
* package identities
* package roots
* target definitions
* dependency edges
* edition and toolchain-compatibility data
* lockfile or resolution facts when present
* selected-target and manifest-loading facts for the active command

Example metadata uses:

* editor workspace loading
* CI validation
* graph visualization
* agent-safe repository inspection

Suggested stability tier:

* `stable` for basic package and workspace identity fields
* `provisional` for richer derived analysis metadata until the architecture settles

## Deterministic Output And Resolution Facts

Some read-side outputs are not diagnostics or symbols, but they still matter to safe change.

These include:

* canonical formatter output for supported syntax
* deterministic dependency-resolution results
* stable target-selection and package-loading facts
* machine-readable build and analysis event records when those are exposed

Core rule:

* if a command participates in the official safe-change contract, the structured facts it emits should be deterministic for the same source tree, manifest state, lock state, and toolchain version

Suggested stability tier:

* formatter output: `stable` within the formatter compatibility policy
* dependency-resolution and package metadata facts: `stable` once exposed as official machine-readable outputs
* build or progress-event streams: `provisional` until event schemas settle

## Read-Side Contract Table

The current intended contract is:

| Output category | Minimum contents | Stability target | First expected phase |
| --- | --- | --- | --- |
| Diagnostics JSON | severity, code, message, primary span, related info, suggested fixes | stable | V1 bootstrap |
| Symbol records | stable symbol id, declaration kind, name, defining location, visibility | provisional -> stable | later V1 / V2 |
| Symbol graph | containment and reference relations where reliable | provisional | later V1 / V2 |
| Project/package metadata | workspace, package, target, dependency, edition facts | stable for core identity | V1 bootstrap |
| Deterministic formatter output | canonical output for supported syntax | stable | V1 bootstrap |
| Resolution/build facts | dependency resolution, target selection, package loading, event records | provisional -> stable by category | V1 bootstrap and later |

## Read-Side Validation Expectations

The read-side contract should carry explicit regression obligations.

Required validation directions:

* compatibility tests for stable JSON schemas and field meanings
* fixture snapshots for representative diagnostic outputs
* fixture snapshots for package/workspace metadata outputs
* regression tests for deterministic formatter output on supported syntax
* output-level regression tests for dependency-resolution and package-loading facts where they are exposed
* fixture snapshots for package-root loading, workspace-member loading, and explicit unsupported-dependency failures

Logging expectation:

* when a structured output changes unexpectedly, the toolchain should make it practical to trace which compiler or driver stage produced the changed record
* manifest parsing, workspace discovery, target selection, and package-loading failures should emit enough structured trace information that output regressions can be explained without ad hoc print debugging

## Mutation-Side Machine Contract

Ferox should define an explicit mutation-side machine contract for safe automated change.

Core rule:

* tools should mutate Ferox code through official structured edit surfaces rather than by relying on ad hoc text rewriting alone

The mutation-side contract should be divided into three classes:

* guaranteed-safe automated rewrites
* review-oriented suggested rewrites
* non-contractual internal transformations

## Mutation Safety Tiers

Each machine-driven change should declare its safety tier.

Initial tiers:

* `safe_apply`: the toolchain can apply the rewrite automatically because the preconditions are fully checked and the result is guaranteed to remain within the validated contract for that operation
* `suggest_only`: the toolchain can propose the rewrite with structured rationale, but a human should review or explicitly accept it
* `internal_only`: compiler-internal transformations not exposed as user-facing mutation surfaces

Core rule:

* Ferox should be conservative about what counts as `safe_apply`

## Official Mutation Surfaces

The initial official mutation surfaces should eventually include:

* structured diagnostic fixes
* formatter rewrites
* import normalization
* symbol-aware rename
* structural rule matches with constrained autofix

The exact rollout can be staged, but these should define the intended long-term surface.

### 1. Structured Diagnostic Fixes

Some diagnostics may include machine-applicable fixes.

Examples:

* inserting a missing delimiter when parser recovery makes the fix unambiguous
* replacing a trivially wrong token with the only syntactically valid intended token in a constrained context
* adding or removing a formatter-only syntactic normalization where meaning is unchanged

Rules:

* a diagnostic fix should identify the exact text range or structured node it intends to replace
* the fix should carry the diagnostic code that justified it
* `safe_apply` fixes must be semantics-preserving within the scope of the diagnosed issue
* fixes that may alter meaning should remain `suggest_only`

### 2. Formatter Rewrites

Formatter rewrites are the simplest official machine mutation surface.

Rules:

* formatting rewrites are `safe_apply` only for source the formatter considers confidently parseable
* formatting rewrites must preserve program meaning
* partial formatting of damaged files must preserve untouched regions when confidence is insufficient

### 3. Import And Module-Surface Normalization

Import ordering, grouping, deduplication, and similar normalization may be exposed as official machine edits.

Rules:

* these rewrites are `safe_apply` only when the compiler can prove the normalized form is semantically equivalent
* import insertion or removal that depends on unresolved names should not be applied automatically
* module-surface rewrites that affect visibility or API shape should default to `suggest_only` unless explicitly proven safe

### 4. Symbol-Aware Rename

Rename is one of the most important future mutation surfaces.

Rules:

* rename must operate on stable symbol identity rather than raw textual matching
* the tool must report every affected location before or alongside the change
* rename is `safe_apply` only when every affected reference and declaration can be resolved confidently
* unresolved references, macro-like text expansion, or ambiguous dynamic lookup should force `suggest_only` or refusal

### 5. Structural Rules And Autofix

Ferox should eventually support first-party structural rule matching with constrained autofix.

Rules:

* structural rules should match over official syntax and semantic identities rather than plain text alone
* autofix attached to a rule must declare the preconditions under which it is valid
* rule-based autofix is `safe_apply` only when the compiler can validate the transformed result against the intended guarantees of that rule
* rules that merely encode style or heuristic cleanup may still be useful, but should default to `suggest_only` unless their safety is very strong

## Mutation Packaging

Machine-driven edits should be represented in a structured form.

Initial direction:

* a mutation result should identify the targeted files
* each edit should include a stable range or node target
* the result should include the originating diagnostic, symbol action, or rule id when applicable
* edit batches should declare whether they are atomic and whether they require revalidation after application

Example logical fields:

* mutation kind
* safety tier
* originating source (`diagnostic`, `rename`, `rule`, `formatter`, etc.)
* target file
* target span or node id
* replacement content or structured edit payload
* post-apply validation requirement

## Boundaries Between Safe Apply And Suggest Only

Ferox should define explicit boundaries.

`safe_apply` should require all of the following:

* the target nodes or spans are resolved confidently
* the preconditions of the mutation are checked by the compiler or official tool
* the transformation stays within the meaning-preserving contract for that mutation kind
* the result can be revalidated immediately by parser/checker/formatter as appropriate

`suggest_only` should be used when any of the following are true:

* the tool is making a style or intent guess
* the rewrite may change semantics
* affected references are incomplete or ambiguous
* package/workspace-wide consequences are not fully known
* the transformation depends on heuristics rather than validated compiler facts

The toolchain should refuse to emit an official machine mutation entirely when:

* the source cannot be resolved confidently enough to target the right symbol or node
* recovery damage is too high
* applying the rewrite would require inventing missing code rather than transforming known code

## Mutation Auditability And Logging

Machine-applied changes should be traceable.

Expected mutation-side logging data:

* mutation kind
* safety tier
* originating diagnostic, symbol action, or rule id
* affected file set
* pre-validation status
* post-apply validation status
* refusal reason when no official mutation is emitted

Core rule:

* tools should make it easy to inspect why a change was auto-applied, downgraded to suggestion, or refused entirely

## Mutation Validation Expectations

The mutation-side contract should carry explicit regression obligations.

Required validation directions:

* scenario tests for each official mutation surface
* before/after corpus cases for formatter, diagnostics-with-fixes, and later structural rules
* rename and refactor tests that verify all affected symbol references are updated or that the tool refuses safely
* end-to-end scripts that apply machine mutations and immediately re-run parser/checker/formatter validation
* trace inspection for failed or refused machine rewrites

## Mutation Rollout Direction

The expected rollout should be conservative.

Suggested order:

1. formatter rewrites
2. structured diagnostic fixes in narrow unambiguous cases
3. import normalization
4. symbol-aware rename
5. structural rules with constrained autofix

This keeps the early mutation contract narrow enough to earn trust before it attempts larger-scale automated rewrites.
