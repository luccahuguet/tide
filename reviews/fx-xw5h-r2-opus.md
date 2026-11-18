Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate tide-site 1e7dbfe

## Verdict

**request_changes**, confidence high (about 0.85). Two text-only blocking findings, both on the
home's benchmark section. Each fix is a few words, and the reviewer who found them (this one) needs
to re-check only those lines. Everything else in the candidate holds up against its sources, and
the code, tests and generators are sound.

## Scope

- Bead fx-xw5h: the body, all comments, and fx-qlek's launch-bar comment of 2026-11-18 plus the
  later standing decisions. Read Tide's AGENTS.md first. GOVERNANCE.md's Merge Rule is restated in
  AGENTS.md and in the bead.
- Candidate: `/home/lucca/pjs/lab/tide-site` at `1e7dbfe66b338bb2e0c576574fc650d3422b8304`, clean.
  The whole diff `9f099a0..1e7dbfe` was reviewed. Its SHA-256 matches the brief:
  `43970c400ee3e502987cfa2a5ef98ba49ed49b52c34b4014139dda5db24339f5`.
- Claims checked against Tide `d835458` with `git show`:
  - E3: `trials/e3-pilot/report.md` and `result.json`;
  - E1: `trials/e1-checked/gate-report.md`;
  - benchmarks: `research/36-benchmarks.md` and its records;
  - `demo/README.md`, the red-team record and the three case-study READMEs;
  - `docs/install.md` and `docs/docker.md`;
  - owner's guide chapters 01–08 and `answers.md`;
  - `README.md`, the paper source, `flake.nix`, `rust-toolchain.toml`, and `crates/tide-admit/src/invocation.rs` (the linker fallback).
- Commands run in the scratch clone `/mnt/build/tide-agents/fx-xw5h-r2-target/site`, all through
  heavy.sh, with load below 12 each time:
  - `nix develop … bun install --frozen-lockfile; bun run build`: exit 0, 4 pages plus
    `llms.txt` and `licenses.txt`.
  - `bun run check:typos; bun run check; bun run test:e2e`: exit 0. Astro check gave 0 errors,
    0 warnings and 0 hints. Playwright passed 20 of 20 at 1440×900 and 320×568.
  - `scripts/build-paper.sh /home/lucca/pjs/lab/tide`: the PDF is byte-identical, SHA-256
    `bc603edb…8fd5461`, as NOTICE states.
  - `bun scripts/chart-data.ts /home/lucca/pjs/lab/tide`: `charts.json` is byte-identical, and
    all three input hashes match.
- The clip:
  - ffprobe shows video only (no audio stream), 1280×720, 47.2 s.
  - Frames were extracted every 3 s and looked at. The poster equals the final frame.
  - Asset hashes and sizes match NOTICE.
- Looked at the full-page screenshots at both widths: home, how-it-works, install and questions.
- Leak scan of `dist/`, the PDF strings and the MP4 strings: no `/home/…`, `/mnt/…` or `.cache`
  path, no email, no internal hostname, no bead ID. The only personal string is the owner's
  public GitHub handle in the starter's copyright notice inside `licenses.txt`, which is intended
  attribution.
- Read r1-codex.md and r1-recheck-codex.md only after forming this view. Codex's B1–B4 are fixed
  in this tree: the recorder fails closed and validates its arguments, `licenses.txt` ships
  d3-selection's ISC notice, and the Shared wording is fixed. None of the findings below repeats
  theirs.

## Blocking findings

### B1 — The home attributes the cold-build cost to "no incremental build", which the note does not say

**Where:** `src/pages/index.astro:193`: "a cold debug build takes 3.8 to 4.6 times plain Rust's,
since the driver has no incremental build".

**Evidence:**
- `research/36-benchmarks.md` §1.1 ties "the driver rebuilds from scratch" to a different row:
  "Rebuild after a one-line edit … 5.2-6.3x … the driver rebuilds from scratch". Its closing line
  says the same: "the edit loop (the driver has no incremental build)".
- A cold build is from scratch for rustc as well, so missing incremental compilation cannot
  explain the cold-build ratio.
- For cold builds, §1.2 lists other contributors and leaves them unseparated: "admission, the
  driver's re-executed compile child, its cache writes, Tide's std's metadata, and a GNU ld link
  … no configuration separates these".
- The note also says "Why a number is what it is was not measured."

The site therefore gives a cause the source does not support. A Rust reader will spot this at
once, because cold builds are never incremental.

**Minimum fix:** delete the "since …" clause. To keep the edit-loop point, state the row it belongs
to instead: "a rebuild after a one-line edit takes 5.2 to 6.3 times, since the driver rebuilds from
scratch".

