Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 1a32e66 (record)

## Verdict

**Changes required.** The arithmetic and commit identities check out. The record does not yet support its claimed completed first-failure screening: required reading was abbreviated, two sampled judgments contradict their evidence, required source/count records are missing, and one operational deviation is omitted. No new catch or privacy breach was established by this review.

Confidence: high on enumeration, arithmetic and the cited discrepancies; limited to sampled semantic judgments.

## Scope

Read-only supervisor review of HEAD `1a32e669ffeb1a513ec1cc2f6c19a81937c8eaa4`, specifically research/44-fx-zu4f.10-concurrency-round.md and research/protocols/fx-zu4f.10-record/arm-b-ledger.tsv, against the registered protocol, bead/comments, working reports, judgment files, retained evidence, local frozen clones and pinned arm-A archives. Candidate files were not edited. This requested review report is the sole output write.

No build, upstream execution, network request, upstream contact, bead mutation, commit or delegation. OpenTimestamps anchoring was checked against the retained registration/record identities, not independently verified against Bitcoin.

## Blocking findings

1. **Filtered reading is counted as completed protocol screening.** research/44-fx-zu4f.10-concurrency-round.md:323 and :583; protocol:272 and :277.

   Protocol §3 requires the inventoried evidence to be read in full before an E2-or-later failure. The decision files explicitly mark 40 such rows as using a compact reader: 9 E2, 28 E3 and 3 E4. These include substantive human threads, for example B09.028's issue 2462 and B10.064's PR1821, as well as bot transcripts. The record's general claim of having read the inventory before every later failure, followed by an optional reviewer re-read, does not satisfy that requirement. There is no registered exception allowing a filtered view to settle a failure.

   The description of the filter is also inaccurate. target/work/read-inv.py's brief mode retains at most three initial lines/300 characters, then **at most six** keyword hits, each truncated to 160 characters. It does not show every matching line as record:586 says. target/work/report.md:56 also describes a separate keyword-sentence viewer, sym.py, and full reading only when its summary did not settle the failure; the record does not bound that additional affected set.

   **Fix:** audit and enumerate the affected rows, finish the required unfiltered reads and any resulting rejudgments, then update counts and reading provenance. Until that work is done, identify these screens as incomplete rather than completed negatives. Disclose the actual truncation and the additional summary-reader method. This requires evidence reading, not a simulator run or a relaxed criterion.

2. **B15.047 fails E1 on a factually false premise.** research/protocols/fx-zu4f.10-record/arm-b-ledger.tsv:3243, row B15.047; record:363 (plotters totals); protocol:288 and :294.

   The row says “API improvement, no malfunction stated.” Its saved associated PR43 body says the conversion to failure::Error failed, includes an E0277 compiler diagnostic identifying Rc<rusttype::Error>, and says replacing Rc with Arc fixes it. The exact first-parent patch changes FontLoadError and its construction from Rc to Arc in src/style/font/ttf.rs. This is a reported failure and its repair in non-test code, not merely an inferred API improvement. It contradicts both registered E1 and the record's stated treatment of quoted build errors.

   **Fix:** assess this row past E1 using the required complete inventory, retain the actual diagnostic with its source, determine the first genuine failure and update the ledger, plotters row, first-failure totals and distinct-site/pass counts. This review does not claim the row passes E3 or becomes eligible; the present E1 count is nevertheless unsupported.

3. **B09.162 uses successful fixed-version verification as the old bug's symptom and omits its linked bug report.** ledger:2037, row B09.162; record:325 and :329; protocol:272 and :299.

   The E2 quote is the statement that the second build failed because clang_sys could not find libclang. PR1809 explicitly places that failure **after** changing the dependency to the proposed fix and deliberately clearing LIBCLANG_PATH; it is the expected validation outcome. The pre-fix observation is instead that the environment changes trigger no rebuild. The quoted fixed-version error does not establish the first parent's bug symptom under E2. PR1809 also explicitly closes issue 1766, but this row's six-entry inventory contains only the commit and PR sources: no issue-1766 body/comments. Those endpoints are absent from the retained evidence cache too.

   **Fix:** read and inventory the linked bug report and complete the E1/E2 assessment. Use an actual symptom of this matched pair if the evidence supplies one, otherwise stop at the appropriate earlier failure; unavailable required evidence remains unresolved. Do not retain the expected post-fix error as an E2 pass. Recount after the decision.

