Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 25954b7 (design)

## Verdict

**Changes required**, for one small blocking finding (B1: the synthetic-address ordinal is underspecified for `localhost` and address peers). Everything else holds. The admission case is honest, the authority argument is sound and stays within research/42's NET-HOST and NET-RESOLVE, and the docs agree with each other and with the landed code. The red team's R1 and R2 limit texts are accurate and should go in as proposed, with the small wording notes under N1. After B1's one-clause fix, I approve without another full round. Only B1 needs a re-check by me.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-6d50`, branch `fx-6d50`, HEAD `25954b7`, base `28f7f6d`. `main` is now `ffbda8c`. Since the base it has only `.beads/issues.jsonl` ledger commits, so there is no overlap with the candidate's files.
- SHA-256 of `git diff 28f7f6d..HEAD`: `516c146091e2b855fbc1b365a4f5a84acd39bc7ccf56d1f3a2e5693240364b67` (it matches the Codex re-check and the red team).
- Read: AGENTS.md, GOVERNANCE.md (the Merge Rule and the admission filter, lines 91–105; research/48 §9 answers all six of its questions), `br --no-db show/comments fx-6d50`, the full six-file diff, research/48 in full, research/42 §§4 and 7 (the NET-HOST decision and the fx-6d50 constraints), reference.md §6.1 and §6.7, companion 14.2, 17 and 21, and 03 §1.1, §4.3 and §9.3 as changed. Landed code read: `net.rs` (Address, `only`, `connect`, `get`), `net/host.rs`, `net/resolve.rs`, `net/sim.rs` (`world_http`, `clear_http_peers`, `get`), `sim.rs` (`World::http`, `HttpPeers`) and `net/tests.rs` (the CALLS tests). Also read: target/work/s1-r1-codex.md, s1-r1-recheck-codex.md and redteam-codex.md. I did not repeat them.
- Checks run: `scripts/count-tokens.sh design/reference.md` gave **24,978** (at most 25,000, so it passes). `git diff --check 28f7f6d..HEAD` is clean, and the tree is clean.
- Not done: no builds and no runtime probes (stage 1 is docs only). I wrote only this report and the optional complaint line.

## Blocking findings

### B1: Which names take a synthetic ordinal is ambiguous (`localhost` and IP peers)

**Where:**
- research/48-net-resolve-v0.md:279–287 ("The n-th distinct peer host name registered in the run … gets `198.18.0.0 + n`")
- design/03-authority-concurrency.md:2223–2225 ("n is the index of the distinct peer name in setup order")
- design/reference-companion.md:465–467 ("the n-th distinct peer name in setup order")
- conformance/rules.md:105 (NET-RESOLVE-2, same wording)

**Evidence:** `sim.world.http("localhost:8080", …)` is a valid registration. `host::entry` keys it as `Host::Name("localhost")`, and `net/tests.rs:459` registers `localhost:7000`. So `localhost` is a "distinct peer host name". However, `resolve("localhost")` always returns `127.0.0.1` through `Host::literal` (research/48 §3 step 5).

Take a setup that registers `localhost:8080` first and `a.test:443` second. Read literally, research/48 §6 gives `a.test` the address `198.18.0.2`. 03's "the index of the distinct peer name … of the domains other than `localhost`" can be read either way. NET-RESOLVE-2 requires exact values on every seed, so stage 2 would have to choose the value itself. The same applies, less sharply, to address peers (`world.http("198.18.0.1:80")`). "Name" and "domain" suggest they take no ordinal, but no text says so. The red team's R1 witness (an ordinal that reveals a count of earlier peers) also depends on this rule.

**Why it blocks:** Stage 2 would have to invent part of the NET-RESOLVE-2 contract: the value a peer-named domain resolves to.

**Minimum fix:** Add one clause wherever the map is stated (research/48 §6, 03 §9.3, companion 17 and NET-RESOLVE-2). My recommendation: "Ordinals count distinct key hosts (after NET-HOST canonicalization, so `EXAMPLE.com.` and `example.com` are one) of domain peers other than `localhost`. `localhost` and IP-address peers take none." The reason is that `localhost` never receives a synthetic address, so it should not use up one. Add a NET-RESOLVE-2 case with a `localhost` peer registered first. Carry the same rule into R1's text (see N1).

## Non-blocking findings

