Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 7e86e6b (code)

## Verdict

**Changes required.** The capability boundary and dependency selection follow the admitted design, but the new resolver, TLS writer, response reader and simulated peer lifecycle have correctness gaps. HTTP manifest replay also lacks its required negative control. Confidence is high on the source-level counterexamples below; I did not execute new Rust probes. No override or landing approval.

## Scope

Read-only round 1 supervisor review of fx-ff13 stage 2, HEAD `7e86e6ba267a9902e69bab6524714e57374e65a5`, parent `0e38e552c0137ba6682804a2279621d6f5d212b0`, branch `fx-ff13`, in `/mnt/build/eon-agents/worktrees/tide-fx-ff13`.

The supplied diff identity is reproducible against the frozen ledger revision `7d998671f12b345099a0879d97ae8a3ce91c990d`:

```text
git diff 7d998671f12b345099a0879d97ae8a3ce91c990d..7e86e6b
35 files changed, 2571 insertions(+), 189 deletions(-)
SHA-256: 86fe4c2bad004442b60ff9ec7bbf891fb80aae9d04446339c2157f0d40de1ebf
```

`main` advanced in the ledger during inspection. At observed main `7dd7ddfba44d5059d2b22af87b06a58512eced93`, the same command using that base gives `1ca56dec37cf8ab84d336324cc2c38ad01ddbf9348628848311235c37589fa5e`, 35 files, +2573/-191. The intervening diff changes only `.beads/issues.jsonl`. Against the code parent `0e38e55`, the candidate is 34 files, +2569/-187, diff SHA-256 `542d1f40ffb8786415f9a982cf43fa707a2f3cb54f80abe42e5846cd5c048e25`. The code candidate remained unchanged and its tracked worktree clean.

Reviewed the changed production code, callers, tests, conformance facade and rules, manifests, lockfiles, public exports, admission/Pure/trust-diff ownership, Proc socket-wait adaptations, README and reference changes. Read GOVERNANCE.md, BR_WORKFLOW.md, `br --no-db show fx-ff13`, research/42, the relevant owning chapter/companion sections, and the stage-1/red-team findings and re-checks. Inspected the pinned url, httparse, rustls and idna_adapter sources in the existing Cargo registry cache. The builder report was discovery, not proof. fx-6d50's future public API is outside this review.

No compilation, Tide execution, dependency installation, heavy.sh invocation, commit, Beads mutation or implementation edit was made. This requested report is the only authored file. The later heavy.sh HOLD was obeyed; no heavy run is requested by this pass.

## Blocking findings

### B1 — A cancelled resolver waiter can strand a free pool slot

**Where:** `crates/tide-std/src/net/resolve.rs:52`, `:69`, `:77`.

Fill the eight resolver slots and queue A, then B. One helper completes, decrements `busy` to seven, and pops/wakes B. If B is cancelled or its deadline expires before it acquires the slot, it returns at `result?` or the next iteration's entry checks. It neither claims the free slot nor wakes A. A remains inside `c.wait`, whose readiness predicate is its own unsignalled `Signal`, not `pool.busy < POOL`. If the other seven abandoned lookups remain blocked, A cannot start despite available capacity; `connect`/`bind` have no operation deadline and can wait indefinitely. A GET can incorrectly spend its entire deadline waiting for this idle slot. This is different from the admitted limit that an actually busy resolver thread remains occupied until its own timeout.

**Fix:** make releasing capacity notify all queued slot waiters, or explicitly transfer a wake when a selected waiter exits without claiming capacity. Apply the same rule to failed helper creation. Keep capacity testing and reservation under the existing mutex. Add a deterministic pool test with controlled helper completion: the selected waiter exits, the next live waiter still acquires the available slot. No real DNS timing is needed.

### B2 — HTTPS requests larger than rustls's outgoing buffer fail before being sent

**Where:** `crates/tide-std/src/net/http.rs:306`–310.

`tls.writer().write_all(request)` buffers the entire HTTP request before `flush_tls` drains anything. Pinned rustls 0.23.46 defaults to a 64 KiB outgoing-record buffer (`src/conn.rs:500`–543); its `PlaintextSink::write` returns the number accepted, including zero when full (`:387`). A valid ASCII URL with, for example, a 128 KiB path fills that buffer. `write_all` then receives zero and returns `WriteZero`; `flush_tls` is never reached. The same URL can be sent over plain HTTP and simulation, and the admitted URL/GET contract specifies no such request-size restriction. This is a partial-write handling defect, not a server's 414 response.

