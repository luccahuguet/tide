# Tide review, fx-gcs.1 round 2 (Opus): converge round

Fresh pass over the fixed candidate (round 1's fixes applied). Three fixes, staged, not committed.

## Fixed

1. `conformance/README.md:91`: "Each prefix belongs to one chapter section" contradicted the prefix table (`ORIGIN`, `LIFE`, `CLOSURE`, `AFREE`, and `CAP` span several sections, and `PAR` already spans 03 §8.1 and §8.2). Now "one chapter", which matches the next sentence about restatements.
2. `conformance/run.sh:65`: `grep -q '^fn main'` became `grep -qw '^fn main'`. A library case with a top-level `fn main_loop` was built as a binary and failed with an unlocated E0601; it now passes as a library. `fn main()` and `fn main<T>()` still count as binaries.
3. `conformance/run.sh:80`: deleted `|| ( -n "$want" && $code == 0 )`, which decides nothing: when `want` is non-empty and rustc exits 0, `got` is empty and `want != got` already fails the case. `( -z "$want" && $code != 0 )` stays; it catches rustc failing without an error line.

## Left alone

- New dependencies `proc-macro2`, `quote`, `syn`, `oracle_macros`, `tide_macros`: the trial's pinned versions with a byte-identical `Cargo.lock`; the stub's prelude re-exports `test`, `sim_test`, `Error`, the contract attributes, and `main` from them.
- The stub's public items: a near-verbatim copy of `trials/e1-dialect/rustc/` (only descriptions, doc comments, and one relative path differ); trimming it would make the sibling beads re-copy it.
- `pub fn` in the cases: removing `pub` changes no verdict but puts dead-code warnings into FAIL logs.
- The README example duplicates `cases/authority/lock_payload_shared.rs`: an inline example belongs in a format spec.
- `--trace` runs a small pipeline per ID × case pair: fast at 26 IDs × 5 cases.
- `rules.md` citations: no design file changed between 2d82c40 and HEAD; LOCK-1/4/5, PAR-4, ADMIT-1/2, ROOT-1, GLOBAL-1, the 02 restatement rows, CLAUSE-1..6, and SEAL-2/3 spot-checked.

## Verification (rustc 1.96.0, offline)

- `conformance/run.sh`: exit 0; AWAITS CHECKER for admit_atomic (FX0520) and lock_payload_shared (FX0606), PASS for the other three.
- `conformance/run.sh --trace`: exit 0.
- Mutations on scratch copies: a flipped verdict, a removed rustc mark, an FX mark changed to a rustc mark, an unknown ID, a missing rules line, a malformed mark, an empty mark, an unmarked uncoded error, and an unmarked test-only error each FAIL. A binary case, a binary with E0580, an `error` mark, a test-only E0308 mark, a case with two IDs, and the `fn main_loop` library case each PASS. A dropped FX mark also passes, the gap the README documents.
- Final `git diff HEAD -- . ':!.beads' | sha256sum`: `d30f39e9d34ccd4c3078559f43bd986fd756110e01daad73679330b4709fb9cb`.
