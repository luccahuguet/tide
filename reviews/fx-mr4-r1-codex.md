Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.**

## Scope

Read the supervisor brief first, then AGENTS.md, README.md, both scripts in full, `fx-mr4`, `fx-3md` and its decisions, and the `/correct` skill and script. No files changed.

The initial prescribed fingerprint **matched**:

`b0bd663a8523fb28b3e3521f8900d95bc0b17498a5812cb56eff49fbbca98f8c`

At completion, it was:

`9d6cb3ad3320e0b4cef8e2e9d89ac756bcd40f6e8b4e03be2245b1b240afb30b`

The difference is the newly untracked `reviews/fx-mr4-r1-opus.md`. The tracked candidate still hashes to the expected fingerprint. I did not read that report.

## Blocking

1. **The periodic trigger bypasses live-agent exclusion.**
   [scripts/autorun.sh:269](/home/lucca/pjs/lab/tide-fx-mr4/scripts/autorun.sh:269)

   A tenth session can finish while a detached checkout-local Codex or Claude worker remains active. The next ordinary iteration would stop in `pick`, but `/correct` starts first. The inherited lock excludes other runners; it does not prevent this runner from launching overlapping sessions that write `.beads/`.

   **Smallest fix:** apply the existing live-agent preflight before starting `/correct`. Add a tenth-session regression where a live worker remains and assert that `/correct` never starts. An isolated dispatch probe reproduced the current ordering.

2. **A failed post-run bead query loses the required log line.**
   [scripts/autorun.sh:178](/home/lucca/pjs/lab/tide-fx-mr4/scripts/autorun.sh:178)

   After `/correct` completes and its cost is charged, `brj` can terminate the runner before line 182 writes anything. A database failure therefore leaves a paid run without its required audit record. The same applies to an error envelope or invalid response.

   **Smallest fix:** handle this post-run query failure locally, write one line with the known cost and exit status plus an explicit “bead IDs unavailable” error, then stop. Do not label the result `none`. Add a regression that fails only the post-run snapshot. A literal-function probe confirmed the missing line.

## Non-blocking

- [scripts/autorun-test.sh:21](/home/lucca/pjs/lab/tide-fx-mr4/scripts/autorun-test.sh:21): the correction stub never records its allowance. Existing budget scenarios exhaust the budget before `/correct`, so they cannot detect omitted correction charges or an excessive correction allowance. One focused scenario should check its reduced allowance and the following session’s remaining allowance.
- [README.md:183](/home/lucca/pjs/lab/tide-fx-mr4/README.md:183): say “every 10th session **of each run**.” Resetting the counter satisfies an upper bound of “at most once every 10 sessions otherwise”; it provides no periodic guarantee across repeated short invocations.

## Reasoning

The new trigger tests are meaningful: they check exact invocation sequences for initial idleness, the tenth session, and suppression of a second correction immediately after the tenth. Removing or shifting the trigger would change those sequences.

The shared `run_agent` preserves permissions, disallowed tools, timeout, closed stdin, signal handling, and lock inheritance. Correction runs do not change session, failure, or handoff counters. Existing terminal stop rules retain precedence.

Cost accounting is correct on the inspected paths. Literal helper probes confirmed remaining-budget clipping, reported-cost charging, full-allowance charging when cost is absent, and refusal after exhaustion. There is no `set -e`; nonzero arithmetic tests and agent exits therefore do not unexpectedly abort execution.

Comparing ID sets correctly avoids treating edits, closures, or reopenings as newly filed beads. The installed `br` documents an unlimited default result count. The post-query failure remains the logging defect described above.

Deduplication belongs to the skill’s step 6. The runner’s ID comparison only reports new IDs; it cannot identify duplicate mistake classes. An additional runner mechanism is not justified without a demonstrated skill failure. However, the stub suite does **not** prove the required consecutive unchanged-history behavior or transcript evidence. Supply that real-run acceptance evidence before closing the bead.

Bash syntax and whitespace checks passed.

Not checked: the full offline suite requires scratch writes forbidden by this sandbox; ShellCheck is unavailable.
Risk still unproven: real `/correct` evidence quality and consecutive-run deduplication.