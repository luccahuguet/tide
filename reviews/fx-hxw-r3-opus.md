Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Reviewed frozen candidate `7cde0b45d77a04b505ce6319d2f8e21f50e54b20` (branch `fx-hxw`, one commit on `dd40b78`), `git diff dd40b78..7cde0b4`, 22 files. Read-only. The worktree was not modified. Nothing under `trials/e1-checked/` was opened or searched. One small, test-only finding blocks. Everything else reads correct against decisions 1–6 and research/21 §§1.9–1.13 and §4.

## Findings

1. **blocking (test-only, small): the 128-connection backlog bound has no falsifying test, although the report claims one.**
   - Evidence: `conformance/cases/authority/tcp.rs:278-294`. The fixture fills the backlog with 128 connects, then:
     - spawns a 129th connect;
     - sleeps 30 ms;
     - accepts one connection;
     - joins the connector.
   - Nothing asserts that the 129th connect was still blocked when the accept ran. Each simulated connect takes 2–20 ms, so with no bound the connector finishes inside the 30 ms sleep and every assertion still holds. The probe below confirms it. With `BACKLOG = 1_000_000`, the 129th connect completed at 1.427 s, before the accept at 1.445 s, and an identically shaped test passed.
   - The bound lives in `crates/tide-std/src/net/sim.rs:194`. `report.md:37` lists the "128-entry backlog" among the admitted checks, so the claimed proof does not exist. Codex r1's finding 5 set the same bar.
   - Fix: in the existing case, record the connector's completion instant and assert that it is not before the accept's instant. The connector sets `clock.now()` after `connect` returns, and the main task reads `clock.now()` just before `accept`. Equivalently, assert from `sim.trace()` that the 129th `net.connect` was blocked until after that `net.accept`. With 128 it is blocked, as the probe shows: completion equals the accept instant. Three or four lines; no new harness.

2. **non-blocking (landing order with fx-u65.4): two conflicts for whichever bead lands second.**
   - **Expected codes change.** fx-u65.4's `conformance/cases/authority/cap_ambient_paths_reject.rs` (in its worktree at `7db157b`) annotates lines 14–15 as `std::net::TcpStream::connect(..) //~ E0433 FX0501` and `std::net::TcpListener::bind(..) //~ E0433 FX0501`.
     - Today they get E0433 because Tide's std has no `net::TcpStream`.
     - After fx-hxw, both paths resolve to Tide's handles, which have no such associated function, so rustc reports E0599. This candidate's own `tcp_has_no_ambient_constructors_or_resolution` (`crates/tide-admit/tests/build.rs`) asserts exactly that.
     - FX0501 should still fire. fx-u65.4's `rust_counterpart` maps `tide_std::net::TcpStream` to Rust's `std::net` re-export and keys `std::net::tcp::TcpStream::connect` (`walk/authority.rs:249-282` there), the same path `File::open` takes today (`E0599 FX0501`).
     - So the second lander changes those two annotations to `E0599 FX0501`. Lines 3 and 16 of that file (`UdpSocket` in driver.rs) are unaffected, since Tide's std still has no `UdpSocket`.
   - **Textual conflict.** Both beads insert rules immediately after `deny("tide_std::__rt::*", ENTRY)` in `crates/tide-admit/src/policy.rs:71`. The two sets don't interact:
     - fx-hxw allows `core::net::*` (pure address data only; no resolver or constructor lives in `core::net`) and `std::net::Shutdown::*`;
     - fx-u65.4 denies `std::net::tcp::*` and `std::net::udp::*` constructors;
     - fx-u65.4's `allow()` constructor keeps its signature, and its test that `tide_std::net::Net::bind`/`connect` match no rule still holds.
   - Fix: none in fx-hxw. The coordinator resolves both when rebasing the later bead and reruns `authority/cap_ambient_paths_reject.rs` plus the TCP build tests.

3. **non-blocking: `Incoming::next` lacks `#[track_caller]`, so its records and deadlock spans name `tide-std` instead of the program.**
   - Evidence: `crates/tide-std/src/net.rs:296-303`. `accept` is `#[track_caller]`, but `next` calls it without forwarding.
   - As a result, `for conn in listener.incoming()` blocked forever reports `net.accept(port N)` at `net.rs:300`.
   - The precedents forward the caller: `fs::ReadDir::next` (`fs.rs:647`) and `io::Lines::next` (`io/shared.rs:1079`). Source-located wait records are 03 §9.1/§9.3's contract.
   - Fix: add `#[track_caller]` to `Incoming::next` (one line). Fold it into the fix round for finding 1.

