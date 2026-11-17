Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 09b7364
Verdict: request_changes

Confidence: high from source inspection. P1 and P2 #2 are fixed, and I accept the P2 #3 rebuttal. P2 #4 remains open because the final output-drain loop does not enforce its elapsed-time bound. The libc calls themselves are sound for the existing Linux callers.

Scope: re-checked only my finding 1 and its round-2 sub-findings. Read `git diff 0ffee88 09b7364`, the affected helper, callers and tests, `r2-recheck-codex.md`, `r3-fix-msg.txt`, updated `report.md`, and the three round-3 logs. HEAD is `09b73645f007a4cd5d71a193da41fb313441ebac`; the worktree was clean. No builds, tests, debugger runs, or subagents were started. Only this requested report was written.

## P1 — Signals cannot reach a reused PID: accepted

`crates/tide-demo/src/lib.rs:1172–1180,1203–1207,1223–1226,1237–1247,1275–1277` retains the direct child's waitable identity through group signalling. The helper no longer calls `try_wait`, reaps the leader early, signals individually scanned numeric member PIDs, or invokes an external kill program. `waitid(P_PID, ..., WEXITED | WNOHANG | WNOWAIT)` observes termination without releasing the leader's PID. That PID is also the PGID. The only signal targets the negative PGID while the leader remains owned and unreaped; reaping happens after the final group signal. No later signal uses the identity after it is released.

This closes the reported PID-reuse race under the existing callers' child-wait ownership. The repository has no competing generic child waiter or explicit SIGCHLD auto-reap policy. The [Linux wait documentation](https://man7.org/linux/man-pages/man2/waitid.2.html) confirms WNOWAIT retains waitable status.

## P2 #2 — No success accepted after the deadline: accepted

`lib.rs:1206` checks `Instant::now() < end` after the exit observation and complete group scan, immediately before accepting completion. A scan or scheduling delay that crosses the deadline therefore routes through timeout cleanup instead of `End::Exited`. Reaping an already-observed terminated child afterward does not retroactively accept a late completion observation.

I accept source inspection for this small ordering fix. The report explicitly states that no deterministic boundary test was added and explains the clock/pause seam it would require. The ordinary timeout tests are useful regressions, but are not presented here as proof of that precise scheduling boundary.

## P2 #4 — Bounded cleanup and truthful failure reporting: still blocking

The specific external-command hang is fixed. `lib.rs:1277–1282` uses the group kill syscall and checks its result; ESRCH is accepted, while other errors produce `End::NotCleaned`. The subsequent polling distinguishes an empty running group from survivors after five seconds. Step 5 handles `NotCleaned` separately (`lib.rs:966–970`) and no longer claims a failed cleanup killed everything. Not joining a reader blocked on a surviving writer also avoids an unbounded join.

**Remaining P2 blocker: the two-second output drain can run indefinitely under sustained output (`lib.rs:1229–1232`).** Its only stop condition is an error from `receive.recv_timeout(grace.saturating_duration_since(Instant::now()))`. After `grace`, this becomes `recv_timeout(Duration::ZERO)`, which still succeeds whenever a message is queued. The loop never tests the clock as a condition for continuing. A surviving writer can keep the reader/channel supplied, so the helper can continue extending `output` past its cleanup limit instead of returning its already-computed failure. A sufficiently large finite backlog also runs past the stated two-second bound.

Concrete case: the command forks a descendant that starts a new session, keeps stdout, and continuously writes, then the command exits. At the deadline, killing the original group succeeds, but the escaped writer keeps the channel supplied during the final drain. Such a writer is correctly outside the group-kill guarantee; that does not authorize an unbounded wait on its output. The same problem delays reporting `NotCleaned` when an actual cleanup survivor keeps writing. The report already acknowledges surviving writers, but considers only a blocked read, not a continuously ready channel.

