Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Reviewed the frozen candidate `5d19c7d` (sole parent `b7a994a`), `git diff b7a994a..5d19c7d`, read-only. This is a converge round for the code and the design review round for the `design/reference.md` §4.2 and `design/01-design.md` D3 edits. Nothing blocks. Findings 1 and 2 are cheap leftovers. I recommend folding them into this candidate before landing; otherwise they go to `fx-djp` with the rest.

## Findings

1. **Non-blocking. The conformance case is tagged with the pointer rule instead of the lossless-cast rule.**
   - Evidence: `conformance/cases/authority/numeric_conversions.rs:1` reads `//@ rules: GLOBAL-4, ADMIT-1`. GLOBAL-4 (`conformance/rules.md:100`) covers raw pointers, address observers and address casts under the `unsafe` grant. The case has none of those.
   - The registry has the right row: `REF-0301` at `conformance/rules.md:167` ("D3; ref §4.2 | Only lossless numeric as casts"). fx-u65.2 added it, and `casts_accept.rs:1` and `casts_reject.rs` use it.
   - The research (`research/25-casts-arena-allowlist.md` §1.6) said "no row … except GLOBAL-4", but it predates that row. Decision 7's "fx-u65.2's row" now exists. The implementer's report repeats the stale claim.
   - Fix: `//@ rules: REF-0301, ADMIT-1`. It is a one-line, text-only change and needs no re-check beyond the conformance run that the full suite already does.

2. **Non-blocking. FX0301's help text still points float-to-integer casts at `try_from`, a stale leftover of this change's spelling change.**
   - Evidence: `crates/tide-admit/src/walk/surface.rs:174` says "use try_from, truncate, cast_signed/cast_unsigned or explicit rounding".
   - Before this change, `try_from` matched the taught float spelling `i64::try_from(f.trunc())?`. After it, a writer who follows the hint on `f as i64` gets E0277. "Explicit rounding" names neither `round_from` nor `try_from_float`.
   - fx-u65.2, which owned the FX0301 text (research §1.1), is closed, so nothing open owns the fix.
   - Fix, minimum: name the real spellings, e.g. "use try_from, try_from_float, truncate, cast_signed/cast_unsigned or round_from".
   - Better: pick the hint by cast kind:
     - float to integer: `T::try_from_float(f.trunc())?`;
     - to a float: `round_from`;
     - sign change with narrowing: `x.cast_unsigned().truncate()`.
   - No test or case matches the message text (grep finds only line 174), so the one-line form is safe. If it is not folded in, file it on `fx-djp` or the `fx-u65` epic.

3. **Non-blocking. The design omits that `truncate` cannot reach a 32-bit target from a pointer-sized integer.**
   - Evidence: the truncate row at `design/reference.md:268` says "(same signedness)", which is accurate. It does not say that `usize`/`isize` reach only 16 and 8 bits. So `let lo: u32 = n.truncate();` with `n: usize` is E0277, and so is `u64` to `usize`. Hashing and indexing code hits this.
   - Decision 1 is correct to mirror core (research §1.1, option D). The workarounds are `(n as u64).truncate::<u32>()` and `usize::try_from(x)?`.
   - The reference has 5 tokens of headroom, so the row cannot carry this now.
   - Fix: carry it as a learnability note on `fx-djp` for the E1 analysis, and make sure the FX0301 hint in finding 2 does not suggest `truncate` for pointer-width targets.

