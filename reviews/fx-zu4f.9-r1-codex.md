Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 1d2ccce (research record)

## Verdict

**changes required**. The registered pool is intact and completely enumerated. I found no eligible candidate in this sample, but several screening decisions or their stated evidence are wrong, and the record overstates the strength of registration and some exclusions. Those are blocking under this review's contract even if correcting them leaves eligibility at zero. Confidence: high in the concrete evidence discrepancies; this sample does not certify the remaining calls.

Candidate: `1d2cccef233b7a07e6daad8f7ec2cf04999611bb`, branch `fx-zu4f.9`.
Base: `1e673c525ca7d0a24c881d664a2bf98d78717cef`.
SHA-256 of the bytes from `git diff 1e673c5..HEAD`:
`489882aed58d97b1e041b0fe70ab02328d6438b1817a3c12e8d0ce2e31fd9783`.

Review only: no tracked-file edits, commits, builds, upstream execution, or `.beads/` writes. This report is the only authored file. The builder's report was treated as claims.

## Scope

Read `AGENTS.md`, `GOVERNANCE.md`, `BR_WORKFLOW.md`, both requested `br --no-db` outputs, the registered protocol, both CSVs, the entire research record, and relevant saved evidence. Independently checked the pool/table/assessment correspondence and counts for all 100 rows. Sampled these **30 rows** substantively, including every listed unsure row and the named nearest misses:

| Rows | Main evidence checked and assessment |
| --- | --- |
| B01, B05 | Public fixes, PR symptom reports, shutdown/channel source. The cited channel lifecycle and graphics operations are real; the historical-runtime exclusion is an inference, not a runtime experiment. B05's seed prediction needs correction (finding 6). |
| B07 | Alert holds stdout/stderr guards across IPC send/receive. E1–E4 call supported. |
| B09 | PR deadlock report and live versus commented source checked. The live HTTP read is real; the corrected E4 exclusion is supported. See delegation discussion. |
| B12 | Reentrant logger report; Mutex guard across IPC send at constellation.rs:345–346. E4 supported despite the hedged crash attribution. |
| B15 | Deadlock title; Condvar waits/notifies and strong-count check at event.rs:70–81,109. E4 supported. |
| B20 | Test-only assertion fix; zero-capacity channel selection and parking_lot dependency. E1 is permitted by the frozen exclusion's actual list; E3/E4 supported. |
| B21 | Issue #388 and single-threaded padding/CAS reproduction. E3 failure supported; memory unsafety is additional, not used to fail E2. |
| B24 | Commit, PR #11 and issue #9: conditional lost-wakeup description and performance observations. E2 failure supported for the named race. Condvar source checked as additional context. |
| B25 | One docs-only change. E1 failure supported by the explicit narrowing. |
| B31 | Bare deadlock title; worker Condvar wait/notify and external queue source. E4 supported. |
| B32 | Squash change and PR #8999 body. Its E2 explanation confuses uncertain attribution with absence of observed symptoms (finding 1). |
| B55 | First parent really includes the PR's new call while holding the transaction lock; prepare_sealing relocks it. E4 supported. |
| B56 | Stuck-client issue and restoration lock/abort_restore source. E4 supported. |
| B58 | Feature deletion is not forbidden by E1's test-only exclusion. Weak upgrade, engine-owned I/O service and shutdown report support E4. |
| N04 | Observed intermittent failures with uncertain diagnosis; actual IPC queries. E2 pass/E4 failure supported and relevant to finding 1. |
| N06 | Issue explicitly states missing log messages; poisoned-lock source checked. E2 failure is wrong (finding 1). |
| N11 | Reproduced crash and transmuted guard/reentrant lock source. E4 supported. |
| N18 | Assertion report, HTTP revalidation path, guarded channel sends and nested locks. E4 supported. |
| N19 | Commit/PR #2266/#2279, comments/reviews and Condvar source. The quoted bare race title is insufficient by itself; the thread also contains a CI-failure statement. Re-audit must identify the relevant symptom rather than rely on title equivalence (finding 3). |
| N21 | getmntent fix and failure statement; actual libc FFI at config.rs:764–771. E4 supported. |
| N22 | Only fix message is a race label; no PRs or linked issue. Current E2 pass widens the symptom rule (finding 3). Block allocation/destruction source supports E4 only if E2 passes. |
| N23 | Reported segfault/double-free and epoch unlink source. E2/E3 pass; memory-unsafety/reclamation exclusion correctly applied at E4. |
| N25 | Correct crossbeam-utils PR #42 fetched separately. Record omits an explicit observed output (finding 2). |
| N31 | PR #466 discussion, tests and review; acknowledgement of a racy static is not an observed malfunction of that race. E2 failure supported. |
| N34 | Saved GitLab issue, empty notes and missing MR response support E1. Fresh GitLab issue GET returned 403 during review, so today's result was not independently reproduced. |
| N35 | Radio race reproduction report and RF233 driver source. Hardware/interrupt exclusion supported. |
| N36 | Commit reports a remaining DMA-read bug rather than a repaired symptom at its first parent. The record discloses that distinction; no candidate established. |
| N37 | Race fix and linked DoubleVote report, signer lock nesting and external lock ownership. E4 supported; use the observed symptom and its uncertain attribution rather than the bare race title when explaining E2. |
| N41 | Advisory and issues #27/#28 report unsoundness and possible consequences. E2 failure is supported by the registered observed-symptom rule; memory-unsafety exclusion belongs at E4 if E2 were passed. |