### B2 — The benchmark chart's title says "plain Rust", but the release panel's baseline is Rust with Tide's flags, and the page's own prose gives a different `tasks` number

**Where:** `src/components/BenchChart.astro:23`: "Benchmarks: Tide's time as a multiple of plain
Rust's". The first panel compares `tide-release` with `rust-release-tide-flags`, built by
`scripts/chart-data.ts`.

**Evidence:**
- The prose directly above the chart, `index.astro:194`, says "the `tasks` program runs 2.29 times
  plain Rust's release build". That is `rust-release`, as in note §1.1, "Tasks runtime".
- The chart, under a title naming plain Rust, shows `tasks` at "1.31x loss". That is against the
  flag-matched twin, after the flags' own 1.74x has been factored out.
- So on one screen the page uses "plain Rust" for two different baselines, and the chart's version
  is the smaller one, which flatters Tide.
- The panel label "against Rust with Tide's flags" is accurate. The figure title is not.

**Minimum fix:** make the title match both panels, for example "Benchmarks: Tide's time as a
multiple of Rust's". Optionally add one line under the release panel: "Against plain
`rust-release`, `tasks` is 2.29x; Tide's flags alone cost 1.74x (note §1.1)."

## Non-blocking findings

**N1. Three of the four try-it paths cannot work at d835458.** They are handed to fx-qlek.7, but
the handoff should name the missing pieces.
- `src/site.ts:38` gives `nix run github:<repo>#tide-demo`. The flake at `d835458` defines no
  `tide-demo` app or package: only `tidelang-admit`, `tidelang-std` and `default`. The command
  fails even after the placeholder is filled.
- Codespaces needs a `.devcontainer`, and none exists at `d835458`.
- The rustup path says "No Nix needed". It is plausible: `tide-demo` supports rustup, and without
  `TIDE_LINKER_BIN` the driver falls back to the host's `/usr/bin` C toolchain. But it is
  unverified, and it silently needs a system `cc`/`ld`.

