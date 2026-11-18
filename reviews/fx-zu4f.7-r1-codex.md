Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 9ecd0e9 (docs)

## Verdict

**changes required**. High confidence in the classification findings below. The search inventory is correct, but several screening statements contradict the saved evidence or the registered protocol. The published first-failure counts therefore cannot stand as an audited result.

## Scope

Read-only supervisor review of `research/39-std-concurrency-search.md`, against the Frozen search criteria in `br --no-db show fx-zu4f.7`, `br --no-db comments fx-zu4f.7`, and the registered `target/work/protocol.md`. I read the builder report as claims, checked the saved requests, responses, assessments, issue threads, timelines and fix dossiers, and made targeted GitHub GETs. I did not rerun any search or change the candidate, Git history or Beads. This review file is the only write.

- HEAD: `9ecd0e9e270d87f10a3f3a79aa8f2b62af56f431`, branch `fx-zu4f.7`.
- Base: `1df19d2f4db63aea56e6d1bbfbe41a7c5d37a8ea`.
- SHA-256 of `git diff 1df19d2..9ecd0e9`: `1b7f5c0dcaf0d8e7131d6aad7754a14ab0b4cab3e9c831e7e7722abed211ffb0`.
- Protocol SHA-256: `5038d09ba8b6441870ab56768f3b016370c61db4933a33ef36503f63410d39e4`.

## Blocking findings

1. **Correct the E1 decisions against the actual linked fixes.** `research/39-std-concurrency-search.md:219`, `:239`, `:175`.

   Three concrete counterexamples invalidate the current E1 partition:

   - **flume#74**, marked E1 with “No fix linked”: the saved `issues/zesterer__flume__74/comments.json` contains the maintainer's [explicit fix attribution](https://github.com/zesterer/flume/issues/74#issuecomment-817757114) to `3f8a925983d074ce9cec2a4b26b5371ce365092f`. A read-only GET of that [public commit](https://github.com/zesterer/flume/commit/3f8a925983d074ce9cec2a4b26b5371ce365092f), “Capped exponential backoff”, confirms the change to `wait_lock` in `src/lib.rs`. This meets registered E1(3). It must proceed to E2; the issue's hypothetical scheduling explanation alone does not establish an observed pre-fix failure.
   - **rayon#938**, marked E1 because the fix supposedly lives elsewhere: the saved thread [names the missing fix as rayon#740](https://github.com/rayon-rs/rayon/issues/938#issuecomment-1139893076). [PR #740](https://github.com/rayon-rs/rayon/pull/740) is public and merged (2020-06-15), and its dossier is already saved for rayon#768. E1 does not impose a same-repository-fix restriction; in any event, #740 is in the searched repository. The fork/rebase explanation is not evidence that no qualifying public fix is linked. Re-screen this issue, including its actual scope and memory-unsafety symptom.
   - **parking_lot#306**, marked E4 after E1 allegedly passes: the saved [comment mentioning c866ba80](https://github.com/Amanieu/parking_lot/issues/306#issuecomment-995004712) describes a previously fixed upgrade problem and says the current report could not be reproduced. The reporter subsequently [withdraws the report as a possible false positive](https://github.com/Amanieu/parking_lot/issues/306#issuecomment-997323598). The saved closing event has no commit. This does not name c866ba80 as a fix of the reported failure, as E1 requires; a mention alone is explicitly insufficient. On the supplied evidence, its first failure is E1.

   **Minimum fix:** correct these assessments using the existing threads and targeted fix reads, then update every corresponding query row, duplicate cross-reference, count and summary. Do not rerun the searches or add a post-search same-repository restriction to E1.

