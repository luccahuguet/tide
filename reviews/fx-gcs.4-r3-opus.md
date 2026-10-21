# fx-gcs.4 review, round 3 (Opus)

Reviewer: Opus, round 3, converge round (a fresh pass that fixes what it
finds; AP-PROOF-001, GOVERNANCE.md).
Candidate: worktree `tide-fx-gcs.4`, branch `fx-gcs.4`, base `755d86e`,
`git diff 755d86e` with new files intent-to-added.
Date: 2026-10-21.

## Verdict

Changed: one comment in a case was corrected.

## Changes

- `conformance/cases/contracts/form_loop_accept.rs:2-4`. The comment said the
  attribute "rewrites `for`, `while`, and `while let` loops". That describes
  the D18 spike, which rewrites only `Expr::ForLoop` and `Expr::While`
  (`spikes/d18/macros/src/lib.rs:322`, `:363`). The design's attribute
  rewrites each loop in the body (04 §1.2 rule 10), and CLAUSE-6 gives a
  `loop` a head. The comment now states the design's behavior and says the
  case has no `loop` because the spike leaves its `invariant!` unrewritten
  (04 CLAUSE-7). The README requires a case's comment to say where the
  spike differs from the design.

## Non-blocking, left alone

- `form_loop_accept.rs`'s `sum` repeats the first loop of
  `clause_loop_names_accept.rs`. Rounds 1 and 2 left this repetition alone
  because each rule keeps its own pair of cases.
- `design/04-contracts.md:501-502`: one line runs past the paragraph's wrap
  width. This is a nit.

## Checks

- Read every contract case, the 04 §1.1, §1.2, and §1.8 text, the
  `rules.md` and README edits, the runner, and the spike's loop rewrite.
  `grep` for FX0804-FX0809 outside reviews finds only 04's definitions.
- `/proc/loadavg` 4.43. `conformance/run.sh </dev/null` ran once, before
  the comment fix: exit 0, with 92 PASS, 19 AWAITS CHECKER, and 0 FAIL.
  After the fix, `conformance/run.sh contracts/form_loop_accept` gave PASS
  with exit 0. The fix changed only a comment, so the full run still covers
  the other cases.
- `git diff --check`: clean.

## Fingerprint

Taken after the fix and before this file was added:
`e2e98cad1700116bd1cba36ad9225fb74e9fda0e123c13fd86e2e36de3a6a83d`.
