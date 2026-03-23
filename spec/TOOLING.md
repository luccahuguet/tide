# Ferox Tooling And Packages

## Formatting Policy

Ferox defines a **single canonical source format**.

Canonical formatting is part of the standard workflow, but formatting is not a prerequisite for executing otherwise valid code.

If a file is syntactically valid but **not canonically formatted**:

* `ferox build`
* `ferox run`
* `ferox test`

must still operate on the program normally.

Formatting compliance is enforced by:

* `ferox fmt`
* `ferox fmt --check`
* lint or CI policy

Formatting is enforced by the official formatter, not by making ordinary execution commands reject valid code.

Example command:

ferox fmt

Goals:

* eliminate style ambiguity
* improve mechanically safer change
* reduce review noise
* stabilize refactoring and code generation
* ensure uniform codebases

## Canonical Formatting Rules

Ferox has one official formatter and one canonical formatting outcome for any supported source form.

Core rule:

* if two source files have the same syntax tree and both are fully formatable, the official formatter should produce the same formatted result

Initial canonical rules:

* declarations appear in a normalized block-based layout rather than a compact one-line style for multi-part constructs
* opening braces stay on the same line as the declaration or control form they belong to
* closing braces appear on their own line aligned with the start of the construct
* statements end with `;` where required by the language and are emitted one per line
* a single ASCII space is used around infix operators and after commas
* there is no alignment-by-columns formatting
* unnecessary blank lines are removed, while top-level items are separated clearly
* the formatter prefers stable readability over preserving user-written local spacing quirks

Example intent:

```ferox
fn add(a: Int, b: Int) -> Int {
    a + b
}

let result = match value {
    some(inner) => inner,
    none => fallback,
};
```

### Design Intent

Canonical formatting should make Ferox source predictable to read and cheap to rewrite automatically.

Goals:

* produce one obvious formatting result
* keep declarations and control flow visually regular
* avoid style choices that depend on manual alignment
* support formatter-driven refactors and code generation

This is part of the broader Ferox goal of reducing semantic entropy and making automated change safer.

## Import Ordering And Grouping

Imports have one canonical ordering and grouping style.

Initial rules:

* all `use` declarations in a file are kept in one contiguous import block near the top of the file after any file-level doc comments
* import declarations are sorted lexicographically by their full canonical path
* grouped imports are sorted lexicographically inside their braces
* aliased imports are ordered by their underlying import path, not by the alias
* the formatter does not create semantic subgroups separated by blank lines in v1

Examples:

```ferox
use core.hash.HashMap as Map;
use core.io.Writer;
use net.http;
use net.http.{Client, Request, Response};
```

### Canonical Grouping

Rules:

* grouped imports are preferred when multiple items are imported from the same module path
* duplicate imports are collapsed into one canonical form where possible
* wildcard imports remain forbidden and therefore never appear in formatted output
* the formatter may normalize repeated imports from the same module into a grouped import when that does not change meaning

### Design Intent

Import layout should be deterministic without requiring style debates or semantic guesswork.

Goals:

* make import diffs stable
* keep import scanning fast for humans and tools
* avoid subjective grouping conventions in v1
* preserve a single obvious formatter result

## Line Breaking, Indentation, And Trailing Commas

The formatter uses a small set of fixed layout rules for multiline constructs.

### Indentation

Initial rules:

* one indentation level is `4` spaces
* tabs are not used for indentation in canonical output
* nested blocks and multiline constructs increase indentation by exactly one level

### Line Breaking

Initial direction:

* short single-line constructs may remain on one line when they fit the formatter's basic readability rules
* multiline blocks place each contained statement or arm on its own line
* multiline parameter lists, argument lists, tuple literals, array literals, map literals, and grouped imports place one item per line
* closing delimiters align with the start of the construct that opened them
* line breaking is structure-driven rather than dependent on manual alignment choices

Example intent:

```ferox
fn build(
    host: String,
    port: Int,
    secure: Bool,
) -> Config {
    Config {
        host: host,
        port: port,
        secure: secure,
    }
}
```

### Trailing Commas

Rules:

* multiline comma-separated constructs use trailing commas
* single-line comma-separated constructs do not use trailing commas unless required by the syntax of a singleton tuple or similar construct
* trailing commas are part of the canonical formatter output for multiline lists because they stabilize diffs and simplify automated rewriting

### Design Intent

These rules should make multiline Ferox code predictable before any sophisticated width-based formatting policy exists.

Goals:

* produce stable diffs for lists and blocks
* keep indentation mechanical and easy to parse
* favor structure over clever width-sensitive layout
* support code generation and refactoring tools without custom style logic

## Formatter Behavior For Partially Invalid Files

The formatter may attempt limited recovery on partially invalid files, but correctness and trust take priority over aggressive rewriting.

Core rule:

* `ferox fmt` must not silently rewrite a file when the parser cannot establish a sufficiently reliable structural understanding of the source

Initial direction:

* if a file is fully parseable, the formatter produces canonical output
* if a file has minor recoverable syntax damage, the formatter may format only the confidently parsed regions and emit diagnostics for the invalid regions
* if structural recovery is too uncertain, the formatter fails with diagnostics and leaves the file unchanged

### Recovery Expectations

Examples of potentially recoverable situations:

* an incomplete trailing expression at end of file
* a missing closing delimiter in a region where the parser can still identify surrounding top-level structure
* a damaged statement inside an otherwise recognizable block

Rules:

* recovery should be local and conservative
* the formatter must not invent or guess missing code
* any partially formatted result must preserve the original source for regions the formatter could not confidently normalize
* diagnostics should identify why full formatting was not possible

### Design Intent

The formatter should still be useful during editing without becoming untrustworthy.

Goals:

* support editor workflows on in-progress files
* preserve confidence that formatted code still reflects the user's program
* avoid destructive rewrites when syntax is badly damaged
* keep formatting failure modes explicit and diagnosable

## Formatter Stability Guarantees

The formatter is part of the compatibility surface of the official toolchain.

Core rule:

* formatter output should not change arbitrarily between routine toolchain updates

Initial stability policy:

* patch releases must not intentionally change canonical formatting for already supported syntax
* minor releases should avoid formatter churn unless the change fixes a correctness issue, supports newly added syntax, or resolves a clearly justified formatting defect
* large formatting changes should be rare and explicitly called out in release notes

### Compatibility Expectations

Rules:

* formatting changes that would rewrite large volumes of existing code should not appear silently in ordinary maintenance updates
* newly introduced syntax may define new canonical formatting without violating stability for older syntax
* formatter changes tied to language-edition boundaries may be allowed when explicitly documented
* `ferox fmt --check` should remain a reliable CI signal across a pinned toolchain version

### Design Intent

Formatter stability should reduce repository churn and make the official style a dependable automation target.

Goals:

* keep formatting predictable across upgrades
* avoid unnecessary mass rewrites
* preserve trust in formatter-driven workflows and CI
* allow carefully justified evolution without treating formatting as frozen forever

---

## Tooling Philosophy

A single official toolchain exists.

The toolchain should be treated as part of the language contract for safe change, not as a thin convenience wrapper around unrelated tools.

Core direction:

* structured diagnostics and machine-readable outputs are first-class
* incremental `ferox check` should be a product requirement
* formatter and parser architecture should support editor and agent workflows on partially complete code
* tooling interfaces should grow from the same compiler data model rather than from scraped terminal text

The official machine-facing surface should eventually include more than diagnostics alone.

Long-term direction:

* stable symbol identity and symbol graphs
* project and package metadata suitable for tools and agents
* refactor-oriented compiler services
* machine-readable build and analysis events with controlled evolution

CLI:

ferox

Long-term responsibilities:

* build
* run
* test
* format
* lint
* benchmark
* documentation generation
* dependency management
* package publishing

Initial implementation scope should stay narrower.

The first bootstrap wave should prioritize:

* `ferox check`
* `ferox fmt`
* diagnostics
* package loading
* basic build-oriented workflows

Commands such as documentation generation, richer dependency workflows, and package publishing remain part of the official long-term toolchain direction, but they are not required to prove the initial safe-core bootstrap.

