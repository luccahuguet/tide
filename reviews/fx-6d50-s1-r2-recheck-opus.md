Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 9f78b8d (re-check)

## Scope and identity

This is the finder re-check of my own s1-r2 review (target/work/s1-r2-opus.md). It covers only B1 and the R1/R2 texts with my N1 edits. I glanced at N2–N8 only for contradictions that the fix itself introduced. It is not a full review. The author's dispositions are in target/work/report.md, under "Round s1-r2 fixes".

- HEAD is `9f78b8d`, base `28f7f6d`. SHA-256 of `git diff 28f7f6d..HEAD` is `4c9770bccfa0b56c62948c6a39cf8f7a5a38f42d80b87c732c9669e64530f203`, which matches the brief.
- `main` is now `06d8e3d`. It differs from the base only in `.beads/issues.jsonl`.
- The fix diff (`git diff 25954b7 9f78b8d`) touches five files and leaves `design/reference.md` unchanged, so it stays at 24,978 tokens.
- `git diff --check 28f7f6d..HEAD` is clean, and the tree is clean.
- I made no edits apart from this file.

## B1 (synthetic ordinal rule): resolved

**The rule is the same in all four places:**
- research/48 §6 (Addresses): "Ordinals count distinct host keys of domain peers after NET-HOST canonicalization, starting at 1 in setup order. `localhost` and IP-address peers take no ordinal."
- 03 §9.3 (lines 2224–2227): the same rule, in the same terms.
- companion 17: the same rule, in the same terms.
- NET-RESOLVE-2 (rules.md:105): the same rule, plus the case from my review. It registers `localhost:8080` first, then an IP-address peer, then `a.test:443`, and expects `a.test` to resolve to `198.18.0.1`. research/48 §11 repeats that case.

**It matches the landed keying:**
- `World::http` (`sim.rs:169–171`) calls `Net::world_http`, which keys `addr` with `host::entry` (`net/sim.rs:532`).
- `host::entry` gives `Key { host: host::host(written), port }`. `host::host` lowercases and IDNA-maps through `url::Host::parse` and drops one trailing dot (`net/host.rs:55–58, 83`). So `HIDDEN.test.` and `hidden.test` share one key host, which is what "after NET-HOST canonicalization" means.
- `localhost` keys as `Host::Name("localhost")`, which the rule now excludes by name. IP peers key as `Host::Ip`, which it excludes as "IP-address peers".
- Duplicate keys (same host and port) panic at registration, so the R1 witness (`hidden.test:80` plus `HIDDEN.test.:443`) is two valid registrations on one host key. That matches "share one ordinal".
- Peers are cleared only when `SimRoot` drops (`HttpPeers`), so "an address, once assigned, never changes during a run" holds.

## R1 text (with the N1 edit): resolved

- research/48 §6 and companion 17 both say: "Resolving a granted name reveals its ordinal, and so the number of earlier distinct peer names, including names outside that Net's grants (`localhost` and IP peers excepted)". This is my N1 wording with B1's exceptions.
- research/48 §6 adds "Changing the setup order can change the address."
- 03 §9.3 carries a one-sentence version that points to research/48 §6.
- The red team's "identical recorded setup replays identically" survives in research/48 §6's existing *Replay identity* bullet, so leaving it out here loses nothing.
- research/48 §11 names R1's two-setup witness as a stage-2 case, with concrete values (`198.18.0.1` and `198.18.0.2`).

## R2 text (with the N1 edit): resolved

- research/48 §6 and companion 17 state the limit for both simulated `get` and `resolve`: success "proves neither DNS encodability nor real resolver acceptance. Real resolver failures are `Dns`."
- research/48 §6 also says NET-HOST leaves DNS label and total-name lengths to the resolver. That is accurate: `host::host` adds no length check.
- `reference.md` was not touched.
- research/48 §11 names R2's witness: a 64-octet label resolves in simulation, while the real backend gets a seam-supplied failure and returns `Dns`. It sends no live query.

## N2–N8: nothing the fix introduced contradicts the landed code or research/42

- **N2:** research/48 §2 now says it scopes research/42 §7 item 2, and the intro points there. This matches `net.rs:87–92, 118`.
- **N3:** companion 21 now checks the deadline before returning a literal, which matches 03 §1.1's order. NET-RESOLVE-1 includes `resolve("127.0.0.1", past)` returning `Timeout`.
- **N4/N5:** the stage-2 plan uses a `#[cfg(test)]` seam next to `CALLS` in `net/resolve.rs` and sends no live query, as tide-lead decided. This fits the current file, which has no seam yet and a `CALLS` thread-local under `#[cfg(test)]`. The incorrect pool-test analogy is gone.
- **N6:** the chained-`only` case (`Denied`, because the full keys intersect to an empty set) matches `net.rs:167–176`.
- **N7:** deferred to stage 2 as decided. The fix took "resolve" out of the exact-key lists in 01 and in companion 21's "What `only` bounds". That removes my N7 concern there, and 01 now says plainly "An `only` entry on any port permits resolution of its host". research/42 §4's NET-HOST decision still names resolve as an outbound peer operation, and that is not contradicted: the any-port host match is research/42 §7 item 1's own rule.
- **N8:** 01 is trimmed to the decision with a pointer to 03 §1.1, and it is dated "admitted 2026-11-19 under A1 (2026-11-18)".

One non-blocking note on a clause the fix introduced:
- rules.md:104 (NET-RESOLVE-1) still begins "One admitted suite checks …", but its real-backend `Dns` case now runs through the tide-std seam, not in the admitted conformance case. An admitted case cannot reach a `#[cfg(test)]` item.
- The Case column ("with seam-backed real coverage in `tide-std`") and research/48 §11 make this clear, so nothing is contradicted. But the stage-2 closer should record that the real-side `Dns` agreement is proved in `tide-std` rather than in the admitted suite.
- This is no weakening of the Acceptance: the admitted suite still compares both backends on literals, `InvalidInput`, `Denied`, `Timeout` and `Cancelled`, and the only alternative for a real `Dns` case is a live query, which tide-lead decided against.

## Verdict

**Approve.** B1 is resolved consistently across all four places and matches `World::http` and `host::entry` keying. R1 and R2 are in place with the N1 edits. The fix introduced no contradiction with the landed code or research/42.

DONE
