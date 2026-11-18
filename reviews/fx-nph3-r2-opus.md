Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 77f265d

# fx-nph3 r2: Opus converge round (report only)

Scope reviewed: `git diff main...77f265d` (scripts/autorun.sh, scripts/autorun-test.sh), read in full
alongside agent-scope.sh, the bead, AGENTS.md, GOVERNANCE.md (Merge Rule), the builder's report,
r1-codex.md and test.out. Environment: systemd 255, GNU timeout 9.4, choom present, cgroup v2.
Passed: `bash -n scripts/autorun.sh`; `bash -n scripts/autorun-test.sh`. Not run: autorun, its test,
or any scope or signal probe, as the brief instructs. trials/e1-checked/ was not opened.

## Verdict: approve

Confidence: high. The change keeps the launch shape. `timeout` stays the outer process-group
leader, and agent-scope.sh's chain `bash → exec systemd-run --scope → exec choom → exec CMD`
replaces images in one pid. So the session keeps timeout's pgid, its env and cwd, and fd 9. Only
its cgroup changes. The builder's experiment (probe pid = timeout pid + 1, pgid = timeout pid) and
Codex's reading of v255 run.c agree with that.

## Blocking findings

None.

## Non-blocking findings

1. **scripts/autorun-test.sh:157-159: the two-failures scope check can pass without checking the
   stalled session.** `scope_ended cgroups` reads only the last line of `cgroups`. With `TO=2s`, each
   launch now includes agent-scope.sh's startup before the stub's first line: two `systemctl --user
   show` calls, the stale-slice loop with up to two `systemctl` calls per aged drop-in,
   `set-property`, and systemd-run's StartTransientUnit wait. If that takes about 2 s under load, the
   stall session is killed before it appends its cgroup. The last line is then the `fail` session's
   scope, which has already ended, so the check passes and proves nothing about the timeout reaching
   the sleep. The same startup latency can also time out the `fail` session itself, giving
   "timed out,timed out" and a spurious FAIL. This is a new flake source: before this change the
   stub started with a bare exec.
   Minimum fix: `check_extra() { [[ -s stalling ]] && scope_ended cgroups; }`, and raise the
   scenario to `TO=5s`. That is still far below the stub's `sleep 30`, and costs about 3 s of test
   time.

2. **scripts/autorun-test.sh:231-232: two interrupt checks are redundant and one is unpolled.**
   `! kill -0 "$(cat stalling)"` runs once with no retry. The sleep is orphaned when the stub dies, and
   `kill -0` succeeds on a zombie until its new parent reaps it, so under load this can flake.
   `scope_ended` already polls for 5 s and covers every process in the session's scope, the stub and
   its sleep included. That makes both `! kill -0 …` and `! pgrep -f "$TMP/stub"` redundant.
   Concrete deletion: drop both and keep `grep -q "stopping: interrupted" … && scope_ended …`.
   Keep the `stalling` readiness wait, since it is what guarantees that the scope being checked
   holds the sleep.

3. **scripts/autorun.sh:277: an OOM kill under the cap is logged as "timed out".** With
   `OOMPolicy=continue`, the kernel kills the largest process in the slice. When that process is the
   session itself, rather than a runaway child, it dies of SIGKILL, timeout exits 128+9 = 137, and
   line 277 maps 137 to "timed out". The run's behaviour is unchanged, since both outcomes count
   toward the 2-failure stop. The log, though, misleads the owner about why the session ended, and
   the cap makes this case more likely than the global OOM killer did before. Minimum fix (text
   only): note in the header's Memory cap paragraph that exit 137 can also be the cap's OOM kill. A
   cheap mechanical alternative is to log "timed out or killed" for 137. Telling the two apart from
   the slice's `memory.events` is not worth it for this bead.

