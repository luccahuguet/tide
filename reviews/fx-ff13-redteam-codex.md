Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Red team, candidate 9ad42b9 (design)

## Verdict

**Changes required.** One blocking misleading-authority claim: the new account of `only` omits bind-time DNS and the authority of accepted connections. The existing TCP paragraph deliberately preserves those operations outside `only`; the new decision and trust-diff explanation instead teach an exact destination boundary for the whole capability. This is fixable by stating the actual boundary, without changing the chosen HTTP API or imposing a new bind policy. I found no separate hole in the proposed `get` check, and no designed route to real networking without a capability or a previously authorized handle. Confidence: high on the documentation contradiction, conditional on stage-2 implementation for the proposed enforcement.

- Candidate HEAD: `9ad42b9c559c8313cffa0ac9192e065a83ebebf7`; base main: `d58f3e425914d717360e04d3d114a926aa7ed4f8`.
- SHA-256 of `git diff d58f3e4..HEAD`: `48e386714d056c32d840ff282fa92f9ca079c8a5896da91535538f52c0b099fa`.
- Reference proposal SHA-256: `071cb4b2c6e7ade038f5a02ef145a5547eb910097b508d37f5da1d601dffe42e`.
- Read the requested bead and comments with `br --no-db`, fx-6d50, AGENTS.md, GOVERNANCE.md, the complete candidate diff and research/42, the companion examples and affected chapter/reference rules, the existing TCP implementation and world boundary, admission/re-export handling, the two supervisor reports, and the Proc red-team precedent. This is an adversarial authority review, not a repeat of B1–B6 or the reference token count.
- Inspection only. No compilation, Tide execution, test, network probe, third-party target interaction, commit, or Beads write. The sole authored artifact is this requested report. Public documentation was read; pinned published crate sources were inspected in the existing Cargo registry cache. Stage-2 code does not exist in this candidate.

## Findings

### F1 — `only([])` can still send arbitrary DNS queries through `bind`; a destination-only trust reading is false

**Classification: misleading docs (blocking).**

**Where.** `design/01-design.md:512`–515 says the exact destination rule covers “every `Net` operation” and runs “before any lookup.” `research/42-net-get-v0.md:657`–668 assigns the destination-list meaning to the trust diff; its soundness reasoning at `:689`–698 says the remaining limits are the listed DNS/destination limits. `design/reference-companion.md:714`–720 and `target/work/reference-proposal.diff:33`–34 teach destinations without the bind exception. In contrast, `design/03-authority-concurrency.md:212`–215 explicitly says `only` does not limit bind or accepted peers. The code implements that exception at `crates/tide-std/src/net.rs:157`–170 and resolves at `:95`–111.

**Attack and concrete steps (paper trace, not executed).**

1. Main delegates `root.net.only(["api.example.com:443"])`, or even an empty list, to code it believes can contact only the printed destinations. It does not call `no_bind`.
2. The recipient constructs an ordinary ASCII hostname containing data it already possesses, such as `<encoded-data>.collector.example`, and calls `net.bind((hostname.as_str(), 8080))`.
3. `bind` checks cancellation and `may_bind`, then calls `Address::resolve`. It never checks `self.only`. The real resolver may query that name, exposing the data to the configured resolver and, with a suitable DNS delegation/cache miss, the name's DNS operator. A later `AddrNotAvailable` or DNS failure does not undo a query. No successful bind, HTTP server, redirect, `Proc`, or additional capability is needed.
4. Independently, `net.bind("0.0.0.0:0")` can create a listener and exchange arbitrary bytes with an arriving peer outside the list. This requires an actual inbound route and cooperating peer, whereas the DNS route does not require a successful listen.

**Impact.** A trust line such as `reaches: main: root.net.only(["api.example.com:443"])` does not promise that those entries bound all network destinations or all DNS names chosen by the code. Even the empty list is not network silence. The omission is consequential for the malware/exfiltration reading that motivates the bead. Today's simulator rejects general TCP names before making a real lookup, so a simulated bind failure would not establish the absence of this real-backend channel.

This is an existing, intentionally retained bind policy, not a newly demonstrated `get` bypass. The old TCP carve-out is correct. The blocking change is making the new whole-capability account and trust interpretation agree with it.

**Minimum design fix.** Preserve bind semantics and use this limit where `only` is taught, including the decision record, companion, proposal and research trust-diff section:

> `only` bounds outbound peer operations (`get`, `connect`, and the planned peer/resolve operations), not `bind` or accepted connections. A binding-capable Net can resolve arbitrary bind names, even if binding fails, and can communicate with unlisted inbound peers. Delegate `only([...]).no_bind()` for an outbound-only destination grant. An empty `only` list alone does not disable networking.

Qualify “every operation” and “before any lookup” accordingly. In the v0 trust-diff explanation, explicitly say a displayed `only` without `no_bind` retains these rights; this does not require implementing v1 exercised reporting. Preserve the existing early `no_bind` refusal before resolution when stage 2 moves resolver code. The cheapest falsifier for that boundary is a resolver-observation case showing `only([]).no_bind().bind(name)` performs no lookup; do not require `only([]).bind(name)` to refuse without changing the chosen policy.

