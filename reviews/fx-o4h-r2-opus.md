Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Candidate: `fx-o4h` at `f6f14ad` on `d02ccf5`; I reviewed `git diff d02ccf5..f6f14ad` and left the worktree clean. I read AGENTS.md, GOVERNANCE.md, the bead and its binding comments (decisions 1-7 and the 2026-11-11 lock-order comment), research §§1.1-1.8 and §4, reference §7.3, design/03 §6.4, and every touched file with its callers. I did not open `trials/e1-checked/`.

## Probes (executed)

The probes are unit-test style. They ran in a scratch copy of the candidate's crates (`git archive f6f14ad`), built into a scratch target directory with `CARGO_BUILD_JOBS=4 nice -n 19 timeout 900 cargo test -p tide_std --lib probe -- --test-threads=1 --nocapture </dev/null`. They started at epoch 1794470949, after quiet-until 1794470930 and before the next booking at 1794473400. Load was 1.2 and 18 GiB was available. The probes were appended to a scratch copy of `sync/mpsc/tests.rs`; the worktree was not touched. All seven passed. Their results:

- **Cross-runtime cancellation stress, bound 0/1/3, 120 runs.** Four producers each send on their own real runtime, with scope cancellation at random times. The receiver runs on another runtime and its scope is also cancelled periodically. Every other run drops the receiver part way through. Together the runs produced 5,265 `Cancelled(v)` sends and 22,113 `Disconnected(v)` sends. In every run:
  - each value had exactly one outcome;
  - `Ok` and `Err` were disjoint;
  - received values were a subset of `Ok` values, with no duplicates;
  - with a live receiver, everything sent `Ok` was received;
  - after an early receiver drop, `Ok` minus received was exactly the buffered capacity;
  - every payload was dropped exactly once.

  I found no lost wakeup, no loss and no duplication on the blocking paths, including last-sender drop and receiver drop racing blocked peers.
- **Panicking payload destructor during a normal receiver drop, real and simulated.** The panic propagates out of `drop(rx)`. Afterwards the channel reports `Disconnected` rather than a poisoned lock, and the simulation runtime stays usable, so both locks were released first.
- **Determinism.** The same seed gives a byte-identical trace. Channel ids restart at 0 in each simulation, even after another simulation has created channels.
- **Past deadline with an empty channel.** It returns `Timeout` at once. The trace shows `wait:"ready"` and then `result:"timeout"`, with no `blocked` event.
- **Unbounded send in a cancelled scope.** It succeeds and sets no echo.
- **Rendezvous `try_send` to a cancelled receiver.** This one fails the contract; see finding 1.

## Findings

### 1. blocking: zero-capacity `try_send` returns `Ok` for a value no receiver ever takes; the value is then dropped silently

**Evidence.**
- `try_send` decides there is room when `state.receiver.is_some() && state.buffer.is_empty()` (`crates/tide-std/src/sync/mpsc.rs:205-213`). `receive` re-checks cancellation before it takes anything (`mpsc.rs:306-311`). The receiver's registration stays set from the moment its scope is cancelled until it re-enters `with`.
- So when the scope is cancelled while the receiver is blocked, a concurrent `try_send(v)` returns `Ok(())` and parks `v` in the rendezvous buffer. The receiver resumes, returns `RecvError::Cancelled`, and every later `recv` in that scope is also `Cancelled`, because cancellation is level-triggered. When the task ends and drops the `Receiver`, `v` is dropped (`mpsc.rs:254-268`).

**Probe results.**
- Simulated: 8 of 16 seeds returned `Ok` from `try_send` and never delivered the value. The scenario is a child blocked in `rx.recv()`, then the parent runs `s.cancel(); tx.try_send(v)`.
- Real: 195 of 200 runs did the same.

**Contract.** This breaks Rust's zero-capacity guarantee that a successful `try_send` means a receiver took the value. It also breaks the sentence this change adds to design/03 §6.4, "A rendezvous send completes only after a receiver accepts its value." A blocking rendezvous `send` in the same race correctly keeps the value pending and gets it back as `Disconnected(v)`. Only `try_send` loses it. The race is ordinary shutdown traffic: a producer polling with `try_send` while the consumer's scope hits its deadline.

**Fix I'd want.** Treat a value that a zero-capacity `try_send` placed in the buffer as already accepted by the blocked receiver. It mirrors the accepted-send rule that `Cancellation::Deferred` already implements for senders.
- In `receive`'s `with`, return a buffered value when `bound == Some(0)` before checking cancellation, without observing cancellation, so no echo is set.
- Since the receiver is the only consumer and is `!Sync`, a buffered value can only exist for the registered receiver, which always resumes. With this change every such value is delivered.
- Record it in §6.4, for example: "a value a zero-capacity `try_send` handed to a blocked receiver is returned even if that receiver's scope was cancelled meanwhile". Record it on the bead too, because it refines decision 1's cancellation > message ordering for this one case.
- Add a regression test using the simulated scenario above.

The weaker alternative is to keep the code and document that `Ok` from a zero-capacity `try_send` can still be lost if the receiver is cancelled and then dropped. I don't recommend that. It gives up Rust's rendezvous meaning for no benefit.

**Why it blocks.** The fix changes `mpsc.rs` and §6.4, both of which this change edits.

### 2. blocking (v0 core tier): reference §7.3 still says only a full `sync_channel` send is a checkpoint

