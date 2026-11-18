Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 702cac8 (docs)

## Verdict

changes required

The paper follows A1 and the chosen positioning. Five bounded factual corrections remain; none calls for changing Tide's design. Confidence: high in the cited source discrepancies. No override recorded.

## Scope

Read-only supervisor review of fx-qlek.4, including `br --no-db show fx-qlek.4` and all its binding comments. Read AGENTS.md and GOVERNANCE.md first. Reviewed the complete added paper, the builder's 48-row claim table and decisions as assertions to verify, and the rendered PDF.

- HEAD: `702cac8de335a6f270cef86cbdad25c46c76048d`, branch `fx-qlek.4`.
- Base: `d4a91a2e49f4e98f01d107f950fceb929c2014fc`.
- SHA-256 of the bytes from `git diff d4a91a2..702cac8`: `8259091dfe3491ba04fc2b949bbfb95c8d820058ae2261122c803d7f9ac72803`.
- Only changed tracked file: `paper/tide-v0.typ`. The worktree was clean before review. The evidence sources below are landed base-tree files, not builder-generated evidence.

Compared the paper with the E3 report and protocol, E1 gate report, research/36, both case-study READMEs, the demo README, the red-team record and all four fx-qlek.5 review/re-check files, install/container docs, relevant owner-guide chapters, D0/D7/D8/D10/D11/D14/D15/D18 and the tier table, and the core reference. Checked the supporting census, RustAssistant research, positioning note and Achilles review. Inspected contract gating and simulator manifest recording where necessary to resolve delivery claims.

No implementation, ledger, branch, commit, main-checkout or other-worktree writes. This report is the only repository artifact written; compilation/rendering scratch outputs went under `/tmp`.

## Blocking findings

### B1. Distinguish checked contracts from sampled contracts and preserve the measured test-build loss

`paper/tide-v0.typ:58`, `paper/tide-v0.typ:133`.

The paper says contracts are runtime checks in debug and test builds, then presents the ordinary-debug benchmark without saying its two sortedness clauses were not evaluated. D10 (`design/01-design.md:730`) and `design/reference.md:877` distinguish cheap checked clauses from sampled clauses, which run only in debug test builds. `crates/tide-macros/src/contract.rs:78` implements that distinction.

This matters to the reported costs: `research/36-benchmarks.md:172` explicitly explains the skipped sortedness checks. Its summary at line 72 and test-build results at lines 193–210 report the build that actually checks them: 7.7x the plain Tide debug build and 9.2x Rust debug, with the separate-run and harness confounders stated. Omitting both the skip and that loss makes the contracts summary materially incomplete under the requested faithful treatment of benchmark losses.

**Minimum fix:** qualify §2.4 with the checked/sampled distinction. In §4.4 state that the ordinary-debug numbers exclude the sampled sortedness checks, and add the test-build observation (7.7x plain Tide debug / 9.2x Rust debug at 200,000), explicitly retaining that it was outside a quiet window, that it also includes harness/cfg differences, and that scanning on every insertion is unrepresentative of typical contract cost. No new measurement is needed.

### B2. The formatter is specified, not delivered

`paper/tide-v0.typ:46`.

“Its formatter is deterministic with no options” presents an existing tool alongside implemented restrictions. D14 specifies that formatter, but `design/reference.md:1060` labels the toolchain loop, including `tide fmt` at line 1071, planned. The landed delivery audit (`reviews/fx-p44q-audit.md:35`) likewise marks it absent. The paper's later statement that the `tide` command is planned does not make this local present-tense feature claim accurate.

**Minimum fix:** say “D14 specifies a deterministic formatter with no options; it is planned.”

### B3. Manifest recording needs the driver-test boundary

`paper/tide-v0.typ:54`.

“A run under the simulator writes a replay manifest” is broader than delivered v0 behavior. `docs/owner-guide/06-replay.md:186` limits replay to test runs started through the driver; lines 190–191 explicitly say a directly executed test binary records no manifest and gets no identity check. `crates/tide-std/src/sim/manifest.rs:26` and `:71` create a manifest only when the driver context exists. Using `Root::sim(seed)` alone is insufficient. Line 74 correctly gives the narrower boundary, but §2.3 independently promises more.

**Minimum fix:** change this to “A simulated test run started through the driver writes a replay manifest …”, retaining the identity check. Optionally add that directly running the test binary records none.

### B4. E1's raw logs are not published in the repository

`paper/tide-v0.typ:185`.

The assurance “trials whose raw records are in the repository” does not preserve E1's evidence boundary. `trials/e1-checked/gate-report.md:83` describes the published snapshots, grades and derived usage records; lines 333–339 explicitly keep raw writer/broker streams, stderr and grading-attempt logs outside the repository and say the derived usage files are not the original raw logs. A reader cannot recover the complete raw writer evidence from the checkout.

**Minimum fix:** say the trials publish reports and supporting records, and explicitly note that E1's raw writer/broker streams and grading logs remain outside the repository. Keep the claim about reproducible measures separate from availability of raw sessions.

### B5. Preserve the governance exceptions and permitted review-record locations

`paper/tide-v0.typ:181`, `paper/tide-v0.typ:183` (also the short disclosure at line 22).

The requested governance description must include the production-code exception: `GOVERNANCE.md:27` permits human-written production code only when explicitly marked as an exception. Quoting the default rule without its exception, followed by the categorical “Changes are agent-written” at line 185, omits that qualification. Also, “Each review is recorded as a file under reviews/” is not the governance rule: `GOVERNANCE.md:75` permits a PR discussion, requiring a file for a direct commit.

