Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-10

# fx-ic2 dependency pruning: Opus review, round 2 (re-check of findings 1 and 2)

Candidate: commit 29a2974. I read the bead fields with `br --no-db show fx-rpwp fx-u65.6 fx-byk fx-m72 fx-ic2`.

## Verdict

approve

## Finding 1 (blocking): resolved

- **fx-rpwp Acceptance.** It now has an "Also (fx-ic2 pruning, owner decision 2026-11-10 …)" paragraph. That paragraph requires the §10.1 Owner-column remap (E1-P6 → fx-byk.1/.2, E1-P12 → fx-m72.1–.3, E1-P13 → fx-yg5.1–.5, E1-P14 stays fx-u65) in the same docs-reviewed protocol edit as the FX0516/0920/0921 exclusion. It cites §10.1 correctly. It also records why readiness.md stays as written.
- **fx-rpwp Scope.** It now reads "Protocol, tasks and thresholds do not change, except the two owner-decided protocol text edits in Acceptance", so the earlier contradiction is gone.
- **fx-byk and fx-m72 Notes.** Both now say "Close when all children close". fx-yg5 already did.
- **fx-ic2 comment.** The newest comment (23:29 UTC) says "28 in the graph, 31 under the current text" until fx-rpwp's edit lands, and that §10.2(1) also needs the three epics closed.
- **Nit (does not block).** After the remap, §10.1 would name fx-yg5.1 and fx-yg5.2. Both are closed, but neither has a `blocks` edge to fx-ic2. The remapped sentence "Every bead in the table blocks fx-ic2" would therefore be literally false for those two rows, although §10.2(1) is met. Either option fixes it:
  - When fx-rpwp writes the edit, word the sentence as "every open bead…".
  - Add closed provenance edges fx-ic2 → fx-yg5.1 and fx-yg5.2, as was done for fx-3iu, fx-ecb and the others.

## Finding 2 (non-blocking): resolved

- **Scope.** fx-u65.6's Scope now lists FX0911 and FX0915. It carves both out of "codes the driver already reports stay with their owners" and gives the reason: their closed owners left no conformance proof.
- **Acceptance.** It now covers both codes. For FX0915, a package-level diagnostic with no source line, it allows two proofs: a package-level conformance check the per-line parser doesn't need, or a tide-admit invocation test that asserts code, severity and verdict.
- **Owner.** The work now has a schedulable task as its owner instead of "at fx-u65 close". fx-u65 can close when its children do.

DONE

