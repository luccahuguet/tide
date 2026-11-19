Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 25954b7 (re-check)

## Scope and identity

Finder re-check of **only B1, B2 and the two non-blocking items** in
`target/work/s1-r1-codex.md`, read first, against the dispositions in
`target/work/report.md` under “Round s1-r1 fixes” and
`git diff 50622b4 25954b7`. This is not another full review.

HEAD: `25954b7027829adb8e480d46e96571cdb1225596`; base: `28f7f6d`.
SHA-256 of `git diff 28f7f6d 25954b7`:
`516c146091e2b855fbc1b365a4f5a84acd39bc7ccf56d1f3a2e5693240364b67`.
The fix changes only the five documentation files named in the dispositions.
The landed TCP files and research/42 are unchanged from base.

## B1 — Resolved

The minimum correction is present everywhere originally listed:

- `research/48-net-resolve-v0.md:88–97` limits list identity to **real
  non-literal lookups given the same resolver answer**, preserving order,
  duplicates and address representations with ports removed. It separately
  states canonical literal results, the unbracketed mapped-IPv6 distinction
  from TCP, and synthetic simulated names. Lines 123–142 carry those
  distinctions into the execution order and lookup-input explanation.
- `design/01-design.md:520–530`,
  `design/03-authority-concurrency.md:259–281`, and
  `design/reference-companion.md:767–784` carry the qualified real-lookup
  promise, mapped-literal distinction and separate simulation contract.
- Research/48's former “literals only” identity claim is removed from §6;
  lines 292–295 instead describe unchanged loopback-only TCP.
- `research/48-net-resolve-v0.md:423–429` and
  `conformance/rules.md:104` explicitly require stage-2 coverage of
  unbracketed `::ffff:127.0.0.1` returning IPv4. Research/48:447–454 and
  rules.md:106 constrain NET-RESOLVE-3 to real non-literal lookup results
  with the same supplied answer, preserving order and duplicates.

This matches the landed distinction: `net.rs:87–92` preserves the parsed
IPv6 tuple value, `:99–101` canonicalizes only its authority key, and `:118`
returns the socket unchanged. `net/real.rs:23–34` uses that socket's address
family. `net/host.rs:23–28,46–50` supplies canonical literal/localhost
results. The non-literal path still uses the shared resolver
(`net.rs:119–122`, `net/resolve.rs:73–74,88–94`). No TCP change is needed.

## B2 — Resolved

`research/48-net-resolve-v0.md:288–295` now says address registrations
support HTTP `get` and explicitly says simulated TCP cannot test resolving
a domain followed by connecting to its synthetic address. A name-only
grant still denies the address spelling. The companion's no-aliasing
summary (`design/reference-companion.md:467–471`) carries the HTTP/TCP
distinction; 03:2227–2228 does too.

The corrected fixture advice matches landed code: HTTP uses its peer table
(`net/sim.rs:590–599`), while TCP requires IPv4 loopback and a TCP listener
(`:195–198`). Bind accepts only IPv4 loopback or wildcard (`:143–146`).
Registering an HTTP peer cannot route TCP to a synthetic address. The fix
preserves this limit rather than expanding landed TCP.

## Non-blocking 1 — Resolved

The simulation summaries explicitly exempt `localhost` and give it
`127.0.0.1` with or without a peer:
`conformance/rules.md:105`, `design/03-authority-concurrency.md:2222–2229`,
and `design/reference-companion.md:462–471`.
`research/48-net-resolve-v0.md:439–446` retains the stage-2 cases with and
without a peer. Research/48:270–276 replaces the faulty
“Ok exactly when get would not be Dns” equivalence with a name-existence
rule after cancellation, shape, authority and deadline checks.

This agrees with `Host::literal` (`net/host.rs:23–28`) and the simulated
HTTP DNS gate (`net/sim.rs:587–591`); later HTTP failures are no longer
mistaken for a successful resolve result.

## Non-blocking 2 — Resolved

`research/48-net-resolve-v0.md:317–320,330–337` acknowledges that a separate
switch could restrict access to the full address list without contacting
the peer. Its rejection now rests on existing paths retaining the same
query names/forms, not on the false claim that the switch bounds nothing.
No additional attenuation is required or introduced.

## Compatibility and checks

Within these fixes, no new contradiction with research/42's NET-HOST,
shared-resolver, authority or simulated HTTP/TCP contracts was found
(§§4, 6, 7 and 9). The corrections distinguish lookup answers from literal
socket representation and HTTP peer fixtures from TCP listeners, without
changing those owners or the existing resolver-configuration limits.

Checks run in `/mnt/build/eon-agents/worktrees/tide-fx-6d50` on the candidate
above, 2026-11-18:

- `scripts/count-tokens.sh design/reference.md`: **24,978**, exit 0;
  **22 tokens below the 25,000 cap**.
- `git diff --check 50622b4 25954b7` and
  `git diff --check 28f7f6d 25954b7`: exit 0.

## Verdict

**approve** — all four original items are resolved by their requested
minimum fixes. No fix-introduced blocking problem found. Confidence: high
for this documentation/source-consistency re-check. No override.

Only this report was written; no build, runtime probe, implementation edit,
Beads write or commit. Stage-2 implementation/runtime behavior and the
subsequent Opus review remain unproved by this pass.
DONE
