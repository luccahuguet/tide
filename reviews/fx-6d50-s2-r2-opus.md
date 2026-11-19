Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 98370da (code)

## Verdict

**Approve.** No blocking findings. Four non-blocking findings follow; N1 and N2 are text-only and need no re-check if taken.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-6d50`, branch `fx-6d50`, HEAD `98370daadfd0894adbb4efe6758d44719611bb6a`, base `301b0e9`.
- SHA-256 of `git diff 301b0e9..HEAD`: `8d5093d60f32032621203e0e29a32c19b04490486825c9265fdd2da1a15b811c`. This matches the builder's and the Codex re-check's hash. The diff touches **18** files (+439 −49), not the 17 the brief says; the hash identifies the tree.
- Read: AGENTS.md, GOVERNANCE.md rules as rendered in AGENTS.md, `br --no-db show/comments fx-6d50`, the full code/test/conformance diff, `net.rs`, `net/{resolve,host,http,sim,tests}.rs` (changed regions plus callers), `task/net.rs` (`NetContext::wait`/`check`), `task.rs` (`Tasks::scope` runs its body on the calling thread), `sim.rs` (`HttpPeers` clear-on-drop), the conformance stub std, research/48 §§2–6 and §11, research/42 §7 and its NetError table, 03 §1.1/§9.3, companion 14.1/17/21, reference §6.1/§6.7/§9, rules.md NET-RESOLVE-1..3, docs/driver.md, conformance/README.md, root README.md, target/work/s2-r1-codex.md and s2-r1-recheck-codex.md.
- My own checks: `git diff --check 301b0e9..HEAD` is clean. `scripts/count-tokens.sh design/reference.md` gives **25,000**, which is within D14's cap and matches the companion's line. I started no build and wrote nothing except this report.
- Proof inspected, not rerun: in `target/work/suites-r1/` (candidate 98370da), `cargo test --workspace` exited 0 with **469 passed / 0 failed** (04:25:42Z–04:30:02Z). It includes `resolve_builds_and_runs_against_production_std`, the five `net::http_tests::*resolve*` tests and `released_capacity_survives_cancelled_selected_waiter`. **The conformance rerun was still running at 04:33Z** (76 PASS so far, no exit file). The 7526d4b run passed `authority/resolve.rs`, but its result is not reused here. tide-lead should confirm the 98370da conformance exit before landing.

## Blocking findings

None.

## Non-blocking findings

**N1. The root README understates `Net`'s v0 surface** (`README.md:208`, and the simulator table at `:247–248`). The README says std provides "`Net` for TCP and one HTTP/1.1 `get`", so a reader of the public alpha's front page won't learn that `resolve` exists. The README was not in the diff. The README makes no false "planned" claim, so this does not block. Minimum fix: "`Net` for TCP, one HTTP/1.1 `get` and `resolve`". In the simulator table's `Net::get` row, optionally add "`Net::resolve` (synthetic `198.18.0.n` addresses for `world.http` domains)". The TCP row's "DNS names" in the not-modelled column stays true for TCP. Text-only.

**N2. research/48 §11 and §3 contain present-tense text that stage 2 made false** (`research/48-net-resolve-v0.md:429–493`, `:155`). The status line now says "built", but:
- §11 still says the rules rows are "planned".
- It says `lib.rs` "still says names 'use inline Rust resolution…'" and that "Stage 2 updates both"; stage 2 already did.
- §3 points to "`http.rs`, lookup `map_err`", which is now `lookup_error(e, until)`.

Minimum fix: one sentence at the head of §11 ("Done in stage 2; kept as the plan of record"), and point §3 at `lookup_error`. Text-only.

**N3. Cheapest missing order witnesses for NET-RESOLVE-CALL steps 2→3→4.** Research/48 §3 fixes the order cancellation → shape → `only` → deadline. The suite proves cancellation over deadline, `only` before deadline (an any-port grant reaching `Timeout`) and deadline before literal. It never runs a call in which two of shape, `only` and deadline fail together. Two lines in `conformance/cases/authority/resolve.rs` would pin the order on both backends: `net.only([""; 0]).resolve("", past)` → `Io(InvalidInput)`, and `net.only([""; 0]).resolve("svc.test", past)` → `Denied`. None of these steps performs a lookup, so this adds no authority risk. rules.md NET-RESOLVE-1 doesn't require these cases, so this doesn't block.

**N4. Simulated synthetic-address bookkeeping** (`net/sim.rs:96–101`, `:554–559`). `SimNet::address` scans `names` linearly, and `world_http` calls it twice per registration, so setup is quadratic in the number of distinct names. This is negligible at test scale. The 131,070-name overflow panic, which research/48 §6 promises, has no test, and a test would be slow under that quadratic scan. A `BTreeMap<Host, u32>` alongside the `Vec`, or in place of it, would make both cheap. Optional.

Out of scope, pre-existing (not caused by this change):
- reference §12.3 lists `UdpSocket` under `std::net` without a `(planned)` label.
- `docs/owner-guide/10-v0-and-planned.md:100` shows a stale `br list` snapshot (fx-6d50 still titled with UDP, fx-ff13 still open).

## Reasoning

1. **Authority.**
   - `Net::resolve` (`net.rs:273–294`) checks in order: `c.check()` → `host::tuple` shape → `only` host match → backend deadline → literal → lookup. That is research/48 §3's order, and nothing before the backend call can reach `resolve::lookup`.
   - `host::tuple` is exactly `impl Sealed for (&str, u16)`'s rule: an `IpAddr` parse through `host::ip` (so an unbracketed `::ffff:127.0.0.1` keys as IPv4), else `host::host`. So `resolve`'s key is `connect((host, port))`'s key.
   - `only` keeps full `(host, port)` keys from `entry` and intersects them on chaining; `resolve` then compares hosts only. So:
     - a port-scoped grant allows its host;
     - the chained `:443`/`:80` intersection is empty and gives `Denied`;
     - `only([])` denies literals;
     - `Host::Name` never equals `Host::Ip`, so name and address grants stay separate, `localhost` included.
   - The resolver receives exactly the key's canonical name (`lookup(c, name, 0, …)`). Rust's null-service `getaddrinfo` attaches the port afterwards, so port 0 sends the same query as `connect`'s lookup. No new string reaches the resolver.
   - Public surface: only `pub fn resolve` is new; NET-HTTP-4's fixed list was updated. `tuple` is `pub(crate)` in a private module; the sim functions are `pub(super)`. `SEAM`, `Answer`, `CALLS` and the seam branch are all `#[cfg(test)]`, and the `#[cfg(not(test))]` arm calls only the system resolver. The stub std compiles `net.rs` as a non-test library, so admitted programs never see the seam.
