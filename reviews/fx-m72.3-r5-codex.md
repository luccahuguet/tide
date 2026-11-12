Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12
Verdict: approve

Scoped re-check of r4 finding 1 on frozen HEAD `2a7ed4aff9c72f221e318e5308ddb29d37063d9b`, parent `d02ccf539f699785b4f5beb2592f2f706e74d8e6`. Interdiff `b45ac30..2a7ed4a` SHA-256: `85500d2f71f32855c7d058b99dd3ce7205f471d6291d7fd5eb214d007d43e13c`. Full diff against main SHA-256: `782fdd0243bbf52a525a97992741e86dcdca524f4051a66682e71a0c53350dc1`.

1. **Fixed — const-argument printing no longer introduces unnecessary braces.** Evidence: `crates/tide-admit/src/synthesis/contracts.rs:266-279`. No blocking or non-blocking finding in this fix.

   - Integral Value constants are decoded from their scalar representation with the type's signedness. Signed values use `to_int` at the scalar's actual width, preserving negative numbers and signed minima; unsigned values use `to_bits`, preserving the full u128 range. Decimal output avoids rustc's diagnostic spelling of extrema as associated paths such as `i16::MIN`. Unsuffixed literals receive their type from the ADT's declared const parameter. Negative literal const arguments are legal without a block.
   - Bool and char Value constants retain rustc's literal printer, including escaping for characters, and no longer receive unnecessary braces.
   - Const parameters remain single bare parameter names. This does not expand the separate policy that skips functions with const generic parameters.
   - Unevaluated expressions and other const forms retain the previous braced fallback. Associated paths and compound expressions therefore keep the braces Rust's const-argument grammar requires. The change does not broadly strip braces from arbitrary printed text. Array-length printing is unchanged.

   **Regression strength:** `conformance/cases/contracts/generated_tests.rs:3` now denies unused_braces. `:278-282` supplies a concrete `Packet<7, { i16::MIN }, true, '>', { u128::MAX }>` input, covering a normal integer, a negative signed minimum, bool, char and the largest unsigned value. Source braces around associated paths are required, while the generated evaluated scalar values need none. `crates/tide-admit/tests/build.rs:1296-1301` requires `concrete_consts` to run and report 100/100 accepted cases; an ignored entry cannot satisfy it.

   Without the printer fix, the same regression fails during the checked driver build: the old unconditional braced rendering emits arguments such as `{ 7 }`, `{ true }` and `{ '>' }`, which the fixture's deny(unused_braces) rejects. This counterfactual follows directly from the old branch and the warning independently observed in r4; no candidate mutation was performed to demonstrate it again. The author's final `fix-r4-generated-final-2.log` reports the focused integration test passing (1 passed, 0 failed, 2.00 seconds).

Confidence: high. Source inspection establishes the parameter and fallback cases; the fixture and author's focused run establish the concrete scalar cases above. It is not an exhaustive dynamic matrix of every integer width, character escape or unevaluated expression.

Verification limits: no independent compiling probe ran. At entry only seconds remained before the 07:05 UTC booked window, and the window was active by the end of inspection. Read the scoped interdiff, relevant printer context, regression assertions and the author's Fix round 4 notes/final generated-test log. `git diff --check b45ac30 2a7ed4a` passed. The interdiff's separate provenance-comment and vacuity-error-location changes were not re-reviewed, as requested. No restricted E1 reads, candidate edits, Beads/history writes, full suites or heavy.sh invocation occurred; the worktree remains clean at the recorded HEAD. Full-suite proof remains with the coordinator.
DONE
