Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check 5dc6b40

## Verdict

changes required. Findings 1–4 are resolved. Finding 5 remains open, and its fix introduced one unsupported proof claim. The documented zero demonstrated catches remains consistent with the frozen caught rule.

## Scope and identity

Finder-only recheck of the five original findings and defects introduced by their fixes. Read the round diff and relevant retained evidence; no repeat review of unaffected candidate content. No candidate, source, or ledger writes; this report is the only write.

- Candidate: `5dc6b40e0254d5dd83ab83b0f2d5a1dd2c2ec6dc`, base `cdf47ec`.
- Round diff: `git diff 1f90f3e..5dc6b40`, SHA-256 `f1d956a0d17d442fad1bb85aa4283d6f88ba11fa6f948c9a7c53ba6468cc0362`.
- Whole candidate diff: `git diff cdf47ec..5dc6b40`, SHA-256 `903602a7d9a01dda1fd9b8747771d18624f729f79f7035a14614dc9908be7e2c`.
- Only the research record changed. Tracked working tree is clean.
- Embedded protocol remains byte-for-byte equal to the registered `target/work/protocol.md`, SHA-256 `0e5cebada4f0c379f72480a10331a55a18fdff37377659510f9cea0b6b424fb0`.

## Finding status

1. **Resolved — full prose screening evidence supplied.** `research/37-external-admission-search.md:78` now distinguishes the deficient first screen from the full r1 pass. The shared reader brief requires each record's metadata and prose to be judged in full under the registered inclusion rule, without keyword/title-only selection. I reproduced all six batch texts from the saved inventory with the supplied dump script and compared them byte for byte. They cover consecutive, nonoverlapping ranges of 252, 252, 252, 252, 252 and 247 advisories. All six per-batch output logs have exactly one matching ID and decision per input; the joined `screen-full.tsv` has 1,507 unique matching paths/IDs/dates and nonempty reasons. Counts are 87 yes, 1,420 no, zero ambiguous; the 87 yes IDs exactly match the research table. The 36 sample indices reproduce from seed 20261118, span all six batches, and their decisions agree with the joined log. The claim about absent intent words in omitted body code fences also matches a mechanical check, without displaying those code bodies. These artifacts supply the missing complete-pass evidence; I do not infer that an output log can independently prove every reader's internal attention.

2. **Resolved — evidence subtotals reconcile.** `research/37-external-admission-search.md:42` uses 12 quoted-code assessments and 75 no-portable-source assessments. The new definitions make the categories disjoint; rustdecimal is explicitly prose-only and its row at line 391 moves into the latter category. A mechanical recount finds 87 table rows: 12 plain `not caught`, 75 `not caught (no portable source)`. The seven-plus-two-plus-three enumeration now equals its stated subtotal. This changes bookkeeping, not any caught outcome.

3. **Resolved under the binding attribution correction.** `research/37-external-admission-search.md:193` explicitly discloses the temporary examples directories outside target/work, their removal, and the reported disposition. It attributes the condition and disposition to tide-desk, while preserving Lucca's actual approval words, “1.2”. I accept the user's correction: this is disclosure of a deviation from tide-desk's added relay condition, not an outstanding demand for an owner exception. The text also requires further archive-derived work to remain inside target/work. The recheck performed no archive access or probe rerun. The historical absence of further work relies on the supplied record rather than an exhaustive filesystem activity log.

4. **Resolved — successful retry artifact supplied and cited.** `research/37-external-admission-search.md:493` cites `search/repos/conda--rattler-retry.json` and the retry timestamp. That file contains a successful exit-0 response at `2026-11-18T05:58:29.772807+00:00` for `conda/rattler`, with 466 stars and primary language Rust. It supplies the precise evidence missing from the first review. No frozen query was rerun during this recheck.

5. **Not resolved — disclosure does not supply the required invocation proof.** `research/37-external-admission-search.md:163` now honestly says that the exact command lines, including timeout values, were not retained. The per-run table is useful: its modification times, heavy.sh slot names, failed summaries and reported diagnostics agree with the retained logs. It removes the misleading command template. However, the registered protocol still requires exact commands and environment for port/admission proof, and the document still begins with “The registered search is complete” at line 3. Nothing in the fix supplies the missing exact invocations or changes that completion claim. Under the requested rule, a remaining frozen-protocol proof gap blocks even when it does not turn a negative into a catch.

   Minimum fix: supply the exact invocations from an existing execution record, if available. Otherwise keep the explicit missing-evidence disclosure and describe the port-proof portion as incomplete; withdraw the unqualified completed-registered-search claim until the required proof is supplied. Do not invent historic arguments or silently waive the registered requirement. The observed diagnostic failures and zero demonstrated catches can still be stated with their limits.

## New blocking defect introduced by the fixes

6. **The failed probe runs do not prove all landed examples passed.** `research/37-external-admission-search.md:180`–181 says, “The landed examples meet their declarations in every run.” The runner at the recorded probe base (`33085fd:crates/tide-admit/tests/examples.rs`, function `examples_meet_their_declared_outcomes`) walks sorted examples and panics on the first mismatch. Each saved run stops at its failing temporary case-study probe. The pinned examples tree also contains `examples/contracts`, ordered after `examples/case-studies`; those later examples are absent from every supplied run log and are skipped after the panic. Seven successful runner self-tests do not prove those skipped examples' declarations. This is an unsupported widening of the retained failed-run evidence.

   Minimum fix: replace the sentence with the narrower supported observation: the landed example variants reached before the failing probe met their declarations; later variants were not checked by these runs. No new suite or probe execution is needed for that correction.

## Proof limits

This recheck used local file inspection, Python bookkeeping/reproduction of text-only batches, diagnostic-only log inspection, and local git reads. The advisory inventory's match to pinned commit `233155b4025b854c68a4b199c0e32d7fd53291a2` was established in the original review; the r1 batch reproduction uses that inventory. I did not independently repeat all 1,507 semantic screening judgments or rerun the six GitHub searches. Artifacts and shared briefs establish recorded coverage and consistent outputs, not the historical absence of unrecorded actions or each reader's internal process.

No network, crate fetching, payload inspection, malicious-code or port-body inspection, Rust compilation, or runtime execution was performed. Skipped unaffected candidate review; the remaining risks are the missing exact invocation proof and the newly widened example-proof claim identified above.

DONE
