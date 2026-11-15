Model: claude-opus-5-5 · Date: 2026-11-15

## Verdict

**request_changes**. There is one blocking finding. It touches only the §7.4 per-trial records; the gate numbers are correct.

## Scope

This is a read-only converge round on `fx-ic2.9` at HEAD `53703546e9c083458f3e5e2735a9365db2fc53c3`. It covers `trials/e1-checked/measures.py` and `measures_test.py`, read against:

- the bead's Scope;
- protocol §6 to §8;
- the record producers: `coordinator.py` (`audit`, `classify_session`, `finish`, `run_slot`) and `harness.py` (`command`, `grade`, `test_command`, `run_binary_cases`, `structure`);
- `run/grader-reviews.md` and the fx-ic2.7 review prompt's verdict vocabulary;
- the author's report and both Codex files.

I did not open `tasks.md`, `references/`, `suites/`, `rust/`, `reference_test.rs` or `reference-review.md`. I wrote nothing inside `run/` or the worktree. The only change in the worktree is the git-ignored `__pycache__/`.

Checks run with plain Python 3.12:

- `timeout 60 python3 trials/e1-checked/measures_test.py`: 9 tests passed.
- The script run on the stored records into a fresh scratch directory exited 0. Both hashes match the author's report: `measures.json` is `7a30a405…f295` and `per-trial.json` is `c1c9ee59…75ef`.
- An independent tally (scratch `tally.py`) read `slot-*.json`, `session.json`, the snapshot directories, `first-grade.json`, `final-grade.json` and `grader-reviews.md`. Its own parser applied the reviews by hand and cross-checked the coordinator's exit-code verdicts. Results:
  - Tide: F 56/60, P 58/60, I 86/58, all-60 mean 126/60.
  - Rust: F 59/60, P 58/60, I 59/58, all-60 mean 99/60.
  - The final failures are C3 slots 032 and 044 (Tide) and 068 and 116 (Rust).
  - The first-check failures are slots 029, 033, 047 and 051 (Tide) and 112 (Rust).
  - All three thresholds are met in exact arithmetic. The I ratio is 86/59 = 1.4576.
  - The tally matches the script per slot, not only in aggregate.
- An independent bootstrap (scratch `boot.py`) used the same draw rule but my own code. All six intervals and their undefined counts match the script to 1e-12.

## Blocking

### B1. Tide lint names are never recorded, and the script does not list them as unavailable

**Location:** `trials/e1-checked/measures.py:88-89`, in `summarize_rows`.

**Evidence:**
- Lint names are taken only from rustc's long-format note (``= note: `#[warn(dead_code)]` on by default``) or from cargo's JSON `compiler-message` code.
- The Tide driver always forces short diagnostics: `crates/tide-admit/src/invocation.rs:267` passes `"--error-format=short"`. Short format prints no notes, so a Tide lint looks like `…/src/main.rs:98:4: warning: function `run` is never used`, with no lint name.
- In the stored records, nine Tide trials have such short-format lint warnings:
  - slots 034, 038, 042, 047, 066, 070, 077, 084 and 088 (B2, B3, B4, B7, B9);
  - 23 grade rows, plus one writer run (slot 034, run 1).
- Every one of them has `diagnostic_codes: []`. Rust's eight trials with dead-code warnings record `dead_code`.
- `unavailable` is empty, so the report would publish Tide as lint-free. That is wrong for nine trials and does not match how the Rust arm is recorded.
- Reproduction: `summarize_rows([{"code":0,"stdout":"","stderr":"/w/src/main.rs:98:4: warning: function `run` is never used\n"}])` returns `diagnostic_codes: []`.
- The Scope lists "each diagnostic code (FX, rustc E codes, lints)" as a per-trial field that is either computed or reported unavailable. Here it is neither.

**Suggested fix:** The name cannot be recovered from short-format text, and regrading with JSON output is heavy and out of scope. So:
1. In `summarize_rows`, count warning lines that carry no code (`^\S+:\d+:\d+: warning: ` with no `[FX…]`/`[E…]` code), and record the count per run or grade as, for example, `unnamed_lint_warnings`.
2. When that count is above zero, add an `unavailable` entry, for example "lint name not printed: driver uses --error-format=short".
3. Note under `definitions` that Tide lint names are unavailable from the stored records.
4. Add a short-format fixture row to `measures_test.py`.

A fixed table that maps message prefixes to lint names ("is never used" to `dead_code`, and so on) would work, but it is brittle. Prefer the count plus the unavailable entry.

Gate numbers do not change. `per-trial.json` and `measures.json` change, so the hashes on fx-ic2 have to be recomputed.

## Non-blocking

