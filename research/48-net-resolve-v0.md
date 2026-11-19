# Net::resolve in v0: a name lookup under NET-HOST (fx-6d50)

Status: stage 1, the design admission (docs only). Stage 2 implements it
after this note's review rounds and a separate red-team pass. This note
fixes the contract of `Net::resolve`, what `Net::only` lets it look up,
its simulated backend and its trust-diff reading, and gives the soundness
verdict that owner decision A1 asks for. It reuses research/42's NET-HOST
(§4, the host key), NET-RESOLVE (§7, the one private resolver pool,
`crates/tide-std/src/net/resolve.rs`), its authority limits (§6) and its
simulated peers (§9). Section 2 scopes research/42's list-identity
rule. UDP (`bind_udp`, `UdpSocket`) stays (planned) outside the alpha
(relayed owner decision, 2026-11-19, on fx-6d50). Labels follow
research/42: **[repo evidence]**, **[source]**, **[inference]**.

## 1. Need

**[repo evidence]** Owner decision A1, 2026-11-18, quoted on fx-6d50:
"Bring Proc::command into the alpha, plus a network capability if the
design is sound." The relay names fx-ff13 (`Net::get`, landed) and fx-6d50
(resolution) as the beads that carry it, and makes admission plus a red
team the condition. research/42 §7 left fx-6d50 four constraints: the same
key, the same mechanism, the same simulated names, and its error type.

**[repo evidence]** What v0 has today. `resolve(host)` is taught as
planned in reference §6.1 and §6.7, in 03 §1.1's `Net` row, and in 01's
Network decision. Tide v0 has no way to learn a name's addresses as data:
`tide_std::net::ToSocketAddrs` is sealed and has no method (`net.rs`,
`mod address`), Rust's ambient lookups are not reachable from Tide code,
and NET-HTTP-4's guard (`std_net_public_items_are_fixed`) forbids a
resolver item in `tide_std::net`. The nearest substitute,
`net.connect((host, port))?.peer_addr()`, yields one address, only after a
TCP connection to it, and only on a port that accepts.

**[repo evidence]** What fx-zu4f.8's 87 advisories need. The quoted
sources fx-zu4f.5 gathered (the four approved archive copies and the
public write-ups in `~/.cache/eon-agents/tide-lead/fx-zu4f.5-work/sources`)
contain no bare name lookup: no `to_socket_addrs`, `lookup_host`,
`getaddrinfo`, `gethostbyname`, DNS crate or `nslookup` (`rg -i` over the
folder, 2026-11-18). The malicious effects are HTTP(S) downloads and
uploads, process launches and file reads. The one network call outside
HTTP is a `UdpSocket::bind("0.0.0.0:0")` plus `connect("8.8.8.8:80")`
trick that learns the local interface address (the finch write-up), which
needs UDP, not a lookup. **So no port in that screen needs `resolve`**: its
counts are the same with or without it. This is stated plainly because A1
named malware ports as the motive.

**[inference]** What honest code needs it for. Code whose result is the
address itself, or that must treat a name's addresses one by one:

1. reporting addresses (a `getent hosts`-style tool, a health check or
   inventory that prints what a service name points at, a config
   validator that checks each configured name resolves);
2. per-address policy over all of a name's addresses: probing every
   replica behind a name, racing IPv4 against IPv6, pinning one address
   for several connections, checking an address before connecting by it.
   These then connect *by address*, so they need address grants or an
   unattenuated `Net` (§4: a name grant does not grant its addresses);
3. later, UDP: `send_to` takes a `SocketAddr`. Not in the alpha.

`get` and `connect` cover the common case, reaching a named service, and
stay the shape to use for it. `resolve` adds exactly one thing neither
gives: the address list as data, without contacting the peer.

## 2. The surface (smallest)

**[inference]**

```rust
impl Net {
    pub fn resolve(&self, host: &str, deadline: Instant) -> Result<Vec<IpAddr>, NetError>;
}
```

