Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 5dcc228 (record)

## Verdict

**changes required**. High confidence in the recount and the reporting omissions; qualified confidence in judgments that depend on PR/issue discussions unavailable within this review's permitted evidence boundary. Both NOT CAUGHT outcomes remain supported by the available evidence. This verdict concerns methods fidelity, not a security assessment.

## Scope

Read-only review of research/46-fx-zu4f.11-agent-marker-round.md at 5dcc228ebf468174ce6c567ebb340033dc505f2f against the amended registered protocol, especially §§3–4 and §§8–9; registration/amendment identities; the bead and comments; builder reports/checkpoint; selection tables, history exports, local first-parent objects and admission TSVs. Only this report was written. No build, model, upstream execution, network request, subagent, contact, commit or Beads write was made.

Private access was confined to count checks: aggregate screening labels, duplicate counts and A outcome/property/model/run counts. No private evidence narratives, policy classifications or PR/issue caches were inspected or quoted. Public Git commit messages and diffs were inspected from the existing frozen local histories. Consequently this review does not independently certify complete reading of every linked discussion or the underlying policy classifications.

## Blocking findings

### B1. Required candidate evidence is absent from the research record

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:129–204; protocol:425–461 and 728–735.

**Evidence:** The 42-row table supplies commit/parent, first-failure label and a paraphrased reason, but no source URLs. Rows that pass E2 almost uniformly omit the required exact symptom quote and its revision/comment identity. For example, 59dfd3df852b at record:170 is reduced to “#801 hang/wrong output”; its public commit message explicitly explains wrong output and non-convergence. 0de4307824a4 at record:181 similarly paraphrases a kernel hang. No table or general rule identifies later criteria as not assessed versus additional. E4 rows generally lack the required file/function/trigger inventory and counted path union: the 348/458 file counts at record:151 are not a function inventory, and fbcbbb8 at record:214 leaves the dns.rs callers uncounted.

**Risk:** A reader cannot audit which stated symptom passed E2, whether it belongs to this exact fix, or what was actually assessed before each negative. The private working records are not a substitute for §8's delivered research record.

**Fix:** Add a compact public evidence appendix keyed by all 42 commits: source links/revision identities supporting each first failure, an exact attributed E2 symptom quote wherever E2 passed, and explicit later-criterion status. Supply the registered source/function/count information for E4 assessments; keep uncompleted counts explicitly incomplete. Preserve the provisional capability tags rather than assuming unexamined helpers pass.

### B2. Missing-source evidence is treated as a completed E3 negative

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:142–146, 202, and the unresolved total at :122; row db8db55bcd58. Protocol:409–421 and 429–433.

**Evidence:** The record says the concurrency-shaped part cannot be resolved publicly because src/relay.rs is absent, yet counts the row as screened, resolved and first-failing E3. The public commit's message distinguishes in-flight-tunnel escalation, recovery probing, and a test-cache race. Its first-parent diff adds warm-loop calls to probe_relay/relay_is_benched and exports those names from the cfg(relay) module; their public non-relay implementations are no-ops/false. The cache serialization change is test-only. Builder report decision 4 likewise acknowledges that the concurrency-shaped repair has no publicly resolvable source pair.

**Risk:** Absence of required source is not evidence that a bug is single-threaded. §3 forbids turning unavailable required evidence into a completed negative, and E1 already requires a publicly resolvable production source/first-parent pair.

**Fix:** Make the unit and evidence treatment explicit. Supply a publicly resolvable production pair supporting an independently assessable visible timer-policy fix, or record the missing pair under E1 / required-evidence unavailability as unresolved, as applicable. Do not justify E3 by source absence. Reconcile the affected first-failure and unresolved totals with the corrected working classification. This does not establish an eligible candidate or overturn NOT CAUGHT.

### B3. The delivered A rows omit the registered core scope/count record

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:224–236; protocol §6 and :734–735.

**Evidence:** All seven rows name a plausible portability blocker, but most give only a few roots and no union count or complete source call/state map. Rows 1, 3, 6 and 7 have no numeric core count; row 5 counts compare_song alone; row 2 counts three whole server files rather than specifying the concurrent-core functions. §6 asks for interacting roots, initialization/shutdown/error paths, required helpers and the union, not merely a shortest blocked path.

**Risk:** The record cannot substantiate whole-repository core coverage or distinguish a measured size failure from an uncompleted inventory. The aggregate 7/7 is a recount of recorded outcomes, not proof of this missing coverage.

**Fix:** Add a concise per-repository scope/map/count appendix. If an independently decisive unsupported boundary stopped enumeration before a complete count, disclose that limitation and identify exactly what was and was not enumerated; do not present a partial file/function count as the complete union. No property quotes need inventing: all seven stopped before property freeze.

