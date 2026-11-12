Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Scope: this re-checks only my round-4 findings 1–4, 2(d) and nits 6–7, on `git diff 6ed1235 e7348f6` (HEAD `e7348f6`, one commit on `dde6c57`, clean tree) and report.md's "Fix round 3" section. I add only defects the fixes introduced. The same held-out and read-only rules held: I opened only the allowed trial files, and ran no bwrap, compiling run or `heavy.sh`.

## Per finding

**1. One interruption failed a trial. Fixed.**
- The placeholder is now saved only in `on_started`, after the grant (`coordinator.py:640-649`). The grant itself happens under `ACTIVE_LOCK` with a cancellation check (`:322-326`), and the callback sits inside the guard cleanup (`:342-344`).
- `run_slot` stops before a further attempt, or after a fault, once the run is cancelled (`:497-514`). `attempt_runner` also refuses after cancellation (`:572`).
- On resume, an attempt without both `launch.json` and `process.json` counts as never started. Its partial preparation moves to `unstarted/` (`:551-552`, `:609-614`).
- Probe A now gives: run 1 raises `RunCancelled` with attempts `[host_interruption]`; the resume launches 000-2; the slot ends `reruns: 1, failed_infrastructure: False`. That is exactly one fault and one fresh writer.

**2(a). A denied outside read caused a rerun. Fixed.**
- `permission_denied` is shared with the strict probe check (`:74-96`).
- A denied outside request is now recorded in `denied_outside` and counted per arm (`:139-142`, `:655`).
- An outside access that was not denied raises `ConfinementBreach` and persists a marker (`:384-386`). Every later `batch` refuses while that marker exists (`:544-545`), and `main` exits 2.
- Probe D: a denied read gives `passed: True, errors: []`, one denied entry and no breach. The same read without a denial gives `passed: False` with one breach.

**2(b). A kill could truncate the last log line. Fixed.**
- `writer_events` drops one unterminated tail, but only after a kill the coordinator made at the run or time limit (`:45-49`, `:368`, `:376`). It records `truncated_tail`. A malformed log without such a kill still faults.
- Probe C (fake CLI cut off mid-line at the 20th run) now gives `end_reason: run_limit, fault: None`.

**2(c). The outer 600 s grader guard could fire on a slow solution. Fixed.**
- `grade()` now sets a shared 480 s deadline (`harness.py:510`). Every `run()` clips its timeout to that deadline, including binary cases and checks. `executable_tests` keeps `min(deadline, now+120)`.
- Running out of that budget adds a `grade_timeout` row with code 124 and no infrastructure flag (`harness.py:528-532`), so the grade exits 1, a solution failure. The 600 s outer guard now covers real hangs only.

**2(d). The guard's 0.1 s wait could turn a normal finish into a broker crash. Fixed.**
- `writer_guard.py:30-40` now waits up to 3 s for the members of its own group (other than zombies and itself) before killing that same group. No new signal target was added.

**3. A grading fault re-sampled the writer. Fixed.**
- `finish()` freezes the final, hashes it and persists `writer-result.json` and `trial.json` before grading (`:554-561`).
- `grade_frozen` grades the same frozen snapshot at most three times in all (`:466-494`). The history persists across resumes, and an interrupted grade counts as one of the three tries.
- When the tries are exhausted, the run stops for the operator with a `RuntimeError`.
- `run_slot` refuses to turn a grading fault into a writer fault (`:504-505`). A resumed session with `final/` goes to grading, never to a new writer (`:576-594`).
- Probe B now gives writers `['000-1']`, graded `['000-1', '000-1']`, `reruns: 0`.

**4. Recovery refused forever after PID reuse. Fixed.**
- `wait_quiescent` (`:208-226`) treats a missing PID, or one whose start time, group or boot ID differs, as quiescent. It then goes on to the command-lock wait.
- The "launch without process identity" refusal is gone. Recovery still sends no signal.
- Probe E: both reuse shapes now recover. A new control, a live matching guard group, is still refused after the bounded wait ("previous writer group is still alive").

**6. The session limit was held twice. Fixed.** `harness.SESSION_SECONDS` and `session_limit()` (`harness.py:28`, `:441-442`) own the limit. `launch_clock` writes `h.session_limit` (`coordinator.py:178-185`), and `run_writer` uses the value in `session.json` (`:323`). Shortening is still dry-only (`:738`).

**7. The canary set an unchecked variable. Fixed.** `E1_CASE_CONTROL` is removed (`gate_checks.py:174`). The final canaries pass without it.

## Defects the fixes introduced (non-blocking)

**N1. The frozen-snapshot resume path classifies less than `run_writer` does (`coordinator.py:576-594`).**
- Trigger: the coordinator dies after the harness captured `final/` at run 20, but before `writer-result.json` is written. That is a window of about a second.
- Effect: the resumed slot is graded as `frozen_snapshot` after only the log audit. Two `run_writer` checks are skipped: the run-20 `budget.json` infrastructure flag and broker exit 2 (`harness_setup`), and the init and broker-lifecycle checks.
- Risk: a session whose 20th run hit a jail setup failure could be scored. That needs a crash inside that window, so the chance is small.
- Fix: factor `run_writer`'s post-session classification into one function and call it from both paths.

**N2. The resumed frozen session's hash is not compared (`:557` with `:583`).**
- On resume, `finish()` recomputes `final_sha256` and overwrites the stored value without comparing.
- Fix: refuse if the new hash differs from `writer-result.json`'s hash.

**N3. Operator cancellations use up grading tries (`:477-479`, `:492`).**
- A Ctrl-C or crash during grading counts as one of the three tries. Two operator stops plus one real fault therefore stop the run for the operator.
- This affects liveness only, never scoring. A cancellation could instead leave its try uncounted.

## Probes and checks run

All were pure Python with a timeout, stdin closed and `PYTHONDONTWRITEBYTECODE=1`, from `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-rpwp/564ab858-f0fa-4873-8e44-7fb84c6ed550/scratchpad/`. The worktree is unmodified.

- `r5_probes.py` re-runs probes A, B, D and E against the round-3 interfaces. A and B are mocked writers that write `launch.json`/`process.json` the way a granted writer does, with `freeze_final` mocked and B grading through the real `grade_frozen`. E adds a live-group control. Results are as listed above.
- `probe_trunc.py` (probe C, unchanged) with a real `run_writer` and `writer_guard.py`: `run_limit`, no fault.
- `CARGO_BUILD_JOBS=4 timeout 180 nice -n 19 python3 harness_test.py GateTests FixRound1Tests FixRound2Tests FixRound3Tests </dev/null`: 44 tests, OK.
- I read the author's final logs but did not rerun them:
  - `r3-final-harness-test.log`: 67 tests, OK.
  - Both `r3-final-canary-*.log`: PASS.
  - `r3-final-dry-run.log`: four slots at `run_limit`, 20 runs, both grades, no infrastructure fault.
  - `r3-final-dry-clock.log`: PASS.

Findings 1–4, 2(d), 6 and 7 are resolved. N1–N3 are optional hardening and do not block landing. DONE
