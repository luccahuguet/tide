Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 7526d4b (code)

## Verdict

**Changes required.** Fix B1–B3. The implementation follows the admitted authority and simulation design, but one resolver error is misclassified, a test retains a live resolver path, and documentation leftovers remain. Confidence: high for these source-level findings. No override.

## Scope

Read-only stage-2 supervisor pass for fx-6d50, branch `fx-6d50`, HEAD `7526d4bed96f3dda0f564476cb54f7f74f9aa7c8`, base `301b0e9`. Reviewed all 16 changed files (+418 / −39). SHA-256 of the exact output of `git diff 301b0e9..7526d4b`:

`541ebc4fbac41cdef86c306f227992ae76276a8af53849ecba6911ad1ca539bf`

Contract: a holder of `Net` calls `resolve(host, deadline)` to obtain address data. Cancellation, shape, retained-host authority and deadline precede the literal/localhost result or shared-pool lookup. Resolver failures are `Dns`; operation expiry and scope cancellation retain their own variants. Simulation uses the ordered canonical peer-name map without consuming time or draws.

Read repository instructions and governance, `br --no-db show fx-6d50`, research/48, relevant research/42 contracts, the stage-1 supervisor/red-team/Opus records, owning design sections, implementation callers and runtime context checks, tests, public-item guard, stub std, README and current driver/conformance documentation. Inspected cached pinned Rust and url/idna sources for the parser and resolver boundaries. The builder report was used only as a navigation aid.

Only this report was written. No build, test binary, resolver probe, commit, implementation edit or Beads write was started.

## Blocking findings

### B1 — A resolver's own timeout becomes an operation deadline timeout

**Where:** `crates/tide-std/src/net/http.rs:583–587,601`; `crates/tide-std/src/net/resolve.rs:103–104`; missing case at `crates/tide-std/src/net/tests.rs:784–791`.

**Concrete failure:** Set the existing seam to return `Err(std::io::Error::from(std::io::ErrorKind::TimedOut))` immediately, then call an allowed non-literal host with a deadline several seconds ahead. `lookup` returns that resolver error through `io::from_std`; `lookup_error` unconditionally turns its kind into `NetError::Timeout`. The deadline has not passed. Research/48 §3 step 6 requires `Dns` when the resolver fails and `Timeout` when the operation deadline passes while waiting. The current failure tests supply only `Other` and an empty answer, so they miss this distinction.

This is also a production error boundary: the pinned Linux std's `socket/unix.rs:45–46` propagates `EAI_SYSTEM` as `last_os_error`, rather than restricting resolver failures to one error kind. The new method cannot assume every resolver failure has kind `Other` or `Uncategorized`. The mapping is inherited from `get`, but is now reused by the new `resolve` contract.

**Suggested minimum fix:** Pass the operation deadline into the shared classification helper and map `TimedOut` to `Timeout` only when the deadline has actually expired; otherwise return `Dns`. The adjacent `dial` code already uses that guard at `http.rs:572`. Keep scope cancellation first and the pool unchanged. Add one seam-backed early-`TimedOut` regression with a future deadline; retain the held-lookup checks for actual expiry.

### B2 — The resolver counter calibration still invokes the live system resolver

**Where:** `crates/tide-std/src/net/tests.rs:114–122`, especially `:121`; fallback at `crates/tide-std/src/net/resolve.rs:84–86`.

**Concrete failure:** `no_bind_refuses_before_resolution` calls private `lookup("localhost", ...)` to show that `CALLS` counts a lookup, but installs no seam. The seam is thread-local, so another test's setup does not cover this call. It therefore executes the real `ToSocketAddrs` fallback. On a supported Linux installation with DNS in its host lookup policy and no local localhost entry, that calibration can send a DNS query; it also depends on the machine's resolver answer. This is an inherited test, not a query introduced by the new resolve cases, but it prevents satisfying this review brief's requirement that no test send a live resolver query.

**Suggested minimum fix:** Install the new seam for this calibration and return a fixed loopback address. Keep both the zero-call assertion for refused bind and the subsequent `before + 1` assertion. No production change or resolver configuration assumption is needed.

### B3 — Current documentation still describes the shipped resolver as planned, and the budget record is stale

**Where:** `docs/driver.md:495–503`; `conformance/README.md:488–489`; `design/reference-companion.md:17–19`.

