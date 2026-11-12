Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Candidate: `fx-rpwp` at `6ed1235` on `dde6c57` (`git diff dde6c57..6ed1235`). This is a read-only converge round for the code, and the docs review for the protocol.md edit. Under `trials/e1-checked/` I read only protocol.md, README.md, readiness.md, harness.py, harness_test.py and the five added files. I opened no task, suite, reference or `rust/` content.

## What holds

- **Protocol edits: exact, and nothing else changed.**
  - §10.1's Owner column: E1-P6 is `fx-byk.1`, `fx-byk.2`; E1-P12 is `fx-m72.1`–`.3`; E1-P13 is `fx-yg5.1`–`.5`; E1-P14 stays `fx-u65`.
  - §10.2(1) adds the same remap plus `fx-dsa` (library boundary), `fx-ecb` (I/O core) and `fx-rpwp` (gate machinery). That matches lucca's 2026-11-10 23:03 comment on fx-rpwp ("carry the remap … L marker fx-dsa, I/O core fx-ecb, gate machinery fx-rpwp").
  - §10.2(3) excludes FX0920, FX0921 and FX0516 and cites the 2026-11-09 owner decision on fx-ic2, as that decision (1) says.
  - No other protocol line changed, and `readiness.md` is unchanged (empty diff).
- **Research copy.** `research/24-e1-gate-machinery.md`, from line 3 on, is byte-identical to `tide-research-gate/report.md` (`cmp` reports no difference).
- **fx-m72.3's code is intact.** `test_command`, which holds the acceptance filter for `__tide_contract_tests` and the libtest log check, and `executable_tests` are not in the diff.
- **Order.** The tuple sort followed by `random.Random(20261031).shuffle` is pinned by hash in `harness_test.py:724`.
- **Budget ownership.** The harness owns the 20-run refusal (exit 3 and `budget.json`), and the broker is 73 lines and relays output verbatim.
- **Signals.** No numeric PID or PGID is signalled anywhere outside an unreaped child. The guard kills only its own group. Codex r3 is right on finding 4.
- **Prerequisites.** The coordinator refuses a non-dry run unless both arms' canaries, the probe and the pre-run store scan are keyed to the current machinery hashes, bwrap version and boot ID.
- **Canaries.** They have positive and negative controls in each of three shapes per arm, and they check tmpfs.

The defects below are in the fault and resume logic, which the per-fix Codex passes did not cover as a whole. Protocol §3 allows a rerun only for "a session that ends through no act of the writer", and a second fault fails the trial. The candidate departs from that in both directions:

- one operator interruption fails a trial;
- several writer-caused or normal endings get a free fresh writer draw;
- after a common PID reuse, resume stalls for good.

## Findings

### 1. One coordinator interruption fails a trial outright. **Blocking.**

**Evidence.**
- `coordinator.py:426-443`: `run_slot` goes straight on to attempt 2 after a `host_interruption`, including one caused by shutdown.
- `coordinator.py:488-490`: attempt 2's `host_interruption` placeholder is persisted before `prepare`.
- `coordinator.py:500-502`: only then does the launch wait see `cancelled` and raise "batch stopped before launch".
- `coordinator.py:475-483`: on resume that placeholder is replayed as a real second fault.

**Failure.** SIGTERM or Ctrl-C during attempt 1 (or during its grading) produces a slot record of [real interruption, placeholder] with no second writer ever launched. The next `run` marks the slot `complete`, `failed_infrastructure: true`, and both measures fail. The same happens to every in-flight slot when `--jobs` > 1 and any worker raises, including the designed "identity changed" stop at `:498`, which also fires after the placeholder is saved. Probe A reproduces this.

**Fix.**
- Do not start a further attempt once `cancelled` is set.
- Persist the attempt placeholder only once the guard has been granted, for example in `on_started`.
- On resume, treat an attempt with no `launch.json` or `process.json` as never started.
- Add a regression: interrupt attempt 1, resume, and expect exactly one recorded fault and one fresh rerun.

### 2. Writer acts and normal budget endings are classified as infrastructure faults, so they get a free rerun. **Blocking.**

**(a) A denied read outside the writer directory.**
- Evidence: `coordinator.py:117-120` with `:357` and `FAULTS` (`:24`).
- Failure: if a writer asks to Read a path outside its directory and the CLI's `--restricted` denies it, the post-session audit raises `log_audit`, and the whole trial is rerun with a fresh writer. Probe D reproduces this. A Rust writer looking up crate source under `~/.cargo` is a plausible case, so this can be arm-asymmetric.
- A *non-denied* outside access is the opposite case: confinement has failed for every session, and §10.2(5) no longer holds. The same per-slot rerun lets the run continue.
- Fix:
  - record denied outside requests as information, counted per arm, using the structured denial that `probe_denied` already parses;
  - treat any outside path that was not denied as a confinement breach that stops the whole run;
  - leave decision 9's "tool outside the set" rule as decided.

