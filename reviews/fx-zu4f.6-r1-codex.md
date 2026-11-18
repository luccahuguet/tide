Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 99b827c (docs)

## Verdict

**changes required**. High confidence in the table checks and evidence discrepancies below. No portable eligible candidate was identified, but several first-failure classifications and one fix attribution need correction.

## Scope

Read-only supervisor review against the bead body's Frozen search criteria and the registered protocol. Read `br --no-db show fx-zu4f.6` and `br --no-db comments fx-zu4f.6`, repository instructions and GOVERNANCE.md, the candidate diff, protocol, builder report, all eight saved request/raw-response/body sets, repository metadata, and the issue/comment/timeline/fix evidence for all 32 assessments. Treated the builder report as claims. Supplemented saved material with read-only GitHub REST GETs for selected issues, PR metadata, patches, reviews, and source. No search was rerun.

- HEAD: `99b827cde89be96d295a7f61d057fb7648c1d961`.
- Base: `7863e8f54110a63e5e435999a0a52182d433704e`.
- SHA-256 of `git diff 7863e8f..99b827c`: `b4cd3fc2648a050de403fa8c6b73386ddf7b60118738398b7a6b26084d3e2513`.
- Protocol SHA-256: `0a02f5be72ea01ca887498fc5d938a35cfc9c25d1de3fa728a89ca5a46576308`. The embedded block is byte-identical to `target/work/protocol.md`, including its final newline.

Only this requested review file was written. No candidate edits, commits, upstream posts, or `.beads/` writes.

## Blocking findings

