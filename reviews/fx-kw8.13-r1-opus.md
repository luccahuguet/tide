# Tide review, fx-kw8.13 round 1 (Opus): converge review round

Candidate: working tree on base ceb39de (`git diff HEAD -- . ':!.beads'` plus the untracked `conformance/cases/authority/admit_no_std.rs`), diff SHA-256
`f5498c9ee87603d60955b43554669fb9c47cea9f72c3b2b2690d78adc6a6648b`. I checked the hash at the start and at the end, and it did not change.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Not clean: two findings block. The mechanism itself is correct and meets the bead's Acceptance through its "or the attribute itself, if rejected" branch:
- `check_expanded` reports each crate-root `no_std` under `--tide-std` as FX0520.
- Because `after_analysis` writes the trust artifact only when `has_errors()` is none (`invocation.rs:63-65`), the artifact writer never sees a `#![no_std]` crate.

The two blockers:
- **B1 (text-only):** the docs still name `#![no_std]` as the route to the writer's "cannot write" failure.
- **B2 (not text-only):** this is the new-surface item. `pub static CELL` should **go**.

There are also four nits, all non-blocking.

## Findings

B1 (bug in docs, text-only, blocks). Three texts still describe `#![no_std]` as the case where the trust-artifact writer fails. The writer is no longer reached for such a crate.
- `crates/tide-admit/src/interface.rs:65-66` says the writer "fails when Tide's std ... is not the crate's `std` otherwise, as under `#![no_std]`". That is now false: FX0520 is an error, so `write` is not called.
- `crates/tide-admit/src/interface.rs:4-5` (module doc) says "a crate whose `std` is neither Tide's nor Rust's, as under `#![no_std]`, fails the check instead".
- `README.md:63-64` says "A crate whose `std` is neither Tide's nor Rust's, as under `#![no_std]`, fails the check". This is still literally true, but it sits in the trust-artifact paragraph and repeats the FX0520 sentence at line 57 with the wrong mechanism.

After this change, `crate_pure` is `None` only when the operator's `--tide-std` names `core` or `alloc` (`crates.rs:66-70`).

Suggested fix:
- In interface.rs, replace "as under `#![no_std]`" with "as when the operator's `--tide-std` is `core` or `alloc`", or drop the example.
- In README.md:63-64, drop the clause "as under `#![no_std]`". Line 57 already covers that case.

Why it blocks: README.md is edited by this change, and interface.rs is in the same v0 tier.