**(b) A kill at the budget truncates the last log line.**
- Evidence: `coordinator.py:348-357`.
- Failure: the coordinator kills the writer group at the moment the broker logs the 20th call's completion, which is exactly when the CLI is emitting that call's tool-result line (up to the 1 MB cap). The same applies at the time limit. An unterminated last line makes `jsonl()` raise, and a session that legitimately hit `run_limit` or `time_limit` becomes a `log_audit` fault and is rerun. Probe C reproduces this.
- Fix: after a kill the coordinator started, drop an unterminated final line and record `truncated_tail`.

**(c) The grader's outer 600 s guard can fire on a slow solution.**
- Evidence: `coordinator.py:406-409`, `:428-431`.
- Failure: a final grade has no aggregate deadline.
  - For a Tide binary task, worst-case time is 2×120 s for checks (`harness.py:263-272`, no shared deadline), plus 120 s for writer tests, plus a 120 s build, plus 10 s per binary case with no aggregate bound (`harness.py:387-396`). A solution that hangs on every case can pass 600 s.
  - When it does, the timeout becomes `infrastructure`, then `harness_setup`, and the writer is rerun. A failing solution gets a second chance.
- Fix: give `harness.py grade` its own aggregate deadline, below the outer guard, so a slow solution fails as a solution. Keep the outer guard for real hangs only.

**(d) Non-blocking part, to verify in the live probe.**
- Evidence: `writer_guard.py:29-30` and `coordinator.py:370-376`.
- Failure: the guard SIGKILLs its group 0.1 s after the CLI exits, but a `done` ending requires the broker's `stop` lifecycle row. Under load, a normal finish can become `broker_crash`. The dry runs all ended at `run_limit`, which skips this check, so a real CLI has never exercised this path.
- Fix: have the guard wait, bounded, for its group to empty before the kill.

### 3. A grading fault or an interruption during grading throws away a finished session and draws a new writer. **Blocking.**

**Evidence.**
- `coordinator.py:428-431`: a grading infrastructure fault becomes a `harness_setup` writer fault.
- `coordinator.py:520-523`.
- `coordinator.py:488-490` and `:532`: the slot keeps the `host_interruption` placeholder, and `trial.json` is written only after grading.

**Failure.** The writer's session ended by its own act. Its final snapshot is frozen and hashed, and §7.4 says grading is reproducible from stored snapshots. Even so:
- a grader setup failure or timeout reruns the writer (probe B: 000-1 is discarded and 000-2 passes);
- a coordinator stop during grading resumes as a host interruption, so a valid trial is replaced. Slow, failing solutions spend longest in grading, so they are the most exposed.

This is the r1 finding 3 fix: the intent (infrastructure must not count as a solution failure) is right, but the remedy re-samples the writer.

**Fix.**
- Persist the writer's result (`end_reason`, final hash) before grading.
- On a grading infrastructure fault, or on resuming a session that has `final/`, re-grade the frozen snapshot with bounded retries.
- If grading still cannot complete, stop the run for the operator. Never rerun the writer for a grading fault.

### 4. Recovery refuses forever once a saved PID has been reused. **Blocking.**

**Evidence.** `coordinator.py:183-200` and `:185-186`.

**Failure.**
- If the old guard's PID now belongs to another process, `wait_quiescent` raises "identity changed; refusing recovery". `pending[:count]` reaches that slot first on every later `run`, so the gate cannot resume without hand-editing state.
- PID reuse is likely here: `pid_max` is 4,194,304, and live PIDs were at 4,170,248 during this review, so PIDs are wrapping now. Resumes happen hours later, in the next cleared slot.
- Linux does not free a PID number while any task, zombie included, uses it as a PGID. Because the saved group equals the saved PID, a vanished or reused leader PID *proves* the old group is empty. That is the provable quiescence the 2026-11-12 coordinator decision asks for.
- Separately, `launch.json` without `process.json` also refuses forever. In that state the guard never got its grant, so it exits on EOF or parent death.

Probe E reproduces the refusal in both reuse shapes.

**Fix.**
- Treat a missing PID, or a start-time mismatch, as quiescent and go on to the existing command-lock wait.
- Refuse only while a matching live group persists past the wait.
- Add a regression with a reused PID.