4. **Slices outlive the runner (lens 1; by agent-scope.sh's design, not this diff).** Scopes end
   with their processes and are collected (`CollectMode=inactive-or-failed`). Exceptions are
   detached leftovers, which the header and lock already cover. Each session's
   `agent-autorun_<id>_<pid>.slice` and its runtime drop-in stay until a later agent-scope.sh launch,
   at least a minute later, stops and reverts them. A real run leaves about one empty slice at its
   end. A full test run leaves roughly the last minute's worth, about 15-20 slices, so the next fleet
   agent launch pays about two `systemctl` calls for each. A fix in autorun would mean
   reconstructing agent-scope.sh's slice name and garbage-collection policy (AP-OWNERSHIP-001), so it
   belongs to eon-overseer if anyone wants it. No change is asked of this bead.

5. **autorun.sh:59-60: the header names only one of two uncapped fallbacks, and not where the
   warning goes.** agent-scope.sh also runs uncapped when `set-property` fails. Both warnings go only
   to the session's `$out.err`, never to the runner's console, so an operator who starts autorun
   where `systemctl --user` is unavailable gets uncapped sessions without seeing why. Text nit: say
   "Without user systemd, or if it cannot set the cap, agent-scope.sh runs the session uncapped and
   warns in the session's .err file." Making autorun check this itself would duplicate
   agent-scope.sh's policy, so don't.

6. **autorun-test.sh:242-245 and 249-251: a dependency failure shows up as a misleading FAIL.** On a
   machine without a readable default agent-scope.sh, the existing `lock` test fails with "second
   runner did not exit 3" (the runner exits 2 first). On a machine without user systemd, no-work,
   two-failures and interrupt fail instead of skipping. The header states both dependencies, and Tide
   runs this test only on this machine, so a skip is not required. This is noted for anyone who runs
   the test elsewhere.

7. **Nit: autorun.sh:94-95 vs 88.** The readability check runs after `mkdir -p "$STATE"`, so a
   failed start still creates an empty `.autorun/`. It is git-ignored and harmless, and the lock is
   correctly not taken. Moving the two lines above `mkdir` would make "starts nothing" literal.

8. **Nit: `AUTORUN_AGENT_SCOPE` (AP-MINIMAL-001, current callers only).** Its one in-repo caller is
   the no-scope test, which could equally run the runner with `HOME="$TMP"`: the runner exits at
   line 95, before git config or br matter. The knob is also how the builder ran the pass-through
   falsifier, and it costs two header lines, so keeping it is fine.

## Reasoning, per lens

### 1. Kill and cleanup paths

- `timeout` (not `--foreground`) calls `setpgid(0,0)` and forks the wrapper.
- `bash agent-scope.sh` execs systemd-run.
- In `--scope` mode, systemd-run registers its own pid as the scope over D-Bus and then execs choom,
  which execs the command, with no fork, setsid or setpgid.
- The session therefore stays timeout's direct child, in timeout's group:
  - timeout's TERM to the child plus `kill(0, TERM)`, and the `--kill-after` KILL, reach it.
  - The trap's `kill -TERM -- -$pid` (pid = timeout = pgid) reaches it.
- If the timeout or trap fires during wrapper startup:
  - before `set-property`: no slice is created and the wrapper dies;
  - after `set-property`: an empty slice drop-in is left for agent-scope.sh's GC;
  - during StartTransientUnit: systemd-run dies and the scope fails on a dead pid.
  In none of these cases is a process left running.
- No-systemd fallback (agent-scope.sh:34-36): `exec choom -n 500 -- CMD`, still in place, so the
  group and lock behave exactly as before. The same holds for the set-property fallback (51-53).
- A systemd-run failure after `systemctl --user show` has succeeded (for example, a process
  migration that is refused) makes the session exit non-zero. autorun logs it as failed and stops
  after two. That is visible and fails closed.
- Escapes: a descendant that calls setsid stays in the scope under the cap and holds fd 9, as the
  header now says correctly (Codex item 1 taken).

### 2. Interaction with autorun's logic

- **live_agent:** it runs only between sessions (in `pick`, and before `correct`), never while
  run_agent waits, so the wrapper chain is never seen live. Each exec replaces the comm, so the
  intermediate bash, systemd-run and choom never appear as claude or codex. A leftover claude in a
  scope is still found by its comm and cwd. The ancestor exclusion is unaffected, since it covers
  the runner's ancestors and not the session's.
- **fd 9:** nothing in the chain closes inherited fds; the sd-bus fds are CLOEXEC. This is proven end
  to end: `lock-held-by-leftover` now runs its `linger` session through the real scope and still
  gets exit 3.
- **Output:** every diagnostic from agent-scope.sh goes to stderr or /dev/null, and systemd-run runs
  with `--quiet`, so `$out` holds only the session's stdout. `charge`'s `jq -rR 'fromjson?'` is
  unaffected, and the budget scenarios pass through the scope.
- **Missing scope:** the exit at line 95 comes before `exec 9>` at line 97, and the no-scope test
  checks that no lock was created.
- **Unchanged behaviour:**
  - `${id:-correct}` is correct under `set -u`. `correct()` declares `local id=""`, which shadows the
    global id for run_agent.
  - Unit names are sanitized to `[A-Za-z0-9_]`, so a slice nests exactly one level under
    agent.slice.
  - systemd-run `--scope` does not expand `$` in arguments by default, so prompts pass through
    verbatim.

### 3. Test strength and flakiness

- **(a) no-work:** the regex checks the full slice and scope path, the bead or correct name, and the
  same pid in slice and scope, for all three sessions. That meets the bead's acceptance ("inside an
  agent-*.slice"). It does not read `memory.max`, which is agent-scope.sh's contract, so that is
  acceptable.
- **(b) two-failures:** it is strong when the stall session records its cgroup. Finding 1 closes the
  gap when it does not.
- **(c) interrupt:**
  - The readiness on `stalling` (Codex item 2) is correct: the sleep is forked inside the scope
    before the marker is written.
  - Signalling only after readiness means `scope_ended` checks a scope that really held the stub and
    its sleep.
  - The 15 s poll against the 20 s timeout is sound.
  - Finding 2 removes its one racy check.
- **Falsification:** the builder's pass-through run failed exactly (a), (b) and (c), so the new
  assertions detect a missing scope.
- **Leftover scopes:** a successful run empties every scope; the `sleep 47` and fake-claude leftovers
  are killed by their `check_extra`. A failed run can leave a `sleep 30` alive for at most 30 s.
  Slices behave as in finding 4.
- **Without user systemd:** finding 6.

### 4. Simplicity and leftovers

- The production change is the minimum: one setting, one fail-fast line, and a three-word array
  prepended to both launches.
- Concrete deletions:
  - the `! kill -0 "$(cat stalling)"` and `! pgrep -f "$TMP/stub"` conjuncts (finding 2);
  - optionally the `AUTORUN_AGENT_SCOPE` knob, in favour of `HOME="$TMP"` (finding 8; not
    recommended).
- No orphaned code, no stale comment (the run_agent comment points to the header), and both
  `.git/info/exclude` fixtures list the new `cgroups` and `stalling` files.

DONE
