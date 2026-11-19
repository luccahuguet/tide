Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 3f3d42c (re-check of O1 and items 1-7)

# fx-zu4f.11 record: re-check of the Opus findings

- **Candidate:** HEAD `3f3d42c430a125a3a094552fd05fca6f83f24e78`, which amends `cfc5c26`.
- **Diff since cfc5c26:** `research/46-fx-zu4f.11-agent-marker-round.md` and
  `research/protocols/fx-zu4f.11-record/b-candidates.tsv` only.
- **Fixer's dispositions:** `target/work/opus-fixes.md`.
- **Rules:** I worked read-only, with no network and no builds.

## O1 (blocking): resolved

- **Script check: every row now lists every thread in its inventory.** For all 42 TSV rows I
  compared the GitHub PR/issue links in `sources` with the thread list in that row's private
  evidence inventory (`evidence.json`) and with its associated PR numbers.
  - Inventoried threads missing from `sources`: **0 rows**.
  - Associated PRs missing from `sources`: 0 rows.
  - Commits with commit comments: 0 of 42.
  - Three rows list a thread that their own inventory lacks. Each of those threads was
    fetched under a sibling row, and that row's read record names it:
    - 23bfd3a lists #1437, fetched under b6d4e7b and b5ca63c;
    - 6bd3fc9 lists #1437 and #1440, fetched under b5ca63c;
    - 5bbb9f6 lists #778, fetched under 859f691.

    These extras are correct, not a gap.
  - The claim at line 249 that every inventoried source was read now holds for all 42 rows.
- **The added threads support the unchanged judgments.**
  - I had already read #62, #54, #982 and #781 in the first round. #62 and #54 name no race,
    hang or shared-state blocking. #982 states the deterministic compaction-frontier cause
    behind 476247f's E3 failure. #781 is a feature PR, so 553d9bc stays E1.
  - The TSV word diff shows that the only changes are added links, plus the df7e31e reason
    (item 1). No first failure, duplicate link, quote or inventory changed.
- **The record cites the new TSV hash.** Line 252 cites `20b19ebf…c45e`, and that matches
  `sha256sum`.
- **The fix is disclosed.** A new row in the methods and deviations table discloses it. It
  says, correctly, that the first screening record had wrongly noted no associated PR for
  2d3f2f8, 2e1479b and e2d89d9. Those lookups return #62, #62 and #802.
- **The private records agree with the record.**
  - `B-screen.jsonl` (SHA-256 `f12cc693…06ed`, which matches `B_sha256` in `resume-state.json`)
    gives first failures of E1 16, E2 4, E3 16 and E4 6 over 42 rows. Counting only rows that
    are not duplicates gives 16/2/16/5. Both sets match the record.
  - The private db8db55 row now says E1.
  - A pre-fix copy is kept as `B-screen.pre-opus-fixes.jsonl`.
- **Nit (does not block).** The deviation row says the rows "gained the PRs/issues in their
  evidence inventories". For 5bbb9f6, #778 came from 859f691's inventory instead. This is
  text only and needs no re-check.

## Non-blocking items

1. **Resolved.**
   - The df7e31e reason now says the server had already sent its end signal and the tunnel
     did not act on it, unlike fbcbbb8, where the other party itself hangs.
   - The TSV and the record table agree.
   - The wording does not overstate.
2. **Resolved.**
   - The text now reads "two are tagged capability-only, provisionally".
   - The 12a36c9 note tells fx-zu4f.13 to check that the landed network capability covers UDP
     DNS, not only HTTP (`Net::get`).
   - The section now says in one clause what fx-zu4f.13 is.
3. **Resolved.**
   - Arm B's opening paragraph now says that db8db55 matched only `racing` and that it fails
     E1.
   - This agrees with my check of its commit message against the registered regex.
4. **Resolved.**
   - A new paragraph says what each arm does and what "caught" means. It matches protocol
     §5-§7: byte-identical replays, a fixed model or a fault-removed control passing all 1,000
     seeds, and real reproduction for arm A.
   - The definitions of marker, marker share and T match protocol §2's expression, including
     the five names allowed in `Generated with/by` lines and the `aider:` subject prefix.
     They carry the `Amp` substring caveat and the "not agent-written" reading.
   - The pool table now has a marker-share column. I checked each figure against
     `activity-markers.tsv`:
     - 25/114 (22%), 203/318 (64%), 49/77 (64%), 2,575/3,982 (65%);
     - 245/270 (91%), 27/116 (23%), 80/108 (74%).
5. **Resolved.** The sentence now reads: "The method supplies each property from upstream text;
   this round froze none and sampled no seed."
6. **Resolved.** The sentence now adds "(#801's E3 pass is tentative)", and that matches the TSV.
7. **Resolved.** The sentence now reads "no finding is withheld".

## Privacy

- The committed changes add only public GitHub links, definitions and wording.
- No committed text quotes a private narrative. The deviation row describes the private
  record's error without quoting it.
- Nothing added describes an undisclosed or unfixed bug.

## Verdict

**Approve.** O1 and items 1-7 are resolved, and the fixes add no new problem. The one nit is
text only and does not block.

DONE
