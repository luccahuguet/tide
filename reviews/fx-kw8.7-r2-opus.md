# Tide review, fx-kw8.7 round 2 (Opus): rustc ICE reports in the candidate tree

## Verdict

**Changes made** (converge confirmation round, fixing): one test change;
the driver code is clean.

## Change

`crates/tide-admit/tests/invocation.rs` (`crashes_write_no_report_files`):
the file walk skipped everything under `out/`, so a crash report written
into the output directory outside `.tide-cache` would have passed, though
the bead's Acceptance forbids it. The walk now skips only `out/.tide-cache`,
sorts the files, and requires exactly `[out/liblib.rmeta,
package/src/lib.rs]`, the metadata the warm-up run copies out plus the
source.

## Confirmed

- Both spawn sites give the compile process exactly `RUSTC_ICE=0`, so
  `ice_path_with_config` returns `None` whatever `-Zmetrics-dir` says and
  neither report write runs (rustc_driver_impl lib.rs:1487, 1561); lazy
  `ice_path()` calls from panics before `run_compiler` read the same. The
  `vars_os().ne([..])` check re-execs on an empty environment (Codex B1).
- Moving the hook: besides the panic hook, `install_ice_hook` only sets the
  backtrace style when `RUST_BACKTRACE` is unset, always so in the clean
  process. Panics before the re-exec use Rust's default hook (Codex B2).
  Exit codes are unchanged (101 for a crash; the parent forwards the child's
  code; `conformance/run.sh` reads non-zero).
- `--tide-test-ice` is in the cache key and the warmed cache exercises it;
  failures are not cached and staging is removed on failure.
- README's two edits match the behavior.

## Check

`cargo test --workspace`, stdin closed: exit 0 (2 + 9 + 14).

## Out of scope

- A crash by signal can leave a core file in the compile child's current
  directory where the core pattern is relative (round 1's F2; RUN-2's OS
  sandbox).
- A host that injects environment variables into every process makes the
  re-exec loop; true of the earlier empty-environment check as well.
- `--tide-test-ice`, like `--tide-test-backstop`, is accepted by any build;
  it can only make a run fail (operators are trusted with these flags,
  fx-kw8.4).
