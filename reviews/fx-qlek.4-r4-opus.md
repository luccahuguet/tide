Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate ba40864

# fx-qlek.4 r4: Opus review (converge round, reports only)

## Verdict

**request_changes**. Confidence: high on the figures and the data; medium-high on the two blocking findings, which are about wording that a skeptical PL reader would attack. Both fixes are one or two sentences of text in paragraphs this change adds, and neither needs a new build check beyond recompiling.

## Scope

- Candidate: HEAD `ba40864` on branch `fx-qlek.4`, base main `27b394e`. Only file changed: `paper/tide-v0.typ` (+194/-6). SHA-256 of `git diff 27b394e ba40864`: `0a3422eabb2c2c2de69ad059b88d280c20a4d5dc4b6fff03bf800bc8d2033627`, which matches tide-lead's record. The worktree was clean, and `git diff --check` passes.
- Read first: AGENTS.md, GOVERNANCE.md (Merge Rule), `br --no-db show/comments fx-qlek.4` (the 2026-11-18 reopen comments, the r2 count finding, A1), and fx-qlek's launch-bar comment (2026-11-18 05:45).
- Built with the pinned typst (nixpkgs 567a49d…, typst 0.15.0) through heavy.sh while the load was 2.2: `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.4-target/r4.pdf`. The PDF has 11 pages; its SHA-256 is `001992df…8384`. That is byte-identical to Codex r3's two builds (`r3-codex/a.pdf`, `b.pdf`), which confirms the `date: none` reproducibility claim. Rendered every page with pdftoppm and looked at pages 4, 5, 7 and 8, with 250 dpi crops of Figures 2 and 3.
- Independent recomputation (Python, before reading Codex r3 or the builder report):
  - **E3** (`trials/e3-pilot/result.json`): S = 72, K_T = 66, K_P = 67. The ratio is 0.98507 ("0.99"), the interval is [0.947368, 1.0] ("[0.95, 1.00]"), the gate is "not met", and `plain_above_two_thirds` is true (1.5 × 67 = 100.5 > 72). The threshold that the paper parses from `sentence` is 1.5. "Bootstrap" is correct (protocol §10.3, 10,000 resamples; report line 151).
  - **Benchmarks** (`research/36-benchmarks/2026-11-18-bf6b958/records.jsonl`): there are 60 summaries and 0 stable ones, with CVs from 0.010356 to 0.467203 ("1.04% to 46.7%"). For each comparison, the median ratio and Tide-slower count:
    - Release run against `rust-release-tide-flags`: contracts 0.9950 (11/30), files 1.0400 (26/30), tasks 1.3140 (30/30), tcp 1.1772 (30/30).
    - Cold debug build against `rust-debug`: 4.6043, 3.9253, 3.8238, 4.4341 (each 30/30).
    - Tasks against plain `rust-release`: 2.2868.
    - No rounds were equal.

    Every number drawn and printed in Figure 3 matches, and so does every number in the note's §1.1, §1.2, §1.4 and §1.6.
- Earlier rounds (reviews/fx-qlek.4-r1-codex, r1-recheck-codex, r2-opus, r2-recheck-opus): nothing they fixed has regressed. The Fable disclosure, the rr wording, the tokio paused-clock caveat, the 48 routes with controls included, the checked and sampled contracts, the planned formatter, the driver-only manifest, the E1 raw-log scope and the governance exceptions are all intact. The r2 count finding is fixed: no hard-coded count remains.

## Blocking findings

### B1. The unforgeable-condition paragraph presents "rustc fixes those holes" as a route to unforgeability, and it leaves out Tide's own trusted code

`paper/tide-v0.typ:204`.

