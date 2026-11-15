# E1 gate report: paired learnability on the checked toolchain

Date: 2026-11-15. Beads: `fx-ic2`, report `fx-ic2.11`.
Protocol: [protocol.md](protocol.md), frozen 2026-10-31; task set:
[tasks.md](tasks.md). Gate revision R:
`675c26e126df6db6090a94ddde6acc8004fe454d`.

**All three protocol §8 thresholds are met on the fixed sample's point
estimates. E1 passes those thresholds.** This is a result for this task set,
writer and checked toolchain, with the limitations below. Tide stayed within
the protocol's margins rather than beating Rust: Rust passed first check more
often (59/60 against 56/60), final pass tied (58/60 each), and Tide needed
1.46 times Rust's iterations against a 1.5 limit. Both arms worked near
ceiling. It is not a confidence-bound claim: the information-only
iteration-ratio interval crosses 1.5. A seeded sampled regrade of 12 trials
reproduced every stored first and final grade (see Reproduction).

## Measures and verdict

Each arm has 30 tasks × 2 independent replicates = 60 trials (120 total).
The order was fixed before the first writer: sorted triples shuffled with
Python `random.Random(20261031).shuffle`. There was one fresh writer session
per trial, `claude-opus-5-5`, default effort, Claude Code 2.1.321, with two
sessions allowed at once. The Tide arm received the frozen reference and
companion; Rust received neither. Both received the arm-rendered task and
preface. The budget was 20 check/test calls and 40 minutes per session.

| Measure | Tide | Rust |
| --- | ---: | ---: |
| First-check F | 56/60 = 93.3333% | 59/60 = 98.3333% |
| Final pass P, after reviews | 58/60 = 96.6667% | 58/60 = 96.6667% |
| Iterations I, passing trials only | 86/58 = 1.482759 | 59/58 = 1.017241 |
| All-60 iteration mean, failures counted as 20 (information only) | 126/60 = 2.10 | 99/60 = 1.65 |

| Protocol §8 threshold | Exact comparison | Verdict |
| --- | --- | --- |
| F(Tide) ≥ 0.8 × F(Rust) | 56/60 ≥ (4/5) × 59/60; ratio 56/59 = 0.949153 | **MET** |
| P(Tide) ≥ P(Rust) | 58/60 ≥ 58/60; ratio 1 | **MET** (tie) |
| I(Tide) ≤ 1.5 × I(Rust) | 86/58 ≤ (3/2) × 59/58; ratio 86/59 = 1.457627 | **MET** |

F includes compilation of the writer's tests and library acceptance suite at
the first snapshot, not execution of that suite. P includes all writer and
acceptance tests, literal structure checks, concurrency and integrity decisions.
I counts **every** check/test run, including any after the first green run,
but averages only over trials passing final. If either arm has no passing
trial, threshold 3 is **not met** (§7.3); both arms have 58 here. Failed trials
remain failures in F/P and contribute 20 only to the informational all-60 mean.

The pre-review mechanical final count was Rust 55/60 and Tide 58/60.
Three Rust integrity holds were reviewed and their acceptance passes stood,
yielding 58/60 in each arm. No other grade was changed or trial excluded.

### Information-only uncertainty

These are the official 95% percentile bootstrap intervals: 10,000 draws of
30 tasks with replacement, retaining both arms and both replicates of each
drawn task together, `random.Random(20261031).choices` over sorted task IDs.
Percentiles use linear interpolation at index `(n − 1) × p`. All 10,000
resamples are defined for each interval. Ratio means Tide/Rust; difference
means Tide−Rust. Rate differences below are in percentage points (pp).

| Measure | Point ratio | 95% ratio interval | Width | Point difference | 95% difference interval | Width |
| --- | ---: | --- | ---: | ---: | --- | ---: |
| F | 0.949153 | 0.862069–1.017857 | 0.155788 | −5.0000 pp | −13.3333–1.6667 pp | 15.0000 pp |
| P | 1 | 1–1 | 0 | 0 pp | 0–0 pp | 0 pp |
| I | 1.457627 | 1.285714–1.637931 | 0.352217 | 0.465517 runs | 0.293103–0.637931 runs | 0.344828 runs |

The F and I intervals are wide for 30 tasks. In particular the rounded
I-ratio interval **1.29–1.64 crosses 1.5**. It is information only, not an
additional pass condition. P's zero-width interval is a property of this
paired sample: only C3 fails final, in both replicates of both arms. It does
not establish equal performance on unseen tasks.

## Published trial records and grading evidence

[per-trial.json](gate-results/per-trial.json) publishes all 120 §7.4 records;
[measures.json](gate-results/measures.json) publishes the aggregates and
unavailable-field explanations. These are the official outputs, unchanged.
The [records archive](gate-results/records.tar.gz) contains the underlying
records and sources. Archive-relative paths in this report become files after
`gate_reproduce.py unpack` below.

