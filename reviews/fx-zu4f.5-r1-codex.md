Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 1f90f3e (docs)

## Verdict

changes required. The zero-catch conclusion follows the frozen caught rule, but the record is not yet a complete, internally reconciled account of compliance with the registered method and the owner's archive conditions. Confidence: high in the findings below; bounded as stated under Reasoning.

## Scope

Read-only methodology and honesty audit of `research/37-external-admission-search.md`, against the frozen fx-zu4f.5 body, its comments and registered protocol, AGENTS.md and GOVERNANCE.md. No implementation, ledger, source, or candidate edits. This report is the only write.

- Candidate HEAD: `1f90f3e5d63ac5883891adb311ec4aa8e905fd7e`.
- Base: `cdf47ec`; only candidate change is the 757-line research record.
- SHA-256 of `git diff cdf47ec..1f90f3e`: `1f2fc065da4b8ce48a157febb707d4439eb8a971359c526ac96a23670ef37782`.
- Registered protocol SHA-256: `0e5cebada4f0c379f72480a10331a55a18fdff37377659510f9cea0b6b424fb0`. The document's embedded protocol matches the registered file byte for byte.
- Advisory snapshot: `233155b4025b854c68a4b199c0e32d7fd53291a2`.

## Blocking findings

1. **The declared screening did not perform the registered prose pass** — `research/37-external-admission-search.md:68` (steps at lines 70–78; binding protocol reproduced at lines 608–616).

   The protocol requires identifying malicious records by reading the metadata and prose of **every** in-date advisory. The reported method instead reads some records only by title: in the 173-record broad-pattern group, bodies are read only where a title leaves doubt; in the remaining 1,249, bodies are read for only 280. Thus at least 969 advisory bodies were not screened by the declared method. Matching the expected categories and finding the expected 87 IDs does not establish that the omitted prose contains no qualifying record. This is a post-registration relaxation of the screening method, not the explicitly registered P2 narrowing. It does not support the unqualified completed-search claim at line 3.

   Minimum fix: complete the omitted metadata/prose screening at the same pinned commit, record its actual scope and any resulting changes, and only then describe the registered search as complete. Do not amend the registered protocol to permit the shortcut. If a complete pass already occurred, document that pass and its evidence rather than the title-only method currently asserted.

2. **The P1 subtotal and its enumeration describe different evidence sets** — `research/37-external-admission-search.md:38` (enumeration at lines 43–47; rustdecimal at line 331).

   The asserted 13 “quoted code ported or assessed” entries are enumerated as seven amaperf crates, two quoted-code assessments, and three other ports: **12**, not 13. The table has 13 plain `not caught` rows only because it also puts rustdecimal in that bucket. Its reason explicitly says no code is quoted; its process-capability assessment is based on prose. Therefore the 13/74 split does not have the meaning the summary gives it. The overall 87 and zero caught remain correct; the evidence subtotals and enumeration do not reconcile as required by the protocol.

   Minimum fix: explicitly include rustdecimal as a prose-only feasibility assessment and define the two subtotal labels so their disjoint meanings fit all rows, or classify it with the no-portable-source rows and consistently use 12/75. Preserve the actual evidence limitation; do not imply a thirteenth quoted-code assessment.

3. **Post-approval probes crossed the owner's explicit filesystem boundary without a recorded exception** — `research/37-external-admission-search.md:129` (archive conditions discussed at lines 140–144 and 280–284).

   The owner approval recorded on the bead allows the four archive copies with the condition, “Everything stays inside the worktree's target/work.” The record says every port ran in temporary `examples/case-studies/zz-probe-*` directories. The saved post-approval `ports/run3.log` through `run6.log` confirm these examples paths for the archived-source probes. Removal afterwards prevents leftovers but does not make those writes comply with the condition. Reusing the normal runner is not a recorded owner exception.

   Minimum fix: disclose the deviation explicitly and obtain the owner's disposition of this boundary violation through the coordinator. Keep any further archive-derived working artifacts within the approved boundary unless the owner grants an exact exception. A factual statement that the temporary examples were removed is useful, but cannot retroactively supply authorization.

4. **The conda/rattler retry is not independently supported by the saved repository artifact** — `research/37-external-admission-search.md:430` (row at line 448).

   `target/work/search/repos/conda--rattler.json` contains only the failed HTTP 502 lookup, with `data: null`. The document reports a successful retry returning 466 stars, but gives no retry timestamp, and no successful retry response is present among the supplied search artifacts. This limits verification of that precise repository-metadata claim; the saved search item is open regardless, so this gap does not produce an eligible closed/merged candidate or change the zero-candidate outcome.

   This blocks under the requested audit rule because the precise retry claim is unsupported by the retained evidence, and the registered protocol requires a lookup timestamp for every result. Its lack of effect on the zero-candidate conclusion does not make it a style issue.

   Minimum fix: retain or cite the existing successful retry response and its lookup timestamp. If those were not retained, disclose the unsupported value and missing required lookup record explicitly and resolve that protocol gap before claiming full compliance. Do not rerun the frozen search query.

