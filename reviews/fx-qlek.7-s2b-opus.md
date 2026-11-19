Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate dbaa719 (code, S2b)

## Verdict

**Changes required.** There are two blocking findings, and both are small:

- B1: the red-team figure's selectors can silently stop matching.
- B2: three figure captions type numbers by hand, although the candidate's own source comment says captions read them from the records.

Apart from these, the figures state their records faithfully. The search pipeline is sound and its falsifier works. The build is deterministic, and the dependency is pinned and gated.

## Scope

Reviewed `git diff 2e6f95a dbaa719`: paper/figures/** (build.mjs, count-reference.mjs, spec/, records/, data/, excerpts/, svg/, package*.json, README), paper/tide-v0.typ, reviews/fx-qlek.7-paper-terms.md and .gitignore.

Also read:
- AGENTS.md and GOVERNANCE.md (Merge Rule);
- the bead and its comments (`br --no-db`);
- the builder's S2b, r1-fixes and r1b sections;
- target/work/s2b-codex.md and s2b-recheck-codex.md.

Runs on the candidate, unmodified:
- `node paper/figures/build.mjs --check` → exit 0 ("22 files").
- `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2b-opus.pdf` → exit 0, no warnings, 22 pages.
- pdffonts shows only LibertinusSerif Regular, Bold and Italic and DejaVuSansMono (and its Bold), all embedded and subset.
- Pages 3, 7, 9 and 14 viewed in colour, and pages 9, 14 and 18 in grayscale (pdftoppm, 80 dpi).

Runs in the copy only: tamper tests in /mnt/build/tide-agents/fx-qlek.7-target/opus-copy (from `git archive dbaa719`, with `npm ci --ignore-scripts`). The copy was restored afterwards and `--check` exited 0.

Not run: count-reference.mjs or any paid call, and no Rust or Nix build. I did not re-derive the paid token counts. Their sum (24,932) and the stored blob hash (14649fc…, equal to `git hash-object design/reference.md`) were checked.

## Blocking findings

### B1. The red-team selectors can silently stop matching, and the rebuilt figure then misstates its record

**Where:** paper/figures/build.mjs:232–244, with spec/redteams.json.

Unlike the search selectors (build.mjs:280, :284, which `need` a match), the red-team counts have no match requirement:
- `blocking` (:232) is a substring filter on `**admitted at`;
- `open` (:233) is a `startsWith("admitted")` filter;
- each `finding` regex (:240–243) and each `nonblocking` regex (:244) is counted with no minimum.

A reworded record therefore yields 0, not an error. The only guard is `held > 0` in the routes branch (:236), and a lost blocking count raises `held`, so that guard passes too.

**Failure scenarios** (all run in the copy, each with `node paper/figures/build.mjs` exit 0, then `--check` exit 0):
- **T1.** reviews/fx-ff13-redteam-codex.md's lines `**Classification: X**` reformatted as `**Classification:** X`. The Net::get row becomes `blocking 0, nonblocking 0`, and the SVG reads "no blocking find". Table 1 on the same page says "1 blocking".
- **T2.** In reviews/2026-11-17-red-team-opus.md, `**admitted at` changed to `**Admitted at`. Admission becomes `held 44, blocking 0`, and the figure no longer shows the two found-and-fixed routes.
- **T3.** In fx-ayh2.1-redteam-recheck2-opus.md, `- **R1. ` changed to `- **Item R1. `. Proc::command's non-blocking count drops from 11 to 8.

Before a rebuild, `--check` reports each of these only as drift. Its own advice ("run: node … build.mjs") then writes the wrong figure, and the check passes. This is the brief's "a selector can silently stop matching" case.

**Minimum fix** (build.mjs only):
1. `need(n > 0, …)` for every `finding` and `nonblocking` selector, as `stageValue` does.
2. In the routes branch, classify every route row's verdict cell (c[3]) by an explicit list: `rejected`, `admitted`, `seccomp denies`, `Landlock denies`, the controls row, and the remaining kernel-outcome forms. `fail` on any row that matches none.
3. `need(blocking === <the number of rows whose c[4] says fixed>)`, or simply `need(blocking > 0)` for this record, which has a recorded fix quote.

For the per-heading records, also `fail` on any `^### [A-Z]\d+\.` or `^\*\*Classification` line that no selector matched. Add one falsifier like the search one, run in memory on every build: T1's rewording must make the build throw.

### B2. Figure captions type numbers by hand, contrary to the rule and to the candidate's own comment

**Where:** the comment at paper/tide-v0.typ:46, and the captions at :101, :167 and :274.

S2b adds the comment "Captions read the same records for the numbers they quote." (:46). Only the E3 caption (:290, `#e3.gate.S`) and the bench caption (:336, `#bench.*`) do that. Three captions type their numbers:
- **Fig. 2 (:101):** "25,000-token budget". The figure reads it from design/reference.md's "Budget" line.
- **Fig. 6 (:167):** "8 of 16 seeds fail, on the fixed one none". data/replay.json holds `seeds` and `buggy_failing`.
- **Fig. 9 (:274):** "0 caught of 87" and "`fx-zu4f.13`'s 1". data/search.json holds both. This caption was written in S2b.

**Failure scenario:** the README's refresh procedure (rerun the record commands, replace records/, rebuild) changes the demo's failing-seed count. replay.svg and data/replay.json update and `--check` passes, but Fig. 6's caption still says "8 of 16". The caption now contradicts the figure above it, which is the drift the pipeline exists to prevent. The relayed decision is that no number is typed by hand, and the brief lists a hand-typed number as blocking.

**Minimum fix:** in paper/tide-v0.typ, add `#let replay = json("/paper/figures/data/replay.json")`, `#let search = json("/paper/figures/data/search.json")` and `#let budget = json("/paper/figures/data/budget.json")`. Set the three captions' numbers from those fields, the way the E3 and bench captions already do:
- `replay.buggy_failing.len()` of `replay.seeds`;
- `budget.budget`, formatted with a thousands separator;
- the fx-zu4f.8 row's `caught.n` and `pool.n`, and the fx-zu4f.13 row's `caught.n`.

If tide-lead instead rules that captions are prose outside the rule, then line 46 must at least stop claiming they read the records. That is tide-lead's call; the json reads are the smaller change that meets the rule as briefed.

## Non-blocking findings

1. **Search `re` selectors sum every match** (build.mjs:283–289). fx-zu4f.10's `caught` selector matches two lines today (research/44:118 and :151, both `| Caught | 0: not caught |`). The sum is correct only because both are 0; had that round caught one, the figure would say 2. Require exactly one match per `re` selector unless the spec marks it as a sum, and anchor .10's selector to one table.
2. **fx-zu4f.15's `eligible` and `caught` use the same `count` regex** (spec/search.json:8–9), so the two stages are equal by construction rather than by the record. The record is frozen, so this is harmless, but `eligible` deserves its own selector.
3. **`--check` ignores stale files in data/** (build.mjs:510). It scans only svg/ and excerpts/, so a data/*.json left behind by a removed figure goes unnoticed. Add data/ to the scan and exempt reference-sections.json, which is an input.
4. **Nothing runs `--check` automatically.** It is wired into no suite or script, and plan.md row 2.7 is the only checklist mention. fx-qlek.20's later reference change and any demo or driver output change will need a records refresh and a recount. Add "`node paper/figures/build.mjs --check` exits 0 at the tag" to plan.md's S6 rows. Also note there that the whole-file count (the companion's Core count) is not tied to the blob hash.
5. **The dependency gate's rejections are recorded only in git-ignored target/work/report.md.** Those rejections are a d3 bundle plus a DOM shim, hand-written scales, and Typst-native drawing. The README records the choice, the pins, the license, the size, the rejected 2026-11-03 patch batch and the replacement cost, but not these alternatives. AP-DEPENDENCY-001 asks for the meaningful rejections to be recorded: add one README sentence, or have tide-lead copy them to the bead. Otherwise the gate is satisfied:
   - d3-scale 4.0.2 is pinned exactly, with overrides on its six d3 dependencies and on internmap;
   - the lockfile carries integrity hashes, every package is ISC, there are no install scripts, and node_modules is 1.7 MB;
   - `npm ci --ignore-scripts` reproduces the install.
6. **The Proc::command "fixed" quote covers only N1–N4** ("N1–N4 are fixed", recheck2). F1–F3's fixes are confirmed in fx-ayh2.1-redteam-recheck-opus.md. Add a second `fixed` quote from that record, so all five drawn "blocking, fixed" items are evidenced.
7. **Table 1 against Fig. 8.** Table 1's Proc::command row says "27 attacks held, 9 findings" (round 1). Fig. 8 draws 5 + 11 = 16 findings across three rounds. The two do not contradict each other, but a reader comparing them will stumble. One caption clause fixes it: "counts span every round of each red team". Tables 1 and 2 are also hand-typed; a Typst `assert` against data/redteams.json and data/search.json would tie them to the figures cheaply.
8. **Fig. 12 shades Tide's "Enforced in dependencies" cell as holding** ("yes, by design; v0 admits one crate"), while v0 admits a single crate. The legend says "with the stated scope", so this is defensible, but a skeptical reader will call it a design claim drawn as a result. Consider leaving that cell unshaded.
9. **spec/diagrams.json's "about" overclaims.** It says "a diagram cannot drift from the text it draws on", but the quotes check fragments, not cells. For example, the Network row's "`Net::get` against declared peers; `Net::resolve`" is unchecked: it is true per README.md's table, but the quote covers only "TCP loopback on `127.0.0.1`". Reword it to "each cell carries a quote that must still occur in its source".
10. **Unguarded `.match(...)[1]` calls** (build.mjs:147–148, 190–191, 229, 341) throw a TypeError instead of a named error when a record is reworded. They fail loudly, so this is not B1, but the messages are opaque. Wrap them in `need`.

## Reasoning

**Fidelity.**
- **Search (Fig. 9).** I evaluated every selector against its record (scratch script) and compared the result with data/search.json, the SVG and Table 2 for all ten rounds:
  - .15: 1/1/1/1;
  - .8: 87/87/—/0;
  - .5: 147 = 87 + 60 / 87 / — / 0;
  - .6: 160/32/0/0;
  - .7: 162/119/0/0;
  - .9: 100/100/0/0;
  - .10: 24 / 5,076 rows with 10 cores / 0 with 1 unresolved / 0;
  - .11: 7 / 42 with 7 cores / 0 / 0;
  - .12: 30 / 9 with 430 / 0 with 47 / 0;
  - .13: 4/4/—/1, drawn plain as .15's.
  
  Each selector except .10's caught matches exactly once (NB1).
- **Search caption.** It carries the binding wording verbatim in substance: a std gap, the fx-zu4f.8 screen, the fix, a new registration, the qualified order, admission and type-checking only, and no rate. fx-zu4f.12 appears only as arm totals.
- **Red teams.** The figure's counts match their records today: 42+2+2+2 controls = 48; Proc 27/5/11; Net::get 12/1/3; Net::resolve 15/0/2. They also agree with Table 1. The problem is their robustness (B1).
- **Budget.** The 15 sections sum to 24,932, and the whole file to 24,960, from the companion's Core count. The 25,000 budget is read from reference.md, and the blob hash is current. count-reference.mjs splits at `## ` outside fences and counts each part with scripts/count-tokens.sh, a baseline-subtracted `claude -p` count. The figure labels the two totals honestly ("each counted alone").
- **Replay.** The SVG agrees with records/demo-run.txt steps 1–3: seeds 0 2 4 8 9 10 12 14, 16/16 after the fix, and 40 events with identical hashes 9d66e1613fc69090 and db1996770f19511f. The manifest excerpt's trace_hash prefix agrees.
- **Excerpts.** The diagnostic and trust-diff excerpts are exact slices of their records, and their captions match: TD0501 at 20:12, E0425 at 20:21, and an added `From<io::Error>` associated function. The trust diff is described only as a summary and never "catches".
- **Pipeline.** The RUN-3 level strings match crates/tide-admit/src/invocation/sandbox.rs:100. The B4 fix is in place.
- **Simulator and tiers.** These agree with README.md's Simulator fidelity table and the tier rows, and planned items are marked "(planned)".

**Pipeline.**
- **Determinism.** No timestamps, randomness, locale or network. The count date is committed input, and building twice gives identical bytes (also shown by Codex).
- **The drift check can fail.** An edited SVG fails it, a changed reference.md fails it through the hash gate, and the in-build falsifier works and fails loudly if research/47 is reworded.
- **Search selectors fail loudly.** That is not true of the red-team selectors (B1).
- **Edge cases.** Missing headings and records `need`/throw, and the copy restored to exit 0.
- **Complexity** is proportionate: one 520-line script for eleven figures, no DOM and one scale package. No dead code beyond the minor points above.

**Reader.**
- Grayscale pages 9, 14 and 18 stay legible: every bar is labelled, ticks and crosses are drawn paths, and the catch row is set off by a tint and bold type.
- The Okabe-Ito hues are colourblind-safe.
- Fonts are embedded.
- First-use definitions for TD0501, the budget labels, the trust-diff marks and the manifest keys are present in Figs. 1, 2, 4 and 7. Codex B2 is resolved, and I found no further undefined figure term.
- No private detail appears.

DONE
