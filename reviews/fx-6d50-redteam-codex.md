Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Red team, candidate 25954b7 (design)

## Verdict

**Sound with stated limits.** No blocking authority hole or misleading authority claim found in what `resolve` adds. Two non-blocking simulator limits below deserve explicit sentences; neither requires changing the chosen authority policy. Confidence: high for the paper authority argument and inspected parser/resolver paths; stage-2 enforcement and replay behavior remain unproved.

Candidate HEAD: `25954b7027829adb8e480d46e96571cdb1225596`, branch `fx-6d50`, base main `28f7f6d`. SHA-256 of the exact output of `git diff 28f7f6d..HEAD`:

`516c146091e2b855fbc1b365a4f5a84acd39bc7ccf56d1f3a2e5693240364b67`

Read AGENTS.md, GOVERNANCE.md, `br --no-db show fx-6d50`, `br --no-db comments fx-6d50`, the complete six-file diff, research/48, the relevant research/42 host/authority/resolver/simulation/trust contracts, and the specified design surfaces. Inspected landed `net.rs`, `net/{host,resolve,sim,real}.rs`, relevant HTTP/check cases, setup input recording and manifest storage, and cached pinned Rust/url/idna sources. Read the two stage-1 Codex reports and the Net::get red-team report/re-check to avoid repeating their findings. This review attacks resolve's additional information, port-free authorization, lookup input, and simulation map; it does not reopen the previous passes.

Only this report was written. No build, test, Tide execution, resolver probe, third-party interaction, commit or `.beads/` write occurred. HEAD, diff identity and initially clean tracked status were checked again before writing.

## Authority argument

The consumer is a Tide function holding a delegated `Net`. Its trigger is `resolve(host, deadline)`; the result is address data, not a connection or a new grant. The decisive boundary is shape and authority before lookup, including for literals. Its important failures are `Denied`, `Io(InvalidInput)`, `Cancelled`, `Timeout` and `Dns` in research/48:118–146's specified order.

For a non-literal host key `h`, authorization requires a **retained** entry `(h, p)` for some valid port `p` (research/48:120–121,164–186). Landed `only` already intersects complete canonical `(host, port)` keys (`crates/tide-std/src/net.rs:167–176`). `connect((h, p))` therefore supplies an existing authorized path to the same lookup: `Address::resolve` passes the canonical name to `resolve::lookup` (`net.rs:119–122`), and that helper passes it to Rust's tuple `ToSocketAddrs` (`net/resolve.rs:69–74`). In the pinned nightly, `library/std/src/sys/net/connection/socket/mod.rs:349–357` zeroes the hints, sets only `SOCK_STREAM`, and calls `getaddrinfo` with a null service; its iterator attaches the port afterwards at :328–331. Resolve's proposed port zero consequently changes neither query name nor lookup hints on this supported platform. No new `AI_*` option, resolver, record type or name expansion is specified.

This proves the intended **query authority**, not equal answers from separate calls, DNS isolation, or equal address representations for literals. The candidate now distinguishes all three. The new information grant is explicit: every address, order and duplicates, even when no service accepts TCP (`research/48:204–211`; `design/03-authority-concurrency.md:278–281`). Learned addresses remain data; they cannot make a name-only Net authorize an address spelling.

## Findings

### R1 — The synthetic address reveals the number of earlier ungranted peer names

**Classification: limit to state (non-blocking).**

**Where:** `research/48-net-resolve-v0.md:204–211,247–249,279–287`; `design/reference-companion.md:460–471`; `design/03-authority-concurrency.md:2222–2229`.

**Attack and concrete steps (paper trace):**

1. In setup A, register `visible.test:443` as the first HTTP peer name. In setup B, register `hidden.test:80` first, then `visible.test:443`.
2. Give the reviewed component only `sim.net.only(["visible.test:443"]).no_bind()` in both worlds. Give it no setup or trace inspector and no grant for `hidden.test`.
3. It calls `resolve("visible.test", deadline)`. The designed answers are respectively `198.18.0.1` and `198.18.0.2`.
4. With additional preceding distinct names, the component learns their count from the returned ordinal. It can report that count through its ordinary return value; no network connection is needed.

