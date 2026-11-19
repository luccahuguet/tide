Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 0b12748 (research record)

## Verdict

**Changes required.** One blocking finding. Two E3 rows were failed without reading the report that their own PR says it fixes. That is the same defect class as Codex r1 B3 (B09.162). The record's statement that r1 fetched missing reports for "every affected row" holds only for the 183-row r1 set, not for the other 122 rows that fail at E2 or later. The fix is small: fetch two issues, read them, record the outcome, and state the link-gap scope. Neither row looks concurrency-shaped, so I expect both arms to stay NOT CAUGHT. Apart from this, the record is accurate, takes no credit it hasn't earned, and is safe to publish.

Confidence: high on the finding and on the counts. The semantic judgments are sampled, so my confidence there is limited to the rows named below.

## Scope

This was a read-only review of HEAD `0b12748d0d26c46b843c3d926668441402d43c49`: research/44-fx-zu4f.10-concurrency-round.md and research/protocols/fx-zu4f.10-record/arm-b-ledger.tsv. I checked them against the registered protocol (63eb32e), the bead and its comments, the Codex r1 report and its three re-checks, r1-fixes.md, report.md, the WORK helpers, and the private decisions, inventories, evidence bodies and frozen clones. The commit message was checked too.

I ran only plain Python and git reads: `GIT_NO_LAZY_FETCH=1` on the frozen clones, plus a scratchpad copy of `link-gap-r1.py` widened to the rows r1 did not cover. There were no network requests, builds, edits to candidate files, `.beads/` writes or subagents. Nothing private is quoted below beyond row IDs and public upstream issue numbers.

Ledger SHA-256 is `8991e42f…b688`, as the record says (record:208). Recount: 5,076 rows, of which E1 path 3,779, comment 76 and manual 914; E2 109, E3 187, E4 10, unresolved 1. E1 passes are 306 (36 duplicate rows), E2 passes 197 (equal to the non-empty `e2_quote` count), E3 passes 10, and every duplicate target resolves. These all match record:93-124 and the commit message. I did not repeat Codex's per-crate recount.

## Blocking findings

### B1. Two E3 failures were settled without the linked report their PR closes, and the record overstates r1's link-gap coverage

- **Where:** ledger rows B02.277 and B09.092; record:365-370 ("In r1 every affected row was re-read in full …, missing reports were fetched") and deviation 8 (record:719-732). Protocol §3, lines 272-280.
- **Evidence.**
  - Protocol §3 requires reading the "linked issue(s) that report or explain this bug" and inventorying them before any failure at E2 or later.
  - r1's `link-gap-r1.py` ran only over the 183 r1 rows: the sym.py set, every row with an r1 decision line at E2 or later, and B15.047.
  - I ran the same extraction, unchanged, over the 122 rows that fail at E2 or later and that r1 did not touch. It reports 27 absent references.
  - Two of them fall in the class r1's fixers declared they fetch (a closing keyword in the row's own PR body):
    - **B02.277** (rustls, E3): PR1993's body ends "fixes #1992". The inventory holds only commit/pulls, commit comments, pulls/1993, its comments and its reviews. There is no issues/1992.
    - **B09.092** (bindgen, E3): PR2499's body ends "Fixes #2498". The inventory holds pulls/2499 and its threads, with no issues/2498.
  - So these two screens are incomplete under §3, exactly as B09.162 was before its B3 fix. Codex treated that case as blocking, and the same standard applies here.
  - The other 25 absent references fall in classes r1 declared it would not fetch:
    - prior-work and release-list URLs: B02.108, B02.335 (whose bug-site row B02.027 holds PR1668), B02.342, B06.340/447, B23.066, B23.140 and B23.148;
    - "see" links to other repositories or features: B02.303, B06.114/152, B06.477 and B23.142;
    - the feature request that the regressing PR closes: B23.155 #211;
    - a discussion fetched by GraphQL rather than as an issue: B23.129 #794.
  - Those 25 need no fetch under the declared rule, but the record should say that the rule was applied to these rows as well.
- **Risk:** low that an eligible row is hidden. B02.277 is the FIPS group configuration; B09.092 is the variadic wrapper skip in code generation; both are single-threaded from the diff. The defect is that the record claims a complete §3 screen for rows where one wasn't done.
- **Minimum fix:**
  1. Fetch rustls/rustls issue 1992 and rust-lang/rust-bindgen issue 2498 (body and all comments) through `inventory-b.py` / `b-evidence.py`'s throttle. Read them in full, then confirm or rejudge B02.277 and B09.092 with superseding r1 lines. If either is unavailable, mark that row unresolved.
  2. Run `link-gap-r1.py` over every row that fails at E2 or later, not only the r1 set, and record that scope and its result (25 references in not-fetched classes) in deviation 8.
  3. Replace "every affected row … missing reports were fetched" at record:365-370 with the true scope.
  4. Regenerate the ledger and update its checksum, and the counts if anything moves.

  I re-check only this fix.

## Non-blocking findings

