Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 98370da (re-check)

## Scope

Finder re-check of B1–B3 from `target/work/s2-r1-codex.md`, read first. Inspected only their fixes in `git diff 7526d4b 98370da` and the connected code, tests, contracts and supplied proof needed to assess them. No full review reopened; no additional problem introduced by these fixes found.

Candidate: `98370daadfd0894adbb4efe6758d44719611bb6a`, branch `fx-6d50`, base `301b0e9`. Tracked status was clean before writing this report. SHA-256 of `git diff 301b0e9 98370da`:

`8d5093d60f32032621203e0e29a32c19b04490486825c9265fdd2da1a15b811c`

## B1 — Resolved

`crates/tide-std/src/net/http.rs:584–589` now takes the operation deadline in `lookup_error(e, until)`. Cancellation remains the first arm. `TimedOut` becomes `Timeout` only when `expired(until)` is true (`monotonic() >= until`); otherwise it becomes `Dns`. Both callers pass their operation deadline: `resolve` at line 602 and `get` at line 623. The shared pool and its cancellation-first wait checks in `net/resolve.rs` are unchanged.

The seam regression at `net/tests.rs:786–797` sets a deadline five seconds ahead, supplies an immediate resolver `TimedOut`, and requires `Dns` from both `resolve` and `get`. The old unconditional mapping would fail these assertions. The held-lookup test at lines 800–829 still requires cancellation to return `Cancelled`, actual operation expiry to return `Timeout`, queued expiry to return `Timeout`, and released capacity to be reusable.

Checked the classification against research/42 §3 steps 4–5 and §5's error table, research/48 §3 step 6, companion 21's `NetError` and `resolve` descriptions, the design/03 Network errors note, and the `NetError` variant documentation. They agree: deadline expiry is `Timeout`; resolver failure or an empty answer is `Dns`; scope cancellation retains precedence. No contract correction is needed.

## B2 — Resolved

`net/tests.rs:116` now installs the existing thread-local seam before the calibration lookup, returning a fixed loopback answer. `net/resolve.rs:76–88` clones that seam on the caller thread and invokes it on the helper thread instead of the system resolver. Thus the formerly unseamed direct `lookup("localhost", ...)` at test line 123 cannot reach `ToSocketAddrs`.

Both calibration assertions remain: the refused bind leaves `CALLS` unchanged (line 120), and the subsequent seam-backed lookup increments it by exactly one (line 124). The other real non-literal cases retain their own seams; the fix adds no unseamed call. The remaining real literal/localhost and refusal paths identified in the original review are unchanged. No resolver configuration assumption remains in this calibration.

## B3 — Resolved

`docs/driver.md:492–506` now describes NET-HOST attenuation, the shared abandonable resolver pool, built `Net::get` and `Net::resolve`, and synthetic addresses for simulated HTTP domains. It limits the rejection of other simulated names to TCP. UDP remains explicitly planned.

`conformance/README.md:488–492` now describes the same pool and abandonment behavior and correctly states that the stub includes production `net.rs`, making `get` and `resolve` real. UDP remains absent. The production include is present at `conformance/rustc/std/src/lib.rs:76`.

`design/reference-companion.md:17–19` records **25,000** core tokens, recounted on **2026-11-19**. The core itself has identical blob `7bac9c5e897e3883bed5d74dedd23b632fcd3ce9` in 7526d4b and 98370da, so the original review's 25,000-token measurement still applies. The record no longer advertises headroom; none remains under D14's cap.

## Proof and limits

- Source inspection and `git diff --check 7526d4b 98370da`: clean on the candidate above.
- Supplied `target/work/s2-r1-tide-std.log`: 38 passed, zero failed, including the early-timeout regression, held-lookup test and resolver-counter calibration.
- Supplied `target/work/s2-r1-admit.log`: two passed, zero failed, covering the fixed public surface and production-std resolve execution. The builder's “Round s2-r1 fixes” record binds these focused runs to 98370da. The repository pins `nightly-2026-08-09`; logs show the Linux worktree and shared target directory.
- At the final inspection (2026-11-19 04:26 UTC), `target/work/suites-r1/candidate.txt` names the exact candidate. The workspace log was still progressing through build tests; no final exit record or conformance result was present. These full suites are pending and are not claimed as passing proof here. Results from 7526d4b are not reused for changed code.

## Verdict

**approve** — B1, B2 and B3 are resolved. Confidence: high for this bounded finder re-check. No override. This verdict completes the Codex finding re-check; full-suite completion and the subsequent Opus review remain separate landing requirements.

Not checked: no new build, test execution, resolver query or token-counter run. Only this report was written; no source, Beads or history changes were made.

DONE