For slot NNN, `sessions/NNN-1/` holds:

- `snapshot-01/source.rs` (first), `final/source.rs` (final), and all intermediate
  `snapshot-NN/source.rs` with `results.json` for every writer check/test call;
- `session.json`, `trial.json`, `grading.json`, `first-grade.json`,
  `final-grade.json` and `final/identity.json`;
- a derived `writer.jsonl` containing only the last result event's cumulative
  `usage`, without prompts, assistant prose, tool calls or credentials.

`slot-NNN.json` retains the original slot/attempt/audit records. Its old audit
token field is preserved as evidence, not used for reporting. The unpacker
changes only each slot attempt's absolute session path in its new copy so the
measures script can find the archived session. Other original paths remain
provenance, including the pinned toolchain, vendor and driver paths required
for grading on the gate machine.

Every per-trial entry gives each run's verdict, exit codes, available diagnostic
codes/lint names and failing-test names; first/final grade summaries; wall
seconds; distinct assistant-message turns; four corrected token totals; peak
context; Tide reference-read sections; first-snapshot bytes and hole count;
iterations; final review decisions; reruns and audit counts. Full diagnostic
and test output is retained in snapshot results and grade records. A structure
failure can have no failing test name: all C3 tests passed.

| Informational observation | Tide | Rust |
| --- | ---: | ---: |
| Writer check/test runs, including final failures | 88 | 61 |
| Writer wall seconds, summed across sessions | 2464.37 | 1865.15 |
| Median session seconds | 40.35 | 30.58 |
| Session range, seconds | 27.08–71.60 | 25.30–40.80 |
| Distinct assistant-message turns | 245 | 126 |
| Input tokens | 490 | 252 |
| Cache-read input tokens | 5,156,350 | 569,596 |
| Cache-creation input tokens | 2,092,183 | 927,510 |
| Output tokens | 215,236 | 158,148 |
| First-snapshot byte range | 2010–6338 | 2220–5681 |
| Recorded `todo!`/`hole!` holes in first snapshots | 0 | 0 |

Wall totals are sums, not elapsed batch time. Peak context and reference reads
are per trial in the JSON, rather than inferred from cache tokens. Recorded
reads show requested sections, not proof of comprehension. The hole count is
the harness's recorded source-text measure, not proof that code is complete.

### Failure classification, including evidence

`classification.md` in the archive preserves all nine evidence-bearing
classifications. `evidence/ic2.7-c3-disputes.md` is the authoritative C3
adjudication: original classifier suggestions of a suite dispute or harness
defect in 044 and 068 were rejected and corrected. The original suggestions
remain visible with correction notices, not silently removed.

| Slot / trial | Stage | Final classification | Evidence in stored snapshot |
| --- | --- | --- | --- |
| 029 C10/tide/2 | First-check | Ordinary bug | `snapshot-01/source.rs:134,142`: non-`move` child borrows a `!Sync` channel receiver; E0277 in writer tests. Final uses an owning closure. |
| 033 C10/tide/1 | First-check | Rust habit leaked | First source line 13: `impl From<SrvError> for io::Error`; FX0401 rejects placing a foreign-trait/foreign-type impl here. Reference §5.2:319–320 requires the type's or trait's module. Final uses an inherent conversion. |
| 047 B4/tide/2 | First-check | Rust habit leaked | First source lines 47,50,100: `Fs::exists` treated as bool; E0308/E0600. Reference §6.3:386–389 says “as std::fs”, returning `io::Result<bool>`; §6.9:557 bans `Path::exists`. |
| 051 B6/tide/1 | First-check | Rust habit leaked | First source lines 62,63,72,79: same bool-returning `Path::exists` habit in writer tests; E0600. Final adds `.unwrap()`. |
| 112 A10/rust/2 | First-check | Ordinary bug | First source line 107 calls `as_ptr_range` on `str`; E0599 in a writer test. Final calls it on `as_bytes()`. |
| 032 C3/tide/2 | Final | Ordinary bug: literal structure slip | Final line 111 calls `sync_channel::<Msg>`; required `sync_channel(` substring absent. All test rows code 0. |
| 044 C3/tide/1 | Final | Ordinary bug: literal structure slip | Final line 30 calls `sync_channel::<i64>`; same missing substring; all test rows code 0. |
| 068 C3/rust/1 | Final | Ordinary bug: literal structure slip | Final line 22 calls `sync_channel::<Msg>`; same missing substring; all test rows code 0. |
| 116 C3/rust/2 | Final | Ordinary bug: literal structure slip | Final line 22 calls `sync_channel::<Msg>`; same missing substring; all test rows code 0. |

The full classification file also cites the corresponding grade JSON lines
and reference lines; `first-source.rs` there maps to `snapshot-01/source.rs`,
and `final-source.rs` to `final/source.rs`. Reference line numbers refer to R.

