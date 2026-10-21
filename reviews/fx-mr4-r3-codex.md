Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**approve — high confidence within this narrow re-check.**

## Scope

Read the supervisor brief first, then `fx-mr4`, the prior reports, and `git diff 9268da8..HEAD` at candidate `500765e`. Reviewed the listed fixes and their surrounding runner/test behavior. No files changed; the worktree remains clean.

## Blocking

None. B1 and B2 are resolved.

## Non-blocking

No new findings.

## Reasoning

- **B1:** [correct():175](/home/lucca/pjs/lab/tide-fx-mr4/scripts/autorun.sh:175) checks `live_agent` before querying beads or dispatching `/correct`. `tenth-live` leaves a checkout-local worker running after session ten and requires exactly ten bead picks plus the live-worker stop. Removing the guard would add `/correct` and fail that assertion.

- **B2:** [correct():185](/home/lucca/pjs/lab/tide-fx-mr4/scripts/autorun.sh:185) handles post-run query failure locally, writes the required line, and then stops. `correct-br-error` fails only the post-run snapshot and requires exactly one line containing `bead ids unavailable (exit 2)` and `unreported`. Restoring the fatal query would lose that line and fail the test.

- **Accounting and scope:** `tenth` now checks the correction allowance of `0.5` and the following session’s allowance of `0.25`, detecting omitted correction charging. The local empty `id` gives the correction stub no stale bead and restores the caller’s value afterward.

- **Regression check:** `brj` retains its fatal-query behavior for existing callers; `brq` provides the recoverable path needed for logging. Trigger ordering, terminal stop rules, and counters remain intact. The README and deduplication wording match the implementation. No additional regression was found.

Both scripts independently passed `bash -n`; `git diff --check 9268da8..HEAD` passed. No further patch is needed.

Not run: the offline suite requires forbidden scratch writes; ShellCheck is unavailable; no real runner was invoked. Opus r2’s 27/27 is reported evidence.
DCG rejected runtime probes over unverifiable generated shell/redirection and an evaluation timeout. Real `/correct` evidence quality and deduplication remain unproven here.