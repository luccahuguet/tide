Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 5c0b382 (docs)

## Verdict

changes required

Two blocking documentation defects remain: the simulator table incorrectly excludes short writes, and the Cargo recipe assumes an unmentioned populated dependency cache. Confidence: high in the fidelity finding and medium-high in the recipe finding, which is based on static dependency inspection rather than a fresh-cache execution. No override recorded.

## Scope

Read-only supervisor review of fx-qlek.3, including its binding comments, under GOVERNANCE.md's Merge Rule. Read AGENTS.md and GOVERNANCE.md first, then the bead through `br --no-db show fx-qlek.3`, the full candidate diff, all eleven changed documents, the builder's claim table and decisions, the cited reports and red-team reviews, install/Docker close records, relevant workspace manifests, simulator implementation and existing conformance witnesses. The builder's report was treated as claims to verify.

- HEAD: `5c0b3823396531e6d947b73e03e71c9a77ef96cc`, branch `fx-qlek.3`.
- Base: `d4a91a2e49f4e98f01d107f950fceb929c2014fc`.
- SHA-256 of the bytes from `git diff d4a91a2..5c0b382`: `b1e93ab023ea386ec71fd0acbe6b000e5dcc9ec7096f625f70a85d1043a5493c`.
- Tracked working tree was clean at entry and remained clean before writing this ignored report. HEAD and diff hash were checked again at the end of inspection.
- Only this requested report is written. No commits, issue writes, compiler invocations, Nix runs, or changes to main or other worktrees.

## Blocking findings

### B1. The simulator does model short writes

**README.md:232.** The “It does not model” column includes “short writes”. That is false for the current TCP simulator. `crates/tide-std/src/net/sim.rs:383` returns the smaller of the input length and remaining directional capacity. The existing simulation witness at `conformance/cases/authority/tcp.rs:251` explicitly requires a 1,048,576-byte write to return only 65,536 bytes. `research/21-channels-tcp.md:216` also names capacity-induced short writes as part of the model.

This gives readers the wrong fidelity boundary: a single write that sends only part of its buffer is already modeled and checked. Exact real-kernel behavior and timing remain outside the model.

**Minimum fix:** remove “short writes or” from that cell, leaving the caveat about real kernel behavior such as RST timing. No implementation change or new test is needed.

### B2. The standalone Cargo recipe cannot fetch Tide std's dependencies

**docs/driver.md:144.** The recipe first runs `cargo build --locked -p tide-admit`, then runs `cargo build --offline --locked -p tide_std`. A newcomer with the pinned toolchain and an empty Cargo cache can complete the first build but fail the second because the std's dependencies have not been downloaded. The driver's only package dependency is `libc` (`crates/tide-admit/Cargo.toml:11`); it does not depend on `tide_std`. The latter separately needs `libm`, `rustix`, and the macro dependency tree (`crates/tide-std/Cargo.toml:11`, `crates/tide-macros/Cargo.toml:11`). Building only the driver does not populate those sources. Entering the documented Nix dev shell does not supply a populated Cargo registry cache either.

The recipe is introduced as the way to build the driver and std with the pinned toolchain, without a prerequisite workspace build or dependency fetch. A fresh clone on a machine with cached crates can pass, so the author's fresh-clone observation does not settle this failure case. This is an inferred command failure from the manifests and Cargo's selected-package build behavior; I did not execute it.

**Minimum fix:** remove `--offline` from the `tide_std` build on line 144. Keep `--locked` and the Cargo JSON artifact discovery. This lets that command fetch its own dependencies and preserves support for hashed output directories.

## Non-blocking findings

### N1. The candidate diff check is not clean

**docs/driver.md:564.** `git diff d4a91a2..5c0b382 --check` reports `new blank line at EOF`. The builder's “git diff --check: clean” claim does not describe the committed candidate diff: running that command without the revision range on an already committed clean worktree checks no candidate changes.

**Minimum fix:** delete the extra blank line at EOF and record the revision-range diff check. This is whitespace only and does not block approval.

## Reasoning

