Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate ba40864 (code+docs)

## Verdict

approve

No blocking findings. Confidence: high for this candidate's additions, their agreement with landed evidence, and the reproducibility and layout checks below. No override. The subsequent Opus round and the owner's approval before posting remain applicable.

## Scope

Read-only supervisor pass for the reopened fx-qlek.4. Read `AGENTS.md` and `GOVERNANCE.md` first, then `br --no-db show fx-qlek.4`, `br --no-db comments fx-qlek.4`, and the parent fx-qlek comments, including A1 and the 2026-11-18 launch bar. Reviewed the complete paper and `git diff 27b394e..ba40864`, the Typst calculations and drawing code, all four earlier `reviews/fx-qlek.4-*.md` records, the builder's `target/work/report.md` as claims, and the relevant landed reports and documentation.

- Full HEAD: `ba4086419b1810e3211e9130bf09c890575e5982`.
- Branch: `fx-qlek.4`.
- Base: `27b394e39ca78bc9e3b722ac8a43fd6e79358b7b`.
- SHA-256 of the exact bytes from `git diff 27b394e..ba40864`: `0a3422eabb2c2c2de69ad059b88d280c20a4d5dc4b6fff03bf800bc8d2033627`.
- Only changed tracked file: `paper/tide-v0.typ` (+194, -6). The worktree was clean, and HEAD/diff digest were checked again before recording the review.

No candidate edits, commits, branch changes, or `.beads/` writes. This requested report is the only repository artifact written. Build/render scratch outputs are under `/mnt/build/tide-agents/fx-qlek.4-target/r3-codex/`.

## Blocking findings

None.

## Non-blocking findings

### N1. Publication still needs public citation URLs (existing limitation)

`paper/tide-v0.typ:251`, `paper/tide-v0.typ:323`, `paper/tide-v0.typ:378`.

The new figure captions retain the paper's repository-root-relative PDF links. Their repository targets exist, but those relative URI targets do not by themselves provide usable citations from a hosted PDF. This was already recorded by both earlier reviewers and on the bead; the launch audit owns publication preparation. It does not block this unposted draft.

**Minimum fix:** resolve repository citations to the chosen public repository/site URLs when preparing the posted artifact. No source change is required for this review's approval.

No new non-blocking defects found.

## Reasoning

### Framing and the unforgeable condition

`paper/tide-v0.typ:204` accurately identifies the conditional requirement: no soundness hole reachable by admitted code, through compiler fixes plus a pin audit or a machine-checked argument about the admitted subset. It explicitly limits a model proof by its correspondence with rustc. `br --no-db show fx-ip8d` confirms that the authority-calculus proof is deferred, post-alpha, and not a compiler proof; it contains no completed proof artifact. The paper claims none.

The only occurrences of “unforgeable” are in this conditional paragraph, which expressly declines the term for v0. The actual v0 claim remains “no accidental ambient authority,” consistent with `docs/owner-guide/03-soundness-bugs.md` and the parent's launch-bar wording. The surrounding text preserves three closed admitted routes from two rustc issues, other open soundness problems, and the malicious-dependency limitation. Replay and admission still lead; the trust diff remains a summary, with no catching-benefit claim.

### E3: independent recalculation

Read `trials/e3-pilot/report.md`, the result JSON, and protocol §10.3. An independent inline Python calculation counted retained plant sessions from `result.json`'s item/session records rather than trusting its aggregate gate or interval fields. Eleven pairs give (trust, plain) = (6, 6); the remaining pair gives (0, 1). Results:

- Trust: 66/72; plain: 67/72.
- Ratio: `66/67 = 0.9850746268656716`, displayed as 0.99.
- Bootstrap: independently implemented SplitMix64 seeded 20261117, drawing 12 pairs with replacement for each of 10,000 resamples; nearest ranks `ceil(0.025*n)` and `ceil(0.975*n)` give `[0.9473684210526315, 1.0]`, displayed as `[0.95, 1.00]`; zero undefined resamples.
- Gate: not met. Two-thirds ceiling: `2*72/3 = 48`; `1.5*67 > 72`.

