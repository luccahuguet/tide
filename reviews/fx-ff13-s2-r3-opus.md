Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 4c10448 (code)

## Verdict

**Changes required.** One blocking finding: a test is missing for a claimed proof surface (B12). I found no authority escape, no trust-boundary defect and no panic, hang or unbounded allocation on peer input. The fixes for B1-B11 hold. Everything else is non-blocking.

## Scope

Read-only Opus converge round (GOVERNANCE.md Merge Rule; AP-PROOF-001) for fx-ff13 stage 2. It reports and does not fix.

- Worktree `/mnt/build/eon-agents/worktrees/tide-fx-ff13`, branch `fx-ff13`, HEAD `4c10448cf9f3ecdf0ec488402e79d2450fd53c98`. Tracked tree clean.
- Base `git merge-base HEAD main` = `0e38e552c0137ba6682804a2279621d6f5d212b0`.
- `git diff 0e38e55..4c10448`: 34 files, +2993 −187. **SHA-256 `81a68651fcbd443a61c0bde06b7f920ddba52e148df1662325333e545d2b1903`.**

What I read:

- **Production code, in full:** `net.rs`, `net/{host,http,resolve,sim,real}.rs`, `sim.rs`, `sim/http.rs`, `task/net.rs` (`socket`, `http_peer`), the real branch of `Context::wait_with_socket` and `lock`/`refresh_deadlines`, `replay.rs`, `codec.rs`.
- **Tests:** all new tests in `net/tests.rs`, `http.rs` and `resolve.rs`; tide-admit `build.rs` and `driver.rs`; the conformance runner's `rule_id` change; the stub std's `lib.rs` and `Cargo.toml`.
- **Admission:** tide-admit's `pure.rs` `NOT_PURE`, the ambient-net rules in `policy.rs`, and the `-L dependency=` / TD0912 extern-crate guard.
- **Lockfiles:** both, checked package by package.
- **Pinned sources:** httparse 1.10.1's `parse_chunk_size`, for the B9 slice arithmetic.
- **Docs:** README, reference §§6.1/6.7/6.8, companion 15.2/17/21, 03 TCP/NET-HOST/HTTP, rules.md NET-HTTP-1..5, research/42 §§3, 8, 9, and 01 D-Network.
- **Earlier rounds:** s2-r1, s2-r2 and the s2-r2 re-check, plus `report.md`, which I treated as leads only.
- **Bead:** `br --no-db show/comments fx-ff13`, AGENTS.md.

Checks I ran myself:

- `scripts/count-tokens.sh design/reference.md </dev/null` gives **24,968**. That is under D14's 25,000 and matches the companion's "Core count" line.
- Companion 15.2's code block diffed against `conformance/cases/authority/net_fetch.rs`: identical except the `// src/main.rs` comment, so "verbatim" holds.
- `openssl x509` on the test certificates: root, server and untrusted are valid 2026-11-18 to 2126-10-25. No near-term expiry.
- The lockfile package lists added in `Cargo.lock` and `conformance/rustc/std/Cargo.lock` are identical (38 packages each).

I ran no cargo, rustc, tide-admit or heavy.sh, made no edits other than this file, and wrote nothing to `.beads/`.

