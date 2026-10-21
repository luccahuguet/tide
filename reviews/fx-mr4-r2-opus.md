# fx-mr4 review, round 2 (Opus)

Reviewer: Opus, round 2, converge round, base `9268da8`, 2026-10-21.
Candidate fingerprint (taken before adding this file):
`3bad8ad9de5b6deaa6e639620c529dd4c2689628e99c598d30c1b1d940b8e05d`.

## Verdict

Clean. This round changed nothing.

## Changes

None.

## Reasoning

- **Trigger.** `((sessions % 10)) || correct` (`scripts/autorun.sh:280`) runs
  only after sessions 10, 20, and so on, and only when no earlier stop in that
  iteration fired. `((sessions == 0 || sessions % 10)) && correct` (line 227)
  covers the idle start and every count that is not a multiple of 10. The
  multiple-of-10 case is safe to skip: a correct run either ran at that count,
  or it was skipped because `allowance` failed (then the budget stop at the
  top of the loop fires first) or because `live_agent` found a worker (then
  `pick` stops first). The scenarios `no-work`, `idle`, `tenth`,
  `tenth-then-idle`, and `tenth-live` pin each point, and every scenario that
  stops for another reason has no `/correct` pick.
- **Round-1 fixes.** The live-agent check comes before the correct run
  (line 175, `tenth-live`). A post-run br failure still logs the paid run with
  "bead ids unavailable", its exit status, and its cost, then stops
  (lines 185-195, `correct-br-error`). The correct run's allowance and charge
  are checked (`tenth`: caps `... 0.5 0.25`). The README says "of each run".
  The header now says the skill may refile a class whose bead was closed.
- **Scope of `id`.** `local ... id=""` in `correct` gives `run_agent` an empty
  `AUTORUN_BEAD`. The stub exits 9 otherwise, and the `none` and filed-ID
  checks would then fail.
- **Simplicity.** `allowance` and `charge` each have two callers. `brq` exists
  because the post-run query must not stop before the log line. I found
  nothing to delete or inline without losing a required behaviour.

## Non-blocking, left alone

- `scripts/autorun-test.sh:113-114`: the comment "The bead it files leaves
  .beads/ dirty, which keeps no later session out" is true of the runner, but
  `tenth` does not exercise it, since session 11 continues a handed-off bead
  and the dirty-tree check applies only to new work. `no-work` does protect the
  `.beads` exclusion, because `b` is picked as new work after `a`'s close
  dirties `.beads/`. This is a nit, so I left it.

## Checks

- `scripts/autorun-test.sh </dev/null` in the worktree: 27/27 PASS, exit 0,
  2m41s.
- `nix run nixpkgs#shellcheck -- scripts/autorun.sh scripts/autorun-test.sh`:
  `autorun.sh` is clean. The base's one SC2015 in `brj` is gone with the `brq`
  split. `autorun-test.sh` reports SC2218, SC2016, SC2001, and SC2329, the same
  kinds as at base `9268da8`, so there is no new kind of finding.
- Not run: the real `scripts/autorun.sh` and any real `claude` session.