**Fix:** write plaintext in a loop and drain encrypted records between partial accepts, observing cancellation and the call deadline as the socket writer already does. Do not remove the buffer limit to hide the issue. Add one local HTTPS test with a request target larger than the rustls buffer and assert that the server receives the complete target and returns its body.

### B3 — Header conversion rejects valid opaque fields and accepts invalid framing whitespace

**Where:** `crates/tide-std/src/net/http.rs:397`–404.

Every header value is converted with `from_utf8`, including headers the client otherwise ignores. Thus `HTTP/1.1 200 OK\r\nX-Note: \xff\r\nContent-Length: 2\r\n\r\nok` becomes `Io(InvalidData)` even though httparse accepts that opaque header value. The pinned parser deliberately permits field-value bytes 0x80–0xFF (`httparse/src/lib.rs:90`–92); the v0 ASCII restriction concerns URL input and hosts, not all response fields.

In the other direction, Rust's Unicode-aware `trim()` silently expands framing syntax. `Content-Length: \xc2\xa05` is decoded to a non-breaking space followed by `5`, then trimmed to `5` and accepted. That is not an ASCII decimal Content-Length with HTTP optional whitespace. The same problem affects Transfer-Encoding. NET-HTTP-2 requires invalid framing to be `Io(InvalidData)`.

**Fix:** leave unrelated field values as bytes. For the two framing fields, split/trim using only HTTP ASCII whitespace (SP/HTAB), then validate their ASCII grammar. Add a valid opaque-header case and a non-ASCII-whitespace framing rejection to the existing local response test.

### B4 — Chunked trailers bypass malformed-response validation

**Where:** `crates/tide-std/src/net/http.rs:478`–495.

The trailer reader treats every nonempty CRLF-terminated line as a valid trailer without parsing a field name, colon or value. For example, a 200 chunked response ending `0\r\nnot-a-header\r\n\r\n`, or `0\r\nX: \0\r\n\r\n`, returns `Ok`. Research/42 §3 requires malformed responses to produce `Io(InvalidData)`; discarding trailers does not make malformed framing valid. The local 64 KiB trailer check also runs only in the no-CRLF branch: a completed trailer block crossing that limit and ending in the latest read can return success without the limit being checked.

**Fix:** reuse `httparse::parse_headers` for the discarded trailer block and enforce its byte bound on both partial and complete parses before accepting it. Add one malformed-trailer rejection and a completed over-limit trailer case. This is not an unbounded-allocation finding: the current read chunk and partial-path check bound overshoot, but validation and the stated local limit are skipped.

### B5 — Scheduling inside a peer callback escapes the GET deadline

**Where:** `crates/tide-std/src/net/sim.rs:598`–619; existing callback regression at `crates/tide-std/src/net/tests.rs:419`–447.

Register a peer that captures `sim.clock.clone()`, calls `clock.sleep(Duration::from_secs(1)).unwrap()`, then returns `Reply::ok("x")`. Call GET with a deadline 30 ms from now in a scope with no scope deadline. After the request packet arrives, the callback runs on the requesting task, but its sleep sees no GET deadline. Virtual time advances to roughly one second; only after the callback returns does `arrive` notice expiry and return `Timeout`. A callback waiting on an otherwise unready simulated resource can instead produce a deadlock report because no GET timer remains registered while it runs. Research/42 §§3/9 promises a whole-call deadline and timeout at that deadline. The existing regression explicitly allows scheduling callbacks, but tests expiry through `s.set_deadline`, which cancels the scope and does not exercise an operation deadline.

**Fix:** ensure the active GET deadline also bounds cooperative waits made while executing its peer callback, preserving the distinction between operation `Timeout` and outer-scope `Cancelled`. Add the one-second callback/30 ms GET test without setting a scope deadline, and assert timeout at 30 ms. A callback's ordinary computation is not the issue; this case yields to the existing scheduler.

### B6 — A peer capturing Net permanently retains its simulation and resources

**Where:** `crates/tide-std/src/net/sim.rs:523`, `:542`; `crates/tide-std/src/sim.rs:23`–35, `:51`–71.

`World::http` accepts an unrestricted Send closure. A normal peer that forwards to another peer can capture `let net = sim.net.clone()`. Storing that closure creates a strong cycle: `SimNet.http -> Peer -> callback -> Net.sim -> SimNet`. Capturing a simulated TcpStream or TcpListener can produce the same cycle through its `state`. There is no HTTP peer teardown in SimRoot, Simulation or SimNet. Dropping the root therefore cannot drop these callbacks, their captured payloads/handles, or the runtime retained by their Net/Network. Repeating such a setup across property-test seeds leaks each world's state, including on unwind. Moving/restoring the callback around invocation does not break the stored cycle.

