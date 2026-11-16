Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: approve

## Scope and identity

Finder's narrow re-check under AP-PROOF-001 of the findings in `r1-codex.md`, `r1-recheck-codex.md`, and `r2-recheck-codex.md`, the author's responses in `report.md`, and `git diff 821d666 898c8ab`. Read `br --no-db show fx-ip4x`, repository instructions, GOVERNANCE.md, the affected RUN-2/RUN-3 and simulation contracts, and connected stream/publishing code. This is not a new full review of unchanged sandbox machinery.

Reviewed HEAD `898c8ab9a17268a78373f83b9fd7ec6750bec093`, branch `fx-ip4x`, based on main `4cb53db9e6bc7f9f53d1acdbe2ccbfdbb615e2d6`. Independently verified `git diff main...HEAD` SHA-256 `5d79167c54e172e44a02680432b441d165e151878cca8f119fa7a496330b36cf` and a clean working tree. Since 821d666, only `invocation/replay.rs` and `tests/build.rs` changed: bounded final diagnostic delivery and the symmetric stalled-stderr regression.

Contract: the sandboxed child writes only its fresh run directory; publication follows no candidate-controlled output link and replaces no other run's artifact; both child pipes drain independently of external consumers; simulation deadline/crash completion remains bounded and reports the last started seed when stderr accepts the report. A permanently stalled stderr cannot promise diagnostic delivery.

Confidence: high in the source-level re-check. No runtime checks or test mutations were executed in this pass.

## Findings re-checked

### N2 [P2]: blocked forwarding withholds timeout/crash completion — fixed

Both final failure branches now call `report_bounded` instead of `eprintln!` (`crates/tide-admit/src/invocation/replay.rs:265`, `:269`). The helper starts a detached writer, uses raw `libc::write(2, ...)` without Rust's stderr lock, handles partial writes and EINTR, and bounds the main thread's wait to two seconds (`:280`, `:285`–`:302`). A writer blocked on the kernel pipe cannot hold the main thread beyond that wait. The stderr forwarder may still hold std's stderr lock (`:340`), but the final diagnostic no longer acquires it. Both branches return `Ok(101)`, and `main` returns that code without another error print (`crates/tide-admit/src/main.rs:106`–`:112`).

Checked the pinned nightly's local std source: `library/std/src/rt.rs:139`–`:145` and `:187` perform runtime cleanup; `library/std/src/io/stdio.rs:729`–`:744` uses stdout `try_lock` and does not lock stderr. Thus the detached stderr writer does not recreate the reviewed lock dependency during Rust shutdown. This conclusion concerns the inspected supported Linux path, not a general guarantee about arbitrary host failures.

### Original 4 [P2]: simulation deadline and seed-on-crash diagnostics — fixed

The receipt's simulation flag selects the driver's limit, defaulting to 300 seconds (`crates/tide-admit/src/invocation/replay.rs:212`–`:213`, `:306`). Seed observation restarts the deadline (`:352`–`:357`); expiration kills and reaps the child (`:376`–`:390`); timeout and signal reports retain the last seed (`:263`–`:270`). tide-std emits and flushes seed starts (`crates/tide-std/src/sim/runner.rs:190`–`:194`). N2's final stderr dependency is now removed. With drained stderr, the existing tests still require the actual timeout/signal diagnostic and seed 47 (`crates/tide-admit/tests/build.rs:5000`–`:5006`, `:5034`). With permanently stalled stderr, bounded failure completion takes precedence over delivering bytes the reader will not accept, as the author explicitly records.

### Original 1 [P1]: widened writable grant / other runs' artifacts — fixed; remains fixed

Exclusive 0700 run-directory creation remains at `crates/tide-admit/src/invocation/sandbox.rs:278`–`:290`; the writable Landlock rule remains confined to its `sim/` (`:225`–`:230`). Directory opens use `O_NOFOLLOW` (`:356`, `:366`), and publication opens `target` and `sim` relative to directory descriptors (`:314`–`:316`). The amendment changes none of these boundaries.

### N1 [P1]: destructive publication into another run's namespace — fixed; remains fixed

The parent derives its own token prefix and rejects child entries outside it (`crates/tide-admit/src/invocation/sandbox.rs:300`–`:302`, `:312`, `:319`–`:321`). Descriptor-relative `renameat2(..., RENAME_NOREPLACE)` refuses existing destinations atomically (`:325`–`:334`). No destination deletion has returned. Only the explicitly supported replacement of a file/link at `target` or `target/sim` remains (`:387`–`:389`), without following it. Honest manifest naming and replay hints still use that namespace (`crates/tide-std/src/sim/manifest.rs:94`–`:103`, `:230`–`:234`). No new publication/confinement gap was introduced by this amendment.

### Original 2 [P1]: inherited stdout/stderr — fixed; remains fixed

Null stdin and separate child stdout/stderr pipes remain at `crates/tide-admit/src/invocation/replay.rs:246`–`:248`. Raw diagnostic writes execute in a parent thread; they do not expose the parent's fd 2 to the child. Reader threads still only observe/drain and enqueue, without writing to external sinks (`:349`–`:373`). Normal completion waits for both forwarders and propagates errors (`:402`–`:416`); abnormal completion bounds that wait. No new normal-exit output loss or reader/forwarder deadlock was introduced. Queued output can still be lost after abnormal-exit grace, and buffering remains unbounded, both already disclosed limitations.

