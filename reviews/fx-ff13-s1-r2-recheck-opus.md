Model: Claude Opus 5.5 · Date: 2026-11-18 · Re-check 4d620d4

# fx-ff13 stage 1, s1-r2 re-check (Opus, finder of B1–B3)

## Verdict

**Approve.** B1, B2 and B3 are fixed. N1's simulation rule, as tide-lead
decided it, is consistent across every place that states it. None of the
other text-only fixes is plainly wrong. The reference proposal measures
24,990, exactly at the cap.

## Scope

- Candidate: `4d620d42345c173fb949c9455829c82833a4de6f`, which amends
  `b772256`. Base: `d58f3e4`.
- SHA-256 of `git diff d58f3e4..4d620d4`:
  `ea211c030136527e384766bca3bd9d7f79ee3e7b0b953c909bc5d10165cbd39b`.
- SHA-256 of the round diff's reference proposal
  (`target/work/reference-proposal.diff`):
  `3994c5b1cd781fe9d3ffdd48213a7c4b01e5ec1834ffb0e0b048f32bd1e324c7`.
- Read: `git diff b772256..4d620d4` (as a word diff, all files), the
  "Round s1-r2 fixes" section of `target/work/report.md`, and the updated
  proposal.

## Items

- **B1 (TLS truncation): fixed.**
  - research/42 §3 step 9.4 now says a close-delimited `https` body ends
    only at `close_notify`. A bare close, rustls's `UnexpectedEof` included,
    is `Reset`, while a complete framed 2xx body stays `Ok` on a bare close.
  - The same rule is in 03's HTTP note and NET-ERR's `Reset` row, and §5
    separates truncation from handshake failure.
  - Companion 21 states it, NET-HTTP-2's row has it, and §14 has cases both
    with and without `close_notify`.
  - The rule is correct and as strict as it should be.
- **B2 (resolver ownership): fixed.**
  - 03 §1.1's TCP limit, 03's HTTP resolver note and research/42 §7 now all
    move both `bind` and `connect` onto NET-RESOLVE, since they share
    `Address::resolve`.
  - The "one resolver, one owner" and "retires the limit" claims are now
    true. The early `no_bind` refusal is kept.
  - A call with no deadline waits cancellably, and no public parameter is
    added.
  - NET-HTTP-5 now has a single owned path to observe.
- **B3 (trust-diff meaning): fixed.** 03 §4.3 (`:1355-1357`) now says that a
  `net` line's `only` names outbound destinations under NET-HOST, not data,
  and that without `no_bind` the `Net` can still resolve bind names and
  accept unlisted peers. The sentence follows the `proc` sentence and
  matches research/42 §10.
- **N1 (simulation rule, as decided): consistent.**
  - research/42 §§5, 7, 9 and 14, 03 §9.3, companion 17 and NET-HTTP-3 all
    agree on these rules:
    - a simulated `get` reaches only `world.http` peers, never TCP
      listeners;
    - an IP host with no HTTP peer is `Refused` after one round trip, even
      when a listener is there, and no accept is triggered;
    - an unknown domain is `Dns` at once;
    - registering a peer on an address a listener holds panics, and a
      later bind on a peer's address is `AddrInUse`, aliases included.
  - The reservation is symmetric, so neither registration order lets
    `get` and TCP see each other.
  - The IP-host case now matches the real backend, which never gives `Dns`
    for an IP.
- **Other text-only fixes: nothing plainly wrong.**
  - N2: the 80 and 443 key defaults are in 03, companion 21, research/42 §4
    and NET-HTTP-1.
  - N3: `Host` carries the key's host with IPv6 in brackets, and SNI is
    sent for DNS names only.
  - N4: ring's maintenance record, with its sources and the choice kept.
  - N5: mixed connect failures return the first error that isn't a
    refusal.
  - N6 and N7: the proposal's §6.8 wording, and "planned UDP
    peers/resolve" in §6.7.
  - N8: the unsupported transfer coding is in NET-HTTP-2.
- **Budget: pass.** I applied the proposal to a scratch copy
  (`target/work/s1r2-recheck/reference.md`; the patch applied cleanly).
  `scripts/count-tokens.sh` measures **24,990**, which meets the cap of at
  most 24,990 but leaves no headroom.

## Non-blocking note (no re-check needed)

`localhost` behaves differently on the two backends. On the real backend,
`localhost` skips lookup (§3 step 5). Under N1's rule it is a domain key,
so a simulated `get` to `http://localhost:p/` with no peer is `Dns` at
once, where the real backend gives `Refused`. A peer at `localhost:p` and a
simulated listener at `127.0.0.1:p` also don't collide, because their keys
differ. Stage 2 can either treat `localhost` like an IP host for the
`Refused` and collision rules or state the difference. A test peer
reachable by an alias is not a security issue, so this doesn't block.

DONE
