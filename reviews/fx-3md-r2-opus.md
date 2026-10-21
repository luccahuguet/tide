# fx-3md review round 2 (Opus, fresh pass)

## Fixed

- `scripts/autorun.sh` `stop()`: the run listed blocked beads only when it stopped because no work remained. The fx-3md decision says "Blocked beads: listed when the run stops for the owner to review", so a stop on a cap, failures, or handoffs left the owner without the list. `stop()` now lists them on every stop. It calls `br` directly as a best effort, so it cannot recurse through `brj`. The no-work branch of the main loop shrinks to `[[ -n "$id" ]] || stop "no work remains"`.
- `scripts/autorun.sh` `dirty()`: removed the `':!.autorun'` pathspec. Tide's `.gitignore` already ignores `.autorun/`, so the pathspec repeated that rule.
- `scripts/autorun.sh` in-progress stop message and header: a session that fails or times out after claiming its bead leaves that bead in progress with no handoff. The next pick then stops and blamed "another session". That is the common case for a real timeout. The message now also names a session that ended without handing off, and the header says that this case stops the run. Behavior is unchanged: stopping is the safe choice, because the tree state is unknown.
- `scripts/autorun-test.sh` scenario setup: the scratch repositories git-excluded `.beads/` entirely, so nothing tested the `':!.beads'` pathspec. That pathspec matters in Tide, where `.beads/issues.jsonl` is tracked and every `br` write dirties it. The scratch repositories now commit `.beads/` as Tide does. Mutation probe: deleting the pathspec fails 7 scenarios.
- `scripts/autorun-test.sh` `blocked` scenario: it now stops on two failures after a bead blocks. That proves the blocked list appears on a stop other than no-work and that the blocked bead is not retried (picks `a b b`). Mutation probe: deleting the listing fails this scenario.
- `scripts/autorun-test.sh` `acts()`: replaced `$(eval echo "\$$1")` with `${!1}`.

## Left alone

- Sessions inherit lock fd 9. A session process that outlives its session would keep holding the lock. Claude ends its children on exit, and timeout(1) kills the process group, so the risk is speculative.
- `@{u}` also moves on a `git fetch`. Stopping on a fetch is conservative and fits "stops if the upstream ref moves".
- A bead handed off while still `open` (never claimed) is picked again in work mode rather than continue mode. AP-PLAN-001 has a session claim its bead before implementing, and the prime step reads the comments anyway.
- Other edge cases: a session that defers its bead logs `failed`. Malformed numeric env values make the run stop at once. Neither is a safety problem.

## Verification

- `timeout 400 scripts/autorun-test.sh </dev/null` on the final tree: 19/19 PASS, rc 0, about 22 s.
- Mutation probes on scratch copies outside the repository: removing `':!.beads'` failed 7 scenarios, and removing the blocked list from `stop()` failed `blocked`.
- shellcheck is not installed, so it was not run.
- `scripts/autorun.sh` was not run in the Tide repository.
