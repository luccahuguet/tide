# Tide review, fx-kw8.8 round 2 (Opus): FX0512 for unmarked main

## Verdict

**Changes made** (converge round, fixing). One correctness fix, one
regression case, two comment re-wraps.

## What was reviewed

The working tree against HEAD `1e0cbac` after round 1's fixes,
`git diff -- README.md crates conformance`, SHA-256
`a61a4ffa7b92a9e884512a4893d0bcd5497b005b37ccdc38f72fd1d6971c7c41`, with
round 1's reports and resolutions.

## Changes

1. `crates/tide-admit/src/invocation.rs` (`check_expanded`): round 1's
   `#![no_main]` check ran in `Expanded::visit_attribute`, so it fired on a
   `no_main` attribute anywhere in the crate, while rustc honours it only at
   the crate root (`rustc_passes/src/entry.rs`, `find_attr!(tcx, crate,
   NoMain)`) and elsewhere only warns. `#[no_main] fn helper() {}` or
   `mod m { #![no_main] }` beside a marked `#[tide::main] fn main` was
   rejected with FX0512. The check now reads only the expanded crate root's
   attributes (`krate.attrs`); `#![cfg_attr(all(), no_main)]` is still
   caught.
2. `conformance/cases/authority/root_no_main.rs`: adds an inert
   `mod inert { #![no_main] }` with no mark, which the old code would have
   flagged.
3. Comment re-wraps in `walk.rs` (`entry_point`) and `invocation.rs`
   (`check_expanded`).

## Round 1's fixes

B1 (`--tide-std` as the switch), B2 (imported `main`, reported before the
locality return at the import's span), and B3 (`#![no_main]`, with the fix
above) are right and complete. The decoy, help text, and README read
correctly; rules.md, conformance/README.md, and the design need no edits.

## Checks after the changes

`cargo test --workspace`: exit 0 (2 + 9 + 13 passed). `conformance/run.sh`,
stdin closed: exit 0, 188 PASS, no FAIL.

## Out of scope

- `#[tide::main] fn start() {}` is accepted: the macro always emits
  `fn main` (existing macro behavior).
- Under the `unsafe` grant, a binary with a marked `main` could export a
  second C `main` symbol through `#[unsafe(no_mangle)]` or `export_name`:
  a link-level matter for UNSAFE-1, not ROOT-1.
- Under `#![no_std]` with `extern crate std`, a genuinely marked
  `#[std::tide::main]` is flagged FX0512, since `Crates::tide_std` is
  `None`; safe direction, part of fx-kw8.13.
