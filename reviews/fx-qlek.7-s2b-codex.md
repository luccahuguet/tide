Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 29348f3 (code, S2b)

## Verdict

Changes required. Four blocking findings. The pipeline builds reproducibly and the current numerical results match the inspected records, but the search counts are hand-transcribed rather than derived, and figure wording and first-use definitions still violate the acceptance rules.

## Blocking findings

### B1. Search counts are hand-entered, and the source validator accepts a fabricated catch

**Files:** paper/figures/spec/search.json:8, paper/figures/spec/search.json:25, paper/figures/build.mjs:273–280.

**Evidence:** The spec contains literal pool, screened, eligible and caught values. The builder copies these into data/search.json and draws them. It verifies that numbers of two or more appear somewhere in a source quote, rather than extracting the value for the relevant field; zero and one are explicitly excluded. Thus this is a second numerical owner, contrary to the requirement that figures derive their numbers from the committed records, never hand-typed numbers. The README's claim that nothing in a figure is typed by hand is inaccurate for this path.

Concrete falsifier, run only in the permitted copy: change fx-zu4f.8's `caught` from 0 to 1 in spec/search.json, leave research/47 and its checked quote unchanged, run the builder, then run `--check`. Both exit 0; the derived data and SVG now report a catch for the record whose opening says “Nothing was caught.” The untouched candidate's eight rows agree with the source summaries; the defect is the manually maintained values and their ineffective source validation, not a claim that the candidate already reports that false catch.

**Minimum fix:** Derive each displayed search count from its source record, including zero and one. Keep source selectors and labels in spec, not duplicate numeric answers. A capture from a source summary or a count of committed rows is sufficient; the 87 + 60 total can remain a computed sum. Add one small falsifier for a changed source result so a stale or incorrect catch cannot be regenerated into a passing check.

### B2. The completed figures introduce terms without first-use definitions

**Files:** paper/tide-v0.typ:86, paper/tide-v0.typ:135, paper/tide-v0.typ:172; paper/figures/excerpts/trust-diff.txt:4; paper/figures/excerpts/manifest.txt:12,17,20; reviews/fx-qlek.7-paper-terms.md:100.

**Evidence:** The rendered PDF first uses TD0501 in Figure 1 on page 3; its explanation is deferred to Figure 3. Figure 4 introduces “associated function” without explaining it. Figure 7 introduces SHA-256, JSON, `roots`, `ordinal` and `input_hash` without explaining them in the preceding text or its caption. The terms artifact still places TD0501's first use at Figure 3 and does not cover these new excerpt terms. The builder's own S2b report acknowledges some of these omissions. The owner's first-use rule explicitly counts figures and captions; leaving them for S5 does not complete S2b's accepted figure text.

**Minimum fix:** Define TD0501 at its new first appearance. Add short explanations before or with the relevant excerpts, or remove unnecessary lines from the excerpts while marking omissions. Complete the terms pass for the actual generated figure text and update its artifact, including other newly exposed vocabulary in the budget/diagram labels. Preserve real output rather than rewriting it to invent definitions.

### B3. The highlighted catch figure omits the binding qualifications

**Files:** paper/figures/spec/search.json:5–19; paper/tide-v0.typ:272.

**Evidence:** Figure 9 highlights `fx-zu4f.15` as `1 row → 1 row → 1 → 1`; the caption says only that the catch is a re-test of chrono_anchor. Neither that row nor its caption discloses the post-screen choice of its pool, the std gap exposed by fx-zu4f.8 and fixed before the new registration's re-test, the qualified chronology, or that this was admission/type-checking only with no port executed. Those facts are correctly stated elsewhere in the paper and Table 2, but the review brief explicitly applies the binding catch wording to figure text and captions, and the owner wants a figure-and-caption skim to convey the story. The source record research/49:30–79 makes the post-hoc pool, gap/fix/re-test and limited scope central to what this single-row result shows.