- **`host: &str`, no port.** A lookup maps a name to addresses; the port
  plays no part in it (Rust's `lookup_host` calls `getaddrinfo(host, NULL,
  {ai_socktype = SOCK_STREAM})` and attaches the port afterwards:
  `library/std/src/sys/net/connection/socket/mod.rs:349–356` in the pinned
  nightly's `rust-src`). `host` is keyed exactly as `connect((host, port))`
  keys its host (`net.rs`, `impl Sealed for (&str, u16)`): an IP address
  as written, unbracketed or bracketed for IPv6, else NET-HOST's host
  rule. So `"example.com:443"`, a URL, `""`, `"a..b"`, a `%`, an `xn--`
  label, a `*` and non-ASCII are `Io(InvalidInput)`. Not `&Url`: a host is
  not a URL, and an entry's host is what `only` grants.
- **`deadline: Instant`**, as `get` takes it (reference §7.2's operation
  deadlines): a lookup can wait for seconds on resolver retries, and the
  caller needs `Timeout` distinct from scope cancellation. It is an
  `Instant` from a `Clock` of the same world, as for `get`.
- **`Vec<IpAddr>`**, never empty on `Ok`. Real non-literal lookups
  preserve the resolver's answer in order, with duplicates. Given the
  same resolver answer, this is the list `connect`'s lookup uses with
  ports removed. IPv4-mapped IPv6 entries in that answer keep their
  representation. An IP literal returns its canonical address, and
  `localhost` returns `127.0.0.1`. An unbracketed `::ffff:a.b.c.d`
  literal returns IPv4. TCP `connect((host, port))` preserves the IPv6
  address for that spelling. In simulation, other names get one
  deterministic synthetic address if a `world.http` peer names them, or
  `Dns` otherwise (§6). `IpAddr` is `Pure`.
- **`NetError`**, not `io::Result`: `resolve` shares `get`'s deadline
  and its `Dns`/`Denied` vocabulary (research/42 §7 item 4 left the choice
  here). `connect` keeps `io::Result`.

This scopes research/42 §7 item 2's list identity to real non-literal
lookups given the same resolver answer, with literals returning the
canonical key address.

Not provided: reverse lookups (`getnameinfo`, PTR), other record types
(SRV, TXT, MX), resolver options or servers, ports or services, a cache,
an async or streaming form. Each is a separate authority question (a
reverse lookup queries a name the code did not hold; TXT is a data
channel from the name's operator) with no current caller.

What it adds. To the language: nothing. To std: one method over the
existing private resolver, one simulated rule (§6) and one trace event.
To the checker: one line in NET-HTTP-4's fixed item list (`pub fn
resolve`). No new authority kind, attenuation, type or dependency.

## 3. What one `resolve` does

**[inference]** NET-RESOLVE-CALL. Both backends follow this order; each
failing step returns at once and no later step runs:

1. **Cancellation.** A checkpoint: a cancelled scope returns `Cancelled`.
2. **Shape.** `host` keys as in §2, else `Io(InvalidInput)`.
3. **Authority.** Under `only`, some entry's key host equals `host`'s key
   host, on any port; otherwise `Denied`. No lookup has happened (§4).
4. **Deadline.** If `deadline` has passed, `Timeout`.
5. **Literals.** An IP host, or `localhost`, returns its key's address
   (`localhost` is `127.0.0.1`, an IPv4-mapped literal its IPv4 address)
   with no lookup (`Host::literal`). For an unbracketed mapped literal,
   TCP `connect((host, port))` retains the tuple's IPv6 address for dialing
   (`net.rs`, `Address::Socket`).
6. **Lookup.** Simulated lookups follow §6. Real lookups use NET-RESOLVE
   with `deadline`: `Ok` with the resolver's answer in order and with
   duplicates (§2); `Dns` when the resolver fails or returns no address;
   `Timeout` when the deadline passes while waiting for a pool slot or
   the answer; `Cancelled` on scope cancellation, which wins at the
   same wait (level-triggered). A cancelled or timed-out call
   abandons the lookup (research/42 §7): its thread finishes on its own and
   its result is dropped. A scope deadline cancels the scope, so it gives
   `Cancelled`, as for `get`.

The key's host is what is looked up (research/42 §4, *One parse*): a
trailing dot is dropped first, so `"service."` is looked up as `"service"`
and the resolver's search domains may apply. That is NET-HOST's existing
behavior for `connect` and `get`, under §5's resolver-configuration limit;
`resolve` keeps the same lookup input for real non-literal names.

**NetError classification** (companion 21, unchanged): `Cancelled`,
`Denied`, `Timeout`, `Dns` (every other lookup failure, the helper
thread's spawn error included, as `get` maps them) and `Io(InvalidInput)`;
never `Refused`, `Reset` or `Status`. `is_transient()` is false for `Dns`, as today: a transient
resolver failure (`EAI_AGAIN`) and a missing name both map to `Dns`,
because the system resolver does not distinguish them reliably and
`get` already maps every lookup failure that way (`http.rs`, lookup
`map_err`). A caller that retries `Dns` does so knowingly.

**Not a binding of later calls.** `resolve`'s answer does not fix what a
later `connect(name)` or `get` dials: each looks the name up again, and
the name's DNS may change in between (rebinding). A check of the
addresses protects only connections made *to those addresses*. Stated
where `resolve` is taught.

## 4. What `only` permits

**[inference]** NET-HOST governs `resolve` as an outbound peer operation,
before any lookup:

| Grant | `resolve(h)` |
| --- | --- |
| unattenuated `root.net` | every host |
| `only(["example.com:443"])` | `"example.com"`, `"EXAMPLE.com."` (one key); not `"www.example.com"`, not example.com's addresses |
| a port-scoped entry, any port | its host: the port does not narrow a lookup, which has none |
| `only(["93.184.215.14:443"])` | `"93.184.215.14"`, its other IPv4 spellings and `"[::ffff:93.184.215.14]"`, returned without lookup; never a name that resolves there |
| `only(["localhost:8080"])` | `"localhost"`, as `[127.0.0.1]` with no lookup; not `"127.0.0.1"` |
| `only([])` | nothing, literals included |

A name grant permits resolving the name; an address grant permits
"resolving" the address, which echoes it; neither grants the other, as
for `connect`. There are no wildcards and no subdomain matching (NET-HOST
entries refuse `*`). `no_bind()` does not affect `resolve`: `resolve` is
governed by `only`, as `connect` is, and `no_bind` governs `bind`. So
`only([..]).no_bind()` resolves exactly the listed hosts, while a
binding-capable `only` Net can still make arbitrary bind-name lookups
through `bind` (research/42 §4's stated exception, unchanged and not
widened by `resolve`).

**Why any port.** research/42 §7 item 1 fixed this, and the reasoning
holds on inspection: the holder of `only(["example.com:443"])` can call
`connect("example.com:443")`, which sends the same lookup through the same
mechanism (§5), so requiring a port on `resolve` would bound nothing.

## 5. Authority, pre-answering the red team

**[inference]** Resolution is an observation and exfiltration channel: the
query name reaches the operator's resolver and, on a cache miss, the
name's authoritative servers. What a `resolve` grant lets a program send
and learn, and how it compares with `connect` and `get` on the same `Net`:

- **Sent.** Only lookups of hosts `only` grants, each the key's host. The
  holder of the same `Net` sends the identical lookup through `connect`
  to any granted entry for that host: the same NET-RESOLVE function, the
  same `getaddrinfo(host, NULL, SOCK_STREAM)` call (§2), the same pool.
  `resolve` adds no query name and no query form. The number and timing
  of lookups are a channel to the resolver and the name's DNS operator, as
  each `connect` and `get` already is (research/42 §6, *Observers*);
  `resolve` is cheaper (no TCP), so it can make more of them per second,
  bounded by the 8-slot pool and nothing else. Stated limit.
- **Learned.** The resolver's full answer for a granted host: every
  address, in order, including those that would refuse a connection, and
  the effect of the operator's configuration (`/etc/hosts`, search
  domains, aliases) on that host. `connect` reveals one address (its
  `peer_addr`), after contact; `get` reveals none. This is the one thing
  `resolve` adds, by design (§1). It is information about granted names
  only, and it widens no reach: an address learned from a name grant is
  still `Denied` to `connect` and `get` unless an entry grants it.
- **Not sent to the peer.** `resolve` makes no TCP or TLS contact, so the
  allowed host's server sees nothing; only the DNS path does.
- **Name spellings, IDNA, IP literals.** By NET-HOST's host rule, before
  any lookup: ASCII only, no `%`, no `xn--` label, no empty label, so no
  internationalized or percent-encoded name reaches IDNA or the resolver,
  and a reviewer reading an `only` entry never meets a homograph. Every
  IPv4 spelling (`0x7f.1`, `2130706433`) keys as its address and is
  returned without lookup; no numeric-looking name reaches `getaddrinfo`,
  whose `inet_aton` parsing would otherwise accept forms NET-HOST keys
  differently (the URL host parser maps every host that ends in a number
  to an address or refuses it).
- **Empty and trailing-dot names.** `""`, `"."` and `"a..b"` are
  `Io(InvalidInput)`; one trailing dot is dropped (§3), so `"example.com."`
  and `"example.com"` are one key and one lookup.
- **Wildcard-free matching.** Key equality only; `"*.example.com:443"`
  grants nothing; `"a.example.com"` is not `"example.com"`.
- **Resolver configuration and caches.** research/42 §6's residual limit
  applies unchanged: the system resolver may share caches across calls
  and `Net`s, substitute configured aliases (`HOSTALIASES`), follow CNAMEs
  and apply search domains; the grant authorizes resolution through this
  operator-controlled mechanism. Tide keeps no cache of its own. A cache
  shared with other processes is a timing side channel (a fast answer says
  someone looked the name up recently); `connect` exposes it too.
- **Timing.** Lookup latency is observable, as for every network
  operation. Two differently attenuated `Net`s share the pool: a narrow
  one can delay a wide one's lookups by filling it, as `connect` can
  today. Stated limit, unchanged.
- **Cancellation.** It abandons the result, not the resolver's remaining
  traffic (research/42 §6); an abandoned lookup holds its pool slot until
  the system resolver returns.
- **Unattenuated `Net`.** `root.net` resolves any name, so data encoded
  in a name (`<data>.collector.example`) reaches that name's DNS operator.
  `root.net` already has this through `connect`, `get` and `bind`; a
  reviewer reads `root.net` as every host (research/42 §10).

**Does `resolve` widen anything beyond `get` and `connect` on the same
`Net`?** Reach: no. Query names and forms: no. Information: yes, the full
address list of each granted host, without contacting it. Nothing else.

**[source]** Precedent. WASI's `wasi:sockets/ip-name-lookup` has the same
shape: `resolve-addresses(network, name)` returns IP addresses only, no
port, and "if the input is an IP address string, the address is parsed
and returned as-is without making any external requests"
([ip-name-lookup.wit](https://searchfox.org/firefox-main/source/third_party/rust/wasi/wit/deps/sockets/ip-name-lookup.wit)).
Wasmtime gates it with a separate `allow_ip_name_lookup` switch, off by
default while TCP and UDP are on
([wasmtime-wasi sockets](https://docs.rs/crate/wasmtime-wasi/36.0.7/source/src/sockets/mod.rs),
[ip_name_lookup.rs](https://docs.wasmtime.dev/api/src/wasmtime_wasi/ip_name_lookup.rs.html)),
because its sockets take addresses, so lookup is the only path by which a
component queries a name. Deno's `Deno.resolveDns` requires `--allow-net`
([docs](https://docs.deno.com/api/deno/~/Deno.resolveDns)); how it matches
the queried name against a host list was not verified here and is not
relied on.

## 6. Simulated backend

**[inference]** Under `Root::sim`:

- **Steps 1–5** run as on the real backend.
- **Names.** After the cancellation, shape, authority and deadline checks,
  `localhost` returns `127.0.0.1` with or without a peer. For other
  domain names, a `sim.world.http` peer on any port makes the name resolve
  to one synthetic address. Without a peer the result is `Dns`.
  Simulated `get` applies the same name-existence rule after its own
  cancellation, shape, authority and deadline checks (research/42 §9).
  NET-HOST leaves DNS label and total-name lengths to the resolver.
  Simulated `get` and `resolve` success for a name proves neither DNS
  encodability nor real resolver acceptance. Real resolver failures are
  `Dns`.
  When `sim.node` is built, a node's name resolves to the node's address,
  which that bead assigns.
- **Addresses.** Ordinals count distinct host keys of domain peers
  after NET-HOST canonicalization, starting at 1 in setup order.
  `localhost` and IP-address peers take no ordinal. The n-th counted
  host gets `198.18.0.0 + n`: `198.18.0.1`,
  `198.18.0.2`, and so on through `198.19.255.254`, a range IANA reserves
  for benchmarking and marks not globally reachable
  ([RFC 6890 §2.2.2](https://www.rfc-editor.org/rfc/rfc6890#section-2.2.2),
  from [RFC 2544](https://www.rfc-editor.org/rfc/rfc2544)). Two peers on
  one host key share its address. Resolving a granted name reveals its
  ordinal, and so the number of earlier distinct peer names, including
  names outside that Net's grants (`localhost` and IP peers excepted).
  Changing the setup order can change the address. An address, once
  assigned, never changes during a run. Registering more distinct names
  than the range holds panics at setup.
- **No aliasing.** To test `get` by synthetic address, register an HTTP
  peer at that address. A request by address without that registration
  gets `Refused` after one round trip. A name grant alone gives `Denied`
  by NET-HOST, even with an HTTP peer at that address.
- **TCP is loopback-only.** Simulated `connect` and `bind` refuse names
  other than `localhost` with `InvalidInput` (03 §9.3). HTTP peer
  registrations support `get` by address. Simulated TCP cannot test
  resolving a domain and then connecting to its synthetic address.
- **Time and draws.** None. A simulated lookup takes no virtual time and
  no draw, in every outcome, as `get`'s `Dns` and literal steps do; the
  simulator has no resolver latency model. A run that never calls
  `resolve` makes exactly today's draws, and one that calls it makes the
  same draws as one that does not. Planned faults (`sim.net.drop_all()`,
  `net_drop`) apply to `resolve` as to `get` when they are built.
- **Trace.** Each completed or pre-cancelled call records one
  `net.resolve` event with task, span, the checked host key (or `-`) and
  the result (the addresses, or the variant). The host is not payload: a
  `get` event already records its key.
- **Replay identity.** No new input or profile identity: the
  name-to-address map is a function of the `world.http` registrations,
  which the manifest already records as inputs in setup order (`sim.rs`,
  `replay_input("http", ..)`), and of std, whose content hash the
  manifest records. A seed that calls `resolve` replays byte for byte.

## 7. Trust diff

**[inference]** No new authority kind: `resolve` is a `Net` method, so an
item that reaches `Net` declares `net`, as before. A `net` line's `only`
list (03 §4.3) now also names the hosts `resolve` may look up: the
entries' hosts, any port. No `net.dns` sub-kind: on an attenuated `Net`
the names `resolve` can look up are the names `connect` already looks up.
A separate kind would distinguish access to the full address list
without narrowing the query names or forms.

## 8. Alternatives rejected

- **No `resolve`; `connect` and `get` resolve internally** (the status
  quo). Sound and smaller, and enough for fx-zu4f.8. Rejected because v0
  then cannot express any honest code whose output or policy is the
  address list (§1); the only substitute, `connect` plus `peer_addr`,
  makes contact and yields one address. The cost of closing the gap is
  one method over machinery that already exists (§10).
- **A separate switch** (Wasmtime's `allow_ip_name_lookup`, a
  `no_resolve()` attenuation or a `Dns` capability). A switch would bound
  access to the full address list without contacting the peer. On an
  attenuated `Net`, `connect` could still send the same queries. On
  `root.net`, `connect`, `get` and `bind` could still resolve any name.
  Those existing lookup paths leave too little benefit for a separate
  attenuation. Wasmtime needs a switch because its sockets take only
  addresses.
- **`resolve` returning `SocketAddr`s, taking a port or an `only`-style
  `"host:port"`**: the port plays no part in a lookup (§2), and a port
  argument suggests a scoping it does not have.
- **Requiring an exact `host:port` entry with a port argument**: bounds
  nothing beyond the host match (§4).
- **Granting the resolved addresses to the same `Net`** (so resolve-then-
  connect-by-address works under a name grant): turns a name grant into
  an address grant for whatever the name's DNS returned at one moment, a
  second, time-dependent policy. Rejected; grant addresses explicitly.
- **`io::Result` like `connect`**: needs a `Dns` stand-in among
  `ErrorKind`s (today's `NotFound` for an empty answer and an
  uncategorized error for a resolver failure) and has no deadline; `get`'s
  `NetError` already names each outcome.
- **A Tide-owned resolver or DNS client** (hickory-resolver, raw UDP to
  `resolv.conf`'s servers): reimplements the operator's configuration,
  needs UDP and a dependency, and research/42 §6 decided against it.
- **Simulated addresses derived from a hash of the name**: independent of
  setup order, but two names can collide in the range; setup order is
  collision-free and already recorded.
- **A `sim.world.dns(name, addrs)` setup call** letting a test choose the
  addresses: a second source of simulated names next to `world.http`, for
  no current test. A later version may add it without changing §6.

## 9. Admission filter

**[inference]** GOVERNANCE.md's questions:

- *Safer change?* Yes, in a narrow way: name lookup becomes declared,
  `only`-bounded `net` authority with a deterministic simulated answer,
  where today it cannot be written at all, so a Rust port that calls
  `to_socket_addrs` has nowhere honest to go.
- *Overlapping ways?* One way to look up a name. To reach a named service
  the obvious shape stays `connect(name)` or `get(url)`. Resolve-then-
  connect-by-address is a different authority (address grants), and under
  a name grant it fails loudly with `PermissionDenied`, which keeps the
  name-based shape the one that works. Companion 14.1 gets the habit row.
- *Local reasoning?* Preserved: the hosts `resolve` may look up are the
  `only` entries' hosts, read in `main`.
- *Fast feedback?* Simulated lookups take no time or draw.
- *One obvious shape?* `net.resolve(host, deadline)`, mirroring `get`.
- *Library instead of core?* Std only.

**Verdict: admit. The design is sound** in A1's sense: every route by which
`resolve` could send a query `connect` could not, or reach an address
`only` does not grant (alternate spellings, IDNA and percent-encoding,
numeric names reaching `getaddrinfo`, port-free wildcards, empty and
trailing-dot names, IP literals, a second parser) is closed by NET-HOST
before the lookup, and `resolve` calls NET-RESOLVE exactly as `connect`
does. What it adds is information, the full address list of granted
hosts. The real backend retains research/42 §6's limits: the operator's
resolver configuration and caches, abandoned lookups that keep sending,
and timing. Section 6 gives the simulated backend's limits. Where
`resolve` is taught, state §3's *Not a binding* rule and §4's bind
exception.

**Worth its cost?** Narrowly yes: the cost (§10) is one method over the
existing pool, one simulated rule and a few reference tokens; the benefit
is closing v0's only gap in expressing name lookup. It is **not** needed
by fx-zu4f.8: no advisory in that pool does a bare lookup (§1), so the
screen's counts do not depend on it.

## 10. Cost

**[inference]** Implementation: about 40 lines in `net.rs` (steps 1–5,
then `resolve::lookup` with port 0, mapped to `IpAddr`, errors mapped as
`get` maps them), about 30 in `net/sim.rs` (the name order list and the
trace event), one line in NET-HTTP-4's fixed list, and the conformance
suite of §11. No dependency. Readers and agents: this stage changes only
`reference.md`'s §6.1 row (the signature, pointing to companion 21),
24,968 to 24,978 tokens by `scripts/count-tokens.sh`, inside D14's 25,000
with no trim; the rules are in the companion (§§14.1, 17, 21) and 03
§1.1. A fuller draft §6.7 line (the signature, result and grant scope)
measured 25,035, so stage 2, which drops the `planned` label, must hold
the file and trim about 60 tokens for a fuller line, or keep the pointer.
Entropy: none at the language level. The divergence an agent can trip on is Rust's habit of
resolving and then connecting by address, which under a name grant is
`Denied`; it fails loudly, never silently differently.

## 11. Stage 2

Stage 2 implements §§2–6 and adds cases for the planned `conformance/rules.md`
rows NET-RESOLVE-1 to -3. When the code lands, drop the `planned` label
of `resolve` from `reference.md`, 03, 01 and companion 21. Update
the exact `host:port` clause in reference §6.7 to say that `resolve`
matches by host on any port.

Add a small `#[cfg(test)]` seam in `net/resolve.rs` next to `CALLS` to
supply or hold a lookup answer. NET-RESOLVE-1's real `Dns` case and
NET-RESOLVE-3's shared-answer and held-lookup cases use the seam in
`tide-std`'s unit-test build. The conformance suite sends no live resolver
queries. Its done condition is the bead's Acceptance:

- **NET-RESOLVE-1, one admitted suite on both backends** (shared
  assertions for the real and simulated `Net`, as in
  `conformance/cases/authority/tcp.rs`): the same values and `NetError`
  variants for `"127.0.0.1"`, `"0x7f.1"`, `"[::1]"`, `"::1"`,
  `"[::ffff:127.0.0.1]"` and the unbracketed `"::ffff:127.0.0.1"`
  (both mapped literals return `[127.0.0.1]` as IPv4), and
  `"LOCALHOST"` (`[127.0.0.1]`); `Io(InvalidInput)`
  for `""`, `"."`, `"a..b"`, `"example.com:443"`, `"http://x/"`,
  `"b%C3%BCcher.example"`, `"xn--bcher-kva.example"` and `"*.test"`;
  `Denied` for an unlisted host, for a name under an address grant and
  for an address under a name grant, and for everything under `only([])`.
  A port-scoped entry grants its host; chained
  `only(["svc.test:443"]).only(["svc.test:80"])` gives `Denied` for
  `resolve("svc.test", deadline)` because the full-key intersection is
  empty. A passed deadline is `Timeout`, including for
  `resolve("127.0.0.1", past)`;
  `Cancelled` in a cancelled scope, winning over a passed deadline; and
  `Dns` for `"nothing.invalid"` without a peer in simulation and with
  a failure supplied through the seam on the real backend.
