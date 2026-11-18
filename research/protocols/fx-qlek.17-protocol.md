# fx-qlek.17: account-capability model protocol

Registration draft, 2026-11-18. Base: `a34ffe25eac689420ce9a98e876f28542cd36a37`;
branch: `fx-qlek.17`. Contract: bead fx-qlek.17, Scope 1–4 and Acceptance.
The builder's step-1 brief ends this run at PROTOCOL-READY. This protocol
models a **proposed** Solana account-capability rule. It never measures a
Tide v0 catch, exploit discovery, or an implemented compiler's accuracy.

## 1. Register, stamp, then go

Commit this file, `fx-qlek.17-cases/` and the §1–§3 research draft in one
commit before assessing any case. Write `target/work/PROTOCOL-READY` with
commit, UTC time, case count and paths, then stop. Tide-lead records hashes,
OpenTimestamps proofs and an explicit “go” before the builder reads fix
diffs, constructs case models or assigns verdicts. No network, build or
case work continues across that stop without the next brief.

The timestamp can establish that registered bytes existed by a time. It
cannot prove what the author had read, or whether private assessment had
occurred. Reading chronology remains self-attested. Record proof filenames,
verification result, stamped commit and go time in the final research
record; do not claim that a local Git date proves prior registration.

Before registration the builder has read the task's named examples,
search-result summaries, the source indexes and incident descriptions
needed for selection, plus general framework/security documentation.
Those descriptions can disclose causes and fixes. That is prior knowledge,
not a blind trial. No per-case fix diff is read and no case is judged in
step 1. Record any accidental case exposure before registration or go in
`target/work/report.md`; preserve it in the final limitations. Reading
Neodyme's generic instructional examples for prior art is not a historical
case assessment.

A pre-go material change appends a numbered amendment with author, UTC,
reason, affected files and whether assessment had begun. Keep original
bytes addressable in Git, commit the amendment, obtain new timestamps for
changed files, and wait for a new go. Follow fx-zu4f.11's A1/A2 pattern; no
silent replacement or backfill. After go, a changed model rule or selection
rule invalidates confirmatory use: report the deviation and any subsequent
assessment as exploratory. Do not tune the rule against a case.

## 2. Sources and deterministic selection

Amendment A1 below supersedes the original one-catalog frame, before any
timestamp, go or case assessment. The finite frame is the **union** of:

1. All 11 `hacks` array entries in Copenhagen0x/solana-security-standard's
   [catalog at f6b6a1374984e0e491b644b65418da4e8e420686](https://raw.githubusercontent.com/Copenhagen0x/solana-security-standard/f6b6a1374984e0e491b644b65418da4e8e420686/hacks/hacks.json),
   data version 1.0.0 (`pool-standard`, identical pinned re-fetch).
2. Every top-level link in the *Exploits* section of
   [sannykim/solsec at 43f91b1dbb91c1454e6177625e46a510fe843822](https://raw.githubusercontent.com/sannykim/solsec/43f91b1dbb91c1454e6177625e46a510fe843822/README.md)
   (`pool-solsec`, identical pinned re-fetch). Follow each nominated
   report only to identify incident, date and stated root cause; record
   non-incident teaching/tool entries as excluded source entries too.
3. All five named incidents in Sec3's [*A Review of Recent Hacks*,
   2022-09-06](https://sec3.dev/blog/recent-hacks) (`pool-sec3`).

These public sources form a bounded retrospective frame, not ground truth
or a census. No fourth catalog is needed to execute the coordinator's
three-source instruction. Freeze original body hashes and retrieval times
in [sources.json](fx-qlek.17-cases/sources.json).

Select an incident when at least one framed source states its root cause
as a missing signer check, missing owner check, missing/incorrect PDA or
seed check, account substitution/spoofing, or account type confusion.
The catalog categories `account-validation`, `missing-authority` and
`type-confusion` count as such statements. For solsec, the nominated
report's root-cause narrative may resolve the index's terse description;
it cannot nominate an incident absent from that section. Inspect no fix
diff and perform no capability-rule assessment during this screen.

Oracle, key-compromise, off-chain, accounting/economic and front-end causes
are excluded. A generic recommendation or occurrence of the word “account”
does not itself establish one of the selected causes. No ranking, loss
threshold, popularity cutoff or target sample size applies. A source's
tool-coverage claims never determine an R1 verdict. A catalog's “owner”
label need not mean Solana's account-data owner.

Deduplicate by protocol and incident dates within seven days; normalize
obvious name aliases (CASH/Cashio, for example), and record every merged
source entry. Use incident date rather than article publication date.
Where a report supplies no incident date, retain a date-unknown row and
disclose the gap; merge only an explicit same-incident reference, never
two undated incidents just because the protocol matches. Every source
entry maps to a row, including non-incidents and unresolved reports.

`selection.tsv` records every framed incident's class, selected yes/no,
short verbatim selection-trigger quote and source location, or the reason
for non-selection. Preserve an unresolved-root-cause flag when access
fails; do not call that evidence of a non-matching cause. `entries.tsv`
records all original source entries and their merge destinations.
Sort selected cases by known incident date (unknown dates last), then
normalized protocol and incident id; assign C01 onward. `cases.tsv` is
the fixed list. There is no backfill for later misclassification, duplicate
discovery, non-Rust code or unavailable evidence. Preserve such rows and
reasons. A small or wholly unassessable sample is an admissible result.

The cases are fixed Solana Rust-program candidates, not verified code
pairs. Verify historical implementation identity after go; availability
of a report does not imply that either code revision is public. Label
affected-team reports versus independent retrospective analyses.

A1's enumeration gives 33 original source entries and 22 selection records
(20 incidents/disclosures plus two non-incident entries). One compound
SolSea/SPL Token report maps to two protocols; Royal Flush names their
combination, not a third independent incident. The 34 entry-to-incident
mappings include 12 repeats merged into existing records. Five cases
are selected and 17 records are not selected; both ledgers retain them.
Their root-cause classes and short source quotes determine selection,
never a prediction of R1's outcome. Where only publication is dated,
incident date stays unknown. A source's coordinated disclosure is labeled
as such, not presented as a verified historical theft.

Save fetched response bodies under `target/work/sources/`; commit only
the manifest, selection ledger and case list, not fetched pages or code.
`sources.json` names URL, exact retrieval UTC, status, body size, SHA-256,
local filename, version/revision where known and role. It includes redirects
and failed requests. A 404/error/challenge body supports only an access
failure. A SHA-256 without the saved body does not make the evidence
recoverable; tide-lead must retain the scratch snapshots through review
and archive them with any final evidence bundle. The committed selection
ledger preserves the catalog ids, labels and report pointers even if the
scratch copy disappears. Fresh changed pages never silently replace a
registered source. All controlled requests are serial and at least one
second apart, redirects included; 403/429 triggers a pause of at least
five minutes. No live-chain requests, contact, posting or execution.
Discovery used three search-tool calls, two containing three queries each.
The calls were more than one second apart, but tool-internal pacing is
unobservable. The saved request log verifies the interval/backoffs for
controlled page fetches only; it cannot certify each internal search
request. This limitation is disclosed before registration.

## 3. Frozen proposed rule R1

The normative model is this section. The API sketch in research/43 §2
explains it; later design alternatives cannot change R1 without the
amendment process. Opus has not adopted R1 as language design.

**Consumer/trigger/result:** an application handler trying to read trusted
account state, write it, authorize an action, or invoke another program
must receive the corresponding validated account capability. Its trusted
boundary checks runtime facts and either returns a bound capability or an
error before the sensitive operation. Admission forbids bypassing those
constructors and operations in application code. It does not invent an
application's policy from its instructions or infer who ought to sign.

R1 consists of these obligations:

1. A trusted entry adapter receives raw accounts, program id and instruction
   bytes from the runtime. It owns checked decoding, provenance validation
   and capability construction. Application handlers receive no raw
   `AccountInfo`, unchecked deserialize, arbitrary CPI, raw account writes,
   or capability constructors. Equivalent bypasses through aliases,
   generated code, traits, dependencies, foreign code or `unsafe` are
   forbidden in the admitted application or named trust holes. An unresolved
   bypass is an uncertainty, not evidence of successful enforcement.
2. `Signer` certifies `is_signer` for one account key in this invocation.
   It does not certify that the key is an administrator, human wallet, or
   appropriate party. A privileged operation must also bind this signer to
   the principal required by its declared, trusted policy. Application-
   supplied unchecked bytes cannot create that policy's root of trust.
3. `Owned<T>` certifies the runtime account owner equals the expected
   program identity and decoding validates T's layout/type discriminator
   and initialization requirements. It does not certify a particular
   instance, pool, mint, authority field or economic correctness. Public
   type tags cannot authorize arbitrary construction or deserialization.
4. `Pda<S>` certifies the actual account key equals derivation
   from the declared seed descriptor S, specified program identity and a
   verified bump. S includes the origins of dynamic seed inputs. Use a
   canonical derived bump when none is part of the declared policy; when
   the original policy supplies a particular bump, verify that derivation
   rather than rewriting its policy. PDA identity alone confers neither
   account ownership nor transaction signer status. `Address<K>` similarly
   certifies an exact declared address, including a sysvar/program identity.
5. Where the original intended operation requires account relationships,
   the operation needs a validated relationship capability tying those
   exact instances and signer together, rooted in a declared trusted
   identity (e.g. a deployment constant or already authenticated config).
   Equality between mutually attacker-supplied values proves only equality.
   A recipe/role specification chooses that root; R1 cannot learn the
   correct mint, pool or principal. Changing a correct-looking recipe to
   the wrong one can still admit a business-logic bug.
6. Capabilities retain key, invocation lifetime, access mode and checked
   data identity. Mutating/closing/reassigning/reinitializing an account
   or making a CPI which may change checked data invalidates dependent
   proofs before reuse; reacquire them by reloading and checking. Merely
   changing a signature to a wrapper does not establish this property.
   Duplicate mutable aliases cannot grant independent conflicting rights.
   Any fixed shape needing an unsupported lifecycle remains unassessable,
   not automatically rejected for a security success.

The trusted adapter, policy recipes and operational library are part of
the trusted computing base. Missing validation inside that boundary is
**not** a rejection credited to application admission. A generic raw-account
ban does not, by itself, establish a causal catch: show which missing
predicate prevents constructing the required capability while preserving
the original operation's declared policy. If a case can use valid
capabilities for the wrong policy, report admission of the buggy shape.
A shape permitted through a named trust hole is not a rejection; report
the admitted bypass and hole instead of counting paired support.

This rule says nothing about arithmetic, oracle quality/freshness/pricing,
market manipulation, economic design or general business logic beyond
its explicit validation predicates. Possession is insufficient if the
trusted constructor, policy or runtime is wrong. R1 imposes no novel
language syntax and makes no soundness or implementation claim.

## 4. After go: evidence and faithful comparison

Assess every fixed case in C01 order. Before its first assessment, record
UTC and source identities. Resolve the row's report URLs, then their public
buggy/fixed source links. Allowed evidence is public historical source,
public fix diffs, and code or precise check descriptions in the named
post-mortems and their source links. Search only to resolve that same
incident/report's missing source, not to find replacement incidents.
Never run upstream code or interact with live contracts in this research
check. Keep the complete fetched material and access failures.

For code, record repository, commit, file and line ranges. Compare the
public fix with its first parent unless its report names a different buggy
revision; identify the relationship explicitly. A merge commit's first-
parent diff is not assumed to isolate one fix. Distinguish intermediary
patches, mitigations and final repair. A report's code excerpt can support
an excerpt-level model; label omitted context and do not infer whole-
program correctness from it. A case with no public code in the report or
linked revisions is **not assessable**, with the reason; a cause label alone cannot supply a code
pair. If only one side has public code, record any supported one-sided
observation but leave the pair not assessable.

First identify the report's stated failure and intended authorization
policy, with source location and a short attributed quote. A tentative
causal diagnosis can accompany a reported failure; unrelated failures
in a wider thread cannot stand in for this incident. Record missing or
contradictory evidence without turning it into an absent bug.

For each side, make a source-to-model map: inputs, actual checks, trust
root, operation, helper calls, account identities, and relevant lifecycle
or CPI. Preserve the vulnerable side's missing checks. Do not insert the
fixed predicate into its validator during translation or replace its
operation with an impossible API merely to obtain rejection. Show any
adapter conversion as proposed work and name the checks it would need.
Model only the source's fix on the fixed side; no extra repairs, invented
principal, omitted interacting check, or stronger property. Report how
Anchor or the source's existing typed wrappers enforce the same checks.
If that faithful mapping is unavailable, the pair is not assessable.

A paper reasoning check against R1 is the planned measurement. Record
both derivations with the exact R1 obligation and evidence. No compiler,
SBF build, exploit reproduction, test-vector execution, or economic result
is inferred from it. If a later authorized spike executes anything, its
identity, command, environment and observed result appear separately;
it does not retroactively turn these paper verdicts into Tide v0 catches.

## 5. Verdicts and denominators

Record two independent fields and a pair summary. Permitted fields:

| Side | Verdict | Required evidence |
| --- | --- | --- |
| Buggy | `rejects buggy shape` | Faithful map; a specific missing R1 proof blocks the required operation outside the trusted boundary. No unrelated unsupported Rust feature used as the reason. |
| Buggy | `admits buggy shape` | Faithful map satisfies R1 despite the reported failure; name the policy/lifecycle/other gap. This is a negative for the sketch. |
| Fixed | `admits fixed shape` | The source's actual fixed checks supply every capability needed along the mapped path, without additional repairs. |
| Fixed | `rejects fixed shape` | A faithful fixed path cannot meet R1; name the excess restriction. This is a negative for the sketch, not a catch. |
| Either | `not assessable` | Name missing public code, inaccessible evidence, ambiguous revision, unsupported mapping, or another concrete uncertainty. Lack of evidence never counts as rejection or admission. |

“Paired support” requires `rejects buggy shape` AND `admits fixed shape`.
An otherwise fully assessable pair with `admits buggy shape` or
`rejects fixed shape` is an assessable negative. A pair with an unknown
side is incomplete/not assessable, preserving any
supported one-sided result. Do not count unknowns as negatives or successes.
Report total frozen, examined, pair-assessable, incomplete, paired support
and assessable negatives, with both per-side counts. Preserve all frozen rows
and reasons, including misclassification discoveries; no early stop after
support and no replacement for a negative. A screened-out catalog row is
not a failed R1 case. State the missing signer/owner/PDA/substitution/type-
confusion coverage actually present, without assuming each class appears.

Report disagreements between the catalog, report and code. Distinguish
Solana owner identity, token-account authority fields, signer status,
program id, PDA address and semantic relationships; none substitutes for
another. No catch rate, prevalence estimate, loss-prevention estimate or
claim of superiority follows from a small, retrospective labeled sample.
A wholly negative result still meets the research purpose.

## 6. Delivery and boundaries

The final record will include prior art, choices and recommendation,
feasibility limits, registration/proof/go identities, all fixed cases,
source maps, both verdicts, reasons, deviations and negatives. Cite this
registered file at its commit and digest, rather than maintaining a second
normative copy. Preserve all source requests in the manifest; after go,
append evidence to a separate assessment manifest so registered bytes
remain unchanged. A correctable transcription error keeps the old result,
explains the discrepancy and identifies invalidated conclusions before a
reassessment; never silently rewrite the rule to improve an outcome.

No v0 language, design/reference.md, tide-std or tide-admit changes. No
spike, compilation or heavy run in step 1. The coordinator owns OTS,
Beads writes, review rounds, merge and any eventual launch claim. Research
may be described only as research toward a direction. It is not an alpha
feature, a deployed defense, or evidence that Tide caught these exploits.

## Amendment A1: union frame before timestamp and go

Requested by tide-lead in `target/work/amend-frame.md`, 2026-11-18;
recorded by the builder at 2026-11-18T21:34Z before additional report
screening. No file has been stamped and no case has been judged.
The previous candidate is `ef356f9f5c757cc4846c80b2583b767c37700f0a`;
tide-lead retained its marker as `PROTOCOL-READY.v1`. At tide-lead's
explicit instruction the builder amends that commit, rather than adding
a second commit. The original Git object and its recorded hashes retain
the previous bytes for comparison.

The one-catalog frame was too narrow for the bead's public-source survey.
Section 2 now takes the union of that pinned catalog, solsec's pinned
Exploits section and Sec3's review. Selection uses a stated root cause
in a registered account-validation class, deduplication uses protocol
and incident date within seven days, and the ledgers preserve every
source entry and merge plus each selection quote or exclusion reason.
No fourth catalog, ranking or target count is added. R1, paired verdicts,
no backfill and the honesty rule remain unchanged. A new marker and
new hashes supersede v1; tide-lead still stamps before go.

For root-cause selection the builder read nominated-report narratives;
those narratives include repair descriptions and transaction examples.
No fix diff was opened and no R1 verdict was assigned. Medium refused
direct HTTP, so the saved web-tool extraction JSON supplies the two SolShield
report narratives. The tool labels them rewritten summaries; the manifest
and quote locators identify that transport. Quotes are verbatim within
the saved extraction, with no claim of byte identity to the original
HTML. The three frame snapshots themselves remain original HTTP bodies.