**Impact:** The chosen map is deterministic for identical ordered setup, but it is not independent of unrelated setup. A granted-name address now carries an exact aggregate fact about earlier ungranted peer names. Before resolve, an otherwise identical HTTP peer response need not expose that ordinal at all. This is confined to simulated state, does not reveal those names' spellings, and does not grant reach to them. Real resolver answers can also reflect wider configuration; Net is not an information-noninterference boundary. The map itself is already stated, so this is a consequence to disclose, not a reason to block admission or replace it with hashing.

**Minimum limit text** beside the simulator map:

> Synthetic addresses depend on the complete ordered peer setup. Resolving a granted name reveals its ordinal, including the count of earlier distinct peer names outside that Net's grants. Reordering unrelated setup can change its address; identical recorded setup replays identically. The map provides no isolation of setup information between Nets.

An implementation witness is the two setups above, with duplicate canonical spellings on different ports not consuming a second name ordinal. This supplements, rather than contradicts, NET-RESOLVE-2's determinism requirement.

### R2 — Synthetic success is not evidence that a host fits DNS length limits

**Classification: limit to state (non-blocking).**

**Where:** `research/48-net-resolve-v0.md:78–83,119–130,270–276`; `design/reference-companion.md:460–466,776–781`; `crates/tide-std/src/net/host.rs:65–81,97–99`; `crates/tide-std/src/net/sim.rs:532–543`.

**Attack and concrete steps (paper trace):**

1. Let `h` be 64 ASCII `a` characters followed by `.test`. Alternatively use individually short labels whose combined name exceeds DNS's total length bound.
2. Register `world.http(h + ":443", ...)`, delegate `only([h + ":443"]).no_bind()`, and call `resolve(h, deadline)`.
3. NET-HOST's restrictions permit the spelling and the URL host parser accepts it as a domain. The pinned `url-2.5.8/src/host.rs:109` calls IDNA with `AsciiDenyList::URL`; `idna-1.1.0/src/lib.rs:132–140` expressly selects `DnsLength::Ignore`. Its optional DNS-length verifier separately rejects labels over 63 bytes and a dotless-final representation over 253 bytes (`src/uts46.rs:463–481`). Tide adds neither check. The peer can therefore be registered, and the proposed simulator returns its synthetic address.
4. With an ordinary real DNS-backed resolver and no local override for `h`, no valid DNS question can represent that overlong label. The real operation cannot get the simulator's successful DNS answer; resolver failure becomes `Dns`. This is conditional on resolver configuration, not a claim that every possible NSS/local naming provider must reject the name.

**Impact:** The simulator can accept a registered spelling whose DNS-backed real lookup fails. This does not query an ungranted name: the same spelling must have passed the same grant check, and existing connect/get already delegate it to the operator's resolver. It does mean that resolve's new successful address result is not a DNS-validity check. Shape rejection (`Io(InvalidInput)`) must not be inferred merely from DNS length. The shared-suite acceptance covers its named cases; it should not be read as universal real/sim DNS fidelity.

**Minimum limit text** beside the simulator rule or its reference pointer:

> NET-HOST validates host keys, not DNS label or total-name lengths. A registered simulated name can resolve even when a DNS-backed real resolver rejects its length. Such real resolver failures are `Dns`; simulated success proves neither DNS encodability nor real resolver acceptance.

No new length restriction or backend change is needed. If stage 2 records a witness, use one overlong-label registration and a controlled failing real resolver answer; do not send a live query to establish this limit.

## Attacks that held

“Held” means the written design and inspected reusable paths close the attack, not that the unbuilt method passed execution.

