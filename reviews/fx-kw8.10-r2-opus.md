# Tide review, fx-kw8.10 round 2 (Opus): converge confirmation round

Candidate: the staged diff on base 0141db4, SHA-256
`b0ba34a096f238e1355564149ae7b413cc0db04be97526e9597bf1e4d94dde09`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 3 blocking, 2 non-blocking, 1 nit. Every round-1 finding is fixed
(reproduced: `Arc<Net>`, granted `fn()` impls, per-item `asm!` holders, the
FX0915 stale artifact, `Poly`, 63/65-deep wrappers, nested holders,
`#![no_std]`, aliases and re-exports). `cargo test --workspace` and
`conformance/run.sh` exit 0.

## New findings

- **B1, blocking** (`invocation.rs:608-615`): the artifact is removed before
  `destination()` and the inside-package check, so with the default
  `--out-dir .` inside a package, a user file `lib.interface.json` is deleted
  and then the run is rejected. Fix: remove it after the output checks.
- **B2, blocking** (`interface.rs:81`, `:103`): items are keyed by path
  alone, so a type and a value of the same path (`pub use std::fs::Fs as
  Handle; pub fn Handle(n: Net)`, or `struct Config` with `fn Config`)
  overwrite each other, and `take_holes` sums their holes. Fix: key by path
  and kind.
- **B3, blocking** (`interface.rs:300-302`): `holding` keyed by definition
  makes nested instantiations opaque: `W<W<Net>>` gives
  `x.0: opaque(W<Net>)`, not `x.0.0: net`. Fix: call it recursion only when
  the new instance is not a part of the held one.
- **N1, non-blocking** (`walk/mono.rs:58-70`): the backstop holds a granted
  impl by generic local functions that select nothing themselves
  (`g<T: Debug>` called with a `fn()`), unlike FX0520. Fix: skip generic
  local users.
- **N2, non-blocking** (`walk.rs:414`, `:1327`): `holder_at` runs for every
  expansion and is quadratic: 6000 functions using `vec!` and `format!`
  take 18 s. Fix: compute the holder only for a granted use.
- **Nit**: the `rust_std_injected` branch is reached only when an operator
  passes Rust's std as `--tide-std`; its doc names the driver's own tests,
  and the README says every crate whose `std` is not Tide's fails.

## Checked

FX0901 and FX0520 against the base walk (no verdict changes); holders for
closures, nested fns, impls, attributes, the backstop, and
`#[trusted_pure]`; authority for aliases, consts, trait and inherent
methods, globs, depth boundaries, and the per-item budget; README and module
docs against the code.
