Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 3e307c8 (re-check)

Scope: I re-checked only my two blocking findings and five non-blocking items from target/work/opus-review.md against `git diff c291b19 3e307c8` (research/45 only, +149 −59) and target/work/fixes.md. I did not redo the full review. The work was read-only, apart from this file.

## Blocking findings

1. **§2 reframe (Decision 1): resolved.** The heading at :109 is "SUP: is it worth anything for Tide?". "What Tide could take from it (inference)" names the classical sharing idea, the use in `tide sim` under D17's Later "Snapshot and branch", and search under fx-onsw.7. It also rejects SUP as a Tide construct and as a speed claim. The quantum comparison now sits in a "Side note" paragraph. The table matches my decision: four rows, with the same ratings and the quantum row demoted to last. The search falsifier is kept. The summary at :7–10 carries the replacement text. The author's wording matches mine and does not overstate it.
2. **Superposed seed exploration (Decision 2): resolved.** The new subsection "Exploring the schedule space: DPOR and shared-prefix forking (inference)" carries my text as decided. It covers the literal seed-named form (small sharing, about log_b N levels) and the choice-vector form, which needs a REPLAY-1/2 format decision. It names the forking obstacle from one OS thread per task, says the two ideas complement each other, and keeps both falsifiers and the exhaustive tiny-model comparison. The table rates DPOR Later/High/Large/No and shared-prefix forking Later/Medium/Large/No, with no change of meaning. Every recommendation stays Later research with no simulator change, so nothing exceeds the bead's scope.

## Non-blocking items

1. **D3-b (Decision 3): resolved.** The "Decision on the partner's proposed payoff" paragraph is my text verbatim. The old phrases "replay may omit schedules" and "Capacity … can expose schedules" are gone. Capacity is now a liveness question, and sink order (SINK-4) is added. The falsifier list adds "interleaved sink output from two stages". One nit from my own decision text needs no re-check: the old qualification listed fault injection among the schedule-exposing factors, and the new paragraph does not. The scope excludes I/O capabilities, so this narrows nothing material. A later fx-onsw.2 note could mention node-crash faults.
2. **Stale status: resolved.** :16–17 now reads "Reviewed under the Merge Rule (Codex r1; Opus review round 2026-11-18, which decided the §3 D3-b disagreement)."
3. **"Opus recommends": resolved.** Both places now read "The independent Opus partner recommended" (:251 and the DPOR subsection).
4. **First-use expansions: resolved.** HVM is expanded at :5 ("HVM, the Higher-order Virtual Machine"), SUP at :7 ("A superposition node (SUP)", and again at §2's evidence), IC at :54 ("The Interaction Calculus (IC)"), and DPOR at its first use in the §2 table and again at its §3 introduction.
5. **Sources: resolved.** T4 lists chapter 03 §9.6 and SINK-4 plus scheduler.rs, task.rs, sim/manifest.rs and replay.rs, and the link definitions resolve (`[T4]` → ../design/03-authority-concurrency.md). Main (3346b7e) has no changes to any T4 file or to design/01 or design/reference.md since the note's stated baseline 19db31f (`git diff --stat` is empty), so the baseline line stays accurate. The P1 entry now names §2.1 and footnote 1 and §3's backtracking mechanisms, and both match target/work/dpor2005.txt. X1 is accurate: target/work/posix-fork.html's DESCRIPTION reads "A process shall be created with a single thread. If a multi-threaded process calls fork(), the new process shall contain a replica of the calling thread and its entire address space". The saved page's SHA-256 (ab97739e…a41c1) matches both posix-fork.sha256 and the hash recorded in the note. `[X1]` resolves to the cited URL.

## New problems introduced

None found. The removed lines are the old summary, the old §2 and the three replaced §3 paragraphs, all superseded by the decided text. No qualification was dropped apart from the fault-injection nit above. No new claim goes beyond the bead's scope, and no reference is broken.

## Verdict

**Approve.**

DONE