Counts: first-check Tide = 3 Rust-habit + 1 ordinary bug, Rust = 1 ordinary
bug; final Tide = 2 ordinary bugs, Rust = 2 ordinary bugs. **Toolchain defects
0, reference gaps 0, design gaps/contradictions 0.** These are the reviewed
classifications of the observed failures, not claims that those defect types
cannot exist. Any toolchain defect would still count as failure (§7.4).

All four C3 disputes were adjudicated **not valid**, and the failures stand.
The task explicitly requires a literal source substring, which §6 checks by
text search even in comments. Each program uses bounded channels correctly
but misses that exact spelling by inserting a turbofish. All writer and
acceptance tests pass. No suite test was removed in either arm; there was no
failed acceptance test to remove, and the structure requirement was not
relaxed. No adjudication regrade was needed.

### Concurrency and integrity reviews

The archive's `grader-reviews.md` contains one fresh read-only Opus review per
record: 28 concurrency reviews (14 per arm) and 3 Rust integrity reviews,
31 entries total, each with source-line citations. Each reviewer saw only that
trial's final snapshot, grade, task and rule, without a reference solution;
at most two ran at once. The coordinator read counts/verdicts rather than
writer sources. For these citations, `source.rs` maps to that slot's
`final/source.rs`.

All of the following concurrency verdicts are **PASS**; they do not override
the independent C3 structure failures:

| Task | Reviewed slots (arm/replicate) |
| --- | --- |
| C1 | 000 rust/2, 030 tide/2, 056 tide/1, 107 rust/1 |
| C2 | 057 rust/1, 063 tide/2, 067 tide/1, 076 rust/2 |
| C3 | 032 tide/2, 044 tide/1, 068 rust/1, 116 rust/2 |
| C4 | 035 tide/1, 049 rust/1, 104 tide/2, 117 rust/2 |
| C5 | 012 rust/1, 062 tide/2, 095 rust/2, 101 tide/1 |
| C6 | 027 rust/1, 082 tide/2, 083 tide/1, 086 rust/2 |
| C9 | 005 rust/1, 060 tide/2, 087 tide/1, 102 rust/2 |

For example, slot 032's review cites producers at final lines 110–117 and
the concurrent merge at 119; slot 027 cites stop checks at 16–17 and the
4096-byte chunk bound at 4,19,25. The archived file provides all per-trial
citations, not just these examples. C10's concurrency is exercised by its
checked case rather than this manual-review list (§6).

| Integrity flag | Decision and cited evidence |
| --- | --- |
| 094 B8/rust/1 | **STANDS**: normal binary exit at final line 65, after stdout/stderr flush at 61–64; writer tests call `run`, not `main`. |
| 103 B8/rust/2 | **STANDS**: stdin-error exit at 61 and normal status exit at 68; tests call `process`, not `main`; no forged test log. |
| 109 B5/rust/1 | **STANDS**: required binary status exit at 91 after output at 89–90; tests call `run`, not `main`; no forged test log. |

No reviewed integrity verdict was overturned, and Tide had no integrity flags.
The mechanical hold was code 1 solely for “process termination” in these
three records. Human-readable review findings justify accepting their
otherwise passing tests; the official measures apply those decisions.

**Structure checks met only by a comment: 0 Tide, 0 Rust.** The official
script checks this by masking comments while retaining literals and rerunning
the frozen structure search. Mentions in comments would have sufficed under
§6, but did not determine any passing structure verdict here.

## Readiness, batches and audit

The archive's `evidence/fx-ic2-comments.json` retains the parent readiness,
run-procedure, completion, audit, adjudication, official-measures and owner
decision comments. The readiness record of 2026-11-14 initially names
`147eb9a`; that run directory was archived without starting writers. The
19:30Z run-procedure update records the fresh readiness at **R=675c26e** after
the CLI 2.1.321 fix: harness tests, both canaries, four-slot dry run, dry-clock,
store scan and live restricted-tool probe passed before gate writers started.
All prerequisite beads were closed; 30/30 reference solutions per arm passed
sandboxed checks, structure and suites, Tide through the driver and production
std. The earlier reference proof is reused only where its inputs did not change.
FX conformance recorded 361 PASS, 4 AWAITS, 0 FAIL; the awaited codes were not
in the reference. FX0920, FX0921 and FX0516 were excluded from E1 gating by
the recorded 2026-11-09 owner decision.

A pre-session C3 case-2 precision correction (`fe1e39d`, 2026-11-14) reduced
items per input from 10,000 to 600 in both arms and the task text, retaining
8 inputs, capacity 2, 20 seeds and the 10-second limit. It put the debug
OS-thread simulator reference within that limit. It still crosses the
512-item/input and 4096-merged boundaries, but narrows volume coverage:
faults appearing only beyond 600/input or 4800 merged (for example a wrapping
1024- or 8192-slot buffer) can pass. This was recorded before any writer,
not adjusted to writer outcomes. No suite changed after the run freeze.

