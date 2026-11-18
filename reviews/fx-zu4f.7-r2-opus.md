Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 087e440 (docs)

# fx-zu4f.7 r2: Opus review round (docs, reports only)

## Verdict

**Changes required.** There are 4 blocking findings, and each has a small text fix. None of them
yields an eligible candidate, so the result stays a documented negative. One finding (B3) changes
the counts by one row.

## Scope

- Candidate: HEAD `087e440c0b8b09dcfccd256272ed504c65fd8824` on branch fx-zu4f.7, base main
  `1df19d2`. `git diff 1df19d2..HEAD | sha256sum` =
  `6c99aa597f4da6a176a59b2af2dbabe204a1ba27a7627c877637193b2a6f350d`. Files: research/37 (+2/-1),
  research/38 (+2/-1), research/39 (new, 1,429 lines).
- Contract: the bead body's "Frozen search criteria" and `target/work/protocol.md` (sha256
  `5038d09b…39e4`, rechecked). The protocol bytes appear verbatim in research/39 (checked by
  substring).
- Read: AGENTS.md, GOVERNANCE.md's Merge Rule, the bead and its comments, all of research/39's
  prose and assessment table, its protocol §§0 and 2-5, research/37's and 38's changed context, 38's
  "What a catch would need", design/03 §6.5 (LOCK-1 to LOCK-7) and CAP-2/3, design/reference.md
  §7.4 and §11, and r1-codex.md.
- Verified against git (commit dates are -03:00, converted to UTC): afb17ea 13:27:20Z, 951e6c3
  13:41:44Z, 7863e8f 13:04:01Z, fe99212 13:22:08Z, 42181e3 05:52:57Z, 687c494 13:53:00Z, bd6c51f
  13:48:10Z. `git log -S` puts each protocol hash's first appearance in exactly the commit each
  disclosure names. The bodies of beads .5, .6 and .7 at their filing commits are byte-identical
  to main's. That supports "frozen" in the strict sense: the bodies were not edited after filing.
  Branches fx-zu4f.8 (f04beb8) and fx-zu4f.9 (891bdac) do carry committed protocols, so 39's "From
  fx-zu4f.8 on" sentence is true.
- Sampled 17 rows against the saved issue, comments and timeline JSON: crossbeam#1102, #397, #451,
  #609; rust#114851, #89571, #96706; parking_lot#361; dashmap#351, #304; rayon#10, #343, #738, #919;
  flume#74; plus the waker source for #1102 and the E1/E2 audit rows. I made two read-only GETs
  beyond the saved evidence: `repos/rust-lang/rust/pulls/122565` with its files, and
  `repos/crossbeam-rs/crossbeam/contents/ci?ref=main`. I posted nothing and reran no search.
- Not redone: Codex's transcription, arithmetic, query and digest checks. I spot-checked the
  outcome counts against assess.tsv (74/11/8/27/42) and the 35 symptom rows.

## Blocking findings

**B1. The record never says which E4 narrowings are Tide's permanent design, which are planned v0
gaps, and which are this protocol's own budget. It also never says which narrowings actually
decided rows.** research/39-std-concurrency-search.md:282-306.

A std-level pool is mostly the implementation of locks, channels and parkers. A skeptical reader
will say that a protocol barring Condvar, guards held across effects, and paths over 400 lines was
bound to exclude it. The record's own evidence answers most of that, but it never says so. Its
section is titled "What a catch would need", yet unlike research/38:279-317 it does not say what
Tide would need or plan. "Tide v0 has … no raw lock" (line 290) leaves out that `Mutex` and
`RwLock` are planned in `std::sync::raw` behind the `unsafe` grant (design/03 CAP-3; reference §7.4
and §11). The protocol barred case-study models from taking that grant.

