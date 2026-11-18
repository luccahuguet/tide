# Net::get in v0: one HTTP/1.1 GET, real and simulated (fx-ff13)

Status: design admission for bead fx-ff13, stage 1 (docs only). It fixes
the contract of `Net::get`, `Url`, `UrlError` and `NetError`, the host
rule that `Net::only` applies to outbound peer operations, the TLS
dependency, and the simulated HTTP peer, and gives the soundness verdict
that owner decision A1 asks for. Nothing here is implemented; stage 2
implements it after this note, its review rounds and a separate red-team
pass are accepted. Labels follow research/30 and research/40: **[repo
evidence]** for what the repository says, **[source]** for cited outside
evidence, **[inference]** for this note's reasoning. Revised after the
Codex supervisor pass s1-r1 (2026-11-18): hosts as written (§§2, 4),
rebinding (§6), status precedence (§§3, 9), the HTTP library comparison
and exact pins (§8), and the TCP key labelled as stage 2 in 03.

## 1. Need

**[repo evidence]** Owner decision A1, 2026-11-18, relayed by tide-desk and
quoted on fx-ff13: "Bring Proc::command into the alpha, plus a network
capability if the design is sound." The relay names fx-ff13 (HTTP) and
fx-6d50 (resolution) as the beads that carry the network capability, and
makes admission plus a red team the condition ("if the design is sound").
fx-zu4f.8, the pre-registered re-screen of the 87 RustSec malicious-crate
advisories, needs it: most of those crates download a payload or send
collected data to a host over HTTP(S), and Tide v0 can only express the
honest, capability-passing version of that code if a fetch exists.

**[repo evidence]** What exists. `Net::get(&Url, deadline: Instant) ->
Result<Vec<u8>, NetError>` is already taught as v0: reference §6.1 (planned)
and §6.7, §7.1's `fetch_both` and §7.2's operation deadlines, companion
§15.2 (the bounded fetcher and its `#[sim_test]`), §17 (`sim.world.http`,
`Reply`, network time and packet loss) and §21 (`NetError`, `Url`), 03
§1.1's Net row and "Network errors" note, 03 §9.3's network time, and The
Feel in 01. fx-p44q's audit found none of it built (W1): `crates/tide-std`
has TCP through `Net` (`connect`, `bind`, `only`, `no_bind`, a simulated
loopback), no HTTP, no TLS and no `Url`. So the API is decided; what is
not decided, and what this note decides, is (a) exactly what one `get`
does on the wire, (b) what `only` grants, as a host rule precise enough
to red-team, (c) the TLS dependency and its trust roots, and (d) the parts
of the simulated peer that the v0 examples need.

## 2. The surface (smallest)

**[inference]**

```rust
impl Net {
    pub fn get(&self, url: &Url, deadline: Instant) -> Result<Vec<u8>, NetError>;
}

pub struct Url { /* private: one parsed URL */ }
impl Url {
    pub fn parse(input: &str) -> Result<Url, UrlError>;
    pub fn scheme(&self) -> &str;
    pub fn host_str(&self) -> Option<&str>;
    pub fn port(&self) -> Option<u16>;
    pub fn port_or_known_default(&self) -> Option<u16>;
    pub fn path(&self) -> &str;
    pub fn query(&self) -> Option<&str>;
    pub fn fragment(&self) -> Option<&str>;
    pub fn as_str(&self) -> &str;
}
// Url: Clone, Debug, Display, FromStr, PartialEq, Eq, Hash, PartialOrd, Ord,
//      Encode and Decode (a JSON string through `parse`), Pure, Send, Sync.
// UrlError: Debug, Display, Error, Pure, Send, Sync; opaque.

#[non_exhaustive]
pub enum NetError {
    Timeout, Cancelled, Denied, Refused, Reset, Dns, Status(u16), Io(io::Error),
}
impl NetError {
    pub fn is_cancelled(&self) -> bool;
    pub fn is_transient(&self) -> bool;
}
// NetError: Debug, Display, Error, Pure, Send, Sync; not PartialEq.
```