### F2 — Simulated peers are not evidence of a real TLS identity or a wire-equivalent request

**Classification: limit to state (non-blocking).**

**Where.** `research/42-net-get-v0.md:150`–170, `:613`–635; `design/reference-companion.md:438`, `:698`–713. Section 9 deliberately models no TLS bytes, serves HTTP and HTTPS at the same peer, and exposes `Request::url() -> &Url` without defining fragment removal.

**Attack and concrete steps.** Register `world.http("allowed.test:443", |_| Reply::ok("ok"))`, then fetch `https://allowed.test/`. The simulated call can succeed even when a real server at that destination has no valid public-root certificate. Injecting `tls_error` can model failure, but success does not validate a certificate. Separately, give a peer a URL ending in `#secret` and let its callback branch on `req.url().fragment()`. If that is the original URL, the simulated peer observes information the real HTTP request never transmits and can return a response no equivalent real peer could choose from that request alone.

**Impact.** Tests can pass or fail for reasons absent from the real transport. Neither route reaches an ungranted host: steps 1–4 still precede peer lookup. These are simulation fidelity/interpretation limits, not capability holes.

**Minimum limit text.**

> An HTTP peer models an application response, not TLS certificate validation or wire parsing. Simulated HTTPS success asserts a successful authenticated transport; it does not prove one. `Request::url()` exposes the request URL with its fragment removed.

Removing the fragment gives the closer server model with no additional public API. Retain a real-backend certificate-failure check. The simulator must also enforce the already shared 64 MiB returned-body limit (`research/42:199`–201); `Reply::ok` is not an exception. A boundary-sized reply is a useful stage-2 witness, but the shared rule already requires it, so its absence from §9 is not a separate confirmed bypass. Raw malformed framing and interim-response coverage belong at the real response reader, as the design already recognizes.

### F3 — System resolution retains aliases, caches and traffic after caller cancellation

**Classification: limit to state (non-blocking).**

**Where.** `research/42-net-get-v0.md:421`–438, especially the categorical no-DNS-cache/no-cross-Net-state wording at `:423`–429 and the “always a granted name plus operator-chosen text” claim at `:436`–437; abandonment at `:449`–457.

**Attack and concrete steps.** Under an operator-supplied `HOSTALIASES` file, map an allowed single-label name `service` to an unrelated full hostname; resolve it through an allowed get/connect. The actual queried name need not contain `service` as a prefix. Run two otherwise separate Nets through a configured system name cache: the second lookup may reuse the first result. Start an allowed lookup and cancel the caller: the abandoned helper can still perform resolver retries after the Tide call returns.

**Impact.** The literal claims about wire query names, cache isolation and network quiescence are stronger than the chosen system-resolver mechanism. No unapproved attacker control of resolver configuration is assumed here, and every new outbound operation still checks its own grant. Name-to-address mapping was already delegated to the operator, so this does not defeat that authority model.

**Minimum limit text.**

> Tide keeps no DNS cache of its own. The system resolver may share caches across calls and Nets, substitute configured aliases, and follow DNS aliases. Cancellation abandons the caller's lookup result; it does not stop the resolver's remaining traffic. The grant authorizes resolution through this operator-controlled mechanism.