4. **nit: real `poll` failure leaves a stale waiter.**
   - Evidence: `crates/tide-std/src/task/net.rs:99` panics on any `poll` errno except EINTR. The runtime lock is not held, so nothing is poisoned. But the `Waiter` pushed at `task.rs:604` is never removed, because the `retain` at `task.rs:646` is skipped.
   - Practically unreachable: it needs ENOMEM, or EINVAL from `nfds > RLIMIT_NOFILE`.
   - Fix (optional): return it as an `io::Error` through the same path as the eventfd fix, or leave it.

5. **nit/non-blocking: Rust call-site coverage of the sealed address trait.**
   - Evidence: `crates/tide-std/src/net.rs:8, 29-83`.
   - Rust's `ToSocketAddrs` also accepts `(String, u16)`, `SocketAddrV4` and `SocketAddrV6`.
   - `SocketAddrV4`, `SocketAddrV6` and `AddrParseError` are not re-exported, so `Result<SocketAddr, std::net::AddrParseError>` does not compile. The implementation still parses through it.
   - Decision 1 names the five re-exports explicitly, so widening is the coordinator's call, not a defect. Record it for a later bead if C10-style code needs it.

6. **nit: TCP-1's row sits in a new trailing section, `## TCP runtime surface`, after "Tier unclear" and "Considered and excluded".**
   - Evidence: `conformance/rules.md:240-244`.
   - Its source is 03 §1.1, and the file groups rules by source chapter. The row belongs under `## 03-authority-concurrency.md` beside CAP-1.
   - Both `//@ rules:` IDs used (`CAP-1`, `TCP-1`) exist.

7. **nit: leftover comma in the crate docs.** `crates/tide-std/src/lib.rs:31` reads "the identities `Rand`, and `Proc`". It should be "the identities `Rand` and `Proc`".

## What I checked and found sound

**Real backend** (`net/real.rs`, `task/net.rs`, `task.rs:582-648`)
- Sockets, listeners and accepted streams are nonblocking and CLOEXEC.
- Connect runs through rustix: EINPROGRESS, then wait for writability, then `SO_ERROR`. A dropped socket aborts an in-flight connect, which is better than research's blocking-connect fallback.
- Each operation loops check → call → WouldBlock → wait; EINTR retries.
- The socket wait breaks `Ready` only when `poll` reports revents on the socket fd, so a spurious eventfd wake cannot return a half-connected stream.
- The eventfd is read before the runtime state is rechecked under the lock, and cancellation sets state under that lock before `Waiter::unpark` writes the eventfd. No wake is lost.
- `set_deadline` and cancellation reach poll waiters through `Waiter::unpark` (`task.rs:323, 390, 1562`).
- The deadline bounds `poll` through `effective_deadline`, and `refresh_deadlines` converts expiry into cancellation.
- Eventfds: allocated per thread and fallibly, outside the runtime lock and before waiter registration, cached only on success, and RAII.
- Writes go through std's `send` with MSG_NOSIGNAL.
- Empty real reads return Ok(0) after the cancellation check.

**Inline DNS**
- Labeled as a limit in `net.rs:4,108`, the crate docs, README, design/03 §1.1, and conformance/README.
- The `only` check runs before any lookup.
- `localhost` maps to 127.0.0.1 without consulting the resolver.

**Simulated loopback** (`net/sim.rs`)
- The SimNet mutex is never held across `begin`, `wait`, `draw`, `now` or `wake`: every guard is scoped or temporary before those calls.
- Lock order is state then Signal, never state then runtime.
- Ports draw from the "tidenet!" sub-stream and redraw while taken, and return AddrNotAvailable only when the range is exhausted. The draw needs no lock because the baton serializes tasks.
- AddrInUse is checked per port, which covers 0.0.0.0 against 127.0.0.1 as on Linux.
- Accepted streams report the dialled address as `local`, and the listener reports the wildcard address.
- Segments arrive at `max(now + latency, previous)`. FIN is ordered behind data, including the zero-latency FIN sent during unwinding.
- Read: empty buffer gives Ok(0); a due reset wins over queued data; it waits until the earliest of next arrival and reset.
- Write: shut or reset gives BrokenPipe; a full peer means a cancellable wait; otherwise it accepts `min(len, space)`.
- Writing to a closed peer accepts the bytes and resets the writer after a round trip (two draws).
- The 65,536-byte capacity cannot underflow. A listener drop resets backlog clients first, so its uncleared `bytes` are never read.
- Connect is one cancellable round-trip timer and atomic at wake; a non-loopback address or a missing listener gives ConnectionRefused.
- Live unwinding cleans up without drawing or recording. Ended and foreign simulations are left untouched (pitfall 8).
- Trace lines are well-formed (`event` leaves the object open; `net_event` closes it) and carry no payloads.

