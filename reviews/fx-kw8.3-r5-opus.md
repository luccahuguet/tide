Model: Opus 5.5. Date: 2026-10-22. Round 5 (converge, fixing), candidate 6077bc9f (uncommitted on ce0f0cc).

# fx-kw8.3 round 5 (Opus): tide-admit owns every rustc invocation

fx-kw8.3 fresh pass (fresh-pass skill, one pass) on the uncommitted candidate over ce0f0cc. I found no correctness defect in the invocation layer and made three small simplifications. Both checks pass on the final tree. I did not commit, did not write beads, and did not touch `reviews/` or `.beads/`.

**Fixed**
1. **Leftover `Default` on `Grants`.**
   - **Problem:** the diff removed `main.rs`'s `Grants::default()`, so the `Default` derive on `Grants` (`crates/tide-admit/src/walk.rs:47`) had no user. Meanwhile `Invocation` kept a separate `unsafe_: bool` that `run()` copied back into a `Grants`.
   - **Fix:** `Invocation` now holds `grants: Grants` (`/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:53`). Parse sets `this.grants.unsafe_` (line 72), the cache key reads `invocation.grants.unsafe_` (line 623), and `Admission` takes `invocation.grants` directly (line 656). The derive is used again through `Invocation`'s `#[derive(Default)]`.
2. **Duplicate test case.**
   - **Problem:** `each_loaded_source_is_guarded_before_expansion` (`/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs:151`) had a top-level `macro_rules!` → FX0101 case. That verdict comes from `check_expanded` after expansion, not from the pre-expansion guard the test is named for. It is already covered by the `macro_rules!` + FX0901 case in `candidate_content_gets_rustc_diagnostics_and_every_verdict` and by the macro-argument cases.
   - **Fix:** I removed the case. Every remaining case expects FX0911, so the `(source, code)` tuples are now a plain list asserting FX0911.
3. **Redundant directory creation in `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/driver.rs`.**
   - **Problem:** both `admit()` and the reject test still called `create_dir_all(&dir)` right before the diff's new `create_dir_all(dir/package)`, which already creates `dir`.
   - **Fix:** I dropped the two redundant calls.

I formatted all three files with `rustfmt +stable --edition 2024`.

**New-surface report**
- `pub fn run()` (`invocation.rs:514`) stays. It is `main()`'s only call into the module. `invocation` is a private module of a binary, so `pub` reaches only within the crate, the same as `walk::check_crate`.
- The match at `tests/invocation.rs:211` is a string literal in a test, not surface.

**Checked and left alone**
- **`#[debugger_visualizer]` does not read outside the loader on this pin.** `rustc_passes/src/debugger_visualizer.rs` reads the file through `source_map().load_binary_file`, which goes through `Sources::read_binary_file` (snapshot `.rs` files only). Argon's claim that it reads a host file outside FileLoader does not hold here.
- **Nested-attribute probes on the built driver:**
  - `#[path]` inside `assert!` arguments: FX0911.
  - `#[requires({ #[path] mod x; })]`: rustc rejects it as an unknown attribute (exit 1).
  - A benign `cfg_attr(..., doc = "path")`: admitted (exit 0).
- **Redundant `.env_clear()` and `.stdin(Stdio::null())` on the `--tide-compile` child (around line 664).** The top-level re-exec already guarantees an empty environment and null stdin, so these two lines do nothing. I kept them because they make the child's isolation visible where it is spawned, and removing them would leave that safety depending on code far away.
- **Kept, as earlier rounds decided:**
  - the env-clearing re-exec: RUN-2 says the driver itself runs with the toolchain's environment;
  - `Sources` being `Clone` with `Arc` fields;
  - three near-identical fixture `rustc` builds in `unavailable_trusted_dependency_source_does_not_crash_error_rendering`. Folding them needs a new helper, which a simplification pass does not add.
- **Out of scope, already reported by round 3/4:**
  - FX0915 looks only one level up from `src`;
  - `#[unsafe(path = ..)]` in macro arguments gets a different diagnostic, though the loader and `check_expanded` still stop it;
  - `fingerprint` follows symlinks in trusted directories with no cycle guard;
  - an unreadable source subdirectory gives exit 2;
  - `rustc-ice-*.txt` files land in the current directory;
  - `destination()` errors on an out-dir path whose missing part contains `..`.

**Verification** (final tree, Linux x86_64, pinned nightly-2026-08-09, run one at a time)
- `timeout 1200 cargo test --workspace < /dev/null`: exit 0. All 20 tests passed (2 + 5 + 13), with no warnings or errors.
- `timeout 1800 conformance/run.sh < /dev/null`: exit 0. 180 PASS, 28 AWAITS CHECKER, 0 FAIL.
- `git status` shows the same set of changed and untracked files as at the start, and nothing stray (`conformance/target/` is ignored build output).
- Probes and the conformance log are in `/tmp/tmp.JoiMdBQC9v` (`/tmp/tmp.JoiMdBQC9v/probe.sh`, `/tmp/tmp.JoiMdBQC9v/conf.log`).

Changed files:
- `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs`
- `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs`
- `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/driver.rs`