Trials 1–3 (slots 000–002) were **gate trials**, included in the 120 throughout.
Their short sessions and suspiciously low audit output-token totals triggered
the archived `evidence/ic2.3-anomaly.md` investigation. It verified real
writer edits, byte-identical first/final snapshots, frozen suite application,
actual acceptance execution and no reference leak. Their grades stand. No
host-class move occurred. The initially discussed Boat move was deferred.

The first three-trial slot was booked 00:10–02:55Z on 2026-11-15. Subsequent
batches continued under the owner's recorded go with the same backend; the
completion record at 06:10Z reports 120/120 at R. The last window ran
05:15–06:07Z and was released early. Each batch ran through heavy.sh in an
E1-exclusive window, with load, quiet-window, timer and store-scan preflight.
The gate's sum of writer times was 72.2 minutes and grading 19.7 minutes;
parallel sessions mean these are not elapsed wall time.

| Audit outcome, gate trials only | Tide | Rust |
| --- | ---: | ---: |
| Completed attempts, all audit-passed | 60 | 60 |
| Infrastructure faults / reruns | 0 / 0 | 0 / 0 |
| Second-fault failures | 0 | 0 |
| Forbidden-tool observations / audit errors | 0 / 0 | 0 / 0 |
| Denied outside requests | 0 | 0 |
| Confinement breaches | 0 | 0 |
| Recorded API-retry events | 0 | 0 |

There are also zero session-level breach files, including sessions outside
slot records. The separate readiness probe intentionally made three outside
requests, all denied; they are not gate-trial audit events. Both stored store
scans passed (1,858,505 files sized, 29,541 hashed, no matches or unreadable
skips). A pre-gate ENOENT race when an unrelated Nix chroot disappeared was
retried under the recorded rule, not scored as a writer failure.

The harness sandbox gives each build/test isolated writable storage, read-only
recorded toolchain inputs, no external network, 4 GiB memory, no swap,
1 GiB tmpfs `/tmp`, `TMPDIR=/tmp`, CPUWeight 1 and four Cargo build jobs.
The tmpfs fixtures avoid the Rust B9 fsync/shared-disk timeout asymmetry seen
in readiness. This is the E1 harness sandbox, not a claim of D13's general
unreviewed-branch guarantee.

## Input identities and durable storage

The full `state.json` and its **entire identity record** are stored unchanged
in the archive (including toolchain versions, all machinery hashes, file
hashes, writer argv, store closures, host, limits and order). Its SHA-256 is
`948fbf44f2b919a652ca2b33fd85b926161392b69cc59a1b773bb4ecf84f7ba6`.
The important identities are:

| Input | Identity |
| --- | --- |
| R / harness repository revision | `675c26e126df6db6090a94ddde6acc8004fe454d` |
| Order SHA-256, 120 triples, seed 20261031 | `9d0951c1c2301efa926f91a70f9bc1dfd9708015b6cbf1bc2c106ca818265c50` |
| Crates Git tree | `65451f47bc3c2bfe75738cb7f003c03f99f2d794` |
| Frozen full trial tree at R, 108 tracked files | `74485bf20efbbac1b989d729c17687ca9c177f44749529f7ae0a8971f5dcdb10` |
| Frozen suites, independent manifest, 37 files | `d22b88f43ed4c60a227b1a05dc6b9f6bf90ada3c1f9678d46abdb3e42f540eb5` |
| Harness suites-tree SHA-256 (different framing) | `266980350754c1c9a491e2cdfcc1a5bb2bf14bed57a872aab375febd644ca866` |
| tasks.md SHA-256 | `91cc5d486086c09dd5f524af01a6f4a233cc95eb6e5cf1a8d20785e91e157dd7` |
| Reference SHA-256 | `c5b12ed238583166e93f77acd7d5d58b610cbbbb12bb480fb548e10270b5057c` |
| Companion SHA-256 | `3b2f2dc8fc17fb1311adb232009f9bcff97b40c9c1c5fb3488fa6c3cc4e00c36` |
| Driver SHA-256 | `4bdaeab417f7a2b6c7dbdfc14803bbd94f1ee3c10ab6b9f8fcfdc4a67b97be93` |
| Production std SHA-256 | `53a19a6b65743390bda272a4589b17a8f6bc0c1a92c072bd16ba269317ec4f9e` |
| Rust arm | rustc 1.96.0 `ac68faa20`, Cargo 1.96.0, edition 2024 |
| Tide arm | nightly-2026-08-09, rustc 1.99.0-nightly `771916f90`, Cargo 1.99.0-nightly |
| Rust dependency lock SHA-256 | `d786dc06f8ba1e6c9b616638b9e90c213ec2a4b7b563767da43878ae8ee1ec1a` |
| Workspace lock SHA-256 | `049a9c2efef2e9327de93819d18e9bc82acf3104c3e14ffdfaa195f0b0a7c122` |
| Model / harness | `claude-opus-5-5`, effort default; Claude Code 2.1.321 |
| CLI SHA-256 | `d0180c5108bd90cb17ee7666bb8a2fc9b1e1ea9f3ff5cb923246439bea8b1d41` |
| Host | kernel 7.2.6-76070206-generic, Python 3.14.8, 16 CPUs, Bubblewrap 0.11.0 |
| Boot ID | `b5a8bafb-dd96-43eb-abc5-27412eac6e76` |
| Measures script revision | `aead157dbb3c21517adf35fe0b78148d80aee473` |
| measures.py SHA-256 | `df4b9a85d980c7ddd1fecdf72d40652ccf2e37a5caaa9c114a7abccd9da16e83` |
| Official measures.json SHA-256 | `c138c6ff2736909d966f15b3d2ddd227805fdb75859b73c6da072d8c8260b649` |
| Official per-trial.json SHA-256 | `fac93ff682bb073371502a5658c5091c69cc146a42b38eb4e650bcb7c1ec1535` |
| grader-reviews.md SHA-256 | `586485dad7da36fc795a22df4963a1d5387fc96f78a8353297ba056982749b0f` |
| classification.md SHA-256 | `d65faacaea7339441021ebdd690d70e7e71c98c22c190dab37b90afdf189afdb` |
| C3 adjudication SHA-256 | `24821a4d52b6f2fe30e2f62571daaba4693451b293c18b5cab4360f3870c97ea` |
| Trial-1–3 anomaly report SHA-256 | `d93234f14b3207f5c094e3653c2f8ef2fa880289ef10fed8640b1cf7ece5d861` |

