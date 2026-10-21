# fx-3md review round 5 (Opus, fresh-pass)

Diff: `git diff 76dfe06 -- . ':!.beads'` (.gitignore, README.md) plus the
untracked scripts/autorun.sh and scripts/autorun-test.sh.

## Fixed

1. scripts/autorun.sh:110-122 (lock leak). Sessions inherited the lock
   descriptor (fd 9). A process a session leaves behind, such as a setsid
   daemon or a detached background task, kept the lock after the runner
   exited. The next runner then exited 3 with "another runner holds the
   lock", which was false, and kept doing so for as long as that process
   lived. The Acceptance says the lock keeps out other *runners*. Probe in a
   scratch repository with the old script: the stub ran `setsid sleep 7 &`
   and closed its bead, and the second runner got rc=3. Fix: both agent
   invocations now close the descriptor (`9>&-`), and the comment above
   run_agent says why.
2. scripts/autorun-test.sh: added the stub action `linger`, which leaves a
   setsid process behind and closes the bead, and the scenario
   `lock-released`. After that run, a second runner must not exit 3. The
   header now lists the new action and `setsid`.
   Mutation (scratch copy outside the repository, `9>&-` removed):
   lock-released FAILS, and every other scenario still passes.
3. scripts/autorun.sh:106 (simplification). `dirty()` was a wrapper with a
   single caller, so I inlined it into pick.

## Left alone

- The same-bead condition in the 3-handoffs rule (`last_handoff`) is close
  to redundant: a handed-off bead is always continued next. It costs two
  lines and spells the Acceptance out literally, so I kept it.
- `stop` exits 0 for both error stops and normal stops. Nothing reads the
  exit status.
- When a session hands off without setting the bead in_progress, the next
  pick treats the bead as ready and sends "work" instead of "continue".
  Prime still reads the handoff comment, so this is harmless.
- The `--disallowedTools` / `--permission-mode auto` invocation matches
  `claude --help`: auto is a listed choice, and the patterns may contain
  spaces, as in the help's own example. It cannot be proven offline. The
  bead's "one real iteration" verification covers it.
- These were noted by earlier rounds and are still left alone: the
  repeated latest_comment call in continue mode, `git fetch` moving `@{u}`,
  and the case where the handoff skill splits a bead into an epic.

## Verification

- `bash -n` passes on both scripts. shellcheck is not installed.
- `timeout 400 scripts/autorun-test.sh </dev/null`: 19/19 PASS before the
  fixes, and 20/20 PASS with rc=0 after them, in about 24s.
- Mutation run of the test suite with the fix removed: lock-released FAILS
  (rc=1).
- I did not run scripts/autorun.sh in the Tide repository and made no
  commit.