**N1. R1 and R2 as proposed additions: accurate. Add them, with these small edits.**
- R1 ("Synthetic addresses depend on the complete ordered peer setup …") is correct against `World::http` and `sim/manifest` input order. The phrase "reveals … the count of earlier distinct peer names outside that Net's grants" is slightly off: the ordinal reveals the count of **all** earlier counted names, and the ungranted count follows from it. Suggested wording: "reveals its ordinal, and so the number of earlier distinct peer names, including names outside that Net's grants". After B1, add "(`localhost` and IP peers excepted)". Put it next to the map in research/48 §6 and companion 17. Also add a one-clause pointer in 03 §9.3, where the map is stated.
- R2 ("NET-HOST validates host keys, not DNS label or total-name lengths …") is correct (url's IDNA call uses `DnsLength::Ignore`, per the red team's citations). It applies equally to simulated `get` by name, which already succeeds for such a peer today. Phrase it as a limit of simulated names ("simulated `get` and `resolve` success proves neither …") rather than of `resolve` alone, so a reader does not take `get` to be exempt. Put it in research/48 §6 and companion 17. Do not touch reference.md, which has 22 tokens of headroom.

**N2. research/48 narrows research/42 §7 item 2 but does not say so.** research/42:502–503 says `resolve(h)` "returns the addresses `get` and `connect` would try for `h`, in the same order". After Codex's B1, research/48 §2 deliberately departs from this for the unbracketed mapped literal (`connect` dials `::ffff:a.b.c.d`, `resolve` returns IPv4). It also scopes list identity to real non-literal lookups that get the same answer. 03 (the policy owner) and 01 carry the scoped rule, so the decision record is consistent. But research/48:7–10 says it "redefines none" of research/42's contracts. Add one sentence to research/48 §2: "This scopes research/42 §7 item 2: list identity holds for real non-literal lookups; literals return the canonical key address." Otherwise a reader of research/42 alone gets the wrong answer for that one spelling.

**N3. Companion 21's list blurs the step order for literals.** reference-companion.md:779–780 reads "an IP host or `localhost` returns its canonical address without lookup; a passed deadline is `Timeout`". This reads as if a literal skips the deadline check. The contract (03 §1.1 and research/48 §3) runs the deadline check (step 4) before literals (step 5), so `resolve("127.0.0.1", past)` is `Timeout`. Fix: swap the two clauses, or write "a passed deadline is `Timeout`, literals included". Also add that case to NET-RESOLVE-1.

**N4. NET-RESOLVE-3's test mechanism does not exist yet.** research/48:449–454 calls for "the same resolver answer supplied to both calls" and "a held lookup … as NET-RESOLVE's pool test does". `resolve::lookup` calls `std::net::ToSocketAddrs` directly, with no seam. The pool test (`released_capacity_survives_cancelled_selected_waiter`) builds a synthetic `Pool`; it does not hold a real lookup. Stage 2 needs a small `#[cfg(test)]` hook in `resolve.rs`, next to `CALLS`, that supplies or holds the answer. Choosing that is a technical choice, not a contract, so it does not block. Correct the "as … does" phrase so the stage-2 builder is not sent to a test that does not do this.

**N5. `nothing.invalid` on the real backend is a live resolver query.** NET-RESOLVE-1 runs one suite on both backends, so its `Dns` case sends a real lookup to the operator's resolver. It would be the first conformance case to do so: `authority/tcp.rs` uses `.invalid` only in simulation or as an `only` entry. RFC 6761 makes the answer stable, and research/48:436–438 already notes that the no-network failure is also `Dns`. Note in research/48 §11 that this is the suite's only live query and that NSS overrides would break it. Alternatively, run it through N4's seam on the real side.

**N6. Name the red team's witnesses in §11.** research/48:462 says "stage 2 adds what it finds as cases". Name them in the list:
- R1's two-setup witness (`visible.test` becomes `.1` or `.2` depending on an earlier ungranted `hidden.test`), with different ports on one name not taking a second ordinal.
- R2's simulated half (a peer whose 64-octet label resolves in simulation). Its real half needs N4's seam; never use a live query.
- The chained-`only` case from the red team's table: `only(["svc.test:443"]).only(["svc.test:80"])` gives `Denied`, because retained full keys intersect to empty. A stage-2 builder could otherwise project entries to hosts before intersecting.

The unbracketed `"::ffff:127.0.0.1"` case is already in NET-RESOLVE-1 and §11, and it is concrete.

**N7. reference.md §6.7 says "exact `host:port` keys" for planned resolve.** reference.md:505–506 lists "planned UDP peers/resolve" under "exact `"host:port"` keys". `resolve` has no port and matches a host on any port. This is not a widening (a reader would assume less authority, not more), and §6.1's row points to companion 21. But stage 2, which must hold the file anyway to drop the `planned` label, should fix the clause, for example "(`get`, `connect`; `resolve` by host, any port)".