Replace the “always ... plus” sentence with that qualified account. The system behavior is documented by [hostname(7)](https://man7.org/linux/man-pages/man7/hostname.7.html), [nscd(8)](https://man7.org/linux/man-pages/man8/nscd.8.html), and [resolver(3)](https://man7.org/linux/man-pages/man3/resolver.3.html). This finding does not request a new resolver, cache partition, or cancellation mechanism.

### F4 — Preserve `NoKeyLog` explicitly; disabling rustls logging does not disable key logging

**Classification: limit to state (non-blocking; implementation guard).**

**Where.** `research/42-net-get-v0.md:493`, `:562`, `:700`–706 and the environment cases in `conformance/rules.md:100`. The record does not mention `SSLKEYLOGFILE`.

**Attack and concrete steps.** Set `SSLKEYLOGFILE` in the launch environment. If a stage-2 author follows a debugging example and installs `rustls::KeyLogFile`, constructing it opens that path and subsequent handshakes append TLS secrets, even with rustls's `logging` feature disabled. This would add a filesystem write and compromise traffic confidentiality outside the advertised Net grant. The pinned crate supports that route, but its default client builder uses `NoKeyLog`, so the environment alone does **not** trigger it in the proposed ordinary builder path.

**Evidence.** Read the published `rustls-0.23.46` package in `/home/lucca/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`: `src/client/builder.rs:172`–180 initializes resumption and `NoKeyLog`; `src/key_log_file.rs:24`–43 opens/appends/creates the file and `:89`–95 reads the variable. `ClientConfig::resumption` defaults to a cache, so the explicit no-resumption design must override it; that requirement already exists at research/42:426–430. These observations are source inspection, not runtime proof. Publisher source locations: [builder](https://docs.rs/crate/rustls/0.23.46/source/src/client/builder.rs), [key logger](https://docs.rs/crate/rustls/0.23.46/source/src/key_log_file.rs); the documentation fetches failed here, so the inspected cached package is the evidence.

**Minimum guard/limit text.**

> TLS uses `NoKeyLog` and disabled resumption; Tide never constructs `KeyLogFile`, and `SSLKEYLOGFILE` is ignored. Disabling the `logging` feature alone does not establish this.

Add that variable to stage 2's environment-invariance observation. This is not a finding that rustls automatically logs keys, nor a demand to change TLS dependencies.

## Attacks that held

“Held” means the written contract closes the route; future implementation remains unproved. Cases above are not silently counted as successes.

| Attempt | Why it held / evidence |
| --- | --- |
| Parse, format, compare, hash or decode a `Url` without Net; try `socket_addrs`, `Deref`, `AsRef` or an inner-URL conversion | The §2 newtype exposes only listed data operations; Decode goes through parse. No inner-value escape is specified. Published url 2.5.8 `src/lib.rs:1340` really does resolve in `socket_addrs`; Display/Debug at `:2795`/`:2810` only inspect stored data. |
| Reach a resolver/TLS/socket through a re-export or a trait | NET-HTTP-4 explicitly forbids the extra public surface. `walk.rs:340`–361 admits re-exports before policy and `crates.rs:222`–235 collects them, so the required guard must inspect associated methods/impls and conversions as well as module names. The current sealed address trait only returns data; it does not resolve. |
| Mint real Net through test/sim APIs or runtime entry | `Net::new` is crate-private; `__rt` is denied by policy (`policy.rs:94`). Sim roots create simulated authority; existing world checks reject real authority in simulation and foreign/outliving simulated handles. A test-root TLS constructor is explicitly crate-private. |
| Hex/octal/short/integer IPv4, case, one trailing dot, IPv6 compression, mapped IPv6 | One shared canonical key is required for entries, connect, get and future resolve. The same checked parsed host supplies the dial/TLS/Host identity; implementations must not dial the original unchecked spelling. |
| IPv6-compatible versus mapped IPv6, bracket tricks, zone identifiers, name/IP aliases | Only mapped IPv6 folds to IPv4; compatible IPv6 remains a distinct IPv6 address, a conservative non-match. Brackets must contain an IPv6 address, zones are refused, and a DNS name never authorizes its numeric address spelling. |
| Userinfo, backslashes, tabs/newlines, escaped Unicode/Punycode, CR/LF injection | Raw-host restrictions follow the parser's preprocessing/span; userinfo is refused at get's shape step. WHATWG path/query serialization prevents request splitting. No later reparsing is allowed. The host-span equivalence check is still stage-2 proof. |
| Wrong/default/zero ports, schemes, wildcard and subdomain expansion | Exact port keys; only entries require 1–65535. Explicit/default 443 coincide through `port_or_known_default`; a port-0 URL cannot match a valid restricted entry. Other schemes fail before network activity. No scheme restriction is promised by a host:port grant: raw TCP and HTTP on port 443 are intentionally available. |
| Chained only, empty/invalid/duplicate lists, intersect a name and its IP alias | Intersection of canonical key sets cannot add a key. Duplicates do not add reach; name/IP intersection stays empty. `no_bind` is monotone and cloning a narrower Net preserves its restrictions. Narrowing does not revoke an already issued wider clone or stream. |
| Redirects, informational responses, 101 upgrades, status/header injection, reuse under a narrower Net | No follow, upgrade, second request, persistent connection or resumption is permitted. Bodyless-status precedence and total interim-head bounds close those response routes. Peer content remains untrusted; framing must be proved in stage 2. |
| Rebinding, SNI/Host mismatch, allowed relay, arbitrary path/query/bytes and response payload | Contact before certificate validation, arbitrary resolved addresses, raw TCP, relay reach, data channels and returned payloads are explicitly admitted limits. None is represented as a fixed-IP or GET-only capability. |
| Environment proxies, system/private roots, credential files, certificate-driven extra fetches | The chosen sans-I/O TLS/head parsers and url data parser do not own a transport. Pinned webpki-roots 1.0.12 is compiled data. No proxy/native-root/credential integration or online AIA/OCSP fetching is selected. Key-log caveat is F4; system resolution is F3. TLS internally uses OS entropy and wall time, not application-supplied random bytes or a trust store chosen by the URL. |
| Sim peer redirects, false errors, fault schedules, altered replay manifest | Steps 1–4 and `only` precede any peer lookup. Replies do not confer a new Net or change destinations. Fault APIs remain planned. Replay rejects changed profile/peer identity under existing source/build/input identity; it does not turn a simulated reply into real I/O. TLS/wire fidelity limits are F2. |

Not checked: no runtime or implementation proof, dependency build, complete transitive-library audit, or Opus review. The report identifies one blocking documentation correction; the proposed HTTP authority mechanism otherwise held under this paper review.
DONE