2. **Four E2 exclusions deny failures that the issue bodies explicitly report.** `research/39-std-concurrency-search.md:121`, `:174`, `:178`, `:194`.

   These are not merely type arguments inferred from source:

   - [rust#125526](https://github.com/rust-lang/rust/issues/125526) supplies a reproducer and reports “compiled successfully and miri reported a data race”.
   - [parking_lot#346](https://github.com/Amanieu/parking_lot/issues/346) supplies concurrent code and explicitly says it was tested with Miri. The faulty trait is `Sync`, whereas the assessment calls it `Send`.
   - [parking_lot#258](https://github.com/Amanieu/parking_lot/issues/258) supplies the PoC's output, including `Return Code: -11 (SIGSEGV)`.
   - [dashmap#201](https://github.com/xacrimon/dashmap/issues/201) supplies a concurrent increment reproducer and identifies unequal cell values as the data race's observed effect.

   These bodies are present in the saved `issue.json` files and were confirmed by GET. Registered E2 expressly accepts observed model-checker failures, crashes and wrong values. A type-bound root cause does not cancel that evidence. Each issue also has a linked merged fix in its saved dossier (#125527, #349, #262 and #202 respectively).

   **Minimum fix:** mark E2 satisfied, quote the reported symptom, and continue through E3-E5 in order. Exclude an issue at its actual first failing criterion, with the specific faithful-model limitation; do not assume that an incorrect E2 explanation is harmless because a later exclusion is likely. Correct the `Sync` description and regenerate the affected rows and counts.

3. **The sole E5 result already fails the registered E4 narrowing.** `research/39-std-concurrency-search.md:149` (also query rows at `:413` and `:452`).

   Crossbeam#993's recorded symptom is a Miri data-race report. The embedded, digest-verified protocol at `:1219`–`:1226` explicitly excludes a bug whose observed wrong behavior is memory unsafety, including a Miri or sanitizer report. The record nevertheless marks E4 satisfied by reducing the path to “one atomic load” and counts it at E5. A smaller affected operation does not waive the symptom exclusion. The issue's repro also includes epoch guards, pointer dereferences and deferred destruction; this is not evidence for an unrestricted one-load model.

   **Minimum fix:** classify #993 at E4 under the registered symptom exclusion, leave E5 unassessed as its first-failure rule requires, and update both query occurrences and the counts. The ordering-only change in #995 can remain a clearly labelled additional reason, not its counted first failure. Preserve the frozen protocol.

4. **Record the symptom quotations required by the protocol.** `research/39-std-concurrency-search.md:120`, `:150`, `:214` (representative rows); requirement at `:1099`–`:1105`.

   The protocol requires the recorded E2 symptom to be a quote. Many assessments instead say only `E2 ✓`, “a reproduction that hangs”, or “TSan”. Those paraphrases do not fulfill that registered evidence requirement. This matters here because several supposed absence-of-observation judgments demonstrably overlooked the issue's actual observations (finding 2). This is an evidence omission, not a style preference, and is separate from the property-before-model rule; no model is needed to record E2 evidence.

   **Minimum fix:** for each assessment that reaches and passes E2, add a short exact symptom quote and identify its issue-body or pre-fix-comment source. Apply E2's timing condition to comments. Keep later checks unassessed where E1 or E2 fails.

5. **The required pre-search protocol commit is not evidenced, and the available history shows a different procedure.** `research/39-std-concurrency-search.md:8`–`:11`, `:56`–`:65`, `:857`.

   The bead's Frozen search criteria require the operational protocol to be committed as well as have its digest recorded before candidate examination. The digest registration did precede the search, but `target/work/protocol.md` is ignored and untracked. The branch reflog goes from creation at 13:27:29Z directly to `5e58a538e6a0b95d0451d16106e7092838400e56` at 13:54:05Z, then the rebase; that original candidate commit first adds the embedded protocol with the completed record. The first search began at 13:34:14.701Z. No earlier protocol commit or owner-approved exception is supplied by the bead comments, ready marker or builder report. The later brief's hash-only procedure does not itself amend the frozen requirement.

   **Minimum fix:** supply the identity and pre-search timestamp of a commit containing the registered bytes, if one exists outside this branch. Otherwise disclose the missing pre-search commit as a protocol deviation and obtain/record the owner's disposition before claiming full compliance. A commit made now cannot retrospectively satisfy that requirement. This finding does not dispute that the registered digest predates the saved searches, and does not call for rerunning them.

## Non-blocking findings

None. No style changes requested.

## Reasoning

The following checks passed on the unchanged candidate in this Linux worktree on 2026-11-18:

- **Query execution and transcription:** independently checked all 56 `request.json` files against the seven registered qualifiers and eight exact terms, including quoted phrases, `is:issue is:closed created:<=2026-11-18`, `per_page=10`, `page=1`, and the command's absence of `sort`/`order`. All 56 raw responses are HTTP 200, their bodies equal the corresponding saved JSON, and `incomplete_results` is false. Each result length is `min(10, total_count)`. Times are in registered order; the shortest end-to-next-start gap is 3.016 seconds. There are exactly 168 search artifacts (one request, raw response and body per query), with no retry artifacts. This supports the recorded single execution of each query; I made no search requests.
- **Every result and duplicate:** all 215 rendered query rows match the saved response's order, URL and title, including their assessment outcome and duplicate locator. The 162 assessment rows exactly match the saved assessment data and occur in first-appearance order. All 162 unique URLs are covered once; 53 later rows refer back to them, involving 50 repeated issues. The seven repository metadata rows match the saved metadata. These are transcription checks, not endorsement of the incorrect judgments above.
- **Scope:** reviewed the 42 scope exclusions (26 rust, five crossbeam, 11 rayon) against the registered readings and saved issue/thread evidence. The exhaustive R1 list, named R2 crates rather than pre-split umbrella code, and R7 exclusion of iterator/futures/test-harness reports are pre-registered narrowings. I found no demonstrated eligible issue wrongly removed by those 42 exclusions. The report's stale concern about the link to research/38 does not apply to this rebased candidate: research/37 and research/38 both exist here.
- **Screening:** inspected E1 fix hints and timeline links across the 75 E1 rows, the E2/E3 assessments, and the E4/E5 dossiers, with targeted GET confirmation of the disputed fixes and observations. Crossbeam#1102's E4 exclusion is supported independently of its asserted line budget: at buggy base `4efd6ad9853d713fc609640a083c752aeaf7abf6`, `SyncWaker::disconnect` holds `inner` at `waker.rs:266` while `inner.disconnect()` calls `entry.cx.unpark()` at `:165`. Both the saved source and a source GET show this. Registered E4 forbids precisely that lock-across-effect structure; Tide's `Shared::with` takes a `Pure` closure. The other principal E4 mechanisms—raw/OS lock paths, async/wakers, reclamation and memory-unsafety symptoms—are permitted exclusions under the frozen narrowing. Findings 1–3 identify cases where the order or prerequisites are wrong.
- **Arithmetic:** the saved partition reconciles mechanically: 215 raw = 162 unique + 53 duplicates; 162 = 42 scope + 120 screened; 120 = 75 + 11 + 12 + 21 + 1. Eighteen queries returned no rows. These equalities pass, but the E1–E5 numbers must be recomputed after correcting the judgments. I have not substituted speculative corrected totals before the remaining criteria are assessed for the promoted rows.
- **Framing and digest:** the negative is expressly limited to this registered sample, gives no catch rate or population inference, and states that the simulator was never asked to find these bugs. The protocol bytes embedded after the delimiter are byte-identical to `target/work/protocol.md` and hash to the registered `5038d09b…39e4`; the final separator is outside those bytes. Registration at 13:33:52Z precedes the first request. `git diff --check 1df19d2..9ecd0e9` passes, and HEAD/diff identity remained unchanged throughout review.

The minimum approval patch corrects the screening record and its derived tables/counts, supplies the required symptom evidence, and resolves the pre-search-commit discrepancy. It need not change Tide, add a case study, build a model or expand the search. I found no demonstrated eligible candidate, but the present false explanations cannot serve as proof that each exclusion followed the frozen criteria.

Not checked: no Rust builds, simulator runs or replay tests; no Opus review or landing. Those checks were outside this docs-only supervisor pass. The research verdict remains unapproved until the findings above are resolved.

DONE