The paper correctly mentions Opus stand-ins and refers readers to other merge exceptions. For an exact summary, those exceptions include planning-only filings and qualifying source-checked Starcompass regeneration (`GOVERNANCE.md:49`–`50`); the human maintainer remains final merge authority (`:51`). The short disclosure should not read as an exception-free Codex-plus-Opus requirement.

**Minimum fix:** add the explicitly marked human-code exception; say reviews are recorded in PR discussions or, for direct commits, in `reviews/`; qualify the normal sequential review description by the governance exceptions, naming the regeneration carve-out as well as planning-only work, and retain final human merge authority. These are descriptions of existing policy, not proposed policy changes.

## Non-blocking findings

### N1. Relative PDF citation links will need a publication base

`paper/tide-v0.typ:191` (and the other repository-path links).

All 56 local link targets exist when interpreted from the repository root. The PDF URIs themselves are relative, however: from `target/work/tide-v0.pdf`, `design/01-design.md` does not locate the root's design file, and a hosted PDF needs an appropriate publication layout or absolute targets. The builder explicitly records this limitation. It does not block this unposted draft.

**Minimum fix:** when preparing the posted artifact, resolve these citations through the chosen public repository/site URLs or a matching hosted layout.

### N2. The census percentage is rounded upward in the source's prose

`paper/tide-v0.typ:38`.

77% matches the cited review's opening prose and `design/01-design.md:104`; the underlying table in `reviews/round1-agent-usability.md:115` gives 76.8%. This is inherited rounding, not a miscopy or an unsupported result. Flagged because the brief specifically asks about upward rounding.

**Minimum fix:** use 76.8% if the paper should preserve the table's precision.

## Reasoning

- **A1 passes.** Replay and admission lead. A direct string comparison confirms that line 100 quotes E3's first paragraph verbatim. The 66/72 versus 67/72 result, interval, missed threshold, ceiling equation, eleven tied pairs, one missed plant, three false positives and small-session limitations match the landed report. The trust diff is presented as a summary, with no asserted catching benefit.
- **E1 passes the requested framing and numerical checks.** The exact fractions and report-precision values are preserved: F = 56/59 = 0.949153, P = 1, I = 86/59 = 1.457627. The denominators, thresholds, uncertainty, reported median times and failure classifications match the gate report. Parity, near-ceiling performance and the single `claude-opus-5-5` writer family are explicit. The paper does not manufacture a learnability benefit.
- **Benchmark values quoted in the paper match the landed note.** There is no speedup claim. Instability, shared-machine scope, paired-round interpretation, differing linkers/std builds and the separate run's lack of a quiet window are carried over. B1 concerns the omitted checked-versus-sampled boundary and the associated measured loss, not a miscopied ratio. Summary numbers otherwise retain the landed note's own precision.
- **Case studies pass.** Each negative comes first. Neither Eon's serial loop nor either hyper report is claimed as an admission catch. The quantitative details match their READMEs; the paper distinguishes rewritten models from the original projects and credits tests someone had to devise.
- **D2 is accurate.** The first route is the function-pointer form of #25860 (`docs/owner-guide/03-soundness-bugs.md:29`–`41`). The red-team result identifies two forms of #57893 (`reviews/2026-11-17-red-team-opus.md:10`–`17`, routes 6 and 6b). The Opus r2 re-check confirms the projection form was admitted at 6a30b7c and then rejected after widening the check. Counting these as three admitted routes across two bugs is supported and avoids inventing a third rustc issue. The paper retains the other-soundness-bugs limitation.
- **Positioning and main limitations pass.** Rust is complementary, with an unmodified rustc rather than a fork. Linux/x86_64, one crate, modeled replay, rustc soundness, partial sandbox availability, direct-binary/build gaps, E1/E3 limitations and the planned roadmap are explicit. Anthropic/Theorem are context only, independent of Tide; no formal-verification or endorsement claim appears. The paper is labelled a draft, with no implication that posting has happened. B2/B3 are the remaining local delivery overstatements.
- **Typst/PDF check passes.** PATH has Typst 0.15.1, so I also used the already-installed pinned binary `/nix/store/g6lmqa0x1ia8n13ja6snp804k79bvzx0-typst-0.15.0/bin/typst`, which reports `typst 0.15.0 (3ae52774)`. `timeout 60 <pinned-typst> compile paper/tide-v0.typ /tmp/tide-v0-r1-codex-pinned.pdf </dev/null` succeeded without Typst warnings. The pin in the source matches `flake.lock`. Both the supplied PDF and the fresh pinned build have nine A4 pages; I visually inspected all nine rendered pages of the fresh build, including the E1 table and E3 quote. No clipping or overlapping text observed. Poppler's `pdfinfo` emits `Syntax Error: Suspects object is wrong type (boolean)` on both PDFs, but reports their metadata/page counts and renders all pages successfully; this is not a Typst compilation failure.
- **Identity/structural checks pass.** `git diff --check d4a91a2..702cac8` exits 0. The final HEAD and diff digest remain the recorded identities. The built artifact is ignored; no PDF is part of the candidate diff.

Not checked: Rust builds, trial reruns, benchmarks, live soundness/sandbox probes, or independent verification of external projects' current web pages. This is a documentation-to-landed-evidence review, not new experimental or security proof. The later Opus round and the owner's posting decision remain outstanding.

DONE
