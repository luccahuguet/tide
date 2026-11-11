Model: claude-opus-5-5 · Effort: high · Date: 2026-11-11

Verdict: approve_with_alternatives

Candidate: `8a7a285` on `65aee6c`, worktree clean. `git diff 65aee6c 8a7a285 | sha256sum` =
`e21fe32e20df07ade0d0b958ccfca1d489d68d1cf88e4dbdf931c6d3823f42ee`, which matches the frozen
candidate. This was a read-only review. I ran no build or probe, so every claim below comes from
reading the code, tests, beads (`fx-o8j`, `fx-zc1`, `fx-o4h`, `fx-hxw`, `fx-nkv.6`), 03 and
`research/20-time.md`. I did not open `trials/e1-checked/`.

## What holds

- **Virtual time.** It moves only in `Scheduler::pick` (scheduler.rs:266), and only when no task is
  `Ready`. It jumps exactly to the first key, asserts that time moves forward, and wakes every timer
  of that instant in key order before choosing a task. A timer leaves with its waiter on any wake
  (`wake`, scheduler.rs:188), so cancellation leaves no stale timer. The cancellation test's "no
  `to:5000000000`" check proves this.
- **Determinism.** Timer keys are `(deadline, class, seq)`. After the jump, the seed chooses among
  the woken tasks. fx-o8j draws no random numbers of its own. `two_sleepers` shows that both
  orders occur across seeds and that a fixed seed replays byte for byte.
- **Wait primitive.**
  - `Context::wait` (task.rs:348) is the single owner of blocking.
  - Both branches check cancellation, then readiness, then timeout, on entry and after every wake.
  - On the real side, the waiter registers and checks its condition under the runtime lock, then
    drops the lock before parking. An `unpark` that arrives between the drop and the park leaves
    a token, so no wakeup is lost. The loop tolerates spurious wakes.
  - The waiter returns `Timeout` only when `monotonic() >= until`, so it never sleeps short.
  - On the sim side, the waiter enters `cancellable` under the same lock as `block`, and
    `wake_waiters` covers that map on every first-cause cancel, including `run`'s failure path and
    the abandoned-spawn path.
- **Arithmetic.**
  - `Add`/`Sub<Duration>` panic with Rust's messages, including Rust's own "…to instant" wording
    on `SystemTime`.
  - `duration_since`/`Sub<Instant>` saturate, as in Rust since 1.60.
  - `SystemTime::duration_since` returns a `Result` whose `SystemTimeError` has Rust's
    `duration()` and `Display`.
- **Fatal path.** `fatal_simulation` (task.rs:74) writes straight to fd 2 under the stderr lock,
  so the test harness's output capture doesn't swallow it. It writes the report and then the full
  trace, flushes, and aborts. The subprocess test checks that the event steps run consecutively
  and that the deadlock event comes last.
- **Range failure.** The wall-time range panic fires after the guard drops (task.rs:894–903), so
  the lock isn't poisoned. A test covers this.
- **Real authority stays outside simulations.** `Time::read` and `Time::sleep` go through
  `outside_simulation` or `Context::simulated`. The misuse test covers all four `Clock` entries.
- **Equivalence.** The conformance std pulls in `time.rs` by `#[path]` and `task.rs` by
  `include!`, and the stub's `GUESS` block is gone. `clock_time.rs` runs against production
  through the driver in debug and release.

## Findings

