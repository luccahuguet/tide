Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

This is the re-check round (AP-PROOF-001). It covers only my r2 findings, on the amended candidate `8f2a0cd`. I read `git diff 5d19c7d 8f2a0cd`: 6 files, +15/−5.

- `git diff --stat 306ed1a 8f2a0cd` touches only `.beads/issues.jsonl`, so the rebased `306ed1a` carries the same content.
- I relied on reading alone, because the heavy landing run is due. I ran no builds or probes and made no writes outside this file.
- The author's post-hold checks are in report.md's "Fix round 1". I read them but did not repeat them: unit tests 5 passed, the build tests 2 passed, and the five conformance cases passed.

## Per-finding results

1. **Rule tag: fixed, correct.** `conformance/cases/authority/numeric_conversions.rs:1` now reads `//@ rules: REF-0301, ADMIT-1`. `REF-0301` is the D3/§4.2 lossless-cast row (`conformance/rules.md:167`). The research copy stays byte-identical, as it should.

2. **FX0301 help: fixed, correct.** The text at `crates/tide-admit/src/walk/surface.rs:174` is now "use try_from, T::try_from_float(f.trunc()), truncate (same signedness, fixed widths), cast_signed/cast_unsigned, or round_from".
   - It names every real spelling, and nothing now leads a float-to-integer cast to E0277.
   - **Pointer-width targets:** the help does not suggest `truncate` for them. "fixed widths" rules out a `usize`/`isize` target. Core's only pair into `usize` is `usize → usize`, which is never a lossy cast. It also rules out `usize → u32`, which is E0277.
   - It slightly understates: `usize → u16`/`u8` does work with `truncate`. That errs on the safe side, so no action.
   - "same signedness" next to `cast_signed/cast_unsigned` points to the two-step form for a sign change.
   - `grep` finds no test or case that matches the message text. The code change is a string literal only.

3. **Pointer-width note: moved to fx-djp, not in scope here.** This matches my r2 disposition.

4. **Pin-bump note: fixed, correct.** `crates/tide-admit/src/invocation.rs:310-311` now says: at a pin bump, delete Truncate if core's truncate has stabilized, and rebuild the conformance corpus without `-Aunstable-name-collisions`. It sits beside the allowance it governs. It is comment-only.

5. **`#[must_use]`: fixed, correct.** It is on `Truncate::truncate` (`convert.rs:19`, a provided method) and on `RoundFrom::round_from` (`convert.rs:35`). The attribute on a trait method declaration applies to calls resolved through the trait.
   - The case, the unit tests and the build fixtures all consume the results.
   - The build test's no-`warning:` assertion still passed in the author's post-fix run.
   - It changes no behavior.

6. **Subnormal tests: fixed, correct.** Three inputs now expect Err from `i64::try_from_float`:
   - `f64::from_bits(1)`, at `convert/tests.rs:60`;
   - `-f64::from_bits(1)`, at `convert/tests.rs:61`;
   - `f32::from_bits(0x8000_0001)` (negative minimum f32 subnormal), at `:68-71`.

   All three are finite and nonzero, and their `trunc` is ±0, which differs from the value, so Err is the right expectation. The author reports 5 passed after the fix.

7. **Bound comment: fixed, correct.** `convert.rs:99-100` now says that f32/u128 becomes +inf and that this is safe because non-finite input is rejected first. That matches the code: `is_finite` is the first conjunct.

8. **Text nits: the taken part is fixed; one part was skipped as agreed.**
   - `conformance/rustc/std/src/lib.rs:8-9` now reads "… and the codec engine / and `std::json` (`codec.rs` and `json.rs`)), …". The orphan line is gone and the parentheses balance: the inner pair closes the file list and the outer pair closes `(`root.rs`: …`.
   - The reference's double colon was skipped, as r2 advised. `design/reference.md` is unchanged, so the count stays 24,995.

## New issues introduced

None. The delta is one string literal, comments, two `#[must_use]` attributes, three test inputs and a rule tag. None of them changes runtime behavior, admission policy or design text.

DONE
