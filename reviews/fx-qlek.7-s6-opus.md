Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide 696580e + site f49a492 (S6 Opus converge, report-only)

## Verdict

**Approve.** There are no blocking findings. Every changed claim I traced matches its record, the texts agree with each other and with the site at the pin, the figure pipeline's change is correct and its check still fails on drift, and the governance edit matches the relayed decision. Three non-blocking findings are below.

Scope: `git diff d6ecd25 696580e -- . ':!.beads'` (14 files) and site `git diff 0f586a2 f49a492` (13 files). I also read GOVERNANCE.md and CONTRIBUTING.md in full, the inputs in target/work, and the standing decisions. I worked read-only. I wrote only this file and a scratch copy of the tree in my session scratchpad. I quote nothing from the audit's [drafts] parts.

## Checks run

| Check | Result |
| --- | --- |
| `node paper/figures/build.mjs --check` on the candidate | "figures match their records (22 files)", exit 0. Since `--check` regenerates every SVG and compares it, this also shows that no SVG is hand-edited. |
| My own falsifiers, on a `git archive HEAD` copy in the scratchpad (not the candidate) | Deleting one held bullet of `reviews/fx-qlek.20-s1-redteam-codex.md` gives drift on `svg/redteams.svg`. Removing a bullet's bold throws "a held item not read by ^- \*\*". Renaming `### R4 — ` gives drift. Renaming `## Held attacks` throws `no "## Held attacks"`. |
| `typst compile --root . paper/tide-v0.typ …/s6o/tide-v0.pdf` | exit 0. The Table 2 assert `(1567, 1567, none, 0)` holds. |
| Site PDF against my compile | `pdftotext` text of site `public/paper/tide-v0.pdf` (`dca9e1b0…`) is identical to my compile of 696580e (diff of 0 lines). |
| Site `status.json` input hashes | at-a-glance `97dc3e1a…`, README `1494d00b…` and 01-design `f9b09169…` equal the sha256 of those files at 696580e. README's at-a-glance block (lines 9–60) is byte-identical to `docs/at-a-glance.md`. |
| Privacy sweep of both diffs' added lines | No home or build paths, no desk or lead names, no private finds. The PDF metadata holds only the title, the author's byline, which was already there, and Typst. |

## Truth, at the records

- **Finding 1 (five red teams).** `reviews/fx-qlek.20-s1-redteam-codex.md`:
  - `:5`: "Sound with stated limits"; "the five draws".
  - `:15`: "Breaks (blocking): None established".
  - `:19,37,61,71`: R1–R4.
  - `:83–102`: exactly 20 `- **` bullets under `## Held attacks` (`:79`).
  - The R1 Boundary paragraph says this "exposes simulator decisions, not host entropy", which the paper quotes at `:247`.
  - The attacks the paper names at `:247` (randomized maps, address entropy, real handles captured into simulated tasks, ranges, cancellation) are bullets 83, 85, 90, 95 and 97.
  - Table 1's new row (`paper/tide-v0.typ:228`) and the abstract (`:90`) are supported.
- **Rand implementation reviews: 2 blocking.** `reviews/fx-qlek.20-s2-codex.md:9` (B1) and `fx-qlek.20-s2-opus.md:22` (B1). Both were resolved in `fx-qlek.20-s2-recheck-codex.md:3` and `-recheck-opus.md:7`. The references at `:462` name files that exist.
- **Finding 8 (fx-zu4f.5).** `research/37:53` ("1,507 in-date advisories screened; 87 qualify"), `:57` (12), `:6`/`:600` (all 60 open), and `:93–97` (every advisory read and judged). Pool = Screened = 1,567 fits Table 2's caption "Screened is what the round judged against its criteria". No other text still says 147 or "87 screened" for that round.
- **Finding 7 (abstract benchmarks).** `research/36:64,66,72` (3.8–4.6×, 5.2–6.3×, 9.2× for the test build).
- **Findings 9 and 12 (TD codes, the trust diff).** Applied in README, at a glance, tide-or-rust, the site glossary, home, how-it-works and the generated at-a-glance. A sweep for "every rejection carries", "names each rejection" and "every change gets" over README, docs, the paper and the site finds no leftover.
- **Findings 5 and 13 (Docker).** `docs/docker.md:83–84,100–107`.
- **Finding 14 (E1 tokens).** 5,156,350 / 569,596 ≈ 9.05.
- **Finding 2, site.** `reviews/2026-11-17-red-team-opus.md:13`: "route 6b, found by the Opus r2 review". The paper's `:211` already says the same.
- **Findings 6, 10, 15 and 16, site.**
  - Status: `research/37:22–31`.
  - Demo: `demo/README.md:1–11` names two claims, replay and the trust diff.
  - Check cost: `research/36:64,66`.
  - The bench table's cell now leads with the panel label.