4. **Required path/core counts and source scope are not recorded.** record:375 (E4 table) and :484 (arm-A table); protocol:316, :457 and :591.

   E4 requires exact files, functions and the nonblank/non-comment line count of the affected path's union, including required helpers. Nine B E4 rows have no numeric path count in either the ledger judgment or its retained decision; B23.129 records a local 49-line function plus a static. The tables give reasons, not the required per-row source/count record. Arm A similarly gives only “Under 400 lines” for tracing-log and env_logger and no numeric union for quinn-proto. Its private maps retain bounds or partial totals instead of complete counted unions. Hashing those maps does not supply the omitted information in the public record.

   The exclusions themselves have recognizable registered grounds; this is not a demand to invent a model or use compiler reachability. It is a missing deliverable required by §§4, 6 and 8, including the counts the review was asked to check.

   **Fix:** supply the required source file/function sets and counted unions, including the arm-A roots and setup boundaries. Clearly distinguish measured totals, an enumerated lower bound proving a budget failure, and work left unmeasured; do not present the latter as a completed required count. If required mapping/count work remains undone, disclose that incompleteness. Preserve the registered restrictions.

5. **The arm-B assessment-order deviation is missing.** record:528; target/work/report.md:43; protocol:264.

   The protocol registers pool order and then lexical SHA order. The working report explicitly says builder 4 took darling_core and env_logger before uuid, and builder 5 judged bindgen while the acquisition chain fetched serde_core onward. The public deviations section discusses the order of the two arms but omits this separate change in within-arm assessment order. Lexically assigned row IDs do not establish the chronology of assessment.

   **Fix:** add this disclosed operational deviation, its acquisition-availability reason and the distinction between row ordering and actual judgment ordering. It does not require redoing an otherwise valid judgment or changing the pool.

## Non-blocking findings

- Move the limitations to the opening as §8 requests, reduce their repetition across verdicts and method notes, and define E1-E5 and SC (sequential consistency) at first use. There is no author-directed judgment, discovery-rate claim, predicted seed failure or affirmative “Tide found” claim.
- B09.297 (ledger:2172) retains inventory hash `590f7a8f…`, whereas the current full-inventory.json hashes to `3074f09b…` and omits issue 833. This is recoverable: appending the two issue-833 entries retained in B09.093's inventory, serialized exactly as inventory-b.py does, reproduces the full recorded `590f7a8fd6d14d4548c398dd85b417b760302f6d8ba10ad74871e7407282a9d0` hash. Preserve or document that original inventory reconstruction and subsequent overwrite. I do not treat this as fabricated or missing evidence.
- B09.251 (ledger:2126) has `e2_quote = as B09.065` and no e2_source. The duplicate link resolves to the quote and source in the same ledger, so the evidence is recoverable. Expanding it would make the row self-contained.
- The capability-only table correctly keeps B09.003 and arm-A rank 9 excluded under the frozen subprocess ban. Name/link their already identified follow-up, fx-zu4f.13, in the follow-up section rather than leaving its identity implicit.

## Reasoning

### Independent recount and identities

Plain Python csv/collections/hashlib recount: 5,076 rows, 5,076 distinct IDs, no missing commit, first parent, reason, source or later-criteria field. Ledger SHA-256: `adceed411662fa684b134fd5185129d0f9d5f4b13c6562d100359b14c6aad0f8`, as recorded.

First failures: E1 path 3,779; E1 comment 76; E1 manual 915; E2 113; E3 182; E4 10; E5 0; unresolved 1. Sum: 5,076. Thus E1 total is 4,770. These reproduce the committed judgments; they do not validate the two disputed judgments above.

Per-crate recount follows. E1 combines all three E1 layers; U means unresolved. Every entry matches the public coverage table, including its separate path/comment/manual E1 columns.

| Crate | Rows | E1 | E2 | E3 | E4 | U |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| serde | 138 | 121 | 5 | 12 | 0 | 0 |
| rustls | 346 | 294 | 31 | 21 | 0 | 0 |
| uuid | 142 | 131 | 8 | 3 | 0 | 0 |
| darling_core | 40 | 38 | 0 | 2 | 0 | 0 |
| serde_core | 138 | 138 | 0 | 0 | 0 | 0 |
| tracing-log | 878 | 859 | 5 | 14 | 0 | 0 |
| schemars | 62 | 58 | 0 | 4 | 0 | 0 |
| env_logger | 130 | 124 | 1 | 5 | 0 | 0 |
| bindgen | 318 | 224 | 30 | 63 | 1 | 0 |
| quinn-proto | 297 | 269 | 12 | 12 | 4 | 0 |
| quinn-udp | 297 | 292 | 1 | 4 | 0 | 0 |
| console | 21 | 20 | 0 | 1 | 0 | 0 |
| arc-swap | 53 | 46 | 4 | 2 | 1 | 0 |
| opentelemetry | 335 | 317 | 6 | 11 | 1 | 0 |
| plotters | 94 | 93 | 0 | 1 | 0 | 0 |
| ureq | 133 | 119 | 4 | 8 | 2 | 0 |
| colored | 18 | 17 | 1 | 0 | 0 | 0 |
| regex-lite | 192 | 192 | 0 | 0 | 0 | 0 |
| mockall | 117 | 117 | 0 | 0 | 0 | 0 |
| mockall_derive | 117 | 109 | 4 | 4 | 0 | 0 |
| windows-collections | 45 | 45 | 0 | 0 | 0 | 0 |
| wit-bindgen-rust-macro | 151 | 149 | 0 | 2 | 0 | 0 |
| insta | 155 | 142 | 0 | 11 | 1 | 1 |
| thrift | 859 | 856 | 1 | 2 | 0 | 0 |