**Concrete failures:** The README directs readers to `docs/driver.md` for what std provides, but that document still lists `Net::resolve` as planned and says real names use an inline resolver that cannot be abandoned. The conformance README repeats the uncancellable-resolver claim and says HTTP is a stub, although the stub includes production networking. A consumer following these current-state descriptions gets the wrong availability and cancellation behavior. Separately, the companion records **24,968** core tokens and explicitly requires updating that line after every core change. This candidate changes the core; the permitted counter run returned **25,000**, so the record incorrectly advertises 32 tokens of headroom.

**Suggested minimum fix:** Update the current network summaries to describe built `resolve`, the shared abandonable resolver and the production-backed stub; retain the planned label for UDP. Update the companion's core count to 25,000 with the recount date. These are documentation-only corrections and need no implementation expansion.

## Non-blocking findings

None requiring a separate change.

## Reasoning

- **Check ordering and authority:** `Net::resolve` obtains `network.context()` first. The context owner rejects a real capability in simulation and a simulated handle outside its own live simulation before parsing, returning a fast-path result or recording an event. Cancellation determines the first failure; shape and retained host matching precede backend lookup. `only` still intersects full `(host, port)` keys before resolve projects retained entries to hosts. Thus the chained `:443` / `:80` case denies access. Name and address grants remain distinct, and `only([])` denies literals too.
- **Host boundary:** The new tuple helper follows the existing tuple-address parsing rule and canonicalizes mapped IP keys. The shared host parser rejects percent escapes, Punycode labels, empty labels, non-ASCII and wildcards before url parsing; url's URL ASCII deny list rejects NUL, controls and separators. No new unchecked string reaches the resolver. Both backends check the operation deadline before literal/localhost success. The new `unreachable!()` is guarded by `Host::literal`: every `Host::Ip` already returned, leaving only `Host::Name`.
- **Lookup ownership and resources:** Non-literals use the existing eight-slot pool with port zero. Pinned Rust attaches the port after its null-service `getaddrinfo` call, so query behavior remains the same as connect's lookup. Resolver order, duplicates and mapped-IPv6 answer representations survive the port removal. Empty answers fail. Cancellation/expiry remove queued waiters or abandon started lookups; helper completion and spawn failure release capacity. The held-seam test exercises cancellation, answer expiry, a full-pool wait and subsequent reuse. B1 concerns error classification, not pool ownership.
- **Simulation:** Setup assigns one ordinal per distinct canonical domain host, excluding localhost and IP peers; duplicate spellings on different ports share an ordinal. The capacity guard bounds the arithmetic to the specified range. Resolve and get share name existence, while get still requires an address-registered HTTP peer for synthetic-address access. TCP remains loopback-only. The new calls allocate no replay input, consume no draw or virtual time, and record one resolve event, including refusals and pre-cancellation.
- **Coverage:** The admitted suite compares real values with four simulated seeds, including mapped literals, any-port grants, name/address separation, empty/chained grants, a passed literal deadline and cancellation over expiry. Seam tests cover answer order/duplicates, failures, empty answers and held lookups. R1's two setups and R2's 64-octet-label witness are present. The new admitted suite's real calls are literals/localhost or stop before lookup; the new non-literal real unit cases install the test-only seam. B2 identifies the remaining inherited live fallback.
- **Surface and trust:** Only `Net::resolve` is added to the public guard. The helper modules remain private, the seam/type/counter and seam selection are `#[cfg(test)]`, and release behavior cannot be overridden through them. The conformance stub includes production `net.rs`. Existing `NOT_PURE` ownership and the trust-diff test classify `Net` as `net`; no new authority type or ambient resolver export is introduced.
- **Proof inspected, not rerun:** `target/work/suites/candidate.txt` names the full candidate hash; `head-check` records unchanged HEAD, and both suite exit files contain zero. The logs total **469 passed / 0 failed** for workspace tests and **389 PASS / 3 AWAITS CHECKER / 0 FAIL** for conformance, including the new resolve checks. This is tide-lead's supplied execution evidence. My own checks were source inspection, `git diff --check 301b0e9..7526d4b` (clean), diff hashing and `scripts/count-tokens.sh design/reference.md` (**25,000**, within the cap). The reference trims preserve the grant/bind rules and widen the companion time pointer to cover resolve; no rule loss found. HEAD and tracked status remained unchanged.

Not checked: no new runtime execution, live resolver query or build. The early-timeout counterexample is established by source flow; the requested seam regression has not been run. Passing supplied suites do not cover it.

DONE
