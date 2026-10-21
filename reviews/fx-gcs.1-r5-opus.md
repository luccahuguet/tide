# Tide review, fx-gcs.1 round 5 (Opus): converge round

Fresh pass after round 4. One fix, staged, not committed. Diff hash (`git diff HEAD -- . ':!.beads' ':!reviews' | sha256sum`): start `77b8084a9523b67c5b79d927140756e531f868334c61042d2947979309c58d3b`, end `0f4488c00183ff06ab28ebde6ead718118bfdf4b4d34b5c03d83ec37af493ef6`.

## Fixed

- Deleted the copied macros crate (`conformance/rustc/macros/`, about 176 lines). It matched the frozen `trials/e1-dialect/rustc/macros` except for doc comments, and nothing in conformance changes it; only the std copy must be separate, because cases extend it. `conformance/rustc/std/Cargo.toml` now takes `oracle_macros` from `../../../trials/e1-dialect/rustc/macros`, as it already takes `tide_macros` from `spikes/d18/macros`, and `conformance/README.md` says the std is copied and the trial's macros crate is used as is. A from-scratch offline build into a scratch target works, `Cargo.lock` is unchanged, and nothing is written under `trials/`. A later case that needs a macros change copies the crate then.

## Left alone

- `oracle_macros` and `tide_macros` stay: the stub's prelude re-exports `test`, `sim_test`, `Error`, the contract attributes, and `tide::main` from them, and the `--test` build needs the `#[test]` shim.
- The stub's `pub` items, `pub fn` in the cases, and the README example: as earlier rounds found.
- `conformance/rules.md` cites "01:956", which this diff's one-line D0 addition moves to 01:957; left because `rules.md` dates every line reference to `2d82c40`, and 02, 03, and 04 are unchanged since.
- No bugs found in `run.sh`'s parsing of marks, IDs, or rustc errors, its binary detection, its exit codes, or `--trace`.

## Verification (stable rustc/cargo, offline)

- `conformance/run.sh` exits 0 (3 PASS, 2 AWAITS CHECKER: FX0520, FX0606); `--trace` exits 0.
- Mutations: a flipped code, a removed mark, a malformed `E27`, an unknown `PAR-99`, no rules line, a wrong code on an uncoded lifetime error, and a nonexistent case each FAIL with exit 1; an uncoded error marked `//~ error`, a binary `fn main(root: Root)` marked E0580, and an error inside `#[cfg(test)] #[test]` code caught by the `--test` build each PASS.
