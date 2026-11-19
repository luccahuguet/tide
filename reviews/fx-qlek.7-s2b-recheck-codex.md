Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate dbaa719 (code, S2b re-check)

## Scope and proof identity

Read-only finder re-check of B1–B4, taken NB1/NB3, and the specified round-1b additions. Compared `git diff 29348f3 dbaa719 -- paper .gitignore` (SHA-256 `3761eefd4901701b77ee3fc9ea556347f36dc59754d43b87f51a1b770fbd990a`), read both decision briefs and the two account sections, and checked the terms-artifact changes. HEAD remained `dbaa719ae2ebcb8c33dc5d30f46a0252519f1387`; the tracked worktree was clean before writing this report. This is a fix re-check, not another full review of unchanged S2b work.

Executable figure checks used Node v24.22.2 and the existing dependency installation in `/mnt/build/tide-agents/fx-qlek.7-target/s2b-recheck-codex-copy/`. All input tampering and regeneration were confined to that copy. No contacts, delegation, commits, implementation edits or `.beads/` writes.

## B1 — Resolved: search counts derived from records

`paper/figures/build.mjs:275–309` derives numeric values through captures, matching-line counts and sums of named variables. `paper/figures/spec/search.json:4–59` holds selectors and labels, not numeric stage answers. Zero and one are parsed too. The 147 pool is computed from the captured 87 and 60; the numeric additions in unit labels (cores, unresolved rows, property rows and modeled properties) are also captured variables. I independently evaluated every selector against its record and compared all ten rounds with `data/search.json` and the SVG text. No displayed numerical answer in the generated search figure is hand-entered. Table 2 and narrative remain authored summaries; their counts were checked against the records, rather than claimed to be builder outputs.

Ran the requested falsifier: in the copy only, replaced research/47's single `**Nothing was caught.**` with `**One was caught.**`, leaving spec and generated files unchanged. `node paper/figures/build.mjs --check` exited 1 and named `paper/figures/data/search.json` and `paper/figures/svg/search.svg` as drifted. Restored the source. The builder's permanent in-memory falsifier also runs on every invocation and requires the changed source to yield caught = 1 (`build.mjs:305–307`). README lines 5–8 and its search-source row describe the actual ownership.

## B2 — Resolved: first-use definitions

`paper/tide-v0.typ:86` defines TD0501 at Figure 1, confirmed in extracted PDF page 3. Figure 2's caption (`:101`) explains its reference-heading vocabulary, including trust holes and the planned toolchain loop. Figure 4's caption (`:137`, PDF page 5) explains the diff marks and associated function. The replay paragraph (`:162`) explains event/output traces before their figure. Figure 7's caption (`:173`, PDF page 7) explains JSON, SHA-256, roots/ordinal, profile/debug build, outcome and the two hashes. The input_hash explanation agrees with `crates/tide-std/src/replay.rs:194–197`, which clears that field before hashing the encoded manifest. Signals and pseudo-terminals are glossed at their text uses.

`reviews/fx-qlek.7-paper-terms.md:167–209` records the figure pass and the additional round-1b terms; TD0501's earlier row now identifies Figure 1. The requested definitions are present at the actual figure appearances. No fix-introduced terminology blocker found.

## B3 — Resolved: binding catch qualifications

The highlighted row's note (`spec/search.json:5`, generated Figure 9 on PDF page 14) says the pool was chosen after the prior screen and that this was admission/type-checking only, with no port executed. The caption (`tide-v0.typ:274`) states the std gap, fx-zu4f.8 screen, fix and new-registration re-test; qualified chronology, including the lack of independent proof; and limited execution scope. It preserves fx-zu4f.8's 0 caught of 87 and expressly denies a combined catch rate. These match research/49's disclosure/result and research/51 §4.

## B4 — Resolved: rejection contract

`spec/diagrams.json:5` now says “A rejected crate gets no artifact.” The regenerated pipeline SVG contains that statement and no universal promise of a TD code. Ordinary rustc rejection is no longer misrepresented.

## NB1 — Resolved: search drawing name

`tide-v0.typ:274` calls Figure 9 “The search record as stages.” It matches the stage table and explicitly labeled logarithmic bars.