### B4. Method changes and the checkpoint stop are underreported

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:47–50, 61–72 and 114–118; target/work/report-checkpoint-builder1.md:5–10; committed mechanical-corrections.json; protocol §§2–3.

**Evidence:** The record names a whole-word regex and directory gate but never identifies them as coordinator-directed pre-registration narrowings, or states why they replaced the earlier matcher (5,076 substring hits including embedded words). It omits the additional UTF-8/tomllib scannability narrowing and the documented pre-registration scanner/acquisition corrections, including the test-parameter masking correction with invalidation/recomputation of earlier scans and the date-shallow fallback. Calling builder 1's stop simply a “post-GO checkpoint” omits its context-budget cause and its exact state: admission 7/7, B enumerated 42 but screened 0, A screened 0, both incomplete. Builder 2's provider stop and builder 3's resume are disclosed, as are the candidate-source change and A2; those disclosures should remain.

**Risk:** A reader cannot distinguish changes authorized before registration from changes made during screening, or determine what screening state passed between builders.

**Fix:** Add one compact methods/deviations paragraph or table. Explicitly date the two B narrowings and scannability restriction before registration, summarize the material corrections and unchanged/recomputed surfaces with a pointer to the committed correction manifest, and state builder 1's stop reason/state and that builder 2 changed no screening state before builder 3 resumed. Expand A1's description sufficiently to say it initially retained nova while excluding any catch there from the external-catch bar, then A2 removed it before GO. Existing source-change and pacing disclosures are adequate on their own terms.

### B5. Mandatory acquisition provenance and environment are missing

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:35–53; protocol:728–731.

**Evidence:** The record includes the base, registration/amendment commits, two correct hashes, GO-read time and examination times. It omits metadata/clone acquisition intervals and their hash-bearing artifact identities, the initial pool's digest, and actual tool versions/platform. These are available in the committed acquisition-summary.json, candidates.tsv, activity-markers.tsv, queries.jsonl and tooling.json. For example, the acquisition summary gives repository search 18:21:38–18:23:36Z and clone/count observations 18:25:40–20:34:12Z on November 18; tooling.json records Python 3.14.10, Git 2.43.0, gh 2.104.0 and Linux/glibc identity. The initial proof commit db130d0 is also absent, although the upgraded proof commit is named.

**Risk:** §8's provenance chain is incomplete in the delivered record; a generic list of tools does not identify the research environment.

**Fix:** Add the required intervals and environment and explicitly identify/hash the committed provenance manifests/tables and registered versus amended pool. Name pending-proof and upgraded-proof commits distinctly. Avoid describing non-atomic metadata/clone acquisition as simultaneous with the snapshot. No new acquisition or timestamping is needed.

### B6. Policy rows have no required URLs

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:257–274; protocol disclosure section and §9.

**Evidence:** A shared classification/read date and bounded document-search method are supplied, but all seven policy rows list filenames only. There is no URL anywhere in the policy section. The checkpoint says exact document revisions, URLs and digests were acquired; the builder report claims URL/date classification is complete, but none of those URLs reaches the record.

**Risk:** The required per-repository AI-report-policy observation is not independently attributable, especially because these were default-branch policy documents rather than necessarily the pool's pinned revision.

**Fix:** Export the already-collected revision-specific policy-document URLs and read/classification date for each repository. Preserve “no rule found in inspected documents” where appropriate; do not broaden a bounded absence observation into a repository-wide assurance. This review has not independently validated the classifications behind those missing citations.

### B7. Counts need complete units and accurate deduplication labels

**Location:** research/46-fx-zu4f.11-agent-marker-round.md:120–127; protocol:742–747.

**Evidence:** The numeric first-failure distribution matches the working labels. However, the B summary omits a separately named unportable count required per arm. It labels 39 deduplicated candidates “unique fixes” even though 15 rows fail E1, including features, documentation and an investigation that did not fix its reported bug. There are 42 distinct matched commit IDs; the three duplicate-site rows reduce the deduplicated screening units to 39, not 39 established bug fixes. The targeted/screened count is repositories (7), while the first-failure denominator is candidate rows (42); the completed 42/42 screen is not explicitly tabulated.

**Risk:** Correct arithmetic is assigned to unlike units, overstating how many actual fixes were established and leaving §8's count requirements incomplete.

**Fix:** Explicitly separate 7/7 repository coverage, 42/42 candidate screening, 42 unique commit IDs, 3 duplicate-site rows and 39 deduplicated candidate units. Name the B unportable count and its definition (currently six E4 rows, five deduplicated units, subject to corrected judgments). Retain both raw and deduplicated E1–E5 distributions and reconcile any B2 correction. Do not call rejected E1 candidates confirmed fixed bugs.

