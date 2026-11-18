Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate b204cb5 (re-check of s2-r3)

## Scope

This is the s2-r3 finder's re-check (GOVERNANCE.md Merge Rule; AP-PROOF-001). It covers only the fixes for B12, NB2, NB3, NB4, NB5, NB7 and NB8, plus anything the follow-up diff introduced. NB1 and NB6 are recorded and deferred by tide-lead, so this re-check does not cover them.

- Reviewed baseline: `4c10448cf9f3ecdf0ec488402e79d2450fd53c98`. Candidate: `b204cb5c2e18081ef89df4cac30a0fa13a776f9f`, worktree clean.
- Follow-up `git diff 4c10448 b204cb5`: 8 files, +138 −29. `git diff --check` is clean.
- I read the whole follow-up diff and `report.md`'s "Fixes s2-r3" section. I also read `s2-r3-cancel-broken.log` and the current code it touches:
  - `net/http.rs`: `fail`, `Conn`, `dial`, `get`, `Url::key`;
  - `net/real.rs`: `retry`;
  - `net/host.rs`, in full to `host()`;
  - `net/sim.rs`: the peer-callback tail;
  - the test helpers in `net/tests.rs`.
- Other checks:
  - `openssl x509 -ext subjectAltName` on `server.der`;
  - the names added to both lockfiles since `0e38e55`;
  - the conformance runner's rule-ID check;
  - a grep for every `impl … for Url` in tide-std.
- I ran no cargo, rustc, tide-admit or heavy.sh. I edited nothing but this file and wrote nothing to `.beads/`.

## 1. B12: resolved

**What the tests exercise.**

- All three tests run `get` inside a real `Tasks::real().scope` with a 50 ms scope deadline. The peer holds the socket open, blocked on a zero-capacity channel until the caller has its result.
- The test asserts the request head arrived complete (`ends_with("\r\n\r\n")`) before it asserts the result. So the cancellation must happen in the response-read wait, after dial and send.
- Over HTTP that path is `Conn::raw_read` → `tcp_real::retry` → socket wait → `fail`.
- Over HTTPS it is `Conn::read` → `fill_tls` → `raw_read` → the same `retry` → `fail`.
- So all three tests reach `fail`'s `is_cancelled` arm on the real backend, the first mapping my report named.
- `real_get_scope_deadline_wins_at_the_same_instant` sets the scope and `get` deadlines to the same `Instant`. It would catch a regression where the socket wait returns `TimedOut` ahead of cancellation, because `fail` would then give `Timeout`. That is the "Cancelled winning at the same wait" clause.

**Before-failure: credible.**

- `s2-r3-cancel-broken.log` shows the patch removing only `fail`'s cancellation arm.
- All three tests fail at `tests.rs:263`, the variant assertion after the head check. Each gives `Err(Io(Kind(Cancelled)))`.
- So the mutation reached the response-read path, as intended, and was caught there.

**Determinism.**

- The result does not depend on timing once the head has reached the peer. Precedence at an equal instant is fixed: `retry` checks `c.check()` before its `TimedOut` test, and the socket wait re-checks cancellation first.
- The only timing dependency is that connect, the TLS handshake and the request write finish within the 50 ms scope deadline.
- The red run finished all three tests in 0.05 s at load 1.19. Since each test waits out the 50 ms itself, setup was a few ms at most.
- On this 16-core machine, under the "start below load 12" rule, the headroom is roughly 10× or more.
- A slow setup fails the test loudly (a head-check panic or a `recv_timeout` panic). It never passes falsely and never hangs:
  - `get` is bounded by its 10 s deadline;
  - the peer gets EOF or is unblocked by `recv`;
  - an un-accepted listener thread is simply leaked by a failing test.
- The 2 s elapsed bound and the 2 s `recv_timeout` are generous.
- I judge it deterministic enough. A cheap hardening is optional, see Residual R1.

**Coverage beyond the minimum fix.**

