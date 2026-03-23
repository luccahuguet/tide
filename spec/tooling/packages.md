# Ferox Tooling Reference: Packages, Workspaces, and Registry

This document defines the Ferox package manifest, workspace layout, and registry model.

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
