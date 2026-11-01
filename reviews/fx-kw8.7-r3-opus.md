# Tide review, fx-kw8.7 round 3 (Opus): rustc ICE reports in the candidate tree

## Verdict

**Changes made** (converge confirmation round, fixing): two small fixes;
the driver code is unchanged and correct.

## Changes

1. `conformance/README.md`: it still said the driver "runs the compiler
   with an empty environment", false since the compiler gets
   `RUSTC_ICE=0`. It now says the environment holds only `RUSTC_ICE=0`, so a
   crash writes no report file. No other doc claims an empty environment.
2. `crates/tide-admit/tests/invocation.rs` (`crashes_write_no_report_files`):
   `.env_remove("RUSTC_ICE")` on the `--tide-print-codes` / `/dev/full`
   subtest, the only part that runs the driver with the test's own
   environment before the re-exec. With `RUSTC_ICE` set in the developer's
   shell, a regression that installs the hook at the start of `main` would
   have passed.

## Confirmed

Both spawn sites give the compile process exactly `RUSTC_ICE=0`, so the
pinned rustc's report path is `None` whatever `-Zmetrics-dir` says
(rustc_driver_impl lib.rs:1408-1412); `vars_os().ne([..])` re-execs on an
empty or any other environment; the hook is installed only after the
environment is normalized; `--tide-test-ice` is in the cache key and never
reaches rustc; failures are not cached and their staging is removed; exit
codes are forwarded; README.md's edits match the code.

## Checks

`rustfmt --check` clean on the test file. `cargo test --workspace`, stdin
closed: exit 0 (2 + 9 + 14). No `rustc-ice-*` files outside `target/`.

## Out of scope

As round 2: core files from a crash by signal (RUN-2's OS sandbox); a host
that injects environment variables makes the re-exec loop; `/dev/full`
ties the test to Linux.
