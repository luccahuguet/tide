Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-10-21

## Verdict

**request_changes** — high confidence. The direction is sound, but reproduced failures violate the proposal, owner-stop, fail-open, and scratch-only contracts.

## Scope

Read `AGENTS.md`, `GOVERNANCE.md`, `BR_WORKFLOW.md`, bead `fx-h6o`, the complete diff and touched files, relevant `autorun.sh` code, and `~/.claude/hooks/context-usage.py`. Reviewed the supplied settings JSON.

Confirmed HEAD `187a22b` and diff SHA-256 `1d7e792d48be3fb0c18c3803a4e54f9aab47d04d0a37685190d0ef5d0f557b1e`, unchanged after review.

Ran shell syntax and diff-whitespace checks successfully. The supplied test script failed because `/tmp` is read-only. Independently exercised its behavioral cases, threshold boundaries, and additional failure cases using anonymous in-memory transcripts and an exported stub `br`.

## Blocking

1. **The proposal matcher blocks ordinary reports and misses direct handoffs.**  
   [scripts/handoff-guard.sh:27](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:27), [scripts/handoff-guard.sh:47](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:47)

   At 250k with a pickable bead, `Review complete. The tests cover resuming a bead in a new session.` produces a block instructing the agent to claim work. Conversely, `fx-a is closed. I'll hand off now.` and `Would you like me to continue?` pass through. All were reproduced.

   **Smallest fix:** replace bare session mentions with explicit proposal patterns, recognize direct `hand off` and the common continuation question, and add these cases to the existing test.

2. **Common owner stop requests are ignored.**  
   [scripts/handoff-guard.sh:29](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:29), [scripts/handoff-guard.sh:52](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:52)

   Owner prompts `please stop`, `Finish this bead and stop.`, and `Finish fx-a. Stop when done.` all permit the hook to block a subsequent continuation offer. Anchoring bare `stop` to the beginning of the entire prompt misses these explicit instructions. The second-stop escape limits repetition but does not satisfy the owner-stop exemption.

   **Smallest fix:** recognize polite and sentence/clause-ending stop requests. Add regression cases alongside the existing continuation-prompt case, preserving negated requests such as `do not stop`.

3. **Missing or failed evidence can still produce a block.**  
   [scripts/handoff-guard.sh:40](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:40), [scripts/handoff-guard.sh:49](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:49), [scripts/handoff-guard.sh:54](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard.sh:54)

   With an offer and ready bead, an empty `usage: {}` blocks while reporting `0k`; a transcript without an owner prompt also blocks. A stub `br` that prints valid ready JSON and exits nonzero still causes a block because its exit status is discarded. These contradict the stated fail-open behavior.

   **Smallest fix:** require usable token-count evidence and an identified owner prompt before blocking. Capture `br` output only on successful exit, then filter it. Add focused failure cases.

4. **Failed scratch setup redirects test writes outside scratch.**  
   [scripts/handoff-guard-test.sh:8](/home/lucca/pjs/lab/tide-fx-h6o/scripts/handoff-guard-test.sh:8)

   When `mktemp` failed here, execution continued with empty `TMP`, attempting writes to `/bin/br`, `/ready`, and `/t.jsonl`. A privileged invocation could overwrite `/bin/br`.

   **Smallest fix:** make the `mktemp` assignment exit immediately on failure, before installing the trap or performing writes.

## Non-blocking

- [README.md:188](/home/lucca/pjs/lab/tide-fx-h6o/README.md:188) says the hook is wired, although `.claude/settings.json` is absent. Describe wiring as pending until the owner installs it.
- Keep the bead’s attended-session behavior trial and context-report accuracy acceptance open. Offline hook checks do not establish either observation.

## Reasoning

Reusing the existing context-report hook is justified: its implementation uses the stated formula, and another project reporter would duplicate ownership.

The `cli` gate fits the stated attended CLI scope. Dropping the session-attributed in-progress condition is reasonable given the missing attribution mechanism; blocking whenever another session holds a claim would defeat parallel work.

The supplied settings content has the expected Stop-hook structure and quoted script path. Its output protocol agrees with the [official hook reference](https://code.claude.com/docs/en/hooks#stop). Owner installation is a reasonable delivery step; it must preserve the `permissions` key from `fx-nu2`.

The threshold, ready-bead filtering, message precedence, headless exclusion, and second-stop escape worked in the reproduced cases. Approval needs focused corrections to the existing scripts, without another enforcement layer.

Not checked: live attended Claude behavior or installed settings; the original scratch-based suite could not complete in this read-only environment.  
Risk: current matching can interrupt authorized stops and direct completed read-only sessions toward unrelated work.