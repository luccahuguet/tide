# Tide review, review-names round 2 (Opus): confirmation of per-bead review names

## Verdict

**approve**: 0 blocking findings, 2 non-blocking notes.

Candidate: uncommitted working tree on HEAD `2d82c40`, `git diff -- . ':!.beads'`
(GOVERNANCE.md:46 and :74, README.md:114-118).

## Closure

| First-round point | Status | Evidence |
| --- | --- | --- |
| How round numbers advance after the pair (Codex B1, Opus N2) | Closed | GOVERNANCE:74: "Each bead or slug starts at round 1, which the parallel pair shares; every later review, Opus confirmation or Codex re-check, takes the next unused round number within that bead". This is one counter per bead across both reviewers, as both reviews proposed |
| Handoff continuation (Opus N2) | Closed | GOVERNANCE:74: "continuing across commits and handoffs" |
| Slug collisions (Opus N4) | Closed | GOVERNANCE:74: "a short kebab-case slug not yet used in `reviews/`" |
| Over-wide README line (Opus N5) | Closed | README.md:114-118 rewrapped. Widths are 70/67/66/73/40, in line with the surrounding paragraph |
| Auto-memory (Opus N1, outside the repo) | Closed | `codex-supervisor-invocation.md` gives `<bead>-r<N>-codex.md` and says the pair shares a round number, with later rounds counting on across both reviewers |
| fx-eit evidence text (Opus N3, Codex non-blocking) | Out of scope for this diff | Planning-only. Not rechecked |

### Sequence walk (bead `fx-x`)

1. Parallel pair: `fx-x-r1-opus.md` and `fx-x-r1-codex.md` (GOVERNANCE:46 "share a round number", :74 "round 1, which the parallel pair shares").
2. Opus confirmation: `fx-x-r2-opus.md`. If it is not clean, more confirmations follow as r3, r4, and so on, each the next unused number.
3. Narrow Codex re-check: `fx-x-r3-codex.md`.
4. The re-check leads to a change, so a narrow Opus round runs: `fx-x-r4-opus.md`. The next narrow re-check is `fx-x-r5-codex.md`.
5. Handoff: the next session takes the next unused number, r6. The highest existing `fx-x-r<N>` in `reviews/` shows what is used.
6. Later commit of the same bead: numbering continues from r6 ("continuing across commits"). See N1 for that commit's own parallel pair.

Each step yields one name only. No two files can share a name, because the
suffix tells the pair apart and every later review gets a fresh number.

## Blocking findings

None.

## Non-blocking notes

**N1. A later commit's own parallel pair.** The parenthetical in :74 ("Opus
confirmation or Codex re-check") does not name the first-round pair of a
second change to the same bead. Line 46 still says the pair shares a number.
That gives `fx-x-r6-opus.md` and `fx-x-r6-codex.md`, the reading consistent
with both lines. A strict "next unused" reading, applied after one file of
the pair has been copied in, could give r6/r7 instead. The names still cannot
collide, so only the numbering could differ. Optional wording: "every later
review takes the next unused round number, except that each change's
parallel pair shares one".

**N2. Escalation reviewer suffix.** The "second independent review" (line 72)
has no suffix unless it is Opus or Codex. Nit. Escalations are rare, and an
agent would pick the reviewer's model name.

No contradiction with the rest of GOVERNANCE.md. Line 46 (pair shares a
number), line 47 (loop structure), and the PR-discussion alternative all
agree with :74. AGENTS.md and `.agent-protocols.local.md` do not name review
files, so no render is needed.