Goals:

* Excellent diagnostics
* Machine-readable compiler output
* Deterministic builds
* First-class testing support

## Package Manifest Format

Each package is described by a manifest file named `ferox.toml`.

Ferox v1 uses TOML as the manifest format.

Rationale:

* easy to read and diff
* already familiar to systems-language users
* straightforward for tools to parse and generate

### Required Top-Level Shape

Initial direction:

* a package root contains one `ferox.toml`
* the manifest contains a required `[package]` table
* dependency declarations live under `[dependencies]`
* target declarations use dedicated target tables such as `[lib]` and `[[bin]]`

Example:

```toml
[package]
name = "http_server"
version = "0.1.0"
edition = "1"

[dependencies]
net = "0.3"
json = { version = "1.2" }

[lib]
path = "src/lib.ferox"

[[bin]]
name = "server"
path = "src/main.ferox"
```

### Package Table

The `[package]` table defines package identity and basic compilation context.

Initial required fields:

* `name`
* `version`
* `edition`

Initial optional fields:

* `description`
* `license`
* `repository`

Rules:

* `name` identifies the package for dependency resolution and publishing
* `version` uses semantic-version-style numbering
* `edition` selects language-edition compatibility, not package version

### Target Tables

The manifest may declare library and executable targets explicitly.

Initial direction:

* `[lib]` declares the library target for the package
* `[[bin]]` declares an executable target
* each target table may specify at least a `path`
* binary targets may additionally specify `name`
* target paths are relative to the package root

### Dependencies Table

Dependencies are declared in `[dependencies]`.

Initial direction:

* the simplest form is `name = "version-requirement"`
* inline tables are allowed for richer dependency declarations

Example:

```toml
[dependencies]
json = "1.2"
net = { version = "0.3" }
core = { path = "../core" }
```

Rules:

* `name = "version-requirement"` declares a dependency resolved from the default package source
* `{ version = "..." }` is equivalent but leaves room for additional dependency metadata
* `{ path = "..." }` declares a local path dependency
* dependency names refer to package names, not to module paths

### Dependency Resolution

Ferox uses deterministic dependency resolution in v1.

Core rule:

* a resolved build graph contains at most one version of a given package name

Initial resolution rules:

* version requirements are interpreted using semantic-version-style compatibility
* if multiple requirements for the same package name are mutually compatible, the resolver chooses one deterministic version that satisfies them
* if requirements are incompatible, resolution fails with a diagnostic instead of silently including multiple versions of the same package
* path dependencies resolve to the package located at that path and are not version-selected from the default package source
* workspace-local path dependencies should remain explicit in the manifest rather than being inferred by package name alone

### Locking And Reproducibility

Resolution results should be reproducible.

Initial direction:

* the toolchain may generate a lockfile such as `ferox.lock`
* normal application builds should use the locked resolution when present
* updating dependency selections should be an explicit tool action rather than an accidental side effect of ordinary builds

### Design Intent

The package graph should stay understandable to humans and predictable to tools.

Goals:

* keep dependency graphs small and explicit
* avoid hidden duplicate-version behavior in v1
* make resolution failures visible instead of magical
* preserve deterministic builds

### Initial Non-Goals

The initial manifest format does not yet define:

* registry source overrides
* target-specific dependency sections
* feature flags
* build scripts

These are deferred to later package-system tasks so the base manifest format stays stable and small.

## Workspace Layout

Ferox supports multi-package repositories through an explicit root workspace manifest.

Initial direction:

* a workspace root contains an `ferox.toml` with a `[workspace]` table
* the workspace explicitly lists member package directories
* each member package keeps its own package-local `ferox.toml`
* package paths are relative to the workspace root

Example:

```toml
[workspace]
members = [
    "libs/core",
    "libs/net",
    "apps/server",
]
```

### Repository Structure

The workspace layout is filesystem-driven and explicit.

Initial rules:

* the workspace root is a coordination layer, not itself automatically a package
* each member directory contains one package manifest and its own `src/` tree
* packages may be libraries, binaries, or both according to their local manifest targets
* source-module layout within each package continues to follow the normal file-and-module rules