- **Text:** "Two routes lead there: rustc fixes those holes, and each new toolchain pin is audited against admission's allowlist; or a machine-checked proof over the admitted subset shows that no admitted program reaches them. […] Until one of these holds, v0's claim stays no accidental ambient authority, and we do not call v0's capabilities unforgeable."
- **Problem 1, epistemic.** The first sentence's necessary condition is right. The first "route" can never be known to hold, though. rustc has no soundness proof, so fixing the known `I-unsound` issues and auditing each pin cannot show that no unknown reachable hole remains. "Until one of these holds" therefore sets an exit criterion that route 1 can never verifiably meet. A PL reader will read it as "once the open issues are fixed, Tide will call its capabilities unforgeable". That is the overclaim this paragraph exists to prevent. The paper's own §4.1 says that the known reproductions are covered "not every variant".
- **Problem 2, scope.** "Every soundness hole that admitted code can reach" includes holes outside rustc, but both routes speak only about rustc. Two of these are on record:
  - `tide-std`'s own `unsafe` code: 19 `unsafe` sites in `crates/tide-std/src`, which fx-ip8d's 2026-11-17 comment lists as trusted unsafe code ("tide-std's runtime").
  - The completeness of admission's own rules. The #57893 routes that the red team found were admission gaps over rustc holes. fx-ip8d's acceptance requires "a written mapping from the calculus to tide-admit's rules with each gap listed".

  Route 1 does nothing for either, and route 2 covers them only as far as the model matches the code.
- **Minimum fix** (text only, same paragraph): replace the "Two routes" sentences with something like: "Two routes lead there. rustc could fix those holes, with each new toolchain pin audited against admission's allowlist, but fixing the known issues cannot show that no unknown one remains. Or a machine-checked proof over the admitted subset could show that no admitted program reaches one. Either way, Tide's own trusted code, tide-std's `unsafe` runtime and admission's rules, must also be correct, and a proof of a model covers rustc and that code only as far as the model matches them." Then keep the deferred-proof sentence and the closing sentence. A shorter acceptable fix: say that route 1 can only shrink the known holes, so only route 2 could support the word "unforgeable", and name the trusted code.

### B2. The third case study drops its README's account of what Rust needs, so its "gain" reads as unique to Tide

`paper/tide-v0.typ:336` (also the summary at `:338` and the abstract at `:124`).

- **Text:** "The gain here is not only a deterministic test of the fix, which Eon already had: the seed finds the interleaving without knowing the window in advance, and the failing seed reproduces it." The summary then says: "Tide makes that test cheap and deterministic: […] a seed can find the interleaving".
- **Source:** `examples/case-studies/eon-discovery-race/README.md`, "What the Rust version would need": "Rust can test this race, and Eon's does, with a hook in discovery. To let a scheduler find it instead, a Rust program needs its file operations behind a trait with a simulated implementation, and a scheduler that controls the threads, such as loom or shuttle. Those two schedule threads but simulate no file system…"
- **Why it blocks:** this is the same defect as r2's B3, where the paper dropped the slow-close README's tokio paused-clock caveat. Shuttle does seeded random schedule exploration, and the paper names it in Related work. A skeptical reader will ask at once why seed-found interleavings are a Tide gain. The README answers that question, and the paper drops the answer. The first case study keeps its "Rust can write the same test" sentence; the third has no counterpart.
- **Minimum fix:** after "…reproduces it." add one sentence from the README, for example: "Rust can let a scheduler such as loom or Shuttle find it too, once its file operations sit behind a simulated implementation the project writes and maintains; Tide's std provides that file system because `Fs` is a program's only file access."

## Non-blocking findings

- **N1. In Figure 2, the 2S/3 line looks like a bar both arms cleared.** `:226-229`. The left panel draws 2S/3 = 48 in the same red dashed style as the D0 threshold on the right, and both bars run well past it. At a glance this reads as "both arms passed". The caption explains it, but the figure itself should not invite that reading. Fix: draw it in `muted` and label it "ceiling" (for example, "above 2S/3 no 1.5× gain fits"). The fix is cosmetic. On the right, the "R = 0.99, 95% interval …" label (`:244`) is crossed by the dotted 1× line between "0.99," and "95%". Moving it to start right of x(1.0), or below the axis, fixes that.
- **N2. The figures are never referenced in the text.** `grep '@fig' paper/tide-v0.typ` returns nothing, although the labels `<fig-pipeline>`, `<fig-e3>` and `<fig-bench>` exist. A PL paper normally points to each figure, for example "(Figure 2)" after the E3 quote and "(Figure 3)" before the summary bullets.
- **N3. The sandbox box in Figure 1 overstates the partial level.** `:176`. "The test runs as a child process under Landlock and seccomp. The driver prints the level, `full` or `partial`, first." Under the actual level `partial, not enforced`, the run is not confined. Suggested wording: "…prints the level first: `full`, or `partial, not enforced`." The body text two lines above is already correct.
- **N4. The abstract's "a seed that finds the failing interleaving" drops "unaided".** `:124`. The r2-era summary sentence "It does not find the bugs" is gone, and the README headline says "Tide does not find it unaided". The summary at `:338` partly covers this ("A test of a property someone had to think of is what fails"). In the abstract, though, "a seed that finds the failing interleaving" can read as Tide finding the bug. Suggested wording: "…and, for a race, a seeded test that finds the failing interleaving once someone states the property, and reruns it exactly."
- **N5. The case-study claim covers a whole directory.** `:330`. "The case studies under `examples/case-studies/` … Admission catches none of them." This becomes false as soon as fx-zu4f.5 lands an admission catch in that directory. The bead already routes that case to this paper, but fx-qlek.7's audit should recheck this sentence.
- **N6. Figure 3's paired counts for builds are the figure's own derivation.** `:323`. The caption says the verdicts follow "as the note reads them (§1)", but the note gives Tide-slower counts only for run and test measurements, not for builds. The "30 of 30" build counts are recomputed from the cited records, and they are correct; I checked them. The caption could say "counted from the records with the note's rule".
- **N7. The links are still relative to the repository root.** This is the existing note from r1 to r3, owned by fx-qlek.7.

