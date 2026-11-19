Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate e0bc079 (re-check of B1-B7)

Scope: finder re-check of B1–B7 and the three actionable non-blocking items from r1, limited to the changes from 5dcc228 to e0bc079d97499d2cf00afa51323a0ba54b649174. This is not a new full review. Only this report was written. No build, model run, upstream request/contact, commit or Beads write was made. No private files were accessed in this re-check.

## B1 — Resolved

The public appendix research/protocols/fx-zu4f.11-record/b-candidates.tsv:1–43 supplies all 42 commit/first-parent identities, source URLs, duplicate links, reasons and E1–E5 statuses. Every row records one first failure with earlier criteria passed or tentatively passed and later criteria not assessed. All 22 E2-pass rows have symptom quotes and source identities. Nineteen quote entries cite commit messages; their quoted segments match the frozen local messages after whitespace normalization and explicit ellipsis splitting. The other three entries cite public issue bodies/comments with identities and dates; their contents were not independently fetched or read from the restricted private cache.

All six E4 rows now identify the affected path/trigger and the deciding unsupported operation; duplicate e1edeb3 links to 9841626's inventory. Incomplete function inventories and counts are labelled INCOMPLETE. The unsupported size conclusion for #801 was removed (record:273–280). This meets r1's requested distinction between independently decisive barriers and unfinished counting; it does not certify complete unions.

## B2 — Resolved

Record:256–269 and :331, appendix row db8db55bcd58 (TSV:41), now put the first failure at E1 and leave E2–E5 not assessed. That is the applicable registered rule: protocol:429–433 requires a publicly resolvable production fix/first-parent pair.

Offline inspection confirms zero entries from git log --all -- src/relay.rs in the frozen public history. The exact fa7acd0..db8db55 production diff adds warm-loop calls whose public implementations are no-op/false, plus a relay version bump; the cache guard is in the test module. It provides no public source pair repairing the stated relay faults. E1 is therefore the required first failure, rather than E3. Keeping unresolved at zero is justified by established absence from the acquired registered history, as distinct from a failed required-object/evidence acquisition. This conclusion makes no claim about the unavailable relay implementation.

## B3 — Resolved

Record:356–390 adds the per-repository roots, enumerated call/state map, omitted work, partial counts and stopping boundaries. Lines 371–380 explicitly say that enumeration stopped at an independently decisive unsupported boundary and that no whole-core union was counted. Rows 2, 5 and 7 no longer claim a measured union over 400 lines. Historical counts are distinguished from the pin. This resolves the reporting finding under r1's explicitly offered partial-enumeration disclosure; complete core inventories remain unproven, not silently asserted.

## B4 — Not resolved: the fix introduces an inaccurate method statement

The omissions themselves are repaired at record:137–155: the pre-registration source change, both B gates, scannability restriction, corrections/recomputation, A1/A2, context-budget checkpoint and its incomplete state, provider stop without screening changes, and builder 3's resume are disclosed.

However, the added row at research/46-fx-zu4f.11-agent-marker-round.md:146 says “No words were added” when describing the change from the inherited substring expression to the registered expression. That is not accurate. The inherited expression in target/work/reference-protocol.md:257 is:

    race|deadlock|dead-lock|hang|hung|livelock|lost wakeup|data race|stuck|freeze|starv

The frozen expression in the binding protocol:372 additionally matches the literal words racy, racing and frozen, none of which matches that inherited expression. A plain-Python comparison confirms False → True for each, alongside True → False for change and trace. This is a reporting error introduced by the B4 fix, not a request to reopen the approved registration or change candidate selection.

Required correction: replace “No words were added” with an accurate description, for example “Used the exact pre-registered expression, including its listed inflections; removed embedded-word matches.” Do not imply the new match set is a strict subset of the old one. Keep the registered expression and the 42-candidate enumeration unchanged.

## B5 — Resolved

Record:48–115 supplies registration/amendment/proof commits and UTC times, the original/A1/A2 protocol digests, both pool digests, acquisition intervals, provenance hashes and tool/platform identities. All 15 tabled provenance hashes, both pool hashes, the appendix hash and all three historical protocol hashes match their actual bytes. Git commit times agree with the stated UTC times. Metadata and clone/count interval endpoints agree with their committed tables. Pending db130d0 and upgraded 584b291 proofs are distinguished. Timestamp cryptography was not reverified.

## B6 — Resolved

Record:411–432 has revision-specific URLs for all 17 documents across seven repositories, fetch times, read/classification date and bounded policy wording. The later policy revision for rank 5 is disclosed. This closes the missing-URL/date finding; live policy state and full policy contents remain outside this re-check.

## B7 — Resolved

Plain-Python recount of b-candidates.tsv confirms the revised record:203–222 and all 42 compact narrative rows:

| Measure | Recount |
| --- | --- |
| Candidate rows / distinct commit IDs | 42 / 42 |
| Duplicate-site rows / deduplicated units | 3 / 39 |
| Raw first failures E1/E2/E3/E4/E5 | 16 / 4 / 16 / 6 / 0 |
| Deduplicated first failures E1/E2/E3/E4/E5 | 16 / 2 / 16 / 5 / 0 |
| Unportable rows / units | 6 / 5 |
| Unresolved / eligible | 0 / 0 |
| B candidates by pool rank 1–7 | 4 / 13 / 1 / 11 / 3 / 0 / 10 |

Duplicate references resolve to nonduplicate rows with the same first failure. Every listed first parent matches the local public Git object. The sole judgment change from r1 is db8db55, E3 → E1; both raw and deduplicated distributions change by E1 +1, E3 −1. The 7/7 repository coverage, A 7/7 unportable, zero modeled/properties/catches and admission totals are unchanged. “Unique fixes” has been replaced with correctly defined screening units.

The disposition file's statement of 24 E2-pass rows is a clerical error: the public TSV has 22 (16 E3 failures + 6 E4 failures). The public record does not repeat that erroneous number.

## Non-blocking items

1. Resolved: E1–E5 and sequential consistency/SC are defined at record:9–19 before use in the outcomes.
2. Resolved: record:285–288 restricts the Tide-primitive statement to recorded first failures and explicitly leaves later criteria unassessed.
3. Resolved: record:337–346 makes both capability tags provisional, identifies uncounted helpers/callers and directs fx-zu4f.13 to count the unions. fbcbbb8's E3 remains tentative; neither row becomes eligible.

## Privacy

No privacy breach was identified in the added text or TSV. Their bug descriptions and quotations are attributed to existing public commits or named public issue/comment sources; the added A material describes scope and portability boundaries, not new bug allegations. Rejected E1 candidates are labelled as such, rather than called established fixes. The db8db55 discussion describes a publicly stated historical repair and the absence of its public implementation, not an undisclosed finding. No private narrative was inspected or quoted. Public issue quotations were checked for attribution, not independently authenticated through the restricted cache or a new request.

## Verdict

**changes required**: B1–B3 and B5–B7 are resolved; all three non-blocking items are resolved. B4 needs the single methodological wording correction at record:146. Counts, the B2 re-judgment and both NOT CAUGHT outcomes stand. No broader review or experiment rerun is requested.

Not checked: live upstream state, full discussion/policy contents, complete path unions or timestamp cryptography. Those limitations do not create additional findings in this bounded re-check.

DONE
