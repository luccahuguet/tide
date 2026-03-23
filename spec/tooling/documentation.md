# Ferox Tooling Reference: Documentation Generation

This document defines the official documentation-generation surface.

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
