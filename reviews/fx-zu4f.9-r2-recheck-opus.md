Model: Claude Opus 5.5 · Date: 2026-11-18 · Re-check 85642fe

# fx-zu4f.9 r2 re-check: Opus finder re-checks B1 and B2 (reports only)

## Verdict

**Approve.** B1 and B2 are both resolved. Nothing the fixes touch contains a new blocking defect.
Two text nits follow, and neither blocks.

## Scope

- Candidate: `85642fe` (it amends 110afe3; parent 891bdac untouched). `git diff 1e673c5..85642fe |
  sha256sum` = `1b80be9b3ede9adea23d255b5d56b65f9396861991f56181003f155163e65467`. The round diff
  `git diff 110afe3..85642fe -- research/41-qin-dataset-search.md` (`68757757…c4a8`) touches only
  the record.
- Read: the whole round diff, report.md's "Round r2 fixes" section, and the final record bytes
  (`git show 85642fe:…`) at every place the diff touches.
- Recounted independently by script, not with the author's r2-check.py. All 100 row-table first
  failures match their assessment blocks. The global counts are E1 2, E2 28, E3 2, E4 68, E5 0.
  Every tab/project cell matches the record's table: non-blocking servo 0/7/1/10, tikv 0/2/0/1,
  crossbeam 0/5/0/3, rand 0/2/0/0, tock 0/1/0/1, parity-ethereum 0/2/0/2, and the blocking rows
  unchanged.
- Checked every new link anchor against the headings: design/03 §1.2, §2.2 and §6.5, and
  reference §6.4, §6.5, §6.9 and §7.3. All of them resolve. The research/39 link resolves only
  after the rebase onto main, which the author discloses.

## B1: E2 widened. Resolved.

- **The rule.** :221 now reads: "A failure passes E2 when the evidence ties it to the row's bug,
  even tentatively (B32, N04, N37); a failure the evidence attributes to another defect, a PR-only
  revision or new code, or that nothing in the evidence links to the bug, does not." The old
  sentence that "passing E2 does not certify…" is gone.
- **The nine rows.** N05, N13, N14, N19, N25, N28, N30, N36 and N40 now record "First failure:
  E2". The row-table reasons match. Each block keeps its original quote as context, states why
  that quote is not this bug's symptom, and records that every §2 item was read. N19's block
  says its failure is on the fix PR, and N30's that its failure is on another repository's PR.
  The former E3/E4 reasons are kept as "(additional)".
- **B32, N04, N37** still pass E2. That is correct: their evidence links the failure to the bug.
- **Summaries:** "E4 (68 rows)", "E2 (28 rows)" (naming the nine rows) and "E3 (2 rows)" (N36
  removed) agree with the counts. "What this round does not show" now names deliberate omissions,
  planned interfaces and frozen experiment rules.

## B2: design omissions mislabelled. Resolved.

- **The table row** is now "Omitted by design, plus a reconstruction ban". It cites the
  four omissions to the right places:
  - Condvar: design/03 §6.5.
  - `select`: design/03 :377 and reference §7.3, `TD0605`.
  - `OnceLock`/`LazyLock` (`TD0520`) and `lazy_static!` (`TD0502`): design/03 GLOBAL-2 and
    reference §6.4/§6.9.
  - Arc counts: reference :447-449, inside §6.4.
- **Kept separate:** protocol §5's reconstruction ban.
- **:145-146** is re-worded to match, and the kernel paths have moved to a new "Platform
  mechanisms" bullet.
- **Platform row:** it now lists B18-B19 and N32-N36, as "including additional grounds for rows
  that fail earlier". I checked that each of those blocks states a kernel, hardware or interrupt
  ground: B18's AHCI driver, N32's kernel allocator, N33's port I/O, N34's syscall code, N35's
  radio and N36's DMA.

## Non-blocking fixes

- **Applied correctly:** the "frozen experiment operation bans (bead E4)" relabel, the
  registration time (14:21:39Z after 14:20Z), the authorship line ("the initial builder, then
  three successive Codex authors"), B26's #11172 wording, B49's heading, B59's path, the N07-N09
  sink sentence and the series links. I found nothing plainly wrong in them.

## Nits (non-blocking, no re-check needed)

1. :160 says "**Most** non-blocking rows share memory through `unsafe` … or report memory
   unsafety". The E4 groups now cover 17 non-blocking rows, and only about 7 of them are of this
   kind. Even counting every non-blocking row with such a ground, it is well under half. Say
   "Many". This sentence predates r2, but the r2 count change sharpens it.
2. Labelling passing checks as "E3 (additional): ✓" (N05, N13, N14, N19, N25, N28, N30, N40)
   stretches protocol §3. There, "additional" names a reason that would *fail* a later check, and
   other later checks are "not assessed". Either wording is clear in context.

DONE