### 5. Gate-operation follow-ups for fx-ic2. **Non-blocking.**

**(a) The Nix store bind.** The whole of `/nix/store` is bound, because the toolchain's loader lives there (`r1-no-store.log`). The size-then-SHA scan (`gate_checks.py:86-109`) catches only byte-identical copies. An older-revision copy of trial files, such as a flake `-source` taken at an earlier commit, would be readable and undetected.
- Prefer binding only the toolchain's store closure.
- At least, also flag store paths containing `trials/e1-checked`.

**(b) The identity check and build parallelism.**
- `cheap()` compares HEAD (`gate_checks.py:55`), so any commit on main stops the run. Run the gate from a dedicated worktree pinned at the recorded revision.
- Writer runs get `CARGO_BUILD_JOBS=4` (`coordinator.py:278`), while grading inherits the operator's environment or the default 6 (`harness.py:181`). Neither value is in the identity record. Pin both and record them.

**(c) No memory bound on jails.** They have no memory limit, and `/tmp` is now a RAM-backed tmpfs of default size (`harness.py:208`). This machine is shared with Eon. Add bwrap `--size` and a cgroup or `ulimit` memory cap.

### 6. The 40-minute value is held twice. **Nit.**

`coordinator.py:320` uses the coordinator's own `seconds` (2400 from the state), and `harness.py:455` hard-codes 2400. The coordinator could read the limit the harness enforces from `session.json`, so the rule has one owner.

### 7. A canary variable that nothing checks. **Nit.**

The `binary` canary shape sets `E1_CASE_CONTROL` (`gate_checks.py:176`), but the probe never checks it. Drop it, or assert it inside the jail.

## Probes run

All were pure Python, mock-only or used synthetic processes, with a timeout, stdin closed and `PYTHONDONTWRITEBYTECODE=1`. Scripts are in `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-rpwp/564ab858-f0fa-4873-8e44-7fb84c6ed550/scratchpad/`. No bwrap, canary, dry run, compilation or `heavy.sh` was used. The worktree was not modified; its pre-existing `__pycache__` dates from 09:03 -03.

**Docs and copies.**
- `git diff dde6c57..6ed1235 -- protocol.md readiness.md`: only the three edit sites in protocol.md; readiness.md is empty.
- `tail -n +3 research/24-e1-gate-machinery.md | cmp - …/tide-research-gate/report.md`: IDENTICAL.

**Tests.**
- `CARGO_BUILD_JOBS=4 timeout 180 nice -n 19 python3 harness_test.py GateTests FixRound1Tests FixRound2Tests </dev/null`: 30 tests, OK. These are the non-compiling gate tests. I did not rerun the compiling `HarnessTests`.

**Probe A (`probe_interrupt.py`).**
- Setup: `batch()` with a mocked `run_writer` that sets `cancelled`, as the SIGTERM path does, then a resume with a healthy writer.
- Result: run 1 raised "batch stopped before launch", leaving attempts [interruption, placeholder] with no `launch.json` for 000-2. The resume launched no writer (calls `['000-1']`). The final slot was `complete: True, failed_infrastructure: True, final_pass: False`.

**Probe B (`probe_grade.py`).**
- Setup: the first `freeze_and_grade` returns an infrastructure final.
- Result: writers launched `['000-1', '000-2']`, and the slot finished `reruns: 1, final_pass: True`.

**Probe C (`probe_trunc.py` with `fake_cli.py`).**
- Setup: a real `run_writer` and real `writer_guard.py` (the pidfd path only), driving a fake CLI that emits init, starts a 200 KB tool-result line, then publishes `budget.json` and the broker acknowledgement.
- Result: `end_reason: log_audit, fault: log_audit` ("Unterminated string").

**Probe D.**
- Setup: `audit()` on one Read of an outside path, denied with the exact structured `permission_denied` frame.
- Result: `['path outside writer directory']`, and `log_audit` is in `FAULTS`.

**Probe E (`probe_reuse.py`).**
- Setup: `reclaim_writer()` with a saved identity whose PID is now an unrelated new group leader (`sleep`), then with one whose PID is an ordinary process.
- Result: both raised "previous writer process identity changed; refusing recovery".

**Host.** `/proc/sys/kernel/pid_max` is 4194304; the highest live PID was 4170248.

Findings 1–4 change `coordinator.py`, which this change adds, and finding 2(c) also changes the grade path in `harness.py`. None touches the v0 core. They block because their fixes change text this change edits. The finder (Opus) re-checks only those fixes. Findings 2(d) and 5 go on fx-ic2 as gate-operation comments, unless the author folds 2(d) into the finding 2 fix. DONE