4. **Non-blocking. The driver's lint allowance needs a pin-bump note.**
   - Evidence: `crates/tide-admit/src/invocation.rs:308-310` allows `unstable_name_collisions` for every user crate built with Tide's std. That is the right call for `Truncate`, which is what the research recommends. But it also silences the lint for user-defined trait methods that collide with other unstable inherent integer methods, such as `widen`, `checked_truncate` and `saturating_truncate`, which share the same feature gate.
   - When a later pin stabilizes such a method, the inherent one wins lookup silently, possibly with different semantics.
   - `rust-toolchain.toml`'s comment lists what is reviewed at a pin bump: the allowlist and the label layer. This is not on it.
   - Fix: extend that comment, or the invocation.rs comment, to say the following. At a pin bump, check whether core's `truncate` stabilized; if so, delete `Truncate` (research §4 pitfall 1). Also rebuild the conformance corpus without `-Aunstable-name-collisions` to surface new collisions.

5. **Nit. `truncate` and `round_from` lack `#[must_use]`.**
   - Evidence: `crates/tide-std/src/convert.rs:19` and `:34`. Core's inherent `truncate` carries `#[must_use = "this returns the truncated value and does not modify the original"]` (pinned `core/src/num/int_macros.rs:3985`).
   - `x.truncate::<u8>();` as a statement compiles silently. `try_from_float` is covered because `Result` is `must_use`.
   - Fix: add `#[must_use]` to both trait method declarations.

6. **Nit. No unit test feeds a subnormal to `try_from_float`.**
   - Evidence: the error lists in `crates/tide-std/src/convert/tests.rs:53-62` have no subnormal, and the brief names subnormals.
   - The behavior is correct by construction: `trunc` of a nonzero subnormal is ±0, which differs from the value, so the result is Err. This is by reading; the probe below could not run.
   - Fix: add `f64::from_bits(1)` and `-f64::from_bits(1)`, or `f32::from_bits(0x8000_0001)`, to an Err list.

7. **Nit. The bound comment is slightly loose for f32 to u128.**
   - Evidence: `convert.rs:96` says "gives the exact exclusive power-of-two bound, even for u128". For `$f = f32` and u128, the final `as f32` turns 2^128 into +inf.
   - That is still correct, because `is_finite` gates the comparison and every finite f32 is below 2^128, but the comment does not say so.
   - Fix: append "(f32/u128: +inf, safe because non-finite input is rejected first)".

8. **Nit. Two text nits.**
   - `conformance/rustc/std/src/lib.rs:8-9`: the reflow leaves the orphan line "//! and `std::json`,". Rewrap it.
   - `design/reference.md:262`: "Lossy `as`: `FX0301`:" stacks two colons. Change it only if a later edit frees the token: "Lossy `as` is `FX0301`:" was the old text. Do not spend budget on it now.

## What I checked and found correct

### Numeric correctness

- **Truncate pairs.** The `truncate_pairs!` list (`convert.rs:65-78`) matches pinned core's `impl_truncate!` (`core/src/num/traits.rs:99-112`) entry by entry, identity included.
  - Decision 1's text says "strictly narrower", but core and the research (§1.1) include identity. Following core is what keeps lookup compatible after stabilization, so the implementation is right and the decision's wording is the imprecise one.
  - The body is `self as $to`, which is two's-complement low bits. All report §5 vectors are in `convert/tests.rs:5-15` and the conformance case.
- **cast_signed and cast_unsigned.** These are Rust's stable inherent methods and are admitted under `core::num::*` (`policy.rs`). Both directions and the cross-sign composition are executed. No Tide inverse is needed.
- **round_from.** The pair list (`convert.rs:89-90`) is exactly decision 3's.
  - The body is `value as Self`, which the Rust Reference defines as nearest, ties to even, for integer to float and for f64 to f32. On SSE2 that uses the default MXCSR, and x87 is already rejected (README).
  - Tests cover both tie directions, negative ties, u64::MAX → 2^64, u128::MAX → +inf, ±1e39 → ±inf, NaN, and −0.0 sign preservation.
  - The pair-set test rejects exact u8 → f64 with E0277.
