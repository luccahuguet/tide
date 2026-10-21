# fx-gcs.5 review, round 1 (Opus)

Reviewer: Opus, round 1, report-only converge round (GOVERNANCE.md; run in
parallel with the Codex pass on the same frozen candidate).
Candidate: worktree `tide-fx-gcs.5`, branch `fx-gcs.5`, `git diff e6e8103`
with new files intent-to-added; fingerprint
`7d0b819efde352e188991ba139c1ff08a7f8e9a634853aec6a89b7a3d4f71db6`.
Date: 2026-10-21.

## Verdict

Changes needed: two small blocking findings, both in the corpus. The design
edits (PAR-5/6/7, the FX0610 resolution, the `= SEND-3` and `= PAR-n`
conversions) are sound.

## Blocking

**B1. PAR-4's Send/Sync failures now carry FX0612, against the marking
convention the rest of the corpus uses.** The README's "a line both report
on carries both" depends on what the checker reports, so the design text
decides. PAR-5 ("not `Pure` or not `Send` is `FX0610`") and PAR-6 ("misses
any of the four") now say outright that the checker reports a closure's
Send, Sync, or Fn failure, and PAR-7 replaces that code with FX0611. So the
double marks in `par_join_reject.rs:12`, `par_shared_reject.rs:11,16`, and
`par_capture_reject.rs:12` are right. PAR-4 says no such thing. "`par_map`
requires `T: Pure + Sync` ... (`FX0612`)" has the same shape as LOCK-1's
"`T: Send + Pure` (`FX0606`)" and GLOBAL-1's, and their rows in `rules.md`
split it ("FX0606; E0277 for `Send`", "FX0502; E0277 for `Sync`"). Their
cases mark E0277 alone: `authority/lock_payload_not_send.rs` (main) and
fx-gcs.3's `global_static_cell.rs`. The base `par_map_cell.rs` did the same,
and this diff flips it (`par_map_cell.rs:6`, plus
`par_elements_reject.rs:10`). The result is one design shape marked two
ways.
Fix: revert those two marks to `E0277` and `par_map_cell.rs`'s comment to
"so rustc rejects it", and split both PAR-4 rows in `rules.md` the way
LOCK-1's is split ("FX0612; E0277 for `Send`/`Sync`"). This is narrower
than re-marking LOCK-1 and GLOBAL-1 in other beads' files.
Optionally, add one README sentence: an FX code goes on a `Send`, `Sync`, or
`Fn` failure only where the design attaches it to that part.

**B2. The `rules.md` row for 03 §3.1 still omits FX0610.** The diff adds
FX0610 to the needing-rule list in 01 D18 and 03 §3.1 (03:781), but the
inventory row that quotes the list (`rules.md`, "§3.1, 03:727 ... FX0517,
or FX0502 / FX0603 / FX0606 / FX0611 / FX0612") was not updated. Add
FX0610. Merge note: fx-gcs.3 rewrites the same row (as `= PURE-1`, adding
FX0518), the same sentence in 03 §3.1, and the same list in D18, so
whichever branch lands second must take the union of FX0518 and FX0610 at
all three places.

## Non-blocking

**N1 (nit).** The README's prefix table still reserves `CHAN` for 03 §6.4.
After this change, §6.4's only compile-time rule is `= SEND-3` and its lint
is excluded, so nothing uses `CHAN`. Delete the row (AP-MINIMAL: delete what
the change orphans).

**N2 (cross-bead duplication).** `par_float_reject.rs`/`par_float_accept.rs`
(PAR-3) and fx-gcs.3's `authority/admit_float.rs` (ADMIT-1) both show
`to_bits` and `sin` as FX0520 and `sqrt` as admitted. At merge, tag the
PAR-3 cases `ADMIT-1, PAR-3` and drop `admit_float.rs`, or keep both
deliberately.

**N3 (nit).** 03 §6.4 cites a bare "(SEND-3)", where 02 §8.5 writes the
file name when it first cites another chapter's ID ("`03-authority-concurrency.md`
PAR-5"). Write "`02-memory.md` SEND-3" in §6.4.

**N4 (nit).** `par_capture_reject.rs:16`: in
`clock.now().checked_add(Duration::from_secs(*x)).is_some()`, only the
capture matters. `xs.par_map(|_| clock.now())` shows the same thing and
needs no `Duration` import. `par_elements_accept.rs:3` says "owned strings
and plain data", but the file uses only strings.

**N5 (nit).** Rewrapping: the 01 D18 line (01:1011) and the 03 §3.1 line
(03:781) now overrun the paragraph width, and so does `par_map_cell.rs:2`.
fx-gcs.3 rewraps the first two anyway.

## Checks

- Fingerprint before review: matched `7d0b819e…71db6`.
- `/proc/loadavg` 6.02 before the run; `conformance/run.sh </dev/null`
  (full corpus, run once): `exit=0`, 121 lines, no FAIL; tasks cases: 5
  PASS, 6 AWAITS CHECKER with the expected FX codes. So rustc confirms
  E0524 (`par_capture_reject.rs:24`), E0507 (`par_shared_reject.rs:16`),
  and every E0277 mark.
- `run.sh --trace`: PAR-3, PAR-4, PAR-5, PAR-6, and PAR-7 each have
  accepting and rejecting cases (PAR-7's accepting cases are
  `par_join_accept` and `par_shared_accept`); SEND-3 gains
  `send_local_accept`.
- Line references checked at `a3b0177`: 02:764/765/768/775 and
  03:1258/1343/1425/1426/1429 are the cited sentences.
- Grep for FX0610/FX0611 across `design/`: 01 D18, 03 §3.1, 03 §8.1, ref:786,
  and companion:56 agree (a non-Pure, non-Send, non-Fn, or non-Sync
  closure is FX0610; a capture of a non-`Pure + Sync` `&T` is FX0611
  instead). Only the `rules.md` §3.1 row disagrees (B2).
- Compared against fx-gcs.3's worktree (`a69b2bb`) for marking convention
  and overlapping edits.

## Reasoning

**Scope coverage.** No rule in 03 §6–§8 is left as `none yet`. For the
excluded rules:
- TASK-1 to TASK-6 are runtime: failure propagation, precedence,
  cancellation, deadlines, and shielding. TASK-1's only static part, the
  `Owned` closure, is SCOPE-3.
- JOIN-1 and JOIN-2 are runtime join semantics. JOIN-1's borrow
  consequence is SCOPE-1, and "dropping a `Task` is allowed" rejects
  nothing.
- SUSP-1 and SUSP-2 classify and reject nothing; SUSP-3 is codegen.
- SEAL-2 and SEAL-3 wait for multi-file cases, which the epic comment
  tracks.

Each exclusion holds. Moving FX0608 to the excluded list also holds: the
row held only a lint, a case marks only errors, and the SINK-8 and LOCK-4
rows keep their lints only beside rejecting parts.

**Conversions.** SEND-3 (02:648) itself lists channel messages and every
boundary 03 §6.6 names, so `= SEND-3` is right for both rows. That leaves
`CHAN` unused (N1). §6.6's "only" is the task-isolation paragraph's
consequence, not SEND-3's text. `send_local_accept` (a non-`Send` `Cell`
live across `job.join()`) is a fair accepting case under the restatement.
`= PAR-5/6/7` follows the README's note that 02 §8.5 restates 03 §8.1.

**FX0610.** The resolution matches ref:785-786, which already put `join`'s
`Pure + Send` and the slice closures' `Fn`/`Sync` under FX0610. PAR-7's
"in place of" settles the overlap with FX0611: a closure that captures
`&Clock` is also non-Pure. The companion row (FX0610 for an owned `Arc` or
capability, FX0611 for `&RefCell` or a borrowed capability) agrees.

**Stub surface.** `std::par::join` is needed by `par_join_*` and
`par_capture_reject`. Its closure bounds are PAR-5's exactly. `RA: Send`
and `RB: Send` are not in the design and are marked GUESS, as the README
asks; they match rayon. `ParSlice::par_for` is needed by
`par_elements_*` and `par_shared_*`, and matches ref:779-780 character for
character. No other new `pub` items exist outside cases.

**Needless cases.** None within the bead. `par_map_cell` could fold into
`par_elements_reject`, but it is pre-existing. The PAR-3 pair overlaps
fx-gcs.3 (N2).
