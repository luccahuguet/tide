Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 378c23c

# fx-1vwf.3 r2: Opus review (converge round, reports only)

Scope: `git diff 316ce5c...378c23c`, which touches only `trials/e3-pilot/harness/src/main.rs`. The diff SHA-256 is `29f3dc515aced84026699cf051a846f6bd5a2b9ef769edd3638b5d4f8540b239` and matches the frozen candidate. HEAD is `378c23c0fc87…` and the tree is clean. I read AGENTS.md, GOVERNANCE.md, the bead, protocol.md in full, the builder's report, and Codex's r1 and re-check. I did not open `sealed/`. I ran no build, test, or model.

## Verdict

**approve.** Confidence: high that the grader counts correctly and that the statistics follow §10; medium-high overall.

## Blocking findings

None.

## Non-blocking findings

1. **Rerunning a control that already has an answer hides whether it failed.** `main.rs:3186-3188`.
   `adjudicate_control` returns `Fail::Done` (exit 3, "already has its answer") for any control with a published answer, without checking pass or fail. Suppose the coordinator's 12-control batch is interrupted and rerun by a script that treats exit 3 as "already done", the way review's exit 3 is usually treated. A control that failed on the first run then reads like a finished one, and session 1 can start against §9.2. The published answer still shows the failure, so this would surface at publication, but by then it is too late. Minimum fix: when the answer exists, compute `pass` from it as the first run does, print the same line, and return `Ok` or `Fail::Setup` (exit 2). Or always return `Setup` for a published control that failed.

2. **Crash recovery for judge records has no test.** `main.rs:3065-3103`, `3212-3232`.
   Judges reuse `settle_attempts`, `recover`, `standing` and `acknowledge_breach`, which review's tests cover. The `valid_answer_attempt` plumbing through those functions is not covered on the judge side. Three paths have no test:
   - an interrupted judge attempt, recovered with `verdict: null`;
   - attempt records that decide a judgement while `answer.json` is missing, which `judged` should then conclude;
   - `acknowledge-adjudication` on a judge breach, including the `adjudication/NNN` path check.

   By inspection all three are correct today. One cheap test that plants each state under `RUN/adjudication/NNN/` and calls `adjudicate` and `acknowledge_adjudication` would guard them.

3. **Small gaps in the grading tests.** These are correct by inspection, but no test would catch a regression:
   - (a) A session with two located findings, where Opus says `true` on one and Sol says `true` on the other, must not be a catch. `Found::identifies` is per finding, so it is correct now, but no fixture has two located findings in one session.
   - (b) §12's "The interval does not rule out no benefit." branch, for `a ≤ 1` or an interval that is not estimable, is untested.
   - (c) The sentence with a numeric ratio and a finite interval is untested.

   A second small `results` fixture with a finite ratio would cover (b) and (c).

4. **The adjudication deadline can start early in a rare crash case (nit).** `settled_ms`, `main.rs:3237`.
   The settle moment is the last attempt's end, not when `verdict.json` was published. If the harness stops between `attempt.json` and `verdict.json`, and the session is concluded days later, the 14-day clock is anchored before the session actually had its outcome. The window is microseconds wide, so I'm only noting it. The verdict file's mtime, or a later `at`, would close it.

5. **The twin rate check reuses the plant count (nit).** `trust_rejected_more_clean_changes` (`main.rs:3996`) divides by the plant-session count `s` where the twin-session count belongs. The two are equal by construction (12 × 3 × m each), so the result is right. Using the twin count from `fp` would read truer.

6. **Void runs (concur with Codex).** The run format has no void record, so `grade` cannot refuse a void run. §11 is upheld only by the coordinator never grading one. This is operational, not a code defect.

## Reasoning

### Counting, recomputed by hand

I hand-graded the 16-session `grading_follows_the_sealed_rules` fixture against `PAIRS_MANIFEST`. c1 has span 65–66 and kind net, so lines 62–69 locate. c2 has spans 6, 50, 64 and 66 and kind fs.

- **Catches:** sessions 1 (line 67, net, both judges true) and 5 (line 50, fs, both true, masked).
- **Located only:** 6 (line 3, within 6−3; Opus true, Sol false) and 8 (line 64; Opus true, Sol faulted, which counts false).
- **Extra findings:** 2 (line 70), 3 (wrong kind), and 8's second finding.
- **Plant fault:** 4. **Twin fault:** 11, which is not a false positive.
- **False positives:** 9 and 15 in trust, 12 in plain.