1. **A15 rejects a race that the issue and fix explicitly name.** `research/38-external-concurrency-search.md:164` says no interleaving is named and fails E3. The saved [issue #8520](https://github.com/tokio-rs/tokio/issues/8520) describes a peer closing the socket between the application's decision to shut down and the syscall. Independently fetched [merged PR #8526's patch](https://github.com/tokio-rs/tokio/pull/8526/files) documents prevention of a race between shutdown and peer closure. An errno normalization can implement a concurrency fix. Accepting the analogous peer-reset/shutdown ordering in A24 while rejecting this is inconsistent.

   **Minimum fix:** mark A15 E3 met, then document E4 failure for the socket/kernel/async path with source evidence. Update Q7.4 at line 426, counts, and derived prose. The candidate remains ineligible; its first failure changes.

2. **A19 and A29 overlook observed symptoms in pre-fix comments.** `research/38-external-concurrency-search.md:179` and `research/38-external-concurrency-search.md:212` reject these as feature requests. E2 permits observed wrong output or behavior in a pre-fix issue comment; a feature-request label is not a veto.

   - A19: the saved [2026-05-28 comment](https://github.com/tokio-rs/tokio/issues/4422#issuecomment-4567054828), before #8381 merged on 2026-10-21, supplies a program and output showing the ordinary listener initially exposing mode 755, although 600 is required; the alternative creates it at 600. This is observed output, not merely a hypothetical vulnerability.
   - A29: the saved and independently fetched [2026-04-03 comment](https://github.com/tokio-rs/tokio/issues/7998#issuecomment-4185076034), before #8030 merged on 2026-04-12, describes the reporter's current loop and the immediate return from `pipe.readable()` after `splice` leaves cached readiness set. The assessment does not address that report.

   **Minimum fix:** quote those symptoms, mark E2 met, and continue E3/E4 in order. Record the actual first failures and update Q7.8/Q7.18 at lines 430/440, counts, and derived prose. The socket/pipe/async paths still provide E4 exclusions if E3 passes; no models or wider search are requested.

3. **A26's E3 pass lacks the second concurrent activity required by the protocol.** `research/38-external-concurrency-search.md:204` calls this a timer race and proceeds to E4. [PR #7882's patch](https://github.com/tokio-rs/tokio/pull/7882/files) describes successive checks within one `Mock::poll_write`: `write()` sees a Wait, then `remaining_wait()` sees it expired. Source at PR base `f21f2693f02aec9a876ac2bd21566c85e15b682e`, `tokio-test/src/io.rs:292` and `:316`, shows `action()` comparing `Instant::now()` with `self.waiting` and removing the action itself. A concurrent timer task need not mutate the state. The current-thread reproduction pauses in the debugger to advance time. The fix names preemption or other delays, not two concurrent activities over shared state. Elapsed time between checks does not satisfy the registered narrowing, just as crossing a clock second does not for A3.

   **Minimum fix:** classify A26 at E3, leaving E4 unassessed, and update Q7.15 at line 437 and dependent counts/prose. Alternatively supply source/thread evidence of a second concurrent activity causing this specific failure; the current evidence does not identify one.

4. **A22 attributes the original starvation fix to a later regression correction.** `research/38-external-concurrency-search.md:185` names wasmtime #13511 as the fix. Its saved body says it corrects an accidental bug introduced by #13085 and closes #13507. Independently fetched [#13085](https://github.com/bytecodealliance/wasmtime/pull/13085), merged 2026-04-14, adds the missing yield and fixes #13040, whose body identifies Tokio #8034 as the investigation that found the starvation. [#13511](https://github.com/bytecodealliance/wasmtime/pull/13511), merged 2026-06-02, makes that yield cancellation-safe. Its [review](https://github.com/bytecodealliance/wasmtime/pull/13511#pullrequestreview-4410754216) asks to ensure #8034 is still fixed; the subsequent approval confirms it remains passing. Thus #13511's diff is not the original starvation fix.

   **Minimum fix:** distinguish #13085's fix, #13511's follow-up, and Tokio #8305's test re-enablement. Show which actual bug-fixing change meets the registered E1 direct-link/named-fix rule; a cross-reference to the follow-up alone does not establish that. If no qualifying E1 link is established, record E1 as the first failure and update Q7.11/counts. Do not widen E1 to accept an unregistered indirect-link rule.

## Non-blocking findings

- `research/38-external-concurrency-search.md:43`: clarify the disjoint filter labels as “primary language Rust, fewer than 1,000 stars: 125” and “non-Rust, also fewer than 1,000 stars: 1.” All 126 filtered unique issues are under the threshold. **Minimum clarification:** add “Rust” to the 125 bucket's label; the total and filter outcomes already reconcile.

## Reasoning

The pool is correct. Local Python assertions checked each saved request's exact registered query, GET method, `per_page=20`, `page=1`, and absence of sort/order parameters. The eight raw responses are HTTP 200, match their parsed bodies, have `incomplete_results=false`, and contain 20 items. Request intervals run consecutively from 13:11:15.323Z to 13:11:24.604Z on 2026-11-18. There is one saved request/response set per query, consistent with one run each and no retry or replacement. Saved artifacts cannot prove the absence of an unsaved request; searches were not rerun.

All 160 table rows match the saved ordered URLs, state/reason, stars, and language. Independently applying the post-cap filters yields 158 unique issues, two duplicates, 119 repositories, 126 filtered unique issues (125 Rust issues plus one Python issue, all below 1,000 stars), and exactly the 32 saved screened issues. No Eon/Tide/fork exclusion was missed and no replacement was added. Saved comment counts match issue metadata for all 32; all saved timelines fit their single pages (largest: 87 events).

The declared first-failure labels mechanically sum to 9 E1 + 3 E2 + 5 E3 + 15 E4 = 32. That proves arithmetic, not classifications: findings 1–4 must be resolved before accepting those category counts. The other E1 exclusions have no qualifying linked fix in the saved threads/timelines; for example, A27's linked cloud-hypervisor #7971 is independently confirmed unmerged. A24's fetched patch keeps the test and rejects unexpected errors while accepting a legitimate platform reset outcome; it is not merely a skip/retry.

The hardest E4 exclusions stand under the frozen mappings. A1's fetched buggy `download.rs` matches the saved bytes: the affected cache/launch path includes async network/download and locking operations, permission setting, and driver execution. A32's two saved source files match fetched bytes at #1963's base: dispatch uses `serde_json`, scans use `egui::Context` between changes and observations, and the worker executes the engine read. These are on the trigger/path, beyond the setup-only exception. Its fix waits for the worker's completed read rather than skipping or retrying. A11's `layerstack.rs` at #692's base is byte-identical to the saved file at `67310175`; its transaction facade delegates query/execute to SQL backends. Replacing that database path with Shared exceeds the registered mapping. Neither near miss establishes a portable eligible candidate.

The framing correctly limits the conclusion to this sample, reports no model/property runs, gives no rate or general Rust claim, and plainly retains the Eon studies as the only seeded-catch/replay demonstrations. The zero eligible/caught outcome remains supported by the portability boundaries checked here, subject to correcting the screening record.

Validation: saved-evidence inspection, local Python assertions, selected GitHub GETs, and `git diff --check 7863e8f..99b827c` (passed), on the recorded HEAD in this Linux worktree. No Rust builds/tests, upstream reproduction, or simulation/replay runs were performed: this is one research Markdown file with no model. Runtime behavior is not newly proved by this review.

DONE
