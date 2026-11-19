Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 50622b4 (design)

## Verdict

**Changes required.** The narrow API and authority policy are sound directions, but the result-list promise and one simulator-fidelity claim contradict the landed implementation. Fix B1 and B2 before approval. Confidence: high for the source-level counterexamples; implementation behavior remains unproved at this design-only stage. No override.

## Scope

Read-only supervisor review of fx-6d50 stage 1, branch `fx-6d50`, HEAD `50622b4dc77a57aae6eea9154cb5baa3455595dd`, base `28f7f6d`. SHA-256 of the exact output of `git diff 28f7f6d..HEAD`:

`ec4675e97acdcbb852820725e281bc545f2e45f7308954181211dbf4f073a9e8`

Reviewed the six-file diff and surrounding owning text, research/48 in full, research/42's host/resolver/error/authority/simulation contracts, relevant fx-ff13 supervisor, red-team and Opus findings/re-checks, and the landed `net.rs`, `net/{host,resolve,sim,http,real}.rs` paths and relevant tests. Checked replay-input recording, the pinned nightly's local Rust source, GOVERNANCE.md, repository instructions, the builder report, and the permitted read-only bead show/comments. Repeated the reported lookup-term search over the builder's advisory-source folder, with no matches; that corroborates the search result, not a fresh audit of all 87 advisories.

The consumer contract is one planned `Net::resolve(&self, host: &str, deadline: Instant) -> Result<Vec<IpAddr>, NetError>`: cancellation, shape and host authority precede lookup; success returns address data without peer contact; cancellation/deadline can abandon the shared resolver wait. Only this report was written. No build, runtime probe, implementation edit, commit or Beads write was performed.

## Blocking findings

### B1 — The exact identity with connect's address list is false

**Where:** `research/48-net-resolve-v0.md:88–92`, `:118–120`, `:285–287`; `design/01-design.md:520–524`; `design/03-authority-concurrency.md:261–264`; `design/reference-companion.md:765–768`.

**Evidence and concrete cases:**

- For `host = "::ffff:127.0.0.1"`, the proposed literal rule returns the key's canonical IPv4 address, `IpAddr::V4(127.0.0.1)`. Landed `connect((host, port))` first succeeds at `parse::<IpAddr>()` and constructs `Address::Socket` with the original IPv6 value (`crates/tide-std/src/net.rs:87–92`). Its authority key folds the address at `:99–101`, but `Address::resolve` returns the original socket unchanged at `:118`. The real backend selects an IPv6 socket from that value (`net/real.rs:23–34`); it does not canonicalize it. Connect's actual list therefore contains `IpAddr::V6(::ffff:127.0.0.1)`, not the proposed IPv4 result. The bracketed spelling takes the other parsing path and does fold. Equality of authority keys does not prove equality of returned `IpAddr` values.
- For a simulated peer `world.http("svc.test:80", ...)`, the proposal returns a synthetic address from `resolve("svc.test", deadline)`, while landed `connect(("svc.test", 80))` returns `InvalidInput` before producing any address list (`net.rs:119–122`). Research/48 §6 acknowledges this exception, but §2 and companion 21 still promise connect/get list identity without a backend qualification. 01 repeats the unconditional connect promise. 03 has a simulation cross-reference, which helps but does not repair the mapped-literal case.

**Why it blocks:** Exact `Vec<IpAddr>` identity cannot be implemented merely by reusing the existing resolver and literal-key rules. Stage 2 must otherwise change landed TCP behavior or break the admitted result contract. This is a contract mismatch, not an authority bypass.

**Minimum fix:** Keep the chosen canonical literal result and explicitly distinguish it from connect's preserved mapped-IPv6 tuple representation. Scope exact order/duplicate preservation to the real non-literal lookup result, with the port removed and the same resolver answer assumed. State the separate synthetic simulation contract alongside that promise in the API summaries, including companion 21 and 01. Adjust research/48's “by construction” and “literals only” assertions. Do not silently change landed TCP to make the sentence true. Keep NET-RESOLVE-3's comparison about non-literal lookup results, and name the unbracketed mapped literal as a distinguishing stage-2 case.

### B2 — An HTTP peer at the returned address cannot test TCP connect-by-address

**Where:** `research/48-net-resolve-v0.md:277–282`; related summary at `design/reference-companion.md:466–469`.

**Evidence and concrete case:** Research/48 says code that resolves and then connects by address is tested against peers registered at addresses. Register `world.http("svc.test:80", ...)`, obtain its proposed `198.18.0.1`, and also register `world.http("198.18.0.1:80", ...)`. Under an unattenuated simulated Net, `connect(("198.18.0.1", 80))` still returns `ConnectionRefused`: `net/sim.rs:195–198` requires exactly IPv4 loopback with a TCP listener. It never consults the HTTP peer table. Binding a TCP listener at the synthetic address is also unavailable (`net/sim.rs:143–146`). The extra peer can make an HTTP `get` by address work, but cannot make TCP connect work. The next research bullet correctly preserves loopback-only TCP, so the suggested fixture does not bridge this limit.

**Why it blocks:** This misstates the simulator's usable coverage for the per-address TCP policies offered as a reason to admit resolve. A stage-2 author or consumer cannot test that path using the documented remedy.

