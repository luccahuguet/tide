# Tide review, fx-gcs.1 round 11 (Opus): confirmation of round 10 fixes

Candidate: `c029dca` on `a3b0177` (worktree `/home/lucca/pjs/lab/tide-fx-gcs.1`).
Scope: `git diff 27e96f8 c029dca`, which touches `conformance/rules.md` only.

## Verdict

approve

Both round 10 fixes are applied correctly, and nothing else changed.

## Blocking

None.

## Non-blocking

1. **`01:956` is true at the named snapshot, but not at the candidate
   (pre-existing nit).** `rules.md:182` cites `01:956` for the
   authority-policies line. At `a3b0177` that line is
   "**Authority policies** (v0): ...", so the citation holds at the commit
   that `rules.md:6` names. This candidate also inserts one line into
   `design/01-design.md` at about line 145. That moves the cited line to
   01:957 at `c029dca`. The header names the snapshot, and its stated scope is
   02/03/04, so the citation is correct as written. `01:84` sits above the
   edit and is unaffected. No change needed. If anything, a later edit could
   say that 01 citations also read at `a3b0177`.

## Checked

- `git diff 27e96f8 c029dca`: exactly two one-line edits, both in
  `conformance/rules.md`. Line 6 changes `2d82c40` to `a3b0177`, and line 177
  changes `SE-0414`'s citation from `02:1336` to `02:1335`. No other file and
  no other line changed.
- `SE-0414` is on 02:1335 at `a3b0177` ("style of Swift's SE-0414 and an
  audit of std, ...").
- Every `NN:N` / `ref:N` / `comp:N` citation in `rules.md` (about 180
  occurrences) was resolved against `git show a3b0177:design/<file>` and
  read. Every rule row lands on its rule text. Samples:
  - 02: 02:857 (`OnceLock` and `LazyLock` not admitted), 02:870 (`Arc`
    omits ...), 02:886/889/893/898/927 (the shifted §9 rows), 02:738
    (`Shared<T>` requires `Send + Pure`), and 02:815 ("These are **not
    `Pure`**:").
  - 03: ROOT-1 through ROOT-5 (03:268-294), GLOBAL-1 through GLOBAL-6,
    LOCK-1 through LOCK-6, TASK/JOIN, PAR-1 through PAR-4, SUSP,
    REPLAY-1/2, and the v1 exclusions FFI-1 (03:969), RUST-1 (03:1778),
    and LOCK-7 (03:1330).
  - 04: CLAUSE-1 through CLAUSE-8, SEAL-1 through SEAL-6, and 04:180/187/202/213.
  - ref: ref:207 (FX0203), ref:307 (FX0402), ref:549 (FX0501), ref:725
    (FX0505), ref:803 (FX0702), ref:953/955/956 (the unsafe-grant table), and
    ref:1044 (FX0516).
  - comp: comp:38 (`thread_local!` row).
- `conformance/README.md` cites by section, not by line. Every section it
  names exists at `a3b0177`:
  - 02: §1.2, §1.4, §2.1-§2.8, §3, §4, §5.1-§5.4, §6, §8.1, §8.2, §8.4, §8.5,
    §9, and §11.
  - 03: §0, §1, §2.6, §3.1, §3.2, §3.5, §6.4, §6.5, and §8.1.
  - 04: §1.2.
- The named snapshot is therefore accurate for every chapter citation in
  both files.
