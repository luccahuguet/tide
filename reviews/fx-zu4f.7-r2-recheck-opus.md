Model: Claude Opus 5.5 · Date: 2026-11-18 · Re-check 7e51b39

# fx-zu4f.7 r2 re-check (B1, B3)

Candidate: HEAD `7e51b39c49eaee33a08c37d15774d33694eb3480`, which amends 087e440, on base `1df19d2`.
I read the round diff `git diff 087e440 HEAD` (research/37, 38 and 39) and checked it against
design/03 §6.5 (LOCK-4), CAP-3, design/reference.md §7, §7.1, §7.4, §11 and the "Not in v0" list
(line 1078), protocol §3, and `target/work/assess.tsv`.

## Verdict

**Approve.** B1 and B3 are fixed. Neither fix introduced a defect in the text it touches.

## B1: fixed

The new bullet at research/39 "Which limits are Tide's design, which are planned, and which decided
rows" checks out against the design text:

- `async` and `Waker` are rejected by design: `TD0601`, reference §7 (line 573).
- The `Pure`, non-nesting `Shared::with` closure is in design/03 LOCK-4.
- "Condition waits are channels" is in design/03 §6.5.
- Raw locks and weaker orderings are planned in `std::sync::raw` behind the `unsafe` grant:
  design/03 CAP-3, reference §7.4 ("Both are planned") and §11 (the "(planned)" row).
- "External crates are not in v0" matches reference line 1078 ("Not in v0: … imported Rust
  crates").
- "The design does not say the simulator would explore weak memory" matches research/38's wording
  and makes no new claim.

The account of which narrowings decided rows matches the record and assess.tsv:

- The cap was never counted. Only crossbeam#1102 and parking_lot#212 mentioned length, and the rows
  "as #212" (#465, #489, #505, #257, #294, #323) fail on `raw_rwlock` and `parking_lot_core`.
- rayon#343 is the only E4 row that turns on Condvar.
- The guard narrowing is what decides crossbeam#1102.

The 400-line cap is now labelled a budget choice of the protocol, not a Tide limit.

## B3: fixed

rust#96706 is now "not a candidate (scope)" in both its assessment row and its R1T2 row 8. The
reason cites the panic machinery, the "also happens for non-scoped threads" comment, and the fix's
files. That is protocol §3's exclusion. The Method paragraph now says that no counted row rests on
the GraphQL POST.

The counts are consistent everywhere they appear:

| Count | Value |
| --- | ---: |
| Not candidates (scope) | 43 |
| of which rust-lang/rust | 27 |
| Screened | 119 |
| of which rust-lang/rust | 11 |
| E1 | 74 |
| E2 | 11 |
| E3 | 8 |
| E4 | 26 |
| E5 | 0 |
| Eligible | 0 |

The same figures appear in the summary line, the counts table, the per-repository line and "26 E4
failures". assess.tsv now tallies 74/11/8/26/43, and 162 = 43 + 119. The three remaining mentions
of rust#96706 agree with one another.

## Other items (no re-check required), noticed in passing

- B2, B4, N1-N5 and N6's 37 parity: applied as proposed. In particular:
  - the "not counted" qualifiers at crossbeam#1102 and parking_lot#212;
  - the publication sentence, corrected in all three files;
  - research/37 now states the bead's commit requirement;
  - the pointer in research/37 now follows the registration sentence.

  None of these is plainly wrong.
- Nit, non-blocking: research/39's re-audit paragraph still says "all 120 screened threads". That
  was the right number at r1, but the record now screens 119, so "(then 120)" would avoid a
  mismatch.

DONE