1. **The E1 viewers are not disclosed.**
   - **What happened:** the record lists the abridged views that settled rows failing at E2 or later (deviation 8) but not those used for E1. `view-b.py` is the "compact E1 view" (report.md:29). It cuts each commit message and each PR body at 15 lines, or 6 with `--brief`. It is missing from the tool table at record:216-245, and so is `read-b.py`. Builder 5 triaged with a one-line scratch viewer, `terse.py`, kept outside WORK and not retained; report.md:44 lists it, and report.md:56 lists it again among the viewers builders 7-9 had. In builder 5's window (20:32-20:45Z), 166 E1 rows were decided. record:359-362 says E1 was decided from "the matched commit's message, every associated PR's title and body"; for rows triaged on one line, that wording claims more than was done.
   - **What I checked:**
     - Across all 914 hand-judged E1 rows, the parts `view-b.py` omits (195 messages and 93 PR bodies longer than 15 lines) contain no strong concurrency term.
     - Of builder 5's 166 rows, only four combine concurrency words with bug words anywhere: B03.069, B06.049, B06.271 and B06.397. All four are correct E1 failures.
     - A whole-text scan of E1 rows for deadlock, hang, stuck, poison and race terms in PR bodies found four more: B02.169, B10.106, B16.077 and B20.069. All are correct E1 failures (incidental words, or a mutex change in a different commit of the PR).
     
     So no hidden candidate.
   - **Fix (text only):** add `view-b.py` (with its caps), `read-b.py` and `terse.py` to the tool table, or to deviation 3, in one sentence each.
2. **Builder 2's decisions are attributed to Opus.** record:38-39 says hand judgments were "made by eight successive Opus builders". Builder 2 (Codex) recorded 11 of them (B01.001 through B01.033, between 19:48:58Z and 19:53:25Z), four of them at E2 or later. Suggested wording: "by builder 2 (Codex; 11 serde rows) and eight Opus builders".
3. **The edit window is stale.** record:173 says "record edits until about 03:50Z". Fixers 6-11 edited until 0b12748, committed at 2026-11-19T06:23:33Z.
4. **The ledger `cap` column is inconsistent.** Only B23.129 carries "not cap-only". B16.018 and B16.128, which record:499-501 names as not capability-only, are blank. Either fill those two cells, or describe the column (record:537) as "cap:proc on capability-only rows; B23.129 is marked not cap-only".
5. **One reason hides the deadlock wording.** B06.397's reason reads "release, no bug", yet its message says the deprecated `TraceLogger` "contains some deadlocks". E1 still fails, because the commit deprecates the API and fixes nothing. A wording such as "deprecates an API with stated deadlocks; fixes none" would show an outsider that the statement was read.
6. **Some terms are undefined for a cold reader.** "RC" (record:637), "desk decision" / tide-desk (record:421, 489, 801) and tide-lead are used without definition. One sentence near the terms paragraph would cover them, for example: "tide-lead, the coordinating agent session; tide-desk, the owner's contact session; RC, Tide's release candidate".
7. **Five E3 rows have `e2_source` without a URL.** B09.130, B09.151, B09.227, B09.235 and B09.253 say "commit message matched commit subject/message". The commit URL is in `sources`, so this is recoverable; a short pointer would make each row self-contained.

## Reasoning

### Judgment sample: rows no Codex pass named

- **E1:** B06.397, B06.049, B06.271, B03.069, B13.033, B13.024, B14.216, B10.106, B16.077, B20.069 and B02.169. Each failure follows the registered E1:
  - a deprecation or release fixes nothing;
  - a test-only race fix fails (B14.216's "race condition" is in the env detector test);
  - work in progress with a stated but unfixed concern fails (B13.024);
  - a refactor or feature fails.
- **E2:** B10.036 ("may require"), B10.134 ("could previously get stuck") and B16.044 (a DoS weakness stated as a capability, with no occurrence). Each follows the inherited "might/could" narrowing. Each would also fail E4: handshake crypto, the async high-level API, or network.
- **E3:**
  - B14.043: the drop policy is a designed capacity limit.
  - B02.027: sequential reconnects reuse one ticket.
  - B10.111: overflow under network reordering in one state machine. I read the whole thread; the poisoned-mutex panic is quinn's knock-on effect.
  - B08.011: initialization order. The message names the separate log-API race and says this change does not address it.
  - B09.187.
  - B02.277 and B09.092 are judged correctly on the diff but have incomplete evidence (B1).

  None hides an eligible candidate.
- **E4 and capability-only rows:**
  - B09.003 stays an E4 failure (subprocess), tagged cap:proc.
  - Arm A rank 9 (bindgen) stays unportable (subprocess), tagged cap:proc.
  - Both are listed for fx-zu4f.13. Neither is relaxed or counted.
  - B23.129's sources carry the GraphQL discussion read with its digest.
- **Arm A:** spot-checked darling_core's pinned source. The only uses are `impl FromMeta for AtomicBool` (construction) and one `thread::panicking()` query, so "no concurrent core" holds.

### Honesty and claims

- The record leads with what the round does not show (record:23-59).
- It gives verdicts per arm, with B23.128 kept unresolved and out of every count.
- It gives no discovery rate, claims nothing for Tide, and judges no project or author.
- "Measured changed-item union", "enumerated lower bound" and "partial" are defined where the numbers appear (record:439-449, 572-595), and each row's label fits that definition.
- The budget is never used to exclude a row.

The overstatement found is B1's scope sentence, plus the E1 method wording in non-blocking item 1.

### Privacy

I searched the ledger's reason, passed, quote and note fields for unfixed, open, security, attacker, DoS, advisory and CVE language. Every hit is a public, fixed upstream item:

- B02.213: the public advisory issue, fixed by the matched commit;
- B02.316: a fixed DoS that credits its reporter;
- B16.044: public issue 28, fixed;
- B09.187: a public review remark that a panic was not fully fixed, already public in the PR thread.

Arm A records no find. No sentence judges a maintainer. I see no sentence a maintainer could reasonably object to.

### Consistency

- Record, ledger checksum, counts, commit message and bead comments agree, including the 03:48 and 06:12 counts, unchanged since 707ab78.
- 27 linked reports were fetched in r1: 24 by `link-gap-r1.py`, plus B09.199, B11.082 and B09.162. This matches record:47 and deviation 8.

DONE
