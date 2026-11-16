Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

## Scope

Finder's narrow, read-only re-check of the findings in `r1-codex.md` and the changes from `60a40e6` to `9be9d30556491a78fa45a33fd45b3b383c6d65fb`. Base/main remains `4cb53db9e6bc7f9f53d1acdbe2ccbfdbb615e2d6`. Independently verified `git diff main...HEAD` SHA-256 `49c4531697dca8c1d519469abe0a3901b437aa18b688d19224dcf285f01bdb47` and a clean working tree. Read the previous review, the author's amended report, GOVERNANCE.md, `br --no-db show fx-ip4x`, the amendment, RUN-2/RUN-3/RUN-4/RUN-6, and the connected simulation monitor and recording code. This is not a repeated full review of unchanged sandbox machinery.

Contract re-checked: isolate the child's writable directory and inherited descriptors, publish only that run's artifacts without following links or damaging other runs, and preserve the simulation deadline and seed-on-crash report when process ownership moves to the driver. Confidence: high in the source-level failure paths below. No runtime or mutation tests were executed.

## Original blocking findings

1. **[P1] Writable grant includes unrelated files/other runs — not fixed completely.**

   The direct kernel grant is repaired: `RunDir::create` exclusively creates a 0700 run directory (`crates/tide-admit/src/invocation/sandbox.rs:267`, `:280`), and the writable rule targets only its `sim/` (`:225`, `:230`). Rule targets use descriptor-relative opens with `O_NOFOLLOW` (`:296`, `:349`). Publishing opens `target` and `sim` relative to directory descriptors, replacing a planted link instead of following it (`:305`, `:370`, `:384`). These changes close the original symlink-induced Landlock grant widening.

   However, the replacement publisher gives the child another way to destroy or replace another run's artifacts. It accepts every child-created directory name and recursively removes an existing published directory with that name (`:308`, `:313`, `:319`, `:321`), then renames the child's directory over it (`:329`). The run token added by the honest recorder (`crates/tide-std/src/sim/manifest.rs:94`) is not enforced by the parent. See new blocking finding N1.

2. **[P1] Inherited stdout/stderr bypass confinement — fixed.**

   Both child streams are now `Stdio::piped()`, and stdin remains null (`crates/tide-admit/src/invocation/replay.rs:244`–`:248`). The child receives pipe write ends rather than the driver's potentially readable, seekable file or connected socket. Parent-side reader threads consume both streams (`:299`, `:318`). This closes the original descriptor access and truncation route. Stream supervision has a separate liveness defect, N2 below. RUN-6 escaping remains explicitly unimplemented; this re-check does not turn that previously acknowledged staged work into a new requirement.

3. **[P1] Unsafe descriptor-cleanup fallback — fixed.**

   The fallback is gone. Preparation requires a successful `close_range(..., CLOSE_RANGE_CLOEXEC)` availability probe (`crates/tide-admit/src/invocation/sandbox.rs:40`–`:47`), and the actual child-side range from fd 3 through `u32::MAX` is checked with `?` (`:120`–`:122`). A host filter rejecting the actual call now fails spawn even if the parent probe succeeded. Existing descriptors above a lowered soft limit no longer escape through a limit-bounded fallback. No new regression test covers this failure path; the original request made the high-fd regression conditional on retaining compatibility fallback, which was removed.

4. **[P2] Lost deadline and seed-on-crash diagnostics — not fixed completely.**

   The ordinary case is restored: receipt configuration selects simulation monitoring (`crates/tide-admit/src/invocation/replay.rs:212`), the default is 300 seconds (`:280`), parsed seed starts reset progress (`:303`–`:307`), and expiry kills and reaps the child (`:329`–`:343`). `tide-std` emits and flushes outermost seed starts again (`crates/tide-std/src/sim/runner.rs:190`; `crates/tide-std/src/test/runner.rs:247`). Timeout and signal failures name the last seed and return 101 (`replay.rs:263`–`:270`).

   But returning that outcome and printing those diagnostics still waits indefinitely for a blocked output-forwarding thread. A simulation can therefore remain hung from the caller's perspective after its evaluation deadline. See N2.

## Folded-in non-blocking finding

- **Degraded-kernel behavioral coverage — not fixed completely; remains non-blocking.** `levels_name_what_the_kernel_cannot_enforce` now covers full/partial reporting for ABI 10, ABI 6, ABI 4, absent Landlock, absent seccomp, and neither (`crates/tide-admit/src/invocation/sandbox.rs:641`–`:649`). This is useful reporting-policy coverage, but it invokes only `level`, not availability probes, construction under older ABIs, or child installation failure. The author's claim of complete resolution is broader than the test. Installation-failure coverage remains explicitly omitted. The separate `none` observation was not claimed folded in; its documented unsupported-platform limitation is unchanged.

## New blocking findings in the fixes

### N1. [P1] Publication lets the child overwrite another run's artifacts

**Evidence:** `crates/tide-admit/src/invocation/sandbox.rs:308`, `:313`, `:319`–`:329`; `crates/tide-admit/src/invocation/replay.rs:260`.

