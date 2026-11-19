Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 01024fb (S2b re-check)

## Scope

This is a finder re-check of my blocking findings B1 and B2 from target/work/s2b-opus.md. I also looked for defects introduced by the non-blocking fixes. It is not a new full review.

Read:
- `git diff dbaa719 01024fb` (7 files);
- the "S2b round 2 fixes" section of target/work/report.md.

Tamper tests ran only in a fresh `git archive 01024fb` copy at /mnt/build/tide-agents/fx-qlek.7-target/opus-r2. I ran `npm ci --ignore-scripts` there. After each test I restored the file and confirmed that `--check` exits 0. The candidate worktree was not edited, and `git status` is clean.

## B1: red-team selectors can silently stop matching. **Resolved.**

build.mjs:217–281 now does four things:
- every route verdict must appear in an explicit `VERDICTS` list;
- the blocking routes must be > 0 and equal the rows mentioning "admitted at" in any case;
- held must equal the held-verdict rows less blocking;
- every `finding` and `nonblocking` selector must match, and any finding heading, classification line or bold item line that no rule reads fails the build (`ITEM`).

An in-memory falsifier runs on every build (:276–280).

I reran my three tamper tests in the copy. Each now makes `node paper/figures/build.mjs` exit 1 with a named error:

| Test | Result before (dbaa719) | Result now |
| --- | --- | --- |
| T1: fx-ff13 `**Classification: X**` changed to `**Classification:** X` | exit 0, "no blocking find" | `Error: reviews/fx-ff13-redteam-codex.md: ^\*\*Classification: (.*)\*\*$ matches nothing` |
| T2: `**admitted at` changed to `**Admitted at` in 2026-11-17-red-team-opus.md | exit 0, blocking 0 | `Error: … the admitted-then-fixed routes no longer read "**admitted at"` |
| T3: every `- **R<n>. ` changed to `- **Item R<n>. ` | exit 0, non-blocking 11→8 | `Error: … ^- \*\*R\d+\.  matches nothing` |
| T3b: only R1 reworded (a partial rewording) | n/a | `Error: … no rule reads this line: - **Item R1. …` |

The unmodified copy builds, and its `--check` exits 0 with 22 files. Its data/redteams.json is unchanged (42/2/2/2, 27/5/11, 12/1/3, 15/0/2).

## B2: captions typed their numbers by hand. **Resolved.**

paper/tide-v0.typ:51–58 reads data/budget.json, data/replay.json and data/search.json. The three captions now take their numbers from that data:
- Fig. 2 (:126) uses `thousands(budget.budget)`.
- Fig. 6 (:192) uses `replay.buggy_failing.len()`, `replay.seeds`, `count-word(replay.fixed_failing.len())` and `replay.recorded_seed`.
- Fig. 9 (:299) uses .15's `pool.n`, .8's `caught.n` and `pool.n`, and .13's `caught.n`.

The comment at :46 is now true for the figure captions.

**Tamper test** (copy): I changed records/demo-run.txt step 1 to "7 passed; 9 failed" and added seed 15, then rebuilt and compiled. The PDF's Fig. 6 caption read "buggy revision 9 of 16 seeds fail", so the caption follows the record.

**On the candidate:** `--check` exits 0 with 22 files, and `typst compile` exits 0 with no warnings (…/tide-v0-s2b-r2-opus.pdf). The pdftotext diff against the dbaa719 PDF shows only three changes: "one-row" became "1-row", Fig. 8's added "counts span every round of each red team", and a reflow of the unshaded Fig. 12 cell. Every caption number renders as before.

## Non-blocking fixes: no defect introduced

- **Exact-match `re` selectors.** fx-zu4f.10's caught is pinned to its two arm rows (`matches: 2`, research/44:118 and :151) and still gives 0.
- **fx-zu4f.15's eligible** is now rows minus out-of-class (research/49:402, 405). It equals 1, unchanged.
- **The data/ scan** exempts reference-sections.json.
- **Proc::command's fix quotes** now include F1–F3, from fx-ayh2.1-redteam-recheck-opus.md.
- **Tables 1 and 2** are now asserted against the data. Tamper test in the copy: research/38's "160 results" changed to "161" made `typst compile` fail with "assertion failed: Table 2 row fx-zu4f.6 disagrees with data/search.json: (161, 32, 0, 0)".
- **Fig. 12's design-claim cell** is unshaded.
- **Wording and error messages:** the diagrams "about" text is reworded, and `grab()` now names the source on failure.

**Remaining nits, not blocking:**
- §4.1's prose ("8 seeds pass and 8 fail (seeds 0, 2, …)", "40 events") is still typed by hand. It is prose, not a caption, and was outside B2.
- The README's "about 1.2 MB" counts package sizes; the installed node_modules takes about 1.7 MB on disk.

## Verdict

**Approve.** B1 and B2 are resolved, and none of the fixes introduces a defect.

DONE