| Attempt | Why it held / boundary |
| --- | --- |
| Resolve an unlisted name, including under `only([])` and `no_bind()` | Shape and retained-host authority precede literal handling, pool acquisition and lookup (research/48:118–134). Empty lists deny even IP literals. The inherited bind exception is already explicit and is not a new resolve route. |
| Turn a name grant into its answer's address grant, or use an address grant to query names pointing there | Names and IPs are distinct `Host` variants (`host.rs:17–20`); no post-answer mutation is specified. `localhost` retains a name key despite returning IPv4. |
| Gain arbitrary-port connections because resolve ignores ports | Any **surviving** host entry permits address information only. Later connect/get still check the complete `(host, port)` key. Resolving under an entry for port 1 does not authorize connecting on 443. |
| Widen by chaining `only(["svc.test:443"]).only(["svc.test:80"])` | The retained full-key intersection is empty (`net.rs:169–173`), so resolve must be `Denied`. Projecting each list to hosts before intersecting would be wrong; the contract says to inspect entries of the resulting Net. Invalid/zero-port entries and duplicates add no key. |
| Hex/octal/short/integer IPv4 (`0x7f.1`, `0177.0.0.1`, `127.1`, `2130706433`), trailing-dot numeric aliases, overflowing/malformed numeric forms | URL host parsing produces a numeric key or refuses; successful numeric keys take the no-lookup literal path. No original numeric spelling is handed to `getaddrinfo` as a name (`host.rs:53–81`; cached url host.rs:109–119). |
| Compressed, bracketed and unbracketed IPv6; mapped and compatible IPv6; zone IDs and bracket/port tricks | Tuple-style parsing plus `host::ip` canonicalizes mapped keys to IPv4. Compatible IPv6 stays a distinct IPv6 key; no mapped/compatible equivalence is granted. Brackets must contain only an IPv6 address; `%` zones fail. Unbracketed mapped literal result versus TCP representation is already expressly distinguished. |
| Case, `LOCALHOST.`, one/two trailing dots, empty labels, wildcard/subdomain expansion | Canonical lowercased key and one-dot removal are shared. `localhost` bypasses the resolver only after authority checks. Empty labels, a second trailing dot and `*` fail; equality provides no subdomain matching. |
| NUL truncation, ASCII controls/DEL, spaces, Unicode, percent escapes and hidden Punycode before `getaddrinfo` | Raw ASCII/percent/Punycode restrictions run first; the special host parser uses IDNA's URL deny list, which excludes all bytes through space and DEL (`idna src/uts46.rs:279–280,344–351`). Thus C-string truncation/control removal cannot separate checked and looked-up strings. Existing NUL tests are evidence of intended coverage, not a test rerun. |
| Exploit `ndots`, suffix search, aliases or stripping an absolute trailing dot to query different wire names | The key's name is the resolver input, expressly including `service.` becoming `service` and search expansion (research/48:138–142). Operator aliases, CNAMEs and search behavior are an authorized residual limit, already stated at :228–234 and research/42:454–465. Exact checked-input identity does not promise exact wire-question identity. |
| Learn unreachable/private/all-family addresses without contacting a listening service | This is the admitted information addition, expressly including addresses that refuse TCP and configuration effects (research/48:204–211). It is not confined to `connect()?.peer_addr()` and creates no new connection grant. Re-resolving later is explicitly not address pinning. |
| Cancel/expire, then make DNS continue; fill the global pool or infer another Net's activity from contention/cache timing | These routes exist, but the candidate explicitly admits continuing abandoned traffic, retained slots, shared caches and cross-Net timing/contention (research/48:228–241). `resolve` makes the channel cheaper (:199–203); eight concurrent helpers are not a rate limit or guaranteed resolver termination. No silence or fairness guarantee is claimed. |
| Duplicate peer spellings, different ports, reordered setup or seed changes alter replay unexpectedly | Keys are canonical before registration; identical host/port duplicates panic, different ports on one canonical name share the planned address. The map is explicitly setup-order-dependent and seed-independent. Ordered inputs are appended, not sorted (`sim.rs:169–171`; `sim/manifest.rs:174–179`); identical setup and std identity suffice. R1 covers the ordinal disclosure, not a replay break. |
| Use a synthetic address as an alias to reach a name-only peer or exercise real TCP | Address HTTP registration is separate and requires an address grant; otherwise `Denied` or `Refused` as documented. Synthetic TCP routing is expressly absent. No synthetic value mints real authority or a resolver handle. |
| Leak real DNS answers through a replay manifest, or treat trace export as confidential | Only simulated resolve records the planned trace, whose successful addresses are synthetic or literal. It adds no real-DNS recording input; ordered peer identities were already manifest inputs. Checked host keys, results and denied-call metadata are intentionally readable trace data; the host can itself contain caller-supplied information. Replay/log exports are not a confidentiality boundary. |
| Misread the core row or a `net` trust line as GET-only, fixed-IP, no-DNS, or no-bind authority | Reference :338 points to companion 21; that section teaches the full list and any-port host authorization, name/address separation and rebinding. 03:1376–1379 extends trust-line meaning to lookup hosts and retains the bind exception. Existing broader Net/relay/data-channel limits remain present. No separate DNS authority or narrower promise is implied. |

Not checked: implementation, runtime timing, resolver traffic, actual replay, or a new dependency/transitive-library audit. The two findings are simulator disclosure/fidelity limits; no production reach bypass was established.
DONE
