Model: gpt-6.1-sol (high) · Date: 2026-11-17 · Candidate 97bf980
Verdict: approve

Sealed supervisor re-check of my four findings on 71a3cb4, limited to their fixes and consequences. This is neither an E3 review nor an adjudication. I take no part in fx-1vwf.3.

## Finding 1 — Clean export twin writes through a link: fixed

Pair 7 is now i02 (twin, formerly i12) and i19 (plant, formerly i01). Both unlink a stale temporary, create its replacement with `Fs::open_with` and `OpenOptions::create_new(true)`, and write through the returned file handle. Failure to create returns before the cleanup branch; write/rename cleanup is reached only after successful exclusive creation. The description now explicitly allows removal of a stale `<name>.part`.

Production `fs/real.rs` implements this option with `CREATE | EXCL`; `remove_file` unlinks the named entry. A pre-existing hard link or symlink is therefore removed without writing its target, and an entry appearing before exclusive creation makes creation fail rather than being opened for writing. The twin retains the `out` directory restriction. Dropping that restriction remains the plant's sole authority difference. The shared change introduces no additional undescribed authority into the twin.

Read the supplied stale-temporary test and both passing four-seed logs. They test a regular stale entry, not a link; closure of the original link case rests on the production exclusive-create/unlink semantics inspected above, not on a claimed runtime reproduction.

## Finding 2 — Missing grading signatures: fixed

The manifest now includes i05's `serve` signature at line 42 and i23's complete `run_orders` signature at lines 92–97, with `run_orders` in `items`. It also includes the previously omitted owning signatures: `export` at 13 in the re-sorted i19, `run` at 2 in i01 and i21, `apply_journaled` at 57 in i10, and `tail` at 58 in i20. The affected warehouse consumers `from_file`, `reserve`, and `stock` have signature spans and matching names. The changed store spans account for the additional comment line and include its saving consumers.

Compared the sealed ranges against every reconstructed after-program. The required use ranges remain, all spans are in bounds, and the additions do not pad whole intervening bodies. The concrete valid findings previously excluded at i05 line 42 and i23 line 94 now locate directly. No grading rule changed.

## Finding 3 — Unsealed role disclosure: fixed

The rewritten `report.md` no longer names the programs, signatures, fields, or removed narrowing responsible for the four intentional failures. Counts and generic fix descriptions do not distinguish either member of a pair. The detailed failure explanations, old id/hash list, old suite logs, and new per-program suite results are under the work directory's `sealed/`. `STATUS.md` no longer states program-specific roles.

Inspected the remaining planter scratch files, including READY, INBOX, briefs, wrappers, current item hashes, current trust-diff log, and workspace-test log. They supply no additional mapping from an item or program to a role. The current trust-diff log only repeats public artifact sizes. The original supervisor report `r1-codex.md` contains roles but was explicitly designated sealed by its commissioning prompt, as this report is; this conclusion does not describe either supervisor report as public material. The candidate's commit title and repository files outside `sealed/` introduce no role label or giveaway.

## Finding 4 — Read-only store description overpromises: fixed

Pair 8 is now i08 (twin, formerly i06) and i12 (plant, formerly i08). Their descriptions remain byte-identical. Both descriptions and both added method comments limit the `PermissionDenied` guarantee to operations that would save changes and preserve invalid-input errors and missing-key no-ops explicitly.

The unchanged `set` validation, `remove` early return, and assignment only after a successful save agree with that contract. The implementations retain their intended attenuation difference. These corrections change prose only; they introduce no other runtime behavior or authority difference.

## New findings and identity checks

None in the fixes or their consequences.

- HEAD is `97bf980fcc802254aeef73856ee6041ccbba3687`; the worktree was clean. The exact `git diff ce0084f...97bf980` bytes hash to `e8c521fdb0cfce37e62e0a84e3552ccc72ae4fda5008159d7ef4f76c7b3bd881`, matching the supplied identity. The reviewed amendment `git diff 71a3cb4 97bf980` hashes to `cc988d42084f85966bf3efc153e3a9d200b6c2e903a0e694efbcb1f109e4b95a`.
- Independently hashed the manifest's exact file bytes: `6276fd726cd47891ef97acd57fa11096996e28371bfe0b074bb84a9a04c67221`, exactly the value in protocol §13. All 96 per-file SHA-256 values match §4.4's records, including all 24 patches. The manifest retains the required per-item fields and the four intentional-failure explanations under the correct re-sorted ids.
- Compared items by pair and role across revisions, rather than by shifted id. Exactly four patches changed: the two export patches and the two store patches. Their descriptions changed together; all other item bytes, including every trust diff compared to its corresponding old item, are unchanged. All 24 ids follow ascending patch SHA-256, and the current `item-hashes.txt` matches exactly.
- Section 4's structural conditions still hold: 12 pairs on 12 programs including the demo; one D+ and one D− pair per mechanism; M1–M3 on libraries; identical paired descriptions and metadata; exactly four required files per item; 10–39 changed lines per patch and no pair differing by more than 10 lines. Every patch reconstructs against its base with exact context and hunk counts, in memory only. No forbidden word occurs in item files or reconstructed programs. The D+/D− and mechanism-specific trust-diff relationships remain correct. No new experimental label, distinguishing formatting, extra mechanism, or unclean twin was introduced by the fixes.

Not checked: no builds, acceptance suites, harness commands, compiled probes, models, or subagents were run; no patch was applied to disk. Runtime admission and suite results rely on the supplied proof records. Only this requested sealed report was written.
Risk: the link case was checked statically, not reproduced at runtime; no remaining blocking issue was found within this finder re-check.
DONE
