Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate dd1de77 (design, docs)

## Verdict

**Changes required.** The direction fits A1 and the existing API, but the design needs the six corrections below before the next review. Confidence: high for the concrete protocol and documentation findings; dependency implementation and runtime behavior remain unproved at this docs-only stage. No owner override.

## Scope

Read-only supervisor review of fx-ff13 stage 1, branch `fx-ff13`, full candidate HEAD `dd1de7730d34303f0277f6e231275ee06a284768`, against base `1a5e5fd`. SHA-256 of the exact output of `git diff 1a5e5fd..HEAD`:

`91e2720d89d09a6af37c16dffd7933b6df1314a9585ceb8489765cf05715dded`

Reviewed all five changed files, the uncommitted reference proposal, the builder report as claims, the existing TCP implementation (`net.rs`, `net/real.rs`, `net/sim.rs`), admission/re-export and Pure ownership, the existing HTTP examples, simulation/replay contracts, GOVERNANCE.md and repository instructions. Read `br --no-db show fx-ff13`, `br --no-db comments fx-ff13`, and `br --no-db show fx-6d50`. A1 authorizes admission conditional on sound design and a red team; this review neither substitutes for that red team nor approves implementation or landing.

The separately reviewed proposal is `target/work/reference-proposal.diff`, SHA-256 `9cd061a700285a01194541c78aca5be99792a768e0bfc33eb46374fb8bfe148c`. Applied it only to `target/work/supervisor-reference/design/reference.md`. No tracked file, commit, or Beads state changed.

## Blocking findings

