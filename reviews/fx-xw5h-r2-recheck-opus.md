Model: claude-opus-5-5 · Date: 2026-11-18 · Re-check tide-site a534c99

## Verdict

**approve**, confidence high. B1 and B2 are fixed. The N2 beat is honest, keeps to "one known
soundness route closed", and its recorder fails closed. N3's charts read well at 320 px and at
desktop width. The round introduced no new blocking defect.

## What I checked

- The round diff `git -C /home/lucca/pjs/lab/tide-site diff 1e7dbfe..a534c99`: 17 files, one
  commit. The site repository was still clean at `a534c99`.
- The builder's "Round r2 fixes" section, read as claims to check.
- Tide sources at `d835458`:
  - `conformance/cases/memory/closure_fnptr_capability.rs`;
  - `flake.nix`, for `tidelang-std`'s `RUSTC` and `lib/deps`;
  - `research/36-benchmarks.md` §1.1.
- Scratch clone `/mnt/build/tide-agents/fx-xw5h-r2-target/site` at `a534c99`. Every run went
  through heavy.sh with load below 12, and the clone's tree was clean afterwards.
  - `bun install --frozen-lockfile; bun scripts/chart-data.ts …; git diff --exit-code -- src/data/charts.json`:
    `charts.json` is byte-identical, including the new `tasksAgainstPlain` (2.2868, 1.7404).
  - `bun run check:typos; bun run check; bun run test:e2e`: exit 0. Astro check gave 0 errors and
    0 warnings, and **22 of 22** e2e tests passed at 1440×900 and 320×568.
  - `recordings/check-fail-closed.sh /home/lucca/pjs/lab/tide`: exit 0, with six `ok` lines.
    - The four argument cases are refused before anything runs.
    - The filmed-timeout case fails "within 1 ms" only after the off-camera rehearsal has passed.
      That rehearsal independently shows plain rustc accepting the new beat, `tide-admit`
      rejecting it with TD0206, and the search and both replays reproducing.
    - The published clip and still are unchanged.
- The new clip:
  - It has no audio stream, is 1280×720 and lasts 55.6 s. Its SHA-256 `449394e1…` matches NOTICE.
  - I extracted a frame every 2 s and looked at them.
  - Its final frame shows the same terminal content as the unchanged still `clip.png`, which
    keeps the hash `bf6686af…`.
- `dist/` after the rebuild: no private paths, emails or hostnames. The only personal string is
  the owner's public handle in the starter's copyright notice in `licenses.txt`, as before.
- No leftovers: nothing references `uptime.rs`, the old caption, "47 seconds" or "TD0501 and
  TD0602".

## Items

**B1, fixed.** `index.astro:190–195`:
- The cold-build sentence no longer gives a cause.
- The edit-loop point now has the row it belongs to: "A rebuild after a one-line edit takes 5.2 to
  6.3 times Rust's incremental rebuild, since the driver rebuilds from scratch."
- That matches note §1.1 ("`tide-debug` takes 5.2-6.3x incremental `rust-debug`'s time … the
  driver rebuilds from scratch").

**B2, fixed.** `BenchChart.astro`:
- The title is now "Tide's time as a multiple of Rust's", and each panel's label names its own
  baseline.
- Under the release panel: "Against plain `rust-release`, tasks is 2.29x. Tide's flags alone cost
  1.74x (the note, §1.1)."
- Both numbers are generated from the run's medians (1.270912/0.555… and 0.967224/0.555…).
- `chart-data.ts` fails unless the note still contains its own sentence "`tide-release` 2.29x
  `rust-release` on `tasks`: 1.74x from the flags". That is a good guard against drift.
- An e2e test asserts both figures against `charts.json`.
- The chart and the prose above it no longer disagree.

**N2, honest, and it fails closed.**
- **What the footage shows** (frames at about 18 s and 22 s):
  1. `cat` shows the conformance case verbatim, with its own comment ("a borrowed `Fs` comes back
     as `&'static Fs` … rustc accepts it") and its `//~ TD0206` mark.
  2. `rustc --edition 2024 --crate-type=lib --emit=metadata --extern std=$TIDE_STD -L dependency=$TIDE_DEPS …`
     prints nothing, and `echo $?` prints 0.
  3. `tide-admit --crate-type=lib …` prints exactly one error, `TD0206: … (CLOSURE-3)` at 11:48,
     and `echo $?` prints 1.
- **The comparison is fair.**
  - The plain rustc comes from the flake's sysroot, `tidelang-std.RUSTC`, the toolchain the driver
    is built on.
  - `--extern std=` points at `tidelang-std`'s `lib/deps`, the std the driver loads by default.
  - rustc must have resolved `std::fs::Fs` from Tide's std: no other std has it.
  - So the only difference between the two runs is admission.
- **Fail-closed on the filmed run** (`record-clip.rs:347–351`), checked by reading the code and by
  the check above:
  - rustc's statuses must all be 0, and the next `echo $?` must print 0;
  - admission must exit exactly 1 and print every `//~ TDnnnn` code in the file, here
    `error: TD0206`, and then `echo $?` must print 1;
  - the rehearsal enforces the same conditions before filming.
- **The claim is kept narrow.**
  - The description says "That closes one known soundness route, not every one."
  - `llms.txt` says "one known soundness route closed".
  - The video's title is "rustc accepts a soundness hole that admission rejects". It names one
    hole and does not generalise.
  - The home's admission section still says three routes are closed. That is consistent: the clip
    shows one of the three.
- **`record.sh`**: the new optional-argument array is correct under `set -euo pipefail`. A false
  `[ … ] &&` is not the script's last command, so it does not trip `set -e`.

**N3, readable at both widths.**
- Both charts use a 300-unit viewBox with 12–13-unit text, capped at 24rem.
- At 320 px the arm labels, ticks, panel titles, the in-panel hedge, the "loss"/"tie" labels and
  the 0.99 point label all read at about 11–12 px.
- At 1440 px they read at about 17 px. E3's "out of reach" label now sits inside the band, left of
  the dashed 1.5x line.
- The new e2e test measures every chart text's rendered size (font size × screen scale ≥ 11 px)
  and checks that the label clears the line, at both widths.

**N5, fixed.** "16 seeded schedules" (`how-it-works.astro:205`), and "Against a malicious crate
or, later, dependency" (`index.astro:105`).

**N6, accepted as rebutted.** Type-checking `chart-data.ts` would need a new types dependency.
Byte-identical regeneration exercises the script instead. The SIGPIPE point was already judged
harmless.

**N1, N4, N7:** with fx-qlek.7, as before. Not re-checked.

## New defects

None blocking. Two nits, neither needs a re-check:
- The benchmark axes now show about three ticks: 0x, 0.5x and 1x on the release panel; 0x, 2x and
  4x on the cold-build panel. So the largest points (1.31x, 4.60x) sit past the last labelled
  tick. Their exact values are printed beside them, so nothing is lost. `ticks(4)`, or a tick at
  the domain's end, would tidy it.
- The clip description says plain rustc "compiles" the case. The command is `--emit=metadata`,
  which type- and borrow-checks without generating code. "accepts" (the title's word) would be
  exact. The command is on screen, so nothing is hidden.

DONE
