# fx-3md review round 6 (Opus, fresh eyes, blocking only)

Diff: `git diff 76dfe06 -- . ':!.beads'` (.gitignore, README.md) plus the
untracked scripts/autorun.sh and scripts/autorun-test.sh. No commit made;
scripts/autorun.sh was not run in the Tide repository.

## Blocking fixes (scripts/autorun-test.sh only)

Two Acceptance rules were untested. I proved this with mutations on scratch
copies outside the repository. With the old test, each mutation passed 20/20:

1. The session prompt. Acceptance: "prime, then continue <id> from its
   handoff" or "prime, then work <id>". handoff-first checked only the
   runner's ` (continue` status line. Mutation: the continue prompt
   replaced by "prime, then work $id." Result: 20/20 PASS.
   Fix: the stub appends its `$1` to `prompts` (git-excluded like
   `picked`/`caps`), and handoff-first gets a check_extra that requires
   prompt 1 to start with "prime, then continue $b from its handoff. " and
   prompt 2 to start with "prime, then work $a. ".
2. The log's bead column. Acceptance: the log line holds time, bead,
   outcome, and range. Only the outcome column (f3) and, in two scenarios,
   the range column (f4) were checked. Mutation: the logged `$id` replaced
   by a constant. Result: 20/20 PASS.
   Fix: whenever `<outcomes>` is given, scenario() now also requires the
   log's bead column, mapped to a/b, to equal `<picks>`. The FAIL line
   prints it. The header and the usage comment now say so.

After the fix, mutation 1 fails handoff-first and mutation 2 fails all 11
scenarios that check outcomes. The real scripts pass 20/20.

## Non-blocking notes (not changed)

- autorun.sh:112-114. The run_agent comment still says "The session does
  not inherit the lock (fd 9), so a process it leaves behind cannot keep
  the next runner out". This is stale from round 5's reverted `9>&-`. It
  contradicts both the code and the design note at lines 63-64. Change it
  to match the kept design.
- autorun.sh:175. rc 137 covers timeout(1) escalating to KILL, and it also
  covers an external SIGKILL of the agent, such as the OOM killer. The
  second case is logged as "timed out" rather than "failed". Both count
  the same toward the 2-failures stop.
- After a timeout, a grandchild that ignores TERM can outlive timeout's
  exit. It can then overlap the next session of the same runner, because
  the inherited lock only keeps out the next runner.
- `--disallowedTools 'Bash(git push:*)'` does not match forms like
  `git -C . push` or `env git push`. The upstream check only sees a push
  that moves `@{u}`. The auto-mode classifier is the other guard. Earlier
  rounds accepted this.
- claude checks `--max-budget-usd` between turns, so a session can overrun
  its cap, and the total, by one turn. A reported cost is charged exactly,
  so the next iteration stops.
- Nothing checks the log's time column for format. Only its position is
  pinned, through the column checks.

## Check results

- `timeout 400 scripts/autorun-test.sh </dev/null` before the change:
  20/20 PASS, rc=0.
- After the change: 20/20 PASS, rc=0.
- Mutation runs with the new test, on scratch copies:
  - prompt mutation: rc=1 (handoff-first FAIL).
  - bead-column mutation: rc=1 (11 FAIL).
- `bash -n scripts/autorun-test.sh` passes. shellcheck is not installed.