Minimum fix: add one ground-by-ground bullet, in 38's form, to that section:
- `async`, `Waker`: rejected by design (`TD0601`, reference §7).
- The `Pure`, non-nesting `Shared::with` closure: permanent (design/03 LOCK-4).
- Condvar: none in safe Tide by design ("Condition waits are channels", design/03 §6.5).
- Raw locks and weaker orderings: planned in `std::sync::raw` behind the `unsafe` grant, which this
  protocol's models do not take. The design does not say the simulator would explore weak memory.
- Memory-unsafety symptoms: safe Tide cannot produce them, by design.
- External crates: not in v0.
- The 400-line cap: a budget choice of this protocol, not a Tide limit.

Then add what the evidence shows. The 400-line cap decided no row: the two rows that cite it,
crossbeam#1102 and parking_lot#212 (with the rows "as #212"), also fail the guard or the
`parking_lot_core` narrowings. Condvar decided only rayon#343, which is also a livelock (E5(d)).
The guard-across-effects narrowing is what excludes the nearest miss, crossbeam#1102.

**B2. "Over 400 lines" is stated as a fact, but nobody counted it.**
research/39-std-concurrency-search.md:167 (crossbeam#1102), :188 (parking_lot#212, "over 400
lines"), :289 (all E4 paths).

Protocol §5 defines the cap as "the modeled non-test path … comments and blank lines excluded". No
count appears in assess.tsv, report.md or the record. Codex r1 calls it crossbeam#1102's "asserted
line budget" and leans on the guard narrowing instead. For #212 the figure is the length of the
whole `raw_rwlock.rs` file, not of the path.

Minimum fix: give the counted figure, or write "(not counted; the outcome rests on <the other
narrowing>)" at 167 and 188, and drop "and over 400 lines" from the generalization at 289.

**B3. rust-lang/rust#96706 fails the scope filter. It is not an E4 failure.**
research/39-std-concurrency-search.md:135 (and the R1T2 row 8 at :350; counts at :37-42, :50,
:285).

Protocol §3 lists "the panic machinery" among the R1 items not in scope. The thread establishes
that the interleaving is not specific to scoped threads (bjorn3, 2022-05-04: "This also happens for
non-scoped threads"). The record's own E4 reason says "the path is std's panic hook writing to
stderr". The fix, rust#122565 (GET 2026-11-18), changes `library/std/src/panicking.rs` and
otherwise only test expectation files, none under the listed sync or thread directories. Its E1
also rests on the audit subagent's GraphQL POST, the one disclosed request outside the GET-only
rule, and its PR files were never saved (protocol §2).

Minimum fix: record it as "not a candidate (scope)" with that reason, and regenerate the counts:
- not candidates 43 (rust-lang/rust 27);
- screened 119 (rust-lang/rust 11);
- first failure E4 26; and "The 27 E4 failures" at :285 becomes 26.

E1, E2 and E3 stay 74, 11 and 8, and eligible stays 0.

**B4. The self-attestation sentence implies that publication will verify these timestamps. It
won't.** research/37-external-admission-search.md:21, research/38-external-concurrency-search.md:83,
research/39-std-concurrency-search.md:55.

"Until the repository is public, every timestamp here, git or br, is self-attested" reads as if
publication ends self-attestation. Every timestamp in these records predates publication, and
publishing the repository shows only that the commits existed by the publication date. 39's
"External verifiability starts when the repository is published" has the same problem. These
paragraphs exist to tell a public reader how much the pre-registration can be trusted, so this
sentence must be exact.

Minimum fix, in all three files: "Every timestamp here, git or br, is self-attested. Publishing
the repository will show only that these commits existed by the publication date, not the order
before it." In 39, this replaces both sentences.

Text-only fixes like these need no re-check under AP-PROOF-001. B3 changes a count and a
classification, though, and B1 adds a design-status claim. I'd like to re-check B1 and B3.

## Non-blocking findings

- **N1. crossbeam#1102: say that the excluding structure is not the bug.** Lines 301-306. The bug is
  that the array flavor's `disconnect_receivers` did not discard buffered messages. #1121 ports
  rust#108164 into `flavors/array.rs`. Waking threads while holding the waker mutex
  (`waker.rs:265-266`, `:157-165`, verified) is incidental to that cause. One sentence saying so,
  and saying the protocol excludes on the path's structure, not the bug's mechanism, pre-empts the
  "designed to fail" objection more honestly than leaving the reader to find it.
- **N2. Name loom.** Line 307. crossbeam's CI also runs loom (`ci/crossbeam-epoch-loom.sh`, present
  on `main` as of the GET above; not checked at the buggy revisions). The bead's Scope asks to note
  loom where a project uses it. loom is the closest analogue to Tide's simulator, more so than Miri
  or TSan.
