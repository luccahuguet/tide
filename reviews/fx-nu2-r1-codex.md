Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-10-21

## Verdict

**request_changes — high confidence.**

The autonomy goal is sound. The overlay introduces two unrecorded protocol overrides, and the settings acceptance condition remains unmet.

## Scope

Read GOVERNANCE.md’s supervisor brief first, then:

- The frozen diff, full overlay, generated AGENTS.md, manifest, exceptions, README, and Beads workflow.
- `br --no-db show fx-nu2`, including Acceptance.
- The pinned Starcompass profile, protocols, and checker instructions.
- The trial prompt, six outputs, each arm’s AGENTS.md, invocation records, and interactive deny/control transcript.

HEAD and diff hash match the supplied candidate. Source-backed Starcompass `validate` and `check`, plus `git diff --check`, pass. The candidate remained unchanged.

## Blocking

1. **The ambiguity default can override the owner’s scope.**  
   [Overlay lines 53–58](/home/lucca/pjs/lab/tide-fx-nu2/.agent-protocols.local.md:53) makes scope an apparently exhaustive list and assigns every unclear case to the agent. [AP-SCOPE-001](/home/lucca/pjs/lab/tide-fx-nu2/AGENTS.md:38) requires stopping when a missing scope choice materially changes the result. Removing an existing deliverable, for example, is absent from the overlay’s list but still changes scope. The default can also bypass the unattended scope-blocking path at [overlay line 77](/home/lucca/pjs/lab/tide-fx-nu2/.agent-protocols.local.md:77).

   **Smallest fix:** confine technical authority to the chosen outcome; remove the exhaustive scope definition and “an unclear case is technical.” Keep the specific technical examples. Any intentional override instead needs an approved, recorded AP-SCOPE-001 exception.

2. **Follow-up filing bypasses AP-PLAN-001’s run boundary without an exception.**  
   [Overlay lines 55–56](/home/lucca/pjs/lab/tide-fx-nu2/.agent-protocols.local.md:55) grants follow-up filing without preserving [AP-PLAN-001’s limits](/home/lucca/pjs/lab/tide-fx-nu2/AGENTS.md:154): one owning issue, or an explicitly authorized planning-only batch, followed by return at completion.

   The trial demonstrates the conflict: after implementing and closing fx-b2, candidate agents continue the same exercise by filing additional issues. Closing fx-b2 does not start another authorized run; leaving follow-ups unclaimed does not make that run planning-only.

   **Smallest fix:** to retain this intended behavior, record a narrowly approved AP-PLAN-001 exception allowing material follow-up filings without claiming or implementing them, then regenerate. The [exception input is currently empty](/home/lucca/pjs/lab/tide-fx-nu2/.agent-protocols.exceptions.json:3). An overlay alone cannot suspend the imported boundary.

3. **The settings handoff blocks bead closure.**  
   [fx-nu2’s Acceptance](/home/lucca/pjs/lab/tide-fx-nu2/.beads/issues.jsonl:74) requires AskUserQuestion to be unavailable **in this project**. `.claude/settings.json` is absent. The isolated deny/control experiment supports the proposed mechanism; it does not install that mechanism here. The [autonomous completion rule](/home/lucca/pjs/lab/tide-fx-nu2/.agent-protocols.local.md:70) forbids closing before Acceptance passes.

   **Smallest fix:** keep fx-nu2 open with the owner’s settings action pending. Close only after application and verification in a fresh interactive project session. Replace the bead’s stale `claude -p` verification instruction. This is a closure blocker; a corrected overlay could land separately while the bead stays open.

## Non-blocking

- Report the final wording’s result as **2/2 final candidate runs**, alongside two earlier exploratory runs. Runs 3–4 differ from the frozen candidate only in wrapping, so their evidence remains applicable.
- Preserve the trial summary and tested identities durably. Temporary paths alone are insufficient proof records.
- The written exercise demonstrates decision and questioning behavior. It does not demonstrate actual durable writes; both final outputs also give “(none)” for the governance-case message, so reporting every decision is not fully demonstrated.

## Reasoning

The trial supports a narrow improvement: both baselines requested permission to file follow-ups; both final candidates decided without asking. Baselines already decided the budget and governance cases autonomously. All six withheld repository access pending owner authorization.

The prompt supplies situations without prescribing decisions, so it is useful behavioral evidence. However, successful instruction-following does not establish that the instruction complies with imported protocols. A clean render likewise proves structural integrity, not semantic compatibility.

Not re-run: Claude sessions or production tests. No files changed.  
Remaining risk: project-level tool enforcement is still pending.