# fx-3md round 1 (Opus, read-only): session runner `scripts/autorun.sh`

Candidate: `git diff 76dfe06 -- . ':!.beads'` (`.gitignore`, `README.md`) plus untracked
`scripts/autorun.sh` (142 lines) and `scripts/autorun-test.sh` (88 lines). Reviewed 2026-10-21.

## Verdict

**revise**: 3 blocking, 11 non-blocking.

The runner is compact and mostly correct. `br` JSON shapes match br 0.7.2: `list` returns
`{issues:[...]}`, `ready` returns an array, `show` returns an array, and comment `id`s are
integers. `br ready` leaves out in_progress and deferred beads by itself. The timeout exit
statuses 124/137 are right, the lock works, and the runner never pushes. Three problems block:
the budget cap undercounts, the test does not prove the "latest comment" selection rule, and
nothing stops a session from starting in a dirty or shared working tree.

## Blocking findings

### B1. The budget cap undercounts spend (`scripts/autorun.sh:107,110-111`)

There are two separate holes:

1. Line 107 sends stderr into the same `.out` file (`>"$out" 2>&1`). On line 110, `jq` reads that
   file as a JSON stream and aborts at the first non-JSON line. If any warning or hook text comes
   before the result object, the cost is never seen and the session counts as $0. I checked this:
   `printf 'warning\n{"total_cost_usd":1.5}\n' | jq 'select(type=="object")|.total_cost_usd'`
   prints nothing and exits 5.
2. A session that times out or gets killed prints no result object, so it also counts as $0.
   Timed-out sessions are likely the most expensive ones. With the defaults, failures alternating
   with successes can spend up to `MAX_SESSIONS × SESSION_USD` = $200 against a $100 "cap".

Smallest fix: parse leniently with `jq -rR 'fromjson? | select(type=="object") | .total_cost_usd // empty'`,
or write stderr to its own file. When the real `claude` reports no cost, charge
`$SESSION_USD`, which is the most `--max-budget-usd` allows that session to spend.

### B2. The test does not prove the "latest comment is a handoff" rule (`scripts/autorun-test.sh:66-67`)

This rule is what keeps the runner off an in_progress bead that a session has resumed and commented
on since the handoff. In `handoff-first`, bead `a` has no comments at all. A mutant whose `pick`
continues any in_progress bead with *any* handoff comment (line 65 replaced by
`jq 'map(select(.text|test("^Handoff")))|last // null'`) passes all 9 checks.

Smallest fix: in `handoff-first`, give `a` an older handoff and then a newer non-handoff comment,
for example `br comments add $a "Handoff (old)."; br comments add $a "Resumed."`. The expected
picks stay `b`.

### B3. Sessions start in a dirty or shared working tree (`scripts/autorun.sh:87-107`)

The bead's Evidence says "the runner must not start one while another works here". The flock
only keeps other *runners* out. Nothing checks for:

- a session killed by `timeout` that left half-made edits. The next iteration, often on a
  different bead because the killed bead is now in_progress with no handoff, starts on top of
  those edits and may commit them under the wrong bead;
- an interactive Claude or Codex session working in this checkout. The selection rules miss it
  while it is still priming, before it claims a ready bead or comments past the old handoff;
- a session in another worktree. Its claims live in that worktree's `.beads/`, so this
  checkout's `br ready` cannot see them.

Smallest fix: before each session, stop with a reason when
`git status --porcelain -- . ':!.beads'` is non-empty, which also catches the timeout leftover.
Also state in the header and README that the lock covers runners only, so the owner should not
work in the checkout or on its beads while a run is going. Detecting processes would cost more
and is optional.

## Non-blocking notes

1. **Ctrl-C does not stop a run** (`autorun.sh:79,81`). Without `--foreground`, `timeout(1)`
   moves itself into its own process group, so terminal SIGINT never reaches the session. Bash
   waits, sees a normal child exit, and starts the next iteration. I confirmed this: SIGINT sent
   to the runner's group left `timeout … sleep 6` (in its own pgid) running, and both iterations
   completed. To fix it, run the session in the background, `wait` on it, and use
   `trap 'kill -TERM -- -$pid; stop interrupted' INT TERM`. Alternatively, accept this and document
   `kill -- -<pgid>`. Closing the terminal kills the runner but leaves the session running, still
   holding fd 9. That is protective, because it keeps a new runner out until the orphan ends.
