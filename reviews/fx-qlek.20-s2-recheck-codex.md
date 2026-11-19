Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate b586c19 (stage 2 re-check)

## B1 — Resolved

`crates/tide-std/src/rand.rs:67–77` now rejects `RangeInclusive` with `!self.is_empty()` before computing its cardinality. The pinned nightly's `library/core/src/ops/range.rs:571–575` defines emptiness as `self.exhausted || !(self.start <= self.end)`, exactly covering the missed state while preserving the old predicate for non-exhausted integer ranges. Its endpoint documentation at `:399–407` explicitly requires this method after exhaustion. `Rand::random_range` validates through `range.span(...)` at `rand.rs:155` before backend dispatch at `:157`; rejection therefore precedes both real source access and the simulated scheduling point. The panic retains the method name and range argument.

Regression coverage reaches both backends:

- `crates/tide-std/src/rand/tests.rs:38–43` creates the exhausted-at-MAX witness and asserts `next() == Some(u8::MAX)` and `is_empty()`. The pinned iterator implementation (`library/core/src/iter/range.rs:1530–1538`, integer step at `:348–352`) confirms that its endpoints become `0..=255` with the exhausted flag set.
- `simulated_edge_cases_take_no_word` includes this witness at `rand/tests.rs:106`, requires the method-specific panic prefix, and asserts an empty trace at `:112`. A scheduling point or draw would fail that assertion.
- `real_draws_read_getrandom_through_the_seam` includes it at `:161` inside an empty-reply seam and requires the same panic prefix; `:167` asserts zero requests. The seam records a request before exhausting replies (`rand.rs:261–262`), so an accidental source call cannot pass as the expected argument panic.
- `conformance/cases/authority/rand.rs:27–32` adds the witness to the unchanged shared real/simulated suite. Its five-panic expected line at `:16` matches `crates/tide-admit/tests/build.rs:4182`; the existing build test runs the real case, the simulated case, and manifest replay (`:4173–4215`). Read-only Python assertions confirmed the fifth case and matching expected prefix.

**Falsification claim:** `target/work/report.md`, “Stage 2 round 1 fixes” (`:364–365,378–385`), reports that restoring the old predicate makes two unit tests fail, while the fixed candidate passes all five Rand unit tests, authority/rand conformance, and four Rand build tests. This is consistent with the source: the old predicate accepts `0 <= 255`; the simulated call returns instead of producing the required panic, while the real call reaches the empty seam and produces the wrong panic plus a recorded request. I accept the claim as author-reported execution evidence supported by source inspection. I did not independently rerun either version; no new runtime proof is claimed.

**Same-class sweep:** source search finds only two production `SampleRange` implementations, both in `rand.rs`. Half-open `Range` has no separate exhaustion flag and already rejects `!(start < end)` at `:61`. All ten supported integer types share the fixed inclusive implementation, with signed mapping and full-width cardinality calculated only after validation. The conformance std directly includes this production module (`conformance/rustc/std/src/lib.rs:60–62`), so it has no stale duplicate. No remaining instance of this validation defect was found in this scope, and the fix introduces no observed regression.

## N1 — Resolved

The cited current-status passages now describe the five delivered draws and seeded simulated randomness, leaving `rng() -> Pcg64` planned:

- `README.md:372–375,392,579`;
- `docs/owner-guide/01-capabilities.md:23`;
- `docs/owner-guide/05-seeds-and-simulator.md:145–152`;
- `docs/owner-guide/10-v0-and-planned.md:23,27,44–46`;
- `docs/owner-guide/answers.md:113–118`.

A focused search of README and the owner guide found no remaining current claim that Rand has no methods or that simulated draws are unmodeled. The quoted deferred-bead listing remains historical context. No new availability promise beyond the implemented draws was introduced.

## New blocking findings

None. No defect introduced by these fixes was found.

## Verdict

**approve — high confidence, within this finder re-check's B1/N1 scope.** Both findings are resolved. The code change stays within B1's validation fix and does not require a restarted full Codex review. This clears these findings for the subsequent Opus review; it is not that review or final landing approval.

Reviewed frozen HEAD `b586c19feff4b476eb777f2d9e94d354dadbeb8a` against prior candidate `97006fd`, with a clean tracked tree. Re-check diff SHA-256: `22dda43e90d5853793228b3159315dbebfe7aa0588822d471ed701439ee8ed34`. `git diff --check 97006fd b586c19` and `git diff --check e2f2d26 b586c19` passed. The one-line suite fix matches its prior form and was not reopened.

Only this report was written. No Rust build/run, fresh full review, full-suite rerun, commit, bead write, or contact was performed. Runtime results above remain the author's reported evidence.

DONE