- The `is_cancelled` arms in `dial` and the resolver are still untested on the real backend. So is the `c.check()` at the top of `Conn::read`'s TLS loop: `retry`'s own check precedes it, so a mutation removing it would pass.
- My report's minimum fix asked for (a), (b) and optional (c), and all three were added.
- Testing dial or resolver cancellation deterministically would need a blackholed route or a live resolver. Neither fits "loopback servers only".
- Residual R2 records this. It does not block.

**NET-HTTP-2 evidence column: now matches.**

- The column cites `real_get_*`, `tls_get_*`, `environment_changes_nothing` and `simulated_get_*`. Every name resolves to the tests listed above.
- It states exactly what is tested: "cancellation over HTTP and HTTPS, with equal deadlines tested over HTTP".
- "TLS chain and name failures precede HTTP data" is backed by `tls_get_verifies_…` (untrusted root, empty head) and NB5's new test.
- The contract column is byte-identical to the baseline, so the rule was not narrowed.

## 2. NB2: resolved, exactly equivalent

`key_host(url::Host<impl AsRef<str>>)` replaces two copies. One was in `host::host`, on `url::Host<String>` from `url::Host::parse`. The other was in `Url::key`, on `url::Host<&str>` from `self.0.host()`.

| Variant | Former `host.rs` | Former `http.rs` | `key_host` |
| --- | --- | --- | --- |
| `Domain` | `Host::Name(d.strip_suffix('.').unwrap_or(&d).to_owned())` | `Host::Name(d.strip_suffix('.').unwrap_or(d).to_owned())` | `domain.as_ref()`, then the same `strip_suffix('.').unwrap_or(domain).to_owned()` |
| `Ipv4` | `Host::Ip(v4.into())` | `Host::Ip(v4.into())` | `Host::Ip(v4.into())` |
| `Ipv6` | `ip(v6.into())` | `host::ip(v6.into())` | `ip(v6.into())` |

- **Domain and trailing dots.** `as_ref()` on a `String` or a `&str` gives the same `&str`. In every version, exactly one trailing dot is removed.
- **IPv4 and IPv6.** The IPv4 arm is unchanged in both callers: it does not go through `ip()`, and it didn't before.
- **Mapped IPv6.** The IPv6 arm still goes through `ip()`, whose `to_ipv4_mapped` normalisation is unchanged.
- **Failure paths.** `host()` was `Some(match url::Host::parse(written).ok()? {…})` and is now `url::Host::parse(written).ok().map(key_host)`. Both give `None` on a parse error. `Url::key`'s `self.0.host()?` is unchanged.
- **Pre-parser checks.** The checks on the host as written are untouched.
- **Result.** The two callers get identical keys for every input. No trust-boundary change.

## 3. NB3, NB4, NB5, NB7, NB8: all resolved

**NB3: resolved as asked.**

- `std_net_public_items_are_fixed` now collects every single-line header in `net/http.rs` that starts with `impl` and ends with ` for Url`. It asserts the exact five: `Display`, `Debug`, `Encode`, `Decode`, `FromStr`.
- That is the full set in the crate: a grep of tide-std for `for Url` finds no other impl.
- The before-failure (a temporary sixth impl, exit 101, `s2-r3-url-guard-broken.log`) shows it bites.
- Its limits are residual R3. It does not see an impl written as `for http::Url` in `net.rs`, one outside `net/http.rs`, or a header split across lines. Those are the limits of the "cheapest hardening" I proposed, so they do not block.

**NB4: resolved, and the stated precedence is what `sim.rs` does.**

- After `catch_unwind(… c.http_peer(until, …))`, the callback is restored and the state is signalled.
- Then `sim.rs` runs `c.check()` → `Cancelled`, then `c.now() >= until` → `Timeout`, and only then `resume_unwind`.
- Companion 17's new "HTTP peer panics" bullet and research/42 §9 state exactly that order. §9's former unconditional "a peer that panics panics the requesting task" is replaced.

**NB5: resolved.**