2. **Two stop qualifiers are unproven.** A mutant that never resets `failures` (line 131) passes,
   so "2 failures *in a row*" is untested. One added scenario fixes that:
   `acts a fail close; acts b fail close`, expecting "no work remains" and picks `a a b b`. A
   mutant that drops the `$last_handoff` same-bead comparison (line 136) also passes. Because
   `pick` always resumes the first handed-off bead, that comparison is close to unreachable.
   Either drop it or accept it as cheap insurance.
3. **The `.id > $b` freshness guard is unproven** (line 122). A mutant without it passes. A
   continued session that adds nothing would then be logged "handed off" when it should be "no
   progress". A `continue` + noop scenario would cover it.
4. **The exit status outranks br state** (lines 118-121). A session that closes or hands off its
   bead and then exits non-zero is logged "failed", for example after reaching
   `--max-budget-usd` at the end. Implementation note 3 says to judge outcomes from br and git.
   Consider checking closed/blocked/handoff first and using `rc` only to classify the rest.
5. **Push prevention is only a prompt line** (line 101). The runner itself never pushes, but a
   session is only asked not to. Cheap defenses: `--disallowedTools 'Bash(git push:*)'` on the
   `claude` call, and/or stopping when `git rev-parse @{u}` changes across a session.
6. **Bead record is still open.** The Verification item "one real iteration … one log line, and
   no push" has not been done (no `.autorun/` exists). The fx-3md decision (adopt) and the
   permission choice (`auto`, not `bypassPermissions`) are not on the bead yet; its only comment
   is the fx-9ag note. Record both before closing.
7. **fx-9ag N4 is only half answered.** The runner lists `status=blocked` beads when no work
   remains, but no text says who revisits them or when. One line in the header or README would
   close it, for example: the owner reviews the listed beads, and the runner never retries them.
8. Nit: the header (line 7) leaves out "not deferred". `br ready` already guarantees it ("open,
   unblocked, not deferred"), so saying so would match the Acceptance wording.
9. Nit: the array-or-object hedge in `status_of` (line 55) is speculative. `br show --json`
   always returns an array.
10. Nit: `AUTORUN_AGENT` is quoted, so it must be a single executable path, not a command with
    arguments. The header's "`$AUTORUN_AGENT <prompt>`" reads as if word splitting applies.
11. Nit: session log names have one-second resolution, so a quick repeat failure on the same bead
    overwrites the earlier `.out`. A `br` error inside `pick`, such as lock contention, also ends
    up as "no work remains": the run still stops safely, but for the wrong stated reason.

## Check results

- `timeout 300 scripts/autorun-test.sh </dev/null` on the candidate: 9/9 PASS (no-work,
  handoff-first, three-handoffs, commit-resets, two-failures, no-progress-skip, session-cap,
  budget-cap, lock). Exit 0, 15 s, offline.
- `shellcheck`: not installed, so not run.
- br 0.7.2 JSON shapes were checked in a scratch repo, and they match what the script assumes.
- Mutation runs, each on a scratch copy of both scripts:

| Mutant | Rule | Result |
| --- | --- | --- |
| `max_by` → `min_by` (latest comment) | selection and outcome | caught |
| `pick` accepts any handoff comment | latest-comment selection | **survives** (B2) |
| handoff counter ignores "no new commit" | 3-handoff stop | caught |
| handoff counter ignores "same bead" | 3-handoff stop | survives (note 2) |
| failures never reset | 2-in-a-row stop | **survives** (note 2) |
| outcome ignores `.id > $b` | handoff outcome | survives (note 3) |

- Checked empirically: `jq test("^Handoff")` anchors at the start of the string, not each line.
  A non-foreground `timeout` runs in its own process group (note 1).
- `scripts/autorun.sh` was not run against the Tide repository.