1. **The pre-parser restrictions do not guarantee ASCII-only, non-IDN hosts.** `research/42-net-get-v0.md:86–93`, `:248–260`, `:455–461`; `design/03-authority-concurrency.md:223–227`.

   NET-HOST explicitly puts the v0 restrictions before the special-scheme host parser. That parser percent-decodes the host before IDNA. Thus `http://b%C3%BCcher.example/` is entirely ASCII input with no raw `xn--` label, but becomes `xn--bcher-kva.example`; `http://%78n--bcher-kva.example/` hides the forbidden prefix; and `http://%EF%BD%81pi.example.com/` normalizes a fullwidth character to `api.example.com`. The assertion that the restrictions guarantee IDNA sees only ASCII labels is false. Merely checking the resulting `xn--` prefix misses Unicode characters that map to ASCII. The declared empty-label rejection is also an extra Tide restriction, not something the WHATWG parser supplies.

   Evidence: [WHATWG host parsing](https://url.spec.whatwg.org/#host-parsing) and [url 2.5.8 host implementation](https://github.com/servo/rust-url/blob/v2.5.8/url/src/host.rs). A local Node WHATWG-URL observation produced those three results and accepted `a..b`; this corroborates the standard, not a claim that Tide or the Rust implementation was executed. The [adapter's own guidance](https://github.com/hsivonen/idna_adapter) confirms that the stub changes non-ASCII processing.

   **Minimum fix:** specify the validation boundary before IDNA: reject non-ASCII percent-decoded host bytes and decoded `xn--` labels, or simply reject percent escapes in the host for v0; retain percent encoding in paths/queries. Explicitly enforce empty-label rejection. Carry the rule into NET-HOST, Url parsing, and NET-HTTP-1 with these negative cases. Keep the existing single parsed host as the owner of authorization and dialing.

2. **HTTPS does not prevent reaching a rebound address.** `research/42-net-get-v0.md:327–338`; related teaching at `design/03-authority-concurrency.md:232–235`.

   The statement “rebinding reaches only servers that hold” a certificate is false at the network boundary. If `api.example.com` resolves to `127.0.0.1`, a permitted `get` connects to loopback port 443 and sends a TLS ClientHello before certificate verification can fail. HTTPS protects acceptance of the peer and transmission of the HTTP request; it does not prevent that connection or handshake traffic. Moreover the same `Net` permits raw `connect("api.example.com:443")`, which has no TLS requirement. Recommending HTTPS without distinguishing these boundaries overstates the mitigation.

   Evidence: the candidate's own ordered connection/TLS steps at `research/42-net-get-v0.md:124–133`, and the [TLS handshake ordering](https://datatracker.ietf.org/doc/html/rfc8446#section-2).

   **Minimum fix:** state that DNS grants permit TCP contact and handshake traffic to any resolved address on the granted port, under HTTPS too. State separately that a completed HTTPS GET authenticates the name before sending HTTP data. Explain that callers with this Net can also use raw TCP. No private-address filter is required: the unrestricted resolution is already a deliberate, stated limit.

3. **The response algorithm mishandles bodyless statuses, and the simulator admits impossible final statuses.** `research/42-net-get-v0.md:150–166`, `:294–303`, `:488–501`; `conformance/rules.md:97–98`.

   The generic chunked/Content-Length/close algorithm omits the higher-priority no-body rules for 204 and 304. A legal `304` with `Content-Length` describing the selected representation, followed by close, becomes `Reset` under the written algorithm instead of `Status(304)`. A 204 on a connection not yet closed can wait until `Timeout`, although its response is complete at the header terminator. Separately, `Reply::status(100)` is expressly accepted by the simulator and mapped to `Status(100)`, while the real algorithm skips interim 1xx and waits for a final response. A lone 101 also needs a deliberate unsupported-upgrade result rather than being treated as an ordinary interim response.

   Evidence: [RFC 9112 §6.3](https://www.rfc-editor.org/rfc/rfc9112#section-6.3) gives status-specific framing precedence. The cited TE/CL and invalid-length claims are correct, but they are not the whole framing algorithm.

   **Minimum fix:** add bodyless-status precedence before generic framing, define rejection of an unsupported 101 upgrade, and restrict simulated final `Reply::status` to the supported final-status range (or model interim replies separately, which this scope does not need). Require stage-2 cases for 204, 304 with Content-Length, and a lone interim/upgrade response, on the appropriate boundaries.

4. **The dependency gate rejects an existing transport boundary on a false premise.** `research/42-net-get-v0.md:447–453`.

   The assertion that cancellation and simulation cannot reach inside ureq because it owns sockets is not true of current ureq. It exposes custom connectors/transports and a resolver. Its `ureq-proto` component specifically owns HTTP/1.1, chunked transfer and 100-continue while excluding sockets, TLS and routing. This is directly relevant to the proposed Tide-owned response/framing code. Default proxy discovery is real, but configurable behavior is not an unavoidable property of every use of the library.

   Evidence: [ureq transport](https://docs.rs/ureq/latest/ureq/unversioned/transport/index.html), [its API stability limitation](https://docs.rs/ureq/latest/ureq/unversioned/index.html), [ureq-proto scope](https://docs.rs/ureq-proto/latest/ureq_proto/), and [proxy configuration](https://docs.rs/ureq/latest/ureq/config/struct.ConfigBuilder.html#method.proxy).

   **Minimum fix:** compare this actual sans-I/O alternative against owning HTTP framing over httparse. Keeping httparse can be justified by dependency weight, policy adaptation, or API stability, but replace the categorical impossibility claim with the real tradeoff. No dependency switch is demanded by this finding.

5. **The admission record does not identify exact dependency pins.** `research/42-net-get-v0.md:420`, `:427`, `:439`, `:447`, `:455–470`.

   The record selects `rustls 0.23`, `ring 0.17`, `webpki-roots 1.0`, `httparse 1.x`, and `url 2.5`, then says every version is pinned with `=`. None is an exact release identity. A future implementation can choose different releases and dependency graphs while claiming to implement this admission unchanged. The IDNA adapter is also an explicit unresolved implementation check, not an already proved graph reduction.

   **Minimum fix:** name the proposed exact releases and feature choices in this docs-only admission record, distinguish direct dependencies from provider/transitive selections, and label lockfile/build/equivalence proof as stage 2. Record the adapter alternatives as conditional exact versions. No Cargo change or compile is requested in stage 1. The observed license families are correct; the missing pin identities prevent attaching that verification to a reproducible chosen graph.

6. **The built-TCP description presents the future host rule as current behavior.** `design/03-authority-concurrency.md:203–218`.

   This paragraph describes the implemented TCP surface and its current resolver limitations, but now says its grants use NET-HOST. Current `Address::key` only lowercases names or parses Rust IP literals; it neither folds numeric aliases nor strips trailing dots nor folds mapped IPv6. The research correctly labels those changes stage 2 at `research/42-net-get-v0.md:276–283`. Readers of the owning chapter receive conflicting availability information.

   **Minimum fix:** keep the current literal-key behavior explicitly labelled as current in the TCP paragraph and label NET-HOST's adoption by TCP as planned for stage 2. The eventual design can remain decided v0. Update the resolver-limit retirement wording to the same stage, rather than implying it must wait for the separate public resolve bead.

## Non-blocking findings

7. **The two pending reference changes need another trim before both land.** `target/work/reference-proposal.diff:22–26`; `research/42-net-get-v0.md:574–581`.

   I independently ran `scripts/count-tokens.sh` on the tracked reference and on the scratch copy with this proposal applied: **24,961** and **24,988**, respectively. The proposal is **+27** and independently passes the 25,000 limit. The supplied Proc candidate takes main to 24,985, a +24 change. Together the stated changes total **25,012**, so they do **not** fit. This is a coordination issue rather than a failure of this proposal against its frozen base; the builder already acknowledges a recount by the second landing.

   **Minimum fix:** remove at least 12 counted tokens from the combined reference, then rerun the official counter on the actual combined candidate. The arithmetic uses the supplied Proc count; I did not apply or recount that other bead's patch.

## Reasoning

The smallest observable contract remains the already taught `Net::get(&Url, deadline: Instant) -> Result<Vec<u8>, NetError>`: one HTTP/1.1 GET over HTTP or authenticated HTTPS, explicit capability, whole-operation deadline and cancellation, 2xx body or the specified error. The signature, bounded-fetcher teaching, error variants, and transient classification agree across the decision record, chapter, companion and proposal. Research/42 clearly calls stage 1 docs-only; the current-TCP availability wording is the exception in finding 6. The conformance additions are honestly marked planned. No implementation test can establish this new behavior yet.

**Authority assessment.** Subject to finding 1, one canonical key, exact required port, no wildcard/subdomain expansion, case folding, trailing-dot folding, numeric IPv4 normalization, mapped-IPv6 folding, and checking before resolution provide a coherent destination policy. Refused names get no lookup or peer callback. No redirect following, other schemes, ambient proxy configuration, credential files, system roots, persistent connection/cache state, or TLS resumption is admitted. A private Url wrapper with only data accessors is the right boundary: [url's socket_addrs](https://docs.rs/url/2.5.8/url/struct.Url.html#method.socket_addrs) really performs resolution, and admission's re-export policy makes public surface review necessary.

`only(["api.example.com:443"])` is not a guarantee that all physical network traffic reaches one fixed public IP or a TLS server: the name may resolve anywhere, operator resolver/search/alias configuration affects lookup, an allowed service may relay traffic, and raw connect can send arbitrary bytes on that destination grant. Paths and queries can exfiltrate data, response bytes are untrusted payload, and bind/accepted peers are explicitly outside `only` in 03. These are stated limits, not reasons to impose a different authority model; finding 2 corrects the remaining overstatement about HTTPS. The bounded resolver pool and abandonment limit are acknowledged costs. The shared private resolver/key are suitable constraints for fx-6d50, without choosing its public error type here.

**External evidence audit.** The security examples were verified against publisher advisories: [CVE-2025-21620](https://github.com/denoland/deno/security/advisories/GHSA-f27p-cmv8-xhm6) concerns cross-origin Authorization forwarding; [CVE-2026-49411](https://github.com/denoland/deno/security/advisories/GHSA-v8fw-85r8-5m23) concerns numeric aliases in Node TCPWrap and was fixed in 2.8.0; [CVE-2026-49859](https://github.com/denoland/deno/security/advisories/GHSA-cpgj-f7g3-2pp2) and [CVE-2026-49860](https://github.com/denoland/deno/security/advisories/GHSA-83pc-3rw9-qpwj) concern missing resolved-IP deny checks in fetch and WebSocket, fixed in 2.8.1. [Deno permissions](https://docs.deno.com/runtime/reference/permissions/#network-access) support the stated optional ports and wildcard behavior. [CERT's httpoxy account](https://kb.cert.org/vuls/id/797896) supports the HTTP_PROXY example and CVE-2016-5385 family. NVD's rendered pages did not supply readable details in this environment; the primary advisories independently verified the claims.

The special-scheme IPv4 number forms and URL serialization claims are supported by WHATWG/url source, with the restriction correction in finding 1. [HTML origin comparison](https://html.spec.whatwg.org/multipage/browsers.html#origin) supports the origin analogy. The Wasmtime hook example is corroborated by the [original issue](https://github.com/bytecodealliance/wasmtime/issues/10527), and wasmCloud's allowlist by its [official documentation](https://wasmcloud.com/docs/kubernetes-operator/workload-security/); those primary pages were used where the cited archive/mirror was inaccessible.

**Dependency assessment.** The capability contracts and replacement boundaries are named. The selected approach avoids owning TLS and URL parsing while fitting cancellable sockets. Verified source/package metadata for rustls 0.23.35 and 0.23.46 gives `Apache-2.0 OR ISC OR MIT`; ring 0.17.14 gives `Apache-2.0 AND ISC`; webpki-roots 1.0.12 gives `CDLA-Permissive-2.0`; url 2.5.8 and httparse 1.10.1 give `MIT OR Apache-2.0`. These observations verify the stated families, not unspecified exact future pins. [Rustls's I/O boundary](https://docs.rs/rustls/0.23.35/rustls/) supports supplied transport and TLS 1.2/1.3; its provider/features match the proposed default-features-off configuration. [Prossimo](https://www.memorysafety.org/initiative/rustls/) supports the maintenance claim. [Ring's build instructions](https://github.com/briansmith/ring/blob/main/BUILDING.md) support the packaged Linux C-toolchain requirement; [AWS-LC requirements](https://aws.github.io/aws-lc-rs/requirements/index.html) support the FIPS CMake/Go claim.

[webpki-roots source](https://github.com/rustls/webpki-roots) supports compiled Mozilla/CCADB roots. [rustls-native-certs source](https://github.com/rustls/rustls-native-certs/blob/main/src/lib.rs) confirms SSL_CERT_FILE/DIR behavior; [native-tls](https://docs.rs/native-tls/latest/native_tls/) confirms its Linux OpenSSL backend. The adapter's published documentation confirms 1.0 stub, 1.1 unicode-rs, and 1.2 ICU4X backends. [httparse](https://github.com/seanmonstar/httparse) and its [chunk-size API](https://docs.rs/httparse/1.10.1/httparse/fn.parse_chunk_size.html) support head/chunk-size parsing; body framing still belongs to Tide and needs finding 3. Hyper's async adaptation cost is a reasonable rejection. The comparative maturity of crypto providers is an engineering judgment, not independently demonstrated audit equivalence. The ureq impossibility claim and exact pin record need findings 4–5.

**Simulator/replay assessment.** The two packet delays, existing net sub-stream, callback on the requesting task, error peers, cancellation/deadline order, payload-free trace, and no-get draw preservation fit the existing model. Inline profile/peer identities plus the existing source/build/config identity provide a plausible REPLAY-1/2 design; closure source alone is not the full identity, but the existing manifest also records arguments, environment and other simulated inputs. Fault injection and drop_all remain explicitly planned. The final/interim-status mismatch in finding 3 must be fixed before backend agreement is claimed. Replay fidelity, sandbox behavior, TLS identity coverage and the public-module guard require stage-2 proof, not another docs-only assertion.

The admission case is honest about the major implementation costs: HTTP framing, TLS and C build support, resolver pool, simulation and replay fields. It does not widen the language or add a second authority kind. Those are sound reasons to continue after the specific corrections, not a basis to approve the current candidate as already sound.

Not checked: no Rust compilation, network implementation tests, red-team pass, or Opus review; this pass reviews a design only. The combined Proc/reference count is arithmetic from the supplied Proc measurement, not a combined-patch count.
DONE