**Failure scenario:** Leave a real `OUT/target/sim/other-run/run.manifest.json` containing a canary. The same unadmitted-child route used by the integration test reads its own allowed context and creates `<context.directory>/other-run/`, optionally containing a replacement manifest, then exits. These are permitted directory/file operations wholly inside its writable grant. The parent subsequently enumerates `other-run`, deletes the existing published `other-run` recursively, and moves the child's directory into that name. This works on a full host without a symlink, a race, or a forbidden syscall. A known previous artifact name is sufficient; the child need not list or read the published directory. Host-path metadata is also expressly available under RUN-3.

The kernel now confines direct writes correctly, but the privileged publisher performs the destructive external write on the child's behalf. Honest recorder name uniqueness does not validate hostile output. This is the remaining form of original finding 1, not an additional unrelated audit finding.

**Smallest sufficient direction:** Make publication destinations belong to the harness's run namespace, independent of arbitrary child directory names, and never delete another run's namespace. Enforce that ownership in the parent. Add a real-directory canary regression in which the child deliberately chooses another run's published name and verify the old artifact's bytes survive.

### N2. [P2] Blocking output forwarding prevents the deadline from completing

**Evidence:** `crates/tide-admit/src/invocation/replay.rs:309`–`:311`, `:325`, `:348`–`:353`, and `:264`–`:269`.

**Failure scenario:** Run a simulation with stdout connected to a pipe whose reader stays open but pauses consumption, while stderr is drained. Let the simulation announce seed 47, then emit enough newline-terminated stdout to fill that external pipe and keep running. The stdout reader blocks in `sink.write_all`, so it also stops reading further seed starts. The supervisor reaches its deadline and kills/reaps the test child, but then blocks forever in `out.join()`: killing the child cannot unblock a write to the separate external stdout pipe. `supervise` never returns, the driver never emits the timeout/seed diagnostic on the available stderr stream, and no failure exit status reaches the caller. The same join can withhold a signal diagnostic after a crash. This is a dependency cycle with a consumer waiting for completion before resuming output reads, and an indefinite stall even without that cycle if the consumer remains paused.

The previous simulation monitor consumed stdout for progress without synchronously forwarding each line to the driver's external sink (`crates/tide-std/src/sim/runner.rs:142`–`:155`). The newly added forwarding introduces this coupling. Two reader threads prevent the usual child stdout/stderr buffer deadlock when external sinks keep accepting data; they do not make these joins deadline-safe.

**Smallest sufficient direction:** Decouple pipe draining and seed observation from potentially blocking external forwarding, and ensure timeout/crash completion does not wait indefinitely for a forwarder. Add a bounded regression with stalled stdout, drained stderr, and a short evaluation limit that verifies both the failure diagnostic and driver termination. Preserve ordinary output bytes when fixing this.

## Regression-test sensitivity

These are source-level counterfactual checks, not executed mutations:

- **Extended kernel-denial test:** The direct control verifies both planted-path reads succeed and redirected stdout can be truncated (`crates/tide-admit/tests/build.rs:4907`–`:4912`). Restoring the original shared writable grant makes the planted `sim` and `other` reads succeed, contradicting the `EACCES` assertion (`:4929`). Restoring inherited stdout allows `ftruncate(1, 0)` and destroys `kept`, contradicting both `:4930` and `:4932`. Removing no-follow link replacement leaves `target/sim` a symlink, contradicting `:4936`. These additions detect their specific old failures. They do not test N1: `other-run` is behind the planted symlink rather than a real published sibling, and the child never creates an artifact directory to attack publication.
- **Timeout/signal test:** The short-limit spin requires the timeout message and seed 47 (`:4957`–`:4961`); removing seed emission/observation fails the seed assertion, and removing the deadline while retaining option parsing leaves the call hung. The 120-second assertion runs only after `.output()` returns, so it is not itself a hang guard; the outer suite timeout must stop that mutation. Removing the signal report fails `:4964`–`:4965`. These checks exercise the actual driver-run simulation path. They capture both streams continuously and cannot detect N2.
- **Level test:** Its assertions detect wrong full/partial strings for the supplied mechanisms (`sandbox.rs:641`–`:649`). The old inline logic already made those decisions; extracting it is not a behavioral fix and these assertions do not prove degraded installation or failure handling.
- **Required close_range:** No new test exercises an unavailable/denied call or the high-fd case. The checked, unconditional child call and deletion of the fallback resolve the original source defect; unavailable-host behavior remains unexecuted in this review.

## Reasoning

The amendment correctly narrows Landlock grants, removes inherited host stream descriptors, and fails closed on descriptor cleanup. Those fixes are meaningful and the new canaries distinguish them from the previous implementation. Approval is blocked by the publisher's trust in child-selected destinations and by supervision waiting on externally blocked forwarding before delivering a deadline/crash outcome. Both defects are introduced by the remedies for the original findings and affect the edited v0 implementation, so they belong in this narrow re-check under GOVERNANCE.md rather than a later-tier backlog.

`git diff --check 60a40e6 9be9d30` passed. No repository files were changed; only this requested report was written outside the repository.

Not checked: cargo, conformance, runtime probes, executed test mutations, or independent verification of the author's focused-run results; full suites belong to the coordinator.
Remaining risk: child-controlled publication can replace other runs' artifacts, and blocked forwarding can withhold timeout/crash completion. This candidate is not approved.

DONE
