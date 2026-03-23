# Ferox Tooling And Packages

This document is the canonical entrypoint for the Ferox tooling, package-system, and machine-contract reference.

Ferox tooling design currently prioritizes:

* safe change as an official toolchain concern, not just a compiler concern
* deterministic formatting, package resolution, and machine-readable outputs
* fast feedback through `ferox check`, `ferox fmt`, diagnostics, and package loading in the bootstrap wave
* stable machine-facing contracts for diagnostics, metadata, and later refactor-safe tooling

The detailed tooling rules now live in topic files under `spec/tooling/`.

## Reference Map

* [tooling/toolchain.md](./tooling/toolchain.md): official toolchain scope, bootstrap priorities, and long-term CLI responsibilities
* [tooling/formatter.md](./tooling/formatter.md): canonical formatting, import ordering, partial-file behavior, and formatter stability
* [tooling/packages.md](./tooling/packages.md): `ferox.toml`, workspaces, dependency resolution, lockfiles, and registry/publishing rules
* [tooling/documentation.md](./tooling/documentation.md): `ferox doc` scope, output model, and workspace documentation behavior
* [tooling/machine_contract.md](./tooling/machine_contract.md): machine-readable outputs, read-side contract, mutation contract, and validation expectations

## Reading Order

Suggested order for most readers:

1. [tooling/toolchain.md](./tooling/toolchain.md)
2. [tooling/formatter.md](./tooling/formatter.md)
3. [tooling/packages.md](./tooling/packages.md)
4. [tooling/machine_contract.md](./tooling/machine_contract.md)
5. [tooling/documentation.md](./tooling/documentation.md)

## Related Documents

* [VISION.md](./VISION.md): project goals and positioning
* [LANGUAGE.md](./LANGUAGE.md): language semantics and surface syntax
* [IMPLEMENTATION.md](./IMPLEMENTATION.md): bootstrap strategy, compiler architecture, validation harness, and self-hosting plan
