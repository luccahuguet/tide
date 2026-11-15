Model: claude-opus-5-5 · Date: 2026-11-15

# fx-ic2.11 Opus review, round 2 (converge-style, reports only)

Candidate: `.worktrees/fx-ic2.11`, branch `fx-ic2.11`, HEAD `b50fdaa`,
`git show --format= HEAD | sha256sum` = `496117ee…f2d0fc` (matches the brief).
Gate worktree G at R `675c26e…`, clean, checked read-only.

## Verdict

**approve**. No blocking findings.

There are four **URGENT** pre-checks for the 16:30Z regrade (U1-U4 below).
None is a defect in the candidate's logic. Each is a condition under which the
command, as written, stops without producing proof. Check all four before
16:30Z.

## Scope

Every Scope item of fx-ic2.11 is present:

- the §8 verdicts with F, P and I per arm;
- the §7.3 details (passing-only I, the no-passing-trial rule, the all-60
  mean counting failures as 20);
- the bootstrap intervals with their widths;
- all 120 §7.4 records (`gate-results/per-trial.json`);
- the classification of every failure, with evidence and toolchain defects
  flagged;
- the concurrency reviews with cited lines (inside the archive, with
  examples in the report);
- the integrity decisions;
- the comment-only structure count;
- the §6 removal (none);
- the reruns, second faults and audits per arm;
- the §2 resemblance notes;
- every input identity, the snapshots and records stored with their SHA-256;
- both reproduction commands.

The seeded grading sample is still pending. The report says so truthfully
(gate-report.md:567-570). After 16:30Z the coordinator appends the result
block: the window, the command result, the `matches.json` hash and the 12
first/final matches. That block is new proof content added after this round.
Keep it to that status paragraph and send it to this reviewer for a re-check
only.

## Blocking

None.

## URGENT (would make the 16:30Z attempt stop without proof)

**U1. Run the command from the fx-ic2.11 checkout, not from main.**

- Where: gate-report.md:515, `REPORT_ROOT=$(git rev-parse --show-toplevel)`.
- Evidence:
  - `gate_reproduce.py` exists only on the unlanded branch.
  - `REPORT_ROOT` is computed from the caller's cwd, before `cd "$G"`.
  - Started from the main checkout, the preflight passes, then
    `python3 $REPORT_ROOT/trials/e1-checked/gate_reproduce.py` fails with
    "No such file" inside the slot. That wastes the window.
  - Started from outside any repository, `set -e` aborts at line 515.
- Fix: run it with cwd `/home/lucca/pjs/lab/tide/.worktrees/fx-ic2.11`, or set
  `REPORT_ROOT` to that path. Optionally add
  `[ -f "$REPORT_ROOT/trials/e1-checked/gate_reproduce.py" ]` before the
  preflight.

**U2. heavy.sh is narrowed to one slot all evening, and its wait has no bound.**

- Where: gate-report.md:523-530.
- Evidence:
  - `eon-overseer/one-slot-until` = 1794796859 (2026-11-16T02:40:59Z). Every
    heavy.sh run, `HEAVY_QUIET_EXEMPT` included, gets only `build.lock`
    (heavy.sh:61 overrides the slot list).
  - Today the Rune timing run held `build.lock` from 13:33Z to 14:49Z at
    `CARGO_BUILD_JOBS=1` (heavy.log). The booking comment expects "a capped
    Rune suite at --jobs 1" during the window.
  - If that suite, or any Eon run granted before quiet-until rose, holds
    `build.lock` at 16:30Z, heavy.sh waits indefinitely. Its loop has no
    timeout; the `timeout 600` is inside the granted command. On a late grant
    the inner guard exits 75, so no regrade runs.
- Fix:
  - At about 16:28Z, confirm in `heavy.log` that the last
    `grant:build.lock` has its `exit:` line, or ask eon-overseer to hold the
    Rune suite or lift one-slot for 16:30-16:45.
  - Bound the wait:
    `timeout $(( E1_BOOKED_END_EPOCH - $(date +%s) )) bash …/heavy.sh …`.

**U3. The QUIET OVER line is mandatory here, unlike in the gate batches.**

- Where: gate-report.md:510 (`: "${QUIET_LOG:?…}"`), together with the archived
  preflight.sh:39-47.
- Evidence:
  - At 16:30Z the 15:00Z standing slot has passed its `over` (16:30:00).
    With `QUIET_LOG` set, the preflight FAILs unless that file has a line
    matching `^[0-9TZ:-]+ eon-overseer: QUIET OVER` whose first field is
    `>= 2026-11-15T14:55:00Z` (a lexicographic ISO compare).
  - Every gate batch ran with `QUIET_LOG` unset. The batch logs show only
    "NOTE QUIET_LOG unset". This check has therefore never passed in practice.
  - I found no file that obviously carries such lines. The search was
    stopped: it is not exhaustive.