**Minimum fix:** Say registering HTTP peers at the returned addresses supports `get` by address only. State that simulated TCP cannot exercise domain-resolve-then-connect-by-address with these synthetic addresses; it remains loopback-only. Carry that distinction into the companion's no-aliasing summary. Preserve the chosen TCP scope rather than adding synthetic-address TCP routing.

## Non-blocking findings

1. **Qualify the simulation summaries for localhost.** `conformance/rules.md:105`, `design/03-authority-concurrency.md:2216–2220`, and `design/reference-companion.md:462–466` say a peer-named domain gets `198.18.0.n` and any other domain is `Dns`. `localhost` is a name key, but the mandatory earlier literal step returns `127.0.0.1` with or without a peer. The ordered contract already decides the outcome. Explicitly exclude `localhost` from these summaries and retain stage-2 cases with and without a peer. Also qualify “resolve is Ok exactly when get would not be Dns” at research/48:287 by successful cancellation/shape/authority/deadline checks: Denied and Timeout are not Dns either.

2. **The separate-switch rejection overstates what it would bound.** `research/48-net-resolve-v0.md:322–327` says a resolve switch “bounds nothing.” It would bound the newly exposed full address list without TCP contact, which §5 correctly identifies as new information; it would not bound DNS query names/forms already available through connect. Rejecting another attenuation remains reasonable. Narrow the rationale to query traffic/reach; no additional switch is requested.

## Reasoning

- **Admission and need:** The note honestly says the malware screen does not need bare resolution and presents general v0 expressiveness as the benefit. A host string, explicit deadline and existing NetError over the existing pool are a small std-only surface. No syntax, authority kind, dependency, resolver options, record types or cache is proposed. Deferral would also be sound and smaller; the note acknowledges it and names the address-as-data capability gap as its reason to proceed. B2 corrects the claimed simulation support for that use.
- **Order and errors:** Cancellation → shape → only → deadline → literal/localhost → lookup follows get's existing checks (`net.rs:249–258`, `net/http.rs:592–603`) and connect's cancellation → shape → only → resolution path (`net.rs:211–217`), with an intentionally new operation deadline. The resolver checks cancellation before expiry while acquiring capacity and awaiting the result. Mapping cancellation to Cancelled, TimedOut to Timeout, and other lookup failures/empty answers to Dns follows get. Companion 21 and `NetError::is_transient` agree that Dns is not transient. Shape failures are Io(InvalidInput); connection/status variants are unnecessary.
- **Port evidence:** Pinned `nightly-2026-08-09` rust-src, `library/std/src/net/socket_addr.rs:200–212`, sends non-literal tuple hosts to `lookup_host(host, port)`. `sys/net/connection/socket/mod.rs:349–357` calls `getaddrinfo` with a null service and SOCK_STREAM; `:328–331` installs the port afterwards. Port zero does not change the system lookup request on the supported platform. This supports the no-port API and non-literal reuse, not B1's literal identity. Separate calls can observe changed resolver answers.
- **Authority:** No new unlisted lookup route is specified. A retained only entry supplies an existing connect port that can already trigger the same canonical-host lookup. Matching retained hosts preserves chained only semantics. Name grants do not grant address spellings; address grants do not grant names. only([]) denies resolve even for literals. no_bind is unrelated to resolve and retains early bind refusal. ASCII/case/trailing-dot/numeric-IP/mapped-key/percent/Punycode rules remain owned by NET-HOST. localhost retains a name key despite its literal result. The complete list, including unreachable addresses, is newly learned information and is declared. Resolver configuration, aliases/search/cache, contention and traffic after cancellation remain stated limits. Non-binding/rebinding warnings are explicit in research/48 §3, 03's API note and companion 21; the core reference points to companion 21.
- **Simulation and replay:** Given the same ordered setup, distinct-name assignment is deterministic, collision-free within the stated range and independent of random draws. `World::http` records canonical keys (`sim.rs:169–171`); `Journal::input` appends ordered inputs (`sim/manifest.rs:174–179`), and the manifest already accepts http inputs. The map can be reconstructed from existing input/build identity without a new independent identity. One trace event, no time/draw and an overflow panic are concrete planned requirements, not passing runtime results. The fixture universe, absent resolver latency, no aliasing and loopback-only TCP are acceptable when described accurately; B1/B2 correct the remaining overclaims.
- **Consistency, labels and stage 2:** 01 records the decision; 03 owns the detailed policy and the reference points to companion 21. New behavior and conformance rows are labelled planned. No implementation is claimed shipped. NET-RESOLVE-1's shared literal/refusal suite, -2's draws/replay observation and -3's real resolver-call/held-lookup checks are concrete. The `.invalid` test needs a recorded resolver environment or controlled answer: operator NSS/hosts configuration can override the DNS expectation, consistent with the existing residual limit. Separate changing lookups are not a stable answer-order binding.
- **Budget and candidate checks:** `scripts/count-tokens.sh design/reference.md` returned **24,978**, exit 0: 22 tokens below 25,000. `git diff --check 28f7f6d..HEAD` passed. HEAD and tracked status remained unchanged during review.

Not checked: no builds, runtime/replay execution, external web verification, new red-team pass or subsequent Opus review. Resolve's implementation remains unproved; this verdict requires design-document corrections only.
DONE
