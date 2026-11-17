Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate b5ea458

# fx-1vwf.3 r2 re-check: Opus, findings 1, 2, 3 and 5

Scope: `git diff 378c23c b5ea458`, which touches only `trials/e3-pilot/harness/src/main.rs` (+212/−10), plus report.md's round-2 section. HEAD is `b5ea458`, the tree is clean, and `git diff 316ce5c...b5ea458 | sha256sum` gives `d987295c…88fc8`, which matches the frozen candidate. I ran no build or test. I did not open `sealed/`. I take the builder's word that 29/29 pass.

## Verdict

**approve.** All four taken findings are fixed. One new non-blocking finding: the path-refusal assertions in the new recovery test don't guard the path check they were meant to cover. The fix is a one-string change to the test. The grading and statistics changes are correct.

## Finding 1: rerunning a control hides a failure. Fixed.

- `adjudicate_control` (`main.rs:3186-3205`) no longer returns `Fail::Done` for a control that already has its answer. It calls `judged`, which returns the published answer at once without launching (`main.rs:3067-3069`). Then it recomputes `pass` the same way as on a first run, prints the same line, and returns `Ok` or `Fail::Setup` (exit 2). A failed control now exits 2 on every rerun.
- The lock, the prompt-hash check and the control lookup still run first, so a rerun can't skip them.
- Regression check: `judges_run_through_the_review_path` reruns controls 2 and 4 (both failed) and asserts `failed(...)`, a `Setup` error containing "failed for judge sol". It reruns control 1 (passed) with `.unwrap()`. Restoring the old `Done` early return would make all three assertions fail, so the test guards the fix.
- `Fail::Done` is still used by `review`, `drop`, `adjudicate` and `drop-judge`, so nothing is orphaned. The module doc's exit-code line ("3 the session already has its verdict") stays accurate.

## Finding 2: judge crash recovery has no test. Fixed, with one gap (new finding N1).

`judge_records_recover_like_review_records` covers the three paths I listed:

- **Interrupted Sol attempt.** It plants `attempt-1/launch.json` (scratch already gone) and `events.jsonl` (START, MESSAGE), with no `attempt.json`. It asserts that the answer's `attempts[0]` is `fault`/`interrupted` with `verdict: null`. That exercises `settle_attempts`/`recover` with `valid_answer_attempt` on the judge side. If the judge validator or the null verdict regressed, `published` would refuse the record or the assertion would fail.
- **Decided while `answer.json` is missing.** It plants attempt 2 as a published verdict. `adjudicate(..., None)` concludes the answer without a login, so it can't have launched anything. It asserts `answer`/`true`.
- **Breach.** It asserts `Fail::Breach` and the empty-note refusal. A valid acknowledgement lets attempt 2 conclude the judgement. The test asserts `attempts[0].acknowledged.note` and `identifies`, and that acknowledging attempt 2 (a verdict) is refused as "is not a breach". All of these would fail on a regression of what they test.
- **The `adjudication/NNN` path check** is the gap. See N1.

## Finding 3: grading test gaps. Fixed.

I checked `split_confirmations_are_no_catch_and_a_finite_ratio_is_written_out` against `crossed()`'s sessions. Session 1 is c1 trust opus, 2 c1 plain opus, 5 and 7 c2 trust, 6 and 8 c2 plain.

- **(a) Split confirmations.** Session 1 has two located findings, at 65 and 66 within c1's 62–69 window, kind net. Opus confirms only the first and Sol only the second. The test asserts K_T = 2 (sessions 5 and 7), the either-judge K_T = 3, trust located-only = 1, and trust disagreements = 2 (one per finding, `main.rs:3440-3442`). If `identifies` were computed per session instead of per finding, K_T would be 3, so the test guards (a).
- **(b) The "does not rule out no benefit" branch.** The pairs are c1 (0,1) and c2 (2,1). Bootstrap resamples give 0, 1 and 2 with probabilities ¼, ½ and ¼, and no 0/0 resamples, since K_P ≥ 1 in every resample. Nearest ranks 25 and 975 of 1000 give [0, 2] for this fixed seed. a = 0 ≤ 1, so the sentence is added, and the test asserts `added_sentences` exactly.
- **(c) The numeric ratio with a finite interval.** The test asserts the whole §12 sentence: "(ratio 1.00, 95% interval [0.00, 2.00]) … does not meet". I checked it by hand: S = 4 per arm, 2 items × 2 models.
- The test-helper refactors (`pairs_run_with`, `answer_to`) are mechanical. `pairs_run` and `answer` still delegate with `crossed()` and finding 1, so the existing tests are unchanged.

## Finding 5: the twin rate uses the plant count. Fixed.

`more_clean_rejected = n_t > 0 && n_p > 0 && 10·fp_t·n_p > 10·fp_p·n_t + n_t·n_p` (`main.rs:3813-3815`). Multiplying f_T − f_P > 1/10 through by 10·n_t·n_p gives exactly this, in exact u64 with small values. When n_t = n_p = S, it reduces to the old `10·fp_t > 10·fp_p + S`, so the result is unchanged. With no twin sessions in an arm, it is false instead of dividing by zero. It is still gated on `met_now`. The first fixture still asserts true (2/4 against 1/4), and the new one false (0/4 against 0/4). No other output field changed.

## New findings

**N1 (non-blocking): the path-refusal assertions match any refusal, so they don't guard the path check.**
`main.rs:5946-5955` checks each wrong path with `setup_error(..., "is not")`. `acknowledge_breach`'s own refusal, "… is not a breach, or NOTE is empty" (`main.rs:2366-2369`), also contains "is not". So does `published`'s "is not a complete record" (`main.rs:255`).

Concrete regression: delete the `place` check in `acknowledge_adjudication` (`main.rs:3214-3225`). Then:
- `adjudication`, `/tmp/x` and `adjudication/../sessions` each resolve to an `attempt-1/attempt.json` that doesn't exist, or isn't a breach.
- `sessions/005/attempt-1` holds a review record, which `acknowledge_breach` either refuses or reads as not a breach.

Every path still gets a `Setup` error containing "is not", so the test still passes.

Fix: use the check's own text as the needle, e.g. `"is not adjudication/NNN or controls/K-JUDGE"`. The code itself is correct by inspection: a component count of 2, `Normal` components only, and a first component of `adjudication` or `controls`. Only the guard is vacuous. Re-checking this fix is trivial.

Nothing else is new. The only production change outside the two fixes is the doc-comment line on `adjudicate_control`. Grading (`Tally`, `Found`, catch rules), the gate, the bootstrap, McNemar and Wilson are untouched by this diff.

DONE
