# fx-3md review round 4 (Opus, fresh-pass)

Diff: `git diff 76dfe06 -- . ':!.beads'` (.gitignore, README.md) plus untracked
scripts/autorun.sh and scripts/autorun-test.sh.

## Fixed

1. scripts/autorun.sh:183-184 (the 3-handoffs rule). The rule treated any commit
   as progress (`range == none`). In Tide `.beads/issues.jsonl` is tracked, and
   beads-only commits are normal here (2d82c40, c336285). The handoff skill
   flushes Beads state and leaves a known tree, so a session that hands off
   and commits only the handoff comment reset the counter. That meant the
   Acceptance stop "handed off 3 times in a row without a new commit" might
   never fire. Progress now means a net change outside `.beads/` between the
   before and after HEADs (`git diff --quiet ... -- . ':!.beads'`). This
   matches `dirty()`, which already ignores `.beads/`. The log's range column
   still shows the real commit range. The header comment (line 16) now says
   "no new commit outside .beads/".
2. scripts/autorun-test.sh: the three-handoffs scenario now has its second
   session commit only `.beads/` (new stub action `beads-handoff`). A
   check_extra asserts that the log's range column is `none,<range>,none`, so
   the seed cannot pass vacuously. The stub's `commit-handoff` now commits a
   real file rather than an empty commit, since an empty commit is no longer
   progress.
   Mutation check: in a scratch copy with the old `range == none` rule,
   three-handoffs FAILS (the runner runs 5 sessions instead of stopping at 3).
   With the fix, it passes.
3. scripts/autorun-test.sh:62,96 (flaky timeout). Every scenario ran under
   AUTORUN_TIMEOUT=2s. On the first run, at load average 13, `session-cap`
   failed because the stub's two br writes did not finish in 2s: the session
   timed out mid-handoff and left the bead in progress. Scenarios now default
   to 30s (`TO`), and only two-failures, which needs a real timeout, uses
   `TO=2s`. Total runtime is unchanged at about 1 minute.

## Left alone

- pick's `for b in $ids; do ...; return; done` idiom for "first id" works,
  so I left it as a style point.
- The runner still does not check for a dirty tree before a continue
  session. That is the recorded decision ("before new work").
- `stop` exits 0 for both error stops and normal stops. Nothing consumes
  the exit status yet.
- The real `claude` invocation (`--permission-mode auto` is a listed
  choice, `--disallowedTools ... --` before the prompt) cannot be proven
  offline. The bead's "one real iteration" verification still covers it.

## Verification

- `timeout 400 scripts/autorun-test.sh </dev/null` before the fixes: 18
  PASS, 1 FAIL (session-cap, timing). After the fixes: 19/19 PASS, run twice
  (rc 0).
- Mutation (scratch copy outside the repo, old rule restored): three-handoffs
  FAILS, as it should.
- `bash -n` on both scripts passes. shellcheck is not installed.
- I did not run scripts/autorun.sh in the repository and made no commit.
