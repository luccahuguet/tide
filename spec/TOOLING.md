# Tark Tooling And Packages

## Formatting Policy

Tark defines a **single canonical source format**.

Canonical formatting is part of the standard workflow, but formatting is not a prerequisite for executing otherwise valid code.

If a file is syntactically valid but **not canonically formatted**:

* `tark build`
* `tark run`
* `tark test`

must still operate on the program normally.

Formatting compliance is enforced by:

* `tark fmt`
* `tark fmt --check`
* lint or CI policy

Formatting is enforced by the official formatter, not by making ordinary execution commands reject valid code.

Example command:

tark fmt

Goals:

* eliminate style ambiguity
* improve agentic reliability
* reduce review noise
* stabilize refactoring and code generation
* ensure uniform codebases

---

## Tooling Philosophy

A single official toolchain exists.

CLI:

tark

Responsibilities:

* build
* run
* test
* format
* lint
* benchmark
* documentation generation
* dependency management
* package publishing

Goals:

* Excellent diagnostics
* Machine-readable compiler output
* Deterministic builds
* First-class testing support

## Package Manifest Format

Each package is described by a manifest file named `tark.toml`.

Tark v1 uses TOML as the manifest format.

Rationale:

* easy to read and diff
* already familiar to systems-language users
* straightforward for tools to parse and generate

### Required Top-Level Shape

Initial direction:

* a package root contains one `tark.toml`
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
path = "src/lib.tark"

[[bin]]
name = "server"
path = "src/main.tark"
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

Tark uses deterministic dependency resolution in v1.

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

* the toolchain may generate a lockfile such as `tark.lock`
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

Tark supports multi-package repositories through an explicit root workspace manifest.

Initial direction:

* a workspace root contains a `tark.toml` with a `[workspace]` table
* the workspace explicitly lists member package directories
* each member package keeps its own package-local `tark.toml`
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
  tark.toml
  libs/
    core/
      tark.toml
      src/
        lib.tark
    net/
      tark.toml
      src/
        lib.tark
  apps/
    server/
      tark.toml
      src/
        main.tark
```

### Initial Non-Goals

The initial workspace model does not include:

* implicit workspace-member discovery
* nested independent workspaces as a primary model
* package-scoped visibility rules across workspace members
* treating one repository as a single giant package by default

This keeps repository structure explicit and easy for both humans and tools to inspect.