This follows from the channel API, without a runtime reproduction: the [Receiver documentation](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.recv_timeout) describes a timeout on waiting for data, and the [unbounded-channel implementation](https://doc.rust-lang.org/src/std/sync/mpmc/list.rs.html) attempts to receive a queued message before checking the deadline.

Minimum correction: put an explicit `Instant::now() < grace` condition around the drain, before each receive, so queued data cannot bypass expiry. Keep the existing receive timeout for an empty channel. Add focused coverage of a writer that remains productive during drain, with test-owned cleanup for any escaped process. Update the report's unconditional claim that output collection lasts at most two seconds. This is a remaining defect within finding 1's bounded-cleanup scope, not a request to extend group membership or kill processes outside the contract.

## P2 #3 — Orphaned-zombie rebuttal and contract: accepted

I withdraw the demand that this helper adopt and reap orphaned descendants. The stated contract (`lib.rs:1159–1164`) is that no member of the target group remains running on `Exited` or `Killed`, not that every orphan has disappeared from `/proc`. The helper owns and reaps its direct child. Init or the nearest existing subreaper owns orphan reaping; this helper need not change process-wide adoption policy inside the parallel test executable.

The test helper's absent-or-zombie check matches that contract. The code and report explicitly retain the exception for descendants that leave the group, and the driver's sandbox still installs `PR_SET_PDEATHSIG` in `crates/tide-admit/src/invocation/sandbox.rs:126`. Accepting external reaping ownership does not assert that an arbitrary external reaper always reaps promptly.

## libc — Unsafe blocks, flags and errors: accepted

`libc = "=0.2.191"` matches tide-admit's pin; the lockfile adds only tide-demo's dependency edge. The report records the needed capabilities and std's missing non-reaping wait/group-signal operations.

In `exited_unreaped`, the initialized, properly aligned `siginfo_t` is valid storage for `waitid`. WEXITED selects termination, WNOHANG prevents waiting for a live child, and WNOWAIT preserves later reaping. Zero initialization makes checking `si_pid` sound even on a no-event return. The accessor is used only after a successful call. Failure returns `last_os_error` rather than interpreting invalid status data.

The kill block passes scalar arguments and dereferences no memory. A Linux child PID fits `pid_t`; negating it selects its process group. Errno is captured immediately on failure. As the [Linux kill documentation](https://man7.org/linux/man-pages/man2/kill.2.html) notes, group-call success means at least one signal was delivered, not necessarily all; the subsequent survivor scan is therefore necessary and is retained. I found no new FFI memory-safety or flag-semantics defect.

## Regression review, report and supplied proof

The `Bounded.status` to `Bounded.end` change reaches the production caller and all three direct regression tests. The early-exiting-command/left-behind-writer test is preserved and now requires `End::Killed`. Normal completion still retains output from the late writer and checks successful `End::Exited`.

The supplied `r3-focused.log` reports 3 passing bounded-run tests in 1.04 s. `r3-demo-test-gdb.log` reports 6 passing tests in 33.81 s, including the real debugger/replay output. `r3-workspace-test.log` reports 359 passing tests with no failed suite. These are supplied observations, not reruns. The debugger log identifies dirty `0ffee88`; association with amended `09b7364` relies on the builder's explicit statement that the tested content was unchanged before amendment. The logs show heavy.sh slot acquisition. My read-only `git diff 0ffee88 09b7364 --check` passed.

The logs do not exercise the sustained-output cleanup boundary or the `NotCleaned` cases. I found no additional regression in this fix beyond the drain defect above. Findings 2–4 from the original review were not reopened.

## New findings

One newly identified manifestation of existing P2 #4: queued or continuous output bypasses the final drain's time limit. No unrelated new findings. Approval needs the small explicit drain-expiry check and focused evidence for that path; the accepted PID, success-deadline and orphan-reaping decisions need not be redesigned.

Not checked: no runtime reproduction, builds, tests, or subagents. Remaining risk: cleanup can overrun its deadline and delay failure reporting while output continues.

DONE
