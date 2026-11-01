# Tide review, fx-kw8.7 round 4 (Opus): rustc ICE reports in the candidate tree

## Verdict

**Clean**: no changes. Converge confirmation round on the tree after round
3 (`git diff -- README.md crates conformance/README.md` against HEAD
`8673252`).

## What was checked

- Pinned rustc (rustc_driver_impl lib.rs:1397-1430): `RUSTC_ICE=0` makes the
  report path `None` before `-Zmetrics-dir` or `current_dir()` is consulted;
  both report writes and the session's report file read that `None`.
- Both spawn sites start the child with exactly `RUSTC_ICE=0`, and
  `vars_os().ne([(RUSTC_ICE, 0)])` re-execs on an empty or any other
  environment (Codex B1).
- `install_ice_hook` runs only after the environment is normalized; earlier
  panics use Rust's default hook (Codex B2).
- `--tide-test-ice` takes no value, is in the cache key, and never reaches
  rustc; failures are not cached and staging is removed; the child's exit
  code and crash stderr pass through.
- `crashes_write_no_report_files` warms the cache, asserts rustc's crash
  message and the seeded panic, covers the `/dev/full` panic with
  `RUSTC_ICE` removed, and requires exactly `out/liblib.rmeta` and
  `package/src/lib.rs` outside `out/.tide-cache`, matching the Acceptance.
- README.md and conformance/README.md match the code; design/03's "an
  environment the toolchain sets" still holds.

## Checks

`rustfmt --check` clean on the three Rust files; `git diff --check` clean;
`cargo test -p tide-admit --test invocation crashes_write_no_report_files`
passed. No `rustc-ice-*` files outside `target/`.

## Nits (not changed)

README's "keeping only `RUSTC_ICE=0`" would be more exact as "setting
only"; the test's doc comment could be misread.

## Out of scope

As rounds 2 and 3.