**Attenuation** (`net.rs:137-196`)
- `no_bind` is checked before address parsing.
- `only` keys parse IP literals and lowercase other hosts, never looking anything up. Invalid entries grant nothing, and intersection keeps only keys present in both.
- `only` restricts neither `bind` nor accepted peers.
- Cancellation precedes policy.

**Semantics**
- `incoming` ends after a Cancelled error and continues after other errors.
- Checkpoints: bind, connect, accept, read, write and shutdown.
- `local_addr`, `peer_addr` and `flush` only assert the world, with no point (decision 3).

**Equivalence and tests**
- One suite runs on real 127.0.0.1:0 and four seeds, comparing ErrorKinds; traces are rerun byte-for-byte.
- The r1 regressions remain falsifying:
  - the sibling wake has no deadline backstop;
  - payloads depend on position;
  - the empty read runs under a deadline;
  - the unwind EOF and listener close run under a deadline.
- Capacity is falsified by `write == 65_536`, and connect latency by the 2–20 ms window.
- The panic-hook counts of 9, 9 and 4 match one real run plus 4 seeds × 2.

**Admission**
- `Incoming` is registered as NET (`pure.rs:89`).
- `core::net::*` holds only pure data.
- Rust's `ToSocketAddrs` impls stay unadmitted: the trait is not exported, and `std::net::*` items under the trait impls aren't matched.
- The stub `include!`s production `net.rs` and its backends, so there is no duplicate runtime.

**Reference.** `design/reference.md` §6.1 and §6.7 are brief but not wrong for the implemented subset; §6.8's "capability operations are scheduling points" is consistent with decision 3, since accessors are not table operations in 03 §1.1. design/03 §1.1 and §9.3 carry the model and its provisional limits accurately. No reference change.

## Probes

Each probe ran under the gate, with `ulimit -c 0`, `nice -n 19`, `CARGO_BUILD_JOBS=4`, `timeout 300` and stdin closed. They ran on a `git archive 7cde0b4` copy of `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/tide-std` and `crates/tide-macros`, under `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-hxw/ec330e0f-95f8-4fba-8b89-2c8f51c016d4/scratchpad/src`, with a separate target dir. The candidate worktree was not touched, and no real sockets were opened.

The probe adds a test, `net::tests::probe_backlog_bound_observed`, which mirrors `tcp.rs:278-294`:
- seed 7;
- 128 connects;
- a spawned 129th connect that records `clock.now()` when it returns;
- a 30 ms sleep;
- a record of `clock.now()`;
- then accept and join.

```sh
python3 /home/lucca/.cache/eon-agents/tide-fx-iob-work/build-gate.py 300   # first attempt: AssertionError load >= 12 (refused); retried later: passed
CARGO_TARGET_DIR=<scratch>/target CARGO_BUILD_JOBS=4 nice -n 19 timeout 300 cargo test -p tide_std --lib net::tests::probe_backlog_bound_observed -- --exact --nocapture --test-threads=1 </dev/null
```

Results, on Linux 7.2.6 with the pinned nightly from `rust-toolchain.toml`:
- **`BACKLOG = 128` (candidate):** `connector_done=1.445364194s accept_at=1.445364194s blocked=true`; 1 passed.
- **`BACKLOG = 1_000_000` (bound removed):** `connector_done=1.427145525s accept_at=1.445364194s blocked=false`; 1 passed.

The fixture's assertions do not distinguish the two runs, which confirms finding 1. A completion-not-before-accept assertion would.

The first gate attempt was refused for load; nothing ran until a later gate passed. I did not rerun the full suites, the conformance runner or held-out C10. Codex r2's focused TCP runs on this HEAD stand as the execution proof for the rest.

DONE