**Minimum fix:** Qualify the highlighted row as a pool selected after the earlier screen. Expand the caption to state the std-gap/screen/fix/new-registration history, that every check is consistent with anchoring before the fix landed but independent proof of that order is not established, and the admission/type-checking-only scope. Keep fx-zu4f.8's zero result visible and do not turn these heterogeneous searches into a catch rate.

### B4. The architecture figure promises a TD code for every rejected crate

**Files:** paper/figures/spec/diagrams.json:5; paper/figures/svg/pipeline.svg:1.

**Evidence:** The new Admission box says “A rejected crate gets a TD code and no artifact.” The quoted source used to validate this box only says that the driver gates artifacts; it does not support the added universal diagnostic claim. Ordinary Rust errors can reject a crate without a TD code. For example, conformance/cases/memory/move_moved_reject.rs:11 expects E0382 and has no TD mark. The runner explicitly requires the driver's TD diagnostics to equal just the marks it owns while also rejecting rustc errors (crates/tide-conformance/src/main.rs:576–586). Thus the figure overstates the driver's diagnostic contract.

**Minimum fix:** Say “A rejected crate gets no artifact,” as the previous figure did, or distinguish Tide-rule TD diagnostics from rustc's own errors. Regenerate the SVG.

## Non-blocking findings

1. **The caption calls the search drawing “funnels,” although it deliberately avoids a proportional funnel.** paper/tide-v0.typ:272; paper/figures/build.mjs:281–302. The actual drawing is a stage table with logarithmic bars, and some units differ between stages (24 crates versus 5,076 rows). The builder correctly chose this form to avoid misleading proportions. Rename it “search stages” or “search record.”

2. **Font embedding is a PDF guarantee, not a standalone-SVG guarantee.** paper/figures/build.mjs:27,76; paper/figures/README.md:43–47. The SVGs name font families but contain neither embedded font resources nor outlined text. The compiled paper embeds its fonts, so the paper delivery passes this check. A site reusing these SVGs must supply those fonts or outline them; otherwise the promised metrics and appearance are not preserved. The builder already records this limit.

3. **The recount helper is an exception to an in-tree-only write claim.** paper/figures/count-reference.mjs:29–38. It writes section text into the operating system's temporary directory and removes it only after every counter call succeeds; an exception leaves the temporary directory behind. The normal figure builder writes only under paper/figures. If strict in-tree writes are required for the optional recount too, put its temporary directory under the figure tree and clean it in `finally`. This helper was inspected, not run.

## Reasoning

**Frozen surface and environment.** Reviewed a913f46..29348f3868ca5bca0b37f66f8e8c637de429620f, diff SHA-256 `951f34527461d4f46774d4559a15001e758ede15b2e658c3ee5e6426af78a4cf`. Read AGENTS.md, GOVERNANCE.md, the bead and its comments, the builder's Stage S2b account, and plan rows 2.7, 2.8 and 2.10. Assumed one author-side figure build, as documented. No delegation, contacts, commits, ledger writes or implementation edits. Build mutations were confined to `/mnt/build/tide-agents/fx-qlek.7-target/s2b-codex-copy/`. The candidate remained at the named HEAD.

