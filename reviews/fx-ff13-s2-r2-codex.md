Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate abc1e4d (code)

## Verdict

**Changes required.** Round 1's B1–B8 are resolved, with distinguishing regression tests and observed passing results. The full fresh review finds three further correctness defects: incomplete validation of chunk-size lines, acceptance of invalid status codes, and incorrect simulated arrival arithmetic at the representable time boundary. The first two affect the untrusted HTTP response boundary. Confidence is high on the source traces below; no new Rust reproducer was executed. No override or landing approval.

## Scope

Full fresh, read-only supervisor pass for fx-ff13 stage 2. Worktree `/mnt/build/eon-agents/worktrees/tide-fx-ff13`, branch `fx-ff13`, HEAD `abc1e4d0aac624984f5238e4b834cac2f8764379`, base `0e38e552c0137ba6682804a2279621d6f5d212b0`. Observed before and again at report preparation:

```text
git diff 0e38e55..abc1e4d
34 files changed, 2851 insertions(+), 187 deletions(-)
SHA-256: 6e4f67cd3f2779be3acb7b219a060c07bcbd4d68c59b210050d6b8ebdad29d9c
```

The round-1 fix diff `7e86e6b..abc1e4d` is eight files, +316/-34. Tracked worktree status is clean; no comparison against moving main is used.

Reviewed the complete change, affected callers and surrounding runtime/admission ownership, the stage-1 contract in research/42 and design/03, companion 15.2/17/21, reference changes, conformance rules, stage-1 and red-team reviews, GOVERNANCE.md, BR_WORKFLOW.md and `br --no-db show fx-ff13`. Inspected the relevant pinned dependency sources and both lockfiles. The builder report supplied leads, not independent proof. fx-6d50's future public resolution API remains out of scope.

No source, test, planning state, commit, dependency installation or heavy run was changed or started. This requested report is the sole authored artifact. The token-count script was explicitly requested and compiles no Tide code.

## Blocking findings

### B9 — A chunk-size line with no digits is accepted as the final zero chunk

**Where:** `crates/tide-std/src/net/http.rs:460`, `:473`–475.

A peer can send:

```text
HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n\r\n\r\n
```

After the valid `abc` chunk, there is an empty size line followed by an empty trailer block. Pinned `httparse::parse_chunk_size` initializes `size` and `count` to zero, accepts CRLF without requiring a digit, and returns `Complete((2, 0))` (`httparse-1.10.1/src/lib.rs:1267`–1270, `:1311`–1315, `:1338`). Tide treats that as a complete terminating chunk and returns `Ok(b"abc")`. A line such as `;x=y\r\n` also supplies no size and follows the same route. RFC 9112 chunk syntax requires hexadecimal size digits; research/42 §3 requires malformed responses to be `Io(InvalidData)`. This is an invalid framed body being accepted as complete, including on HTTPS where a complete framed body deliberately needs no `close_notify`.

The same parser explicitly skips arbitrary octets after `;` (`:1327`–1332). Thus a line `1;x=\0\r\n` also passes Tide despite its invalid chunk-extension syntax. Validating the new trailer block does not validate these earlier framing bytes.

**Suggested fix:** retain httparse for size conversion, but validate the chunk-size line's required hexadecimal prefix and extension syntax before using its result as framing. Add local response-reader cases for a missing size and an extension containing NUL, alongside a valid extension control. Both malformed cases must give `Io(InvalidData)` rather than success. This needs no dependency or second body reader.

### B10 — Invalid status codes escape the malformed-response rule

**Where:** `crates/tide-std/src/net/http.rs:391`–400, `:433`–437.

`httparse` validates three decimal digits, not the HTTP status-code range: its `parse_code` at `httparse-1.10.1/src/lib.rs:933`–941 accepts `000` through `999`. Tide applies no range check. For example:

```text
HTTP/1.1 000 Invalid\r\nContent-Length: 0\r\n\r\n
```

returns `Status(0)`. Replacing `000` with `600` returns `Status(600)`. These are invalid HTTP status codes, rather than valid non-2xx final responses; the admitted malformed-response result is `Io(InvalidData)`. A code outside the range with no framing and a peer held open can instead consume the deadline before Tide ever reports anything about the invalid head. The simulated constructor already refuses 600, so it cannot model this real-backend result.

**Suggested fix:** reject a parsed code outside 100–599 immediately after parsing the head, before informational/bodyless/framing decisions. Keep unrecognized codes inside that range working under the existing class/framing rules. Add one table-driven local-server regression for `000`, `099`, `600` and `999`, plus an unrecognized valid final status control.

### B11 — Simulated arrival overflow can panic or turn a late reply into success

**Where:** `crates/tide-std/src/net/sim.rs:576`–580, `:593`, `:597`, `:631`.

Two public-input cases demonstrate the inconsistent arithmetic:

1. At virtual time zero, register a peer returning `Reply::ok("late").after(Duration::MAX)` and pass `sim.clock.now() + Duration::MAX` as GET's deadline. The request consumes a positive packet delay. Adding the reply delay and its packet delay therefore exceeds the largest representable instant. Line 631 saturates arrival to `Duration::MAX`; at that instant `arrive` checks `now >= at` before expiry and returns success. GET returns `Ok("late")`, although its actual arrival is later than the finite deadline. The same defect can return Status or Reset instead of Timeout.
2. Register a peer, advance the clock to `Duration::MAX - Duration::from_nanos(1)` with `sleep_until`, and call GET with the last representable instant as its deadline. The deadline is still in the future. Line 593 adds a 1–10 ms request latency with ordinary `+` and panics on Duration overflow instead of timing out. The no-peer response addition at line 597 has the same unchecked operation.

These inputs are representable through the public Clock/Instant API. Plain Python arithmetic using the inspected seed-0 net sub-stream yields first packet delays of 7,322,485 ns and 3,446,026 ns and confirms both overflows; this is arithmetic corroboration, not a Tide execution. Research/42 §§3/9 promises Timeout when the whole-call deadline prevents arrival. The special `Answer::Hang => Timeout` at line 640 fixes only the earlier reachable-unreachable case.

**Suggested fix:** handle all request/reply arrival additions consistently. An unrepresentable arrival must remain unable to complete before the deadline: wait cancellably to the deadline and return Timeout, rather than overflow or treat a saturated instant as an arrived reply. Keep cancellation precedence. Add the two boundary cases above, with a control for an ordinary representable arrival. No change to the general scheduler is needed.

## Non-blocking findings

None separately. No style changes, new dependency, public API expansion or out-of-scope planning write is requested. The documented rebinding, resolver cache/alias/continuing-traffic, bind/accepted-peer, payload and simulator TLS-fidelity limits are not findings.

## Reasoning

### Round-1 dispositions

The IDs B1–B8 refer to `target/work/s2-r1-codex.md`; B9–B11 above are new findings from this full pass.

| Finding | Disposition and distinguishing regression |
| --- | --- |
| B1 | **Fixed.** `Pool::release` drains and wakes every waiter under the capacity mutex, on helper completion and failed creation. `released_capacity_survives_cancelled_selected_waiter` uses controlled local capacity and an abandoned last waiter, then proves that the live waiter acquires the slot. |
| B2 | **Fixed.** TLS plaintext writes accept a partial prefix and flush ciphertext before accepting more. `tls_get_sends_a_request_larger_than_the_plaintext_buffer` verifies receipt of the complete 128 KiB target over local HTTPS. |
| B3 | **Fixed.** Unrelated header values stay opaque; only framing fields are decoded, trimmed with SP/HTAB and checked against the framing grammar. `real_get_keeps_opaque_headers_and_rejects_non_ascii_framing` distinguishes valid 0xff, invalid NBSP framing and valid ASCII OWS. |
| B4 | **Fixed.** Discarded trailers use httparse field parsing, with the byte limit checked on both partial and complete blocks. `real_get_rejects_malformed_chunked_trailers`, `completed_trailers_still_obey_the_byte_limit` and `partial_and_many_trailer_fields_remain_valid` cover malformed, prebuffered oversize, split opaque and 101-field cases. B9 concerns the separate chunk-size path. |
| B5 | **Fixed for the reported mechanism.** The callback executes in a private child scope whose deadline bounds cooperative waits without cancelling the calling scope. Cleanup restores the callback and wakes contenders before result selection. `simulated_peer_waits_obey_the_get_deadline_without_cancelling_the_scope` checks sleeping and unready-channel callbacks at exactly 30 ms, subsequent outer checkpoints, and ordinary panic/restoration across eight seeds. The existing concurrent-callback/outer-cancellation regression also passes. |
| B6 | **Fixed.** A root-owned field guard clears the HTTP registry before Simulation ends, on normal exit and unwind; callbacks drop outside the SimNet lock. `simulated_peers_release_captured_capabilities_on_root_exit_and_unwind` uses a captured Net and weak-observed payload to prove release in both cases. |
| B7 | **Fixed.** The same body reader runs before non-2xx Status selection; 204/304 retain their earlier bodyless precedence. `non_success_responses_still_validate_the_body` covers truncated and oversized 404 bodies and close-delimited HTTPS with/without close_notify. |
| B8 | **Fixed.** `an_http_seed_replays_and_refuses_changed_or_missing_peer_identity` enters the driver, inspects inline address/profile, compares replay roots, trace bytes and output bytes, and refuses changed profile, changed address and missing input. The deliberate omitted-recording control fails at `HTTP identity recorded inline`. |

I inspected these tests and the coordinator's exact-candidate workspace log, which reports every named regression passing. `s2-r1-before.log` independently shows the eight intended failures for B1–B7 (two B4 cases); `s2-r1-http-replay-broken.log` shows B8's recording omission failing. The focused logs also show 41 passing net/simulation tests, the extended callback test passing, and the four build tests plus trust-diff driver test passing. No test described here was launched by this reviewer.

### Contract, authority and real backend

The observable contract remains one capability-checked, whole-call-bounded HTTP(S) GET, with no redirect, prescribed body/error results and deterministic simulated peers. The response reader retains aggregate head limits (100 headers per head and 64 KiB across interim/final heads), bodyless status precedence, both-framing/unsupported-coding refusal, bounded body/chunk storage, and authenticated end-of-stream for close-delimited HTTPS. B9/B10 identify validation that the chosen parser does not own. There is no new claim of unbounded response allocation or a destination escape.