B2 (new-surface report, `conformance/cases/authority/admit_no_std.rs:8` `pub static CELL`): **go**. The fix is not text-only, and it blocks.
- **It pins a known hole as expected output.** `run.sh` requires the driver to report exactly the marked FX codes (`run.sh:126-129`). So line 8's `//~ FX0520` asserts two things about a crate that line 5 already rejects:
  - Tide's std's `sync::Shared` is treated as a non-admitted foreign path.
  - There is no FX0502.
  Both are symptoms of the bypass that the bead closes, not contracts. If the rejected alternative (identifying Tide's std from `--tide-std`) is ever adopted, this case breaks for reasons unrelated to ADMIT-1.
- **It misattributes coverage.** The header `//@ rules: ADMIT-1, GLOBAL-1` makes `run.sh --trace` list this case as GLOBAL-1 evidence, yet the case shows GLOBAL-1 *not* applying. rules.md maps GLOBAL-1 to FX0502, and this case has no FX0502 mark.
- **The Acceptance does not need it.** "or the attribute itself, if rejected" is met by line 5 alone, with `extern crate std;` kept as the Acceptance asks.
- **The motivation already has a home.** The reason it illustrates belongs in prose, where 03 ADMIT-1's new bullet and the comment already put it.

Fix:
- Delete line 8 and drop `GLOBAL-1` from line 1.
- Trim lines 2-4 to the reason, for example: "Every crate's std is Tide's, which rustc injects and the rules key on; under `#![no_std]` an `extern crate std` would load it as an ordinary crate, outside the `Pure` rules."

N1 (nit, non-blocking; text-only if reworded). The driver test at `crates/tide-admit/tests/driver.rs:644-648` lost its purpose.
- It used to cover the writer's "cannot write" path. It now proves only FX0520, which `admit_no_std.rs` owns, and "a failed check leaves no artifact", which the `build.rs` step at lines 636-642 already proves.
- Its comment, "Without Tide's std as its `std`, no authority can be decided", gives the old reason, not the rule that now fails the check.

Preferred fix: delete the step, since it is duplicate proof (AP-TEST-001). The minimum fix is to reword the comment to "`#![no_std]` is not admitted (FX0520), and the failed check leaves no artifact."

The writer's `Err` branch then has no test. It is now reachable only through operator misconfiguration and is a two-line guard, so I would not add one. If B1 is fixed this round, fold this in.

N2 (nit, text-only, non-blocking). The rewrapped doc comment at `crates/tide-admit/src/invocation.rs:514` is 119 columns; its neighbours wrap at 100. Reflow lines 512-515.

N3 (nit, text-only, non-blocking). The rewrap left `README.md:58` at 103 columns ("caches successful metadata checks ... against Tide's"); the paragraph wraps near 78. Reflow lines 56-59.

N4 (nit, text-only, non-blocking). The stated reason covers only one route:
- the comment at `invocation.rs:557-559`;
- 03's bullet (`design/03-authority-concurrency.md:561-563`).

Both say the problem is that `extern crate std` loads Tide's std as an ordinary crate. The bead's recorded decision gives the fuller reason: under `#![no_std]` the injected std is `core`, so `Crates::tide_std` is `None` and the `Pure` rules are off even with no `extern crate std`.

Suggested wording: "under `#![no_std]` the injected std is `core`, so no crate is Tide's std and the `Pure` rules are off, and an `extern crate std` loads it only as an ordinary crate."

## Checked and acceptable

- **`#![cfg_attr(all(), no_std)]` is caught.** The check reads `resolver_for_lowering`'s expanded crate. rustc's `InvocationCollector` expands `cfg_attr` on the crate root in place (`take_first_attr`/`expand_cfg_attr` in `rustc_expand/src/expand.rs`, pinned nightly), so the expanded `krate.attrs` holds the plain `no_std`. This is the same mechanism the existing `no_main` check relies on. A crate-root inner attribute cannot come from a macro. A `#[no_std]` that is not at the crate root is inert in rustc, so checking only the root is right.
- **The `tide_std` gate.** It mirrors `no_main`'s gate: the driver's own tests on Rust's std pass no `--tide-std` and are unaffected, and conformance and production always pass it.
- **Two `if tide_std` blocks.** Merging them into one loop with an `Executable` branch is not clearer, so keep them as they are.
- **`#![no_core]`.** It is feature-gated, so `-Zallow-features=` closes it (ADMIT-2), as the comment says.
- **The message and the test string agree.** The message is "`#![no_std]` is not admitted: …"; driver.rs:647 matches "FX0520: `#![no_std]`".
- **`root_main_no_std.rs` and `admit_crate_identity.rs` gaining `//~ FX0520` on `#![no_std]`.** This is required by `run.sh`'s exact matching.
  - Both now test behaviour inside a crate already rejected at the attribute. `root_main_no_std.rs:8`'s FX0520 on `fn main` is the same kind of hole symptom as `CELL`.
  - They predate this change, and their main assertions (FX0512 independent of std injection; the allowlist keyed on crate identity) are properties worth keeping if the rejection is ever relaxed. So I leave them.
  - Their updated comments are accurate.
- **03 ADMIT-1 bullet, rules.md ADMIT-1 row, README line 57.** All three are consistent with the code and the bead's recorded decision.

Checks: read-only inspection only (`git diff`, `sha256sum`, `br show --no-db fx-kw8.13`, `grep`, `sed`, and reading rustc's pinned sources). Nothing was rebuilt or rerun; I relied on the recorded `cargo test --workspace` exit 0 and `conformance/run.sh` exit 0 (194 PASS).