- **NET-RESOLVE-2, simulated determinism**: `localhost` returns
  `[127.0.0.1]` with and without a peer. Register `localhost:8080`
  first and an IP-address peer second. The next peer, at `a.test:443`,
  resolves to `198.18.0.1` on every seed, following §6's ordinal rule.
  Other domains without a peer return `Dns`. The call takes no virtual
  time and the run's draws equal a run without it;
  one `net.resolve` trace event per call; replaying a seed that resolves
  is byte-identical.
- **NET-RESOLVE-3, real backend**: a `Denied` or `InvalidInput` resolve
  makes zero resolver calls (NET-RESOLVE's `CALLS` counter, as
  NET-HTTP-5). With the seam supplying the same non-literal answer to
  both calls, `resolve` returns the list `connect`'s lookup uses with
  ports removed, preserving order and duplicates. Use the seam to hold
  a lookup past its deadline or cancel it mid-wait. The call returns at
  once with `Timeout` or `Cancelled`, and the abandoned thread releases
  its slot when the test lets the lookup finish.

The red-team witnesses add two cases:

- **R1.** In one setup, register `visible.test:443` first. In another,
  register `hidden.test:80` and `HIDDEN.test.:443` before `visible.test:443`.
  Give the caller only `visible.test:443` in both setups. Resolving
  `visible.test` yields `198.18.0.1` and `198.18.0.2`, respectively. Both
  registrations of `hidden.test` share one ordinal after canonicalization.
- **R2.** Register a peer with a 64-octet label and resolve it in
  simulation. For the same name on the real backend, supply a failure
  through the seam and check `Dns`.

Leftover for stage 2 (outside this stage's docs scope): `lib.rs`'s `net`
module comment (lines 280–287) still says names "use inline Rust
resolution that is not yet cancellable", which fx-ff13 retired; the
crate-root "Not provided yet" list names "explicit name resolution".
Stage 2 updates both.
