Model: claude-opus-5-5 · Date: 2026-11-11

# fx-nkv.6 — round 2 Opus review (converge, report-only)

Candidate: `419b07d` on `32c5bc7`, worktree `.worktrees/fx-nkv.6`. `git diff 32c5bc7 419b07d | sha256sum` = `134504951f2bb13e31cd332c4a6ffcf4f806c2fbdcb54fce4ee1a1c0088b6213` (matches the brief). I read the Codex round 1 report and checked its claims myself. `trials/e1-checked/` was not opened.

## Verdict: approve_with_alternatives

No finding blocks. The runtime mechanism holds up under the cases I traced:
- task 0 and baton lifetime: `Simulation::new`/`Drop`, `Enter::drop`'s ended-context filter, and the `_main` field drop order inside `Simulation`/`SimRoot`;
- unwinding: dropping the root in a live scope, a body panic during `evaluate`, and drop during an unwind (no assert, no exit event, the scope still joins under its captured context);
- cancellation and echo paths from nkv.5: unchanged, and task 0's exit goes only through `Scheduler::finish`, never through `Runtime::finish`;
- sink capture: formatting happens before the lock. `Shared::with` callbacks run without the runtime lock, so a sink cannot re-enter it;
- `Fs` gating: every `pub fn` goes through `check`, including `dir`, and `read_only` keeps the identity;
- `Rand` rejection;
- the macro's lowering.

The design matches the 2026-11-10 decisions 1–7. The witness is a genuine split read-modify-write that the scheduler finds. Docs describe the partial surface truthfully. I found no leftovers: the `oracle_macros` dependency and lockfile entry are gone, and the conformance stubs are trimmed to the remaining `node`/`run_for`/`World`/`Faults` methods.

Probe (one, read-only, gates passed: load 4.5, 15 GB available, no quiet window; `nice -n 19`, stdin closed, 120 s timeout). I ran the existing driver-built `target/tmp/build/simulation-runner/out/demo-test --exact simulation::fixed_updates` without `--nocapture`. It exited 0 and printed 54 lines to the terminal: the child's `running 1 test`, 20 `TIDE_SIM_SEED_START n` lines, and a second `test result: ok. 1 passed; ... 6 filtered out` line ahead of the parent's own. This settles finding 1.

## Findings

1. **Non-blocking: a passing `#[sim_test]` bypasses libtest's output capture and prints a nested libtest run.**
   Evidence: `crates/tide-std/src/sim/runner.rs:23-25` forwards the child's stdout and stderr with `io::stdout().lock().write_all` / `io::stderr().lock().write_all`. libtest only captures through the `print!` family, so this output always reaches the terminal. The probe above shows a passing run emitting the child's `running 1 test`, every seed-start line, `determinism checked`, and a duplicate `test result:` summary. With the default 100 seeds that is over 200 lines per passing test, interleaved across parallel tests. Anything that counts libtest summary lines would also double-count, which matters for fx-ic2's graded E1 runs. Tide's own `__sink` also writes to host streams directly, so user `println!` was never captured. The forwarded text here is harness noise, though, not user output.
   Fix: on success, forward nothing (or only the child's one-line `simulation …: N passed` summary). On failure, forward everything. Alternatively, forward through real std's `print!`/`eprint!` so libtest captures it and shows it under the failing test's `---- stdout ----` section. Either way the seed-start parse stays internal to the reader thread.

2. **Non-blocking: the 300 s wall-clock backstop covers the whole seed range in one child, not each seed.**
   Evidence: `runner.rs:11` (`TIMEOUT`), `runner.rs:84-97` (one deadline from `start`). The same constant is restated in `crates/tide-std/src/lib.rs` docs, `README.md`, and `design/03-authority-concurrency.md:1786-1797`. A correct test with a large `seeds = N` (03 §9.2 expects `tide sim --test` to run more), or a slow debug-build body, fails as a timeout. The failure is then attributed to whichever seed happened to be running, which is innocent. Decision 4 asks that abort or timeout be "attributed to the last started seed". A per-run deadline makes that attribution arbitrary for timeouts.
   Fix: reset the deadline each time the reader sees a `TIDE_SIM_SEED_START` line, making it a per-seed backstop. The reader already parses those lines, so this is a few lines in `child`. Restate it once as "300 s per seed". Optionally drop the numeric constant from design/03 §9.2, since it is an implementation budget and README/lib.rs already own it.

3. **Nit: on an ordinary seed failure, the parent's panic message names the wrong seed.**
   Evidence: `runner.rs:26-38`. When seeds panic normally, the child exits 101 after finishing the range. The parent then reports "child failed or did not run the test (last started seed: 19)". Seed 19 is the last seed in the range, not a failing one. The real failing seeds appear only in the out-of-band forwarded stderr (see finding 1).
   Fix: name the last started seed only for abnormal ends: a signal (`status.code().is_none()`), a timeout, or no `simulation …: N passed; M failed` summary line. Otherwise say "see failing seeds above" or carry the child's `simulation failure seed=` lines into the panic message.

4. **Nit: a failing plain `#[test]` that uses `Root::sim` loses all its sink output.**
   Evidence: `crates/tide-std/src/task.rs:513-525` diverts every `out`/`err` record into the trace. Only the `#[sim_test]` runner prints a failing seed's trace (`runner.rs:202`). In a plain test, the debugging `println!`s of a failing in-process simulation are never shown. This follows decision 6, so it is not a defect. Note it, or consider having `Simulation::drop` print the sink records to stderr when `thread::panicking()`.

5. **Nit: one wrapper layer too many.**
   Evidence: `task.rs:474-485`. `Simulation::{elapsed, trace, records}` only delegate to `Tasks::sim_*`, which `evaluate` already calls directly (`runner.rs:129`).
   Fix: have `SimRoot` call `self.tasks.sim_*()` (or `self.simulation.tasks().sim_*()`) and delete the three pass-throughs.

6. **Nit, for fx-nkv.7: under `#[sim_test]`, task 0's spawn and exit span points into the runner, not the test.**
   Evidence: `Root::sim` is `#[track_caller]` and is called at `runner.rs:117`. Task 0's `spawn_span`, and so its `exit` event's span, therefore name `crates/tide-std/src/sim/runner.rs:117` instead of the user's test function. Field names and spans are provisional until fx-nkv.7, so record this there instead of changing it now. A `#[track_caller]` path from the macro-generated call, or an explicit span argument to `__run`, would fix it.

## Not checked

I did not rerun the full suites. The implementation proof (workspace 149 passed; conformance 220 PASS / 24 AWAITS / 0 FAIL at 4bb58e1, range-diff unchanged onto 419b07d) is taken from the report. I did not re-probe Codex's `should_panic` ordering result. I made no repository, Git, or `br` writes.

DONE