- `server.der`'s SANs are `DNS:localhost, IP Address:127.0.0.1`.
- The new test serves that certificate, trusted through the test root, on `127.0.0.2`. It expects `Io(InvalidData)` and an empty head at the server.
- The same certificate and root succeed for `127.0.0.1` and `localhost` in `tls_get_verifies_…`. So the only variable is the name check. rustls's verifier for `ServerName::IpAddress` is now proven.
- `127.0.0.2` is loopback on Linux, and Tide v0 is x86_64 Linux only (README).
- A DNS-name mismatch stays untested. Only `localhost` resolves without DNS, so the IP case is the right one.

**NB7: resolved.**

- The new sentence in research/42 §8 lists `utf_types` 1.0.0, optional `serde`, `serde_core`, `serde_derive` and `syn` 3, and other-target `windows-sys`, `windows-targets`, `windows_*` and `wasi`.
- I took the `+name` entries of both lockfile diffs from `0e38e55`. They are identical, and every name in them is either in §8's inventory or in this sentence. `syn` 3.0.10 is there beside 2.0.119.
- Nit: the line is not wrapped like the rest of §8.

**NB8: resolved.**

- `net_fetch.rs` is tagged `NET-HTTP-3` only.
- The runner checks only that a case's IDs exist in `rules.md` (`main.rs:479`). No rule must be cited by a case, so leaving NET-HTTP-2 without a case is valid. NET-HTTP-1 and NET-HTTP-5 already have none.
- NET-HTTP-3's evidence column now names `companion_fetcher_builds_and_its_sim_test_passes`, which exists (`build.rs:4148`).
- The focused conformance run passed `authority/net_fetch` on the amended candidate.

## 4. Introduced by the follow-up: nothing blocking

- **Production code.** The only production change is NB2's extraction, which is equivalent (item 2). No leftover: both old match blocks are gone, and `key_host` has its two callers.
- **Test refactor.** `tls_serve` now delegates to `serve_tls`, which builds the `ServerConfig` and `ServerConnection` before spawning. `tls_serve`'s behaviour is unchanged: the same SNI and head capture, the same early return on an empty head, the same close modes.
- **Assertions.** `tls_get_verifies_frames_and_needs_close_notify` is unchanged. No assertion was removed or loosened.
- **`environment_changes_nothing`.** It still reruns only `--exact net::http_tests::tls_get_verifies_frames_and_needs_close_notify`. The new `tls_get_*` tests do not join the child run.
- **`rules.md`.** The contract columns of NET-HTTP-2 and NET-HTTP-3 are unchanged. Only the evidence columns were rewritten, and nothing they claimed is lost: the 64 MiB limit, the untrusted-certificate check (now "chain and name") and the trace-payload omission are all still there.
- **Single-holder files.** `design/reference.md` is unchanged, so its count stays valid. The companion's new bullet is outside the core count.

## Residuals (non-blocking; tide-lead may record them or apply them later)

- **R1.** The B12 tests give setup 50 ms: connect, TLS handshake and request write. Measured headroom is large, but a scope deadline of about 500 ms (with the 10 s `get` deadline and 2 s bounds kept) would remove the remaining flake risk on a loaded machine. It would cost about 1.4 s of test time.
- **R2.** The real-backend `Cancelled` arms in `dial` and the resolver, and `Conn::read`'s own `c.check()`, have no distinguishing test. NET-HTTP-2's evidence column no longer claims them.
- **R3.** NB3's guard sees only single-line `impl … for Url` headers in `net/http.rs`. Scanning all of `net/` for `for Url` and `for http::Url` would close the remaining gap.

## Verdict

**Approve.** B12 is resolved: the three real-backend tests reach `fail`'s cancellation mapping in the response-read wait over HTTP and HTTPS, with a credible before-failure, and the evidence column now matches. NB2 is exactly equivalent for every variant. NB3, NB4, NB5, NB7 and NB8 do what my report asked, and NB4's stated precedence matches `sim.rs`. The follow-up changes no behaviour beyond its findings and crosses no trust boundary in a way that would need a full fresh review. Landing still needs the full suites on b204cb5 (exit 0), which tide-lead is running.

DONE
