# fx-ic2.11 — Opus review, round 5

Model: Opus (CLI `--model opus`). Date: 2026-11-15. Read-only.
Candidate Git tree: ba5fe545bef4fa9b8aefd767ebcfcc98b5295fda

I've checked both blocking fixes against the archived `preflight.sh` and `heavy.sh`, so here is the verdict.

# Finder re-check of the round-4 Opus fixes, fx-ic2.11

**Verdict: approve.** Confidence about 0.85. Both blocking findings are resolved and nothing blocking remains. I ran nothing and edited nothing. I read the round-4 review, `opus-fix.diff`, the current `gate-report.md:408-568`, the archived `evidence/preflight.sh`, and `eon-overseer/heavy.sh`.

## B1: fixed

The command block is now `gate-report.md:503-527`.

- **Preflight passes the quiet hold.** The block requires `E1_GRANT_UNTIL`, sets `HEAVY_QUIET_EXEMPT=1` and exports all four booking variables (`E1_BOOKED_END_EPOCH`, `QUIET_LOG`, `HEAVY_QUIET_EXEMPT`, `E1_GRANT_UNTIL`), so the child `preflight.sh` sees them. Its branch at `preflight.sh:31` passes when three things hold:
  - the exemption is set;
  - `E1_GRANT_UNTIL` exactly equals the content of `quiet-until`;
  - `end <= q`. A 16:30 start with `MIN=14` ends at 16:44, before 16:45.

  If eon-overseer's guard has moved `quiet-until`, the equality fails and the attempt stops safely.
- **heavy.sh does not hold the run.** `heavy.sh:77` skips the quiet hold for an exported `HEAVY_QUIET_EXEMPT=1`.
- **One owner for the booked end.** `E1_BOOKED_END_EPOCH=$E1_GRANT_UNTIL`. The guard `now <= end − 600` lets the run start until 16:35. A preflight that runs its full 240 s from 16:31 still leaves time, and a late heavy.sh grant exits 75.
- **The prose matches the command.** Lines 471-475 and 529-535 tie the exemption to the recorded explicit grant.
- **Fails closed.** Under `set -euo pipefail`, a failed `timeout 240 … preflight.sh` stops the block before heavy.sh.

**Beyond the finding:** under `HEAVY_QUIET_EXEMPT=1`, heavy.sh also skips its first-come queue (`:34`) and the rule that a run leaves slot 2 to an older waiting Nix build (`:65`). That is heavy.sh's documented meaning for "the granted benchmark" (`:19-20`). It is the variable my finding asked for, used for the grant it names, so this is not a new trust-boundary crossing. The coordinator should still know the run jumps the queue during the window.

## B2: fixed

- Both blocks now set `REPORT_ROOT=$(git rev-parse --show-toplevel)`. In the regrade block it is computed at line 511, before `cd "$G"` at line 516.
- If the block runs outside a checkout, the failed command substitution stops the assignment under `set -e`.
- The literal worktree path now appears only in the provenance sentence of the "Executed 2026-11-15" paragraph (lines 438-441).

## Nonblocking findings and rebuttals

- **N1 (stale grade file):** the author bounded it as in my review: the sample's expected exits are all 0, and no path that skips writing a grade exits 0. Line 559 states that limit, and the sampled proof claims nothing beyond it. Accepted.
- **N2 (concurrency cited lines):** rebuttal accepted. `grader-reviews.md` is in the in-repo archive, its hash is recorded at `:303`, and `:163-165` say it holds all 31 entries with line citations.
- **N3:** resolved at `:559-561` (no failing structure verdict or integrity hold in the sample).
- **N4:** resolved at `:483-485` (a slow scan can use up the attempt).
- **N5:** resolved at `:494-499`. Stale readiness means rebooking; renewal compiles and needs a writer and a credential, and the window does not cover it.
- **N6:** resolved at `:488-490`. Any discrepancy that load or timing may have caused must be investigated and repeated under cleared load, and only matching stored grades satisfy acceptance.
- **N7:** deferral accepted. These were nits.
- **N8:** resolved at `:445-448` (helper tests, Bash syntax checks and diff check recorded).

## Remaining items, all nits

1. **The exemption can outlive the command.** If the coordinator pastes the regrade block into an interactive shell instead of running it with `bash` or in a `( … )` subshell, the shell keeps `HEAVY_QUIET_EXEMPT=1`. Later heavy.sh calls from that shell would then skip quiet holds and the queue after the grant ends. A note to run the block as a script or subshell (one clause) would close this. The coordinator can also just do that. Not blocking.
2. **Formatting:** `:490` is an over-long line and `:559` re-wraps poorly. Text only, no re-check needed.

## Scope of the changes

- The changes are text only, in `gate-report.md`. They do not change the helper, the test or the archive.
- No fix changes behavior beyond its finding. B1 adds the queue skip noted above, which is the documented meaning of the grant B1 required.
- No trust boundary is crossed beyond the grant the booking already records.
- The sampled regrade is still pending. I treated it as pending, as instructed, not as a defect.