Example intent:

```text
repo/
  ferox.toml
  libs/
    core/
      ferox.toml
      src/
        lib.ferox
    net/
      ferox.toml
      src/
        lib.ferox
  apps/
    server/
      ferox.toml
      src/
        main.ferox
```

### Initial Non-Goals

The initial workspace model does not include:

* implicit workspace-member discovery
* nested independent workspaces as a primary model
* package-scoped visibility rules across workspace members
* treating one repository as a single giant package by default

This keeps repository structure explicit and easy for both humans and tools to inspect.

## Registry And Publishing Strategy

Ferox v1 uses one default official package registry.

Core rule:

* ordinary versioned dependencies are resolved from the default official registry unless the manifest uses an explicit local path dependency

Initial direction:

* the ecosystem has one canonical public registry for publishable packages
* package names are globally unique within that registry
* published package versions are immutable once accepted
* packages are published explicitly with `ferox publish`

### Published Package Contents

Publishing produces a source package plus registry metadata.

Initial direction:

* a published package includes its `ferox.toml`, source files, and lockfile or dependency metadata as required for reproducible builds
* published content should be deterministic so the same source tree produces the same package artifact
* generated local build artifacts are not part of the published package

### Versioning And Immutability

Registry versions are append-only in normal operation.

Rules:

* a given `(package name, version)` pair may be published at most once
* overwriting an existing published version is not allowed
* if a version must be withdrawn from normal selection, the registry may support yanking without deleting historical artifacts
* publishing requires the package manifest to contain a concrete `name` and `version`

### Default Source Model

The default source model stays intentionally small in v1.

Rules:

* path dependencies are for local development and workspace composition
* published dependencies should resolve through the official registry rather than through ad hoc source locations
* alternate registries or per-dependency source overrides are deferred
* package publishing and dependency fetching are official toolchain responsibilities, not separate third-party workflow assumptions

### Design Intent

The registry model should make the ecosystem easy to reason about before it becomes highly customizable.

Goals:

* keep package identity global and unambiguous
* make published builds reproducible
* avoid hidden source-selection complexity in v1
* preserve room for more advanced registry features later if the ecosystem justifies them

### Initial Non-Goals

The initial registry model does not include:

* multiple first-class public registries in ordinary manifests
* per-dependency registry overrides
* mutable published versions
* mandatory build scripts as part of package publication

## Documentation Generation

Documentation generation is an official toolchain feature.

Primary commands:

* `ferox doc`
* `ferox doc --open` may later open the generated documentation locally, but opening behavior is not required for the initial design

### Documentation Scope

The initial documentation model is API-oriented.

Rules:

* generated documentation is derived from package source and public declarations
* `pub` items form the primary visible surface in generated package docs
* doc comments attached to declarations become the source text for generated API documentation
* documentation generation should understand modules, public functions, public types, and public traits directly from the language structure

### Output Format

Initial direction:

* `ferox doc` generates static HTML documentation by default
* the output is deterministic for a given source tree and toolchain version
* generated docs should preserve package, module, and declaration hierarchy
* cross-links between documented declarations should be generated automatically where the target is known

### Package And Workspace Behavior

Rules:

* running `ferox doc` in a package generates documentation for that package
* running `ferox doc` at a workspace root may generate an aggregated documentation site for workspace members
* workspace documentation should keep package boundaries visible rather than flattening all declarations into one undifferentiated namespace

### Design Intent

Documentation should come from the same explicit source structure the compiler and tooling already understand.

Goals:

* make public APIs easy to browse
* keep documentation generation deterministic and toolchain-native
* avoid hidden documentation configuration as the primary model
* support package and workspace-scale browsing with minimal extra ceremony

### Initial Non-Goals

The initial documentation model does not include:

* multiple official output formats beyond the default static HTML site
* arbitrary user-programmable documentation pipelines
* documentation visibility rules separate from the language's `pub` model

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

Logging expectation:

* when a structured output changes unexpectedly, the toolchain should make it practical to trace which compiler or driver stage produced the changed record
