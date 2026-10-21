# Tide review, fx-9ag round 1 (Opus): autonomous-run rules in the overlay

## Verdict

**revise**: one blocking finding, seven non-blocking notes.

The section adds the three rules fx-9ag asks for. It narrows no imported
protocol, and the `br` command is valid. One clause goes beyond the bead's
Acceptance and conflicts with the overlay's Decisions rule: it makes *any*
change to a done condition an owner scope call, not just a weakening (B1).
Under that clause an unattended agent that finds a technically wrong
acceptance criterion has two choices, and both break a rule. It can edit the
criterion, which B1's clause forbids. Or it can park a technical question on
the owner, which Decisions and the section's own parenthetical forbid.

## What was reviewed

The uncommitted working tree against HEAD `901c512`: `git diff -- .
':!.beads'`, SHA-256
`92aea9d59514c1c697d8e4a0f16c6b76dbc7503f09ff9dd8d3f1a9298a07a67f`.
`git diff --check` passes, and `git diff --stat` shows only the overlay,
`AGENTS.md`, `.agent-protocols.json`, and `.beads/issues.jsonl`.

- `sha256sum .agent-protocols.local.md` gives `e55e2f4f...3cf3`, which
  matches the manifest's new `overlay.sha256`.
- AGENTS.md from `### Tide` onward is byte-identical to the overlay (`diff`
  is empty).
- `starcompass check-consumer` with the four Tide paths exits 0 ("consumer
  files agree"). I did not run `starcompass check` from the pinned checkout.
- I tested `br`: `br update --help` lists `--status` and `--add-label`. In a
  scratch repository (not Tide's), `br update <id> --status blocked
  --add-label owner` moved the bead from `in_progress` to `blocked` and added
  the label in one call. `br ready` then left that bead out.
- I read `br show fx-9ag` and its decision comment, `br show fx-3md`, the
  imported AP-SCOPE-001, AP-PLAN-001, AP-SESSION-001, and AP-EXCEPTION-001
  text, and the overlay's Decisions section.

## Blocking findings

### B1. "Changing what done means" conflicts with Decisions

`.agent-protocols.local.md:60-63` (AGENTS.md:274-277): "Never weaken it, or
the bead's acceptance, to close the bead; changing what done means is the
owner's scope call."

Problem: the bead's Acceptance asks for "Never weaken it to close the bead",
and the candidate adds the wider clause "changing what done means". That
clause covers every change: a correction, a stronger check, or a different
design choice. Decisions (`.agent-protocols.local.md:48-50`) makes technical
design choices the agent's, "including a bead's 'owner decision' or 'stop and
ask' on a design choice". Rule 3 (line 66-67) agrees: "a technical question is
not one".

So take an unattended agent whose bead names a flag that does not exist, or
prescribes a mechanism the agent rejects on technical grounds. Under the
clause, fixing the criterion is an owner scope call, so the agent must block
the bead with `owner`, and that parks a technical question on the owner.
Decisions forbids that, and the decision comment on fx-9ag says this section
prevents it. The candidate is itself an example: it changed the Acceptance's
"end the session" to "end the run". That change is correct, but it is a
change to what done means, so the new clause would forbid it in an
unattended run.

Smallest fix: narrow the clause to the risk the bead names, for example
"weakening what done means is the owner's scope call; a technical correction
that keeps or strengthens the check is the agent's (see Decisions)". Or drop
the clause after the semicolon, which matches the bead's Acceptance exactly.

## Non-blocking notes

N1. **No explicit close gate.** Lines 60-63 forbid weakening the check in
order to close, but no rule says "close the bead only when the check passes".
Nothing in the section lets an agent close a bead whose check fails, and rule
3 sends that case to `blocked`. So the bead's test passes. Still, a common
unattended failure is to close with a note while the check fails, without
editing anything. Starting rule 1 with "Close the bead only when that check
passes" would remove that literal reading. If B1 is fixed, this fits in the
same sentence.

N2. **"without them" reads two ways.** In line 57, "These rules apply when
the owner has asked for a run without them", the nearest noun for "them" is
"These rules", which makes the sentence contradict itself. The trigger is
also unclear next to the owner's standing direction on fx-9ag ("Tide should
run autonomously"). An agent cannot tell whether that direction makes every
run autonomous. Suggested wording: "when the owner does not attend the run
(they asked for an unattended run, or the session runner started it)". The
stakes are low, because rules 1 and 2 are harmless in any run.

N3. **"Distinct approaches" is followable but loose.** The plural means at
least two, and AP-SESSION-001's budgets cap the upper end. The lower end is
the risk: two near-identical tries could end a bead's attempt early, and with
no `owner` label nothing brings that bead back (N4). A few words would make
it firm: "at least two approaches that differ in mechanism, not reruns or
tweaks of one".

N4. **A bead blocked without `owner` has no path back.** `br ready` leaves
out `status=blocked` (tested), and fx-3md's runner picks from `br ready`. A
bead that dead-ends on failed approaches, without the `owner` label, is
parked forever: no rule says who reopens it or when. This is in scope for
fx-3md (the runner's stop and selection rules), not this section. Note there
that `br blocked` lists dependency-blocked beads, not beads with status
`blocked`. In the scratch test it printed "No blocked issues" while the probe
bead was `blocked`. Use `br list --status blocked` to find parked beads.

N5. **Plateau versus budget.** "A plateau is not a stop. When progress
stalls, change approach" (line 64) could be read as overriding
AP-SESSION-001: from 400k "add no scope or new steps", and past 600k hand
off. The intro line says the rules add to AP-SESSION-001, so the budget wins
on a careful reading. Adding "(AP-SESSION-001's budgets still apply)" would
make that explicit for a stalled agent.

N6. **"End the run" is right, but the record does not mention it.** The
Acceptance says "end the session". The candidate says "end the run", which
matches AP-PLAN-001's terms: a run lasts until control returns, a writing run
owns at most one issue, and it returns at "completion, block, or handoff".
So after blocking one bead, the agent must return control and cannot pick up
another bead in the same run. That is the better wording. The decision
comment says the rules are stated "as the bead states them", though, and it
does not record this change.

N7. **The scope-call comment content comes from imports.** In rule 3's
owner-scope-call case there may be no attempts to report. The owner then
needs the choice named and a recommended answer, and AP-SCOPE-001 ("Recommend
and justify an answer to every choice") already requires both. That is fine,
because imports still bind, but "or the scope choice needed and a recommended
answer" would put it where the agent reads the rule.
