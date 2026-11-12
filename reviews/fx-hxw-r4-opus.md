Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

This round re-checks only my r3 findings 1, 3 and 4 (AP-PROOF-001).
- **Candidate:** `86991870f8e0540885cf8f6530a8e48f8a01a076`, which replaces `7cde0b4`. It is still one commit on `dd40b78`, and the worktree is clean.
- **What I read:** `git diff 7cde0b4 8699187` (6 files) and report.md's "Fix round 2".
- **Out of scope here:**
  - Findings 6 and 7 are text-only. I saw them applied correctly: TCP-1 now follows CAP-1 under the 03 table, and the comma is gone.
  - Finding 5 went to fx-djp.
  - The coordinator resolves finding 2 at rebase.
- **Constraints kept:** read-only, and nothing under `trials/e1-checked/` was opened.

## Per-finding results

1. **Resolved: the backlog bound is now falsifiable.**
   - The change, at `conformance/cases/authority/tcp.rs:283-295`:
     - the 129th connector returns `sim.clock.now()` after `connect` returns;
     - the parent reads `sim.clock.now()` right after the 30 ms sleep and immediately before `accept`;
     - it asserts `connector.join()? >= accept_at`.
   - Why it is correct:
     - With the bound, the connector can complete no earlier than the `accept` that frees a slot. My r3 probe measured equality: 1.445364194 s for both.
     - Without the bound, the connector completes 2–20 ms after spawn, strictly before the 30 ms sleep ends. In r3 that was 1.427 s, against an accept at 1.445 s.
     - So `>=` separates the two cases with no timing slack to tune. The added `clock.now()` points don't change that ordering.
   - Sol's negative control, `BACKLOG = 1_000_000` through the real driver test, failed with the intended assertion message and exit 101. That agrees with my r3 measurement.
   - `net/sim.rs` has no fix-round diff, and `const BACKLOG: usize = 128;` is intact (`sim.rs:7`). The corrected coverage claim in the report is accurate.

2. **Resolved: `Incoming::next` now forwards the program's source location.** `crates/tide-std/src/net.rs:296` has `#[track_caller]`, so `accept`'s `Location::caller()` resolves to the program's `next()` or `for` site, matching the fs/io iterator precedents. Nothing else changed.

3. **Resolved: a real `poll` failure no longer leaves a stale waiter, and the change introduces nothing new.**
   - What changed:
     - `task/net.rs:24-41`: `poll` returns `io::Result<bool>`; EINTR still means `Ok(false)` (recheck), and other errnos become `io::Error`.
     - `task.rs:634-639`: the poll result is taken after re-locking, and an error `break`s out of the loop. That reaches the single exit, so `runtime.waiting.retain(..)` (`task.rs:651`) removes the waiter before returning. The runtime lock is never held during `poll`, so nothing is poisoned.
     - `NetContext::socket` (`task/net.rs:128-140`) propagates with `?`, so `retry` and `connect` return the error. `connect` drops its owned fd.
   - Why it introduces nothing new:
     - `wait_with_socket` has only two callers: `wait_for`, which passes `None`, and `NetContext::socket`.
     - On the `None` path `poll` is unreachable, so `wait_for`'s `.expect` cannot fire.
     - The simulated branch only wraps its existing results in `Ok`.
   - The renamed regression `socket_wait_failures_preserve_runtime_and_can_retry` is extended. It caches a wake fd on the same thread, then lowers only the child's soft `RLIMIT_NOFILE` to 1. Linux's `poll` then rejects `nfds = 2 > 1` with EINVAL, though both fds already exist. The test asserts:
     - errno 22;
     - no waiter left for the thread;
     - a working checkpoint;
     - normal deadline cancellation once the limit is restored.
   - Sol's before-fix run failed with `network poll: Invalid argument (os error 22)`, so the case reaches the failure it targets.

No new findings.

## Probes

The gate refused once for load. The next attempt passed at 2026-11-12T17:18:32Z, and nothing ran before it.

I ran on a `git archive 8699187` export of `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/tide-std` and `crates/tide-macros` into my scratchpad (`.../scratchpad/src4`), with a separate target dir. The worktree was not touched. Real sockets were only on 127.0.0.1:0.

```sh
python3 /home/lucca/.cache/eon-agents/tide-fx-iob-work/build-gate.py 300
ulimit -c 0; CARGO_TARGET_DIR=<scratch>/target CARGO_BUILD_JOBS=4 nice -n 19 timeout 300 \
  cargo test -p tide_std --lib task::net::tests::socket_wait_failures_preserve_runtime_and_can_retry -- --exact --nocapture --test-threads=1 </dev/null
```

Result: `ok`, with 1 passed and 141 filtered out (Linux 7.2.6, pinned nightly). The parent requires the child's execution marker, so the EMFILE and EINVAL subprocess paths both ran.

For finding 1, I relied on Sol's mutation control and my r3 measurements instead of rebuilding the driver. I did not rerun the full suites or held-out C10.

DONE
