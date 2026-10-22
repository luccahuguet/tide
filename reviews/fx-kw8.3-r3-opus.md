Model: Opus 5.5. Date: 2026-10-22. Round 3 (converge, fixing), candidate 01b330bc (uncommitted on ce0f0cc).

# fx-kw8.3 round 3 (Opus): tide-admit owns every rustc invocation

I made one fresh pass with the `fresh-pass` skill and made five changes. I found no correctness defect in the invocation boundary itself. Both required checks pass on the final tree. I did not commit, did not write beads, and did not touch `reviews/` or `.beads/`.

### Fixed
1. **`Admission` moved from `main.rs` into `invocation.rs`.**
   - **Where:** `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:27-42`, and the code removed from `/home/lucca/pjs/lab/tide/crates/tide-admit/src/main.rs`.
   - **Why:** the diff made `invocation::run` build `crate::Admission`, while `Admission` held `invocation::Sources` and its callback called `invocation::check_expanded`. That is a cycle between `main` and `invocation`, and it is the only reason `Sources` and `check_expanded` were `pub`.
   - **Result:** the struct and its `Callbacks` impl now sit beside their only constructor. `main.rs` keeps only the module list and `main()`. Its unused imports (`Callbacks`, `Compilation`, `Compiler`, `TyCtxt`, `Grants`) are gone.
   - I import `check_crate` directly rather than the `walk` module, so `Sources::snapshot`'s local `fn walk` no longer shadows the module name.
   - I updated the module doc on line 1.
2. **`Sources.cwd` removed.** `current_directory` now returns `std::env::current_dir()`, which is exactly what rustc's own `RealFileLoader` does. The driver never changes directory, so the stored copy only duplicated it.
3. **Hard-coded host triple fixed** in `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs` (cache test, near line 356).
   - The test built the std path from `x86_64-unknown-linux-gnu`, so it would fail on any other host.
   - It now reads the triple from the `host:` line of `TIDE_COMPILER_ID`, which this diff's `build.rs` already exports.
4. **Leftover crash report deleted:** `crates/tide-admit/rustc-ice-2026-10-22T23_02_55-1452005.txt`.
   - Its message is the known "TyKind::Error constructed but no error reported" crash from round 2's seed check that forced `foreign = false`. That seed's run wrote the file, and it should not be in the diff.
   - The full test run did not create a new one.
5. **README line reflowed** in `/home/lucca/pjs/lab/tide/README.md` (around line 79). One line ran to about 110 columns.

I formatted the edited Rust files with `rustfmt +stable --edition 2024`.

### New-surface report
- **`Sources` (`invocation.rs:189`): no longer `pub`** (fix 1).
- **`check_expanded` (`invocation.rs:457`): no longer `pub`** (fix 1).
- **`run` (`invocation.rs:500`): stays `pub`.** It is `main()`'s only call into the module, so `main.rs` must be able to see it.

### Left alone
These are earlier review findings I judged not to be defects, or decisions earlier rounds already recorded:
- **Process count:** a cache miss starts the driver three times and hashes the executable on every run. This costs time but is not a defect, and the up-front re-exec follows RUN-2's wording that the driver runs with an environment the toolchain sets.
- **The token scan of macro arguments** (`forbidden_macro_attributes`):
  - It is kept for located FX0911 diagnostics; the file loader is the real boundary.
  - It wrongly rejects `stringify!(#[path = ..] ..)`, which is the cautious direction.
  - It misses `#[unsafe(path = ..)]` written inside macro arguments. That only changes which diagnostic appears: the loader and `check_expanded` still stop any read.
- **The three near-identical fixture `rustc` invocations** in `unavailable_trusted_dependency_source_does_not_crash_error_rendering`. Collapsing them would mean adding a helper, which this pass excludes.
- **Out of scope, reported only:**
  - FX0915 detection only looks one level up when the entry's directory is `src`.
  - Denied reads carry no file location.
  - The FX0101 exemption uses `in_external_macro`, which is broader than needed.
  - `fingerprint` follows symlinks with no cycle guard. This only affects trusted operator dependency directories.
  - Unreadable subdirectories in the snapshot cause a setup error (exit 2) instead of reading as missing.
  - The cleared environment means rustc's own crash reports (`rustc-ice-*.txt`) land in the current directory, which can be inside the candidate tree. That would be a write outside the cache.

### Verification (final tree, Linux x86_64, pinned nightly-2026-08-09)
- `timeout 1200 cargo test --workspace < /dev/null`: exit 0, 20 tests (2 + 5 + 13), 0 failed, no warnings.
- `timeout 1800 conformance/run.sh < /dev/null`: exit 0, 180 PASS, 28 AWAITS CHECKER, 0 FAIL. The full log is `/tmp/claude-1000/-home-lucca-pjs-lab-tide/8a915604-6e15-4fb6-bdec-19f443dee621/scratchpad/conf.log`.

Changed files: `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs`, `/home/lucca/pjs/lab/tide/crates/tide-admit/src/main.rs`, `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs`, `/home/lucca/pjs/lab/tide/README.md`. The ICE file is deleted.
