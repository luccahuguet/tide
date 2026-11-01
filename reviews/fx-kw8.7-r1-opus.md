# Tide review, fx-kw8.7 round 1 (Opus): rustc ICE reports in the candidate tree

## Verdict

**Clean**: no blocking findings; two non-blocking findings and nits. A
converge round that reported instead of fixing, beside Codex round 1 on the
frozen candidate (`git diff -- README.md crates` against HEAD `8673252`,
SHA-256 `4ed1225e4b0559a76c2dcc89274ea999688d328c61d94eb31cf07123ce9004f6`).

## What was checked

The diff, the bead, RUN-2, `run()`, `Invocation::parse`, the cache key,
`Admission`, `main.rs`, and the pinned rustc's `ice_path_with_config`
(rustc_driver_impl/src/lib.rs:1397-1430) and the hook's writes (lib.rs:1487,
1561). `cargo test -p tide-admit --test invocation`: 14/14.

- The compile child always gets exactly `RUSTC_ICE=0`, so `ICE_PATH` is
  `None` and both of rustc's report writes skip; `-Zmetrics-dir` cannot
  override it. The parent forwards the child's stderr on failure.
- The test runs inside the candidate package and asserts the tree holds only
  the source; without the fix rustc writes `rustc-ice-*.txt` there.
- The clean-environment exception accepts only `RUSTC_ICE` set to `"0"`,
  what the driver sets itself.
- The test-only flag can only make the compile fail, failures are never
  cached, and it is in the cache key.

## Findings (none block)

1. `invocation.rs:579`: started with an empty environment, the driver skips
   the re-exec, so its own process runs without `RUSTC_ICE=0` while the ICE
   hook from `main.rs` is installed; a panic there would write a report.
   Theoretical (no candidate-triggerable panic found). Fix: re-exec unless
   the environment is exactly that one variable.
2. Out of scope: a crash by signal (stack overflow, double panic) may write a
   core file into the compile child's current directory where core dumps are
   enabled with a relative pattern; RUN-2's build sandbox (later work) owns
   it.

## Nits

- `const ICE: (&str, &str)` reads less clearly than two named consts.
- The flag's doc says tests show "where its crash report goes"; it goes
  nowhere.
- The test's doc promises less than it asserts.

## Resolution (author)

F1 fixed with Codex B1; the nits are applied (`ICE_VAR`/`ICE_OFF`, the
flag's doc, the test's doc). F2 is left to the OS sandbox.
