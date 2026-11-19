Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 9dae1f1 (re-check)

Scope: a finder re-check of my B1 and of N1-N7 only, on `git diff 0b12748 9dae1f1`. That diff touches only research/44 and arm-b-ledger.tsv. The re-check was read-only: plain Python and git reads with `GIT_NO_LAZY_FETCH=1`, and no network or build.

## B1: linked reports for B02.277 and B09.092, and the link-gap scope

**Resolved.** Each of the four parts checks out.

1. **Both reports were fetched and read.**
   - The private inventories now hold:
     - `repos/rustls/rustls/issues/1992` and its comments page (1 comment);
     - `repos/rust-lang/rust-bindgen/issues/2498` and its comments page (4 comments).
   - All four requests returned 200, between 06:40:02Z and 06:40:08Z, 2.1 s apart.
   - The inventory SHA-256s (`6e904269…`, `2fa68c9d…`) equal the ledger's new `inventory_sha256` cells, and both issue URLs are now in `sources`.
   - I read both issues in full:
     - rustls#1992: a FIPS-mode provider lists X25519, so a FIPS server answers with a HelloRetryRequest. This is deterministic group configuration.
     - rust-bindgen#2498: `--wrap-static-fns` emits a wrapper for a static variadic function that drops the `...`. The comments discuss va_list forwarding. This is deterministic code generation.
   - Neither shows threads, shared state, ordering or blocking, so E3 stands for both rows.
   - Each row has an appended superseding decision line with an `r1` field naming "Opus review B1", the superseded time, the failure and the inventory digest, and `read_in_full` names the issue and its comment count. The pre-fix copies are kept as `full-inventory.r0.json`.
2. **The widened check reproduces.**
   - `python3 -I link-gap-r1.py --all` gives "306 rows checked, 61 absent references".
   - My own variant over the 120 rows outside the r1 set (all E2-or-later rows except B02.277 and B09.092, now fixed) gives 25.
   - Every closing-keyword reference left is outside the fetch class:
     - B09.045 and B09.304 point to servo#426, a renamed-repository self-reference, as r1 recorded.
     - B09.230's "try to fix #2179" names a different bug this commit leaves open; it is from the r1 set and was already accepted.
     - B23.129's #794 is the discussion read by GraphQL.
     - B23.155's #211 is the feature request closed by the regressing PR.
   - The record's class breakdown (18 + 5 + 1 + 1 = 25, plus r1's 36 = 61) matches the script output, including B02.342's eleven.
3. **The scope sentences are now accurate.** Method item 3 now states the 183 re-read rows, the link-gap scope (183 rows in r1, then all 306), the reports fetched (24, then B09.199, B11.082 and B09.162, then B02.277 and B09.092) and the 61 references left in not-fetched classes. The opening bullet and deviation 8 say the same. The earlier overstatement ("every affected row … missing reports were fetched") is gone.
4. **Totals and identities are consistent.**
   - The ledger SHA-256 is `5a884b74…ed95`, as cited.
   - Counts are unchanged: E1 path 3,779, comment 76 and manual 914; E2 109, E3 187, E4 10, unresolved 1; 306 E1 passes and 197 quotes.
   - The decision files hold 1,518 lines over 1,220 IDs, matching deviation 9.
   - The ledger diff is exactly 10 rows.
   - `git diff --check` is clean.
   - The commit message adds a paragraph that matches the fix.

## Non-blocking items

- **N1, E1 viewers: resolved.** Deviation 3 describes `view-b.py` (15-line caps, 6 with `--brief`, and `--diff N`), `read-b.py` and `terse.py` (not retained). Method item 3 points there. The tool table lists `view-b.py` (`c59e9d63…`) and `read-b.py` (`86d2cde1…`); both hashes match the files. "Builder 7 also had" terse.py matches report.md's builder-7 notes.
- **N2, builder 2's rows: resolved.** The record now reads "builder 2 (Codex; 11 serde rows) and eight Opus builders".
- **N3, edit window: resolved.** The record names 0b12748 at 2026-11-19T06:23:33Z, which matches the committer date, then fixer 12's GETs and edits.
- **N4, `cap` column: resolved.** B16.018 and B16.128 now carry "not cap-only", agreeing with record:499-501 and the column description. `cap-b-r1.py` takes the row and tag as arguments, so its hash is unchanged; its tool-table label was updated.
- **N5, B06.397's reason: resolved.** The reason now quotes "contains some deadlocks" and says the commit deprecates the API and fixes nothing. E1 is unchanged. `field-b-r1.py` (`2df9ee78…`) is hashed in the tool table.
- **N6, undefined terms: resolved.** One sentence after the terms paragraph defines tide-lead, tide-desk ("desk decision") and RC.
- **N7, missing URLs: resolved.** B09.130, B09.151, B09.227, B09.235 and B09.253 now carry their commit URL in `e2_source`.

The fix introduced no new problem. Method item 3 has one overlong source line, which is style only.

## Verdict

**Approve.**

DONE