## Reasoning

- **(a) The unforgeable condition.** v0 is never called unforgeable. The only occurrences of the word are the heading and the closing denial at `:204`. Elsewhere the paper keeps "no accidental ambient authority" (`:202`), consistent with the launch bar ("never 'unforgeable'") and `docs/owner-guide/03-soundness-bugs.md`. The paragraph's statement that fx-ip8d is deferred and not started matches the bead (DEFERRED, P3, Later). Its hedge "prove the model, not the compiler" is right. B1 is about what the paragraph implies would count as enough. It does not claim anything false about v0.
- **(b) The figures.**
  - **E3 (Figure 2)** draws exactly the JSON's numbers: bars at 66 and 67 of 72, the interval from 0.947 to 1.0 with the point at 0.985, the threshold at 1.5 and "Gate: not met". The caption opens "a pre-registered pilot with a negative result", and the verbatim protocol §12 quote sits directly above. The figure reads as a negative pilot, apart from the visual cue in N1.
  - **Benchmarks (Figure 3)** puts the instability hedge in red above both panels: "0 of 60 … descriptive, not settled measurements". The axes start at 0×, so they are not truncated. Every loss is labelled "loss". The one sub-1× row, contracts at 0.99×, is drawn in neutral ink and labelled "tie (Tide slower in 11 of 30 rounds)". No row reads as a win. The caption's explanation of 1.31× against 2.29× is correct (1.3140 and 2.2868 recomputed).
  - **The pipeline (Figure 1)** matches `docs/owner-guide/06-replay.md` (`--tide-run-test --seed` records a manifest; `--replay` checks identity, then compares traces) and `08-sandbox.md` (driver-started runs, `--tide-run-test` and `--replay`, are sandboxed). Its caption keeps the direct-binary and RUN-2 limits. Apart from N3, it claims nothing the system does not do.
  - The data plumbing is sound. `--root .` with absolute `/…` paths, the schema assert and the per-summary `assert(found.len() == 1)` all fail loudly on drift. The win, tie and loss rule matches the note's 20-of-30 sign-test rule, and `fmt`/`pct3` round as the note does.
- **(c) The third case study** matches the README fact for fact: eon-8ehhn, Eon's single-reread fix with a hand-placed hook, the four modelled file operations, 13/100, 100/100 and 11/100 (confirmed by the `//@ reports:` lines in `buggy.rs` and `out_of_order.rs`), and the out-of-order variant shows reliance on the order, not that Orbit keeps it. The negative comes first. B2 is the one omission that changes how a reader weighs the claim. The A1 framing holds: replay and admission lead, E3 is negative and quoted verbatim, and the trust diff appears only as a summary of what a change may do, with no catching claim (`:254`, abstract).
- **Length and readability.** At 11 pages the paper sits within the bead's 8–12. The figures land next to their sections (Figure 1 at the top of page 4, Figure 2 under the E3 quote, Figure 3 before the verdict), and nothing is clipped or overlapping except the small label collision in N1. The new paragraphs keep the paper's plain register. The case-study section is now the longest evidence block, but every paragraph earns its place.
- **Codex r3**, read after I had formed this view, approved with only N1 (the links), and its numbers agree with mine. My two blockers concern how claims read, not the data, which may be why its pass did not raise them.

DONE