- Fix: before 16:30Z, run
  `grep -E '^[0-9TZ:-]+ eon-overseer: QUIET OVER' "$QUIET_LOG" | awk '$1 >= "2026-11-15T14:55:00Z"'`
  and confirm it prints a line. If eon-overseer writes no such inbox line, get
  it written first.

**U4. `E1_GRANT_UNTIL` must equal `quiet-until` exactly at preflight time.**

- Where: gate-report.md:509-513, together with preflight.sh:31.
- Evidence:
  - `[ "$E1_GRANT_UNTIL" = "$q" ]` is a string equality check, and
    `end = now + 14 min` must be `<= q`. With q = 16:45:00Z, the only valid
    start is 16:30:00-16:31:00Z.
  - quiet-until now reads 15:30:23Z, the standing slot. eon-overseer still has
    to arm 16:45Z. Its guard also extends quiet-until under pressure
    (AGENTS.md). Any change after booking makes the preflight FAIL.
  - Pasting a later q as `E1_GRANT_UNTIL` would also move the
    `E1_BOOKED_END_EPOCH` guard past the 16:45Z booking.
- Fix: at 16:29Z, `cat ~/.cache/eon-agents/eon-overseer/quiet-until` and
  confirm it is exactly the 16:45Z grant epoch before exporting it. If it is
  not, rebook. Do not reuse the new value as the booked end.

Also run it unsandboxed, as the gate batches were. The harness needs
`systemd-run --user`, bwrap and `/tmp` (the helper sets `TMPDIR=/tmp`).

Checked and fine for 16:30Z:

- canary and probe keys: boot ID `b5a8bafb…` is unchanged, and all three
  archived records give `key_current=True` (checked from scratch with `-B`;
  G is still clean);
- driver and std hashes match `state.identity.cheap`;
- all sysroot, vendor, deps and store paths of the 12 sampled sessions exist;
- the timers (cargo-sweep, nix-profile-trim, apt-daily-upgrade) next fire
  after the window;
- `$T/ic2.11-reproduction-records` is identical to a fresh unpack apart from
  the relocated `slot-*.json`;
- `$T/ic2.11-regrades` does not yet exist;
- past store scans took 31 s to 2 min 5 s, within the 240 s cap;
- the gate graded about 10 s per trial, so 12 trials take about 2-3 min
  inside the 600 s budget.

## Non-blocking

**N1. The summary omits the effect's direction and the ceiling caveat.**

- Where: gate-report.md:8-14.
- Evidence:
  - "E1 passes those thresholds" is true. The thresholds are non-inferiority
    margins: Tide needed 1.46× Rust's iterations (limit 1.5×) and passed
    first check 56/60 against 59/60.
  - The paragraph a launch post would quote leaves out both facts, and also
    that both arms were near ceiling. Those appear only at line 344.
- Fix: add one sentence to the summary, for example: "Tide stayed within the
  margins rather than beating Rust: first check 56/60 against 59/60, 1.46× the
  iterations against a 1.5× limit, final pass tied at 58/60; both arms were
  near ceiling, with sessions of about 30-40 s."

**N2. The §2 resemblance notes are listed but not read into the results.**

- Where: gate-report.md:378-385.
- Evidence:
  - The Scope asks for them "read into the results".
  - From `per-trial.json`, the 10 resembling tasks give F 19/20 against 19/20,
    P 20 against 20, and an I ratio of 1.43.
  - The 20 non-resembling tasks give F 37/40 against 40/40 (ratio 0.925),
    P 38 against 38, and an I ratio of 1.474.
  - All thresholds hold on the non-resembling subset as well, with a narrower
    I margin. The resembling B4 accounts for one Tide first-check failure
    (047).
- Fix: add that one-line sensitivity, labelled information only.

**N3. A stale copied grade file is masked only by the exit-code check.**

- Where: gate_reproduce.py:81-94.
- Evidence:
  - `copytree` copies the stored `first-grade.json` and `final-grade.json`
    into the regrade session. If the harness exits without rewriting them,
    `actual == expected` trivially.
  - Today only `run.returncode == expected_exit` catches that. The harness
    always writes before returning 0 or 1, so it is safe in practice.
  - The frozen coordinator unlinks the file first (coordinator.py:464).
- Fix: `(session / f"{phase}-grade.json").unlink()` before each harness call.

**N4. No explicit refusal of the frozen run directory.**