5. **Port commands are templates rather than the exact recorded invocations** — `research/37-external-admission-search.md:131`.

   The command uses `TMPDIR=.../tmp` and `timeout <secs>`, and the prose provides minute-level run endings rather than exact invocations and a per-run status record. The diagnostic logs support the reported failures, but do not contain the missing command arguments. This falls short of the protocol's requested exact port-proof record and makes reproduction harder; it does not create a catch claim.

   This blocks under the requested audit rule because exact invocation proof is part of the registered protocol, not a formatting preference. A negative result does not waive that requirement.

   Minimum fix: transcribe the actual command arguments and outcomes from the existing execution record. If those details were not retained, disclose that gap and resolve the missing proof before claiming full protocol compliance. No new Rust run is needed if the existing execution record supplies the details.

## Non-blocking findings

None.

## Reasoning

The operational P2 narrowing was registered before examination, and is conservative: apply `language:Rust`, cap at ten raw default-sort results, then check repository stars and primary language without refill. It does not widen the frozen pool. All six saved request command arrays have the exact registered keywords, GET endpoint, page 1 and per_page 10, without sort/order. Their timestamps are 2026-11-18T05:50:10Z through 05:50:15Z; each saved response reports incomplete_results false and ten items. I compared every document occurrence against its saved URL, title, order, kind and state. The 60 URLs are unique. The successful saved repository metadata agree with the displayed stars/languages; the 20 threshold-passing detail responses are open, so no closed-issue/merged-PR bug or fix assessment is needed. The remaining 40 exclusions reconcile, subject to finding 4's evidence limitation.

Using local `git ls-tree` and `git show` at the pinned advisory commit, I independently derived 1,507 Markdown crate advisories and compared every saved inventory path and text with that snapshot. Dates are parseable, range from 2016-05-09 to 2026-11-16, and all satisfy the inclusive cutoff. The stated screening partitions reproduce as 85 malicious-category records, 173 additional broad-pattern records, and 1,249 remaining records. Every table ID, crate package, date and pinned path matches the snapshot. Its 87 IDs consist of 84 malicious-category records plus logprinter, arone and aronenao. The described affiliation-only exclusions, ordinary vulnerabilities and deliberate-unsafety exclusions are consistent with the intentional-malice inclusion rule. Neither faster_log nor async_println appears in the pinned crate inventory or its advisory text. These checks verify the inventory and the listed selections; they do not cure the omitted full-prose screen in finding 1.

The 87 table rows reconcile mechanically as 13 plain not-caught rows and 74 rows explicitly labeled no portable source. Shared incidents remain separate advisory IDs. The general reasons distinguish missing quoted functions, unavailable faithful counterparts, and dependency-resolution failures. The documented four archive URLs match the exact owner-approved originals. I checked retained write-up prose and advisory classifications/effect descriptions without inspecting malicious code or port bodies. I did not fetch, download or execute any crate, payload, or advisory-linked source.

The saved diagnostic-only log evidence supports the important distinction: chrono_anchor's buggy probe reports TD0602 twice and TD0501 three times, while its capability counterpart reports only two E0599 missing-command errors. The current std's Proc declaration has no operations, and reference §6.1 marks command as planned. Under the frozen conjunction—named TD rejection **and** admitted fixed/capability counterpart—this is correctly **not caught**, not a stretched hit. Amaperf's logs likewise support its named TD rejection and failed counterpart; finch-rust and uniswap-utils show dependency-resolution errors rather than named Tide catches. I did not independently validate malicious source-to-port faithfulness or recreate any probe.

No case study or production change is in the diff. The record reports counts rather than an effectiveness rate, distinguishes rejection of ambient effects from a demonstrated qualifying catch, and makes no malware-proofing claim. The embedded protocol retains the explicit rustc-soundness-hole limitation. A workspace suite would not resolve the documentation findings and was not run.

Proof limits: saved requests establish six retained query executions, not the historical absence of other executions; I cannot independently prove “once” or “not rerun” from those files alone. Registration/order, past access restrictions, source-fetch history, and absence of payload execution rely partly on the bead's contemporaneous statements and retained artifacts rather than a complete execution transcript. No network or Rust commands were run during this review. Skipped malicious-code inspection and runtime re-execution; remaining risks are the declared incomplete screening and owner-boundary disposition above.

DONE