- **Site clip rule.** The files the new README rule lists match what `recordings/record-clip.rs:203–247` reads from `TIDE_SOURCE`: the conformance case, `demo/service`, `demo/tests` and `demo/patches/lost-update.patch`.

## Agreement and leftovers

- No other text counts red teams or lists the attacked surfaces: README:146–151, the owner's guide, the site and tide-or-rust name only the 2026-11-17 red team, and no README, docs or site text names any other `*-redteam-*` record. So "five" leaves nothing else to update.
- Agreed text in the paper:
  - fig-redteams' caption ("every blocking find has been fixed, … stated limits") stays true with Rand's row.
  - `figures/README.md:34` says "eight records", and the spec names 8 records.
- `build.mjs:226–238` is minimal and correct:
  - `item` is optional, so the existing teams count as before.
  - A list line it does not match fails the build.
  - The new `rows.length > 0` guard turns an empty table from a silent −1 into a failure.
  - The `Rand` back-tick regex matches the new team name.

## Governance

- `GOVERNANCE.md:11–33` matches the decision as the standing decisions record it:
  - The human list is steering only: "scope, priorities and direction, and the owner calls".
  - The four human duties are dropped.
  - Agents design, specify, write, review and validate.
  - The Philosophy line now agrees.
  - The Merge Rule is byte-unchanged, including `:52`.
- CONTRIBUTING, README (`:562–565`, `:704–706`), the paper (`:94`, `:424`) and the site (footer, home disclosure, llms.txt) all say the same.
- A sweep for "supervision", "directs and decides", "approved decisions", "humans define/review/validate", "owner review/approve" and "under human" finds only two kinds of hit:
  - E3's untested "human reviewers" population (README:261, paper:327, site index:207, owner-guide 09:179);
  - the Merge Rule's "final merge authority" (GOVERNANCE:52, paper:426), which is excluded by the decision.

## Blocking findings

None.

## Non-blocking findings

**N1. "The agents made the technical choices" is stated without exception next to recorded owner technical decisions.**

- Where:
  - repo `paper/tide-v0.typ:94`;
  - site `src/pages/index.astro:282–284`;
  - repo `CONTRIBUTING.md:10–11` ("they make the technical choices").
- The records it sits beside:
  - `README.md:205–207` and `docs/owner-guide/09-experiments.md:210–212`: "The owner's decision of 2026-11-19 reversed that removal", a std conversion whose cost is that 17 scope sites name their error type.
  - `design/01-design.md:1099–1102` (D18): "Owner decisions, 2026-10-19: the direction … and the gate passed with checker architecture A then B".
- Scenario: a cold reader sets the disclosure against the README's std paragraph or D18 and asks which side made that call. "Made the owner calls" covers both, but nothing says what an owner call is.
- Why it doesn't block: the decision's own wording is "the owner calls", and the texts list them beside "technical choices", so nothing here is false.
- Minimum fix, if wanted: use "made the other technical choices", or name an owner call once, for example "the choices agents bring to the owner, recorded as owner decisions". The second is a GOVERNANCE.md wording change, so it is the holder's to schedule.

**N2. CONTRIBUTING gives two approvers for a landing.**

- Where: `CONTRIBUTING.md:11` says "an AI review approves each landing", and `:54`, unchanged, says "A maintainer runs the review … and decides whether the change lands".
- Both fit GOVERNANCE (an AI verdict is required, and the maintainer is the final authority), but read together they leave unclear who approves.
- Minimum fix: at `:11`, "and each landing needs an AI review's approval".

**N3. Table 1's Tried column is worded unevenly (nit).**

- The Rand row says "20 attacks held, 4 limits" (`paper/tide-v0.typ:228`). Net::resolve's row counts its 2 simulator limits as "2 findings" (`:227`).
- Both are true to their records, which use those words.
- Minimum fix, optional: "20 attacks held, 4 limits stated", or leave it.

Out of scope, noted only:

- Audit note (f), the re-capture of the figure records at HEAD before captions name a commit, stays open as the builder reports.
- The builder's scratch copy under `/mnt/build/tide-agents/fx-qlek.7-target/s6/tamper` and mine (`s6o/`) are for cleanup at retirement.

DONE