**Data fidelity.** Read all data/*.json and spec/*.json and traced the eight search summaries to research/37,38,39,41,44,46,47,49. The mixed units and fx-zu4f.10's one unresolved row are retained. Recounted the red-team classifications/table rows against the admission, Proc, Net::get and Net::resolve reviews and their fix records: 42 held/2 blocking/2 open plus 2 controls; Proc 27 held/5 blocking/11 non-blocking across the named rounds; get 12/1/3; resolve 15/0/2. These are recorded attacks/findings, not an exhaustive security audit or independent trials. The diagram cells' source quotes occur in their named documents; B4 is an unsupported addition despite a valid quote.

E3 matches trials/e3-pilot/result.json: 66/72 and 67/72, ratio 0.9850746268656716 (displayed 0.99), interval [0.9473684210526315,1] (displayed [0.95,1.00]), threshold 1.5 and gate not met. The ceiling is computed from S=72. Benchmark paired slower counts agree with the raw records; ratios use the recorded summary medians (which are rounded), matching the note and existing method. The SVG retains the unstable-runs warning and distinguishes the tasks profile-flags comparator. The budget reads 25,000 from the reference, 24,999 from the companion, and section counts summing to 24,971 from the committed counter output; the source blob matches. I did not independently repeat the paid token counts. SVG text was inspected for all eleven figures and spot-checked against these values and PDF text.

**Real excerpts and records.** The records name a913f46ead5d5bd4d432a82f554fedb4809b2e83 and their capture commands. The relevant examples, demo and driver sources are unchanged between that revision and the candidate. Code excerpts equal the current loader functions, and the diagnostic's line/column matches the ambient call. The available raw demo-run log contains the committed body after its documented path normalization and removal of build-wrapper lines. The committed manifest equals the available original manifests. Its suite-source SHA-256 equals demo/tests/src/hits_sim.rs, its library SHA-256 equals service/src/hits.rs with the committed lost-update patch applied, and its embedded trace has 40 events with the recorded full trace hash. Demo source also checks byte equality of the replay traces; this review did not execute it. The manifest excerpt's count and hash truncations are derived and marked with ellipses. No secret, private report, executable malware payload or undisclosed find was identified in the added records/spec/data/excerpts.

**Build and drift proof.** On Node v24.22.2 in the permitted copy, `node paper/figures/build.mjs --check` exited 0 and reported 22 matching files. Two successive builds left all generated data, excerpts and eleven SVGs byte-identical to the candidate. A changed SVG failed `--check` with exit 1 naming the file. Adding a newline to the copied design/reference.md failed with exit 1 and the recount instruction. Restoring the copy returned the check to exit 0. A reference edit therefore requires a recount and regeneration; the hash gate does not silently reuse the old budget. The future recount's date is capture metadata read as committed input by the deterministic build, not a build-time timestamp. The builder uses no random IDs, locale formatting, live font measurement or network fetch. B1's separate falsifier demonstrates the source-validation gap even though ordinary output drift is detected.

**Dependencies.** The builder's AP-DEPENDENCY-001 comparison records the scales capability, the rejected full-D3/DOM and native alternatives, the owner-directed choice, eight-package transitive scope, ISC licenses and approximately 30-line replacement cost. README records the abbreviated choice. package.json fixes d3-scale 4.0.2 and every transitive override exactly; the lockfile fixes versions, registry URLs and integrity hashes. Installed packages match all eight pins and declare ISC, with no install/postinstall scripts. No incompatible dependency terms were identified against the repository's Apache-2.0 license. The build imports the installed scale library and fetches nothing. I reused the existing installation and did not run npm install.

**Paper delivery.** `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2b-codex.pdf` exited 0 with no warnings; `pdftotext -layout` extracted 20 pages. All figure placeholders are replaced; pending repository/tag defaults are deliberate and disclosed in Status. Each major numbered section has a visual, including E1's existing table. The trust-diff caption remains a summary of declared authority, with no claim that it improves catching. The PDF contains font programs for Libertinus Serif regular/bold/italic and DejaVu Sans Mono regular/bold. The palette plus direct numeric labels, ticks/crosses and filled/open dots makes grayscale and colorblind reading plausible without relying solely on color. This review inspected SVG structure and extracted PDF text, not rasterized pages or a physical grayscale print. `.gitignore` covers node_modules and target output; `git diff --check a913f46 29348f3` passed.

Not run: count-reference.mjs, the paid API, Rust/Nix builds or runtime probes. No claim is made that this review independently reproduces compiled tool output or visually verifies page layout.

DONE
