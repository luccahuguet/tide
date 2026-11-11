Model: claude-opus-5-5 · Effort: low · Date: 2026-11-11

Verdict: approve

Candidate: `85b9e20` on `65aee6c`, worktree clean. `git diff 65aee6c 85b9e20 | sha256sum` =
`b7322858575d8287384a1f4759cb7dd2c66d7537ffbd5ce7dd6ef80e9b9b5f94`, which matches the frozen
candidate. I read `git diff 8a7a285 85b9e20` (task.rs, scheduler.rs, 03) and the beads `fx-o8j` and
`fx-o4h` with `br --no-db`. I ran no build or probe and did not open `trials/e1-checked/`.

## Finding 1 (stall counter reset): resolved

- `Scheduler::wake` now resets `stalled` right after it removes the waiter's timer, when the map is
  empty (crates/tide-std/src/task/scheduler.rs:191–195). The reset sits where I asked: it covers a
  cancel followed by a new `block` with no point in between, which a reset in `point` would miss.
- The timer map has only two removal paths: `wake` (scheduler.rs:192) and the jump in `pick`, which
  already resets (scheduler.rs:300). `abandon` and `finish` never hold a timer: an abandoned spawn
  never blocked, and a task finishes only after a wake has removed its timer. So the counter now
  counts exactly the consecutive points with a timer pending and no advance, as research §2 item 4
  and 03:1777–1780 define it. If another timer is still pending after an early wake, the count
  carries on. That is correct, because a timer waited the whole time.
- The regression (scheduler.rs:458–490) runs `stall_limit = 3` points, wakes the first sleeper
  without a jump, blocks a second sleeper with no point in between, and then shows that the second
  sleeper gets all 3 points and the 4th point reports a stall, with `now` still zero. Before the fix,
  `stalled` would be 3 there, so the second sleeper's first point would return `Err` and the
  `unwrap` would fail. The author's red log records exactly this. The test falsifies the defect.
  The `scheduler.cancel` call only writes to the trace and is harmless.

## Finding 2 (wait-primitive comment and lock order): resolved

- The comment at crates/tide-std/src/task.rs:343–350 no longer promises that a later wait kind
  only adds an `Operation` and a `wait_done` arm. It now says what later beads must change:
  - readiness becomes a caller-supplied check, or the resource keeps its own waiter handles;
  - fx-hxw needs a Thread-or-eventfd waker;
  - the lock order is fixed as runtime, then resource;
  - a resource keeps its own waiter Thread handles and never takes the runtime lock while it
    holds its own lock;
  - both locks are released before parking.

  No abstraction was added (AP-MINIMAL-001).
- The order holds up on both sides:
  - Real side: the waiter registers in the resource and checks it while holding runtime and then
    resource. A releaser takes only the resource lock and unparks its own handles, so no ABBA
    deadlock is possible and no wakeup is lost.
  - Sim side: a releaser that must call `Scheduler::wake` takes runtime first, which is the same
    order.
- The stale inline comment in the real branch (the old "take runtime then resource" line) is gone.
  Nit 8's sentence replaces it (task.rs:361), so the order is stated in one place.
- fx-o4h records the order with its date and source (comment 2026-11-11 07:14 UTC) and says it
  replaces decision 3's "channel then runtime". The ABBA conflict I raised is closed in planning
  as well.

## Introduced by the fixes

Nothing blocking. Two nits, neither of which needs a re-check:

- task.rs:361 is about 104 columns, over the 100 that rustfmt uses for code. rustfmt doesn't wrap
  comments and its check passed, so this is cosmetic.
- "A resource keeps its own waiter Thread handles" sits next to "fx-hxw also needs a
  Thread-or-eventfd waker". "waiter handles" would read more cleanly for the eventfd case. The
  meaning is clear either way.

I glanced at nits 6–8 without re-checking them:
- 03:1779 now names the task whose point passes the bound;
- `event` now writes `result`/`until` only for `cap` events (scheduler.rs:364), and the new test
  at scheduler.rs:495 covers all four operation shapes;
- nit 8's sentence is present.

Nothing there regresses the trace format that the existing tests check, and the workspace proof on
85b9e20 (128 passed, conformance 216/24/0) includes both new tests. DONE