**Evidence.**
- `design/reference.md:695` reads: "`recv`, `recv_deadline`, and `send` on a full `sync_channel` are checkpoints".
- Decision 1, the code (`mpsc.rs:115`, `self.bound.is_some() && cancelled()`), the fixture (`conformance/cases/tasks/channels.rs`, "A free bounded slot still checks cancellation") and design/03 as this change edits it (`03-authority-concurrency.md:328` and `:1499-1501`) all make every `SyncSender::send` a checkpoint, including one with free capacity.

**Why it matters.** The reference is the agent-loadable core and E1's input. A reader of the reference would expect a bounded send with room to succeed in a cancelled scope. Tide now returns `Cancelled(v)` there. The divergence comes from this change: before it, §6.4 matched the reference.

**Fix I'd want.** Change reference §7.3 to "`recv`, `recv_deadline`, and every `SyncSender::send` are checkpoints". That is about the same token count. `design/reference.md` is a single-holder file, so the coordinator has to schedule the edit with whoever holds it, including the D14 recount. It must land with or before this bead, not after.

**Why it blocks.** The fix changes the v0 core tier.

### 3. non-blocking: on the real runtime every channel operation unparks every waiting thread

**Evidence.** `Channel::with` always calls `runtime.wake_waiters()` (`crates/tide-std/src/task/channel.rs:49`). On a real runtime that unparks every thread in `runtime.waiting` (`task.rs:369-372`): sleeps, joins, limit waits and all channel waiters. Each of them then re-locks the runtime and parks again.
- Real channel hand-offs already wake the right thread through its `Signal`. Only the simulator needs `wake_waiters` here, to move resource-ready tasks to runnable.
- `clone_sender` also takes the runtime lock and runs `wake_waiters` just to increment a counter (`mpsc.rs:93-97`).

The result is correct, as the stress probe confirmed, but heavy channel traffic on one runtime causes a thundering herd.

**Fix I'd want.** In `with`, call `wake_waiters` only when `context.task.is_some()`, which is the simulated case. Have `clone_sender` take only the channel lock.

**Classification.** This is efficiency, not correctness. Fix it in this round if the author is already editing `with`; otherwise it goes as a comment on `fx-djp`.

### 4. nit: every close event's span points inside `mpsc.rs`, not the drop site

**Evidence.** The `Drop` impls call `std::panic::Location::caller()` (`mpsc.rs:246`, `:251`, `:259`). `Drop::drop` cannot be `#[track_caller]`, so the trace span of every `close` event is `mpsc.rs`, not the user's line.

**Fix I'd want.** Either record the channel's creation span for `close`, or note in §6.4 that close events carry no user span. No re-check needed.

### 5. nit: two blank lines left in the conformance stub

**Evidence.** Removing the placeholder `mpsc` module left two consecutive blank lines before the closing brace of `pub mod sync` (`conformance/rustc/std/src/lib.rs:188-189`).

**Fix I'd want.** Delete one.

## Checked and accepted

- **`Cancellation::Deferred` (`task.rs:79-95`, `:553-634`).**
  - Existing callers keep their behavior. `wait()` maps `true` to `Checkpoint` and `false` to `Shielded`, which covers sleeps and scope exit. Limit waits pass `Checkpoint`. `join` and `spawn_isolated` pass `Shielded`. The simulator's `cancellable` wake set is unchanged.
  - Echo attribution is right. The receiver always re-observes through `with`, because cancellation comes first. A sender observes only when its ticket is still pending, so an accepted send is `Ok` with no echo; the existing test plus the stress probe agree.
  - It is the smallest correct shape. A `Shielded` wait would not wake on cancellation, and a channel-owned wait would duplicate the single wait owner.
- **Lock order.** Runtime then channel, matching the 2026-11-11 comment. Neither lock is held across a point or a wait. Cross-runtime wakes go through `Signal`. Real waits re-check readiness under the runtime lock before parking, and the unpark token covers the gap.
- **Decisions 3-7.**
  - Simulation takes one point on entry, before looking at the channel; `try_` forms are points; creation, clone and drop take none.
  - Receiver order is cancellation > message > Disconnected > Timeout.
  - A ready message beats a past deadline; a past deadline with the channel empty times out without blocking.
  - There is no barging; the invariant "pending non-empty implies buffer full" holds for n > 0.
  - Chan events carry no payload.
  - The deadlock report names `recv(chan id)` and `send(chan id)`, and the driver test asserts both.
- **Drop after the simulation ends.** It falls back to task 0 with no point, and the existing test covers it.
- **Payload destructors.** They run outside both locks, including a reentrant destructor during unwind (existing test) and a panicking destructor (probe).
- **Admission.** The `policy.rs` entries cover only `TryRecvError::*` and `TrySendError::*`. The test asserts that `std::sync::mpsc::channel` stays unmatched.
- **Equivalence.** The stub includes production `sync.rs`, which includes `sync/mpsc.rs`, and the placeholders are gone.
- **Docs.** The README, crate docs, conformance README and rules rows match the code, apart from findings 1 and 2.
- **Size.** I found no accidental complexity beyond finding 3. The adapter and the error types are about as small as the decisions allow.

## Not checked

- Full suites. I ran no heavy builds and no `heavy.sh`, per the brief.
- Driver tests under `tide-admit`.
- The future rebase over `fx-m72.3`.

DONE
