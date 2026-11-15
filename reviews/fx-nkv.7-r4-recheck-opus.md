Model: claude-opus-5-5 · Date: 2026-11-16

## Verdict

approve.

I reviewed `git diff b9da8ed 5f66208` (HEAD `5f66208`, single task commit on `16698e1`), the author's "## Opus re-check fix" section, and the r4 logs. I ran no build or test: the diff and logs settle every item, and another agent held the heavy slot.

## Nested `#[sim_test]` seed-start leak: fixed and complete

`evaluate_inner` now calls `crate::sim::runner::start(seed)` only when `config.simulation && !nested` (`crates/tide-std/src/test/runner.rs:247–249`). `nested` is the same `manifest::active()` flag that already gates `begin` and `finish`.

Nothing else emits `TIDE_SIM_SEED_START`. `start` is the only emitter, and `evaluate_inner` is its only caller. Outer evaluations are unchanged, including the determinism re-run through `evaluate()`, which is not nested once the first evaluation's `finish` has cleared the slot. So a sim-child parent's `last_seed` now sees only the caller's own seeds.

The changed lines add no new defect. The guard reuses an existing local, and both branches were already reachable.

## The regression exercises the leak

`nested_evaluation_stays_in_its_caller` now runs the built binary directly in sim-child mode (`__TIDE_SIM_CHILD=demo::caller --exact caller --nocapture`). That output is exactly the stdout a parent's seed tracker reads. The test asserts that this output carries only the caller's own starts. That is none for a `#[test]` caller, and `TIDE_SIM_SEED_START 0` twice for a `#[sim_test(seeds = 1)]` caller (the evaluation plus its determinism re-run). The test also asserts that the driver's `--tide-run-test` output carries no start record.

`r4-nested-before-final.log`, with the production sources at b9da8ed, fails exactly the three nested-sim scenarios:
- from a test: 1 start, expected none;
- from a simulated task: 2 starts, expected none;
- from a sim child: 4 starts, expected the caller's 2.

The three nested-property scenarios pass in that log, which is correct, since they never call `start`. `r4-replay-after.log` shows 11 of 11 `replay_` tests passing.

The full suites on 5f66208 are recorded:
- `r4-workspace-final.log`: 325 passed;
- `r4-conformance-final.log`: 361 PASS, 4 AWAITS CHECKER (FX0809, FX0808, FX0206, FX0205, as in r3), 0 FAIL.

## Doc note: nested simulation root's events

Done. README now says that a nested simulation root's events are not part of the caller's trace, and that its seed-start records stay internal. design/03 still names only choices and roots. That is a nit, not a blocker, since the README is the user-facing statement.

## Doc note: a `#[sim_test]` inside a simulated task

Done. README and design/03 now list, beside the const-test and test-`main` exclusions, that such a call fails with "a task runtime is already active". The author left `main.rs` and `--help` unchanged, because they list only driver-mode exclusions. That is reasonable.

DONE