Totals: trust `[2,3,0,1,1,0,0,2,1]`, plain `[0,3,2,2,0,2,1,1,0]`. K_T = 2, K_P = 0, S = 4, gate met. The either-judge gate is 2/2, not met. The masked-as-not-identifying gate is 1/0, met. McNemar has b = 2, c = 0, p = 0.5. False positives are 2/4 and 1/4; 0.25 > 0.10 with the gate met, so the flag is true. All of this equals the test's assertions.

### The catch rules in code

- **Located:** `first <= line + 3 && line <= last + 3`, plus the kind check. This equals §9.2's `first − 3 ≤ line ≤ last + 3` without unsigned underflow.
- **Identifies:** both judges must answer `true` on the same finding. A fault counts as `false`.
- **Catch:** requires a `reject` verdict.
- **Located only:** a located finding with no catch under `Rule::Both`, using whatever answers exist (Codex's r1 fix, confirmed).
- **Extra findings:** non-located findings on plant sessions.
- **False positives:** twin sessions that `reject`; a twin fault is not one.

The M1–M3 and M4–M6 rules belong to the frozen adjudication prompt and the judges. The grader rightly does not split by mechanism.

### Statistics against §10

- **Gate:** `2·K_T ≥ 3·K_P && K_T > K_P`, in exact integers.
- **S:** the count of retained trust-arm plant sessions, which equals 12 × 3 × m.
- **Bootstrap:** one SplitMix64 stream seeded 20261117, `next() % 12` over pairs in pair-number order, whole pairs kept in both arms. 0/0 resamples are dropped and counted. +∞ sorts last via `total_cmp`. Nearest rank is `div_ceil(k·n, 1000) − 1`. It is not estimable when `undefined·20 > 10000` or nothing is kept.
- **Independent recompute:** I re-implemented this in Python with exact `Fraction`s. It gives `[13/12, 4]` with 0 undefined, "not estimable" with 3538 undefined, and `[7, +∞]`, matching the test's constants.
- **McNemar:** an exact two-sided tail, capped at 1. b = c = 0 gives 1.
- **Wilson:** 5/10 by hand gives [0.2366, 0.7634].
- **§10.5 flag:** `3·K_P > 2·S`.
- **Ratio text and sentence:** they follow §10.3 and §12. The "does not rule out no benefit" rule (`a` not unbounded and `a ≤ 1`, or not estimable) is correct. The smallest ratio above 1 that can occur is 72/71, so the two-decimal display cannot round an `a > 1` down to "1.00".

### §11 paths

- **Dropped model:** a drop record excludes the model from the retained set, the adjudication order, `render-adjudication`, and every statistic. Its sessions are still listed with `retained: false`, and the "did not finish" sentence is added.
- **Order stays fixed:** the order cannot change after adjudication starts. `drop` refuses a model with no open session, and every input to the order (verdicts, drop records, a hash-checked manifest) is write-once.
- **Missing judge:** `drop-judge` refuses before 14 days after the last settle, refuses a judge with nothing pending, and settles decided records first. After it, `grade` gives a null gate and null primary catches, the `missing` answers, located-only counts per arm, the gate from the finished judge alone as information, and the no-result sentence.
- **Breach:** the judge path halts with exit 4 until `acknowledge-adjudication`, the same as `review`.
- **Void run:** no record exists (finding 6).

### §2 compliance

- The original tests module is a byte-identical prefix of the new one: SHA-256 `8c0f084a…` for both.
- `prompt/`, `controls/`, `protocol.md`, `Cargo.toml` and `Cargo.lock` are unchanged, so the prompt hashes are unchanged.
- The refactors of `attempt` (now taking the parser), `settle_attempts`, `acknowledge_breach`, `frozen_item`, `attempt_shape` and `last_json_block` are mechanical and keep `review`, `render`, `schedule` and the canary behaving as before. `verdict_parser` builds the same record value as the old inline code.
- Each command that reads the manifest first refuses until every session is settled, then checks §13's SHA-256 before parsing. The constant equals §13's `d1522520…ab35`.

### Records and crash safety

- Answer files and judge drop records are published once through `publish` (hard-link, no overwrite) and read back through `published` with shape checks.
- `grade` cross-checks each answer's judgement, session, finding and judge against the order, and each verdict file against its schedule entry.
- Mutating commands hold `RUN/lock`. The read-only commands (`grade`, `adjudication-status`) cannot race into a wrong result, because `grade` needs every answer or a judge drop record first.

### Complexity

About 1,300 new production lines for six commands, the result file and its breakdowns. That is proportionate. `Rule`, `Tally`, `Graded` and `Found` each have one job, and nothing is speculative.

DONE