These values match the stored aggregates and the geometry/text at `paper/tide-v0.typ:212`. The point and interval use the numeric fields; the interval is not inferred from the rounded label. The report's first paragraph is quoted byte-for-byte as text at line 210, checked by Python assertion. The section and figure caption explicitly call E3 a pre-registered negative pilot, preserve the ceiling explanation, and do not imply a benefit on harder cases.

### Benchmarks: independent recalculation

Read `research/36-benchmarks.md`, particularly §1 and §1.1, and independently parsed the main run's `records.jsonl` with Python. For each plotted arm, checked 30 unique paired rounds with identical round keys; recomputed medians from individual `wall_s` measurements and checked the stored medians within their recorded precision. Counted strict Tide-slower/Rust-slower comparisons separately; there are no equal measurements in these comparisons.

| Panel | Pair | Recomputed ratio of medians | Figure | Tide slower | Verdict |
| --- | --- | ---: | ---: | ---: | --- |
| Release run, matching flags | contracts | 0.9949890055 | 0.99× | 11/30 | tie |
| Release run, matching flags | files | 1.0400092503 | 1.04× | 26/30 | loss |
| Release run, matching flags | tasks | 1.3139789749 | 1.31× | 30/30 | loss |
| Release run, matching flags | tcp | 1.1771949211 | 1.18× | 30/30 | loss |
| Cold debug build | contracts | 4.6043487674 | 4.60× | 30/30 | loss |
| Cold debug build | files | 3.9253042066 | 3.93× | 30/30 | loss |
| Cold debug build | tasks | 3.8237598120 | 3.82× | 30/30 | loss |
| Cold debug build | tcp | 4.4341300909 | 4.43× | 30/30 | loss |

All eight plotted values and verdicts agree with the committed records and the note. The eight current comparisons satisfy the note's 20-of-30 rule; no row is a win. In particular, the sub-1× contracts point is explicitly labelled a tie. The caption explains the matching-profile comparator and the tasks difference: 1.31× between matching-flag twins versus 2.29× against ordinary release Rust, with the 1.74× flags cost retained in the body.

The JSONL contains 60 summaries, zero stable. Their recorded CV extrema are 1.0356% and 46.7203%, correctly displayed as 1.04% and 46.7%. The hedge “0 of 60” and “descriptive, not settled measurements” is inside Figure 3, above its panels. The larger test-build contracts loss and its separate-run/harness qualifications remain in the surrounding text; the figure does not displace those earlier corrections.

### Pipeline and case studies

The diagram at `paper/tide-v0.typ:172` matches the existing implementation explanation, `docs/driver.md`, owner-guide chapters 6 and 8, and the applicable admission/simulation/replay rules in `design/03-authority-concurrency.md`. Its path is explicitly a simulated test started through `--tide-run-test`: admission before artifacts, sandboxed child execution, seeded scheduling with time advancing only when every task is blocked, then identity-checked replay. The diagram reports full/partial availability, while the immediately preceding component description specifies “partial, not enforced.” Direct-binary execution gets neither sandbox nor manifest; the build remains outside RUN-2's planned sandbox. The paper retains same-machine and model-fidelity limitations and makes no production-replay promise.

The third case study at line 336 matches `examples/case-studies/eon-discovery-race/README.md`, landed as `ace1b4e42323206b2d37df5eef875677f05f97c4`: negative first, two ordinary discovery file operations, four modelled operations before the fix's extra read, Eon's pre-existing deterministic hook-based test, 13/100 buggy failures, 100/100 fixed passes, and 11/100 failures under reversed host ordering. It distinguishes the runner's trace-checked seed rerun from the driver's manifest-based replay and does not claim to verify Eon's host ordering. The gain is finding an interleaving without an insertion hook, conditional on a hand-written property and model.

The abstract, section introduction, closing synthesis, and references include the third study without stale “two studies/four reports” wording. There are three studies covering five bug reports: eon-8wh7h, eon-pimct, hyper#2595, hyper#2808, and eon-8ehhn. “Admission catches none” remains consistent with all three READMEs. The older studies' 500 ms, 14 terminals, 6992 ms, 6011 ms, 250 ms, 6 s, 3.78 s, and 20-seed statements still match their reports.