The full-tree digest is SHA-256 of sorted lines `sha256(blob)  repo/path\n`;
`evidence/frozen-tree.sha256` stores that manifest at R. It differs from the
initial readiness record's `95cae340…` because that record precedes the CLI
fix; the suites and task hashes are unchanged. The identity's `suite_freeze`
field says “measured only”; the parent pre-session freeze comment supplies
the freeze declaration. Actual session launch omitted `--safe-mode` and used
the explicit disabling settings in the final state identity, superseding the
earlier readiness argv example.

[records.tar.gz](gate-results/records.tar.gz) is **1,638,424 bytes**, with
1,393 files totaling 33,738,599 uncompressed bytes. It contains all 120 first
and 120 final snapshots, all 149 writer-run snapshots/results, slot/session
records, full grades, audits, review/classification evidence, readiness records,
manifest and derived usage events. First snapshots are included among the
149 run snapshots. Its SHA-256 is:

`f8b6f1d7aa798b62bc8b3657591ceeee0609951cd0e919d7f5faa9a933b00f72`

The visible official JSON files are 8,611 and 306,880 bytes, respectively.
[SHA256SUMS](gate-results/SHA256SUMS) covers the archive and both JSON files;
the archive has an additional per-member `SHA256SUMS` before relocation.

Raw writer/broker streams, stderr and grading-attempt logs remain outside the
repository under `/home/lucca/.cache/eon-agents/tide-e1-gate/`.
`evidence/raw-logs.json` records 1,563 distinct raw-log paths with byte sizes
and SHA-256, including the original full `writer.jsonl` streams and batch logs.
That manifest's SHA-256 is
`5285def60b8e11a920da15a11784ea77b09d9b6645fc8dea5d6c1ce73df9e7f6`.
The derived usage-only files are explicitly not the original raw logs.
No config-home, launch environment, credential file or writer working directory
was copied. A credential-marker grep over all extracted stored files and the
visible JSON files passed; its fail-closed reproduction is below.

## Interpretation, coverage and limitations

- **Near ceiling and short sessions.** First checks passed in 56/60 and 59/60,
  final in 58/60 each. Sessions were about 30 seconds in the initial batch;
  overall medians were 30.6 seconds Rust and 40.3 seconds Tide. This mostly
  measures rapid, small, single-file solutions with self-written tests; it
  provides little separation on difficult maintenance or larger programs.
- **One writer model.** Both arms used `claude-opus-5-5`. On 2026-11-15 the
  owner declined a second-model run (“nah, lets focus on making progress”),
  so this is not evidence of a model-independent learnability advantage.
- **All-Claude grading chain.** Suites/reference readiness and the semantic
  concurrency, integrity, classification and C3 adjudication chain were
  agent-produced/reviewed through Claude. Mechanical execution constrains
  judgments, and reviewers were fresh and lacked reference solutions, but
  shared-model blind spots remain. Codex's measures/report work is not an
  independent semantic regrading chain.