**Fix:** give the simulation lifetime an explicit teardown owner that clears the peer registry before the simulation ends, including unwinding; a root-owned field guard can preserve the existing ability to move fields out of SimRoot. Do not impose a new ban on capability captures in peers. Add a drop-counter or weak-reference regression in which a registered callback captures Net and verify that its captured resource is released when the root ends, both normally and on panic.

### B7 — Non-2xx responses skip the admitted body-failure rules

**Where:** `crates/tide-std/src/net/http.rs:425`–427.

The reader returns `Status(code)` immediately after framing-header validation for every non-2xx status except the earlier 304 special case. For a 404 with `Content-Length: 5` followed by only `ab` and EOF, it returns `Status(404)`, never `Reset`. A non-2xx close-delimited HTTPS body ending without `close_notify` also returns Status, and an oversized error body bypasses the body-limit check. Research/42 §3 step 9.4 explicitly applies framing to “every other status”; its body-size and premature-close rules precede step 10's Status result. NET-ERR likewise calls a close before the framed body is complete Reset. The builder's report records early Status as an implementation decision, but that report does not revise the admitted owning contract.

**Fix:** keep the 204/304 bodyless precedence, otherwise validate/read or discard the framed body under the existing size, cancellation, TLS-end and deadline rules before selecting Status for a non-2xx final response. Reuse the current reader rather than adding a second framing implementation. Add a truncated non-2xx framed response and a non-2xx close-delimited HTTPS truncation witness. If early Status is intended instead, it needs an explicit owning-contract change covering these failure precedences and limits, rather than an implementation-only decision.

### B8 — The new HTTP manifest path has no replay/identity negative control

**Where:** `crates/tide-std/src/net/tests.rs:406`–415; `crates/tide-admit/tests/build.rs:4148`–4159; `crates/tide-std/src/sim.rs:160`; `crates/tide-std/src/replay.rs:225`.

The test named `simulated_get_cancellation_wins_and_replays` creates two fresh roots and compares trace strings. The companion build test executes the test binary directly. Neither enters the driver's recording/`--replay` path, inspects an `http` manifest input, or proves refusal when its profile/address changes. Removing `World::http`'s `replay_input` call would leave both tests passing, despite violating research/42 §9's profile/peer identity contract. The new allowed manifest kind and its end-to-end wiring are therefore not protected by the claimed replay checks.

**Fix:** add one driver-backed HTTP recording/replay integration test, following the existing `a_proc_seed_replays_and_refuses_changed_model_data` at `build.rs:3833`: inspect the inline peer address and `net-http-v0` value, replay and compare trace/output bytes, then reseal a manifest with a changed profile or peer address and assert the replay root/input refusal. Include a broken control showing that omitting the HTTP input is detected. This tests the changed boundary rather than duplicating the existing seed-determinism test.

## Non-blocking findings

None separately. No style-only changes or new dependency are requested. The stated rebinding, resolver-cache/alias, bind/accepted-peer, payload and application-peer/TLS-fidelity limits are not findings.

## Reasoning

The observable contract is one `Net::get(&Url, deadline)` that checks cancellation, URL shape and NET-HOST authority before resolution or a simulated peer, performs one HTTP(S) GET without redirects, returns the specified body/error, closes its connection, and has a deterministic simulated counterpart. Stage 1 admitted that contract subject to its stated limits; it did not admit extra ambient network paths.

**Authority:** I found no concrete unlisted-destination or capability-free lookup route in this candidate. Url is a private-field newtype with data accessors and Decode through parse, with no socket_addrs, Deref or inner-URL conversion. Its raw-host extraction follows the WHATWG preprocessing and authority delimiters used in the inspected pinned parser. Host percent escapes, non-ASCII, empty labels, xn-- labels, wildcard entries, NUL and zone identifiers are refused before a lookup. Entry and address keys normalize numeric IPv4, mapped IPv6, case and one trailing dot; scheme default ports are used for GET. Userinfo and unsupported schemes fail at the shape step. `only` intersects, and `no_bind` refuses before address resolution. The key supplies DNS, dialing, TLS identity and Host. `=` is not a separator in this path and does not create a second interpreted destination. Bind and accepted connections intentionally retain the documented rights.

