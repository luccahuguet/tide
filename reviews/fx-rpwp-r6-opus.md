Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Scope: I re-checked only N1 and N2 from my round 5, on `git diff e7348f6 6e6b9ee` (HEAD `6e6b9ee`, one commit on `dde6c57`, clean tree) and report.md's "Fix round 4". The same held-out and read-only rules held. I ran no bwrap, compiling run or `heavy.sh`.

## Findings

**N1: one post-session classifier. Fixed.**
- `classify_session` (`coordinator.py:294-339`) now holds all of the post-session classification:
  - the run-20 `budget.json` infrastructure flag and the budget reason;
  - the log read, with the coordinator-kill tail rule;
  - the init assertion and the audit;
  - the confinement-breach stop;
  - CLI exit and result classification;
  - the broker lifecycle check, broker fault rows and broker exit 2.
- Live `run_writer` calls it at `:420`, adding only wall time and drift. The frozen resume path calls it at `:593` whenever no `writer-result.json` exists. No second copy of the logic remains.
- The frozen path's ad-hoc audit and the invented `frozen_snapshot` reason are gone.
- A frozen session classified as a fault returns from `finish` ungraded and takes the ordinary single rerun.

**N2: the resume path refuses on a hash mismatch. Fixed.**
- `finish` (`coordinator.py:566-569`) recomputes the frozen hash and compares it with the saved `final_sha256` whenever `writer-result.json` exists.
- On a mismatch it raises an operator stop before overwriting the result, writing `trial.json` or starting a grader.

**New defects.** None blocking. One behaviour change, which I do not count as a defect:
- `run_writer` now takes `budget.json`'s reason whenever the file exists, not only once the coordinator saw the broker's acknowledgement.
- So a writer that ends on its own after its 20th run is classified as `run_limit`, not checked for `cli_error`.
- Its final was captured at run 20 and the budget ended the session, so this is consistent with protocol §4.2.
- The `killed` flag still comes from the live reason, so tail-dropping still needs the coordinator's own kill.

## Probes and checks

These were pure Python with a timeout, stdin closed and `PYTHONDONTWRITEBYTECODE=1`. The worktree is unmodified.

**`r6_probe.py`**, in the scratchpad `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-rpwp/564ab858-f0fa-4873-8e44-7fb84c6ed550/scratchpad/`, independent of the author's tests. Each case is a synthetic frozen session at run 20 with `final/` and a `host_interruption` placeholder, resumed through the real `batch`. Only `run_writer`, `freeze_and_grade` and `prepare` are mocked as tripwires. Results:
- **Budget infrastructure flag plus broker exit 2:** attempt 1 is reclassified `harness_setup`. It is not graded, and the run goes on to a fresh attempt 2 (the `prepare` tripwire fired).
- **Healthy session:** attempt 1 ends `run_limit` with no fault. It is graded once, with no writer launched.
- **Saved hash differs from the final:** the run stops with "final snapshot hash changed". Nothing is graded, and the saved result's bytes are unchanged.

**Tests.** `CARGO_BUILD_JOBS=4 timeout 180 nice -n 19 python3 harness_test.py GateTests FixRound1Tests FixRound2Tests FixRound3Tests </dev/null`: 46 tests, OK.

**Author's logs**, read but not rerun:
- `r4-harness-test.log`: 69 tests, OK.
- `r4-canary-rust.log` and `r4-canary-tide.log`: PASS.
- `r4-dry-clock.log`: PASS.
- The four-slot dry run, per the report: four slots at `run_limit`, the 21st call refused with exit 3, both grades, no rerun.

N1 and N2 are resolved, and the fixes add no defect. DONE
