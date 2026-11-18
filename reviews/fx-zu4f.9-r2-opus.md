Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 110afe3 (research record)

# fx-zu4f.9 r2: Opus review round (research record, reports only)

## Verdict

**Changes required.** There are 2 blocking findings, and each has a text-level fix. Neither yields
an eligible row, so the result stays a documented negative with 0 eligible. Finding B1 changes the
first-failure counts for up to 9 rows. Finding B2 corrects what the record says about Tide's design.

## Scope

- Candidate: HEAD `110afe3813d0e624537227367df25574e9e9a402` on branch fx-zu4f.9, on top of
  registration commit `891bdacc3d46a908ee05be1fc996cb28988e9008`, base main `1e673c5`.
  `git diff 1e673c5..HEAD | sha256sum` =
  `9d8c3934874578bf10736c8ffcedb4845ab58cd3f79923fd117a1f1b9378e143` (the same diff Codex
  re-checked). Files: research/41 (new, 1,253 lines), the protocol and the two pool CSVs.
- Contract: the bead body's "Frozen search criteria" and the committed protocol. I re-hashed all
  three files and they match the bead comment of 14:20 UTC and the record's table.
- Read: AGENTS.md, GOVERNANCE.md's Merge Rule, `br --no-db show` and `comments` for fx-zu4f.9,
  the whole protocol, research/41's prose, tables and every assessment block I cite below,
  r1-codex.md, r1-recheck-codex.md, report.md's completion section, main's research/38 and 39,
  reviews/fx-zu4f.7-r2-opus.md, design/reference.md §6.4-6.5, §7.4 and §11, and design/03 §6.5
  and the lines on `select`, `OnceLock` and `std::sync::raw`.
- Registration, checked against git and the saved logs: 1e673c5 was committed at 14:13:02Z and
  891bdac at 14:19:58Z. The bead comment recording the hashes is at 14:20 UTC, PROTOCOL-READY at
  14:20:23Z, and the first per-bug GET in fetch-log.txt (servo commit 6d3429ad) at 14:21:39Z.
  The order the record claims holds.
- Counts: I recomputed these by script from the row table, and checked each row against its
  assessment's "First failure" line and the CSVs' column C. All 100 rows agree: E1 2, E2 19,
  E3 3, E4 76, and every tab/project cell matches.
- **Rows sampled that neither Codex pass sampled (23):** B10, B11, B14, B17, B22, B26, B29, B30,
  B41, B48/B59 (the duplicate), B49, N02, N03, N07, N08, N09, N10, N17, N20, N24, N29, N32, N38.
  These cover both tabs and servo, tikv, rust-threadpool, parity-ethereum, crossbeam and redox,
  with E2 and E4 rows. I checked them against the saved commit, PR, issue, review and timeline
  JSON under target/work/rows, and for B14, B49, B10 and N38 against the saved source under
  target/work/src. I made no upstream GET. The E1 rows (B25, N34) and E3 rows (B21, N06, N36)
  had all been sampled by Codex, so I didn't resample them for their own calls. Finding B1 does
  revisit N36 and eight other Codex-sampled rows, because it concerns the rule they share.
- Not redone: Codex's byte-matching of sources, its N34 GitLab recovery and its structural check
  of reaudit.tsv.

## Blocking findings

**B1. Since the re-audit, E2 passes on failures that belong to another defect or to nothing
identified. That widens the frozen E2.**
research/41-qin-dataset-search.md:211-215 (the rule), and the assessments of N05 (:925), N13
(:997), N14 (:1006), N19 (:1051), N25 (:1105), N28 (:1132), N30 (:1150), N36 (:1204) and N40
(:1237). Counts at :90-92 and :104-118, and the bullets at :131, :158 and :166.

The bead's frozen E2 says "the symptom is stated … Quote it". The bead's property rule then
builds the property from "the report's own statement of the expected behavior". So the symptom
has to be the row's own bug's symptom. The protocol cannot widen this (protocol :6-9: "never
widens … the bead wins"). Its sentence "Every statement of an observed failure in the evidence
counts" (protocol :212-213) sits inside the hedging narrowing. It means an observed occurrence of
*that* wrong behavior anywhere in the evidence overrides hedged wording elsewhere. It doesn't
mean any failure in the thread will do.

Codex r1's B1 was right that a stated failure whose *diagnosis* is hedged still passes (B32, N04,
N37: the evidence links the failure to the bug, even if tentatively). The re-audit stretched that
into "no observed failure is silently erased because it concerns another defect in the wider PR"
(report.md, B3 line). It moved six rows from E2 to E4 or E3 on that basis (report.md's change
table: N05, N13, N14, N25, N36, N40). By the record's own words, the quoted failures are not
the row's bug:

- **N05:** "an executed transmute experiment during the PR, not an observation of the first-parent
  race". The PR author adds, of the race, "we've never had any problems that I'm aware of".
- **N14:** regressions that "the author traces … to changing copy-on-write into in-place
  mutation", the PR's own change. Linked #13177 states only that `RefCell` is not `Sync`.