### Landed provenance and earlier fixes

Every added measurement has a landed report: E3's `report.md`/`result.json` (landed `f5a732a43f8611aedbdcf5879a3dd4e47e535ba6`), research/36 and its raw records (landed `09da0ffdaa23ccef3df1decc90abde6abc8b8e7b`), or the discovery case-study README (`ace1b4e`). They are all in base `27b394e`, not new builder-generated evidence. The surrounding text cites the reports, and figure captions link the exact datasets. Input SHA-256s checked:

- E3 result: `a1786c12f3556072a82f90dfeb7a6c9c6a971d005c2c04c0c16aec29243f2404`.
- Benchmark records: `6c7d946267e2d7110e48de0a6fc7b12969937534ce7adb8aa4e7066e5ed96c39`.
- Benchmark report: `47d4775fb203933acaa0835c368085c46d95bf32a6f987728b20d3f046deffe1`.

Earlier blocking fixes remain intact: checked versus sampled contracts and the test-build loss; planned formatter; driver-only manifest recording; E1's externally retained raw logs; governance exceptions and permitted review-record locations; Claude Fable disclosure; rr and its chaos-mode qualification; and tokio's paused-clock caveat. Earlier corrections to counts, control inclusion, limitations, terminology and reference callouts also remain. No regression found.

The package choice is a bounded technical implementation choice under the reopen's “technique is the agent's call”: the source uses Typst-native primitives and no package imports. The builder's preference for avoiding plotting dependencies does not change the required three regenerated figures. No package-version promise needs an unpinned download; the relied-on drawing implementation is pinned with Typst itself.

### Build reproducibility and visual proof

Environment: Linux `7.2.6-76070206-generic`, x86_64; the flake's nixpkgs revision is `567a49d1913ce81ac6e9582e3553dd90a955875f`. `nix shell` from that pin reports `typst 0.15.0 (3ae52774)`.

Ran the following twice, sequentially, with `<out>` equal to `a.pdf` and then `b.pdf` under `/mnt/build/tide-agents/fx-qlek.4-target/r3-codex/`. Each start was gated by `awk '{exit !($1 < 12)}' /proc/loadavg`; both passed. Both builds exited 0 with no Typst warnings:

```sh
bash ~/.cache/eon-agents/eon-overseer/heavy.sh timeout 900 \
  nix shell github:NixOS/nixpkgs/567a49d1913ce81ac6e9582e3553dd90a955875f#typst \
  -c typst compile --root . paper/tide-v0.typ <out> </dev/null
```

`cmp` exits 0. Both PDFs have SHA-256 `001992df19096b661881f7ac46a5ebef9f951e5c7b70dea15b7dbb89b5558384`, size 212,264 bytes. `pdfinfo` reports **11 A4 pages**, within the bead's approximate 8–12-page scope. `date: none` removes the build timestamp in the source.

Rendered with `pdftoppm` and visually inspected Figure 1 on page 4, Figure 2 on page 5, and Figure 3 on page 7 at 130 dpi. No overlaps, clipped labels, or missing plotted values. The E3 threshold, interval and negative verdict are legible; the benchmark hedge and loss/tie labels are visible in the figure. Also inspected the E1 table/contracts discussion on page 6 and the third case study on page 8. Poppler prints the previously recorded `Suspects object is wrong type (boolean)` metadata warning; it still reads and renders the PDF successfully. This is unchanged from the earlier reviewed paper.

`git diff --check 27b394e..ba40864` passes. All 62 literal repository-link occurrences have existing targets, as does the computed benchmark-data link.

Not checked: Rust builds, fresh experimental/benchmark runs, runtime sandbox or soundness probes, current external project pages, or a site-chart review. This pass verifies the paper against its landed evidence and freshly built PDF. Publication URLs remain unfinished; approval does not authorize posting or replace Opus review.

DONE