- **N3. rayon#10's second reason stretches the narrowing.** Line 278. The narrowing excludes "thread
  exit or process exit ordering". A worker dying by panic is not that. The first reason is sound and
  enough: Tide v0 re-raises a child's panic through its scope (reference §7.1), so a silently dead
  worker cannot keep its structure. Drop the second clause.
- **N4. crossbeam#397: state the alternative reading.** Line 185. The thread says "this is this
  issue: #388", whose fix is crossbeam#391. If that counted for E1, E3 would fail exactly as it does
  for #388 (padding bytes). Saying so closes the obvious objection to the E1 call.
- **N5. "Almost all real, observed, fixed bugs".** Line 285. Every E4 row passed E1-E3 by
  construction. After B3, say "all … (rayon#938's E1 reading is marked doubtful)".
- **N6. Style.** The three disclosure paragraphs are single unwrapped lines in otherwise wrapped
  files. The pointer mixes curly outer and straight inner quotes. In research/37:15 the pointer
  lands between "examined" and the colon that introduces the file path; it reads better after the
  sentence. research/37's disclosure does not say that the bead body required the protocol to be
  committed, unlike 38 and 39. Adding it would keep the three parallel.
- **N7. Sampled rows that hold.** These held on the saved threads, reasons included:
  rust#114851, rust#89571, crossbeam#451, crossbeam#609, parking_lot#361, dashmap#351, dashmap#304,
  rayon#738 (doubtful, as marked), rayon#919, rayon#343 (the promoted E2 comment predates the
  2017-05-30 merge) and flume#74.

## Reasoning

**Honesty and framing.** Lines 18-26 do what they must. There is no rate and no population claim,
the record says the simulator was never asked, and it says plainly that the excluded bugs are real
bugs. Lines 313-316 repeat it, and the record names its own process breach, the GraphQL POST at
:102. What is missing is the answer to the strongest objection: that the narrowings guarantee
failure on this pool. The evidence answers it well. The cap and Condvar barely mattered, and the
deciding narrowings follow from permanent design, not from choices made for the search. But the
record has to say that, and say which limits will lift (B1). The unsupported line counts (B2) are
small, but they are the kind of detail a skeptic checks first.

**Pre-registration disclosures.** Every time and commit they cite matches git. The order they
describe holds for all three records: criteria committed before the search in .6 and .7, and only
after it in .5, which 37 states. Each says what was not committed and when it first was. The
"correction" label appears only in 37 and 38, which had already landed, which is right. They are
proportionate and plain, and every use of "pre-registered" in the three files carries the pointer.
The one inexact sentence is B4. The placements fit: next to the registration claim in 37's
introduction, and after "Registration" in 38's Method.

**Eligibility.** Of the 17 rows sampled, 1 has a wrong first failure (B3), 3 have reasons that need
tightening (B2, N3, N4), and the rest hold. I found no wrongly excluded candidate: every corrected
row still fails a later criterion or the scope filter.

**Usefulness.** The record is consistent with design/03 and reference on what v0 offers. It falls
short of research/38's standard on what Tide would need (B1).

DONE
