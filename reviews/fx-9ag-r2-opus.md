# Tide review, fx-9ag round 2 (Opus): narrow confirmation of the autonomous-run rules

## Verdict

**approve**: 0 blocking findings, 2 non-blocking notes.

Candidate: the uncommitted tree on HEAD `901c512`, `git diff -- . ':!.beads'`
SHA-256 `36ba47a0b23fc45c5c16965365488129ba29288ac50931e85c7d24d356347134`.
The overlay's SHA-256 (`b92fe0bb...1bb`) matches the manifest's new
`overlay.sha256`. AGENTS.md from `### Tide` onward is byte-identical to the
overlay. `starcompass check-consumer` with the four Tide paths exits 0.
`git diff --check` is clean. `git diff --stat` lists only the overlay,
`AGENTS.md`, `.agent-protocols.json`, and `.beads/issues.jsonl`. I did not
rerun the source-backed `starcompass check`. The r1 Codex pass ran it on the
same rendered files, and only the overlay text has changed since then.

## Closure

- **B1, closed.** The text now reads: "Never weaken it to close the bead;
  weakening what done means is the owner's scope call, while a technical
  correction that keeps or strengthens the check is the agent's (see
  Decisions)." It no longer treats every change as a scope call. A
  correction of a wrong flag or mechanism goes back to the agent, which
  matches Decisions. The candidate's own "end the session" to "end the run"
  change would now be allowed.
- **N1, handled.** "close the bead only when it passes" is now an explicit
  close gate.
- **N2, handled.** The trigger is "when the owner does not attend the run:
  they asked for an unattended run, or a session runner started it." The
  sentence no longer refers back to itself.
- **N3, handled.** "at least two approaches that differ in mechanism, not
  reruns or tweaks of one." This also answers the Codex note on "distinct
  approaches".
- **N5, handled.** "whose budgets still apply" makes AP-SESSION-001 take
  precedence over "a plateau is not a stop".
- **N7, handled.** "or with the scope choice and a recommended answer."
- **N4** is recorded on fx-3md (comment at 04:52, including the
  `br list --status blocked` versus `br blocked` distinction). **N6** is
  recorded in fx-9ag's second comment ("'End the run' replaces the
  Acceptance's 'end the session'...").

## Blocking findings

None.

Conflict check on the revised text:

- **AP-SCOPE-001.** A missing scope choice still stops the work: block, comment
  with the choice and a recommended answer, end the run. Nothing widens
  scope. AP-SCOPE-001 still requires the recommendation to be justified, and
  that requirement continues to bind.
- **AP-PLAN-001.** "End the run" after a block matches "return at
  completion, block, or handoff" and the one-issue-per-run rule. A
  failure-and-attempts comment counts as "material failures/negatives". The
  close gate fits "Keep status/prerequisites honest". The `br update`
  command uses the issue tool, not its storage.
- **AP-SESSION-001.** The budgets are explicitly kept. A budget handoff
  stays separate from a dead end, so the text does not turn a budget stop
  into `blocked`.
- **AP-EXCEPTION-001.** The section only adds rules. It narrows no import and
  needs no exception.
- **Decisions.** The B1 fix and the "a technical question is not one"
  parenthetical both send technical calls to the agent.

Acceptance: all three rules are present in three bullets. The done condition
comes from Acceptance and is never weakened. A plateau means changing
approach. A dead end follows the defined conditions, adds a comment, sets
`--status blocked`, adds `--add-label owner` when only the owner can unblock
it, ends the run, and never waits for an answer. The render is generated,
and the drift check passes. The wording runs a little past "a few lines",
but every addition closes an r1 note.

## Non-blocking notes

N1. "They add to AP-SCOPE-001, AP-PLAN-001, and AP-SESSION-001, whose budgets
still apply": grammatically, "whose" could attach to all three protocols,
but only AP-SESSION-001 sets budgets. A careful reader resolves this
correctly. "...AP-SESSION-001; its budgets still apply" would remove the
ambiguity.

N2. "a technical correction that keeps or strengthens the check" leaves one
case open: a criterion that is technically impossible as written and can be
fixed only by loosening it. Under the text, that loosening is an owner scope
call. That matches the bead's intent, since weakening done is scope, and it
is consistent with Decisions. The agent can still choose the technical fix
and recommend it. This note records the edge so a later reviewer does not
take it for a regression. No change needed.