- **Only 30 tasks × 2 replicates per arm.** Task-cluster resampling keeps
  replicates together; it does not create more independent tasks. The
  I-ratio interval 1.29–1.64 crosses 1.5 and remains information only.
- **All four final failures are literal C3 structure-check slips**, two per
  arm, adjudicated to stand. They distinguish exact spelling rather than
  observed behavior; all corresponding tests and concurrency reviews passed.
  Keeping the frozen requirement makes the threshold outcome faithful to
  the protocol while limiting the meaning of “final failure”.
- **Token-accounting defect.** `coordinator.audit` sums message-start usage
  placeholders for output tokens. In slots 000–002 it recorded 419/522/24
  versus the actual cumulative 2072/3112/3443. The official measures read
  all four token totals from each writer stream's last `type=result` event,
  without falling back to audit tokens; the archive stores those totals.
  The frozen harness was not repaired mid-run. Tokens do not gate E1.
  “Turns” here means distinct assistant-message IDs, not CLI `num_turns`
  (2/3/3 versus 3/4/4 in those initial sessions).
- **Tide lint names unavailable under short error format.** The driver forces
  `--error-format=short`; warnings can omit lint names/notes. The measures
  retain available FX/rustc E codes, unnamed-warning counts and 17
  field-availability entries, rather than inventing the missing lint names.
- **Held-out resemblance (§2).** Tasks are new but related to earlier teaching:
  companion 15.1 arguments/files → B1, B2, B5; 15.2 deadline outcomes → C5;
  15.3 quantified contracts → C7, C8; 15.4 append-only log → B4; reference
  6.3 atomic replace → B9; e1-dialect `ring` → C7 and `scan` → A10
  `with_word`; e1-origins `group_words` → A1's drop case. Strong performance
  on these tasks is evidence about related reference-taught patterns, not
  wholly unfamiliar concepts. Read into the results: on the 20 tasks with no
  such resemblance (40 trials per arm), Tide F 37/40 against Rust 40/40
  (ratio 0.925), P 38/40 each, I 56/38 against 38/38 (ratio 1.474), so all
  three thresholds still hold there, with an even narrower
  iteration margin.
  On the 10 resembling tasks: F 19/20 each, P 20/20 each, I ratio 30/21 =
  1.429. Computed from `gate-results/per-trial.json`. No writer program or task was copied into
  teaching material for this report; §2 remains binding until publication.
- **Intended asymmetries (§11).** Tide reads a long reference, has FX checks,
  generated contract tests and feature-specific structure requirements;
  Rust has no reference and uses stable 1.96.0, Tide pinned 1.99 nightly.
  A bounded static method/path audit of all 60 Tide finals identified **0**
  uses of APIs stabilized after 1.96, against the pinned nightly's local
  core/alloc/std stability annotations. That is not an exhaustive language
  compatibility proof; no cross-toolchain compilation was performed.
  Rust allows in-process test-forging constructs Tide admission rejects;
  identical integrity searches plus the three reviews mitigate, but do not
  authenticate, acceptance-process output. Tide simulated I/O/time and Rust
  real-I/O/time suites still differ; tmpfs removes the known disk exposure.
- **Delivery limits (§5).** The checked driver does not supply D14's typed
  holes or full machine-readable fix plans; diagnostics have ordinary
  compiler suggestions. A failing simulation seed is not accompanied by
  companion 17's source/build-bound replay manifest (`fx-nkv.7` is not a
  prerequisite). These limits count against the delivered Tide arm.
- **Coverage (§11).** The suite exercises ownership/borrowing, arithmetic,
  traits/generics/errors/derives, capability I/O, durability, time, structured
  tasks/cancellation/isolation, Shared, channels, parallel regions, contracts
  and TCP. It does not test Rand, HTTP (`Net::get`, Url, simulated HTTP),
  Proc, simulated distributed faults/crashes, multi-file/granted packages,
  or `tide review/fix/eval/explain/fmt`. No broader capability is inferred.

## Reproduction

### Measures: run in full, matching exactly

Use Python 3.14.8 (the recorded environment; the scripts use only the standard
library). From a checkout containing this report, run the following with new
output paths. No compilation, model call or grading is involved. The unpacker
verifies the archive and all member hashes before relocating copied slot paths.