The site cannot ship before fx-qlek.7 fills the repository placeholder. The `site.ts` comment
assigns the check to fx-qlek.7, and fx-qlek.7's bead lists the devcontainer and the rustup check.
Neither names a flake `tide-demo` app. Recommendation: record on fx-qlek.7 that the flake needs an
`apps.tide-demo` (or the site's command must change), and that the rustup line should mention the
host C toolchain if verification confirms the need.

**N2. Clip beat (a) is honest but not the strongest rejection available.**
- In the footage, rustc's own `E0433` and `E0599` print before `TD0501` and `TD0602`. The caption
  admits this.
- A skeptic will read it as "rustc rejected it; Tide's std just lacks those functions", so the
  beat shows little that admission adds.
- The launch bar asks for "the strongest honest admission rejection" if fx-zu4f.5 finds no
  external case.
- A clearer beat already exists: `conformance/cases/memory/closure_fnptr_capability.rs`. Plain
  rustc accepts it, through the rust#25860 hole, and `tide-admit` rejects it with `TD0206`. It
  ties straight to the A1 headline (three soundness routes closed).
- `record.sh` already takes another file and `lib`. Recommend this swap in fx-qlek.7's rerun, with
  the caption and description updated.

**N3. The chart text is unreadable on phones.**
- The SVGs use a 360-unit viewBox. At 320 px the viewBox scales to about 288 px, so the 10-unit
  `.note` and `.axis` text renders at about 5–8 CSS px: labels, ticks and the in-panel hedge line.
- The mobile rule `.chart svg { font-size: 12px }` (`site.css:142`) does not reach text that
  `site.css:112` sizes explicitly.
- The numbers stay readable in the tables, so this is not an accessibility failure. It is a weak
  first impression on a phone.
- Also, at desktop width the E3 chart's "out of reach" label (`E3Chart.astro:56`) is crossed by
  the dashed 1.5x line.

**N4. The FAQ sends readers to a stale page.** `questions.astro:16` links "the longer versions" to
`docs/owner-guide/answers.md` at `d835458`. That file's item 7 still says "E3 has no data yet: the
plants are being written, then the reviews run". The site's own answer is correct. The fix belongs
in the Tide repository: a guide update before the RC, which suits fx-qlek.7's pre-RC fixes. It is
out of the site's scope.

**N5. Small wording points.**
- `how-it-works.astro:205`: "run seeds 0 to 15 and you have tried 16 interleavings". Distinct
  seeds can produce the same schedule. "16 seeded schedules" would be exact. The wording comes from
  the guide.
- The home's limit paragraph speaks of "a malicious dependency", while how-it-works says "there are
  no dependencies yet". "a malicious crate or, later, dependency" would reconcile them.

**N6. Two small code points.**
- `record-clip.rs:29`: `grep -m1` can end the driver with SIGPIPE (141), and `driver_failed`
  accepts that the same as a test failure. The replay beat still proves reproduction, so this is
  harmless.
- `tsconfig.json:4` excludes `scripts/`, so `bun run check` does not type-check `chart-data.ts`.

**N7. Known and already assigned.** The PDF's repository links are relative until
`githubRepository` is set and the paper is rebuilt (fx-qlek.7, as the site README says).

## Reasoning

**The A1 framing holds.**
- The home leads with replay, then admission, under the heading "No accidental ambient authority."
- No page, nor `llms.txt`, says "unforgeable", and an e2e test guards that.
- The trust diff appears only as "a summary of what a change may do", with its blind spots listed
  and E3's null result stated beside it.
- Both E3 paragraphs are quoted verbatim, including the protocol §10.5 ceiling text, and the chart
  draws that ceiling (out of reach above 72/67 = 1.07x).
- E1 is framed as "Parity with Rust, not a benefit". Its numbers match the gate report exactly:
  56/59, 58/58, 86/59, 0.949153, 1.457627, and the interval 1.285714–1.637931.
- No speedup is claimed anywhere.
- The benchmark section quotes "Every comparison above is a loss for Tide or a tie. None is a win."
- The hedge is visible twice: in red in the figure caption ("0 of 60 series … CVs from 1.04% to
  46.7%") and inside each panel.
- Losses are labelled losses, and contracts is a tie. The slower-round counts were recomputed from
  the records: 11, 26, 30 and 30, then 30 for every cold-build row. They match the note's 20-of-30
  rule.
- The case studies open with their negatives ("admission catches none of them"), and each summary
  matches its README's first paragraph.

**Every other number checks out against its source.**
- Demo: 8 of 16 seeds (0, 2, 4, 8, 9, 10, 12, 14), 40 events, identical hashes, the fixed revision
  16 of 16.
- Red team: 48 routes including controls, two left admitted on purpose (routes 22 and 24).
- Soundness: three routes closed from two rustc issues, `TD0206` and `TD0402`.
- Install: disk sizes, the Ubuntu 24.04 image, example output, sandbox lines, "Linux only
  observed".
- FAQ: 17 rustc libraries, 389 conformance cases, no CI.
- Every `repoFile` path exists at `d835458`. The one non-file path is the demo's generated
  `target/tide-demo/out/...`, which the guide uses too.

The two blocking findings are in prose and a title around the benchmarks. That is exactly where an
HN reader probes first. Both are false or self-contradictory as written, and both lean in Tide's
favour.

**The try-it block** follows the decided order: Codespaces, rustup, `nix run`, then Docker as the
macOS/Windows fallback. One placeholder drives every repository reference, with `.invalid` names
that cannot resolve. It makes no time or size claim, and an e2e regex guards that. The commands'
present non-functionality is N1. It does not block, because publication is impossible until
fx-qlek.7 fills the placeholder and verifies each path, as the bead's 06:00 comment assigns.

**The clip is real and fails closed.**
- The footage matches its caption word for word: the four diagnostics, `echo $?` printing 1, the
  eight failing seeds, and two `replay reproduced … seed=0` lines.
- The recorder now proves the filmed run, not the rehearsal:
  - it reads each prompt's `PIPESTATUS`;
  - it scans the typescript segment for expected output;
  - it applies bounded timeouts;
  - it validates its arguments against injection;
  - `record.sh` installs only after the recorder succeeds.
- bubblewrap with a private `/home/tide` keeps host paths off screen, as the frames confirm.
- Under reduced motion the video stays on its final-frame poster and plays only when chosen. That
  is tested at both widths. With autoplay, the visible `controls` satisfy WCAG 2.2.2's pause
  requirement.

**The code is sound.**
- `chart-data.ts` reads only pinned blobs, records their hashes, refuses schema drift, and checks
  that the note still states the 20-of-30 rule. Its derived values are exactly the note's.
- `build-paper.sh` is byte-reproducible.
- The `licenses.txt` route builds its notices from their sources.
- The tests cover:
  - metadata;
  - assets;
  - the try-it order;
  - no "unforgeable";
  - keyboard paths;
  - layout without JavaScript;
  - clip behaviour;
  - chart numbers against `charts.json`;
  - readout focus;
  - Axe at both widths.
- The D3 dependency gate is recorded in the builder's report. Only d3-selection ships to browsers,
  as a 12.5 KB chunk; d3-scale runs at build time only, and the bundle was confirmed to contain no
  scale code.
- No leftovers: the starter's `sample.mp4` and `preview.png` are removed, and nothing references
  them.

**Re-check scope:** only `index.astro:193–194` and `BenchChart.astro:23`, plus any line the
optional addition under B2 touches. No rebuild is required for review, though e2e will run anyway
at landing.

DONE
