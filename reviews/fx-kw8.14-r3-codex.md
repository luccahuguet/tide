Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

## Verdict

**approve — high confidence.** The fix resolves round-1 blocking finding 1. No additional defect was found in the delta.

## Scope

Read-only re-check of `2dd08da..7a904a7`, the review records, affected callers, regression cases, and pinned compiler wiring.

The scoped working-tree contents match `7a904a7d72bec537ca4ffdf105eb1d5d742699c0`. The requested raw diff shows two apparent deletions because the new case files are untracked; their blob hashes match the candidate exactly. The remaining scoped diff is empty.

## Blocking

None.

## Non-blocking

**C1/C2 accepted without change.** Under `--test`, references to a library’s plain crate-root `main` remain rejected (C1), and a crate-root `#[test] fn main()` is newly rejected (C2), although neither necessarily mints authority. These are conservative rejections of safe code under the accepted ROOT-5 policy. Renaming the helper or test avoids them; narrowing detection would add mode-dependent handling.

The diagnostic’s runtime explanation is inaccurate for those harmless functions, but does not block approval.

## Reasoning

- [walk.rs:1325](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1325) now checks the resolved `main` regardless of span origin. Core’s generated test descriptor therefore cannot hide its call behind `generated_by_toolchain`. The exemption remains intact for the other admission checks.
- [root_main_test.rs:4](/home/lucca/pjs/lab/tide/conformance/cases/authority/root_main_test.rs:4) captures the problematic attribute ordering and requires FX0506 on the attribute line. The runner checks the diagnostic marks exactly across binary and test modes.
- No new false positive was found in ordinary entry wiring, harness generation, or ordinary `#[test]` functions beyond C2. Tide’s wrapper references `__tide_main`; rustc’s entry wrapper is generated below HIR; the harness references descriptor constants. Ordinary test descriptors reference their own function identities, including module-local functions named `main`.
- [crates.rs:130](/home/lucca/pjs/lab/tide/crates/tide-admit/src/crates.rs:130)’s `Option<DefId>` comparison preserves identity semantics: an absent entry or foreign definition cannot accidentally match. The documentation changes accurately describe the fix.

Not checked: builds or execution, as requested; relied on recorded workspace success and 193 conformance PASS results for `7a904a7`.  
Remaining limitation: the accepted C1/C2 restrictions on harmless crate-root `main` functions.