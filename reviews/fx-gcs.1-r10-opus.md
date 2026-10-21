# Tide review, fx-gcs.1 round 10 (Opus): narrow check of the rebase onto a3b0177

Candidate: `27e96f8` on `a3b0177` (worktree `/home/lucca/pjs/lab/tide-fx-gcs.1`).
Scope: `git range-diff 76dfe06..ad837c0 a3b0177..27e96f8` and
`git diff ad837c0 27e96f8 -- conformance`. The only change since the approved
candidate is the eight `02:NNN` citations in `conformance/rules.md`.

## Verdict

revise

The eight shifted citations are correct. One stale line is left: the rebase
moved the inventory's 02 line numbers to `a3b0177`, but its header still says
the inventory was taken at `2d82c40`. The fix is one word.

## Blocking

1. **`conformance/rules.md:6` names the wrong snapshot commit.** The header
   says the inventory is "taken at `2d82c40`", and the "file:line of the
   definition" column reads against that commit. fx-t99 (`a3b0177`) is the
   only design change between `2d82c40` and `a3b0177`. It inserted 6 net lines
   into `design/02-memory.md` after line 817, and the amended citations now
   match `a3b0177`, not `2d82c40`. At `2d82c40`, line 02:857 reads "nor create
   one (D7).", 02:870 reads "is not `Pure` whatever its fields. ...", and
   02:898 reads "then `write!(f, "{n}")`) ...". None of these is the cited
   rule. So 8 of the 61 `02:` citations are wrong at the commit the file names.
   Fix: change `2d82c40` to `a3b0177` on line 6. That is accurate for every
   chapter: 03 and 04 did not change between the two commits, and fx-t99 added
   no compile-time rule (see Non-blocking 2), so the rule set does not change.
   This blocks because the inventory belongs to this change, and its citation
   contract (each row names the definition's file:line at the stated commit)
   no longer holds. The fix is text-only.

## Non-blocking

1. **Off-by-one on the `SE-0414` non-rule note (pre-existing, carried
   through).** `rules.md:177` cites `02:1336`. `SE-0414` is on 02:1335 at
   `a3b0177`, and that line held it as 02:1329 at `76dfe06`/`2d82c40`, while
   the approved candidate cited 02:1330. The +6 shift kept the original
   off-by-one. Line 1336 is the next line of the same bullet ("for
   thread-affine state..."), so a reader still finds it. Suggest `02:1335`
   while editing line 6. It is a nit: it sits in the "Considered and
   excluded" notes, not in a rule row.
2. **fx-t99's new not-`Pure` items need no new row.** The new items are
   `SimRoot`/`NodeRoot`, the test-only simulator handles (`sim.world`'s type,
   `Faults`, `SimNode`, `NodeTask`), and `LogConfig`. Each extends the data of
   one existing rule: `Pure` is decided structurally, plus the explicit
   not-`Pure` list. Row `rules.md:65` already covers that rule ("§9, 02:789
   (not-Pure list 02:815)"), and 02:815 is still "These are **not `Pure`**:"
   at `a3b0177`. Using the simulator types at all in non-test code is
   ROOT-3's row (`rules.md:84`, FX0504). The companion §17 sentence ("an
   entry cannot capture a node or the fault controls") is a consequence of
   these not-`Pure` facts plus the entry-capture bounds. It is not a new
   rejection rule, and the companion is outside the inventory's stated scope
   (02, 03, 04). A later case could exercise `LogConfig` or `SimNode` as a
   not-`Pure` capture under the 02:789 row, but the inventory does not need it.

## Checked

- Range-diff: the only content change is the 8 citation edits in
  `conformance/rules.md` (hunks at 65-73 and 174-177). No other file in the
  fx-gcs.1 commit changed.
- Every `02:N` and `02:N-M` citation in `rules.md` (61 in both versions),
  checked by script: the line text at `76dfe06` for the old number equals the
  line text at `a3b0177` for the new number. All 8 shifted citations
  (851→857, 864→870, 880→886, 883→889, 887→893, 892→898, 921→927,
  1330→1336) land on the same text as before. Every unshifted citation is
  at or below 815, above fx-t99's first edit (line 818), so it is unaffected.
  The 02:561-600 range is unaffected too.
- `design/reference-companion.md`: the only citation into it is `comp:38`
  (`rules.md:89`), above fx-t99's edit at line ~443. `conformance/rustc/std/src/lib.rs:425`
  cites "companion 17" by section, which is still §17 at line 388.
  `conformance/README.md` cites `02-memory.md` §11, which is still
  "## 11. Soundness sketch" at line 1297.
- No other file fx-t99 touched (beads, `reviews/fx-t99-*`) is cited from
  `conformance/`.
- `bash conformance/run.sh` on `27e96f8`: exit 0, 3 PASS, 2 AWAITS CHECKER
  (`admit_atomic.rs` FX0520, `lock_payload_shared.rs` FX0606), the same as the
  handoff. The run refreshed the gitignored `conformance/target/`, which
  existed before it. `git status` is clean.