```bash
set -euo pipefail
REPORT_ROOT=$(git rev-parse --show-toplevel)
T=/home/lucca/.cache/eon-agents/tide-e1-gate
E1_COPY="$T/ic2.11-reproduction-records"
E1_MEASURES="$T/ic2.11-reproduced-measures"
cd "$REPORT_ROOT"
(cd trials/e1-checked/gate-results && sha256sum -c SHA256SUMS)
python3 trials/e1-checked/gate_reproduce.py unpack "$E1_COPY"
# Exit 1 means absent, exit 0 means credential marker found, exit 2 is an error.
if grep -rIl -- 'sk-ant' "$E1_COPY" trials/e1-checked/gate-results/*.json; then
  echo 'credential marker found' >&2; exit 1
else
  rc=$?; [ "$rc" -eq 1 ] || exit "$rc"
fi
python3 trials/e1-checked/measures.py "$E1_COPY" "$E1_MEASURES"
cmp trials/e1-checked/gate-results/measures.json "$E1_MEASURES/measures.json"
cmp trials/e1-checked/gate-results/per-trial.json "$E1_MEASURES/per-trial.json"
sha256sum "$E1_MEASURES/"*.json
```

**Executed 2026-11-15** on the report candidate based on `27921c9`, Python
3.14.8, Linux x86_64, from
`/home/lucca/pjs/lab/tide/.worktrees/fx-ic2.11` and the displayed external
output paths. Both `cmp` commands exit 0:
byte-identical to the stored official files and to `measures-r3/`. Output
hashes are exactly the official hashes above. To repeat, choose new copy and
output names: the helper and measures script refuse an existing destination.
The two non-compiling helper tests (`python3
trials/e1-checked/gate_reproduce_test.py`) also passed, as did all three Bash
snippet syntax checks and the candidate diff check. This proof is for
stored-record reproduction, not execution of writer code.

### Grading: coordinator-run, not executed by the report author

The deterministic sample is
`sorted(random.Random(20261031).sample(range(120), 12))`, zero-based slot
numbers in the frozen order. It covers both arms (9 Rust, 3 Tide).

| Slot | Trial | Stored first | Stored final |
| --- | --- | --- | --- |
| 005 | C9/rust/1 | PASS | PASS |
| 009 | B10/rust/1 | PASS | PASS |
| 026 | A4/rust/1 | PASS | PASS |
| 028 | A4/rust/2 | PASS | PASS |
| 030 | C1/tide/2 | PASS | PASS |
| 031 | C7/rust/1 | PASS | PASS |
| 039 | B3/rust/1 | PASS | PASS |
| 048 | B7/rust/2 | PASS | PASS |
| 060 | C9/tide/2 | PASS | PASS |
| 065 | C8/tide/1 | PASS | PASS |
| 093 | C7/rust/2 | PASS | PASS |
| 097 | A7/rust/2 | PASS | PASS |

The coordinator books an **E1-exclusive** slot with eon-overseer, providing
`E1_GRANT_UNTIL` (its granted end epoch, also used as the booked end) and
`QUIET_LOG` (the inbox receiving
QUIET OVER). The 2026-11-15 14:28Z bead comment records the booked window
**16:00–16:45Z**, with an eon-overseer quiet-hold exemption. The command uses
a 14-minute preflight envelope (4 minutes preflight, 10 minutes hard runtime
backstop). **Start at 16:30–16:31Z**: the archived preflight keeps the standing
15:00Z quiet slot closed until 16:30Z, even if a QUIET OVER line arrived
earlier. Starting in this one-minute interval fits its envelope before the
16:45Z grant ends. The gate averaged about 10 seconds of grading per trial;
10 minutes is a capped attempt, not a guarantee of completion. A timeout or
incomplete sample requires a new booked attempt and is never reported as a
success or a writer failure. The four-minute preflight cap is tighter than
the gate procedure's ten-minute allowance; a slow store scan can consume
this attempt without producing any regrade proof. No other Tide agent
may overlap. The booking permits a capped Rune suite at `--jobs 1` to finish;
an unexpected regrade timeout must be checked again before the coordinator
counts it as a mismatch. Any other discrepancy potentially caused by timing
or shared load must also be investigated and repeated under cleared load;
only matching stored grades satisfies acceptance. This exception is the
recorded booking condition,
not permission to add load. The gate worktree stays
clean at R, with its recorded toolchains, vendor caches and driver/std
artifacts present. Current-boot canaries and restricted-writer probe must
match the machinery key. If stale, do not use this narrow window: rebook
and renew them **in the copied records directory** with the original
procedure, not in the frozen run. Canary renewal compiles; renewing the
restricted probe launches a live writer and needs a credential. Neither
renewal is included in this 14-minute attempt. No writer or credential is
needed by the regrade itself when readiness is current.

The exact command, **after booking**, using the unpacked records above. Run
it as a Bash script or in a subshell, so the exported exemption cannot persist
in an interactive shell after the grant:

```bash
set -euo pipefail
: "${E1_GRANT_UNTIL:?coordinator supplies the eon-overseer grant end epoch}"
: "${QUIET_LOG:?coordinator supplies the QUIET OVER inbox}"
E1_BOOKED_END_EPOCH=$E1_GRANT_UNTIL
HEAVY_QUIET_EXEMPT=1
export E1_BOOKED_END_EPOCH QUIET_LOG HEAVY_QUIET_EXEMPT E1_GRANT_UNTIL
T=/home/lucca/.cache/eon-agents/tide-e1-gate
REPORT_ROOT=$(git rev-parse --show-toplevel)
G=/home/lucca/pjs/lab/tide/.worktrees/fx-ic2
E1_COPY="$T/ic2.11-reproduction-records"
E1_REGRADES="$T/ic2.11-regrades"
export REPORT_ROOT G E1_COPY E1_REGRADES
cd "$G"
# The archived run-procedure preflight writes only the disposable copy's scan.
timeout 240 bash "$E1_COPY/evidence/preflight.sh" "$E1_COPY" 14 --scan </dev/null
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  timeout 600 bash -c '
    now=$(date +%s)
    [ "$now" -le "$((E1_BOOKED_END_EPOCH - 600))" ] || exit 75
    awk "{exit !(\$1 < 12)}" /proc/loadavg || exit 75
    exec python3 "$REPORT_ROOT/trials/e1-checked/gate_reproduce.py" \
      regrade "$E1_COPY" "$G" "$E1_REGRADES"
  ' </dev/null
```

This recorded booking requires the explicit eon-overseer quiet-hold
exemption. The command exports `HEAVY_QUIET_EXEMPT=1` and the coordinator
supplies `E1_GRANT_UNTIL`; both preflight and heavy.sh receive them. The
booked-end guard uses that same granted end. These settings apply only to
that authorized grant, not to an arbitrary quiet window. A preflight failure
or late heavy.sh grant stops the attempt. Do not extend the timeout past
the booked end.

The helper verifies clean R and the driver/std hashes, creates a **new**
regrade directory, and copies each selected archived session there. For each
it invokes the frozen harness from G twice:

```bash
python3 trials/e1-checked/harness.py grade COPIED_SESSION --first --compile-only
python3 trials/e1-checked/harness.py grade COPIED_SESSION
```

Those invocations run only inside the booked heavy.sh command. The harness
pins build jobs to 4 and supplies its 1 GiB tmpfs `/tmp` and `TMPDIR=/tmp` to
each jail; the helper also sets TMPDIR for its child. The 10-second test and
480-second aggregate grade limits remain unchanged; each harness invocation
has a 600-second outer guard. No `--local` path is used.

Comparison retains exit status, structure, timeout/infrastructure verdicts,
per-row codes/stages/case numbers, observed exits, exact test logs/listed test
identities and integrity flags. It ignores timestamps, memory/CPU readings,
temporary paths and compiler prose, which are not reproducible bytes.
`matches.json` records each completed first/final comparison; any mismatch or
setup/timeout error exits nonzero and preserves diagnostic logs. Concurrency
and integrity review decisions remain the archived semantic reviews, not new
model judgments. All sampled trials have stored first/final mechanical exit
0. The random sample therefore does not exercise reproduction of a failing structure
verdict or an integrity hold.

**Sampled regrade result: 12/12 match.** The coordinator ran the command
above on 2026-11-15 in the booked E1-exclusive window (16:00–16:45Z,
eon-overseer quiet-until 16:45:00Z, `QUIET_LOG` holding eon-overseer's
`2026-11-15T15:07:49Z eon-overseer: QUIET OVER` line), from this report's
worktree at commit b50fdaa, unsandboxed, in a runner that adds only two
abort checks (quiet-until is the 16:45Z grant; the start falls in
16:30:00–16:31:00Z) and an outer timeout ending 30 s before the grant. Start 16:30:05Z; the archived preflight passed
every check (load 2.89; gate worktree clean at R; Rust and Tide canaries and
the restricted probe current for this boot; store scan exit 0); heavy.sh
granted `build.lock`; end 16:33:33Z, exit 0. All 12 sampled slots (005, 009,
026, 028, 030, 031, 039, 048, 060, 065, 093, 097) reported "first and final
match". `ic2.11-regrades/matches.json` (outside the repository, beside the
records copy) has SHA-256
`3355d86fa9c31ef2eb20d85e8ea5e9826bbbaef3f76459c2efb28bf2e0eeff79`. Python
was the host `python3` (3.12.3). No timeout or load discrepancy occurred.

Review record: the author ran pre-checks of its own before submitting
([Codex rounds 1](../../reviews/fx-ic2.11-author-check-r1-codex.md),
[2](../../reviews/fx-ic2.11-author-check-r2-codex.md),
[3](../../reviews/fx-ic2.11-author-check-r3-codex.md),
[Opus round 4](../../reviews/fx-ic2.11-author-check-r4-opus.md) and
[round 5](../../reviews/fx-ic2.11-author-check-r5-opus.md)). These are not the
GOVERNANCE.md Merge Rule reviews. The independent Codex supervisor pass and
Opus review of the frozen candidate are recorded in this bead's landing
commit and in `reviews/fx-ic2.11-r*`.
