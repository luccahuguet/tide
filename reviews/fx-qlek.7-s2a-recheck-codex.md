Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 0d55380 (re-check)

## Scope and identity

Finder re-check of B1–B6 and the taken non-blocking item NB1 in `target/work/s2a-codex.md`, against their cited sources. Read that review first, then the builder's “S2a round 1 fixes” and `target/work/brief-s2a-fixes.md`. No fresh full review; NB2 was not taken and is outside this pass. The only remaining problem below was introduced by a fix.

Reviewed HEAD `0d553805635f71c720bf32a1a5f62f6727a6779f`, with clean tracked status. SHA-256 of `git diff 35d22ee 0d55380`: `038ff601c4c3695f3b8ea3cc40383a23764687a1c7ced5f8fa47ab0bf21d4942`. The fix diff touches only `paper/tide-v0.typ` and `reviews/fx-qlek.7-paper-terms.md`.

## B1 — Resolved

`paper/tide-v0.typ:340` now describes .6 as “160 raw GitHub results (158 unique), before language and star filters,” with 32 unique screened issues. This matches the opening and Counts in `research/38-external-concurrency-search.md`: 160 raw, 158 unique, 126 rejected by the filters, 32 screened.

At :343, .10 now says “5,076 keyword-match rows” and preserves the unresolved row. `research/44-fx-zu4f.10-concurrency-round.md:105` onward distinguishes 4,524 keyword-matching unique repository commits, 5,076 crate-membership rows, 306 E1 passes and one unresolved row. The paper no longer calls all 5,076 fixes.

At :356, “every candidate that reached the portability check failed it” matches the first-failure counts in research/38, /39 and /41: 15, 26 and 68 E4 failures, respectively, with no eligible rows or subsequent E5 failures. It no longer claims that most of the 251 screened rows failed portability.

## B2 — Resolved in both link modes

`paper/tide-v0.typ:35–39` suppresses repository hyperlinks while either input is a placeholder. The Status sentence at :145 visibly explains that the public repository and tag are pending and that paths print without links. With real inputs, the helper still selects `blob` for files and `tree` for directories and uses the supplied tag.

Independently compiled and inspected both outputs:

- Defaults: 18 pages; extracted Status text includes the pending notice. `pdfinfo -url` reports no repository link annotations and no destinations containing `OWNER/tide` or `RC-TAG`.
- `--input tide-repository=https://github.com/luccahuguet/tide --input tide-tag=v0.0.0-alpha`: 18 pages; the pending notice disappears. There are 118 repository link annotations, covering 50 distinct targets, all under that repository's `blob/v0.0.0-alpha/` or `tree/v0.0.0-alpha/`. The directory targets include `examples/case-studies/` and `reviews/`. No placeholder destination remains.

These checks establish generated link behavior, not the public existence of this illustrative tag.

## B3 — Resolved for this revision

`paper/tide-v0.typ:177` now explicitly says UDP, randomness draws and `Proc::exit` are planned, and that `Rand` is currently a capability identity with no draw methods. `Proc::command`, TCP, HTTP `get` and `resolve` remain described as implemented. This agrees with `README.md:372–374`, the capability table in `design/reference.md:338–343`, and `crates/tide-std/src/lib.rs:292–298`, which declares `Rand` and documents the absence of draw methods. No later release tier is assigned to draws.

The brief's required S6 follow-up is present as `target/work/plan.md:98`, row 6.1c: when fx-qlek.20 lands, update the paper's Rand sentences to the landed surface. It is marked todo, waiting for that bead. This follows the brief's current-revision wording decision without claiming unlanded methods.

## B4 — Not fully resolved: the fix introduces an undefined acronym

The originally named gaps are fixed: AI and std are defined in the abstract (:139), protocol and HTTP at :141, and TCP and TLS at :177. The corresponding terms-table entries now quote actual paper text. The other taken acronym edits likewise supply plain wording for CLI, OS, IO/CPU, PLDI and ABI.

However, the new TLS explanation at `paper/tide-v0.typ:177` is “TLS (the encryption layer of HTTPS).” **HTTPS is newly introduced by this fix**, appears nowhere else in the paper, and has no definition or row in the terms artifact. The baseline had zero occurrences; this candidate has one. `reviews/fx-qlek.7-paper-terms.md:72` repeats this explanation while :12 still declares “MISSING entries: none.”

