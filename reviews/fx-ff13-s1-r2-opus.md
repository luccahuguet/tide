Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate b772256 (design, docs)

# fx-ff13 stage 1, round s1-r2 (Opus review)

## Verdict

**Changes required.** The design is sound, and its admission case holds.
Three findings block, and each has a small text fix. B1 is a TLS truncation
rule that stage 2 would otherwise have to invent. B2 is a resolver claim that
is false for `bind`. B3 is that the trust-diff owner never records what a
`net` `reaches:` line means. Each fix adds or corrects a rule, so it is not
text-only, and it needs my re-check of that fix alone. No full rerun is
needed: none of the fixes crosses a boundary that the earlier passes did not
already examine.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-ff13`, branch
  `fx-ff13`, HEAD `b772256a4454451d71a88537c345a99fd8a13ff3`, base main
  `d58f3e4`.
- `main` is now at `505922f6297b686aa0d9243b42bf310f5b20789e`. Since
  `d58f3e4` it has three ledger commits, which touch only
  `.beads/issues.jsonl`, so they don't affect this review.
- SHA-256 of `git diff d58f3e4..HEAD`:
  `ab79e174c850353b0b990058d87aa09d10aea9f4dd7ae10f8db9dd2a587f99dc`.
- Reference proposal `target/work/reference-proposal.diff`, SHA-256
  `98f018d29c6767c5da319a8d64583298604a94019c829d0cce331b9ae1d64e3e`.
- Files read in full or in their changed regions:
  - `research/42-net-get-v0.md`;
  - the diff to `design/01-design.md`, `design/03-authority-concurrency.md`
    (§1.1 and its context, §4.3, §9.2/§9.3, §12.2),
    `design/reference-companion.md` (15.2, 17 and 21) and
    `conformance/rules.md`;
  - the proposal, applied;
  - `crates/tide-std/src/net.rs` (`Address::key`, `resolve`, `bind`, `only`);
  - GOVERNANCE.md (the Merge Rule and the admission filter);
  - fx-ff13 with its comments, and fx-6d50.
- Not repeated: the Codex pass s1-r1 and its re-check, and the Codex red team
  and its re-check.
- Budget: I applied the proposal to a scratch copy
  (`target/work/s1r2-opus/reference.md`; `patch -p2`, clean).
  `scripts/count-tokens.sh` measures it at **24,989**, within the 24,990 cap.
  The current `design/reference.md` measures 24,985.

## Blocking findings

### B1. A close-delimited `https` body has no `close_notify` rule

`research/42-net-get-v0.md:190-206` (§3 step 9.4 and step 10) and
`design/03-authority-concurrency.md:269-275`.

- **The gap.** When a response has neither `chunked` nor `Content-Length`,
  "the body runs to connection close". Under TLS, a TCP FIN or RST without the
  peer's `close_notify` is not an authenticated end of stream.
- **Why it matters.** An on-path attacker can cut the body at any byte. The
  current text then returns `Ok(truncated)` from an `https` `get`, which the
  docs present as authenticated to the name. rustls reports this case as an
  `UnexpectedEof` read error. Nothing in §3, NET-ERR (§5) or 03 says whether
  that error is `Reset`, `Io(UnexpectedEof)` or a normal end. NET-ERR's
  `Reset` row covers "a close before the … framed body was complete", and a
  close-delimited body is not framed.
- **Consequence.** Stage 2 would have to invent this integrity policy itself.
  The red team did not raise it.
- **Minimum fix.** Add one sentence to research/42 §3 step 9.4 or step 10,
  and the same clause to 03's HTTP note: "Under `https`, a body delimited by
  connection close ends only at the peer's TLS `close_notify`; a close without
  it is `Reset`." Then add a NET-HTTP-2 stage-2 case: a local TLS server sends
  a close-delimited body and drops TCP without `close_notify`, and the result
  is `Reset`, not `Ok`. A framed body that is complete before a bare close
  stays `Ok`. That is the usual client rule, and it keeps servers that skip
  `close_notify` working.

### B2. "Retires the cannot-be-abandoned limit" is false while `bind` keeps the inline resolver

`design/03-authority-concurrency.md:215-220` and `:287-288`;
`research/42-net-get-v0.md:472-473`.

- **The claim.** All three places say stage 2 moves **`connect`** onto
  NET-RESOLVE, "which retires the 'cannot be abandoned' limit … one resolver,
  one owner".
- **Why it is false.** Today `bind` and `connect` share one path:
  `Address::resolve` (`net.rs:95-112`) is called from both `bind`
  (`net.rs:164`) and `connect` (`net.rs:198`). If only `connect` moves, a
  real `bind(("name", p))` still resolves inline and still cannot be
  abandoned. The limit is not retired, and there are two resolvers, not one.
- **Effect on NET-HTTP-5.** Its "zero resolver calls observed" for `bind`
  also needs `bind`'s resolution to go through an observable, owned path.
- **Minimum fix.** Say "`bind` and `connect` move onto it", since they share
  `Address::resolve`, in all three places. Alternatively, keep the limit
  stated for `bind` and drop "one resolver, one owner". I recommend the
  first: it is the smaller code shape, and it gives NET-HTTP-5 one resolver
  to observe.

### B3. The trust-diff owner doesn't record what a `net` `reaches:` line means

`design/03-authority-concurrency.md:1334-1349` (§4.3), against
`research/42-net-get-v0.md:687-701` (§10).

- **Where the decision sits.** research/42 §10 decides the v0 meaning of a
  `net` `reaches:` line: the entries bound outbound peer operations, not
  data, and without `no_bind` the line keeps bind-name resolution and
  unlisted inbound peers. 03 §4.3 owns the trust diff and says none of this.
- **Why it misleads.** §4.3's own example is
  `reaches: main: root.net.only(["db.internal:5432"])`, which has no
  `no_bind`. Its next paragraph warns explicitly about `proc` lines ("only
  names programs, not a sandbox") but has no matching warning for `net`.
  Reviewers meet `only` in the trust diff, so it reads as a complete network
  boundary. That is the misreading the red team's F1 blocked on. F1's fix
  reached 01, 03 §1.1, the companion and the reference, but not this section.
- **Minimum fix.** Add one sentence after the `proc` sentence in 03 §4.3: "A
  `net` line's `only` list names outbound destinations under NET-HOST (§1.1),
  not data; without `no_bind` the `Net` can still resolve bind names and
  accept unlisted peers." Whether the line itself also prints a marker stays
  a stage-2 display choice. The sentence is enough for v0.

## Non-blocking findings

1. **Simulated IP hosts, and how `get` relates to the simulated TCP loopback,
   are left undecided.**
   - The text: research/42 §9 (`:654-662`), companion 17 (`:455-456`) and
     03 §9.3 say "a host that no peer or node names gives `Dns` at once".
   - The divergence: a `get` to an IP literal with no peer then gives `Dns`
     in simulation, where the real backend never gives `Dns` for an IP.
   - The open case: nothing says whether a simulated `get` to
     `127.0.0.1:p` reaches a simulated `TcpListener` bound there, or whether
     `world.http("127.0.0.1:p")` and a simulated bind on the same address
     clash.
   - Suggestion: a simulated `get` reaches only `world.http` peers. An IP
     host with no peer is `Refused` after one round trip. A peer address
     that a simulated listener also holds panics at setup.
2. **The default port in the key isn't taught where readers look.**
   research/42 §3 step 3 keys a URL by `port_or_known_default` (80 and 443).
   Neither 03 NET-HOST nor companion 21 says so. A reader of companion 21
   can work it out from "exact `host:port`", but the clause "a URL without a
   port keys as 80 for `http` and 443 for `https`" would make
   `only(["example.com:443"])` refusing `http://example.com/` obvious.