I found no concrete route to an unlisted outbound peer or lookup without Net/an authorized handle. Url is a private-field newtype with only data accessors and Decode through parse, no socket_addrs, Deref or inner-URL conversion. The raw host restrictions precede the WHATWG host parser and IDNA; entries/addresses and GET use canonical keys for numeric IPv4, mapped IPv6, case and one trailing dot. Required/exact ports and scheme defaults are enforced; userinfo/other schemes fail at the shape step. Host, lookup, dial and TLS identity on GET derive from the checked key. No later parsing introduces another authority interpretation. `=` has no special destination-separator meaning in this path; NUL, percent escapes, wildcard entries and IPv6 zones are refused. `only` intersects; `no_bind` refuses before resolution. Bind and accepted connections retain their explicitly documented rights.

Backend modules/constructors are private, public net items are guarded, and no newly public type holds a socket. Existing Net/TCP/Incoming owners remain in NOT_PURE; the new Url/UrlError/NetError and Request/Reply are data. Admission still denies ambient networking and __rt, and the trust-diff regression names the new declared net path. It correctly claims declared, rather than exercised, authority. Live-world context validation precedes operations, accessors, attenuation, Debug and ended incoming iteration; no new foreign-world operation was found.

Sockets and TLS state are RAII-owned across connection attempts and error returns. Nonblocking read/write retries observe cancellation/deadline and Interrupted; TLS partial writes now drain between accepts. Resolver helpers remain bounded at eight, with intentionally abandoned lookups finishing under the system resolver's timeout; their continuing traffic is a stated limit. The two Host::Ip unreachable arms require literal() to return None for an IP, which it cannot do. No new unsafe block was introduced in the HTTP/resolver/peer implementation. B11 is a separate reachable arithmetic panic/result error.

### TLS, dependencies, simulation and documentation

Both manifests and lockfiles match the admitted direct pins: rustls 0.23.46 (defaults off, ring/std/tls12), ring 0.17.14, webpki-roots 1.0.12, httparse 1.10.1 (defaults off), url 2.5.8 and idna_adapter 1.0.0. Parsed lock inventories show no removed package or shared external-version disagreement. The transitive versions listed in research/42 are present; utf_types follows utf8_iter, and optional serde/derive/syn and other-target Windows/WASI entries explain the additional lock entries. There is no ICU, alternate TLS provider, proxy transport or native trust-store dependency. The cached 1.0.0 adapter is dependency-free, and the observed 1.1.0 experiment passes the same three host tests; the delivered graph uses 1.0.0.

TLS configuration explicitly selects ring, compiled roots, NoKeyLog, disabled resumption and HTTP/1.1 ALPN. There is no public root override or proxy/TLS-environment integration. Local certificate/SNI/truncation and environment-invariance tests pass. Existing Nix and Docker derivation logs show ring and tide_std compiling and finishing on the earlier candidate; they corroborate the dependency delivery gate, not a new delivery-build proof for changed code on abc1e4d.

HTTP simulation uses the existing scheduler, timer/signal machinery, net sub-stream and trace owner. Registration draws nothing; a run that never calls get gains no HTTP scheduling/draw. Reply outcomes have seeded value/timing assertions; HTTP recording now has the explicit driver-backed replay and negative controls described under B8. Callback serialization releases the host mutex before yielding and restores it on panic. The private scope bounds cooperative callback waits. Proc's three socket-wait adaptations pass None and retain their existing cancellation behavior; the coordinator's Proc build/replay regressions pass. I found no new Proc invariant break.

README, reference, companion, conformance facade and rules reflect implemented GET/HTTP peers and keep UDP/faults/nodes/drop_all planned. The conformance runner's hyphenated rule-ID parsing reaches NET-HTTP-N through the net_fetch corpus case. The crate-private URL codec helper introduces no authority path. `scripts/count-tokens.sh design/reference.md </dev/null` independently exits 0 with **24,968**, matching the companion and staying within D14's 25,000. `git diff --check 0e38e55..abc1e4d` passes.

### Observed proof and limits

`target/work/suites/candidate.txt` names the full abc1e4d SHA. The coordinator's workspace run starts at **2026-11-18T21:25:43Z**, ends at **21:29:59Z**, and records `test.exit = 0`. Summing the log's completed groups gives **454 passed, 0 failed**; all new regression tests pass. This is stronger than reusing 7e86e6b's 444-test result, but those tests do not cover B9–B11.

The current `conformance.log` is still growing, with PASS entries and no completion marker observed at report preparation. I do not claim the final full conformance result. The earlier focused net_fetch/TCP log has 2 PASS, 0 FAIL. Prior delivery builds, adapter comparison and red controls are identified above with their narrower surfaces; none is widened into an independently executed fresh run.

Not checked: no new Rust fault-injection/probe or heavy build was run. B9–B11 are source-level counterexamples that need the proposed regressions; full conformance completion and the subsequent Opus verdict remain unobserved.
DONE
