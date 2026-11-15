# fx-ic2.11 — Opus review, round 4

Model: Opus (CLI `--model opus`). Date: 2026-11-15. Read-only.
Candidate Git tree: 7477a0ea4aa3bc193975e80bf76d7681e4c55d25

**Verdict: request_changes.** Confidence about 0.8. Two blocking findings, both small text-only fixes in `gate-report.md`. The script logic is sound for the fixed sample. After these two fixes I would approve.

Scope of this review: read-only, on frozen tree `7477a0e`, current files identical to `candidate.diff`. I ran nothing. As instructed, the pending sampled regrade is not treated as a defect, and the measures reproduction (already byte-for-byte) was taken as given. I did not recompute the per-trial aggregates in the informational table (medians, token sums), since that would need computation over `per-trial.json`.

## What I checked and found correct

- **Headline figures:** F, P, I, the all-60 means, the §8 inequalities, the bootstrap intervals and widths, and the 17 unavailable fields all match `gate-results/measures.json`. The run counts add up: 88 + 61 = 149 snapshots, 88 − 86 = 2 and 61 − 59 = 2 failed-trial runs, 126 and 99 for the all-60 sums.
- **Slot labels:** every sampled slot (12/12), every classification slot and every concurrency/integrity slot matches `state.json` order (index i is line 3+5i).
- **Evidence citations:** the C3 turbofish lines (032:111, 044:30, 068:22, 116:22), 033's line 13 `impl From<SrvError> for io::Error`, the slot 032/027 citations in `grader-reviews.md`, the token-defect numbers (419/522/24 against 2072/3112/3443 in `slot-00N.json` and the derived `writer.jsonl`) and the comment facts (`147eb9a`, 361 PASS, 72.2/19.7 min, 05:15–06:07, 120/120, the owner's quote) all check out.
- **Regrade logic against the harness at R:**
  - `expected_exit` reproduces `harness.py:748`. The `--first` / `--compile-only` flags mirror `coordinator.py:460`.
  - `signature` keeps the deterministic fields (`code`, `stage`, `observed_exit`, `test_log` such as `"ok case1\n"`, the expected/observed test lists, `flags`) and drops the noisy ones (`command` with temporary grade paths, memory/CPU readings, compiler prose).
  - Expected grades are read from `records`, not from the copy being overwritten.
  - Grade work goes to `G/target/e1-checked`, which is ignored, so G stays clean.
  - The jail conditions (tmpfs `/tmp`, CPUWeight, jobs=4) come from `harness.run`. Calling the harness without `writer_guard.py` only drops process-group cleanup.
