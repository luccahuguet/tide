Model: Opus 5.5. Date: 2026-10-22. Round 2 (converge, fixing), candidate 3088826c (uncommitted on ce0f0cc).

# fx-kw8.3 round 2 (Opus): tide-admit owns every rustc invocation

One pass made three deletions. Both required checks pass on the final tree. I did not commit, and I did not touch `reviews/` or `.beads/`.

### Fixed
1. **The thread-local source map, `Sources::attach`, and the `psess_created` hook are gone.**
   - **Where:** `crates/tide-admit/src/invocation.rs` (the old `SOURCE_MAP` thread_local, `attach`, `foreign_source`) and `crates/tide-admit/src/main.rs` (`Admission::config`).
   - **Why:** the pinned rustc already exposes the session's source map as `rustc_span::source_map::get_source_map()` (`rustc_span/src/source_map.rs:1150`). The Session's own map is that same Arc (`rustc_session/src/session.rs:1239`). So the Weak/RefCell TLS, the hook that filled it, and the second `Sources` clone in `config` were duplicate machinery.
   - `read_file` now computes `foreign` inline from `get_source_map()`. `foreign_source` had one caller and an unused `&self`, so it is inlined too.
   - Imports removed: `RefCell`, `Weak`, `SourceMap`.
   - **Seed check:** I forced `foreign = false`, and `unavailable_trusted_dependency_source_does_not_crash_error_rendering` failed with "FX0520: compile-time file read denied". I restored the code and the test passes. The new lookup is load-bearing and finds the right map.
2. **`hash()` is inlined** at its only call site in `run()`. It was a single-call wrapper.
3. **A duplicate write is deleted** from `crates/tide-admit/tests/invocation.rs` (cache test): `deps/input` was written with "one" just before a loop that writes "one" again.

I formatted the three files with `rustfmt +stable --edition 2024`.

### New-surface items
- **`attach`:** removed (fix 1).
- **`Sources` stays.** `Admission` in main.rs holds it and installs it as `config.file_loader`.
- **`check_expanded` stays.** main.rs's `after_expansion` calls it, and it must run before `walk::check_crate` lowers the crate. It is the post-expansion FX0911/FX0101 check.
- **`run` stays.** It is main's only entry point.
- `pub` inside the private module of a binary is already crate-only, which matches `walk.rs`.

### Left alone
- **Token scan of macro arguments** (`forbidden_macro_attributes` plus `Guard::visit_mac_call`). The loader and `check_expanded` already keep bytes confidential without it. Removing it would only change an outside `#[path]` in macro arguments from a located FX0911 to an unlocated FX0520. That is an intended diagnostic choice, not a defect. Its known false positive on `stringify!(#[path = ..] ...)` is conservative.
- **The extra re-exec at startup and hashing the 82 MB exe on every run.** These cost time but are not defects. The re-exec follows RUN-2's literal "the driver runs with an environment the toolchain sets".
- **Three near-identical fixture `rustc` invocations** in `unavailable_trusted_dependency_source_does_not_crash_error_rendering`. Collapsing them would mean adding a helper, which this pass excludes.
- **Out of scope, not fixed:**
  - The cache test hard-codes `x86_64-unknown-linux-gnu` for the std rmeta path.
  - FX0915 detection only looks at the entry's parent's parent when that parent is `src` (round-1 N9).
  - Denied reads carry no location (round-1 N3).
  - The FX0101 exemption uses `in_external_macro`, which is broader than intended (round-1 N10).

### Verification (final tree, Linux x86_64, pinned nightly-2026-08-09)
- `timeout 1200 cargo test --workspace < /dev/null`: exit 0, 20 tests (2 + 5 + 13), 0 failed.
- `timeout 1800 conformance/run.sh < /dev/null`: exit 0, 180 PASS, 28 AWAITS CHECKER, 0 FAIL.

Changed files: `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs`, `/home/lucca/pjs/lab/tide/crates/tide-admit/src/main.rs`, `/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs`.
