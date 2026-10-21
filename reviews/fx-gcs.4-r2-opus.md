# fx-gcs.4 review, round 2 (Opus)

Reviewer: Opus, round 2, converge round (fresh pass that fixes what it
finds; AP-PROOF-001, GOVERNANCE.md).
Candidate: worktree `tide-fx-gcs.4`, branch `fx-gcs.4`, base `755d86e`,
`git diff 755d86e` with new files intent-to-added.
Date: 2026-10-21.

## Verdict

Changed (one comment fix in a case).

## Changes

- `conformance/cases/contracts/clause_param_accept.rs:6-8`. The comment said
  the D18 spike reads only the `Clone + Pure + Sync` parameter at return. The
  spike's lowering (`spikes/d18/macros/src/lib.rs:105-109`) runs the body in a
  non-`move` closure and evaluates `ensures` afterwards, so it reads every
  by-value parameter at return, including the reassigned `Copy` parameter in
  `inc`. CLAUSE-7 lists this gap as "entry values of by-value parameters in
  `ensures`". The README requires a case's comment to say where the spike's
  behavior differs from the design's, so the comment now names both `inc`
  and `grow`.

## Round 1 findings

Both round 1 blocking findings (Opus B1, Codex 1 and 2) and Opus N1-N8 are
addressed in the candidate. N9 (the tier of `#[contracts]`) stays open in
`rules.md`, as decided.

## Non-blocking, left alone

- `design/04-contracts.md:501-502`: one line runs past the paragraph's wrap
  width after the CLAUSE-7 edit. This is a nit.
- `form_loop_accept.rs`'s `sum` repeats the first loop of
  `clause_loop_names_accept.rs`. Each rule keeps its own pair, so the
  repetition is acceptable, as round 1 found.
- `design/reference.md` §10 calls the FX0804-FX0809 rejections "an error"
  without a code, as it did before. That is consistent with its own
  statement that codes are placeholders.

## Checks

- `/proc/loadavg` 9.02. `git diff --check`: clean.
- `conformance/run.sh </dev/null`, run once after the fix: exit 0, with 92
  PASS, 19 AWAITS CHECKER, and 0 FAIL. Of the 20 contract cases, 13 PASS and
  7 AWAITS CHECKER (FX0802-FX0809).
- Read every contract case, the 04 §1.1, §1.2, and §1.8 edits, the
  `rules.md` and README edits, and the spike's contract lowering.

## Fingerprint

Taken after the fix and before this file was added:
`962c56aaa5f8e407ba5a2bb688c217e1963ae0e1c214ad3d99d8e2e47da07703`.
