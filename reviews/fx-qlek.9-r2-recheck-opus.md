Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 27d3a29

# fx-qlek.9 r2 re-check: Opus finder, findings 1, 3 and 4

Scope: `git diff d4a98d5 27d3a29` (4 files, +21 −11) and report.md. The SHA-256 of
`git diff 23d30f0...27d3a29`, which I recomputed, is
287a330b68064f153011fd75777fc29332f17eb52483a393e7dc7f853a8c2984 and matches the brief. I also read
`seed_set`, the code at the owner guide's new line range, tide-std's `Evaluation::signature`, and every
place that builds a `Demo`. I ran no builds or tests and changed no files except this one.

## Verdict

**approve**

## Finding 1: a relative or configured target dir sent the workspace test to a second target

**Fixed.**

- `Demo` has a new field, `target: Option<PathBuf>`. `run` uses it when it is set. Otherwise it falls
  back to the old resolution, `$CARGO_TARGET_DIR` or `<repo>/target`, so `cargo run -p tide-demo` is
  unchanged.
- `tests/demo.rs` passes `env!("CARGO_TARGET_TMPDIR")`'s parent. Cargo resolves this value to an
  absolute path at compile time, whether the target dir came from a relative `CARGO_TARGET_DIR`,
  `--target-dir` or `build.target-dir`. So the demo builds into the suite's own target. That removes
  both the stray `crates/tide-demo/<rel>` directory and the second full build.
- This matches the builder's evidence: the relative-`CARGO_TARGET_DIR` run passed 3/3 in 15.3 s, with
  no rebuild and no stray directory.
- `main.rs` uses `Demo::default()`, and the test's struct literals use `..Demo::default()`. So no
  construction site was missed, and all three test cases get the field through `demo()`.

**Shared state.** I checked that the fix shares nothing new with other tests:
- The target the cases now use is the target that the default and absolute-`CARGO_TARGET_DIR` setups
  already resolved to. The r2 analysis of the shared target therefore still holds: only the first
  inner build relinks the uplifted driver and std, under Cargo's build lock and before any driver
  runs.
- Each case's work dir is still `<tmpdir>/tide-demo-<case>-<pid>`, unique to the case and the
  process, and the `Work` guard still removes it.
- The test adds no mutable state shared with other tests in the target.

## Finding 3: the outcome check depends on the harness's internal outcome text

**Fixed.** The `LOST_HITS` doc comment now names `Evaluation::signature` in
`crates/tide-std/src/test/runner.rs`. That function exists (`runner.rs:199-200`, `impl Evaluation`).
It writes a panic as `panic: {:?} …`, with the payload as an `Option<&str>`, which is the source of
the `panic: Some("…` prefix. The check itself is unchanged and still fails closed.

## Finding 4: the seed-set line was hard-coded

**Fixed.**
- `seed_set` now returns `cases`, and the line prints `seeds = {cases}` and `0 to {cases - 1}`.
- `cases - 1` cannot underflow. `seed_set` takes `cases` from the first recording, so the list is
  not empty, and it returns `Ok` only when `recordings.len() == cases`. A `cases` of 0 is therefore
  an error before the print.
- The step-3 order is unchanged: a broken seed set is still reported before the status check.

## Other changes in the diff

- **Owner guide line ranges.** `06-replay.md` and `README.md` now point to `lib.rs:664-785`. At
  27d3a29, line 664 starts `record_and_replay`'s doc comment, line 666 is the `fn`, and line 785 is
  its closing brace. Both the prose and the `sed -n` command agree.

## New findings

None blocking.

1. **Nit, plausible, not verified: an explicit `--target <triple>` may defeat the fix's comment.**
   - From my memory of Cargo, which I did not verify (no Cargo source is on this host),
     `CARGO_TARGET_TMPDIR` comes from the unit's layout. Under `cargo test --target <triple>` that
     would make it `<target-dir>/<triple>/tmp`, so `scratch.parent()` would be `<target-dir>/<triple>`,
     not the target dir.
   - The demo would still pass. It would just do one more build, as d4a98d5 did in that setup, so
     this is no regression.
   - If this is right, the new comment in `tests/demo.rs` ("however the target dir was set") claims
     slightly more than holds. Nobody passes `--target` to this workspace's suite. Leave it.
2. **Nit, residual from finding 1, out of the taken fix's scope.** Interactive
   `cargo run --target-dir X -p tide-demo`, or a `build.target-dir` setting, still sends the demo's
   inner builds to `<repo>/target`, because Cargo exports neither to the child. Only
   `CARGO_TARGET_DIR` reaches the child, as before. The `target` field's doc comment describes this
   accurately. Leave it.

DONE