Fresh read-only upstream checks resolved **29 sampled commits** with their exact SHA. All **35 saved source files** for these sampled rows matched fresh raw-source GETs byte for byte at the recorded first parent. I inspected the relevant cited ranges and patches; matching bytes alone was not treated as proof of an eligibility call. Also fetched the disputed issue/PR texts directly. Nothing upstream was built or run.

## Blocking findings

### 1. E2 rejects stated symptoms by demanding an unhedged diagnosis

**Location:** `research/41-qin-dataset-search.md:876` (N06), `:584` (B32); affected summaries at `:86`, `:104`, `:112`, `:113`, `:152` and the row table.

N06's linked [issue #12638](https://github.com/servo/servo/issues/12638) states “we are missing log messages if the lock is poisoned.” That is an explicit lost-output symptom, not just a change request. The comment's uncertainty concerns whether this explains particular missing panic logs. Protocol §3 requires the symptom, and expressly accepts lost data/output; it does not require an unhedged causal diagnosis. The record accepts that distinction for N04 and B56 but rejects it here.

B32's [PR #8999](https://github.com/openethereum/parity-ethereum/pull/8999) likewise says panics happen and refers to panic errors seen in issue reports. “Possibly” qualifies the explanation involving aborted Drop handlers, not the existence of those errors. It is reasonable to conclude that the double-exit deadlock fixed during PR development was absent from the first parent. It is not accurate to turn that into a claim that the PR states no observed crash/panic at the buggy revision.

**Minimum fix:** pass E2 for N06, and re-evaluate B32's E2 against the quoted observed symptoms. Assess the next criteria separately, identifying which reported failure and code path are being assessed; do not manufacture an eligible model. Recompute the row, tab/project and global first-failure counts and the E2 summary. If an exclusion remains, its reason must be the actual earliest failing criterion.

### 2. N25 calls an explicitly executed reproduction hypothetical

**Location:** `research/41-qin-dataset-search.md:1047` (and the claim of complete reading at `:1045`).

