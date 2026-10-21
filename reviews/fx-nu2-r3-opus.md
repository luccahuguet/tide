Reviewer: Opus (fresh subagent) · Round: fx-nu2 r3 · Date: 2026-10-21

## Verdict

**Clean.** Neither edit introduces a conflict, an ambiguity or a grammar problem.

## Findings

None block.

**Optional nit:** the clause "this grants no write that a canonical protocol withholds" says "write", so a literal reader could ask whether it also covers actions that are not writes. Every authority-gated action in AP-GIT-001 is a write (branch, reset, force-push, history rewrite), and so are AP-SCOPE-001's implementation, external-state and durable-planning writes. The wording is sufficient as it stands, and I recommend no change.

## Checked and fine

**Candidate identity:**
- HEAD is 9489d09.
- `git diff -- . ':!reviews' | sha256sum` is `77a0e6a6…1b7b`, which matches.
- The diff touches only `.agent-protocols.local.md`, `AGENTS.md` and `.agent-protocols.json`, apart from the untracked review files.

**Render:**
- The overlay's sha256 (`007354da…b711`) equals the manifest's `overlay.sha256`.
- The Decisions section in `AGENTS.md` (lines 266-278) is byte-identical to the overlay (`.agent-protocols.local.md:51-64`).
- `.agent-protocols.exceptions.json` is still empty, which is consistent with no exception being approved yet.

**Edit 1, `.agent-protocols.local.md:61-62`, "When a technical choice is unclear":**
- It closes round 2's finding 1. Research-then-decide now covers only choices already classed as technical.
- A case where it is unclear whether a choice is scope or technical falls back to AP-SCOPE-001's "stop when a missing scope choice would materially change the result". That is the intended split.
- It agrees with Autonomous runs: "a technical question is not one; see Decisions".

**Edit 2, line 57, "; this grants no write that a canonical protocol withholds.":**
- It closes round 2's finding 2. A literal reader can no longer stretch "how to deliver" to cover AP-GIT-001's authority-gated operations.
- It also blocks any reading that would get around AP-PLAN-001's single-issue writing limit, or create an exception that AP-EXCEPTION-001 has not approved.
- Grammar is sound. The semicolon joins two independent clauses, and "this" clearly points back to the whole grant in the preceding sentence. It does not point only at the last list item.
- It sits before the "Loosening a check…" sentence, which still reads as a separate limit and still agrees with Autonomous runs ("weakening what done means is the owner's scope call").

**The paragraph as a whole:**
- The scope/delivery split, the ban on asking (limited to technical choices), and research → decide → record → report form a consistent sequence.
- It does not conflict with AP-SCOPE-001, AP-PLAN-001 (recording on "the bead" is a write to the run's own issue, which is allowed), AP-GIT-001, AP-EXCEPTION-001, or GOVERNANCE.md's Merge Rule as summarized in AGENTS.md.
