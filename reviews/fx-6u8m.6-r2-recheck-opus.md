Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 1de46fe (re-check of c8f9b20 findings)

## Verdict

**approve**. Every finding the author took is resolved, and none of the fixes broke anything.

Scope: I re-checked only the fixes in `git diff c8f9b20 1de46fe` (4 files, +88/−48), against `report-r2.md`'s map. I ran no builds. The worktree is clean at 1de46fe. Two non-blocking notes follow the findings.

## Findings

**1. A failed start leaves files behind: resolved.**
- `Bench::start` now runs these steps in order:
  1. checks that the output directory is empty;
  2. resolves the pinned rustc;
  3. resolves the stable rustc;
  4. builds both stds;
  5. only then creates the directory and `records.jsonl` (`pairs.rs:278-304`).
- A missing `rustup` or `stable` returns `Err` before anything exists under `--out`. A failed std build panics in `build_std` before that point too. A retry with the same `--out` is therefore no longer refused.
- The error now names the program (`cannot run rustup: …`) or the whole command (`{command:?} failed`).
- The README says `stable` is required and is checked before anything is created. That is accurate: the stable lookup comes before the std builds.

**2. `bench = false`: resolved.**
- What Cargo documents:
  - The target's `bench` field controls only whether `cargo bench` benchmarks the target *by default*.
  - `--benches` selects "all targets that have the `bench = true` manifest flag set". `--all-targets` is `--lib --bins --tests --benches --examples`, so it skips the target too.
  - An explicit `--bench NAME` names the target directly, whatever its `bench` flag says.
- So `cargo bench -p tide-admit --bench pairs -- …`, the README command, still selects it. Plain `cargo bench`, `--benches`, `--all-targets` and `cargo test --benches` no longer do.
- The author's checks agree: `--bench pairs --no-run` builds it, and plain `cargo bench -p tide-admit --no-run` lists only `src/main.rs`.
- The Cargo.toml comment and the README match this.

**3. Rust linker in the header: resolved.**
- New header fields: `pinned_rust_linker` and `stable_rust_linker`. They come from `--print target-spec-json` under `RUSTC_BOOTSTRAP=1 -Zunstable-options`, which only prints and touches no measurement.
- The flavor parse does not depend on spacing, because it splits on `"` after the key.
- For today's pins it reports `gnu-lld-cc, bundled linker on`, as the author's quick-r2 header shows. That matches the README's claim.
- A non-blocking note on how robust this parse is follows below.

**4. The std disclosure was too narrow: resolved.**
- The README now covers what was missing: the profile's checks in std code, and that the debug information in `tide-debug`'s std adds to its link.

**5. Cache writes and the second process were unstated: resolved.**
- The README's build bullet now says a Tide build's time includes the re-executed compile child and its cache writes.

**6. Run directory and nice value: resolved for the part taken.**
- The header now records `nice` (from `getpriority(PRIO_PROCESS, 0)`, with a SAFETY comment; it reads 19 under heavy.sh) and `run_dir` (`<out>/run`).
- The README's header list is updated.
- Moving the run directory was skipped, as allowed. Recording it is enough for anyone comparing datasets split across `--pair` runs.

**8. Wall-time boundary for runs: resolved.**
- The README now says run wall time includes `timeout`'s own start.

**10. Simplifications: (a), (b) and (c) resolved; (d) skipped as allowed.**
- (a):
  - `pairs.rs` declares `pub mod peak_rss; pub mod stats;`. `tests/benchmarks.rs` drops its own two `#[path]` includes and their `allow`, and uses `pairs::{peak_rss, stats}`.
  - The tests still compile the same source. Inside `pairs.rs`, which the tests load through `#[path]`, the nested `#[path = "support/…"]` paths resolve relative to `benches/`. They therefore name the same `benches/support/peak_rss.rs` and `stats.rs` as before. The existing `admitted.rs` include already relies on the same rule.
  - A `pub` module inside the private `mod pairs` is reachable from the test crate's root.
  - The `#[allow(dead_code)]` on `mod pairs` covers the nested modules, which replaces the dropped `allow` on `peak_rss`.
  - In the bench binary, every item of both modules is still used, so `pub` adds no dead-code warning. The author reports no warnings.
- (b): `header` uses `self.arg(p)`, which behaves the same as before.
- (c): `Config::informational()` is `matches!(compiler, Compiler::Stable)`. It gives the same values as the removed field: true only for `stable-release`. Both callers were updated.

**11. Doc nits: resolved.**
- The default results path now reads `<target dir>/tmp/bench-pairs/results/`.
- The dev and release rows now say "code generation", and the README states that Cargo's codegen-unit count and incremental mode do not apply.
- Neither the driver (`invocation.rs:342-398`) nor the Rust arms pass `-Ccodegen-units` or `-Cincremental`, so that statement is accurate.

## Non-blocking notes (no re-check needed)

**A. The bundled-linker test in `rust_linker` is a substring heuristic** (`pairs.rs:223-226`).
- It is correct for today's object form, `{"components": ["linker"]}`.
- If a future pin writes `link-self-contained` as `true`, the field would read "off" although the bundled linker is on. If it writes `false` and a later key spells `"linker"` before the next `}`, the field would read "on".
- This is informational header text, and the README already states the current linker fact. If it matters later, report the raw `link-self-contained` value instead of interpreting it.

**B. `cargo test --bench pairs` would still start a full measurement.**
- That command names the target explicitly, so it runs `main` with no arguments, which means full mode.
- The README's "runs only when named with `--bench pairs`" covers this case, but "`cargo test` never start[s] a measurement" reads broader than it is.
- Optional wording: "a `cargo test` that does not name it".

DONE