Proof observed (tide-lead's runs on this candidate, `suites/candidate.txt` = 4c10448):

- `cargo test --workspace`: `test.exit` 0, 23:00:41Z to 23:05:04Z, **459 passed, 0 failed** (summed from `test.log`).
- `tide-conformance`: **still running when I finished this report.** `conformance.log` had 346 PASS, the 3 standing AWAITS CHECKER (TD0809, TD0808, TD0205) and 0 FAIL, with no exit file. Landing needs its exit 0 on this candidate.

## Blocking findings

### B12 — NET-HTTP-2 claims real-backend `Cancelled` behaviour that no test exercises

**Where:** `conformance/rules.md` NET-HTTP-2. The rule covers "NET-ERR's variants on both backends: … `Timeout` at the deadline, `Cancelled` winning at the same wait …", and its evidence column cites `net::http_tests::real_get_*` and `tls_get_*`. The code paths are `crates/tide-std/src/net/http.rs:200-209` (`fail`), `:327` (TLS read loop), `:575` (`dial`) and `:603-606` (resolver mapping).

**Failure scenario:**

- Every `NetError::Cancelled` assertion in the change is on the simulated backend: `simulated_get_cancellation_wins_and_replays`, `simulated_peer_callbacks_can_schedule_and_cancel` and `simulated_packet_arrival_overflow_obeys_deadlines_and_cancellation`.
- No real-backend `get` ever runs in a cancelled or deadline-bearing real scope. The authority/tcp.rs case covers TCP cancellation on both backends, but not `get`.
- The real mapping is separate code: `fail`'s first arm, `dial`'s `is_cancelled` arm, the resolver's `is_cancelled` arm, and the TLS `Conn::read` loop's own `c.check()`.
- Suppose a regression maps a cancelled socket wait to `Reset` or `Io`. For example, reordering `fail` so `ErrorKind::Cancelled` reaches the `_ => NetError::Io(e)` arm, or dropping the `c.check()` from `Conn::read`. Every listed test still passes. Yet the bead's Acceptance ("`NetError` classification matches companion 21") and the rule's "both backends" claim would be false.
- I traced today's code and found it correct. The real `Context::lock` refreshes deadlines, so `c.check()` sees a passed scope deadline, and `wait_with_socket` re-checks cancellation before returning `Timeout`. What is missing is the proof, not the behaviour.

**Minimum fix:** add one real-backend test in `net/tests.rs`. Use a real `Tasks` scope, as `task/net.rs`'s eventfd test already does. The server is a loopback peer that reads the head and holds the connection open, as `plain(b"", "hold")` does.

- (a) `s.set_deadline(now + 50 ms)`, `get` with a 10 s deadline: expect `Err(NetError::Cancelled)` well under 2 s.
- (b) Scope deadline and `get` deadline at one instant: expect `Cancelled`.
- (c) Optional: (a) over HTTPS with the test root, which covers `Conn::read`'s TLS loop.

The alternative is to narrow NET-HTTP-2's wording and evidence to what is tested. That would weaken a stated proof surface, which is the owner's call.

## Non-blocking findings

1. **Per-chunk buffer drain is quadratic in buffered tiny chunks** (`http.rs:534-536`).
   - After every chunk, `self.buf.drain(..self.pos)` moves the rest of the read buffer, up to about 64 KiB.
   - A peer that sends a valid chunked body of 1-byte chunks (`1\r\nx\r\n`, 6 bytes each) makes each 64 KiB read cost about 10,900 drains averaging about 32 KiB, roughly 350 MB of memmove. That is tens of milliseconds of CPU per 64 KiB received, so about 2 MB/s of input keeps a core busy until the caller's deadline.
   - It is bounded (by the deadline, 64 MiB and HEAD_LIMIT), so it does not block. Fix: drain only when `pos` passes half the buffer, or once per `more()`.

2. **The `url::Host` → `Host` key mapping exists twice** (`host.rs:70-74` and `http.rs:137-141`).
   - 03 NET-HOST says entries, URLs and addresses "reduce to one key by one function". Today the two copies agree, and `net_host_keys_aliases_as_one_destination` checks that indirectly.
   - Under AP-OWNERSHIP-001 the key should have one owner. A `pub(crate) fn key_host(url::Host<&str>) -> Host` in `host.rs`, used by both, removes the chance that the copies drift apart, for example on trailing dots or mapped IPv6.

3. **NET-HTTP-4's guard does not cover trait impls on `Url`** (`build.rs` `std_net_public_items_are_fixed`).
   - The guard scans lines starting with `pub `. A later `impl Deref<Target = url::Url> for Url` or `impl AsRef<url::Url>` would pass it.
   - Either impl would make `socket_addrs` (ambient DNS) callable through auto-deref without naming the `url` crate, which TD0912 blocks. Nothing leaks today.
   - Cheapest hardening: also assert the set of `impl … for Url` header lines in `net/http.rs`.

4. **A peer's panic becomes `Timeout` or `Cancelled` once the deadline or scope has ended** (`sim.rs`, after `catch_unwind`, the `c.check()` and `now >= until` returns before `resume_unwind`).
   - B5's fix depends on this: a peer that unwraps a sleep cut short by the child-scope deadline panics.
   - It also hides a genuine assertion failure in a peer that runs after virtual time reached the deadline.
   - research/42 §9 says "A peer that panics panics the requesting task", and companion 17 says nothing about it. Fix: state the precedence in companion 17, or resume the panic unless it came from a cancelled wait.

5. **TLS name verification has no test.**
   - `tls_get_verifies_frames_and_needs_close_notify` proves chain rejection (untrusted root) and success for `localhost` and `127.0.0.1`. Both are in the certificate's SANs.
   - It never asks for a name or IP outside the SANs. AP-DEPENDENCY-001 asks to prove the relied-on behaviour; rustls's default verifier owns this, so it is non-blocking.
   - Cheap test: bind the TLS server on `0.0.0.0` or `127.0.0.2`, fetch `https://127.0.0.2:port/`, expect `Io(InvalidData)` and no HTTP data at the server.

6. **The trace shows std-internal spans.** `http_peer` opens a child `Tasks::scope`, so each served simulated `get` adds Scope/ScopeExit scheduling points and trace events whose span is in `task/net.rs`. This is deterministic and replays, but the trace shows std-internal locations inside a user's `get`. Cosmetic.

7. **The lockfile's extra entries are recorded only in the builder's report.** research/42 §8 says the committed lockfile "fixes them exactly and its report lists any that differ". The extras are `utf_types`, the optional `serde`/`serde_derive`/`syn` 3, and the other-target `windows-*`/`wasi` entries. A one-line note in research/42 §8 would make the owning document complete. Text-only.

8. **`net_fetch.rs` is tagged `NET-HTTP-2`** but exercises only the simulated backend. NET-HTTP-3 alone fits it. Text-only.

## Reasoning

**Earlier fixes.**

- **B1 (resolver pool):** `Pool::release` drains and wakes every waiter under the mutex, both on helper completion and on spawn failure. A woken waiter that leaves cannot strand the slot.
- **B2 (TLS writer):** `send` accepts a prefix, flushes, and repeats. After a flush, rustls's buffer is empty, so `write` returning 0 is unreachable apart from a genuine rustls error.
- **B3/B4/B7/B9/B10 (response reader):**
  - Framing fields are decoded and trimmed with SP/HTAB only; other values stay opaque.
  - Trailers are parsed with `parse_headers` and bounded on both partial and complete paths.
  - Non-2xx responses run the same body reader before `Status`.
  - `chunk_line` requires hex digits and checks the extension grammar. httparse's `Complete` always ends at CRLF, and a bare LF before any digit is an `Err`, so `len - 2` cannot underflow.
  - Codes outside 100–599 are refused right after the head.
- **B5/B6 (simulated peer lifecycle):** the private child scope bounds the callback's waits without cancelling the caller. The `HttpPeers` field guard drops callbacks outside the SimNet lock, before `simulation`, on both exit and unwind.
- **B8 (replay):** the driver-backed replay test changes the profile, changes the address, and removes the input.
- **B11 (arrival overflow):** `None` stands for an arrival that never comes, and it waits only to the deadline.

**Authority.**

- **`Url::key` and its checks:**
  - `Url` is a private newtype. `Url::key` derives the destination from the url crate's own parsed host, so the checks on the host as written (`url_host`) restrict input but never select the destination.
  - I traced `url_host` against WHATWG's special-authority, authority (last `@`) and host (`:` outside brackets) states. It agrees, and a mismatch could only widen refusals: an `xn--` key can never equal an entry, because entries refuse `xn--`.
  - Userinfo and non-http(s) schemes are refused before `only`. The port is `port_or_known_default`.
- **What the key drives:** lookup, dialing, SNI, certificate name and `Host` all come from the key. There are no redirects.
- **Request line:** the target is the url crate's path and query. The input is ASCII only, and the parser has percent-encoded spaces and controls and removed CR/LF/TAB, so the request line and `Host` cannot be split.
- **Entries and addresses:**
  - `connect` and `bind` addresses go through `host::host`, `SocketAddr` (zone refused) or the `(&str, u16)` IP fast path, and all key alike.
  - `only` intersects. `no_bind` refuses before `resolve`. `bind` and inbound peers are open, as the docs say.
- **No ambient route:** the backend modules are private, `Host` is `pub` but unnameable, and the test root exists only under `cfg(test)`. `NOT_PURE` already lists `Net`, `TcpStream`, `TcpListener` and `Incoming`; `Url`, `UrlError` and `NetError` hold no handle. Non-std crates are not nameable: deps are passed as `-L dependency=` and `extern crate` is TD0912. So `url`, `rustls` and `ring` cannot be reached directly.
- **Stated limits:** rebinding and the resolver's caches and continuing traffic are stated limits, not findings.

**Untrusted input.**

- **Heads:** at most 100 headers and 64 KiB in total (interim heads included). Each read is at most 64 KiB, so the buffer stays under 128 KiB.
- **Body memory:** under 64 MiB + 64 KiB per path. Content-Length bodies peak near 2 × 64 MiB because of the copy.
- **Framing errors:** both framings, an unsupported coding, an invalid or repeated Content-Length, and a too-large Content-Length are all `InvalidData`.
- **HTTPS close-delimited bodies:** they end only at `close_notify`. rustls learns of EOF through `read_tls(&[])` returning 0, and a bare close surfaces as `UnexpectedEof`, then `Reset`.
- **Bodyless statuses:** 1xx is skipped, 101 refused, 204/304 return at the head.
- **Waits:** every wait passes the call's `until` to the socket poll, and `retry` checks cancellation and the deadline before any I/O.
- **Panics and hangs:**
  - No reachable panic: the two `unreachable!` arms need `literal()` to return `None` for an `Ip`, which it never does.
  - No hang past the deadline other than the documented soft deadline of an abandoned resolver helper.
  - CPU is bounded by the deadline (non-blocking 1).
- **TLS configuration:** ring provider, TLS 1.2/1.3, compiled-in webpki roots, `NoKeyLog`, resumption disabled, ALPN http/1.1, and no dangerous verifier.

**Simulator.**

- **One trace event:** every path through `Net::get` with a simulated `Net` records exactly one `net.get` event (key, delays, result, no path, query or body). The only exception is a re-raised peer panic.
- **Order and draws:** a domain miss returns `Dns` before any draw. An IP or `localhost` with no peer returns `Refused` after two draws. `only` and shape are checked before any draw. Registration draws and schedules nothing.
- **Arrival arithmetic:** checked arithmetic throughout, with cancellation checked first.
- **Peer callback:** taken out under its mutex, waited for cancellably on a SimNet signal (no lost wakeup, since only one simulated task runs at a time), and restored before the panic is resumed.
- **Collisions:** HTTP peers on loopback and wildcard addresses reserve their port against later binds and ephemeral picks.
- **Replay:** the manifest's `http` kind is allowed, and a changed or missing peer identity is refused.

**Dependencies.**

- `crates/tide-std` and the stub std pin the same six crates exactly: `rustls =0.23.46` (`ring`, `std`, `tls12`, no defaults), `ring =0.17.14`, `webpki-roots =1.0.12`, `httparse =1.10.1` (no defaults), `url =2.5.8`, `idna_adapter =1.0.0`. The two lockfiles add the same 38 packages.
- research/42 §8 records each crate's capability, its rejections (native-tls, aws-lc-rs, platform verifier, OpenSSL, and others), license, ring's maintenance status, and the replacement cost.
- `report.md` records the RustSec check and the lockfile entries beyond §8's list (non-blocking 7).
- ring is the first C build. The nix and Docker delivery builds passed on 7e86e6b. Nothing after that changed the dependency graph, so that proof still covers dependency delivery, and only dependency delivery.

**Contract and docs.**

- The code, reference §6.7/§6.8, companion 17/21, 03 (TCP, NET-HOST, HTTP), 01's Network entry, rules.md NET-HTTP-1..5 and the README all agree.
- Every planned label for `get` and `world.http` is gone, and UDP, faults, nodes and `drop_all` stay planned.
- research/42 is correctly marked as stage 1's record with a stage-2 status line.
- The one overclaim is B12's proof column.

**Leftovers and tests.** I found no dead code or orphaned stubs: the stub std's `get`, `NetError`, `Url`, `Request`, `Reply` and `World::http` stubs are removed and production is included. Tests check observable effects. The only near-restatement is `released_capacity_survives_cancelled_selected_waiter`, which drives a local `Pool` by hand. It is acceptable, because it isolates `release`'s contract.

**What landing still needs:** the B12 test (or the owner's narrowing of the rule), `tide-conformance` exit 0 on the final candidate, and a rerun of `cargo test --workspace` if B12 adds a test. B12's fix adds a test without changing behaviour, so a finder re-check of that fix is enough; no full fresh review is needed.

DONE