- **`Url` is a Tide newtype, not a re-export.** The `url` crate's `Url` has
  `socket_addrs`, which resolves the host through Rust's ambient resolver
  ([url docs](https://docs.rs/url/2.5.8/url/struct.Url.html#method.socket_addrs)):
  re-exported, it would be a DNS lookup with no capability, outside `only`
  and outside simulation, and admission admits every public item of Tide's
  std (`walk.rs`, `Role::TideStd`). The newtype exposes the accessors above,
  which read the parsed value only, and no setters: a `Url` is immutable
  once parsed. Companion 21's four accessors are kept and `scheme`, `port`,
  `query` and `fragment` added, since a caller that decides what `get` will
  do needs them. `join`, setters, `from_file_path` and `to_file_path` are
  left out.
- **`Url::parse`** is the WHATWG URL parser of the `url` crate
  ([URL Standard](https://url.spec.whatwg.org/)), behind v0 restrictions
  that run first, on the text as written, each returning `UrlError`:
  1. the input is ASCII;
  2. for `http` and `https`, the host as written (§4, *Host as written*)
     contains no `%`, has no empty label, and has no label that starts with
     `xn--` (any case).

  The host parser percent-decodes a host and then applies IDNA, so without
  rule 2's `%` ban an all-ASCII input still reaches Unicode:
  `http://b%C3%BCcher.example/` becomes `xn--bcher-kva.example`,
  `http://%78n--bcher-kva.example/` hides the `xn--` prefix, and
  `http://%EF%BD%81pi.example.com/` maps a fullwidth `ａ` to
  `api.example.com` ([URL Standard, host parsing](https://url.spec.whatwg.org/#host-parsing):
  the host parser percent-decodes the host before its domain parser runs
  IDNA). With ASCII input and no `%` in the host, the host the parser sees
  is the host as written, and IDNA sees only ASCII labels, which it
  lowercases or refuses. Percent-encoding stays legal in paths, queries
  and fragments; callers percent-encode non-ASCII text there. So v0 has no
  internationalized host names, in Unicode, Punycode or percent-encoded
  form, and a reviewer reading a URL or an `only` entry never meets a
  homograph. Relaxing these rules later is additive. Other schemes parse
  under rule 1 only, and `get` refuses them before reading their host
  (§3 step 2); a percent-encoded non-ASCII host in one of them may parse
  or fail depending on §8's `idna_adapter` choice, which reaches no
  network either way.
- **`NetError` is `#[non_exhaustive]`**, so a later variant (a TLS one, for
  example) is not a breaking change; companion 15.2's `match` already has a
  catch-all arm. Its variants and classification are companion 21's,
  unchanged (§5).

What it adds. To the language: nothing; no syntax, no checker rule, no new
authority kind (`net` exists and `Net` is already a capability). To std:
one method, two types, an HTTP/1.1 client over the existing nonblocking
TCP path, TLS, a private resolver (§7), and a simulated peer (§9). To the
checker: nothing beyond guarding that `tide_std::net` exposes no other
public path to the network (§11). To readers: a few tokens net in
`reference.md` (§12), the rest in the companion and 03.

## 3. What one `get` does

**[inference]** NET-GET. Both backends follow this order; every step that
fails returns at once, and no later step runs:

1. **Cancellation.** `get` is a checkpoint: a cancelled scope returns
   `Cancelled` before anything else.
2. **Shape.** The scheme is `http` or `https`; the URL has a host (a domain
   or an IP address) and no userinfo (`user:pass@`). Otherwise
   `Io(InvalidInput)`. Userinfo is refused rather than sent, so an
   `Authorization` header never appears from a URL, and a reader never has
   to decide which side of `@` is the host.
3. **Authority.** The URL's host and port (`port_or_known_default`: 80 for
   `http`, 443 for `https`) are checked against `only` by the host rule of
   §4. Outside it: `Denied`. No lookup, connection or simulated peer call
   has happened yet.
4. **Deadline.** If `deadline` has passed, `Timeout`.
5. **Resolution** of a domain host, through the private resolver of §7,
   once per call; `Dns` on failure. IP hosts and `localhost` skip it.
6. **Connection** to the resolved addresses in order, one TCP connection,
   over the same nonblocking sockets and cancellable waits `connect` uses.
   If all attempts fail, `Refused` when every address refuses; otherwise
   `Io` of the first non-refusal connect error. A later successful attempt
   still proceeds normally; cancellation and deadline retain precedence.
7. **TLS** for `https` (§8): the certificate is verified for the key's host
   (a DNS name, or an IP address against the certificate's IP entries); SNI
   carries the key's DNS host and is omitted for an IP host; ALPN offers
   only `http/1.1`. Any handshake or verification failure is `Io(InvalidData)`.
   There is no fallback to plaintext and no option to skip verification.
8. **Request**, exactly:

   ```text
   GET <path>[?<query>] HTTP/1.1
   Host: <host>[:<port> when not the scheme's default]
   User-Agent: tide
   Accept: */*
   Connection: close

   ```

   `Host` uses the key's host, with IPv6 in brackets, and the port only
   when not the scheme's default. The path and query are the parser's
   serialization, which
   percent-encodes spaces and control characters and removes tabs and
   newlines from the input ([URL Standard, basic URL parser](https://url.spec.whatwg.org/#concept-basic-url-parser)),
   so the request line cannot be split. The fragment is never sent. No
   body, no other header, no cookies, no `Accept-Encoding`.
9. **Response.** HTTP/1.1 or HTTP/1.0 status line and headers, parsed by
   `httparse` (§8), at most 100 headers and 64 KiB of heads in all, interim
   ones included. Then, in this order, the first rule that applies decides
   ([RFC 9112 §6.3](https://www.rfc-editor.org/rfc/rfc9112#section-6.3)
   orders them so; `get` sends no `HEAD` or `CONNECT`, so its rules for
   those never apply):
   1. **`101 Switching Protocols`** is `Io(InvalidData)`: `get` sends no
      `Upgrade`, and a server "MUST NOT switch to a protocol that was not
      indicated by the client" ([RFC 9110 §7.8](https://www.rfc-editor.org/rfc/rfc9110#section-7.8));
      `get` never speaks another protocol on the connection.
   2. **Other `1xx`** (interim) responses end at their blank line and are
      skipped; `get` reads the next head. A close before a final head is
      `Reset`.
   3. **`204` and `304`** end at the blank line after the head, whatever
      `Transfer-Encoding` or `Content-Length` they carry (a `304`'s
      `Content-Length` describes the representation, not a body:
      [RFC 9110 §8.6](https://www.rfc-editor.org/rfc/rfc9110#section-8.6)). `get`
      returns at once, without waiting for the peer to close: `204` is
      `Ok` with an empty body and `304` is `Status(304)`.
   4. **Framing** of every other status: `Transfer-Encoding: chunked`, else
      `Content-Length`, else the body runs to connection close. A response
      with both is `Io(InvalidData)`, since §6.3 says it "ought to be
      handled as an error", and so is an invalid `Content-Length`
      (differing repeated values included), which a user agent "MUST"
      discard. A transfer coding other than
      `chunked` is also `Io(InvalidData)`, stricter than §6.3's
      read-to-close, because `get` undoes no coding.

      Under `https`, a body delimited by connection close ends only at the
      peer's TLS `close_notify`; a close without it is `Reset` (rustls's
      `UnexpectedEof` included). A framed body complete before a bare close
      stays `Ok` for a 2xx response; no `close_notify` is needed then.

   The body is returned as received after chunked decoding (trailers are
   read and discarded); no content coding is undone. A body over 64 MiB is
   `Io(InvalidData)` (provisional). The connection closes before `get`
   returns.
10. **Result.** A `2xx` status returns `Ok(body)`. Any other final status,
   `3xx` included, returns `Status(code)` and discards the body. A
   connection reset, or a close before the final head or the framed body
   is complete, is `Reset`. A malformed response is `Io(InvalidData)`.

**Deadline.** `deadline` is an `Instant` from a `Clock` of the same world
(reference §7.2; instants of different clocks compare as plain offsets,
as `time.rs` says). It bounds the whole call: resolution, every connection
attempt, the handshake, the request and the full body. When it passes
during any step, the call stops, drops the connection and returns
`Timeout`. Scope cancellation at any wait returns `Cancelled`; when both
hold at one wait, `Cancelled` wins, since cancellation is level-triggered
and checked first. A scope deadline (`s.set_deadline`) cancels the scope,
so it yields `Cancelled`, not `Timeout`: companion 15.2 passes its deadline
to `get` to get `Timeout`, and 03 §12.2, which sets both to one instant,
gets `Cancelled`, because all timers of an instant fire before any task
runs (03 §9.3). Like every kernel wait in v0, the deadline is soft
by the time an uninterruptible step takes (TASK-3): §7's resolver is the
one step that cannot be interrupted, and `get` abandons it rather than
waiting for it.

**Redirects are not followed.** A `3xx` is `Status(code)`. Following one is
a second request to a URL the server chooses, with its own rules for
method changes, header carrying, loops and scheme downgrades; Deno's
`fetch` once carried an `Authorization` header across origins on a
redirect (CVE-2025-21620, [GHSA-f27p-cmv8-xhm6](https://github.com/denoland/deno/security/advisories/GHSA-f27p-cmv8-xhm6)).
Following only within `only` would be safe for authority, since each hop
could take the same check, but it is a second policy for no current
caller: code that knows the final URL fetches it. Stated limit: a URL that
redirects (a moved page, `http` to `https`) fails with its `3xx`. A later
version may add following as a separate, opt-in call; changing `get` to
follow would change its results, so it will not.

**HTTP/1.1 only, one request per connection.** No HTTP/2 (it needs ALPN
negotiation, HPACK and stream multiplexing, several times the client and a
second dependency), no HTTP/3, no keep-alive, no pipelining, no connection
pool. Every `get` opens and closes its own connection.

## 4. What `only` grants: the host rule

**[source]** How others match host allowlists:

- Deno's `--allow-net` takes hostnames or IP addresses with an optional
  port; a name does not match its subdomains unless listed, `*.example.com`
  matches subdomains, and an entry without a port allows every port
  ([Deno permissions](https://docs.deno.com/runtime/reference/permissions/#network-access)).
  Its checks compare the literal host string before resolution: in
  `node:net` and `node:http` the decimal and hexadecimal aliases of an
  address (`2130706433`, `0x7f000001`) passed a deny rule for `127.0.0.1`
  (CVE-2026-49411, [GHSA-v8fw-85r8-5m23](https://github.com/denoland/deno/security/advisories/GHSA-v8fw-85r8-5m23)),
  and `fetch` and WebSocket did not re-check resolved addresses against
  deny rules (CVE-2026-49859, [GHSA-cpgj-f7g3-2pp2](https://github.com/denoland/deno/security/advisories/GHSA-cpgj-f7g3-2pp2);
  CVE-2026-49860, [GHSA-83pc-3rw9-qpwj](https://github.com/denoland/deno/security/advisories/GHSA-83pc-3rw9-qpwj)), until
  Deno 2.8.1 added a post-resolution check.
- The URL Standard's host parser turns every IPv4 spelling it accepts
  (decimal, octal and hexadecimal parts, fewer than four parts) into one
  32-bit address when the host ends in a number, and lowercases and
  ASCII-encodes domains ([URL Standard, host parsing](https://url.spec.whatwg.org/#host-parsing));
  the `url` crate implements it (`host.rs`, `ends_in_a_number`).
- Wasmtime's `wasi:http` has no built-in allowlist; a host that wants one
  checks the request's authority in its `send_request` hook
  ([wasmtime issue #10527](https://github.com/bytecodealliance/wasmtime/issues/10527)),
  and wasmCloud's `allowedHosts` does so for its components' `wasi:http`
  calls ([wasmCloud workload security](https://wasmcloud.com/docs/kubernetes-operator/workload-security/)).
- Browsers' same-origin policy compares scheme, host and port exactly
  after URL parsing, with no lookup
  ([HTML, origin](https://html.spec.whatwg.org/multipage/browsers.html#origin)).

**[inference]** Deno's failures share one cause: the check and the
connection read the host differently (a string versus a resolved
address, or one parser versus another). Tide avoids both by checking a
canonical key derived from one parse, before any lookup, and by granting
names, not addresses (an allowlist, where Deno's failures were in deny
rules).

**Decision.** NET-HOST, one rule for outbound peer operations: `connect`,
`get`, and the planned `resolve` and `bind_udp` peer operations. `only`
does not limit `bind` or accepted connections. A binding-capable Net can
resolve arbitrary bind names, even if binding fails, and communicate with
unlisted inbound peers. Delegate `only([..]).no_bind()` for an
outbound-only destination grant. An empty `only` list alone does not
disable networking. Bind semantics stay unchanged; `no_bind` refuses bind
before resolution.

- **Entries.** An `only` entry is `"host:port"`, with an IPv6 address in
  brackets (`"[::1]:8080"`). The port is required and must be 1–65535;
  there are no wildcards, no subdomain matching, no port ranges, no
  schemes. An entry that does not parse under this rule grants nothing,
  as today (`"https://example.com"`, `"example.com"` without a port,
  `"*.example.com:443"`, a non-ASCII, `xn--`, percent-encoded or
  empty-label name). Chained `only` calls intersect, as today.
- **Host as written.** The v0 checks below read the host before the URL
  Standard's host parser, so before its percent-decoding and IDNA:
  - for an entry or a `connect` address, the text before the final
    `:port` (inside the brackets for IPv6);
  - for an `http` or `https` URL, the text the basic URL parser would hand
    to its host parser: after its own first steps (leading and trailing
    C0 controls and spaces trimmed, tabs and newlines removed), the text
    after `scheme:` and any run of `/` and `\`, up to the first `/`, `\`,
    `?` or `#`, then after the last `@` and before the first `:` outside
    brackets ([URL Standard, special authority ignore slashes, authority
    and host states](https://url.spec.whatwg.org/#special-authority-ignore-slashes-state)).
    Stage 2 tests that this span equals the host the `url` crate parses,
    on NET-HTTP-1's cases.
- **v0 restrictions on the host as written**, each refusing the entry
  (it grants nothing), the address (`InvalidInput`) or the URL
  (`UrlError`): ASCII only; no `%`, so no percent-encoded host
  (`b%C3%BCcher.example`, `%78n--bcher-kva.example`,
  `%EF%BD%81pi.example.com` are all refused, though the parser would turn
  them into an IDN, a hidden `xn--` label and `api.example.com`); no empty
  label (`a..b`, `.a`, two trailing dots; one trailing dot is allowed),
  which the URL parser itself accepts; no label that starts with `xn--`,
  in any case. Brackets hold an IPv6 address and nothing else (so no zone
  identifier, which needs a `%`). These checks run before the host parser,
  which then sees only ASCII without escapes, so its IDNA step never sees
  Unicode.
- **Canonical key.** Entries, URL hosts and `connect` addresses all reduce
  to one key by the same function:
  - a host that passed the restrictions is parsed by the URL Standard's
    host parser for special schemes, so every IPv4 spelling it accepts
    (`0x7f.1`, `2130706433`, `0177.0.0.1`) becomes the address it denotes,
    `127.0.0.1`;
  - an IPv6 address is compared as an address (`[::1]` equals
    `[0:0:0:0:0:0:0:1]`); an IPv4-mapped address (`[::ffff:127.0.0.1]`)
    becomes its IPv4 address, so each destination has one key; zone
    identifiers are refused, as the URL parser refuses them;
  - a domain is ASCII-lowercased and loses one trailing dot
    (`Example.COM.` equals `example.com`);
  - the key is the host with the port: `(host, port)`. A URL without a port
    keys as 80 for `http` and 443 for `https`.
- **Matching** is equality of keys. `only(["example.com:443"])` allows
  `https://example.com/` and `https://EXAMPLE.com.:443/x`, and refuses
  `https://www.example.com/`, `http://example.com/` (port 80), and
  `https://93.184.215.14/` even if that is example.com's address: a name
  entry grants the name, an address entry grants the address, and neither
  grants the other. `localhost` is a name like any other: it does not match
  `127.0.0.1`.
- **No outbound peer lookup before the check.** The check uses only the
  key, so a refused peer is never resolved, connected or shown to a
  simulated peer. This does not constrain bind-name resolution above.
- **One parse.** The key's host is resolved, dialled, verified by TLS and
  sent in `Host` (IPv6 bracketed) and SNI (DNS only). Nothing re-parses the
  URL string later, so no parser difference can separate the check from the
  connection.

**Change to TCP (stage 2).** `connect` today keys a host by
`to_ascii_lowercase()` or `IpAddr` parsing (`net.rs`, `Address::key`).
NET-HOST changes three cases, all toward one key per destination: numeric
IPv4 aliases become their address (today `"0x7f.1:80"` is a name that only
an identical entry matches, and Rust's resolver then dials `127.0.0.1`),
an IPv4-mapped address keys as IPv4, and a trailing dot is dropped. A
non-ASCII, percent-encoded, empty-label or `xn--` host becomes
`InvalidInput`. TCP-1 takes these as cases.

## 5. `NetError` classification

**[repo evidence]** Companion 21 and 03's "Network errors" note fix the
variants and `is_transient()`: true for `Timeout`, `Reset`, `Refused` and
`Status` 408, 429 or 5xx; false for `Cancelled`, `Denied`, `Dns`, `Io`.

**[inference]** NET-ERR: which failure maps to which variant, on both
backends.

| Variant | Real backend | Simulated backend |
| --- | --- | --- |
| `Timeout` | `deadline` passed at any step of §3 | the same, in virtual time; a `Reply::hang()` peer |
| `Cancelled` | the scope was cancelled at entry or at any wait | the same |
| `Denied` | host and port outside `only` | the same |
| `Refused` | every resolved address refused the connection | an IP host, or a domain some peer or node names, on a port no HTTP peer serves; after one round trip |
| `Reset` | the connection was reset, closed before the head or framed body was complete, or ended a close-delimited HTTPS body without TLS `close_notify` | `Reply::reset()` |
| `Dns` | the resolver failed or returned no address | a domain that no peer or node names, at once |
| `Status(code)` | a final status outside 200–299 | `Reply::status(code)`, 300–599 |
| `Io(e)` | `InvalidInput`: §3 step 2; `InvalidData`: TLS handshake/verification failure, malformed or oversized response, unsupported framing, a `101`; other connect or socket errors with their own kind | `InvalidInput` for step 2; `Reply::tls_error()` gives `InvalidData` |

`Display` names the variant and, for `Status`, the code (`HTTP status
404`); `Io` displays its error. `is_cancelled()` is true only for
`Cancelled`. A TLS handshake or verification failure is `Io(InvalidData)`,
not a new variant: no current caller acts on it differently (it is not transient either way),
and `#[non_exhaustive]` leaves room for one. TLS stream truncation follows
the `Reset` rule above rather than the handshake-failure mapping.

## 6. Authority semantics, plainly

**[inference]** `only` bounds where `get` and `connect` go: which names and
addresses, on which ports. It bounds nothing else.

- **Data to an allowed host.** The path and query carry whatever the code
  puts in them to the allowed host, and `connect` to the same `host:port`
  sends any bytes at all, including a hand-written HTTP request with any
  header. `get`'s fixed request (no custom headers, no body) is a
  simplicity choice, not a security boundary. An allowed host that is
  hostile, compromised, or a general relay (a URL-fetching service, an
  open proxy, a CDN that routes by path, a paste site) receives the data,
  and a relay makes the grant full network authority in practice, as an
  allowed shell does for `Proc` (research/40 §4). Stated limit.
- **Data from an allowed host.** The body is whatever the host returns.
  Code that holds `Proc` or `Fs` can run or store it; `Net` alone cannot.
- **Names are trusted to their owners.** A name entry grants whatever the
  name resolves to when `get` runs, through the operator's resolver
  (§7). Whoever controls the name's DNS can point it at any address,
  including loopback and private ones (DNS rebinding). A name grant
  therefore permits TCP contact and handshake traffic to any address the
  name resolves to, on the granted port, under `https` too: `get` connects
  and sends its TLS ClientHello before the peer's certificate can be
  checked ([RFC 8446 §2](https://datatracker.ietf.org/doc/html/rfc8446#section-2)),
  so a rebound name reaches, say, a loopback service on that port. What
  `https` adds is narrower: a completed `https` `get` has authenticated the
  name, by certificate, before it sends any HTTP data, so no request line,
  path or query reaches a peer without a certificate for the name. Under
  `http` the request reaches any server at the resolved address, which
  then sees `Host: <the granted name>`. And code that holds the same `Net`
  can `connect` to the granted `host:port` and send raw bytes with no TLS
  at all. Tide does not filter resolved addresses: there is no
  private-address filter, since one would break the common grant of an
  internal name (`db.internal`), and an allowlist entry is a statement
  that the code may talk to whatever that name's owner points it at.
  Stated limit; grant an IP address where the distinction matters.
- **Observers.** Each `get` shows the granted name to the resolver and the
  network (DNS query, TLS SNI) and the client's address to the server.
  Timing and the number of requests are a channel to the allowed host and
  to its DNS operator.
- **Unattenuated `Net`** (`root.net`) allows every host, as today.

What `get` never does, on either backend, whatever the environment says:

| Input | Behavior | Reason |
| --- | --- | --- |
| `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` and their lowercase forms | ignored; `get` always dials the URL's host | a proxy makes the TCP peer a host outside `only`; httpoxy (CVE-2016-5385 and siblings, [httpoxy.org](https://httpoxy.org/), [CERT VU#797896](https://kb.cert.org/vuls/id/797896)) showed request input reaching `HTTP_PROXY` |
| `SSL_CERT_FILE`, `SSL_CERT_DIR`, system trust stores | ignored; roots are compiled in (§8) | trust is not steerable from the environment, and replay identity covers the roots |
| `SSLKEYLOGFILE` | ignored; `NoKeyLog`, never `KeyLogFile` | no ambient filesystem write or TLS-secret disclosure (§8) |
| `.netrc`, cookie jars, HSTS, Tide-owned DNS caches, TLS session tickets | none exist | no Tide-owned state crosses calls or attenuated `Net`s (below); system resolver caches remain |
| `file:`, `data:`, `ftp:`, `ws:` and every other scheme | `Io(InvalidInput)` at step 2 | `get` is HTTP; `file:` would be an `Fs` read through `Net` |

**Connection reuse.** None. Each call opens, uses and closes one
connection; TLS resumption is disabled. Tide keeps no DNS cache of its own.
Two `Net`s attenuated differently share no connection or TLS session, and
a narrowed `Net` cannot ride a connection or ticket that a wider one made.
Cost: a full handshake per `https` call.

**Residual limit, resolver configuration.** Name-to-address mapping is the
operator's: `/etc/hosts`, `nsswitch.conf`, `resolv.conf` and its search
domains, and the C library's own variables (glibc's `HOSTALIASES`,
`RES_OPTIONS`, `LOCALDOMAIN`). The system resolver may share caches across
calls and Nets, substitute configured aliases, and follow DNS aliases
([hostname(7)](https://man7.org/linux/man-pages/man7/hostname.7.html),
[nscd(8)](https://man7.org/linux/man-pages/man8/nscd.8.html),
[resolver(3)](https://man7.org/linux/man-pages/man3/resolver.3.html)).
Cancellation abandons the caller's lookup result; it does not stop the
resolver's remaining traffic. The grant authorizes resolution through this
operator-controlled mechanism. Tide does not reimplement resolution to
avoid this.

## 7. Resolution and fx-6d50

**[repo evidence]** `bind` and `connect` share `Address::resolve`, which
resolves names other than `localhost` and IP literals with Rust's inline
resolver, which "cannot be abandoned on
cancellation" (03 §1.1, `net.rs` "shortcut" comment), pending "`Net::resolve`
owns resolution". Simulation rejects such names for TCP.

**[inference]** NET-RESOLVE, the private mechanism both beads share:

- One private function in `tide_std::net` resolves a domain with the system
  resolver (`getaddrinfo` through Rust's `ToSocketAddrs`, so `/etc/hosts`
  and `nsswitch` apply), on a helper thread from a small bounded pool
  (provisional size), while the calling task waits cancellably with the
  call's deadline if any. On cancellation or deadline the caller returns at once
  and the lookup is abandoned: its thread finishes on its own, its result
  is dropped, and nothing it does is visible to Tide code. When the pool is
  full, a call waits, cancellably, for a slot. Stated limit: an abandoned
  lookup keeps its thread until the resolver's own timeout and may keep
  sending resolver traffic after the Tide call returns.
- `get` uses it from stage 2. Both `bind` and `connect` move onto it in
  the same stage, since they share `Address::resolve`, which retires 03's
  "cannot be abandoned" limit: one resolver, one owner. Bind retains its
  existing authority policy and early `no_bind` check. Calls without a
  deadline wait cancellably, without adding a public deadline parameter.
- Addresses are tried in the order the resolver returns them, sequentially
  (no Happy Eyeballs racing), within the caller's deadline when it has one.

What fx-6d50's `Net::resolve(host)` must keep consistent:

1. **Same key.** `resolve` takes a host without a port and parses it with
   NET-HOST's host part. Under `only`, it is allowed when some entry has
   that host, on any port: resolution reveals addresses, not a connection,
   and a code that may connect to `example.com:443` could learn the
   address anyway. It returns `Denied`, before any lookup, otherwise.
2. **Same mechanism.** It calls NET-RESOLVE, so `resolve(h)` returns the
   addresses `get` and `connect` would try for `h`, in the same order.
3. **Same simulated names.** Under simulation, a host that a `world.http`
   peer or a node names resolves (to a simulated address fx-6d50 assigns,
   deterministically from the setup), and any other domain is `Dns`, as
   `get` returns. IP literals need no name lookup. `get` itself needs no
   simulated address and reaches only `world.http` peers, not TCP listeners.
4. **Its error type.** `resolve` returns `io::Result` like `connect`, with
   `PermissionDenied` for a refused host, or `NetError` like `get`; that
   choice is fx-6d50's, as long as it maps the same conditions.

## 8. Dependencies (AP-DEPENDENCY-001)

Today `tide-std` depends on `libm`, `rustix` and `tide-macros`, all pure
Rust. Stage 2 adds four capabilities: a TLS client, trust roots, HTTP/1.1
response reading and URL parsing.

**TLS client.** Contract: TLS 1.2 and 1.3 client with certificate and host
name verification (DNS names and IP addresses), SNI and ALPN, driven over
the caller's own nonblocking socket, so `get` keeps Tide's cancellable
waits.

**TLS guard.** TLS uses `NoKeyLog` and disabled resumption; Tide never
constructs `KeyLogFile`, and `SSLKEYLOGFILE` is ignored. Disabling the
`logging` feature alone does not establish this. The pinned rustls builder
defaults to `NoKeyLog` but enables resumption, which Tide must override;
constructing `KeyLogFile` reads the variable and can create or append a
file ([builder](https://docs.rs/crate/rustls/0.23.46/source/src/client/builder.rs),
[key logger](https://docs.rs/crate/rustls/0.23.46/source/src/key_log_file.rs);
published crate source inspected locally, 2026-11-18).

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `rustls` (Apache-2.0 OR ISC OR MIT) with the `ring` provider; exact pins below | **chosen** | memory-safe protocol code; sans-I/O `ClientConnection` (`read_tls`/`write_tls`) fits Tide's socket waits; no environment input; maintained under ISRG's Prossimo; default features off, `ring`, `std` and `tls12` on, so no `brotli`/`zlib` certificate compression and no `aws-lc-rs` |
| `rustls` with `aws-lc-rs` (its default provider) | rejected for v0 | builds AWS-LC, a much larger C and assembly library, for no capability `get` needs; FIPS builds also need CMake and Go ([aws-lc-rs requirements](https://aws.github.io/aws-lc-rs/requirements/index.html)) |
| `native-tls` (OpenSSL on Linux) | rejected | links the system OpenSSL, whose behavior and trust store follow host configuration and `SSL_CERT_*` variables; the library sits outside the replay identity's hashed dependencies (03 §9.2) |
| `openssl`, `boring`, `s2n-tls` | rejected | C or C++ TLS stacks with system or CMake builds; same objections, larger |
| `rustls-rustcrypto`, `graviola` providers | rejected for now | pure Rust or Rust-plus-assembly, but younger and less reviewed than `ring`; replacing the provider later is one value (below) |
| a TLS stack of Tide's own | rejected | the riskiest code to own |

**Trust roots.** Contract: a fixed set of public web roots, the same on
every machine, unaffected by the environment.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `webpki-roots` (CDLA-Permissive-2.0): Mozilla's root program, from CCADB, compiled in | **chosen** | identical on every host, no file or environment read, covered by the build identity, updated with the crate |
| the platform store (`rustls-platform-verifier`, `rustls-native-certs`) | rejected for v0 | on Linux it reads the system bundle through `openssl-probe` and honors `SSL_CERT_FILE` and `SSL_CERT_DIR` ([rustls-native-certs `lib.rs`](https://github.com/rustls/rustls-native-certs/blob/main/src/lib.rs)); trust would vary by machine and be steerable from the environment; private and intercepting CAs, its main benefit, are not a v0 need |

Stated limits: private CAs, corporate TLS interception and roots added
after the pinned crate are not trusted in v0; a later explicit root-store
attenuation, built from an `Fs` read in `main`, would add them. The pinned
version must be refreshed like any dependency.

**HTTP/1.1 response reading.** Contract: parse a status line and headers
from bytes the caller supplies, and apply §3 step 9's framing (status
precedence, chunked, `Content-Length`, close) with Tide's stricter error
mapping, without owning the socket. Two candidates fit; both are sans-I/O,
so cancellation and the simulator stay outside them either way.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `httparse` (heads and chunk sizes) plus Tide-owned framing | **chosen** | no dependencies; the head parser `hyper` uses, fuzzed upstream; stable 1.x API; parses chunk sizes ([`parse_chunk_size`](https://docs.rs/httparse/1.10.1/httparse/fn.parse_chunk_size.html)). Tide owns the framing rules of §3 step 9, about 150 lines, which NET-HTTP-2 tests case by case |
| `ureq-proto` (sans-I/O HTTP/1.1 client: chunked, `Content-Length`, close, 100-continue, connection states; no sockets or TLS, [docs](https://docs.rs/ureq-proto/0.7.1/ureq_proto/)) | rejected for v0 | it would own the framing Tide must otherwise get right, and its `response_body_allowed` already gives 1xx, 204 and 304 precedence (`body.rs`). Against it: its policy differs from §3 and Tide would re-check every head anyway (a response with both `chunked` and `Content-Length` is decoded as chunked, after RFC 2616, where §3 refuses it; a `101` is handed back as an upgrade); most of its 5,600 lines (request bodies, 100-continue, redirect and pool states, a server side) serve no fixed GET; it adds `http` (with `bytes` and `itoa`), `base64` and `log` to `httparse`, which it uses itself; and its 0.x API broke three times in fifteen months (0.5.0 2025-08-17, 0.6.0 2026-03-21, 0.7.0 2026-11-01, [crates.io](https://crates.io/crates/ureq-proto/versions)), each a review of a library inside the trust boundary |
| `ureq` 3 with a custom `Transport` and `Resolver` | rejected | its transport and resolver traits do let a caller supply sockets and lookup, but they sit in `ureq::unversioned`, whose breaking changes "will NOT be reflected in a major version bump" ([ureq docs](https://docs.rs/ureq/3.5.2/ureq/unversioned/index.html)); it brings redirect, proxy (picked up from `ALL_PROXY`, `HTTPS_PROXY`, `HTTP_PROXY` and `NO_PROXY` by default: [`ConfigBuilder::proxy`](https://docs.rs/ureq/3.5.2/ureq/config/struct.ConfigBuilder.html#method.proxy), [`Proxy::try_from_env`](https://docs.rs/ureq/3.5.2/ureq/struct.Proxy.html#method.try_from_env)) and cookie policy that `get` would have to configure away and keep proving off; it is `ureq-proto` plus everything `get` refuses |
| `hyper` | rejected | async; needs an executor Tide does not have |
| a hand-written head parser | rejected | more owned parsing code for the same contract as `httparse` |

[inference] The choice is close. `ureq-proto` would remove most framing
code from Tide, and the first supervisor review of this note (s1-r1,
finding B3) showed that framing is easy to get wrong;
`httparse` keeps the dependency graph one leaf crate and the API stable,
and Tide's framing is the policy the conformance cases pin either way.
Over the whole lifecycle (review of each upgrade, replay identity, the
strict error mapping, a fixed GET with no body or reuse) the smaller,
stable graph wins. Replacement: the framing sits behind one private
response reader, so moving to `ureq-proto` later replaces that reader and
keeps NET-HTTP-2's cases.

**URL parsing.** `url` (MIT OR Apache-2.0), the WHATWG parser that Rust
code already uses, behind the newtype of §2. Its `idna` dependency is
reached only for ASCII labels without `%` or `xn--`, which §2's and §4's
restrictions guarantee. `idna` takes its Unicode back end from
`idna_adapter` by version: 1.0.x is a stub with no Unicode data that
rejects non-ASCII domains, 1.1.x uses unicode-rs, 1.2.x (Cargo's default)
uses ICU4X ([idna_adapter README](https://docs.rs/crate/idna_adapter/latest)).
Stage 2 pins the 1.0.0 stub if a test shows that NET-HTTP-1's ASCII hosts
parse identically under it and under 1.1.0, and otherwise pins 1.1.0.
Rejected: a Tide-owned URL parser (a second WHATWG implementation to keep
correct).

**Pins.** Proposed exact releases, read from crates.io on 2026-11-18
(versions, publication dates, licenses and feature lists from
`https://crates.io/api/v1/crates/<name>/<version>`). This stage changes no
`Cargo.toml` or lockfile; stage 2 adds them, builds them and proves the
relied-on behavior (§14). A newer patch release at that time may replace
a version here if stage 2 records why.

Direct dependencies of `tide-std`, pinned with `=` as `rustix` and `libm`
are:

| Crate | Version | Features | License | Role |
| --- | --- | --- | --- | --- |
| `rustls` | `=0.23.46` (2026-10-14) | `default-features = false`; `ring`, `std`, `tls12` | Apache-2.0 OR ISC OR MIT | TLS client; no `aws_lc_rs`, `logging`, `brotli`, `zlib` or `prefer-post-quantum` |
| `ring` | `=0.17.14` (2025-03-11) | default (`alloc`, `dev_urandom_fallback`), as `rustls`'s `ring` feature enables them | Apache-2.0 AND ISC | the crypto provider; named directly only to pin it, used through `rustls::crypto::ring` |
| `webpki-roots` | `=1.0.12` (2026-11-10) | none | CDLA-Permissive-2.0 | compiled-in Mozilla roots |
| `httparse` | `=1.10.1` (2025-03-03) | `default-features = false` | MIT OR Apache-2.0 | response heads and chunk sizes |
| `url` | `=2.5.8` (2026-01-05) | default (`std`) | MIT OR Apache-2.0 | `Url::parse` |
| `idna_adapter` | `=1.0.0` (2024-10-29), conditionally `=1.1.0` (2024-10-29) | none | Apache-2.0 OR MIT | named directly only to select `idna`'s back end; Cargo otherwise picks 1.2.x and ICU4X |

Transitive dependencies these bring, at the newest release each
requirement admits on 2026-11-18; stage 2's committed `Cargo.lock` fixes
them exactly and its report lists any that differ:

- through `rustls`: `rustls-webpki` 0.103.17 (`ring`, `std`),
  `rustls-pki-types` 1.16.0, `subtle` 2.6.1, `zeroize` 1.9.2, `once_cell`
  1.21.4;
- through `ring`: `untrusted` 0.9.0, `getrandom` 0.2.17, `cfg-if` 1.0.5,
  `libc` (already in the lockfile, 0.2.191, through `getrandom`), and at
  build time `cc` 1.11.2 with `shlex` 2.0.1 and `find-msvc-tools` 0.1.15;
- through `url`: `idna` 1.1.0, `percent-encoding` 2.3.2,
  `form_urlencoded` 1.2.2, `smallvec` 1.16.6, `utf8_iter` 1.1.0;
- only under `idna_adapter` 1.1.0: `idna_mapping`, `unicode-bidi`,
  `unicode-normalization` and their own dependencies.

`ring` is `tide-std`'s first dependency with C and assembly sources; it
"requires a C (but not C++) toolchain", and its crates.io package needs no
Perl ([ring BUILDING.md](https://github.com/briansmith/ring/blob/main/BUILDING.md)).
The build environment already has one (rustc links through `cc`); stage 2
checks the nix flake and Docker builds. The admission driver's dependency
identity hashes the built library like any other, so replay identity
covers it.

**ring maintenance (checked 2026-11-18).** crates.io still lists 0.17.14
(2025-03-11) as newest: about 20 months without a release
([crate metadata](https://crates.io/api/v1/crates/ring)). In February 2025,
the rustls maintainers gained publishing access after the author's hiatus
([maintainer record](https://github.com/rustsec/advisory-db/pull/2230)).
RustSec withdrew RUSTSEC-2025-0007 and later records a return to normal
maintenance, rather than security-only maintenance
([advisory updates](https://rustsec.org/advisories/RUSTSEC-2025-0007.html));
the upstream [release request](https://github.com/briansmith/ring/issues/2840)
remains open. The release lag is a maintenance risk, not evidence of no
maintenance. The choice stays `ring` for the smaller build and established
rustls integration; a provider swap remains one feature and one
`CryptoProvider` value. Stage 2 rechecks releases and advisories when pinning.

**Replacement cost.** The TLS stack sits behind one private module
(connect, handshake, `Read`/`Write`): swapping the provider is a feature
and one `CryptoProvider` value; swapping `rustls` for another stack
rewrites that module only. `httparse` and the framing sit behind the
response reader. `url` sits behind `Url`'s accessors, which are its names,
so replacing it keeps the API. Stage 2 proves the relied-on behavior
through NET-HTTP-1 to -5 (§14): the lockfile, the build and the
equivalence test for `idna_adapter` are all stage-2 proof, not claims of
this note.

## 9. Simulated HTTP peers

**[repo evidence]** Companion 17 and 03 §9.2/§9.3 already specify
`sim.world.http(addr, peer)` with `peer: impl Fn(&Request) -> Reply + Send
+ 'static`, `req.url()`, `Reply::ok(body)`, `.after(d)`,
`Reply::status(503)`, `Reply::hang()`, network time (one round trip,
2–20 ms, plus `after(d)`), `Refused` for an unserved port after one round
trip, `Dns` at once for an unnamed host, and packet loss as delay.

**[inference]** In v0:

- **Setup.** `world.http(addr, peer)` parses `addr` with NET-HOST's entry
  rule; an invalid, duplicate or TCP-listener-held address panics at setup.
  HTTP peers reserve their address: a later simulated TCP bind there is
  `AddrInUse`, including wildcard/loopback aliases of that listener address.
  A simulated `get` reaches only `world.http` peers, never a TCP listener.
  Setup takes no scheduling point, time or draw, and records one `world`
  trace event with the address, as `world.file` does. A peer serves both `http` and `https`
  URLs to its port; the simulator models no TLS bytes.
- **Limit.** An HTTP peer models an application response, not TLS
  certificate validation or wire parsing. Simulated HTTPS success asserts
  a successful authenticated transport; it does not prove one.
- **Replies.** `Reply::ok(body)`, `Reply::status(code)` (a final status:
  a code outside 200–599 panics, so no simulated peer sends an interim or
  `101` response, which the real backend skips or refuses (§3 step 9); a
  `2xx` code returns an empty body), `Reply::hang()`
  (never replies), and two new ones: `Reply::reset()` (`Reset`) and
  `Reply::tls_error()` (`Io(InvalidData)`, standing for a failed handshake
  or certificate). `.after(d)` delays any reply. Every `Reply`, `Reply::ok`
  included, obeys §3's 64 MiB body limit: larger is `Io(InvalidData)`.
  `Request::url() -> &Url` exposes the request URL with its fragment
  removed (provisional; there are no request headers to show).
- **Timeline of one `get`.** Steps 1 to 4 of §3 run as on the real backend,
  taking no time or draw. A domain no peer or node names returns `Dns` at
  once; an IP host needs no lookup. Otherwise the request takes one packet
  delay, drawn from the net sub-stream as TCP packets are (1–10 ms); the
  simulator then calls the registered peer, if any, on the requesting
  task's thread; the reply arrives after `after(d)` and a second packet
  delay. An address no HTTP peer serves returns `Refused`
  after one round trip, IP hosts included, even if a TCP listener is there;
  no peer callback runs in that case. The requesting task waits cancellably
  for arrival with `deadline`: `Timeout` exactly at the deadline if it is earlier,
  `Cancelled` on scope cancellation. A peer that panics panics the
  requesting task, like any code that task runs.
- **Loss and other faults.** `sim.faults.net_drop(p)` and the rest of
  companion 17's fault table stay planned; when built, loss adds
  retransmission delay to these two packets as 17 says. Stage 2 does not
  need them: `after`, `hang`, `status`, `reset` and `tls_error` cover the
  outcomes the v0 examples test. `sim.net.drop_all()` (The Feel in 01)
  also stays planned.
- **Attenuation.** `only` applies under simulation exactly as on the real
  backend (§4), before any peer is consulted.
- **Determinism.** A run that never calls `get` makes exactly today's
  draws. Each completed or pre-cancelled `get` records one `net.get` trace
  event with task, span, scheme, the checked `host:port` key, delays and
  the result (variant, status code, body length), never the path, query or
  body, which are payload (03 §9.3's rule for TCP).

**Replay identities.** Under REPLAY-1/REPLAY-2 (research/32), a run that
registers a peer records inline the HTTP profile identity (`net-http-v0`),
each peer address, and the seed of the net sub-stream; peer closures are
source code, which the source identity already covers. Replay refuses a
manifest whose profile or peer set differs. A seed that uses `get` replays
byte for byte.

## 10. Trust diff

**[inference]** No new authority kind: an item whose signature reaches
`Net` declares `net`, and exercised kinds stay v1 (03 §4.1). `get` and
`connect` share `only`, and a raw `connect` to the same `host:port` can do
everything `get` does, so splitting `net` into `net.http` and `net.tcp`
would show a difference that bounds nothing. The `reaches:` line already
prints the `only` list from `main` (03 §4.3); for `net` its meaning is
fixed here: the entries bound outbound peer operations under NET-HOST,
not the data sent. A displayed `only` without `no_bind` retains arbitrary
bind-name resolution, even if binding fails, and communication with
unlisted inbound peers. `only([])` alone is not network silence;
`only([..]).no_bind()` is the outbound-only destination grant. This is
the v0 trust-line interpretation, not v1 exercised reporting.
An unattenuated `root.net`
reaching a changed path prints as `root.net`, which a reviewer reads as
every host.

## 11. Admission filter

**[inference]** GOVERNANCE.md's questions:

- *Safer change?* Yes. The honest form of "fetch a URL" becomes a declared,
  reviewable `net` authority with an exact destination rule and a
  deterministic, replayable test peer, where today it cannot be written
  without hand-rolling HTTP over `TcpStream` and has no TLS at all.
- *Overlapping ways?* One way to make an HTTP request; `connect` stays the
  raw byte path. No second URL type, no options object, no client object.
- *Local reasoning?* Preserved: a function without `Net` cannot reach the
  network except through an authorized handle, and `only` in `main` lists
  outbound peer destinations; its bind and accepted-connection exceptions
  (§4) and the limits of §6 are stated where `only` is documented.
- *Fast feedback?* Simulated tests make no host connections and run in
  virtual time (companion 15.2 asserts `sim.elapsed() <= 2s` for a 30 s
  peer).
- *One obvious shape?* `net.get(&url, deadline)`, already in every example.
- *Library instead of core?* Yes: std and runtime only.

Verdict: admit. **The design is sound** in the sense A1 asks for: every
route by which a `get` could reach a host outside `only` (alternate host
spellings, percent-encoded and internationalized hosts, parser
differences, redirects, proxies and other environment input, shared
connections or sessions, resolution of refused names) is closed by a rule
above. What remains are stated limits that no host allowlist can bound:
where an allowed name's DNS points (contact and handshake traffic reach
any resolved address on the granted port, §6), and what an allowed
destination may do with the data. Separately, `only` leaves bind and
accepted connections unrestricted: arbitrary bind-name DNS (even on bind
failure) and unlisted inbound peers remain possible. Only with `no_bind`
is it an outbound-only destination grant; an empty list alone is not
network silence. It stays sound only with these exceptions and §6's
limits written where `only` is taught, and with the stage-2 guard below.

Stage-2 guard (as PROC-3 for `Proc`): `tide_std::net` exposes no public
path to the network except `Net`'s methods and the handles they return. A
test lists its public items, `pub use` re-exports included, and fails on
additions, so a re-export of `url::Url` (`socket_addrs`), a `rustls` or
`httparse` type, or a resolver function cannot appear. Every new type that
can hold a connection goes into `NOT_PURE`; `Url`, `UrlError` and
`NetError` are `Pure`.

## 12. Cost

**[inference]** Readers and agents: §6.7 of `reference.md` grows and trims
elsewhere pay for it: compact capability-receiver and simulation summaries,
with simulated `get` timing in companion 17, which already states it. Measured
with `scripts/count-tokens.sh`, the proposal takes 24,985 tokens to 24,990,
inside D14's 25,000 with room for `Proc`'s stage 2. Companion 21 and 03
§1.1/§9 text is served on demand. Implementation: an
HTTP/1.1 client of a few hundred lines over the existing TCP path, a TLS
module, the resolver pool, the `Url` newtype, a simulated peer table on the
existing scheduler, manifest fields; four capabilities in six directly
pinned crates (two named only to select an implementation, §8), one with
C sources. Entropy: none at the language level. The divergences an agent
can trip on, against Rust's popular clients (`reqwest`, `ureq`): no
redirect following, no proxy variables, no custom headers, compiled-in
roots, ASCII-only URLs with no percent-encoded hosts. Each fails loudly (`Status(3xx)`, `Dns` or
`Refused` behind a proxy-only network, `Io(InvalidData)` for a private CA,
`UrlError`), never silently differently.

## 13. Alternatives rejected

- **Keep HTTP out of v0**: v0 then cannot express the honest version of
  code that fetches, and every v0 example built on `get` stays unbuilt.
- **A full client in v0** (headers, methods, bodies, redirects, cookies,
  proxies, HTTP/2): each is a policy the red team must attack and none has
  a current caller; `connect` already offers the raw path.
- **`only` with wildcards, port-less entries or address filtering**
  (Deno-style): wider grants than the examples use, and address filtering
  breaks internal names (§6).
- **Re-checking resolved addresses against address entries** (Deno's
  CVE-2026-49859 fix): needed for deny rules, where a name can resolve to
  a denied address; Tide has no deny rules, and a name grant means the
  name's owner.
- **Platform TLS or platform roots**: environment-steerable trust,
  machine-dependent results (§8).

## 14. Stage 2

Stage 2 implements §§2–9 and the guard of §11, adds `conformance/rules.md`
rows NET-HTTP-1 to -5 with their cases, and drops the `planned` labels of
`get` from `reference.md` and companion 21 when the code lands. Its done
condition is the bead's Acceptance:

- companion 15.2 builds through tide-admit and its `#[sim_test]` passes;
- a real-backend test in `tide-std` fetches over HTTPS from a local test
  server, whose certificate chains to a test root that a crate-private
  constructor adds (no public root override exists);
- `NetError` classification matches §5 (companion 21) on both backends;
- the dependencies of §8 are pinned, with this note as their record, the
  committed lockfile lists the transitive releases, the nix flake and
  Docker builds compile `ring`, and the `idna_adapter` equivalence test of
  §8 decides between 1.0.0 and 1.1.0.

Cases stage 2 adds beyond the examples, each on the backend that can show
it:

- NET-HTTP-1: the refused hosts of §4 (`b%C3%BCcher.example`,
  `%78n--bcher-kva.example`, `%EF%BD%81pi.example.com`, `a..b`), as `Url`s
  (`UrlError`), `only` entries (granting nothing) and `connect` addresses
  (`InvalidInput`); a `%` in a path or query still parses and is sent as
  written; the span of §4's *Host as written* matches the `url` crate's
  host on every case.
- NET-HTTP-2, real backend against the local test server: a `204` with the
  connection held open returns `Ok` with an empty body before the
  deadline; a `304` with `Content-Length: 10` and then a close returns
  `Status(304)`; a `100` and then a `200` returns the `200`'s body; a lone
  `100` and then a close is `Reset`; a `101` is `Io(InvalidData)`.
  An unsupported transfer coding is `Io(InvalidData)`. A local TLS server
  that sends a close-delimited 2xx body then drops TCP without
  `close_notify` yields `Reset`; the same body ending with `close_notify`
  succeeds. A complete Content-Length or chunked 2xx body stays `Ok` if
  TCP closes without `close_notify`. Mixed failed connection attempts
  return the first non-refusal error, and all-refused attempts give `Refused`.
  A server certificate that chains to no configured root fails with
  `Io(InvalidData)` before any HTTP data is sent. Environment-invariance
  includes `SSLKEYLOGFILE`: no key-log file is created or appended and
  results stay unchanged, alongside the proxy and `SSL_CERT_*` cases.
- NET-HTTP-3: `Reply::status(100)`, `(101)` and `(600)` panic at the
  peer; `Reply::status(204)` returns `Ok` with an empty body. A request
  ending in `#secret` gives the peer a URL without a fragment.
  `Reply::ok` of exactly 64 MiB succeeds; 64 MiB + 1 byte is
  `Io(InvalidData)`, matching the real response reader's boundary cases.
  An IP with no HTTP peer is `Refused` after one round trip, with a TCP
  listener too (no accept is triggered); an unknown domain is `Dns` at once.
  Registering an HTTP peer on a simulated listener's address panics at
  setup; binding after peer registration gives `AddrInUse`.
- NET-HTTP-5: on the real backend, observe the resolver while
  `only([]).no_bind().bind(("unlisted.test", 8080))` returns
  `PermissionDenied` before any resolution; observe zero resolver calls,
  rather than inferring this from a simulated bind failure.

The red team attacks §§3–7 and §9; stage 2 adds what it finds as cases.
