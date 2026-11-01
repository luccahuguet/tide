# Tide review, fx-kw8.8 round 3 (Opus): FX0512 for unmarked main

## Verdict

**Clean**: no changes. Converge confirmation round on the tree after
round 2's fixes (`git diff -- README.md crates conformance`, SHA-256
`dba4333feeac2dc8e5025f07491608c2403746cb6c2c73e8dad6e0e6729f6ae9`).

## What was checked

Traced the code and probed the built driver with the conformance stub std.

- Accepted, correctly: `#[tide::main] fn main`; the renamed
  `use std::tide::main as entry; #[entry] fn main`;
  `#[cfg_attr(all(), tide::main)] fn main`; an unmarked `use m::main` under
  `--test`, where the harness's `main` is the entry.
- FX0512, correctly: `#![cfg_attr(all(), no_main)]` beside a marked `main`;
  `mod m { pub fn main() {} } use m::main;` (at the import);
  `#[requires(true)] fn main`; a crate-root `#![no_main]`. A nested
  `#![no_main]` is left to rustc's warning.
- rustc itself rejects a `main` declared in an extern block.
- Decoys: aliasing core's `test` as `main` cannot escape (rustc drops the
  item in a binary build; under `--test` the harness supplies `main`). User
  code cannot produce an `AstPass` expansion, and only Tide's std exports an
  attribute macro named `main`.
- An imported `main` is never the generated one: the macro drops the
  function's visibility, so the generated `main` cannot be re-exported to the
  crate root. The `Termination` follow still runs for a local `main` only, so
  moving it after the FX0512 report loses nothing.

Docs and tests: the README states FX0512 is checked only with `--tide-std`;
reference.md §6.2, 03 §2.1, and rules.md's ROOT-1 row already name FX0512.
The cases (plain, unmarked, decoy, no_std, no_main, admit_main_return,
accept) and the imported-`main` driver test cover each round-1 bypass.

Checks were not rerun, since nothing changed; round 2 recorded both exiting
0 (188 PASS).

## Nits (not changed)

- `check_expanded`'s doc (`invocation.rs:491`) does not say the
  `#![no_main]` rejection needs Tide's std; `Admission::tide_std` and the
  README do.
- The `Admission` initializer lists fields in a different order from the
  struct.

## Out of scope

`conformance/cases/authority/out-admit_backstop_accept/` is an empty,
untracked directory from an earlier run (Oct 31 19:31). Round 2's
out-of-scope notes still apply.
