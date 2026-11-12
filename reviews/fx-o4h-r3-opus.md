Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Scope: I re-checked only my round-2 findings 1-5, on the amended HEAD `cd18e74` (parent `d02ccf5`), using the interdiff `git diff f6f14ad cd18e74` (9 files). I left the worktree clean. I did not open `trials/e1-checked/`.

## Probes (executed under the gates)

Both runs used `nice -n 19`, `CARGO_BUILD_JOBS=4`, closed stdin and an outer timeout. They built in a scratch copy of the crates with a scratch target directory.

- **Run A**, started at epoch 1794471673: f6f14ad's `mpsc.rs` and runtime, with cd18e74's `tests.rs`.
- **Run B**, started at epoch 1794471694: cd18e74's `mpsc.rs`, `task/channel.rs`, `task.rs`, `sync.rs` and `tests.rs`, plus my seven round-2 probes appended.

Gate conditions at both starts:
- quiet-until was 1794470930, already past;
- the next booking was 1794473400, and both runs finished well before it, and before the 10:15Z Eon booking;
- load was about 7.8, under 12;
- about 19 GiB was available.

## Findings

### 1. Fixed: an accepted zero-capacity `try_send` hand-off survives cancellation

**Code.** `receive`'s `with` now pops a buffered value when `bound == Some(0)`, before it calls the cancellation observer (`crates/tide-std/src/sync/mpsc.rs:307-312`). The value is returned without an echo, which matches the coordinator's recorded decision.

**Why this is enough.** Only a zero-capacity `try_send` can put a value in a rendezvous buffer, and it does so only while the single, `!Sync` receiver is registered. That receiver always resumes in the same receive loop, so it always takes the value. All other receives keep the cancellation-first order.

**Regressions fail without the fix.** In Run A, both new tests fail on f6f14ad:
- `cancelled_rendezvous_try_send_is_delivered_or_returns_its_value` panics with exactly the loss outcome: `lost rendezvous value: (Ok(()), Err(Cancelled))`.
- `accepted_rendezvous_receive_beats_cancellation_without_marking_an_echo` fails its `received == Some(7)` assertion (`tests.rs:385`).

**They pass with it.** In Run B both pass. My original probe now shows no loss:
- simulated: 0 of 16 seeds;
- real: 0 of 200 runs, with 178 `Ok` results, every one received.

The cross-runtime stress probe (120 runs) still holds every exactly-once invariant.

**Docs.** design/03 §6.4, the README, the crate docs and the method docs all state the exception. §6.4 words it as the decision does: returned "without observing cancellation or setting an echo".

### 2. Fixed: the reference now names every bounded send as a checkpoint

**Edit.** `design/reference.md:695` now reads "`recv`, `recv_deadline`, and every `SyncSender::send` are checkpoints". This matches decision 1, the code and design/03. That one line is the only core change in the interdiff. The bead records that fx-o4h holds `design/reference.md`, which is a single-holder file.

**Count.** The companion count is updated to 24,995 tokens on 2026-11-12 (`design/reference-companion.md:14`), which is under D14's 25,000 budget.
- I did not re-run `scripts/count-tokens.sh`, because it spends Opus calls.
- The reported figure is consistent with the history. `design/reference.md` last changed in `c331d03` (2026-11-09), the commit that recorded 24,998. The only change since is this one-phrase swap, "`send` on a full `sync_channel`" to "every `SyncSender::send`", and a 3-token drop is plausible for it.
- The author's measurement is on the exact amended file. Since my check is consistency, not a re-measurement, I accept the count as reported.

### 3. Fixed: real-runtime channel operations no longer unpark every waiter

**Code.** `Channel::with` now calls `wake_waiters` only for a simulated context (`crates/tide-std/src/task/channel.rs:49-51`). `clone_sender` now only increments the sender count under the channel mutex (`mpsc.rs:93-95`).

**No real waiter depended on the removed wakes.** A real `wait_for` wakes for one of four reasons (`task.rs:566-598`). None of them needed a wake from a channel operation:
- **Cancellation.** `Runtime::cancel` unparks `runtime.waiting` itself (`task.rs:304-306`). An expired deadline is applied by `refresh_deadlines` inside `Context::lock`, which channel operations still call, and it goes through `cancel`.
- **Resource Signals.** These are channel hand-offs and disconnects, join and isolation completion, and limit grants. Each one unparks its own thread.
- **`wait_done` (Join and ScopeExit).** Channel operations never change that state.
- **Timeouts.** These use `park_timeout`.

**Evidence from Run B.** It passed every `task::` unit test (task controls, cross-runtime limit grant and cancellation, sleeps, joins, isolation), all 14 mpsc tests and my cross-runtime stress probe: 77 passed, 0 failed. This is an efficiency fix, so no failing-without regression is expected.

**Side effect, which I accept.** `clone_sender` no longer validates the domain, so cloning a sender outside its simulation no longer panics. That matches `Shared::clone` (`sync.rs:195-199`) and the channel's own `Drop`. Any use of the clone is still checked on its first operation.

### 4. Fixed: close-event span (documentation)

design/03 §6.4 now says close events carry an implementation span rather than the user's drop site. That was one of the two options I offered. Text only, so no re-check was needed.

### 5. Fixed: stub whitespace

The extra blank line in `conformance/rustc/std/src/lib.rs` is removed.

## Not checked

- Full suites, `heavy.sh` and the `tide-admit` driver tests. The author reports passing focused driver runs.
- A fresh token count, for the reason given under finding 2.
- The future rebase over `fx-m72.3`.

DONE
