Model: Opus 5.5. Date: 2026-10-22. Round 4 (converge, fixing), candidate ce9debaf (uncommitted on ce0f0cc).

# fx-kw8.3 round 4 (Opus): tide-admit owns every rustc invocation

I made one fresh pass with the `fresh-pass` skill and made two changes. I found no correctness defect in the invocation code itself. Both required checks pass on the final tree. I did not commit, did not write beads, and did not touch `reviews/` or `.beads/`.

**Fixed**
1. **The environment-canary test could not fail.** It is `inherited_environment_and_candidate_configuration_cannot_configure_rustc` in `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs`, around lines 77-97.
   - **Problem:** none of its four `env!`/`option_env!` cases can ever print the canary. ADMIT-1 rejects each one after expansion, so stderr and the output directory stay clean whether or not the driver clears the environment.
   - **Seed check:** I removed the top-level re-exec and both `env_clear()` calls in `invocation.rs`. The test still passed with the four original cases. So it did not prove RUN-2's "the value appears nowhere" for the caller's environment.
   - **Fix:** I added one case, `compile_error!(env!("TIDE_TEST_CANARY"));`. On this pin `compile_error!` expands its argument eagerly, so an inherited value would reach stderr (same mechanism as the existing `compile_error!(include_str!(..))` trusted-read case).
   - **Result:** with the same seed, the test now fails at the `!contains("PRIVATE_ENV_CANARY")` assertion. With the real code it passes. I put the source back from a copy after the seed run and confirmed it matched exactly.
2. **Inlined `Sources::denied`.** It was a two-line `-> !` helper with one caller, `read_file`, in `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs`.
   - The `eprintln!` + `FatalError.raise()` now sit at the end of `read_file`.
   - The comment saying why this must not return `Err` (rustc_parse reopens the path after any loader `Err` on this pin) moved with it.

I formatted both files with `rustfmt +stable --edition 2024`.

**New-surface report**
- `pub fn run()` (`crates/tide-admit/src/invocation.rs`, now near line 512) stays. It is `main()`'s only call into the module, so `main.rs` has to see it. `invocation` is a private module of a binary, so `pub` only reaches within the crate, which matches `walk.rs`. Round 3 already made `Sources` and `check_expanded` private.
- The second match, at `tests/invocation.rs:209`, is a test string literal, not new surface.

**Left alone**
- **Three processes on a cache miss** (env-clearing re-exec, then the `--tide-compile` child). Earlier rounds kept this on purpose: RUN-2 says the driver itself runs with an environment the toolchain sets.
- **`Sources` is `Clone` with `Arc` fields only so `config` can clone it.** Moving it out of an `Option` instead would save little; it is a matter of taste, not a defect.
- **`foreign` is recomputed on every `read_file` by scanning the source map.** rustc reads only a few files per crate, so the cost is negligible.
- **Test name** `each_loaded_source_is_guarded_before_expansion`: its `macro_rules!` case gets FX0101 from `check_expanded` after expansion, not from the pre-expansion guard. The name is slightly misleading, but not a defect.
- **Still out of scope, as round 3 reported:**
  - FX0915 looks only one level up from `src`.
  - The macro-argument token scan misses `#[unsafe(path = ..)]` (only the diagnostic changes; the loader and `check_expanded` still stop the read).
  - `fingerprint` follows symlinks in trusted dependency directories with no cycle guard.
  - An unreadable subdirectory in the snapshot gives exit 2.
  - `rustc-ice-*.txt` files land in the current directory under the cleared environment.

**Verification** (final tree, Linux x86_64, pinned nightly-2026-08-09, run one at a time)
- `timeout 1200 cargo test --workspace < /dev/null`: exit 0. All 20 tests passed (2 + 5 + 13), no warnings.
- `timeout 1800 conformance/run.sh < /dev/null`: exit 0. 180 PASS, 28 AWAITS CHECKER, 0 FAIL.
- `git status` shows the same files as before, with no stray files. The seed backup and the conformance log are in a `mktemp -d` scratch directory outside the repository: `/tmp/tmp.seNm8aDikm`.

Changed files: `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs`, `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs`.
