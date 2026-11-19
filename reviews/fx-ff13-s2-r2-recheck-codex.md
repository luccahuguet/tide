Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 4c10448 (re-check of B9-B11)

## Verdict

**Approve.** B9, B10 and B11 are resolved. Each has a distinguishing regression that fails on the named before state and passes with the fix. No fix widens accepted peer input or changes behavior beyond its finding; these fixes do not require a full fresh review. Confidence: high. This is the bounded Codex finding re-check, not the subsequent Opus verdict or landing approval. No override.

## Candidate and evidence identity

Observed HEAD `4c10448cf9f3ecdf0ec488402e79d2450fd53c98`; tracked worktree clean. Independently inspected `git diff abc1e4d 4c10448`: exactly three files, **150 insertions, 8 deletions**, confined to `crates/tide-std/src/net/http.rs`, `crates/tide-std/src/net/sim.rs` and `crates/tide-std/src/net/tests.rs`. Independently computed its SHA-256:

`4b2f29a41d083f01b977cb5a70c007acccbab3478fbe50284bf73976dbcf2509`

Read the original B9–B11 in `target/work/s2-r2-codex.md`, the fixer's “Fixes s2-r2” in `target/work/report.md`, the affected source and test helpers, and the corresponding source at `abc1e4d`. The tests-only diff from abc1e4d to this candidate independently hashes to `35dc8659fb94732c2dc26f1a9ab57ca583e20514deec4abcc4868ea97de3665f`, matching `s2-r2-before-state.txt` and the reported abc1e4d-plus-regressions before state. Read `s2-r2-before.log` and `s2-r2-net-sim-final.log`; the former records all five new tests failing, the latter all five passing within **46 passed, 0 failed**. These are observed existing run artifacts, not reviewer-executed Rust tests. `git diff --check abc1e4d 4c10448` passes.

## B9 — resolved

`http.rs:348` adds the chunk-line validator; `http.rs:517` applies it before advancing the buffer position or interpreting a zero size. A line must start with a hexadecimal digit. Empty and extension-only lines therefore cannot terminate the body. Every extension must have a token name and, when present, a token or properly terminated quoted value. Quoted text and escaped bytes exclude NUL and other disallowed controls, while retaining SP/HTAB, valid escapes and opaque quoted bytes. The check covers ordinary and terminal chunks.

Inspected pinned `httparse-1.10.1/src/lib.rs:1263–1338`: its completed length includes CRLF, so removing the last two bytes for validation is justified. Its old zero-initialized size and ignored extension octets explain the reported failures without relying on the fixer's conclusion.

`real_get_rejects_chunk_size_lines_without_digits` (`tests.rs:566`) reproduces the exact valid-abc-then-empty-size scenario and an extension-only control. `real_get_validates_chunk_extensions` (`:576`) includes `1;x=\0`, malformed names/values/quotes/escapes and valid extension controls. Before-log failures show the empty-size and NUL-extension cases returning no InvalidData; tracing the old reader shows successful bodies. Both tests pass after the fix. The original failure scenarios are closed.

## B10 — resolved

`http.rs:433` rejects codes outside 100–599 immediately after the complete head is parsed, before informational, bodyless or framing selection. The old source has no such check; pinned httparse's `parse_code` at lines 933–940 validates decimal digits only.

`real_get_refuses_invalid_status_codes_before_body_framing` (`tests.rs:598`) covers 000/099/600/999, an unframed 600 response whose peer remains open, and valid unrecognized 299/599 controls. The existing helper gives GET a ten-second deadline; the held-open peer sleeps three seconds, while the assertion requires rejection within two seconds. Thus the latter assertion distinguishes immediate head rejection from waiting for body completion. The before run fails on 000 returning no InvalidData; the old path would return Status(0). The fixed test passes, including body delivery for 299 and Status(599). Both invalid-code acceptance and delayed rejection are closed.

## B11 — resolved

`sim.rs:573–585` retains cancellation-first arrival handling and represents an impossible arrival as None. That state waits to the call deadline and returns Timeout. Checked additions now cover the request (`:594`), no-peer response (`:598`), and both registered-peer reply additions (`:632`). Overflow can neither panic nor become a ready reply at Duration::MAX. For representable arrivals, the existing arrival/deadline ordering and wake target remain equivalent. Hang also uses the impossible-arrival state.

`simulated_unrepresentable_reply_arrivals_time_out` (`tests.rs:611`) covers delayed Ok/Status/Reset/Hang with a last-representable-instant deadline and checks the final clock. Its before failure records `Ok([108, 97, 116, 101])`, the erroneous “late” body. `simulated_packet_arrival_overflow_obeys_deadlines_and_cancellation` (`:629`) covers request overflow at MAX−1ns, no-peer response overflow after a representable request, cancellation at the final instant, and an ordinary representable successful reply. The disposable same-seed latency draw places the no-peer case at the intended second addition. Its before failure is the unchecked Duration-addition panic. Both fixed tests pass. The original timing failure scenarios and cancellation requirement are closed.

## Scope and acceptance

B9 and B10 add rejection guards after the same parsers, without replacing parsing or changing successful body handling. Their accepted wire input is therefore a subset of the previous input. The new valid controls preserve extension forms and unrecognized in-range statuses relevant to the fixes. B11 changes only HTTP arrival overflow/never-arrival handling; the scheduler, RNG owner, callback lifecycle and ordinary representable arrival path remain unchanged. No authority, URL, admission, dependency or public API change occurs in this fix diff. No behavior beyond B9–B11 was found that would restart full review under the Merge Rule.

## Full-suite observation and limits

At **2026-11-18T23:01:56Z**, `target/work/suites/candidate.txt` names the full 4c10448 SHA and `test.start` records **23:00:41Z**. The workspace log is still in the 116-test build group; only the earlier 16-test and 12-test groups have completed, both with zero failures. There is no `test.exit` or `test.end`. No conformance log, start or exit artifact exists yet. Neither full-suite completion nor full conformance success is claimed.

No heavy build, Rust probe, source/test edit, planning write or commit was performed. This requested report is the sole authored artifact. B1–B8 and the rest of the change were not re-reviewed. Remaining unobserved gates are the running full suites and subsequent required Opus review.
DONE
