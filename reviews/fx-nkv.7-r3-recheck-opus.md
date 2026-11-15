Model: claude-opus-5-5 · Date: 2026-11-15

## Verdict

request_changes. Both r3 blocking findings are fixed. The taken non-blocking items are correct, and the TCP rebuttal stands. One new defect in the changed lines blocks: a nested `#[sim_test]` now prints its internal seed-start records into the caller's output. The fix is one line.

Re-checked `git diff c06d86a b9da8ed` (HEAD `b9da8ed`, single task commit on `16698e1`) and the "## Opus r3 fixes" section of the author's report.

My own run, on b9da8ed through `heavy.sh env CARGO_BUILD_JOBS=4 timeout 1500 …`, at 1-minute load 2.9:
- `cargo test -p tide-admit --test build replay_ -- --test-threads=4`: 11 passed, including all 9 regressions;
- two probes with the built driver, described below.

The script is `/tmp/claude-1000/-home-lucca-pjs-lab-tide/e3e607df-e147-4ed5-b3f9-ae12e97442dc/scratchpad/probes/run2.sh` and its log is `…/scratchpad/run2.log`.

## Regressions: fail before, pass after

Confirmed from the logs and my run:
- `r3-final-tests-before.log` shows exactly the 9 named regressions FAILED, 0 passed. The production sources were restored to c06d86a bytes for that run.
- `r3-final-tests-after.log` and my run show all 11 `replay_` tests passing.
- `r3-workspace-final.log` shows the 9 regressions passing on the frozen tree.

The 9 tests are strong. Each nested scenario:
- runs the binary directly;
- records exactly one manifest through the driver, with no nested choices and exact caller roots;
- replays twice with equal roots, trace and outcome.

## Blocking 1 — nested property and sim evaluations: fixed

How the fix works:
- `evaluate_inner` detects `active()` and suspends the caller's recording with an unwind-safe guard (`Suspended`, restored in `Drop`).
- It skips `begin`/`finish`, so nested runs write no manifest and add no choices or roots.
- `run_config` ignores driver seed and replay controls when nested.
- `sim::__run` dispatches in-process when nested.
- `begin` checks `!active()` before reading the context or reserving an artifact directory.

My earlier probe (`#[test] fn caller() { prop(); }`) now passes. The guard is correct for nested-of-nested calls (the `SUSPENDED` flag) and on simulated task threads (`is_simulating`).

Two consequences are acceptable but undocumented. Non-blocking: say them in the README.
- A nested sim root's events are not in the caller's trace. They run with fixed case seeds against an identity-checked source, so this does not weaken replay. The README sentence names only choices and roots.
- A `#[sim_test]` called from inside a simulated task now fails with "a task runtime is already active". The parent ran it in a separate child process. This is a deliberate narrowing, and the regression locks it in. Name it beside the const-test and `main` exclusions.

## New blocking — nested sims print internal seed-start records

The cause is at `crates/tide-std/src/test/runner.rs:247–249`: `if config.simulation { crate::sim::runner::start(seed); }`. It still runs when `nested` is true.

My probe, under the driver:
```rust
#[sim_test(seeds = 2)] fn simulation(sim: std::sim::SimRoot) { sim.tasks.checkpoint().unwrap(); }
#[test] fn caller() { simulation(); }
```
`--tide-run-test caller` printed `TIDE_SIM_SEED_START 0` and `TIDE_SIM_SEED_START 1` into the caller's output.

This breaks the documented rule that "seed-start records stay internal" (README, design/03 §9). When the caller is itself a `#[sim_test]` child (the `nested_*_from_sim_child` scenarios), the parent's `last_seed` tracking (`sim/runner.rs:144`) reads the nested seeds as the caller's. A later timeout, signal or missing summary would then name the wrong "last started seed".

**Fix:** `if config.simulation && !nested`. Extend `nested_evaluation_stays_in_its_caller` to assert that no `TIDE_SIM_SEED_START` appears in the caller's output; the sim-child scenarios already give it that output to check.

## Blocking 2 — remapping: fixed, and the TCP rebuttal is accepted

How the fix works:
- Remapping now applies only when `self.link && (self.test || self.tested_library)`.
- It uses `--remap-path-scope=macro,object`, the spelling the pinned nightly accepts.
- The flag also enters the normalized semantic options, so identity stays consistent.

`replay_remapping_keeps_diagnostics_and_non_test_locations_real` covers:
- real paths in diagnostics for metadata, bin-link and test-link builds;
- real panic locations in a non-test binary;
- logical `tide-source/` in test panic and task-0 trace origins.

The main identity test's relocated replay still reproduces, so the trace spans stay normalized.

The TCP rebuttal is correct. That fixture is a linked `--test` harness, where logical paths are intended. My finding concerned bins and compile diagnostics.

Keeping the `read_file` reverse-mapping branch is justified by the logged FX0520/ICE when it is removed. The branch still returns only empty text and never reads a host file. Docs are updated in README, `main.rs` and design/03.

## Non-blocking taken: replay hints only on failures — fixed

`finish` now takes `failed = matches!(outcome, Outcome::Failed(_))` instead of matching on the signature text. A passing ordinary test and a discarded or vacuous case print no `replay:` line, and the regression test checks both. Failures still print one, as the main identity test shows.

## Non-blocking taken: refusal preserves outputs — fixed

- `clear_outputs` returns early in replay mode.
- In the cache path, replay calls `run_test_process(…, Some(&entry))` before the publish loop.
- In the paired path, replay runs from staging before the renames.

The receipt is read from the same private directory, the executable is copied and its digest checked against that receipt, and the build is compared with the current identity, so the execution boundary is unchanged. Replay needs full admission, so the cache path is always the cacheable one, and `entry` persists. The regression checks byte-identical executable, receipt and interface after compiler/configuration refusals on both paths, and after a corrupted cache executable is refused.

Nit: replay no longer echoes the cached compile stdout and stderr, so warnings are not shown.

## Non-blocking taken: doc gaps — fixed

- design/03 now links research/32.
- research/32 has a decision-status header and the corrected shrunk-tape falsifier.
- The stale fs "names are provisional" note is replaced with `trace/1`.
- Const tests and a test-decorated `main` are documented in README, design/03, `main.rs` and `--help`.

## Deferred items

These are acceptable as deferred:
- `target/sim` retention;
- per-draw manifest rewrite;
- repeated hashing;
- the once-per-process receipt guard;
- a trusted driver verdict line;
- symlink path normalization.

None of them affects correctness of what was built.

DONE