- Where: gate_reproduce.py:66-76 and gate-report.md:522.
- Evidence:
  - The helper only reads `records` and writes only to a new `output`, so
    with the documented command nothing touches `$T/run`.
  - Nothing refuses `records`, `output` or the preflight's RUN resolving
    under `$T/run`. Run against it, the preflight `--scan` would overwrite
    the frozen `store-before.json` and create `coordinator.lock` there.
- Fix: in `regrade`, raise if `records` or `output` is under
  `/home/lucca/.cache/eon-agents/tide-e1-gate/run`. In the report, add
  `[ "$E1_COPY" != "$T/run" ]`.

**N5. The interpreter is not pinned.**

- Where: gate-report.md:413 and :528.
- Evidence:
  - The report says to use Python 3.14.8 (the gate's recorded interpreter,
    `~/.nix-profile/bin/python3`).
  - The command runs bare `python3`, which in this Bash environment resolves
    to `/usr/bin/python3` 3.12.3. Through `sys.executable`, the frozen harness
    then grades under 3.12.
  - All five machinery files parse under 3.12, and I found no 3.13+ API, so
    it should work. It is still a gate-condition deviation the report does
    not mention.
  - The measures reproduction matched byte-for-byte under 3.12.3 (mine and
    Codex's).
- Fix: use `/home/lucca/.nix-profile/bin/python3` in the regrade command, or
  record the interpreter used.

**N6. The sample cannot reproduce a failing verdict.**

- Where: gate-report.md:563-565.
- Evidence: all 12 sampled trials are stored first/final PASS (9 Rust,
  3 Tide). The report discloses this.
- Fix: none required. If time allows later, a supplementary regrade of the
  four C3 finals and the three integrity holds would exercise
  failing-structure and integrity rows.

**Nits:**

- gate_reproduce.py:30 takes `SHA256SUMS` line 0 by position rather than by
  the name `records.tar.gz`.
- gate_reproduce.py:46 hardcodes "120".
- The concurrency citations and classification sit only inside the tarball.
  `grader-reviews.md` and `classification.md` could also sit unpacked in
  `gate-results/` for readers.

## Reasoning

What I ran (nothing that compiles, no grading):

- `sha256sum -c SHA256SUMS`: OK.
- `gate_reproduce.py unpack` into the scratchpad: archive and all 1,392
  member hashes OK, every member a regular file.
- `measures.py`, then `cmp` against both published JSON files: identical
  (Python 3.12.3).
- `gate_reproduce_test.py`: 2 tests OK.
- A credential-marker grep of the extracted archive for `sk-ant`, `oauth`,
  `OAUTH`, `Bearer `, `API_KEY`, `apiKey`, `access_token`, `refresh_token`
  and `eyJ`: 0 files, counts only.
- `session.json` holds no environment or credential. `writer.jsonl` is the
  usage-only result event.

Regrade correctness:

- The sample is `sorted(Random(20261031).sample(range(120), 12))` =
  [5, 9, 26, 28, 30, 31, 39, 48, 60, 65, 93, 97]. It covers both arms. The
  sessions are `NNN-1`, and there are no reruns.
- The flags match the frozen coordinator (coordinator.py:460:
  `--first --compile-only`, then no flags).
- `expected_exit` reproduces harness.py:748.
- `signature` keeps every verdict-bearing field: grade-level structure,
  timed_out and infrastructure; per row code, stage, case and observed_exit;
  `test_log` (libtest `--logfile`, deterministic `ok <name>\n`); the
  expected and observed acceptance test lists; integrity flags.
- It drops command, stdout/stderr, memory and CPU readings. Binary-case
  output differences already show in the post-verdict `code`, so dropping
  stdout hides no mismatch. Row count is compared through list length.
- A `grade_timeout` row or a flipped `timed_out` correctly counts as a
  mismatch, re-checked under the booking condition.
- The harness writes only into the copied session and `G/target/e1-checked`
  temp directories (git-ignored). G stays clean, and nothing writes to
  `$T/run`.

Claims:

- Trials 1-3 are called "gate trials" (line 232). "Pilot" appears nowhere.
- The verdict is labelled point-estimate-only, with the I interval crossing
  1.5.
- Nothing claims D0's gate closed (D0 and protocol §1).
- The C3, integrity and classification statements agree with the per-trial
  data I sampled.
- §2: the archive holds the writer programs (all snapshots and finals),
  grades with test logs, grader reviews, the classification, the C3
  adjudication, the parent comments and the readiness records. It holds no
  tasks.md, suites or reference solutions; those are already tracked in
  `trials/e1-checked/`. Storing them under `trials/` copies nothing into the
  reference, companion, diagnostics or examples, so §2 is respected.

DONE