## NB3 — Resolved by inspection: recount scratch cleanup

`paper/figures/count-reference.mjs:28–43` creates its scratch directory under `paper/figures/node_modules/.count-reference-*` and removes it in `finally`, including when a section counter throws. It no longer uses the operating-system temporary directory. I inspected this change without executing the paid recount helper.

## Round 1b: fx-zu4f.12 and fx-zu4f.13 — Resolved

Both rounds appear in Table 2 (`tide-v0.typ:264–265`), §4.4.3 (`:277`) and Figure 9. For .12, `spec/search.json:48–53` captures pool 30, screened 9 fix candidates plus 430 property rows, eligible 0 fix candidates, 47 modeled properties and caught 0 from research/52's selection table and arm-result summaries. The generated data/SVG agree. The paper uses arm totals only: no per-crate property/test/failure counts, project name or detail that narrows the private candidate. The prose calls it model-only and uncounted, consistent with research/52's disclosure and result.

For .13, `spec/search.json:54–59` captures pool/screened 4 and caught 1 from research/51, with eligibility unspecified. `same_catch_as` names .15; the builder validates that reference and draws this repeated count without catch highlighting. Table 2 says “1, not new”; its pool cell, the SVG note, caption and prose all say this is .15's catch counted once. The wording agrees with research/51 §4 (`:282–307`): previously screened pool, capability-triggered re-screen, no independent second test. No rate is reported. Extracted PDF pages 13–14 retain the full rows and qualifications.

## Round 1b: manifest omissions — Resolved

`build.mjs:138–141` computes omitted keys from the recorded objects and adds visible `"…"` entries. I compared record and excerpt keys: the build omission lists exactly `target, options, versions`; the top-level omission lists exactly `evaluation, config, choices, complete`. Both marks appear in the compiled paper on page 7. README's manifest row now says this is a subset and describes those marks plus the shortened hashes/compiler/outcome and elided dependencies/trace. It matches the excerpt.

## Round 1b: budget and reference drift — Resolved

All 15 `(tokens, heading)` pairs in `target/work/count-reference-r1b.log` exactly match `data/reference-sections.json` and the section list in `data/budget.json`. Their sum is 24,932. The whole-file figure is 24,960, matching the companion's recorded Core count and the round-1b account of the whole-file counter run. Section 6 is 5,987; the budget remains 25,000. SVG text and PDF page 3 show these values.

The 28-token difference is between independently counted sections and one whole-file count, not a missing section or 28 tokens of spare budget. The helper invokes `scripts/count-tokens.sh` separately for every section; that script measures each prompt relative to a separate `.` prompt, so these independently measured token costs need not add to the whole-file measurement. Figure 2 itself labels the sections “each counted alone” and states both totals in its footer. It therefore discloses the distinction rather than presenting 24,932 as the whole-file count. The exact difference is observed from the logged counts; no new paid measurement was made here.

`git hash-object design/reference.md` is `14649fcf345368f484817909e5d5b14f9ce60953`, equal to the new stored blob. Baseline `node paper/figures/build.mjs --check` exited 0 (22 matching files). Adding a newline to the copied reference made the check exit 1 with the reference-changed/recount error; restoring it returned the check to exit 0. The hash gate works on the new reference.

## Typst and regeneration — Resolved

`typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2b-recheck.pdf` exited 0 with empty stdout/stderr: no warnings. `pdftotext -layout` succeeded; inspected affected figure/caption text on pages 3, 5, 7, 13 and 14 of the 22-page PDF.

In the copy, `node paper/figures/build.mjs` exited 0 and wrote 22 files. SHA-256 comparison before/after found every generated data, excerpt and SVG byte-identical. The final restored-copy `--check` exited 0. `git diff --check 29348f3 dbaa719 -- paper .gitignore` exited 0.

## New blocking findings

None introduced by these fixes. No override or alternative design needed.

## Verdict

Approve. The requested findings and round-1b additions are resolved within this finder re-check's scope.

Not checked: paid count-reference/API calls, Rust/Nix builds, runtime probes, another link-mode build, rasterized page layout or physical grayscale output. PDF text and SVG structure were inspected; visual page-layout quality remains outside this re-check's proof.

DONE