The correct [crossbeam-utils #42 comment](https://github.com/crossbeam-rs/crossbeam-utils/pull/42#issuecomment-422051850) does not stop at a possible-UB statement and a program. It ends: “This prints `"bar"` on my machine, which indicates UB.” The [author's response](https://github.com/crossbeam-rs/crossbeam-utils/pull/42#issuecomment-422174035) quotes that observation and accepts the problem. Calling the program hypothetical and saying there was no observed failure is false. This evidence was fetched independently during this review; the correct-repository thread was not present among the saved thread files I found.

There is a separate revision issue: the Default constructor was introduced and removed during this PR, while the first parent already had the Rc-based sharing problem. Those are distinct defects. An observation concerning intermediate code cannot simply be attributed to the first parent's Rc problem, but it cannot be erased either. Memory unsafety itself is not an E2 exclusion.

**Minimum fix:** quote and account for the executed reproduction, distinguish the Default-constructor defect from the Rc/RefCell defect, and justify the first failing criterion against the frozen first-parent model. Update counts if the call changes. Preserve the correct PR evidence so the claimed full read can be checked. Do not substitute a different buggy revision.

### 3. A bare race label is treated as a symptom, widening E2

**Location:** `research/41-qin-dataset-search.md:1020` (N22); analogous explanations at `:993` and `:1155`.

For N22, the complete [commit message](https://github.com/crossbeam-rs/crossbeam/commit/9d423946835b69edb395a5654b85649161f34d3b) is “channel: fix a race in disconnection”; commits/SHA/pulls returns `[]`, and it names no issue. The frozen protocol's E2 definition (`research/protocols/fx-zu4f.9-protocol.md:198`) enumerates observable wrong behavior. A race names a cause, without stating whether anything hung, was lost, panicked, or produced a wrong result. The record itself concedes that no symptom is described. A bare deadlock title is different because deadlock is an enumerated symptom. Deriving the lost MARK_BIT from the patch is E3/source analysis, not a quoted symptom report.

**Minimum fix:** fail N22 at E2 unless a symptom in an already allowed evidence source can be cited. Update its first-failure counts. Apply the same distinction throughout the delegated portion. For example, N19 has a CI-failure comment to evaluate, and N37 has an observed DoubleVote report with uncertain attribution; neither needs an invented rule equating all race titles with reported symptoms. Do not amend the registered protocol to permit this shortcut.

### 4. The registration account overstates what publication proves and what diff-check printed

**Location:** `research/41-qin-dataset-search.md:28`, `:41`.

“Self-attested until the repository is public” suggests publication will independently establish pre-screen order. Git author/committer times and these local fetch-log timestamps remain self-attested after publication. Publication establishes only that the commits existed by publication time, not when screening happened relative to their creation. There is no independent pre-screen timestamp attestation here.

The deviation also says diff-check printed both CSVs in full. Reproducing `git diff --check 1e673c5 891bdac -- research/protocols/fx-zu4f.9-pool` exits 2 and prints 166 flagged content lines. Every data row is exposed, but not literally every physical line: embedded-newline header text and final summary lines are omitted. This does not conceal an additional bug row, but the requested exact disclosure should not claim a full dump.

**Minimum fix:** keep the commit identities, self-attested chronology and disclosure; explicitly say publication does not prove screening order. Describe diff-check as exposing every data row and most CSV text, rather than both complete files. Do not retrospectively stamp it as evidence of pre-registration.

### 5. Limitations conflate safe design, missing implementation and protocol exclusions

**Location:** `research/41-qin-dataset-search.md:131`, `:139`, `:141`, `:166`.

The record mostly attributes the negative to what Tide v0 cannot express. That is too broad for the registered rules. Tide admits unsafe operations with an unsafe grant, but this protocol refuses that grant and excludes any memory-unsafety symptom even when the operation is tiny. The 400-line ceiling is a search/model budget. The ban on replacing an external operation and on reconstructing a missing primitive is a faithful-model restriction. These are different from safe Shared's Pure-closure design. Raw Mutex/RwLock and atomic interfaces are planned, as the protocol's own table says; their absence is not a promise that Tide will always prevent those shapes. No row reached E5, so weak-memory rejection did not decide a first failure here.

**Minimum fix:** add a compact, source-backed distinction between design restrictions, planned/unshipped primitives, and protocol-only modeling/budget exclusions, tied to the rows they actually decided. Qualify “rules this shape out by construction” as a safe Shared claim. Say which rules did not decide any first failure. Preserve the narrow protocol; correct the explanation of the negative instead of broadening the experiment.

### 6. B05 predicts a failed seed budget without a model

**Location:** `research/41-qin-dataset-search.md:343`.

The PR describes a residual shutdown race after the fix. That supports a warning about incomplete repair. It does not establish that a faithful Tide model would fail at least one of the frozen 1,000 seeds. No model was built, and a remaining possible interleaving need not occur within a finite random sample. The surrounding record correctly says no runs were made, which makes this categorical prediction unnecessary and misleading.

**Minimum fix:** retain the upstream residual-race disclosure as an additional concern, and say it was not modeled or tested. Remove the assertion that the fixed model would necessarily fail the seed budget. Its E4 exclusion can stand on the independently cited path restrictions.

### 7. B52 expressly skips a thread the frozen protocol requires

**Location:** `research/41-qin-dataset-search.md:762`; blanket method claim at `:63`.

The B52 block says the column-C PR #3505's thread was not needed because E2 passed. Protocol §2 (`research/protocols/fx-zu4f.9-protocol.md:163` and `:177`) requires every named column-C item and its comments/timeline to be read in full before a row can fail E3, E4 or E5. Passing E2 is not a stop condition for this reading. The omission also contradicts the record's claim to follow that method as registered. This is visible from the record itself, beyond the 30 substantive row sample.

**Minimum fix:** read the required #3505 thread by GET, then confirm or correct B52's earliest failure. Audit the delegated portion for the same early-stop shortcut and disclose any other omissions. Do not relax the registered reading rule after screening.

## Non-blocking findings

1. **Old Servo citation prefixes.** At `research/41-qin-dataset-search.md:304`, the January-2014 files live under `src/components/`, not `components/`. The saved files and fresh upstream copies resolve the intended citations, so this does not invalidate that exclusion. Minimum fix: include `src/` in the paths.

2. **Reproduction inventory is not mapped to rows.** `research/41-qin-dataset-search.md:177` lists reproduction directories and says none was opened, but protocol §6 (`research/protocols/fx-zu4f.9-protocol.md:421`) asks for existence/read status per row. No eligible model depends on these reproductions. Minimum fix: map the known directories to their row identifiers, or explicitly label an unresolved mapping; no upstream execution is needed.

## Reasoning

**Registration and contract.** The three digests match both the bead and `PROTOCOL-READY`, as well as their committed and WORK copies:

| Artifact | SHA-256 |
| --- | --- |
| Protocol | `56e9f6e14989d66346c229def74b70dbd943697553ac1be0edf337757f737d57` |
| Blocking CSV | `f7946a0d61f2478f3cf7a13ba008062c5106b45e0e7273dc15070a082ac8dac8` |
| Non-blocking CSV | `4f5ce8902b98686804d315e44ec3fe683b1ff8844c99c4001b64506cffe9c1a0` |

The bead description in `1e673c5` equals the current description. That commit reports 2026-11-18 14:13:02Z. Protocol/pool commit `891bdacc3d46a908ee05be1fc996cb28988e9008` reports 14:19:58Z. The bead hash-registration comment is 14:20 UTC. The first saved bug fetch in `target/work/fetch-log.txt` is B01 at 14:21:39Z. Thus the saved chronology is consistent with registration before the first bug-source screen, subject to finding 4 and the disclosed precommit CSV exposure. The local logs cannot independently prove a negative about earlier examination.

The protocol adds restrictive eligibility, modeling and evidence rules; I found no widening in the registered text. Its one-recording/two-replay rule is at least as strong as the body's replay requirement, and its 0..999/fixed-pass rule is unchanged. The record references the exact committed protocol with its digest. There is an internal wording inconsistency between protocol §0's request to reproduce it verbatim and §9's instruction to reference the committed copy; the user explicitly permits exact reference for this pass, so that is not a blocker.

**Completeness.** Independent CSV parsing produces 106 blocking records/59 data rows and 62 non-blocking records/41 data rows. There are 99 distinct normalized column-B identities, including the redox issue identity; one fix SHA has two bug-site rows. All 100 table entries correspond in order to the exports, with matching project and column-B prefix. There are 100 assessment headings and 100 first-failure entries: 99 commit/identity assessments plus the B59 pointer to the combined B48/B59 assessment. Both duplicate sites have E4 reasons. The recorded E1=2, E2=22, E3=1, E4=75, E5=0 totals agree arithmetically with the table and all tab/project rows. These are correctly transcribed counts of the current decisions, not independently validated final counts; findings 1–3 require reconsideration and recounting.

**Delegation.** The record explicitly discloses delegation at `research/41-qin-dataset-search.md:169`. The builder's spot check did catch B09's commented-out-lock citation error, and I confirmed that the corrected exclusion uses the live HTTP path. It did not catch N06's symptom/diagnosis mistake, N25's omitted executed observation, N22's bare-race shortcut, or B52's explicit reading omission. More than one delegated assessment is wrong. **Re-audit the whole delegated B07–N41 portion against the unchanged protocol**, especially complete thread reading and the E2 boundary, rather than correcting only this sample. This is an audit of the existing pool, not authorization for another search round. The reviewers should re-check the resulting corrections under the governance rules.

**Negative and attribution.** The record correctly credits Qin, Chen, Yu, Song and Zhang and the pinned rust-study artifact, identifies the sheet/tabs, and says the dataset's authors selected the bugs. A fresh GET of the pinned [artifact README](https://raw.githubusercontent.com/system-pclub/rust-study/403b06f30809d10d2bb066d2fbcd33caf3446d17/README.md) corroborates the dataset size and column-B/tab locations. It explicitly says no model, property or seeds were run and makes no catch-rate claim. Those are useful limits. Correcting the evidence and limitation statements above is required before approving the negative; the fact that multiple independent E4 grounds remain is not a reason to keep false evidence or first-failure counts.

**Verification limits.** No Tide or upstream builds, model runs, seed sweeps or replays were performed. N34 used saved evidence after a fresh GitLab GET returned 403. `research/39` is absent from this frozen tree; the available earlier ground-by-ground comparison is `research/38-external-concurrency-search.md:277`. Neither registration order nor the remaining 70 substantive calls is independently certified by this sampled pass. The tracked working tree remained clean and the frozen diff hash was rechecked after writing this report.

DONE
