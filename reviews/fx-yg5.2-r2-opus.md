Model: claude-opus-5-5 · Effort: narrow re-check (round 2, report-only) · Date: 2026-11-09

# fx-yg5.2 round 2: Opus re-check

## Verdict

**approve.** B1 and B2 are resolved. The taken items N1–N6 are resolved as the coordinator decided them. I found no new blocking defect in the changed lines; one nit is below.

## What I checked

- `git diff 0823998 fx-yg5.2` (HEAD `7b50b3f`), fix1-brief.md, and fix1-report.md.
- Tree identity: `git diff --binary 0823998 7b50b3f` minus the new oracle file hashes to `9c24fe2e…0ad2`, and `conformance/oracles/float_math.py` hashes to `50c5192b…a36a`. Both match the report, so the logged final runs were made on the committed candidate.
- Logs:
  - `target/fix1-workspace.log`: every suite ok, 0 failed.
  - `target/fix1-conformance.log`: `PASS authority/float_math.rs`.
  - The worktree's `target/e1-checked/`: `tide-vendor-87ed651a014d1f27` exists beside the moved-aside `cargo-home.pre-fix1-…`, and a `find` for libm in the recreated `cargo-home` returns nothing.
- Independent recomputation, using Python `fractions`, not the code under test:
  - `1.5^7 = 17.0859375` is exact, and 1/17.0859375 rounds to f32 `0x3d6fbab4` and f64 `0x3fadf75680feb65f`. Both match the re-recorded vectors.
  - Rust core's angle constants equal the oracle's rounded constants in both widths: f64 `180/PI` and `PI/180`, and f32 `PI/180`.
- No rustc or cargo probes. The logged runs and my recomputation settle every item, so no probe was needed.

## Per-finding status

**B1 (powi reciprocal-first): resolved.**
- `crates/tide-std/src/math.rs:40-51` now squares and multiplies on `x`, then returns `1.0 / result` for `n < 0`. This is the same order as compiler-builtins' `pow`, including breaking the loop before the last squaring. There is no fallback path.
- `i32::MIN` is still handled by `unsigned_abs`. `powi(-1, i32::MIN)` gives 1, and `powi(-0.0, -3)` gives 1/(-0.0) = -∞. The existing fixture lines 163–169 and 442–448 still cover both.
- Vectors are re-recorded at `conformance/cases/authority/float_math.rs:149,414`. Lines 185 and 473 now expect `+0` for `powi(2f32,-149)` and `powi(2f64,-1074)`. Lines 516 and 560 add `powi(10,-2)` equal to the correctly rounded 0.01 (f32 `0x3c23d70a`, f64 `0x3f847ae147ae147b`).

**B2 (harness clean-state build): resolved.**
- `trials/e1-checked/harness.py:142-174` generalizes the old `rust_vendor` into `offline_vendor(arm, project, channel)`. It keeps the lock-immutability check, the atomic rename, the cache reuse, and the failure cleanup.
- `tide_vendor()` vendors `ROOT/Cargo.lock` with the pinned nightly cargo into `tide-vendor-<hash16>`. On failure it prints the remedy `cargo +nightly-2026-08-09 fetch --locked`.
- `base_config` (`harness.py:492-503`) builds with `--config source.crates-io.replace-with="vendored"` and `source.vendored.directory=<json-quoted path>`, under the isolated `CARGO_HOME`.
- `trials/e1-checked/README.md:70` documents the setup.
- `harness_test.py:381-408` mirrors the Rust-arm test: pinned flags, manifest path, cache hit, failure cleanup, and remedy text.
- The clean-home A5 run built the std with no hand-copied libm. That is the falsifier B2 asked for.

**N1 (self-recorded vectors): resolved.**
- `float_math.rs:2-5` labels the original literals as self-recorded regression vectors.
- `conformance/oracles/float_math.py` is a committed generator. It never calls Tide, libm, or host transcendental functions: it computes 256-bit MPFR directed intervals and accepts a value only when both ends round alike. Its checked-in output is `float_math.rs:477-569`, and the script's default mode verifies that the fixture still matches.
- The ULP bounds come from upstream libm-test's `precision.rs` at the audited commit, and the script links it.
- The report records the provenance (system MPFR 4.2.1, `/usr/bin/python3`).

**N2 (x87 caveat): resolved as decided.**
- `crates/tide-std/src/float.rs:5-6` adds `compile_error!` under `all(target_arch = "x86", not(target_feature = "sse2"))`. Conformance shares the file through its path import.
- This condition covers every `use_arch_required` site the report lists. All eight are `x86_no_sse` or i586 floor/ceil, each a subset of non-SSE2 x86.
- `design/01-design.md:327-328` adds the D3 clause.
- The report's audit agrees with what I read of libm 0.2.16 in round 1 (`configure.rs`, `support/macros.rs`, the exp/floor/ceil sites).
- The guard probe simulates the cfg on the host; no real i586 target was built. The report says so.

**N3 (core compositions): resolved as decided.**
- `math.rs:5-6` adds the sentence the coordinator decided on. midpoint, to_degrees, to_radians, and clamp still delegate to core, which is the coordinator's choice.
- The new formula assertions (`float_math.rs:520-521,564-565`) pin the angle results, and I confirmed their constants above.

**N4 (test support helper): resolved.**
- `crates/tide-std/tests/support/mod.rs:9-12` adds `--message-format=json` and `env_remove("RUSTFLAGS")`.
- Lines 22-24 clear the `artifacts` directory before copying.
- Lines 26-35 scan the `"filenames":[…]` arrays: rlib, so, dylib, and dll files are copied, and build-script executables and rmeta files are skipped. Fresh units still emit artifact messages, so a cached build also works.
- Line 36 asserts that the std artifact is present.
- The unused `run` helper is gone from `tests/entry.rs`, and `Output` and `Stdio` are still used.
- Limitation, not a defect: the scan assumes paths without JSON escapes (`"` or `\`). That holds on Unix target directories.

**N5 (CARGO_BUILD_JOBS scope): resolved.**
- `environment()` (`harness.py:182`) is fixed at `"6"` again.
- Only the trusted `build_env` (`harness.py:497`) honors the operator override.
- `harness_test.py:86-87` asserts that an inherited `4` does not leak into the writer and grading environment.

**N6 (Bits bounds): resolved.** `float.rs:16` now reads `type Bits: Copy + Eq + Ord + core::hash::Hash + core::fmt::Debug`, and both `u32` and `u64` satisfy it.

## New findings in changed lines

**Nit (non-blocking): the oracle's f32 exact and formula path rounds twice.**
- In `conformance/oracles/float_math.py`, `rounded(result, 32)` converts a `Fraction` to an f64 first, then to an f32 (`struct.pack(">f", …)`). The angle formulas take the same route through `x * factor`.
- So the "exact" f32 expectation is computed as rational → binary64 → binary32. That can differ from the correct single rounding when the f64 lands exactly on an f32 halfway point.
- None of the current inputs hit that case; I checked 0.01 and the powi values independently.
- If the generator gains inputs later, round the `Fraction` straight to binary32, for example with nearest-even on the scaled numerator. Alternatively, assert that the f64 intermediate is not an f32 tie.

DONE
