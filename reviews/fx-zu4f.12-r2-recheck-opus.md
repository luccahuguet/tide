Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 3c0e5f0 (B1 re-check)

## Verdict

**B1 resolved.** No remaining cell, total, footnote or deviation row in the record can be compared with the committed ledger to single out the private candidate's crate. The five non-blocking fixes are applied as asked, and none of them reintroduces a per-rank count.

## Scope

This re-check covers only my B1 and the non-blocking fixes from target/work/r2-opus.md, on HEAD `3c0e5f0ad1f91b88f4fb8044123739b9f2fdbf6c`. I read the "Round 2 fixes" section of target/work/report.md, `git diff -M 1caca4f 3c0e5f0` (the only change is the record, renamed to `research/52-…`), the new record's text wherever it names a rank or a count, and the unchanged commit message. I compared them with the fx-zu4f.12 run comments that are already in the committed ledger, as I read them in round 2. I made no edits apart from this file, ran nothing and wrote nothing to `.beads/`. This file names no rank or crate in connection with the candidate.

## Reasons

- **The leak is gone.** The ledger gives per-rank figures from before round 1. The record's per-crate Rows, Modeled (with union lines) and Tests columns, the "N modeled" cells in the pool table, the rank-4 "six properties in four tests" phrase and the rank-8 deviation row's per-category sum were the cells that could be compared with those figures. All are removed or made count-free: the pool table now reads "≥1 modeled", the per-crate table keeps only first-failure reasons, and the deviation row at `:231` says "3 fewer unportable rows" with no total. The fixer also folded the Over budget, Unportable and Not assertable cells into arm totals. That goes beyond my minimum fix and is correct: with the ledger's per-rank modeled counts, those cells would give first-version row totals and narrow the candidate further.
- **What remains carries no per-rank change.** The arm totals compared with the ledger's totals show only +2 modeled, +2 generated-input tests and +3 rows, which the record attributes to round 1 without a rank. The other rank-naming text says nothing about a change in counts: the deviation rows for ranks 4, 5, 8, 9 and 10, "three rank-8 tests", and "Rank 4's list counts a property with eight parts". The split-row note can be tied to a rank through its V0/V1 wording, but that row was already in the first-version list and is no part of round 1. The commit message gives only arm totals.
- **Residual narrowing.** [Redacted by tide-lead before commit: this passage described how private items relate to each other; it is withheld under the round's disclosure rule.] tide-lead removed the sentence this point concerned from the record before landing (5b347a5).
- **Non-blocking fixes.**
  - NB1 (`:36-39`, `:122-123`): the correction notice and the Arm A result now say the candidate's 2 properties were listed after the first runs.
  - NB2 (`:120-122`, `:417-419`): both places now say the control fails the competing claim from the same conflict, so no control can pass all three.
  - NB3: [Redacted by tide-lead before commit: this passage described how private items relate to each other; it is withheld under the round's disclosure rule.] (sentence later removed by tide-lead.)
  - NB4 (`:516`): the Framing line now notes the repeated case for the 16 fixed-input tests.
  - NB5 (`:64`): "existed by" replaces "existed at".
  - None of these names a crate.
- **Landing note.** I agree with the fixer's note: any review file that quotes 1caca4f's per-crate counts recreates the leak when read with the ledger. r2-opus.md and this file cite line numbers and totals only.

DONE