This leaves the same verification gap under the binding 2026-11-18 19:51 first-use rule quoted in the original finding and at `reviews/fx-qlek.7-paper-terms.md:5–7`: every acronym must be defined in plain words at its first appearance. A reader must already know HTTPS to use the supplied TLS explanation. This is a fix-introduced gap, not a reopened audit of unchanged terms.

**Minimum fix:** replace the parenthetical with “TLS (which encrypts the connection)” and update the TLS row to quote it. Alternatively, define HTTPS as the encrypted form of HTTP and record that definition in the terms table. No other terms expansion is requested by this re-check.

## B5 — Resolved

`paper/tide-v0.typ:185` now opens “D9's parallel regions give schedule-independent successful results without exploring task schedules.” This matches the distinction in `research/45-bend-hvm-ideas-for-tide.md` §3, especially :187–223: schedule-independent parallel-region results versus reproducing one effectful execution, with explicit implementation qualifications. `design/03-authority-concurrency.md` §8.2 PAR-1 supplies the successful-result/sequential-equivalence contract.

The opening no longer excludes ordinary real-runtime tasks from physical parallel execution, consistent with that chapter's §7.3 and the thread spawning in `crates/tide-std/src/task.rs:1372–1375` and :1521–1529. The paragraph retains the previously accepted panic, trust-hole, non-theorem, partial-runtime and sequential-simulator qualifications. Its changed terms-table row tracks the new opening.

## B6 — Resolved

The caption at `paper/tide-v0.typ:254` now distinguishes source/build checks before running, use of the recorded seed and choices, and failure when inputs, trace or outcome differ. This matches REPLAY-2 at `design/03-authority-concurrency.md:2385–2389`, the before-start and during-replay boundaries in `docs/driver.md:411–427`, and the root/input, outcome and trace comparisons in `crates/tide-std/src/sim/manifest.rs:205–222` during `finish`.

The abstract and its terms-table entry now say the manifest “records,” rather than “pins,” the run's seed, sources and inputs. Neither replacement claims an independent immutable-seed check or pre-execution refusal for input mismatches.

## NB1 — Resolved

`paper/tide-v0.typ:273` now states that the committed records do not establish whether each red team was independent of the change's author or briefed with only the attack goal. This accurately qualifies the provenance available in the linked admission, Proc, HTTP and resolve reports (`reviews/2026-11-17-red-team-opus.md`, `reviews/fx-ayh2.1-redteam-opus.md` and its two re-checks, `reviews/fx-ff13-redteam-codex.md` and its re-check, and `reviews/fx-6d50-redteam-codex.md`). Those records describe the claim, surfaces and review activity without establishing both properties. The separate disclosure that the admission red team also wrote fixes remains at :294. No independence or blinding property has been invented.

## Verification

Environment: this Linux worktree at the candidate above; Typst `0.15.1 (9dfd3a08)` and system Poppler. Both commands exited 0 without Typst diagnostics:

```sh
typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2a-recheck-codex-default.pdf </dev/null
typst compile --root . --input tide-repository=https://github.com/luccahuguet/tide --input tide-tag=v0.0.0-alpha paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2a-recheck-codex-pinned.pdf </dev/null
```

Inspected extracted text with `pdftotext -layout <pdf> -` and annotations with `pdfinfo -url <pdf>`; the link assertions above passed. Page-count `pdfinfo` emitted a “Suspects object is wrong type (boolean)” metadata diagnostic and returned the counts successfully; this is not asserted as a fix-introduced finding. `git diff 35d22ee 0d55380 --check` passed. No Rust/Nix build, runtime test, external access, contact, commit or Beads write. Only this report and the two permitted Typst outputs were written.

## Verdict

**changes required** — B1, B2, B3, B5, B6 and NB1 are resolved. B4 remains open solely for the newly introduced undefined HTTPS acronym. Confidence: high; the remaining change is bounded to one explanation and its matching terms-table row.

Not checked: NB2, a fresh full-paper review, S2b visuals, live release destinations or new runtime/security proof. The approval risk remaining in this pass is the stated first-use-definition requirement.

DONE
