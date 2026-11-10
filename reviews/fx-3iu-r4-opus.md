Model: claude-opus-5-5 · Effort: low · Date: 2026-11-10

# fx-3iu round 4: rebase re-check (Opus, read-only)

Candidate: `d0982af` on main `e1e1751` (was `dffb442` on `07e104b`).
Scope: only what the rebase changed (`git range-diff 07e104b..dffb442 e1e1751..fx-3iu`,
plus the merged files). No builds or probes ran: the 1-minute load was 12.06, at the
limit, and no question needed one.

## Verdict

approve

## Per-item status

1. **The shared helper keeps both sides' hardening.** OK.
   `crates/tide-std/tests/support/mod.rs` is now the only build path. Both
   `tide-admit/tests/build.rs:7-8,26` and `tide-std/tests/entry.rs:10,29` call it, and
   the branch's duplicate cargo-rustc/direct-rustc paths are gone. Hardening from
   fx-yg5.2 that survives:
   - artifact names come from Cargo's JSON `filenames` (`:57-74`);
   - the `artifacts` directory is cleared before each build (`:50-54`);
   - `libtide_std.rlib` is asserted (`:79`);
   - RUSTFLAGS is gone, because `env_clear()` (`:20`) also drops `CARGO_ENCODED_RUSTFLAGS`,
     `CARGO_BUILD_*`, and `RUSTC_WRAPPER`/`RUSTC_WORKSPACE_WRAPPER`. That is stricter than
     main's `env_remove("RUSTFLAGS")`.

   Hardening from fx-3iu that survives:
   - `env_clear` keeps only HOME/CARGO_HOME, with a fixed PATH and `CARGO_BUILD_JOBS=4`;
   - the build uses `--frozen`, which is stricter than main's `--offline --locked` only in
     also forbidding network access;
   - RUSTC is the absolute `<sysroot>/bin/rustc`;
   - the macro's name is computed from `DLL_PREFIX`/`DLL_SUFFIX` (`:55,65-66`). Cargo
     reports a hashed `libtide_macros-<hash>.so`, and the helper asserts that the
     artifact was reported (`:75-78`), so it no longer guesses the name.

   Both the std crate and the trusted macro are still built by plain rustc, never by the
   admission driver: the wrapper variables are cleared and RUSTC is the toolchain's own
   binary. The syn/quote/proc-macro2/libm rlibs and the macro `.so` reach the flattened
   directory through the JSON loop. The trusted-build boundary is not weakened; clearing
   inherited wrappers makes it tighter than main's.

   Compiler choice: the helper queries `rustc --print sysroot` with the inherited
   environment from `CARGO_MANIFEST_DIR`, so `rust-toolchain.toml`, RUSTUP_TOOLCHAIN, and
   RUSTC resolve exactly as main's cargo call did. For tide-admit this is the same
   toolchain that sets `TIDE_SYSROOT` in `crates/tide-admit/build.rs:7-14`. The branch's
   earlier explicit use of `TIDE_SYSROOT/bin/rustc` gives way to main's resolution, so
   main's behavior did not regress. No `.cargo/config*` exists in the worktree, its
   ancestors, or `~/.cargo`.

2. **No test was lost or weakened.** OK.
   - `git diff e1e1751 fx-3iu` does not touch `tide-std/tests/entry.rs`; it equals main.
   - `tide-admit/tests/build.rs` has 206 insertions and 0 deletions against main.
   - The Error-test suffix from `fn error_derive_builds_and_executes_against_production_std`
     to the end of the file is byte-identical to dffb442's (checked with `diff`).
   - The author's script found every `(file, test fn)` pair from both bases.
   - The round-1 N6 entry.rs reflow is gone with the branch's entry.rs changes, which
     settles that nit.

3. **Cargo.lock pins match both sides.** OK.
   - Root `Cargo.lock` against main adds only proc-macro2 1.0.107, quote 1.0.47,
     syn 2.0.119, unicode-ident 1.0.27, and tide-macros' dependency edges.
   - Main's libm 0.2.16 entry and checksum are unchanged, and `tide_std` keeps its `libm`
     dependency.
   - Version, source, and checksum lines of the syn family match dffb442 exactly.
   - `conformance/rustc/std/Cargo.lock` holds the same five pins. Against main, its only
     change is tide-macros' dependency list.
   - The manifests keep their `=` pins: `tide-std/Cargo.toml:10` and
     `tide-macros/Cargo.toml:12-14`.

4. **Docs are accurate.** OK.
   - `README.md:169-171` inserts the Error derive clause with its exclusions before main's
     task-runtime sentence. The cancellation and float/math sentences are intact.
   - `tide-std/src/lib.rs:19-22` adds the derive and its exclusions next to main's float,
     math, and collections bullets.
   - `:42` drops `Error` from the unavailable derives, and main's rewrite of that list
     (collections, float, and math now available) is preserved.
   - The conformance oracle std drops `oracle_macros::Error` as reviewed earlier, with no
     conflict against main's libm addition.

5. **Interaction with newly landed code.** No issues.
   - Sinks (`walk/sinks.rs`): `sink_expansion` fires only for Bang macros named
     `dbg`/`print`/`println`. The derive is a Derive-kind macro and emits only
     `::core::write!` (`tide-macros/src/error.rs:410`), so neither the generated
     Display impls nor the walk's new path produce a spurious FX0507/FX0508.
     `lint_sink_expr` keys on `mpsc::channel` and `Shared::with*`, which the derive
     never emits.
   - The branch's `path_args` change in `walk.rs:1830-1839` merged next to main's
     `self.lint_sink_expr(e, t)` call, which survives at `:1832`.
   - Collections, task cancellation, and float/math share no code path with the derive.
     Fields of those types format and convert through the user's own format strings
     and `#[from]`, as with any type. The new `<(dyn core::error::Error*)>::*` policy line
     is independent of main's policy changes.
   - The author's suites on `d0982af` cover the combined surface: workspace 93 passed;
     conformance 213 PASS, 24 AWAITS, 0 FAIL.

## New findings

None blocking.

- Nit (non-blocking): the doc comment on `build_std` (`support/mod.rs:4-5`) does not
  mention that the build is hermetic (cleared environment, `--frozen`, absolute RUSTC).
  A one-line note would help the next editor keep that property.

DONE