1. **non-blocking: the stall counter is never reset when the timer set empties without a jump.**
   - Evidence:
     - `Scheduler::point` returns early when `timers.is_empty()` but leaves `stalled` as it is
       (scheduler.rs:136).
     - `stalled` is reset only on a jump (scheduler.rs:297) and after a report (scheduler.rs:143).
     - When a cancel wakes a sleeper, `wake` removes its timer (scheduler.rs:192) and the count
       stays.
   - Failure: with `stall_limit = 1000`, main does 900 points while sleeper A waits, then cancels
     A's scope, so A wakes early and no jump happens. Then sleeper B registers, and about 100
     points later the seed fails with a stall report, although B has waited only about 100
     points. This contradicts the guard as research §2 item 4 defines it ("consecutive points
     during which a timer is pending") and the new 03 text (03:1777–1779).
   - Fix: reset `stalled` wherever the timer map becomes empty, in `wake` after the `remove`. A
     reset in `point`'s empty branch is not enough: a cancel followed by a new `block` passes no
     point in between. Add a regression test that uses a small `stall_limit`.

2. **non-blocking (rework risk): the readiness hook and the documented lock order don't fit the
   recorded designs of fx-zc1, fx-o4h and fx-hxw.**
   - Evidence:
     - task.rs:346–347 says "A later wait kind adds an `Operation` and its `wait_done` arm".
     - task.rs:358–359 says "take runtime then resource".
     - `Runtime::wait_done` (task.rs:203) can see only `Runtime` state.
     - `Join`'s arm unwraps `baton` (task.rs:205), so it panics if a real-side caller ever uses it.
     - Real wakers are `Vec<thread::Thread>` in one `Runtime` (task.rs:27, 158).
   - Conflicts:
     - fx-o4h decision 3 records the lock order "channel then runtime", the reverse of this
       comment. Followed as written, the two orders form an ABBA deadlock on the real side.
     - fx-zc1's `Limit` lives in the `Tasks` handle, and research §1.6 notes that it is shared
       across `Runtime`s (one per top-level scope). It can't be read from `Runtime::wait_done`,
       and a release can't find waiters registered in another `Runtime`.
     - fx-hxw decision 4 waits in `poll(2)` plus an eventfd, which `Thread::unpark` can't
       interrupt.
   - Fix: don't build the abstraction now (AP-MINIMAL-001). Change the comment to say what each
     later bead will have to change:
     - readiness becomes a caller-supplied check, or the resource keeps its own waiter handles;
     - the waker becomes Thread or eventfd for fx-hxw;
     - pick one lock order.

     Then record that order on fx-o4h (or fix fx-o4h's decision 3) before fx-o4h or fx-zc1 starts.
     The usual choice: the resource keeps its own `Thread` handles and never takes the runtime
     lock while holding its own, so "runtime then resource" holds everywhere.

3. **non-blocking (rework risk, fx-zc1): the stall guard and the timer firing are not ready for
   class-0 scope deadlines.**
   - Evidence:
     - `timers: BTreeMap<TimerKey, TaskId>`, and `pick` treats every fired key as a blocked
       task's timer (`unreachable!("a timer outlived its wait")`, scheduler.rs:266ff).
     - The advance is sealed inside `Scheduler`, which can't call `Runtime::cancel` or
       `wake_waiters`.
     - `point` counts a stall whenever any timer is pending.
   - Consequence: once fx-zc1 adds a deadline timer for each scope with a deadline, a CPU-only
     scope that passes 1,000,000 points under a far deadline gets a false stall report. fx-zc1
     will also have to move the advance up to `Runtime` level, through `Baton::pass` and its
     callers, so that a deadline cancels and wakes waiters "before any task runs".
   - Fix: nothing to change in this candidate. Add a note on fx-zc1 that the guard should count
     only class-1 (operation) timers, and that the jump has to reach `Runtime`.

4. **non-blocking (alternative): an `Instant` near its clock's origin breaks common Rust code.**
   - Evidence: `monotonic()` is anchored at first use (time.rs:78), and the simulated `Instant` is
     virtual now starting at 0. So `clock.now() - Duration::from_secs(60)` panics at the start of
     every simulation and during a real process's first minute. On Linux, Rust's `Instant::now()`
     counts from boot, so that line works in practice. Patterns like "start the last tick one
     interval in the past" are common in Rust code that agents write. That is exactly the
     learnability surface E1 measures.
   - Decision 1 adopted "offset from a process anchor / virtual now", so this is not a defect.
   - Alternative for the coordinator: keep decision 1's single representation and one Debug form,
     but put a fixed base offset into both origins, for example `Instant(BASE + elapsed)` with a
     fixed `BASE` of days. `sim.elapsed()`, trace times and everything else stay as they are. If
     declined, nothing changes: 03:200–201 already states the limit.

5. **non-blocking (acceptance): the E1 B10 item is still unobserved.** The proof covers the
   workspace and conformance only. report.md:123 leaves "B10 no longer fails on `task::Cancelled`
   or `time::Duration`" to the coordinator, and the bead's Acceptance names it. Run it, as the
   held-out comment allows, before closing `fx-o8j`.

6. **nit: 03:1777–1779 says the simulator "panics in the polling task".** The panic lands in
   whichever task takes the point past the bound. With several runnable tasks, the seed decides
   which one that is. Fix: "panics in the task whose scheduling point passes the bound".

7. **nit: `sched` events carry the pending operation's extra fields.** `event` (scheduler.rs:347,
   362–368) appends `result` for `clock.now`/`system_time` and `until` for sleeps to any event
   with that pending operation, including `pick`'s `sched` decision line. Fix: write those extras
   only for `cap` events. This is optional, since field names stay provisional until fx-nkv.7.

8. **nit: research §2 item 6 asks to "assert it in the wait primitive".** The real branch of
   `Context::wait` has no assertion of its own and relies on its callers (`Context::of`,
   `Time::read`) to keep real time out of simulations. That holds today. A one-line
   `debug_assert!` in the real branch, or a sentence in the comment, would let later wait kinds
   inherit the guarantee.

Nothing here blocks the landing. Findings 1, 6 and 7 are small, local changes that the author
should make or rebut. Findings 2 and 3 are notes to record on fx-o4h and fx-zc1. Finding 4 is the
coordinator's call. Finding 5 has to be observed before the bead closes. DONE
