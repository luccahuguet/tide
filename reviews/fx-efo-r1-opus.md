Model: claude-opus-5-5 · Effort: high · Date: 2026-11-09

# fx-efo round 1, Opus review (docs review round, read-only)

## Verdict

**request_changes**, confidence medium-high. One blocking ambiguity, B1, and its fix is a short clause. Everything else meets the Acceptance and agrees across the repository.

## Scope

Candidate c9bdc8d on base 0583ecb (`git diff 0583ecb c9bdc8d`). It touches `.agent-protocols.local.md`, `AGENTS.md`, `.agent-protocols.json`, `GOVERNANCE.md` and `README.md`. I read every touched file in full and the bead (`br --no-db show fx-efo`, including its comments). I also checked the fx-ic2 comments that relay the item-2 decision.

To find other text that describes the review loop, I searched tracked files outside `reviews/`, `archive/` and `.git/` for these phrases: confirmation round, narrow re-check, comes clean, at most twice, Codex re-check, supervisor, converge, Opus review. I also searched `.beads/issues.jsonl` for the same phrases. Results:
- `.claude/` is not tracked in the repository. The main checkout's `.claude/` holds only `scheduled_tasks.lock`.
- The repository has no `CLAUDE.md`.
- `scripts/` has no review-loop text. `autorun.sh` runs `claude -p` implementers (see N1).
- The only open beads that mention the old loop are fx-efo and fx-ic2, and both mention it as history. The other matches are closed beads or historical review records.

Starcompass checks:
- I ran `starcompass check-consumer` on the candidate. It printed `valid: consumer files agree` and exited 0.
- The overlay's SHA-256 is `b7652fa8…b4d7e5`, which matches the manifest.
- The repository-local rules section of `AGENTS.md` is byte-identical to the overlay.
- I could not independently run the source-backed `check`. `~/pjs/lab/starcompass` is checked out at 6b9cdfd, not the pinned 37bc4b0, and the brief does not allow creating a worktree. The diff leaves the canonical protocol section and the protocol digests unchanged; only the overlay hash changes. So the author's reported pass is credible, but I did not reproduce it.

## Blocking

**B1. In docs-only design work, "text-only fixes need no re-check" can remove every re-check.**
- Where: `GOVERNANCE.md:47`. The overlay and `AGENTS.md` (`.agent-protocols.local.md:21-23`, `AGENTS.md:279-281`) import the same rule through "as AP-PROOF-001 says".
- The problem: AP-PROOF-001 writes this exemption for code changes, where text means comments and docs that carry no behaviour. AP-PROOF-001 also skips review entirely for docs-only work. Tide's Merge Rule is different: it reviews docs-only design work, and there the text is the contract (`design/01-design.md` owns every language decision).
  - Read literally, every fix in a docs-only design change is "text-only". A blocking finding against the v0 core would be fixed and land with nobody re-checking the fix. That conflicts with the sentence before it in the same bullet: "the reviewer that found each finding re-checks only its fix or rebuttal".
  - The old text points the other way. It said "text-only fixes restart this loop at most twice", which shows that Tide used "text-only" for fixes that still needed a re-check.
  - The owner's quoted approval in the bead comment of 2026-11-09 names the finder re-check and the full-rerun condition. It does not mention the text-only exemption. Bead Scope item 2 does include the exemption, so it is in scope. But applying it to design text is a technical reading that this change has to settle; the Decisions section of the local rules makes that reading the agent's call.
- Smallest fix: in `GOVERNANCE.md:47`, define the term. For example: "text-only fixes (wording, typos, links or cross-references that change no decision, rule, contract or example meaning) need no re-check; in docs-only design work, a fix that changes what a design text decides or requires is re-checked by its finder." The overlay already defers to GOVERNANCE.md and AP-PROOF-001, so it can stay as it is. If the overlay changes, re-render `AGENTS.md`.

## Non-blocking