- **Preflight window:** the archived `preflight.sh` does block until 16:30Z (the 15:00Z standing slot's `over`), and `MIN=14` with a 16:45 grant gives exactly the stated 16:30:00–16:31:00 start interval.
- **Canary/probe freshness:** `probe.json` and both canaries are in the archive, and the current boot ID (`b5a8bafb…`) equals the gate's, so preflight step 5 should pass today.

## Blocking findings

**B1. The "exact" regrade command cannot pass in the recorded booking** (`gate-report.md:488-517`).
- The report (line 469) and the bead's 14:28Z comment say the 16:00–16:45Z window is a heavy.sh quiet-hold exemption, with quiet-until at 16:45Z.
- At 16:30 `preflight.sh:27-33` then requires an exported `HEAVY_QUIET_EXEMPT=1` and `E1_GRANT_UNTIL` equal to quiet-until. Otherwise it fails with "heavy.sh quiet hold". `heavy.sh:77` would also hold the run.
- The command checks and exports only `E1_BOOKED_END_EPOCH` and `QUIET_LOG`. Lines 514-517 leave the other two to the coordinator, but if they are set without `export`, `preflight.sh` and `heavy.sh` never see them.
- With a one-minute start interval, that wastes the booked window. It fails safe, but the acceptance check does not happen.
- **Smallest fix:** in the command block add `: "${E1_GRANT_UNTIL:?coordinator supplies the eon-overseer grant end}"` and `HEAVY_QUIET_EXEMPT=1`, export both alongside the existing exports, and reword 514-517 to say this booking needs them.
- **Optional simplification:** set `E1_BOOKED_END_EPOCH=$E1_GRANT_UNTIL`, so one value owns the booked end. They are the same instant for this booking.
- **Status:** text-only (markdown). Because it changes a command the coordinator will run, I'd like a quick finder re-check of that block only.

**B2. `REPORT_ROOT` is hard-coded to the bead worktree** (`gate-report.md:419` and `:496`).
- `.worktrees/fx-ic2.11` is removed after landing (AP-GIT-001), so the published commands would `cd` into a path that no longer exists.
- This also contradicts line 413, "From a checkout containing this report", and AP-CONTRACT-001's rule that docs match current paths.
- **Smallest fix:** `REPORT_ROOT=$(git rev-parse --show-toplevel)` in both blocks (computed before `cd "$G"` in the regrade block). Keep the literal worktree path only in the "Executed 2026-11-15 … using the displayed paths" sentence as provenance.
- **Status:** text-only; no re-check needed.

## Nonblocking findings

1. **Stale grade file can be read as the regrade result** (`gate_reproduce.py:81-91`).
   - `copytree` copies the stored `first-grade.json` / `final-grade.json` into the session the harness writes to. If the harness exits before writing, `actual` is the stored file, so the signatures match trivially.
   - The gate coordinator deletes these files first (`coordinator.py:464`).
   - This cannot cause a false match for this sample: every expected exit is 0, and every no-write path exits 1 or 2. It could for any trial whose expected exit is 1.
   - One-line fix: `(session / f"{phase}-grade.json").unlink()` before the run, or `copytree(..., ignore=shutil.ignore_patterns("*-grade.json"))`. That is a script change, so its finder would need to re-check it; optional.
2. **Concurrency cited lines are mostly only in the archive.** The Scope asks for the per-trial concurrency reviews with cited lines. The report gives all 28 verdicts but lines for only 2 (slots 032 and 027). The other 26 are in `grader-reviews.md` inside the in-repo, hash-recorded archive, which arguably meets durable publication. A one-cell-per-slot line column would make the report itself complete, as the integrity table already is. A rebuttal on the archive is acceptable.
3. **The sample tests passing verdicts only.** It is 9 Rust and 3 Tide, and contains no failure or integrity-hold trial. It meets the Acceptance (seeded, at least 10, both arms), and line 542 states the all-pass fact truthfully. Readers should know the regrade can't show that a failing structure or integrity verdict reproduces.
4. **Preflight budget is tighter than the gate's.** The gate procedure budgeted P = 10 min for preflight with the store scan; the report allows `timeout 240`. It fails safe, but a slow scan costs the window. Worth stating as a known risk.
5. **A stale probe would need a writer.** Lines 483-486 say the coordinator renews canaries or the probe if stale, while "no writer or credential is needed". Renewing `probe.json` (`coordinator.py:687-721`) launches a live restricted writer, a model call with a credential, and canaries compile. Neither fits the 14-minute envelope. Suggested wording: "if stale, rebook; renewal needs a writer and is out of this window." Not triggered today because the boot ID matches.
6. **Concurrent Rune suite load.** The booking allows a Rune suite to keep running, and jails run at CPUWeight 1 under nice 19. A load-induced test failure that is not a timeout would show up as a mismatch. Extend the "checked again" rule (lines 479-481) to any mismatch whose diff is timing-only.
7. **Small simplifications (nits):**
   - The helper's `TMPDIR` and `CARGO_BUILD_JOBS` overrides do nothing: the harness pins both per jail and uses `STATE` for grade directories. Line 531's "the helper also sets TMPDIR" can go with them.
   - The inner `timeout=600` per harness call can never fire before the outer 600-second run limit.
   - `REVISION` duplicates `state["identity"]["cheap"]["head"]`; reading it from state would leave one owner.
   - `unpack` reads the archive hash from the first line of `SHA256SUMS`; looking it up by name is sturdier. It also prints a hard-coded "120"; printing the actual count is better.
   - The test imports `tarfile` inside the test and uses `reproduce.json`.
8. **Unit-test proof not recorded.** The report doesn't cite a run of `gate_reproduce_test.py` (no compile involved). The coordinator's bead record should hold it.

## The two export lines: both stay

- **`gate-report.md:494`, `export E1_BOOKED_END_EPOCH QUIET_LOG`: stays.** `preflight.sh` is a child process and reads `QUIET_LOG` from its environment. The single-quoted `bash -c` body reads `E1_BOOKED_END_EPOCH`. Both must be exported, because `: "${…:?}"` also accepts unexported variables. Per B1, `HEAVY_QUIET_EXEMPT` and `E1_GRANT_UNTIL` belong on this line too.
- **`gate-report.md:500`, `export REPORT_ROOT G E1_COPY E1_REGRADES`: stays.** The single-quoted `bash -c` body reads all four through heavy.sh → `env` → `timeout`, which pass the environment through. The exports reach the harness but not the jails, because `harness.environment()` builds a fresh environment. A short name like `G` is therefore harmless.
- **Alternative (optional):** pass the four as positional arguments, `bash -c '…' _ "$REPORT_ROOT" "$G" "$E1_COPY" "$E1_REGRADES"`. That avoids leaking `G` into the environment but is not simpler.