### Original 3 [P1]: descriptor-cleanup fallback — fixed; remains fixed

Preparation still requires the close_range/CLOEXEC probe (`crates/tide-admit/src/invocation/sandbox.rs:40`–`:47`), and child installation checks the actual call covering fd 3 through `u32::MAX` (`:120`–`:122`). No unsafe soft-limit fallback was restored. The final parent-side diagnostic thread starts after child execution and does not affect inherited-descriptor cleanup.

## Folded-in non-blocking findings

- **Degraded-kernel behavioral coverage — not fixed completely; remains non-blocking.** The extracted level-policy test covers ABI 10/6/4, absent Landlock, absent seccomp, and neither (`crates/tide-admit/src/invocation/sandbox.rs:644`–`:652`). It does not exercise probes, old-ABI ruleset installation, or installation failure. The latest author response accurately acknowledges this; nothing changed here since the preceding re-check.
- **Unsupported-platform `none` — not fixed; explicitly skipped rather than claimed fixed.** It remains disclosed at `README.md:493`–`:495`. This is not an additional blocker in the currently supported x86_64 Linux execution path.

No regression exercises close_range failure. This remains a proof limitation, not an unresolved instance of original 3: the unsafe fallback was removed, and that finding requested a high-fd regression conditionally on retaining fallback compatibility.

## Regression-test sensitivity

These are source-level counterfactual checks, not executed mutation results.

- **New stalled-stderr regression:** `run3_stalled_stderr_does_not_hold_the_limit` first requires a successful admitted run, then swaps in a child that emits seed 47, floods stderr with about 7 MB, and stays alive (`crates/tide-admit/tests/build.rs:5048`–`:5057`). It drains stdout, reads stderr only through the pre-child `sandbox:` line, then holds that pipe open without reading (`:5058`–`:5067`). It actively polls for termination, kills/reaps and fails at 120 seconds, and requires failure status (`:5068`–`:5080`). Restoring 821d666's synchronous `eprintln!` in the timeout branch leaves the driver waiting behind the stderr forwarder's lock after the two-second evaluation limit and forwarding grace; the hang guard fails. Thus it distinguishes the remaining reviewed bug under the supported test environment. It does not independently test the raw-write loop's EINTR/partial-write branches, an exact two-second reporting bound, or stalled-stderr signal termination. The latter shares the inspected helper; the existing drained signal test checks its diagnostic branch. No diagnostic assertion is appropriate against the deliberately unread stderr pipe.
- **Retained stalled-stdout regression:** the child announces seed 47, floods stdout and stays alive; stdout remains unread while stderr is drained (`:5022`–`:5034`, helper `:4869`–`:4871`). Restoring coupled draining/forwarding or unbounded forwarder waiting causes its hang guard to fail. Its seed-bearing diagnostic assertion also protects the changed report path when stderr is writable.
- **Retained deadline/signal regression:** `:5000`–`:5006` fails if seed observation/reporting is removed or the signal report disappears; removing deadline enforcement causes the active 300-second guard (`:4872`–`:4878`) to fail. This covers both callers changed to `report_bounded` with writable stderr.
- **Retained confinement/publication canaries:** `:4965`–`:4975` distinguish restored shared grants, inherited stdout, or failure to replace the planted output link. The hostile child demonstrably creates `forged`, while the real published sibling must remain `original` (`:4932`–`:4938`, `:4977`–`:4983`); restoring 9be9d30's destructive publisher fails that assertion. As previously reported, that unprefixed canary does not independently distinguish removal of only the prefix check or only no-replace when the other defense remains. No matching-prefix collision test was added; the inspected checked syscall owns no-replacement.
- **Level-policy test:** its assertions detect incorrect reporting strings, but the pre-extraction inline policy already made those decisions. It adds coverage, not proof of repaired degraded-host installation behavior.

## New blocking findings

None within the requested finding-specific re-check and amendment.

Non-blocking nit: both `git diff --check 821d666 898c8ab` and `git diff --check main...HEAD` report `crates/tide-admit/tests/build.rs:5082: new blank line at EOF`. They do not pass whitespace checking; this does not affect behavior or block the verdict.

## Reasoning

The amendment repairs exactly the surviving stderr-lock dependency: the parent's final diagnostic can neither wait indefinitely for Rust's stderr lock nor force the main thread to wait indefinitely for a raw write. Existing reader/forwarder separation preserves seed observation and pipe draining; normal exits retain their complete-forwarding path. The confinement, no-follow publication, parent-owned namespace, atomic no-replacement, and required descriptor-cleanup protections remain unchanged and intact under this narrow review. The new regression would expose the previous synchronous diagnostic hang, while retained tests cover delivered timeout and signal reports. No further corrective implementation is required for the reviewed blockers.

This approves the requested Codex re-check. It does not replace the coordinator's full-suite proof or the subsequent Opus review required by GOVERNANCE.md. Repository files were unchanged; only this requested report outside the repository was written.

Not checked: cargo, conformance, runtime probes, executed mutations, or independent reproduction of the author's focused results; full suites belong to the coordinator.
Remaining limitations: stalled stderr can lose the final diagnostic, abnormal exits can drop queued output, buffering is unbounded, degraded-host/close_range failure coverage remains absent, and RUN-6 escaping is still staged work.

DONE