## Non-blocking findings

- Define E1–E5 and SC (sequential consistency) at first use. The opening uses those terms before its later criterion summaries; E5 is never briefly defined in the record.
- “No row failed for a missing Tide v0 primitive” at :158–159 is broader than the first-failure design permits. Prefer “No recorded first failure was attributed to a missing Tide v0 primitive”; later criteria are unassessed for many rows.
- Keep 12a36c9 cap:net explicitly provisional. The opening “only because” at :208 is firmer than its uncounted dns_client helpers justify. fbcbbb8 cap:proc is correctly tentative at E3 and linked to fx-zu4f.13; neither row should be relaxed into eligibility.
- No discovery-rate claim, unaided “Tide found” claim, author judgment or marker-share/source-authorship equation was found. The negative-first opening and separated admission section are appropriate.

## Reasoning

Independent plain-Python recounts in this unchanged candidate/environment established:

| Surface | Recount |
| --- | --- |
| Selection tables | 2,139 candidates; 2,105 metadata survivors; 1,521 activity survivors |
| T=50/40/30/25/20 marker survivors | 66 / 98 / 137 / 164 / 208 |
| Corresponding scan survivors | 5 / 5 / 5 / 5 / 8 |
| Registered / A2 working pool | 8 / 7; nova alone excluded, no backfill |
| Frozen full-message exports plus registered directory gate | 10,853 commits; 64 keyword matches; 22 path exclusions; 42 candidates |
| B recorded first failures | E1 15, E2 4, E3 17, E4 6, E5 0; eligible 0 |
| Candidate identity checks | 42 distinct commit IDs; all 42 listed first parents match local Git objects; 3 duplicate-site rows |
| A count-only working records | 7/7 unportable; 0 properties, models or runs |
| Admission TSVs | 7/7 present; every per-repository cell and total matches the record |

Admission import/reference totals are fs 33/342, net 46/267, process 23/153, env 14/256, clock 34/309; discarded/unresolved spawn handles 18/7. Categories remain separate, descriptive and outside catch counts. Both published protocol/pool-amended SHA-256 values match their files.

Judgment samples used existing public commit messages/first-parent changes, not private narratives:

- E1: 23bfd3aaa96f, 553d9bc121f6, 80e2d19df7b5, 8a76bbff6a8b and a6eee82b10be. Their investigation/no-behavior-change/new-source/comment-removal/feature-removal explanations are consistent with E1. Additional feature-squash messages were inspected, without treating their full linked discussions as independently verified.
- E2: all four rows, 6bd3fc9f24f9, b5ca63cb63f5, b6d4e7b15342 and d59e32974399. The RedisJSON messages distinguish potential cache races from corruption that persisted with the cache change. The compiler row describes encoding changes and verification, not a stated occurrence attributable to those fixes. The recorded exclusions are consistent with the available messages, subject to the unreviewed discussion evidence.
- E3: 476247f7028b, f16c6cff2b96, 0de4307824a4, 1d26685da430 and 30d9d54e4d27. Their stated defects are non-converging value feedback, miscompilation/infinite loops or sequential UI ordering. They do not expose an overlooked eligible concurrent path in the sampled evidence. db8db55bcd58 was additionally checked and produces B2.
- E4: all six rows, 59dfd3df852b, 984162659a4a, e1edeb3d665b, 12a36c940b10, 8566deddfd99 and fbcbbb8b47d0. The timely operators/frontiers, DNS network race, TCP-peer timer interaction and child-process wait identify real registered barriers. Capability rows remain tagged for fx-zu4f.13, with the net row provisional. No sample justifies changing an excluded row to eligible.

Privacy: the new record contains only descriptions traceable to public upstream commits, admitted lexical observations and portability limitations. No undisclosed/unfixed finding was identified. Some B entries are rejected keyword candidates rather than fixes, which is legitimate to enumerate but makes the “unique fixes” label misleading (B7). The private aggregate reports no eligible B row or modeled A candidate; this review did not inspect private narratives or independently search for undisclosed defects.

The minimal approval patch is a research-record correction/appendix covering B1–B7 and any necessary corrected working classification for db8db55. No language/toolchain change, extra repository, upstream run or widened catch rule is warranted. No alternative experimental design is proposed.

Not checked: live upstream state, full PR/issue threads, private narratives, timestamp cryptography or policy contents. Residual risk is judgment/provenance fidelity at those unverified surfaces; no runtime proof or bug-free claim is made.

DONE