There are 4,524 distinct repository/commit pairs; 238 merge memberships representing 218 merge commits; zero roots; 78 duplicate-linked rows (42 E1, 9 E2, 26 E3, 1 E4). The recorded E1-pass count is 305, with 36 duplicate rows removed to give 269 recorded bug sites; E2 passes total 192, E3 passes 10. All duplicate targets resolve.

I checked **all 5,076 memberships / all 4,524 unique matched commits**, exceeding the requested commit spot-check: each is a real commit reachable in its recorded local clone, with the recorded first parent and parent count. With lazy fetching disabled, git log over all refs independently reproduced the exact registered full-message regex memberships for each of the 24 crates. It also reproduced 50,868 enumerated commits across 21 repositories. Every clone HEAD matches its retained manifest pin. All 18 registered artifact digests and the clone-manifest digest match the record.

### Judgment sample

Manual E1 sample (10): B01.001, B02.253, B03.061, B04.024, B07.051, B09.156, B10.081, B14.170, B15.047, B20.043. Checked messages, associated PR bodies and relevant first-parent changes. B15.047 is finding 2. The other sampled E1 exclusions have support in the registered bug-fix requirement; no eligible concurrency candidate was established there.

E2 sample (5): B01.033, B03.103, B09.077, B09.228, B10.127. Their saved evidence supports stopping for absent qualifying observed symptoms. In particular, the h3 flake in B10.127's PR is explicitly unrelated and must not be borrowed as this row's symptom.

E3 sample (5): B06.340, B09.033, B09.162, B09.264, B10.225. The sampled code supports the stated absence of a qualifying concurrency interaction, but B09.162 lacks a valid demonstrated E2 pass and complete required evidence (finding 3). B10.225's linked comment about a hanging channels branch is not evidence of a hang in the matched first parent.

All ten E4 rows were checked at the restriction/evidence level: B09.003 (subprocess); B10.064 and B10.157 (external crypto path, with further restrictions); B10.226 and B10.233 (async notification); B13.031 (memory-unsafety symptom); B14.259 (async/runtime); B16.018 and B16.128 (network and kernel behavior); B23.129 (subprocess, guard across an effect, OS error and poisoning). These are registered E4 restrictions. I found no supported basis to relax them or call any of these rows eligible. The missing source/count records and abbreviated reading remain findings 1 and 4. B09.003 remains cap:proc only; the socket rows and B23.129 correctly have additional restrictions.

For arm A, all ten archive digests match the registration, and 476 extracted Rust source/manifests match their archive members byte for byte. Read all ten private maps and spot-checked the decisive source sites: caller-supplied serialization under serde guards, rustls's shared send adapter, external dependencies, env_logger's guard across writes, bindgen's subprocess pipes and the construction/type-only uses in darling_core and schemars. serde_core's tree matches serde's core. The 8 unportable / 2 no-core / 0 portable summary agrees with the working report. This review did not independently reconstruct every complete core union; finding 4 identifies the missing counts.

### Deviations, unresolved evidence and privacy

The record includes every deviation specifically listed in the brief/bead: arm overlap, builder 2's filter stop, E1-first, path pass and its unexplained intermediate totals, the declared whitespace/comment pre-pass, request-spacing failures and overlapping bindgen queues, the short lexical arm-A maps, compact reading, and builder 8's reported duplicate B10.064 line. The final layer has one decision per each of 1,220 manual IDs. The disclosed earlier path totals and duplicate-line history cannot be established from the final snapshot alone; the record appropriately leaves their historical explanation unresolved. Finding 5 is an additional omitted deviation from the working report; finding 1 explains why deviation 8 is materially understated.

B23.128 is retained as UNRESOLVED, with the unavailable PR949 evidence and an explicitly uncounted provisional reading. It is not included in any completed-negative first-failure bucket.

All ledger commit identities are public-upstream-history identities in the frozen clones. Many keyword matches are intentionally non-bug-fix E1 exclusions; mere commit existence does not turn them into fixed bug cases. The sampled bug descriptions are tied to public fixed changes. No committed text reviewed describes an undisclosed, unfixed finding, and arm A records no candidate. No privacy breach was found. This is a review of retained evidence and sampled semantics, not an independent security audit of every upstream change.

Not checked: cryptographic timestamp anchoring, upstream execution, exhaustive semantic rejudgment, or complete source-union reconstruction. No heavy run was needed. Risk: raw counts currently reproduce incomplete and partly incorrect screening decisions.

DONE