**N1. The Agent roles section says two things that bead Scope item 1 does not.** The section is at `.agent-protocols.local.md:39-47` and `AGENTS.md:297-305`. The extra text is:
- the coordinator's duties: "briefs agents, decides, runs reviews, and lands";
- "The owner may assign a bead's implementation to another agent."

The first sentence elaborates "stays thin" and is harmless. The second is not in the bead or its comments. It does follow from the owner's general authority over scope, and it is the sentence that reconciles the rule with `scripts/autorun.sh`, which runs `claude -p` implementers and which README.md describes. Suggestion: add a line to the bead saying where the sentence comes from, for example the autorun and uclaude practice, so a later reader does not take it for an unrecorded decision.

**N2. "Codex … fixes review findings" overlaps with the Merge Rule's "the author fixes or rebuts each finding".** When the owner assigns implementation to another agent, that agent is the author. Possible wording: "Codex, as the usual author, fixes or rebuts review findings". This is optional.

**N3. Round numbers for parallel finder re-checks are not specified.** The recording paragraph in GOVERNANCE.md lets only "each change's parallel first-round pair" share a number. If Opus and the supervisor re-check their own fixes at the same time, each re-check takes the next unused number. That works, but nothing says so. One clause, either "parallel re-checks share a number" or "each re-check takes its own number", would prevent a collision when sessions record reviews in parallel.

**N4. The supervisor no longer sees fixes made for Opus's findings.** The old narrow Codex re-check covered every change since the candidate. Now the supervisor's verdict covers the candidate plus only its own findings' fixes. AP-PROOF-001 accepts this, the owner approved it, and the full-rerun trigger (a fix that changes behaviour beyond its finding or crosses a trust boundary) bounds it. No change is needed; I note it because the Primary rule says "without an AI supervisor pass".

**N5. The quoted approval is an option label.** The bead comment records "chose 'Align (Recommended)'", dated 2026-11-09, rather than the owner's own words. Together with the comment's statement of what was approved, this meets "explicit yes, quoted here with its date", so the Acceptance is met.

**N6. README.md:309-311 reads as if every fix is re-checked.** "each reviewer re-checks only the fixes to its own findings" leaves out rebuttals and the text-only exemption. It is acceptable as a summary, and B1's fix keeps it accurate.

**N7. Spelling.** GOVERNANCE.md uses "behaviour", while AP-PROOF-001 uses "behavior". This is a nit.

## Reasoning

**Acceptance.** Taking each item in turn:
- Item 1: `AGENTS.md`'s Repository-local rules state the rule in a new Agent roles section. It matches the bead's Scope wording: Codex implements in `.worktrees/<id>`, fixes findings, and drafts research with sources and options plus a recommendation; Opus decides, runs round 1 beside the supervisor pass, and approves landing; the coordinator stays thin.
- Item 2: it is applied, and the bead records the owner's dated approval.
- `check-consumer` passes. The source-backed `check` rests on the author's report and an unchanged canonical section; see Scope.
- This docs review round, together with the Codex pass, is the remaining Acceptance item.

**Consistency.**
- GOVERNANCE.md, the overlay and `AGENTS.md` (which is identical to the overlay), and README.md now describe the same loop.
- The new loop matches AP-PROOF-001 as rendered at `AGENTS.md:146-150` and the Starcompass 37bc4b0 README.
- The converge skill's description ("the reviewer that found it re-checks only those fixes") now agrees with the Merge Rule. The old "later converge rounds fix as usual" line is gone.
- No tracked file outside historical records still states "confirmation rounds until one comes clean" or the "at most twice" loop.
- The Agent roles section does not conflict with the Decisions section. Codex's "options plus a recommendation" are for Opus to decide, not a menu for the owner.

**Why B1 blocks.** The exemption itself is approved and is fine for code changes. The problem is that Tide's Merge Rule reviews docs-only design work, which AP-PROOF-001 does not. In that setting, the undefined term "text-only" can be read as removing all re-checks from design changes, the project's main kind of change. A one-clause definition settles it.

DONE