2. **Resources and time.**
   - A pool slot is taken only at `break`. Every exit path releases it: the helper thread releases on every outcome (Ok, error, empty); spawn failure releases at once; queued waiters remove their signal on every wake and return through `result?`.
   - The B1 guard is sound. `NetContext::wait` returns `Ok` on timeout, and `lookup` itself returns `TimedOut` only after `expired()` (`crate::time::monotonic() >= until`). `lookup_error` re-reads the same monotonic clock, so a deadline that `lookup` saw pass is always `Timeout`. A resolver's own `TimedOut` before the deadline is `Dns`.
   - `get`'s other paths already used this guard: `dial` (`http.rs:572`) and `fail` (`http.rs:196–199`, connect/TLS/read/write). B1 brought the lookup, the one outlier, into line. This now matches research/42's NET-ERR table ("`Timeout`: deadline passed at any step"; "`Dns`: the resolver failed") and companion 21 ("the operation's deadline passed").
   - Cancellation is checked first at every wait, so it wins over a passed deadline.
3. **Simulator and replay.**
   - `names` is pushed only in `world_http`'s insert branch, and only for a host with no `address` yet. Literals and `localhost` return from `literal()` first, so they never take an ordinal. NET-HOST canonicalization merges `A.test.:80` with `a.test:443`, so they share one ordinal. The arithmetic gives `198.18.0.0 + n` with n ≤ 131,070, i.e. up to `198.19.255.254`.
   - `get`'s `Dns` test changed from `http.keys().any(..)` to `address(..).is_none()`. The two are equivalent during a run: `names` and `http` grow together, and `http` is cleared only by `SimRoot`'s drop guard.
   - The sim `resolve` takes no `c.wait`, `latency`/`draw` or `begin` point. It records one `net.resolve` event, also for refusals and pre-cancellation, and adds no `replay_input`. The tests compare the next draw and elapsed time with and without resolve, and compare traces across reruns.
   - Simulated TCP still refuses names (`InvalidInput`) and synthetic addresses (`ConnectionRefused`).
4. **`get` after B1.** Covered under item 2. The regression test proves `Dns` for an early resolver `TimedOut` on both `resolve` and `get`. The held-lookup test still proves `Timeout` at real expiry and `Cancelled` on scope cancellation.
5. **Contract, docs and budget.**
   - Code, rules.md, 03 §1.1/§9.3, companion 14.1/17/21, reference §6.1/§6.7, driver.md, conformance/README.md and the `lib.rs`/`net.rs` comments agree. UDP stays planned everywhere.
   - The reference trims ("unrestricted" → "open", "`Net::get`: companion 17" → "`Net`: companion 17", dropping "peers/resolve" from the planned list) lose no rule; companion 17 covers all simulated `Net` time.
   - Count: 25,000 = 25,000.
   - Only N1 and N2 remain.
6. **Tests and leftovers.**
   - NET-RESOLVE-1: one admitted suite, run on both backends through tide-admit. The real values must equal every seed's values, and each backend also asserts the expected list itself. The suite sends no live query: every real call is a literal, `localhost`, or stops before lookup.
   - NET-RESOLVE-2: setup order, the localhost-first case, the IP peer, port duplicates, R1's two setups, R2's 64-octet label, no time, no draw, trace count and rerun identity.
   - NET-RESOLVE-3:
     - zero `CALLS` on refusal (with a seam that would fail loudly if asked);
     - the shared answer through `connect`'s own `Address::resolve` path, keeping order, duplicates and the mapped IPv6 representation;
     - `Dns` for a failure, an empty answer and an early `TimedOut`;
     - held lookups giving `Cancelled` before the call deadline, `Timeout`, a queued `Timeout`, and slot reuse after release.
   - The seam is thread-local and read on the caller thread, and `Tasks::scope` runs its body on that thread, so the held-lookup test cannot fall through to the system resolver.
   - The B2 calibration is seamed. No remaining tide-std test sends a non-literal name to the system resolver.
   - The tests check observable results through the public method, not internals; the connect-equivalence case necessarily runs through the shared seam, which is what research/48 §11 specifies.
   - No dead code. The stale `lib.rs` comments research/48 named are fixed.

DONE
