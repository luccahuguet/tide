# The paper's figures

`paper/tide-v0.typ` embeds the files in `svg/` with Typst's `image()` and sets
the files in `excerpts/` as raw blocks. `build.mjs` generates both, and the
derived numbers in `data/`, from committed records only. Every number in a
figure is read from a committed record or computed from one. The diagrams'
text and the figures' labels are written in `spec/`, each diagram cell with a
quote that `--check` finds in its source. The site may reuse the SVGs.

```sh
cd paper/figures && npm ci --ignore-scripts && cd ../..
node paper/figures/build.mjs           # regenerate data/, excerpts/ and svg/
node paper/figures/build.mjs --check   # exit 1 if a committed file drifted
```

`--check` regenerates every file in memory and fails when one differs from the
committed copy, is missing, or is no longer generated. It also fails when a
quote in `spec/` no longer occurs in its source, and when
`design/reference.md` changed since its sections were counted: then rerun
`node paper/figures/count-reference.mjs` (it calls `scripts/count-tokens.sh`,
so it needs a logged-in `claude` CLI) and rebuild.

## Sources

| Figure | Generated file | Read from |
| --- | --- | --- |
| fig-code | `excerpts/code-rust.rs`, `code-tide.rs` | `pub fn load` in `examples/capabilities/ambient_read.rs` and `config.rs`; verdicts from `records/admit-*.txt` |
| fig-budget | `svg/budget.svg`, `data/budget.json` | `data/reference-sections.json` (per-section counts, with the reference's blob hash); the whole count from `design/reference-companion.md` ("Core count"); the budget from `design/reference.md` ("Budget") |
| fig-diagnostic | `excerpts/diagnostic.txt` | `records/admit-ambient_read.txt` |
| fig-trust-diff | `excerpts/trust-diff.txt` | step 4 of `records/demo-run.txt` |
| fig-pipeline | `svg/pipeline.svg` | `spec/diagrams.json` (`pipeline`), quotes checked in `paper/tide-v0.typ` |
| fig-replay | `svg/replay.svg`, `data/replay.json` | steps 1 to 3 of `records/demo-run.txt` |
| fig-manifest | `excerpts/manifest.txt` | `records/demo-seed0.manifest.json`: a subset of its fields; hashes, the compiler line and the outcome cut, the 36 dependency entries and the 40 trace events elided, each cut marked `…`; a last `"…"` entry in `build` and at the top level names the recorded fields left out |
| fig-redteams | `svg/redteams.svg`, `data/redteams.json` | counted by the rules in `spec/redteams.json` from the six red-team records it names |
| fig-search | `svg/search.svg`, `data/search.json` | selectors in `spec/search.json`, one row per round; every count derived from the round's record; on every build a falsifier changes research/47's "Nothing was caught." to "One was caught." in memory and requires fx-zu4f.8's derived catch to become 1 |
| fig-e3 | `svg/e3.svg`, `data/e3.json` | `trials/e3-pilot/result.json` |
| fig-bench | `svg/bench.svg`, `data/bench.json` | `research/36-benchmarks/2026-11-18-bf6b958/records.jsonl`, with the note's rule (a loss is Tide slower in at least 20 of the 30 paired rounds) |
| fig-related | `svg/related.svg` | `spec/diagrams.json` (`related`), each cell with a quote checked in `paper/tide-v0.typ` |
| fig-sim-model | `svg/sim-model.svg` | `spec/diagrams.json` (`sim_model`), each cell with a quote checked in README.md's "Simulator fidelity" table |
| fig-tiers | `svg/tiers.svg` | `spec/diagrams.json` (`tiers`), each item with a quote checked in its row of the tier table in `design/01-design.md` |
| fig-merge | `svg/merge.svg` | `spec/diagrams.json` (`merge`), each step with a quote checked in `GOVERNANCE.md` |

`records/` holds tool output captured at commit
`a913f46ead5d5bd4d432a82f554fedb4809b2e83`; each `.txt` file's header names
the commit, the command and the one normalization applied (absolute paths).
To refresh them, rerun those commands, replace the records, and rebuild.

## Style and dependency

Every SVG is 459 units wide, the paper's text width in points, and uses
Libertinus Serif and DejaVu Sans Mono, which Typst bundles and embeds in the
PDF. Colours are from the Okabe-Ito palette, which is colourblind-safe, at
distinct grey levels, and every mark is also labelled, so the figures read in
grayscale print.

`d3-scale` 4.0.2 (ISC) provides the scales; the SVG is written as text, with
no DOM. `package.json` pins it, `overrides` pins its six d3 dependencies to
their long-standing releases (not the batch of patch releases published on
2026-11-03), and `package-lock.json` pins all eight packages (about 1.2 MB, no
install scripts). Replacing it costs about 30 lines: linear and log scales.
Rejected: the full d3 bundle with a DOM shim (jsdom or linkedom), a large tree
for output that needs no DOM; hand-written scales, viable but against the
owner's direction to draw the figures with D3; and Typst-native drawing, for
the same reason, since the figures are uniform D3 SVGs the site also reuses.
