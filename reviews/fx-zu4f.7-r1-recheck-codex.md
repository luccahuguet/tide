Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check 9a3ade4

## Verdict

**changes required**. The five original findings have been addressed within this re-check's scope. Two blocking defects remain in the additional changes made by the re-audit, below. Neither requires another search or a simulator run.

Candidate: `9a3ade4caa56ea7a8dceb7268957722c3ecdad7d`, based on `1df19d2f4db63aea56e6d1bbfbe41a7c5d37a8ea`. SHA-256 of `git diff 9ecd0e9 9a3ade4`: `42486c11ac86faea12d14e3aae15e0d2dfc2246678350b3451749ade916dace2`. Whole candidate diff from base: `96d022ce59288bc83d1f9e3c296a8f816afcda7c7b5aa69b51d80aae00537317`.

I checked only the r1 fixes, the related E1/E2 re-audit changes, the requested disclosures and their consequences. The builder report and audit table were treated as claims. No frozen searches were rerun; external reads in this re-check used GET only. This report is the only write.

## Original findings' status

1. **Fixed — incorrect E1 decisions.** `research/39-std-concurrency-search.md:236`, `:256`, `:192`.

   Flume#74 now passes E1 on the named public commit and fails E2 because its thread reports a hypothetical problem from reading. Rayon#938 now passes E1 on the maintainer's explicit attribution to #740 and fails E4 on its observed Miri memory-unsafety report. A fix predating an issue about a fork that lacks that fix does not violate E1; E2's issue-body alternative has no pre-merge-date restriction. Parking_lot#306 now fails E1 because the earlier upgrade fix is not identified as the fix of this withdrawn report. The matching query rows and duplicates were updated.

2. **Fixed — four incorrect E2 exclusions.** `research/39-std-concurrency-search.md:138`, `:191`, `:195`, `:211`.

   Rust#125526, parking_lot#346 and #258 now pass E2 and E3 and fail E4 on their observed memory-unsafety symptoms. The #346 description correctly says `Sync`. Dashmap#201 now passes E2 on the reproducer's statement of unequal cell values: the frozen criterion requires a reported observation, not pasted terminal output. Its E4 exclusion is independently supported by the buggy source at `82c2323`, confirmed by GET: shards use `parking_lot` locks over `hashbrown` tables, both external crates. The record also identifies the unrepresentable plain-memory race.

3. **Fixed — crossbeam#993's first failure.** `research/39-std-concurrency-search.md:166`.

   It is counted at E4 under the registered memory-unsafety-symptom exclusion. E5 is explicitly unassessed; the ordering-only fix is an additional observation. Both query occurrences agree, and E5's count is zero.

4. **Fixed — missing symptom quotations.** `research/39-std-concurrency-search.md:98` and the revised assessment rows.

   Independently checked all 34 current E2-pass rows against the saved JSON: every quote is an exact substring of its stated source; the source URL and date match; and the quote, source and date appear in the assessment. Thirty-three quote issue bodies. Flume#44 quotes a 2020-09-15 comment, preceding #69's 2021-02-07 merge. Finding B1 below will require one additional quote when that row is corrected.

5. **Addressed by the requested disclosure disposition — facts verified.** `research/37-external-admission-search.md:21`, `research/38-external-concurrency-search.md:83`, `research/39-std-concurrency-search.md:55`.

   I accept tide-lead's disposition as instructed and do not reopen it. Main's Git history confirms the disclosed UTC commit times and first ledger appearances:

   | Record | Criteria first committed | Protocol hash first committed | Earlier local registration |
   | --- | --- | --- | --- |
   | 37 / fx-zu4f.5 | `42181e3`, 05:52:57Z | same commit | br 05:49:42Z |
   | 38 / fx-zu4f.6 | `7863e8f`, 13:04:01Z | `fe99212`, 13:22:08Z | br 13:10:48Z |
   | 39 / fx-zu4f.7 | `afb17ea`, 13:27:20Z | `951e6c3`, 13:41:44Z | br 13:33:52Z |

   The first search-request times stated in the records are respectively 05:50:10Z, 13:11:15Z and 13:34:14Z. I checked `git show`, first hash additions with `git log main -S`, and the corresponding ledger entries/comments. All three embedded protocols still hash to their registered digests. Records 37 and 38 landed in `687c494` and `bd6c51f` before these corrections. Their additions accurately distinguish local registration from later Git recording and identify themselves as corrections. Narrative uses of “pre-registered” or “registered before” point to the disclosure, or sit beside it; the preserved protocol text remains verbatim. No factual correction to those disclosure paragraphs is needed.

## Other re-audit decisions and the POST disclosure

Except for B1 below, the additional E1/E2 changes are supported by the saved threads and targeted GETs:

- **rust#96706:** the merged, connected #122565 explicitly demonstrates the same interleaved panic output before its change and describes preventing that output afterward. Taken together, the issue connection and the PR's own fix description support E1(2); that alternative does not require a literal issue-number string in the PR body. Its observed output passes E2; the unmapped OS write excludes it at E4. The record discloses the interpretation rather than inventing a closing keyword.
- **rayon#738:** the thread identifies #740 as fixing a different crash. Its proposed connection to #934 remains conditional, and #934 explicitly names #913 and #929. Under the frozen “mention alone” narrowing, retaining #738 at E1 is defensible. Its observed segfault would independently exclude it at E4 if the attribution were accepted.
- **crossbeam#1108:** the linked Miri-side resolution supports E1 without a same-repository restriction; the reported Miri error passes E2 and the explicit symptom narrowing excludes it at E4.
- **crossbeam#1020:** its throughput/cache-contention report supplies none of E2's wrong-behavior symptoms. Moving it to E2 is correct.
- **crossbeam#397:** the thread names only issue #388 and “fixed in master”, not a directly linked fix PR/commit. The direct-link requirement supports E1 failure.
- **flume#2:** the maintainer names merged #24 as addressing it, but it reports a requested feature rather than an observed failure. E2 is the correct first failure.
- **once_cell#1:** the first named fix is `0af68fb`. The later forced-reordering experiment uses modified code and is posted after that fix; it cannot supply the required pre-fix-comment observation. E2 failure is supported.
- **rayon#827:** the report persisted after #800's release, and the later resolution credits an advisory rather than a linked PR or commit fixing this issue. E1 failure is supported.
- **flume#44:** the record preserves the hedged attribution in #69 and distinguishes it from #70's test re-enablement. Its pre-fix observed CI hang is now quoted.

The GraphQL disclosure at `research/39-std-concurrency-search.md:102`–`:103` is adequate: it identifies the single query, its POST transport, the GET-only violation, the issue lookup it served, and the absence of a state change. It does not pretend that a read-only GraphQL request was a GET or conceal it inside the frozen searches. The preceding GET account is bounded to the original screening period. This disclosure does not authorize further POSTs.

## New blocking defects introduced by the fixes

**B1 — Do not require a confirmed cause at E2.** `research/39-std-concurrency-search.md:280` (also query outcome at `:871`).

Rayon#343 is newly rejected at E2 because the reported Servo CI timeouts are linked to the spinning problem only as a hypothesis. But its saved [2017-05-24 pre-fix comment](https://github.com/rayon-rs/rayon/issues/343#issuecomment-303666382), confirmed by GET, reports “more tests timing out or having intermittent failures on CI”. This predates #348's 2017-05-30 merge and is an observed symptom of exactly the kind E2 lists. E2 does not require a proved causal connection to the proposed fix; cause and concurrency shape belong to E3. The current row acknowledges the timeouts but adds an unregistered causal-certainty condition to exclude them. The bounded instrumented spin is not the only reported evidence.

**Minimum fix:** pass E2 and quote that pre-fix comment. Continue in order through E3; with the existing concurrency mechanism accepted, the already-documented Condvar path excludes the issue at E4. Update its query row, first-failure counts and derived text. Restoring E4 would change the partition to E1 74, E2 11, E3 8, E4 27, E5 0, with 35 E2-pass quotations. No model or new search is needed.

**B2 — The new “Seven E4 rows” observation count is false as written.** `research/39-std-concurrency-search.md:294`–`:296`.

The sentence counts rows whose observed symptom is only a Miri/sanitizer report or segfault, but there are more than seven in the revised table. Its five examples (#993, rust#125526, parking_lot#258/#346, rayon#938), plus crossbeam#663, crossbeam#1207 and dashmap#213, already give eight; all eight have those recorded symptoms. More rows qualify as well. Seven appears to count rows whose stated E4 ground specifically invokes the symptom exclusion, which is a different assertion from counting their observations.

**Minimum fix:** replace the exact count with “Several E4 rows”, or explicitly count rows classified under the symptom narrowing rather than rows observed through those tools. This is a factual wording correction, not a request for another audit.

## Verification and limits

The revised assessment partition mechanically reconciles as 42 scope + 120 screened, with 74 E1 + 12 E2 + 8 E3 + 26 E4 = 120 before B1's correction. All 215 query rows and all 162 assessment rows match the revised assessment data, the original saved response order and duplicate references. These checks do not validate B1's classification. `git diff --check 9ecd0e9 9a3ade4` passes. HEAD and tracked worktree state remained unchanged during this review.

Not checked: no fresh full review, Rust builds, simulator/replay runs, Opus review or landing. Approval remains blocked only on B1 and B2 above.

DONE
