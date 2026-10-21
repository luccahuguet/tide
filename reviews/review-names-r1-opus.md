# Tide review, review-names round 1 (Opus): per-bead review file names

## Verdict

**approve**: no blocking findings, five non-blocking notes. N1 is outside
the repository diff, but the author should fix it with this change.

The new rule is unambiguous where it has to be: two sessions can no longer
produce the same file name. GOVERNANCE.md and README.md agree. No other
file outside `reviews/`, `archive/`, `trials/`, and `research/` prescribes
the old naming. The one live prescription of the old naming is the user's
auto-memory (N1). It is outside the candidate, but every session loads it.

## What was reviewed

The uncommitted working tree against HEAD `2d82c40`: `git diff -- .
':!.beads'`, SHA-256
`1e79116904e703116d7c66512896ef80714034cb270f60f1b7833c222d15ff63`.
`git diff --check` passes. The diff edits GOVERNANCE.md:46 (the first-round
pair "share a round number" in place of "take consecutive round numbers,
Opus first") and GOVERNANCE.md:74 (the naming rule), and README.md:114-116
(the `reviews/` entry).

Searched for `round<N>`, `roundN`, `round[0-9]+-`, `reviews/`, and "round
number" in AGENTS.md, `.agent-protocols.local.md`, BR_WORKFLOW.md,
TASKS.md, `design/`, `scripts/`, and `spikes/`. Read `br show` for every
open or deferred bead whose body mentions rounds or `reviews/`: fx-eit,
fx-djp, fx-e3t, fx-0o3, fx-qzp, fx-t99, fx-gcs. Also read the project
memory directory.

## Checks

| Case | Result |
| --- | --- |
| Bead ID with a dot | Clear. The example `fx-gcs.2-r1-codex.md` covers it. The name parses from the end (`-r<N>-(opus\|codex).md`), so a bead ID containing `-r1` still parses |
| Change without a bead | Clear: "a short kebab-case slug for `<bead>`" (GOVERNANCE:74). This review's own name, `review-names-r1-opus.md`, follows it |
| Parallel first-round pair | Clear: GOVERNANCE:46 "share a round number", and :74 gives `-opus`/`-codex`, so the pair is `<bead>-r1-opus.md` plus `<bead>-r1-codex.md` |
| Opus confirmation rounds, narrow re-check | No two sessions can collide. The numbering is underspecified, though (N2) |
| GOVERNANCE vs README | Agree on the pattern and on the history status of `round<N>-*.md`. README gives no slug case and no example, which is fine because GOVERNANCE owns the rule |
| Overlay / AGENTS.md | Neither names review files, so no render is needed. The repository-local text needs no change |
| `design/` | Cites specific historical files (`reviews/round1-soundness.md`, etc.), and those files stay. No naming rule |
| `scripts/` | `count-tokens.sh` only. No review naming |
| BR_WORKFLOW.md, TASKS.md | No review naming |
| Open beads | fx-djp, fx-e3t, fx-0o3, fx-qzp, fx-t99, and fx-gcs cite specific historical files, or "a review round in `reviews/`". All still work. fx-eit still works but carries a measurement trap (N3) |

## Blocking findings

None.

## Non-blocking notes

**N1. The auto-memory still prescribes the old naming (outside the repo;
fix with this change).**
`~/.claude/projects/-home-lucca-pjs-lab-tide/memory/codex-supervisor-invocation.md`
gives the Codex command as `-o <scratchpad>/roundN-codex.md` and says "Its
last message becomes `reviews/roundN-codex.md`". Its last bullet says "the
pair takes consecutive round numbers, Opus first". Every session in this
project loads that memory, and its exact command is more specific than
GOVERNANCE:74. Parallel worktree sessions that follow it will reproduce the
collision this change exists to remove. Smallest fix: change the command to
`-o <scratchpad>/<bead>-r<N>-codex.md` and the target to
`reviews/<bead>-r<N>-codex.md`. Change the last bullet to "the pair shares a
round number (`<bead>-r1-opus.md`, `<bead>-r1-codex.md`)". This edit does
not touch the repository diff and does not reopen it.

**N2. Say whether round numbers are shared across reviewers after round 1.**
"round within that bead" (GOVERNANCE:74) together with "share a round
number" (:46) most naturally means one counter per bead: `r1-opus` +
`r1-codex`, then `r2-opus` (confirmation), then `r3-codex` (narrow re-check),
then `r4-opus`, and so on. An agent could instead keep a counter per
reviewer (re-check as `r2-codex`). Either reading avoids collisions, but the
numbers stop meaning the same thing across beads, which matters for fx-eit
(N3). Optional clause after "reviewer:": "rounds are numbered in order
within the bead, across both reviewers; only the first-round pair shares a
number". Further: a bead that lands in several changes, or passes through a
handoff, continues from the highest existing `<bead>-r<N>` in `reviews/`.

**N3. fx-eit's measurement must count files, not round numbers.**
fx-eit (Evidence) points at `reviews/round<N>-*.md` as "one file per
round". It asks for rounds per landed bead before and after `e32775f`.
After this change, the "after" sample lives mostly in `<bead>-r<N>-*.md`
files. Under the new names the parallel pair is one round number, but
under the old names it was two. A script that globs only `round*.md`, or
counts distinct round numbers, will understate the "after" cost by
construction. Each new file name also carries its bead, which makes
per-bead counting easier. Smallest fix, a planning-only comment on fx-eit,
by whoever owns that bead's state: "Count review files under both schemes
(`round<N>-*.md` mapped to beads through `git log`; `<bead>-r<N>-*.md` by
name). Count Opus and Codex reviews separately, not round numbers, because
the first-round pair shares a number from <commit>."

**N4. Slug collisions.** Two parallel bead-less changes could pick the same
slug (`readme-fix`). This is rare, and git would show it as an add/add
conflict, not silent loss. Optional: "a slug not already used in
`reviews/`".

**N5. README.md:116 is 118 characters wide.** The surrounding paragraph
wraps at about 70 characters. Rewrap lines 114-117. Nit.
