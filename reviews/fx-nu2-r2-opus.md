Reviewer: Opus (fresh subagent) · Round: fx-nu2 r2 · Date: 2026-10-21

## Verdict

**Clean.** Nothing blocks. Two findings are non-blocking.

The candidate matches what was handed in:
- HEAD is 9489d09.
- The diff hash without `reviews/` is `bcab03b3…727a`.
- The diff touches only `.agent-protocols.local.md`, `AGENTS.md` and `.agent-protocols.json`.
- The overlay's sha256 (`d2f1b885…2b32`) equals the manifest's `overlay.sha256`.

## Findings

**1. Non-blocking. `.agent-protocols.local.md:62-63` (AGENTS.md:276-277): "When in doubt, take a web research pass, then decide".**
- The round-1 fix removed "an unclear case is technical", but this sentence can still be read as that rule. An agent unsure whether a choice is scope or technical could apply it and decide on its own, against AP-SCOPE-001's "stop when a missing scope choice would materially change the result".
- Its position, right after "For a technical choice…", makes the intended reading the likelier one. The HEAD text had the same sentence, so this is not a regression.
- Smallest fix: "When a technical choice is unclear, take a web research pass, then decide…".

**2. Non-blocking. `.agent-protocols.local.md:54`: "How to deliver a chosen outcome is the agent's to decide".**
- Round-1 Opus proposed a guard clause and it was not added. A literal reader could count AP-GIT-001's authority-gated operations as "how to deliver": creating a branch, rewriting history, force-pushing.
- The examples after the colon make this reading unlikely, and nothing in the trials showed it.
- Smallest fix, optional: append "; this grants no write a canonical protocol withholds."

**Bookkeeping, not candidate text:**
- fx-nu2 cannot close yet. Acceptance item 2 needs case 3 decided without asking, and both final-wording runs asked, as the text intends. Acceptance item 1 needs `.claude/settings.json`, which is still missing from the worktree.
- The bead comment already records both points honestly.
- The bead's Verification line (`claude -p`) and its Implementation Notes ("filing follow-up beads… unclear case, the agent decides") are now stale. Update them when the owner's AP-PLAN-001 exception is settled.

## Checked and fine

**Round-1 findings resolved:**
- Codex blocking 1 and Opus 1 (scope override): the closed scope list and the unclear-case default are gone. Authority now covers only how to deliver a chosen outcome, which fits AP-SCOPE-001's "Implement only a chosen outcome".
- Codex blocking 2 (AP-PLAN-001): the grant to file follow-ups is removed. Filing is left to an owner-approved exception, which is the route AP-EXCEPTION-001 requires. `.agent-protocols.exceptions.json` is empty, and that is consistent with the exception not being approved yet.
- Opus 2 ("features"): the list is gone.
- Opus 3: "Loosening a check that a bead's Acceptance or GOVERNANCE.md requires stays the owner's call" agrees with Autonomous runs ("weakening what done means is the owner's scope call…"). Both cross-references still resolve.
- Opus 4: the ban on asking, menus and "want me to…?" now applies only to technical choices, so scope questions and planning approvals can still be put to the owner with a recommendation. "which leftovers and in-scope fixes" replaces the wider "what a pending change includes".

**Nothing lost from HEAD:**
- the AP-SCOPE-001 anchor;
- "technical design decisions are the agent's";
- no technical questions to the owner, even when a bead says "owner decision" or "stop and ask";
- research → decide → record on the bead and the owning doc → report. "owning document" is a fair widening of "owning design document".

**No new conflict:**
- with AP-SCOPE-001, AP-PLAN-001, AP-EXCEPTION-001, AP-GIT-001 (beyond finding 2), or GOVERNANCE.md's Merge Rule;
- with Autonomous runs. Its "a technical question is not one; see Decisions" still agrees.

**Trial (cand-5, cand-6):**
- Both runs used an AGENTS.md byte-identical to the candidate's, and they behave as the text intends:
  - **Case 1:** each decided without asking. Each first checked whether fx-a1's Acceptance fixes 25,000 and said loosening it would be the owner's call. Each recorded the decision in D14 and on the bead and reported it.
  - **Case 2:** each added the leftover to the pending change, did not ask, and planned narrow re-review. That re-review is correct under the Merge Rule.
  - **Case 3:** each reported with a recommendation and asked approval to file. This is correct under AP-PLAN-001 as the text now leaves it. These offers are planning approvals, not technical choices, so they do not break the ban.
  - **Case 4:** each declined to act on a third party's comment and asked the owner. This is correct.
- **Behavior shift to keep in mind:**
  - Both final runs raised D14 (to 26,000 and 25,500), where both baselines kept 25,000 by moving an example into a chapter.
  - The text allows this ("budgets and thresholds"), and the bead's evidence shows the owner wants the agent to make that call.
  - But naming budgets as agent-owned does push agents toward raising them.
  - The bead comment reports the shift accurately.
- **Rendering:** AGENTS.md's Decisions section matches the overlay exactly.
