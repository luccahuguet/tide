Reviewer: Opus (fresh subagent) · Round: fx-nu2 r1 · Date: 2026-10-21

## Verdict

Findings: 1 blocking, 4 non-blocking. Candidate hash matches (`bdcaaf4e…48823`, HEAD 9489d09). The diff touches only `.agent-protocols.local.md`, `AGENTS.md` and `.agent-protocols.json`.

## Findings

**1. Blocking. `.agent-protocols.local.md:53-58` (AGENTS.md:267-272).** The rule says who decides, but it reads as if it also grants permission to write, and that permission conflicts with imported protocols. No exception records the conflict.
- "every technical decision is the agent's", "filing follow-up beads for material findings" and "an unclear case is technical" together let an agent file beads during a run that only asked for inspection or review. AP-SCOPE-001 says such a run "authorize[s] no … durable-planning write".
- The same text overrides AP-PLAN-001's "Report unapproved findings; … deferred outside authorized batches". That is the reading both baselines followed.
- An agent can also label operations that AP-GIT-001 gates on authority as "technical", such as branch creation or rewriting history.
- AP-EXCEPTION-001 says only a recorded exception may narrow or replace a protocol, and `.agent-protocols.exceptions.json` is empty.
- Smallest fix: write the follow-up-bead part as the owner's standing approval, which is additive and fits as overlay, and add a guard clause. For example: "The owner approves in advance filing beads for material findings in a writing run (AP-PLAN-001). This section decides who makes a choice; it grants no write that AP-SCOPE-001 or AP-GIT-001 withholds." The alternative is an AP-PLAN-001 exception record (scope, reason, approver lucca, date, review when Starcompass resolves the wording).

**2. Non-blocking. Line 56-57: "Scope means new features".** In a language-design repo, "features" reads naturally as language features, and those are design choices the old text gave to the agent. This can bring back the asking failure in the other direction. The list also leaves out "priorities" and "direction", which line 53 names. Since "means" reads as a closed list, an agent could treat re-prioritising beads as technical. Fix: "Scope means priorities, direction, new workstreams the beads do not cover, and publishing, sharing, or moving the repository; an unclear case is technical."

**3. Non-blocking. Lines 54-56 vs 70-74.** Decisions makes "budgets and thresholds" and "review and governance mechanics" technical. Autonomous runs makes "weakening what done means" the owner's scope call. A bead whose Acceptance names a budget or a governance gate now gets two conflicting answers. A literal reader could also take "governance mechanics" as permission to drop a required review. Fix: add "(loosening a check a bead's Acceptance or GOVERNANCE.md requires stays scope; see Autonomous runs)". Alternatively, narrow the item to "how to apply and keep GOVERNANCE.md consistent".

**4. Non-blocking. Lines 58-60.** "Do not … offer a menu of options, or end with an offer such as 'want me to…?'" stands as its own clause, so it also reads as covering scope questions. AP-SCOPE-001 and Autonomous runs (line 80) require putting a scope choice to the owner with a recommended answer, and "Should I add them…?" in the control case is exactly that kind of offer. Fix: "For a technical choice, do not ask the owner, offer a menu, or end with…". "what a pending change includes" (line 55) is also wider than the recorded case, which was a leftover. Read loosely, it permits the silent scope growth that AP-SCOPE-001 forbids. Fix: "which leftovers and in-scope fixes a pending change includes".

**5. Non-blocking. Trial claim and bead bookkeeping (not candidate text).**
- The outputs support the stated counts:
  - base-1 ("Shall I file them?") and base-2 ("Should I create it?") both asked before filing.
  - cand-1 to cand-4 decided all three cases.
  - All six runs asked on case 4.
- But the baselines already decided cases 1 and 2 themselves. Only case 3 tells the two arms apart.
- The note that runs 1-2 raised the budget understates the remaining shift. cand-4, on the final wording, also raised D14 to 26,000. In total 3 of 4 candidate runs raised the budget and 0 of 2 baselines did.
- The `reference-budget-25k` memory records that the owner chose 25,000 on 2026-10-19, and the agent later decided to keep it. Raising it is still within the agent's authority, but the bead summary should report this shift honestly.
- The final rewrapped text was not trialled itself. A pure rewrap is fine.
- Acceptance item 1 is unmet until the owner applies the settings deny. The bead must not close before then. Its Verification line ("`claude -p` … cannot call AskUserQuestion") cannot fail, because `claude -p` never offers the tool. Replace it with the interactive tmux observation.

## Checked and fine

- Render: AGENTS.md's Decisions section is byte-identical to the overlay's. The `.agent-protocols.json` change is only the overlay sha.
- Autonomous runs' "see Decisions" cross-references still resolve and agree on "a technical question is not one".
- The rewrite keeps everything the old text said: the AP-SCOPE-001 anchor, the web-research → decide → record → report sequence, and the override of bead "owner decision" and "stop and ask". "owning document" correctly widens "owning design document" to cover governance and overlay decisions.
- No conflict with GOVERNANCE.md's Merge Rule or with "human maintainer remains final merge authority".
- The bead's scope is respected: no canonical text was edited, and the settings file is correctly left for the owner.
- The wording is plain and the em-dash-free style matches the rest of the overlay.