- **try_from_float.** The accepting condition is `is_finite && trunc == value && value >= MIN && value < 2^N`.
  - NaN fails `is_finite`, which also rejects ±inf. Fractions and subnormals fail the `trunc` test. −0.0 passes `>= 0.0` for unsigned targets, and `as` gives 0.
  - The upper bound is the exact 2^N for every width. For N ≤ 53, MAX+1 is exact. Above that, MAX rounds to 2^N and +1 is absorbed.
  - The lower bound −2^(N−1) is exact in both float types.
  - The value just below 2^63 maps to i64::MAX−1023. The value just past −2^63 is Err. Both sides of the i128/u128 limits are tested.
- **usize/isize portability.**
  - The bound expression is width-generic, so on 32-bit `usize::MAX as f64 + 1.0` is 2^32 exactly.
  - The truncate pairs are target-independent, as in core.
  - None of the claims in the docs, README or tests is 64-bit-specific. The only pointer-width literal is `usize::MAX.truncate::<u16>() == u16::MAX`, which holds on both widths.
  - The driver links x86_64 only, so leaving 32-bit execution untested is an honest evidence limit, not a gap in what the change claims.

### Admission

- **FX0301.** `lossless_cast` and the FX0301 path in `surface.rs` are untouched, and no policy rule changed.
- **Prelude resolution.** The three traits resolve through `declare_prelude!` (`root.rs:180`), per decision 5.
- **Nothing new admitted.** `pub mod convert { pub use std::convert::*; … }` re-exports the same items the old `pub use std::convert` did. Unstable items stay feature-gated, because stability is checked on the original item, and `-Zallow-features=` blocks core's own `truncate`, `widen` and the rest.
- **trunc.** `trunc` was already admitted (`policy.rs:134`).
- **Lint allowance scope.** The allowance is added only on the single `--tide-std` path (`invocation.rs:190`, `:308`). The stable oracle's collision warning is ignored by `run.sh`, which counts only errors.

### Decisions 1-7 and Sol's extra choices

- Decisions 1-7 are all met. See the decision 1 wording note above, and finding 1 for decision 7's tag.
- Sol's extra choices:
  - the lint allowance: right, but needs finding 4's lifecycle note;
  - private pair-marker seals: right;
  - the computed range bound: correct, see finding 7;
  - the shared fixture: right.

### Design review

- **Exact and minimal.** §4.2 changes only what decision 6 names: the truncate row's signedness clause, row 4's spelling, and the provisional-names sentence. The tightened allowed-cast paragraph pays the token cost.
- **No meaning lost**, clause by clause:
  - "same-signed integer widening" keeps the old meaning;
  - "unsigned to wider signed": "wider" is already strict, so u32 → i32 stays excluded;
  - "`bool` to integer", "fieldless enum to integer holding every discriminant" and "Addresses (11) count as `usize`" are equivalent to the old text;
  - the `usize`/`isize` 32-to-64 sentence is unchanged.
- **Consistency.** D3 (`01-design.md:318-325`) records the same semantics, including the fractional-input rejection that research §6 item 2 flagged as silent. 01 is the right owner for that decision. D3's link anchor `#1-fx-yg53-cast-replacements` matches the research heading. README, the crate docs and the trait docs agree with D3 and §4.2.
- **Token count.** `scripts/count-tokens.sh design/reference.md` gives **24,995**, the base gives 24,998, and the companion records 24,995, under D14's 25,000.
- **Stale spellings.** No live doc still teaches `i64::try_from(f.trunc())`. The only hits are the historical arm snapshots in `trials/e1-origins/`, which correctly stay as they are.

### Research copy

`tail -n +3 research/25-casts-arena-allowlist.md | cmp - /home/lucca/.cache/eon-agents/tide-research-yg5/report.md` gives IDENTICAL.

### Equivalence, docs and simplicity

- Production and the stub share `convert.rs` through `root.rs`, so they are equivalent by construction.
- `ADMIT-1` exists in `rules.md`. `GLOBAL-4` exists but is the wrong row (finding 1).
- I found no dead code or leftovers other than findings 2 and 7.