1. **Failing doctest names are missed.** Location: `measures.py:99`.
   - Evidence: the pattern `^test (\S+) \.\.\. FAILED` cannot match libtest's doctest names, which contain spaces (`test src/lib.rs - parse (line 3) ... FAILED`). The Rust writer-test doctest row (`harness.py:439`) has no `--exact`, so a failing doctest records no name. `summarize_rows` on such a row returns `failing_tests: []`.
   - No doctest failed in the stored records, so the outputs are unaffected.
   - Fix: use `(?m)^test (.+?) \.\.\. FAILED$`.
2. **The integrity vocabulary is wider than the review prompt's.** Location: `measures.py:48`; the test pins it at `measures_test.py:120`.
   - Evidence: `ic2.7-review-prompt.md:29` fixes the vocabulary as `STANDS | DOES-NOT-STAND | n/a`. The script also accepts `STAND` and treats it as a pass. That is an unsupported extension of the integrity decision, which gates final pass.
   - Fix: drop `STAND` so that a stray token fails closed, and update the test.
3. **Lint capture can record names that are not fired lints.** Location: `measures.py:88-89`.
   - Evidence: the pattern also accepts `#[allow(...)]` and scans all compiler stdout and stderr. A rustc source snippet that quotes the writer's own `#[allow(dead_code)]` would be recorded as a lint.
   - The group in "(part of `#[warn(unused)]`)" is recorded as the code `unused` next to `dead_code`. The stored output has 8 such pairs.
   - Fix: match only the note forms rustc emits (``= note: `#[(warn|deny|forbid)(NAME)]` on by default`` and "the lint level is defined here"), and keep the first name only.
4. **Breach records outside slot records are not checked.** Location: `measures.py:174`, `:308-310`.
   - Evidence: confinement breaches are counted from slot-record audits only. `classify_session` (`coordinator.py:327-329`) raises before any breached attempt is recorded, and it writes `sessions/*/confinement-breach.json`.
   - A breach therefore never reaches a slot record. At most it shows up indirectly, as a `host_interruption` attempt with a null count.
   - Fix: also count `sessions/*/confinement-breach.json` files and refuse or report when any exist. The cost is one glob.
5. **A non-30-task sample is only flagged, not refused.** Location: `measures.py:311-312`.
   - Evidence: `protocol_sample` is reported, but threshold verdicts are still issued on a sample that is not 30 tasks.
   - The stored run has 30 tasks.
   - Fix: when it is false, set `all_thresholds_met` to null, or refuse unless a test flag is given.

## Reasoning

Each requested point, checked against the code and the records:

- **F (§7.1).** F is computed from the retained attempt's first grade: `--first --compile-only` checks the writer's tests and, for libraries, the acceptance boundary. It also requires `runs > 0`, structure is correctly ignored, and timeouts fail. A faulted last attempt fails. The fixture covers a no-run trial.
- **P (§7.2).** Final pass requires the structure check and every non-integrity row. A timeout fails. A concurrency review is required for C1 to C6 and C9, and FAIL fails the trial. An integrity review is required whenever an integrity row exists, and DOES-NOT-STAND fails the trial. An unresolved infrastructure grade raises. Suite removals other than "none" fail closed, which is acceptable because the record says none. All 28 concurrency reviews are PASS, and the three integrity rows (094, 103, 109) are STANDS.
- **I.** I is the mean of `len(retained runs)` over final passes only. With no pass it is None, and threshold 3 is then false. The all-60 mean counts every failure as 20.
- **§8 inequalities.** They are exact `Fraction` comparisons: 4/5 and 3/2 times Rust's value, and P(Tide) >= P(Rust).
- **Bootstrap.** One `random.Random(20261031)` draws `choices(sorted_tasks, k=30)` 10,000 times. Each draw keeps all four of a task's trials together, so both replicates and both arms go together. Intervals use linear-interpolation percentiles. Undefined values are dropped and counted per interval, and the width is reported. Drawing sequentially with k=30 uses the same stream as one large draw, so the output does not depend on how the draws are batched.
- **Tokens.** All four come from the last `type=result` event. I compared all 120 sessions with the audit's per-message sums: the three input-side totals agree exactly, and the audit under-counts output tokens (for example 2072 against 419). The result event is therefore the right source, and the script uses no fallback.
- **§7.4 fields.** These are recorded or reported unavailable: run verdicts, E and FX codes, failing test names (via `--exact` and `caseN`), wall time, turns (documented as distinct message IDs), peak context, Tide reference sections, first-snapshot bytes and holes. Every Tide trial has reference reads. `design/reference.md` is 1,072 lines, so a Read with no limit covers the whole file, and only the Read tool is allowed. Lints are the exception (B1).
- **Counts.** All are zero in both arms. That is consistent with the records: every slot ended with `done`, with no reruns and no faults.

B1 is a recording gap in a Scope-listed field. Because the script reports nothing as unavailable, the gap is silent, and only the Tide arm is affected. It has to be fixed before the hashes are recorded on fx-ic2.

DONE