3. **The form of `Host` and SNI is unstated.** "The host that was checked is
   the one … sent" (03 NET-HOST, research/42 §4) leaves open whether `Host`
   and SNI carry the key's form (lowercased, trailing dot dropped) or the
   form as written. The request template in §3 step 8 doesn't show brackets
   for an IPv6 `Host`. Name it: the key's host, with IPv6 in brackets in
   `Host`, and no SNI for an IP host.
4. **ring's maintenance isn't recorded (AP-DEPENDENCY-001).** The rustls row
   records its maintenance. The `ring` row and pin (`=0.17.14`, released
   2025-03-11, the newest on 2026-11-18) say nothing of ring's upkeep. ring
   has had no release in about 20 months, and its stewardship changed in
   2025. Record that status, and that the provider swap stays one
   `CryptoProvider` value (already in §8's replacement cost). This doesn't
   change the choice.
5. **Error mapping when one connection gets mixed failures.** §3 step 6 says
   "`Refused` when every address refuses; another connect error is `Io`",
   but not which error a mix of refusals and other errors returns. Say "the
   last address's error", or "`Io` of the first non-refusal error".
6. **Reference §6.8, as proposed, hides where HTTP peers live.** It lists
   "HTTP peers" under planned `sim.faults` and `sim.node`, but peers come
   from `sim.world.http`. The current text has the same grouping, and
   companion 17's table is correct. Fix it when tokens allow.
7. **"planned peers/resolve" in reference §6.7 is terse.** "planned UDP
   peers/resolve" would tell a reader which peers are meant, if the budget
   allows.
8. **The NET-HTTP-2 row is missing one case.** It omits the unsupported
   transfer coding (`Io(InvalidData)`) that research/42 §3 step 9.4 adds.
   Add it there, or to stage 2's case list.

## Reasoning

1. **Admission filter.**
   - **A smallest surface.** `get` already appears in the reference,
     companion 15.2/17/21, 03 §12.2 and The Feel. This change fixes its wire
     behavior and adds no syntax, checker rule or authority kind. The
     additions are each justified by a current caller or an existing test:
     - the four `Url` accessors (`scheme`, `port`, `query`, `fragment`);
     - `Reply::reset` and `Reply::tls_error`, which let a simulated test
       reach the `NetError` variants companion 21 already lists;
     - `#[non_exhaustive]`;
     - `Encode`/`Decode`, which The Feel's `link: Url` needs.
   - **Dependencies.** The rejections hold up:
     - redirects, proxies and HTTP/2 are policies no current caller needs;
     - `ureq` and `ureq-proto` lose on an unstable API and on proxy and
       redirect defaults;
     - `url::Url`'s `socket_addrs` would be ambient DNS, which settles the
       newtype question.
     The httparse-versus-`ureq-proto` call is close and argued on the whole
     lifecycle. rustls with ring and webpki-roots is the standard,
     environment-independent choice. The `idna_adapter` pin plan is sound.
   - **Cost.** The cost to readers is stated honestly: about +4 tokens net
     in the reference, which I measured. §12 names the ways `get` differs
     from Rust's popular HTTP clients.
2. **Ownership and consistency.**
   - 01 records the decision and its limits. 03 §1.1 owns NET-HOST, and the
     companion, the rules rows and the proposal summarize it without
     contradicting it. The `connect` change is labelled as stage 2 (planned)
     throughout.
   - The rule is defined once for `connect`, `get` and fx-6d50's `resolve`.
     fx-6d50's constraints, a shared key, a shared resolver and the same
     simulated names, are concrete.
   - The design agrees with research/40's Proc stance: an allowed relay is
     full authority, as an allowed shell is.
   - Nothing claims shipped behavior. Every new row and section is marked
     planned, and the claim that `no_bind` refuses before resolution is true
     today (`net.rs:161-164`).
   - The remaining gaps are B2 (the resolver claim) and B3 (trust-diff
     ownership).
3. **Learnability.** The reference with the proposal applied, plus companion
   21, is enough to write a correct `get`: parse with `Url::parse`, build
   the deadline as `clock.now() + d`, match `NetError::Timeout` and the
   rest, and treat `Status(3xx)` as final. The 15.2 example shows the
   simulated test. A reader can tell that `only` checks exact keys before
   lookup, that it leaves `bind`, its name resolution and inbound peers
   open, that `only([])` is not silence, and that `no_bind` makes the grant
   outbound-only. They can also tell that DNS for an allowed name is
   trusted, that rebinding reaches TCP and the TLS handshake, and that path
   and query data are not bounded. Non-blocking 2 and 3 would make the key's
   port and form explicit.
4. **Stage-2 readiness.**
   - NET-HTTP-1 to 5 and research/42 §14 are concrete: named hosts, status
     sequences, size boundaries, environment variables, and the observation
     NET-HTTP-5 makes.
   - What stage 2 would still have to invent: the TLS truncation rule (B1),
     which resolver `bind` uses (B2), and smaller simulation choices
     (non-blocking 1, 3 and 5).
5. **Budget.** 24,989, at most 24,990: pass.

DONE
