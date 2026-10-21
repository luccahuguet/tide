# fx-mr4 review, round 1 (Opus)

Reviewer: Opus, round 1, report-only converge round (GOVERNANCE.md first
round, parallel with the Codex pass).
Candidate: worktree `tide-fx-mr4`, branch `fx-mr4`, base `9268da8`; diff
fingerprint `b0bd663a8523fb28b3e3521f8900d95bc0b17498a5812cb56eff49fbbca98f8c`
(the same before and after this review, not counting this file).
Date: 2026-10-21.

## Verdict

Clean. Nothing blocks. The trigger fires only at the acceptance points, no
existing behaviour regresses, and the bead diff and cost accounting are sound.
The non-blocking items are one inaccurate comment, risks the bead's real run
must check, and small gaps in the tests.

## Blocking

None.

## Non-blocking

- **N1. The comment about duplicates is wrong, and the gap in dedupe is real**
  (`scripts/autorun.sh:38-39`). The header says "The skill keeps out duplicates
  by searching existing beads before it files." `br search` leaves out closed
  beads by default; in a scratch repository, `br search x` returned `[]` and
  `br search x --status all` returned `["closed"]`. `correct` files `decision`
  beads that `br ready` returns, so the runner itself picks them and may close
  them. Transcript history only grows, so a later `/correct` run sees the same
  corrections again, finds no open bead, and files the class again. That can
  happen within one run: correct after session 10, the bead closed in
  session 11 or 12, then correct again when no work remains. The bead's literal
  acceptance (unchanged history) still holds, but the comment overstates the
  guarantee. Fix: reword the comment ("searches open beads"). Report to
  Starcompass that `correct` step 6 should search every status
  (`br search --status all`) and treat a closed class as already decided,
  which is what fx-mr4's scope asks for.
- **N2. Runner prompts may count as owner corrections (unproven).** Every bead
  session's prompt ends "This run is unattended: … Do not push." If
  `owner_messages.py` sees `claude -p` prompts as origin kind `human`, the
  prompt repeats across sessions, and the skill counts "a restated
  instruction" as a correction. A bogus "agents push" class would follow. I
  could not check the transcript `origin` field: the classifier refused reads
  of `~/.claude/projects`. The bead's real-run Verification has to check this.
  `/correct` without arguments is already excluded.
- **N3. The tests leave three things unchecked.** No test checks that the
  correct run's cost reaches `spent` and therefore the budget cap, the
  `unreported` cost path, or the `(exit N)` suffix. Mutations that drop these
  pass. One scenario, for example `BUDGET=1.2` with a correct run between
  sessions, or a stub correct action that prints no cost and exits 1, would
  cover them.
- **N4. Nit.** `run_agent` passes `AUTORUN_BEAD="$id"` to the correct run, so
  the stub sees the previous bead's ID, or an empty one. This is harmless
  because the stub branches on `$1`.
- **N5. Note.** With the default `AUTORUN_MAX_SESSIONS=10`, every full run
  ends with a correct session, adding up to $20 and 3 hours before the cap
  stop. The 10-session count resets each run, so an idle runner started on a
  schedule runs `/correct` every time. This matches the acceptance and is
  documented. Recorded here only so the owner knows the cost.

## Checks

- Fingerprint before and after:
  `b0bd663a8523fb28b3e3521f8900d95bc0b17498a5812cb56eff49fbbca98f8c`.
- `scripts/autorun-test.sh </dev/null` in the worktree: 25/25 PASS, exit 0,
  2m16s.
- `br list --help`: the default limit is unlimited. In a scratch repository,
  `br list --status all --json` returned blocked, open, deferred, and closed
  beads, so the before and after diff of IDs sees every bead `correct` files.
- `br search` in the scratch repository: closed beads are excluded by default
  (N1).
- shellcheck: not installed; not run.
- Not run: the real `scripts/autorun.sh` and any real `claude` session.

## Reasoning

**Trigger.** After a session, `((sessions % 10)) || correct` runs only after
sessions 10, 20, …, and only when none of the checks before it (upstream,
2 failures, 3 handoffs) stopped the run. On no work,
`((sessions == 0 || sessions % 10)) && correct` covers both the idle start and
any count that is not a multiple of 10. It skips the stop right after a
10th-session run, which can only have happened at that count, because a
skipped run means `allowance` failed, and then the budget stop at the top of
the loop fires first. The session cap, budget, failure, handoff, upstream,
dirty-tree, live-agent, in-progress, and br-error stops never call `correct`.
The tests pin each point: `no-work`, `idle`, `tenth` (fires after session 10,
not at cap 12), and `tenth-then-idle` (no second run). Every scenario that
stops for another reason lists no `/correct` pick. Changing the modulus,
dropping the `sessions == 0` guard, or making the no-work run unconditional
would each fail a named scenario.

**Regression.** `allowance` and `charge` move the old inline code without
changing it. `remaining` becomes local, and nothing else read it. The lock,
outcome judging, the session-log format, and the stop rules are untouched,
and all of the earlier scenarios still pass. A correct run inherits fd 9 and
the INT/TERM forwarding through `run_agent`, and it checks the upstream ref as
a session does.

**Cost.** `allowance` gates the correct run and caps it, `charge` adds its
cost, or its whole cap when it reports none, to `spent`, and the next
`allowance` sees that spend. This is correct, but untested (N3).

**Simplicity.** The two helpers each have two callers, and `correct` is about
20 lines. I found nothing simpler that keeps the separate log line and the
bead diff. Mixing `/correct` lines into the session log, with cost in
column 4 where sessions log a range, is what the acceptance asks for, and
column 2 tells the two apart.