**N8. Style and date.** 01's bullet (design/01-design.md:520–530) now carries implementation-level detail (mapped-literal representation, synthetic addresses) that 03 already owns. A decision record could stop at "uses the shared resolver; a name grant does not grant its addresses; any port; 03 §1.1". Also, "decided 2026-11-18" is the A1 conditional date; admission is decided by this round on 2026-11-19. Consider "admitted 2026-11-19 under A1 (2026-11-18)".

## Reasoning

**1. Admission filter.**
- The surface is one std method over the existing NET-RESOLVE pool. It adds no authority kind, attenuation, type, dependency or syntax.
- Each added piece has a reason:
  - Port-free grants: research/42 §7 item 1 already decided this. It is sound because a holder of any retained `(h, p)` entry can make `connect((h, p))` send the identical `getaddrinfo(h, NULL, SOCK_STREAM)`.
  - The literal and `localhost` paths: they reuse `Host::literal`, which `connect` already takes, so no new path exists.
  - The synthetic map: research/42 §7 item 3 requires peer-named names to resolve to "a simulated address fx-6d50 assigns". Without it, the shared real/sim suite in the Acceptance could not exist.
  - The deadline: it mirrors `get`, and a hung lookup otherwise only ends by scope cancellation.
- The note is honest about cost and need. It says plainly that none of fx-zu4f.8's 87 advisories needs a lookup, and it rests the case on general v0 expressiveness, which matches the relayed "Ok" of 2026-11-19. It names the real trap for agents (Rust's resolve-then-connect-by-address habit gives `Denied` under a name grant) and says that trap fails loudly.
- The rejected alternatives hold:
  - The status quo is admitted to be smaller and sound, and rejected only on expressiveness.
  - The narrowed switch rationale is correct after Codex NB2.
  - Auto-granting resolved addresses would create a second, time-dependent policy.
  - Hashed simulated addresses can collide, while setup order is collision-free and already recorded.

**2. Ownership and consistency.**
- 01 records the decision and 03 §1.1 owns the detailed order and policy. Companion 21 and 17, rules.md and research/48 restate it consistently. I compared the order, the error set, the any-port host match, the name/address separation, `no_bind` independence and the bind exception. The only exceptions are the ordinal gap (B1) and the clause order in companion 21 (N3).
- NET-HOST is reused, not redefined. The key is the `(&str, u16)` path (`parse::<IpAddr>` then `host::host`), matching on `only` stays plain key equality, and `only` intersection is on full keys (`net.rs:167–176`). NET-RESOLVE is reused as `resolve::lookup` with port 0.
- Claims checked against landed code:
  - The mapped-literal distinction (`net.rs:87–92`, `:118`).
  - Loopback-only simulated TCP (`net/sim.rs:195–198`).
  - Simulated `get`'s existence rule: `http.keys().any(|k| k.host == key.host)` after its deadline check (`net/sim.rs:588–592`), which is the same "any port" rule `resolve` uses.
  - `Dns` mapping and non-transience as for `get`.
- Everything new is labelled planned. rules.md rows say "Planned (fx-6d50 stage 2)". Nothing claims shipped behavior.

**3. Learnability.**
- From reference.md §6.1 (signature plus a pointer to companion 21) and companion 21, a reader can write `net.resolve("svc.example", deadline)?`. They can learn:
  - it returns a nonempty `Vec<IpAddr>`;
  - a grant on any port permits it;
  - a name grant does not grant the returned addresses, so they should connect by name;
  - an address grant does not grant names: "names never matching their addresses" in the "What `only` bounds" bullet;
  - the answer does not bind a later `connect` or `get` (rebinding);
  - `only([])` denies literals too, since authority precedes literals in the order.
- Companion 17 tells them what the simulator models: no time or draws, synthetic `198.18.0.n`, peer-at-address to `get` by address, and no TCP to synthetic addresses. R1 and R2 complete the "does not model" side.
- The mapped-literal sentences in companion 21 are implementer-level, but they are correct and short.

**4. Stage-2 readiness.**
- NET-RESOLVE-1 to 3 and research/48 §11 name concrete inputs and expected variants, including the unbracketed mapped literal. The trace-event content, the replay identity (no new input, since `world.http` registrations are already ordered manifest inputs) and the range overflow panic are specified.
- What is left is B1 (a contract value) and N4 to N6 (test mechanism and named witnesses). Of these, only B1 leaves stage 2 to invent a contract.

**5. Budget.** `reference.md` measures 24,978 tokens, 22 under the 25,000 budget (recounted here). R1 and R2 must not go into reference.md.

DONE