### fx-yg5.4 overlap

fx-yg5.4 candidate `29316c6` inserts at the same three points:

- `root.rs`, after `pub mod collections;`;
- `declare_prelude!`, after `pub use crate::marker::Pure;`;
- README, after the paragraph ending at line 387.

Each gives a textual rebase conflict that resolves by keeping both insertions. The `tide-std/src/lib.rs` hunks are 2-3 lines apart and may merge cleanly. These conflicts are inherent to both beads, not needless, and the recorded AP-PLAN-001 exception covers them. The later bead's reviewers re-check what the rebase changed.

## Probes

Probes 1-4 are reads and greps on Linux x86_64 at about 14:01-14:14Z, against pinned `nightly-2026-08-09`'s rust-src (rustc 1.99.0-nightly 771916f90). No build ran.

1. **Core pair set.** Read the pinned core source with `grep -rn "integer_widen_truncate\|fn truncate" $(rustup run nightly-2026-08-09 rustc --print sysroot)/lib/rustlib/src/rust/library/core/src/num/`, then `sed` on `traits.rs:1-140` and `int_macros.rs:3970-4060`. The `impl_truncate!` list is identical to `convert.rs`'s, and core's method is `#[must_use]`.
2. **Research copy.** `tail -n +3 research/25-casts-arena-allowlist.md | cmp - /home/lucca/.cache/eon-agents/tide-research-yg5/report.md` gave IDENTICAL.
3. **Token count.** `scripts/count-tokens.sh design/reference.md` gave 24995. The same script on `git show b7a994a:design/reference.md` gave 24998.
4. **Greps** (excluding `e1-checked`, `archive`, `target` and `.git`):
   - stale float `TryFrom` spellings: only the `trials/e1-origins` snapshots;
   - `can lose information`: only `surface.rs:174` (finding 2);
   - `REF-0301`: `rules.md:167`, `casts_accept.rs:1` (finding 1);
   - `trunc` in policy: `policy.rs:134` allow.
5. **`try_from_float` against an exact oracle.** `scratchpad/probe.rs` includes the production `convert.rs` via `#[path]`. It compares `try_from_float`, for all 12 integer targets from both f32 and f64, against an oracle that decodes the float's bits exactly (sign, exponent, mantissa → u128 magnitude or fractional/overflow) and applies integer `TryFrom`. The inputs are:
   - f64: specials (±NaN, ±inf, ±0, min subnormal, max negative subnormal, ±MIN_POSITIVE, MAX, MIN); ±70 ulps around every ±2^k for k = 0..=130, plus p±1 and p−0.5; and 20M random bit patterns plus 20M random integers and their +0.5;
   - f32: all 2^32 bit patterns.

   Command, after `build-gate.py 300` passed: `CARGO_BUILD_JOBS=4 nice -n 19 timeout 120 rustc +nightly-2026-08-09 --edition 2024 -O probe.rs -o probe </dev/null` with `ulimit -c 0`, then `nice -n 19 timeout 300 ./probe </dev/null`.

   Result: **not run.** `build-gate.py` refused every attempt from 14:03Z to 14:14Z ("load >= 12"; load was 12-29). A background loop retried every 10s until 14:14Z and printed "gate never passed". I made no further attempts because of the 14:20Z hold, so nothing was compiled.

   The `try_from_float` conclusions above (subnormals, bounds, −0.0, NaN/inf) therefore rest on reading the code and on the arithmetic argument given there. The probe source stays in my session scratchpad: `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-yg5-3/2bdbacfe-a560-499b-9da2-0219d92580fd/scratchpad/probe.rs`. It is cheap to run after 15:30Z if the coordinator wants exhaustive f32 evidence; its first run would also confirm finding 6's subnormal claim.

I ran no full suites and no `heavy.sh`, made no repository, staging or `br` writes, and did not open `trials/e1-checked/`.

DONE
