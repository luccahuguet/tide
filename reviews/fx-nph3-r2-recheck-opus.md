Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate cdf18b8 (re-check of 77f265d items)

# fx-nph3 r2 re-check (Opus finder)

## Scope and checks

- Scope: `git diff 77f265d cdf18b8 -- scripts/`.
  - It touches only scripts/autorun-test.sh, 6 lines.
  - scripts/autorun.sh is unchanged.
- Worktree HEAD is cdf18b8, and scripts/ is clean.
- `bash -n` passes on both scripts.
- I did not run the test.

## Verdict: approve

Both items are resolved, and the fixes broke nothing.

## Item 1: two-failures stall scope (r2 finding 1). Resolved.

The change is `check_extra() { [[ -s stalling ]] && scope_ended cgroups; }` with `TO=5s`.

- Only the stall action writes `stalling`, and it does so after the stub's first line has appended
  its cgroup.
  - So a non-empty `stalling` proves that the stall session recorded its scope, and that its sleep
    was forked inside that scope.
  - The `fail` action exits before it could write `stalling`.
  - The run stops after the second failure, so no later session can append another line.
  - So `tail -1 cgroups` is the stall session's scope.
- `stalling` is relative, and check_extra runs with cwd `$dir`, where the stub writes it
  (`STUB_DIR=$dir`). It was already listed in that fixture's `.git/info/exclude`.
- 5 s gives wrapper startup headroom and stays far below the stub's `sleep 30`.
- The expected outcomes "failed,timed out" and the stop text are unchanged.
- The new comment matches the check.

## Item 2: interrupt redundant checks (r2 finding 2). Resolved.

- The unpolled `! kill -0 "$(cat stalling)"` and `! pgrep -f "$TMP/stub"` are gone.
- `grep -q "stopping: interrupted" … && scope_ended "$TMP/interrupt/cgroups"` remains.
  - `scope_ended` polls for up to 5 s.
  - It requires the path to be an autorun scope.
  - It covers every process in that scope.
- The readiness wait on `stalling`, and its separate startup-failure branch, are kept unchanged. So
  the scope being checked is known to hold both the stub and its sleep, and the dropped checks lose
  no coverage.
- The test header ("an interrupt empty its scope") still describes what the test checks.

## Nit, no re-check needed

- The stub's comment says `stalling` "holds its pid", but nothing reads that pid any more; only the
  file's presence matters.
- Writing the pid is harmless and still useful when debugging.

DONE