The net/host/resolve/http backend modules remain private and their public reachable surface has no resolver or TLS/socket constructor escape. The new public data types hold no connection; the existing Net/TcpStream/TcpListener/Incoming owners remain in NOT_PURE. Admission still denies the runtime entry, applies TD0501 to ambient Rust networking, and admits Tide's guarded facade. The new trust-diff test proves a declared `net` path while Url adds no authority; it intentionally does not claim exercised-authority reporting. Existing ambient-net and runtime-denial tests passed in the coordinator's workspace log.

**Real backend:** socket ownership is RAII through failed connection attempts, TLS construction/handshake, request and response errors; the resolver's abandoned helper lifetime is the design's bounded, stated exception. Nonblocking read/write retries check cancellation/deadline and retry Interrupted; connection attempts check their deadline, and mixed failures preserve the first non-refusal error. Partial TLS request writes need B2. The 64 KiB aggregate interim/final head-byte bound and 64 MiB successful-body bound are present; chunk data buffers are drained and retained memory is bounded. B3/B4/B7 concern response validity and result precedence, not an invented unbounded body-buffer claim. Bodyless statuses, informational heads, rejected 101, chunked decoding, complete framed TLS bodies, and close_notify for a close-delimited successful body have meaningful local-server tests. There is no new unsafe block in the HTTP/resolver/simulation implementation.

**TLS and dependencies:** both manifests pin rustls 0.23.46 with default features off and ring/std/tls12, ring 0.17.14, webpki-roots 1.0.12, httparse 1.10.1 without defaults, url 2.5.8 and idna_adapter 1.0.0, matching research/42 §8. Parsed both lockfiles: shared external package versions agree. All listed proposed transitive versions are present; the additional utf_types is utf8_iter's dependency. Serde/derive/syn and Windows/WASI lock entries are explained by optional or target-specific upstream metadata, rather than an unexpected ICU or second TLS provider. The selected idna_adapter has no dependencies and its inspected implementation is the ASCII stub; the pre-parser rules keep HTTP host parsing in its supported domain. The separate 1.1.0 log runs the same three host tests successfully; the delivered lockfiles and final Net log return to 1.0.0. Configuration explicitly installs ring, compiled-in roots, NoKeyLog, disabled resumption and HTTP/1.1 ALPN, with no public root override. No proxy, native-root, key-log or credential-file integration is present. Nix and Docker derivation logs each explicitly show ring and tide_std compiling and reaching Finished; these observations corroborate delivery builds without rerunning them or treating the builder's prose as proof. This is not a new complete RustSec audit.

**Simulation and Proc:** HTTP uses the existing net RNG, waits, signals, task identity and trace owner; registration does not draw or schedule. Domain misses, unserved IP/localhost ports, collisions with TCP listeners, two packet delays, fragments, Reply variants and body limits have seeded assertions. No-get code has no new HTTP draw. Public Net and TCP wrappers validate world context before operations, address/debug access and ended incoming iterators; existing drop uses the live-world closing guard. The two Host::Ip unreachable arms are unreachable because literal() returns Some for every Ip; the representable-maximum Hang case no longer panics. Callback restoration/waking on panic fixes the mutex-held-across-yield defect but does not supply B5's deadline or B6's teardown. Proc's three changed callers pass None to the new socket-wait parameter; its scope cancellation semantics remain owned by the unchanged runtime. The coordinator's Proc replay/build regressions passed.

**Observed checks:** I independently ran `scripts/count-tokens.sh design/reference.md </dev/null`: exit 0, **24,968**, matching the companion and below D14's 25,000 limit. `git diff --check 0e38e55..7e86e6b` passed. Read the focused Net/admission/conformance logs and the separate adapter and delivery logs; their positive cases pass, but none exercises B1–B8's distinguishing counterexamples. The coordinator's `target/work/suites/candidate.txt` names the exact full candidate; `test.start` is 2026-11-18T20:45:40Z, `test.end` is 20:50:42Z, and `test.exit` is **0**. The workspace log has no failed tests. At this report's preparation the complete conformance run was still appending PASS lines and had no completion marker; I do not claim its final result. All build/test execution described here belongs to those existing runs, not this reviewer.

Not checked: no new Rust reproducer, fault injection or heavy build was run; the blocking scenarios are source traces requiring the focused regression checks above. Full conformance completion and the subsequent Opus verdict remain outside this pass's observed proof.
DONE