- **N25:** the executed output uses "the `Scope::default()` constructor introduced and removed
  during the PR, absent at the frozen first parent".
- **N36:** wrong bytes "in the newly wired DMA driver … it remains unfixed … not evidence of that
  output at the first parent".
- **N40:** "stated failures in the wider PR, not observations of the duplicate-session race". The
  E3 line adds "This is separate from the PR's signature-calculation bugs". The commit itself
  says only "fixed possible race".
- **N13, N19, N28, N30:** bare CI results ("Test failed", "CI failed", "Build failed", "Timed out")
  that, as each block says, nothing attributes to the race. N19's "CI failed" is on the fix PR
  itself (tikv#2266; I checked the saved comments). N30's is on another repository's PR
  (rust#24526).

Codex's re-check approved these, but on the same reading. The outcome doesn't change: every one
of these rows also fails E4 or E3. The counts do change, and they are a public claim. The record
currently blames 76 rows on E4, "not portable to Tide v0", when up to 8 of those rows have no
stated symptom of their own bug. The fix also keeps 41 on the standard of 38 and 39, where E2
needs the row's own symptom.

*Minimum fix.* Re-judge E2 for these nine rows by one rule, stated once at :211:

> a failure passes E2 when the evidence ties it to the row's bug, even tentatively (B32, N04,
> N37); a failure the evidence attributes to another defect, a PR-only revision or new code, or
> that nothing in the evidence links to the bug, does not.

Keep each quote in its block as disclosed context, so nothing is erased. Move the current E3/E4
reason to "additional". If all nine fail, the counts become:

- global: E2 28, E3 2, E4 68;
- non-blocking tab: servo E2 7 / E3 1 / E4 10; tikv 2/0/1; crossbeam 5/0/3; rand 2/0/0;
  tock 1/0/1; parity-ethereum 2/0/2.

Update the bullets at :131, :158 and :166 to match, and rewrite :211-215 as the rule above. The
row-table reasons for the nine rows change. Then delete report.md's "N05/N13/N14/N36/N40 also
retain" justification from any text that lands. The fix changes counts and calls, so it needs my
re-check.

**B2. Four deliberate design omissions are presented as "missing primitives".**
research/41-qin-dataset-search.md:145-146 ("**No v0 primitive.** `Condvar` waits, `Once` and lazy
initialization … have none") and :183 (the row "Missing primitives plus a reconstruction ban":
"no `Condvar`, `select`, sync once-cell or needed Arc count/try-unwrapping operations").

The brief asks whether the map from narrowings to design, planned features and protocol budget is
accurate against design/03 and reference.md. For these four it is not. Each one is a decided
omission with a named replacement, not a gap that v0 has yet to fill:

| Omission | What the design says |
| --- | --- |
| `Condvar` | "Condition waits are channels" (design/03 §6.5, :1699). research/39 says, after its own Opus round: "Safe Tide has no condition variable by design". |
| `select` | "send one enum through one channel; there is no `select`" (design/03 :377; reference :698, `TD0605`). |
| `OnceLock`, `LazyLock`, `lazy_static!` | not admitted (`TD0520`, `TD0502`); replaced by a `const`/`static` or a value built in `main` (design/03 :481-482, :526, GLOBAL-2). |
| `Arc` counts, `try_unwrap`, `get_mut` | "`Arc` has no `strong_count`, `weak_count`, `get_mut`, `try_unwrap`, or `into_inner`, because their results depend on other tasks" (reference :447-449). |

A Rust reader would take "missing primitives" to mean these are coming. That is false, and it
is the B1 distinction of fx-zu4f.7 r2: design, planned, or protocol budget.

*Minimum fix.* Rename the table row to something like "Omitted by design, plus a reconstruction
ban", and cite the four sources above in it. Re-word :145-146 the same way. The kernel paths of
Redox and Tock belong under "Platform mechanisms" (where :184 already has them), not here. This
changes what the record claims about the design, so it needs my re-check, a short one.

## Non-blocking findings

1. **The "protocol-only" label for external crates, network and subprocesses (:182).** These three
   bans come from the bead's frozen E4 ("no network, no subprocess and no external crates"), not
   from the protocol. Tide itself has `[dependencies]` (reference :1035) and a `Net` capability
   (reference :338). So "experiment rule, not a Tide limit" is right in substance, but the
   attribution should say "frozen experiment rule (bead E4)". Only the "no similar Tide substitute"
   clause is the protocol's (from fx-zu4f.6). research/39 says "External crates are not in v0", so
   the two records disagree. That is for fx-qlek.7's launch-claims audit, not this change.
2. **Registration chronology.** :22-31 says the hashes were recorded "before any bug was examined"
   but gives no time for the first screening request. research/39 states its first search time.
   Add one clause: "the first per-bug request (fetch-log.txt) was at 14:21:39Z, after tide-lead's
   bead comment at 14:20Z". The self-attestation sentence (:29-31) matches research/37, 38 and 39
   word for word. There is correctly no OpenTimestamps claim.
3. **Authorship line (:5).** "Initial builder: Claude Opus 5.5; r1 corrections: Codex" leaves out
   that the Opus builder made the first r1 corrections (B4, B6, N1 in 8960f2a). Suggested: "r1
   corrections: the initial builder, then three successive Codex (gpt-6.1-sol) authors".
   **For tide-lead:** the launch scripts in target/work show one Opus builder
   (pane-…-builder.sh) and three Codex builders (builder2-4). The bead comment of 15:56 and this
   round's brief say "four Codex authors". The record rightly gives no number. Don't carry
   "four" into public text.
4. **B26 (:593).** The timeline's cross-reference #11172 is not "a later unrelated PR". It is a
   2019 PR by BurtonQin, the dataset's first author, which calls its own bug "similar to #592" and
   states no #592 symptom. The E2 call stands. The wording should say this, since it also shows
   how the authors met the bug.
5. **B49 heading (:797)** names `parity-ethereum#3686`, but column C (and the row table) say
   `#3702`. Use "(parity-ethereum#3702; issue #3686)".
6. **B59 (:888)** cites `util/network/host.rs:447`; the shared block (:793) uses
   `util/src/network/host.rs`.
7. **N07-N09 (stdout interleaving).** Tide v0 does have `println!` and `stdout()` as task-owned
   sinks, recorded in program order (reference §6.5, :453-466, :540). "OS mechanism" alone invites
   a reader to ask why that doesn't map. Add half a sentence: §5's mapping table has no sink
   mapping ("anything else: unportable"), Tide records whole records and not a process byte
   stream, and the symptom is observed by an external harness parsing that stream. The outcome
   stands either way.
8. **Series links.** The opening (:3) calls this the final round, consistent with fx-qlek.7's
   framing ("fx-zu4f.9 (Qin et al., the final round)") and :80-83. It would help a reader to link
   research/38 and 39 as the earlier rounds there.
9. **Rebase note for tide-lead.** main now has research/39, 40 and 42. Number 41 is still free,
   but the candidate needs the usual rebase before landing.

## Reasoning

**1. Honesty of the negative.**
- The headline is honest: 0 eligible, no model, no seed run. It says plainly "The dataset's authors
  chose these bugs, not Tide" (:18-19), with the Qin et al. citation and the pinned artifact
  (:11-17).
- "What this round does not show" disclaims a rate and says Tide found nothing unaided.
- "Safe `Shared`'s prevention claim does not extend to planned raw interfaces" (:192-193) is
  correctly bounded, and the 400-line cap and E5 are correctly said to decide nothing.
- Two things are off. The E2 widening (B1) inflates the E4 count that the "Why the rows fail"
  summary leads with. The design-status table mislabels four design decisions (B2). Both errors
  make Tide look more limited, not better, so they don't overclaim in Tide's favor. But they are
  still inaccurate public statements.

**2. Registration and disclosure.**
- Accurate and checkable (see Scope). The diff-check deviation is disclosed exactly: "every data
  row and most CSV text (166 flagged lines…)".
- The delegation and re-audit disclosure (:203-210) is candid. It says the subagent drafted
  B07-N41, Codex found wrong calls, and every one of the 94 rows was re-audited without a subagent.

**3. The calls.**
- Of my 23 unsampled rows, all first failures stand on the frozen criteria.
- B11, B17, B26, B30, B41, N10, N20, N24, N29 and N32 fail E2 correctly. Each names only the change,
  only a possible failure, or in B17's case no failure at all.
- B14 (a std `RwLock` read guard in a match scrutinee into `wl()`, plus gRPC), B22 and B49
  (Condvar at queue/mod.rs:309, confirmed in saved source), and B29 (parking_lot,
  writer-preference) are correct E4 calls. So are B48/B59, N02, N03, N17 and N38
  (`Arc::try_unwrap` at host.rs:639, confirmed).
- B10 is a correct E4 call (glutin `wait_events`), though its E3 is rightly marked weak.
- The duplicate B48/B59 is handled as protocol §1 requires: assessed once, sites assigned and the
  assignment disclosed as inferred.
- The one wrong-in-a-countable-way pattern is B1's, and it sits in rows Codex sampled and
  approved.

**4. Consistency with research/38 and 39.**
- Same E1-E4 standards, the same "documented negative" and "not a rate" language, the same
  self-attestation sentence, and E5 added as the bead requires.
- 41 diverges from 39 in two places: E2 (B1) and the Condvar design status (B2).
- 41's "final round" framing matches fx-qlek.7's.

**5. Readability.**
- A Rust concurrency reader can follow "Why the rows fail" in a few minutes. Its groups (nested
  locks, external crates, unsafe sharing, old runtime, network) are the right ones, and B01, B05
  and N18 are good named near-misses.
- After B1 and B2, the summary will also be accurate about which exclusions are Tide's design and
  which are this experiment's rules.

DONE
