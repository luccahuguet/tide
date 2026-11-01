# Tide review, fx-kw8.8 round 1 (Opus): FX0512 for unmarked main

## Verdict

**No blocking findings.** One non-blocking hardening gap, one out-of-scope
note, and nits. A converge round that reported instead of fixing; it ran
beside Codex round 1 on the same frozen candidate.

## What was reviewed

The working tree against HEAD `1e0cbac`, `git diff -- README.md crates
conformance`, SHA-256
`b53d820afbf8052fb2a5965e66d1d216ba81cdcc1593cfc33610e770cbb5a3ec`.

Ran `conformance/run.sh` on the 9 cases the runner builds as a binary:
root_main_decoy, root_main_plain, root_main_unmarked, admit_main_return,
root_main_accept, and pure_routes PASS; root_main_called (FX0506),
root_captured and root_borrowed (FX0503) AWAIT CHECKER only for codes this
change does not decide. Ran the driver by hand on scratch cases.

Correct entries not flagged: `#[requires(true)] #[tide::main] fn main`,
`#[tide::main] #[ensures(true)] fn main() -> ExitCode`,
`use std::tide::main as entry; #[entry] fn main`, and a local
`mod tide { pub use std::tide::main as main; }`. Unmarked entries flagged:
`mod m { pub fn main() {} } use m::main;` (not under `--test`, correctly)
and the decoy.

## Findings

**F1 (non-blocking): an imported main from another crate skips FX0512.**
`walk.rs:1190` returns on `main.as_local()` before the report. rustc 1.96
accepts an imported `main` (`use std::process::abort as main;` compiles), so
a non-local entry never reaches the check. Not reachable today: the driver
takes no `--extern`, and no admitted std fn with no parameters returns a
`Termination` type. Fix: report before the `as_local` return, at the
import's span from `tcx.resolutions(()).main_def`.

**Out of scope: `#![no_main]`** is admitted, so a binary with no Rust
`main` passes; it can run code only through `#[unsafe(no_mangle)]` or a
link section, which needs the `unsafe` grant.

## Nits

1. FX0512 is reported at `tcx.def_span(main)` without `source_callsite()`,
   unlike every other walk diagnostic; a `main` from a user macro would be
   reported at the macro definition (user macros are already FX0101).
2. The decoy does not exercise the macro-identity branch, since `#[pure]`
   leaves `main`'s span outside an expansion; it does catch a check by
   spelling, the realistic risk.
3. The help "add `#[tide::main]` above `fn main`" misleads for
   `use m::main;`.
4. `#[tide::main] fn start() {}` is accepted: the macro always emits
   `fn main` (existing behavior).

README, rules.md's ROOT-1 row, conformance/README.md, and the design need
no edits; `driver.rs:366` runs without `--tide-std` and still expects only
FX0520.

## Resolution (author)

Fixed with Codex round 1's findings: FX0512 is reported before the locality
return, at the import's span for an imported `main`, through
`source_callsite()`; `#![no_main]` on a binary is FX0512 (Codex B3); the
help now reads "write `#[tide::main] fn main` in the crate root".