- **A1 framing passes.** Replay and admission lead. The capability paragraph distinguishes three forge routes from two rustc issues, qualifies the evidence, and retains the other soundness holes. The trust diff is described as a summary of declared authority, with no claim that it improves catching. E3's first two paragraphs match the published report verbatim after removing Markdown quote prefixes and normalizing line wrapping; the 66/72, 67/72, 0.99, [0.95, 1.00], 1.5x threshold, 11/12 ties and ceiling explanation all match. The model-only and small-change limits remain visible.
- **E1 passes.** Counts 56/60 versus 59/60, 58/60 each, and 86 versus 59 runs over 58 passing trials match the gate report. Ratios `0.949153`, `1`, `1.457627` and interval `1.285714`–`1.637931` retain the report's published precision. The README explicitly states parity within margins rather than benefit and carries the interval crossing 1.5.
- **Benchmarks pass the requested evidence check.** The stability quotation matches research/36. All summary numbers match its §1.1: 3.8–4.6x cold debug, 5.2–6.3x edit rebuild, 0.901/0.906 s and 11/30, 1.05x at 1,200,000, 1.19x at 200,000 and 30/30 outside the quiet window, files 26/30 with paired median 1.07x, TCP 1.18x and 30/30, and tasks 2.29x with 1.74x and 1.31x components. There is no claimed speedup. The future tens-of-milliseconds claim is labelled a target. Earlier trial figures in the Reading Guide also match their grading records (6/10, about 9/10, 10/10, about 3.5x annotations, 11/11).
- **Case studies pass.** Each negative is stated first. Admission is credited with catching neither Eon's loop nor hyper's reported failures. The model-test claims and hyper's 20 seeds match the case-study records. The demo's 8/16 failures and two identical replays match its published output.
- **Scope coverage passes apart from B1.** Alpha/x86_64 Linux status, evidence and install links, the fx-gbq4 name/tagline, the single-crate/no-dependencies restriction, absent packages/CLI/simulator faults/HTTP/Proc, planned tiers, fidelity table, and model-versus-program replay boundary are present. Host/container `full` observations and the absence of macOS/Windows observations match the install/Docker records. Builds are explicitly unsandboxed. Known holes include the red team's two intentional admissions, remaining rustc soundness issues, future core glob additions, metadata reads, loader behavior, unescaped output, unsafe grants and availability/side channels. The TD0402 over-rejection and fx-0514 destructor/child-panic limitation are stated.
- **Quickstart static equivalence passes.** Extracting the unindented `sh` blocks shows the README block is byte-identical to the concatenation of install steps 1–5; install step 6 alone is omitted. Reading `scripts/clean-install-check.sh` confirms it extracts all six committed install blocks in order, permits only the documented clone-URL substitution to a HEAD bundle, and checks output, a sandbox line and a passing test. Its setup differences are explicit in the script. I read the existing transcript: pinned Ubuntu image, Linux 7.2.6, Docker 29.10.0, revision `e338521e7cc20757b0187c0cfc5d9986de6c1978`, expected example output, `sandbox: full` with Landlock ABI 10, one passing test, and final PASS at 05:11:40Z. The README shell block is byte-identical between that revision and this candidate, and `git diff --quiet e338521 HEAD -- docs/install.md scripts` succeeds. The unreleased `<repository-url>` remains the explicit placeholder already used by the accepted install route; the check substitutes the bundle as documented.
- **Driver move passes fidelity review.** Comparing old README lines 87–158 plus 163–635 with the new file after its introduction shows only the reported opening-sentence adjustment, relative-link corrections, and recipe/caveat edits. The JSON filename discovery is retained and does not assume `target/debug/deps`; the std selection uses Cargo's un-hashed top-level library artifact. The chosen package and example paths exist. Both README and driver reference explain whole-workspace rebuilding after the demo and replay's dependency-artifact refusal. B2 is the remaining recipe defect.
- **Links pass static checks.** All local Markdown paths/fragments in the eleven changed files resolve. Repository searches for README.md anchors found no removed local target: Reading Guide remains. The three design citations to README Status still reach the retained Status section and its driver-reference link; design/03's sandbox-status citation still reaches the missing-parts summary. No broken design citation was found.
- **Required absence check passes.** `rg -n 'design phase' README.md` returns no matches (exit 1).

Not checked: fresh builds, clean Nix reruns, live sandbox probes, or external website availability. The recorded clean-check transcript supports the unchanged quickstart; it does not prove the separate Cargo recipe with an empty dependency cache. The two blocking findings need correction before this candidate is Codex-clean for the sequential Opus review.

DONE
