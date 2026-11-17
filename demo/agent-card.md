# Agent card: the hit-counter demo

Partial v0. Everything here runs today; the `tide` command does not exist yet.

**Run:** `demo/run.sh` (exit 0 pass, 1 a step failed, 2 setup). Honors
`CARGO_TARGET_DIR`; works in `$CARGO_TARGET_DIR/tide-demo/`, logs in its
`logs/`. x86_64 Linux, pinned nightly with `rustc-dev`, git.

**Code:**
- `demo/service/src/hits.rs`: the service, fixed. `pub fn serve(tasks: &Tasks, load: &Load) -> Result<u64, ServeError>`.
- `demo/patches/lost-update.patch`: buggy revision (read and write in two `Shared::with` calls).
- `demo/patches/metrics-report.patch`: adds `metrics: &Net` to `serve`.
- `demo/tests/src/hits_sim.rs`: `#[sim_test(seeds = 16)] fn every_view_is_counted`.

**Driver commands** (`D=$CARGO_TARGET_DIR/debug/tide-admit`, default `target/`; `STD` and dep dirs as `demo/run.sh` collects them):

```sh
# admit and run the suite against one revision; add --seed N for one fresh seed
$D --test --crate-type=lib --emit=link --crate-name=hits_sim \
  --tide-std $STD --tide-dep-dir DIR... --out-dir OUT \
  --tide-test-library LIB/src/hits.rs demo/tests/src/hits_sim.rs \
  --tide-run-test every_view_is_counted [--seed N]
# replay a recording: same build arguments, then
  --replay OUT/target/sim/<run>/run.manifest.json
# trust artifact (OUT/hits.interface.json), then compare two
$D --crate-type=lib --crate-name=hits --tide-std $STD --tide-dep-dir DIR... --out-dir OUT LIB/src/hits.rs
$D --tide-diff BASE/hits.interface.json HEAD/hits.interface.json
```

**Read results:**
- Failing seed: `simulation failure seed=N`; summary `simulation every_view_is_counted: P passed; F failed (16 seeds)`; exit 101.
- Each failing seed prints paste-ready `fresh rerun:` and `replay:` commands.
- Replay success: `replay reproduced: test=... seed=N outcome=...`, exit 101 when the recording failed; a mismatch is a driver error, exit 2.
- Each evaluation writes `OUT/target/sim/<run>/run.manifest.json` (`tide-replay/2`) and `trace.ndjson`; `"kind":"sink"` events are the output.
- Diff: `~ item (kind) widened` then `+ path: kinds` per gained access path. Declared authority only.

**Rules:** keep `serve`'s capabilities explicit in its signature; `Shared::with` closures must be `Pure`; a scope's tasks finish before `scope` returns. Owners: `design/03-authority-concurrency.md` §4.3, §6.1, §6.5, §9.6; `design/01-design.md` D11, D15, D18; the repository `README.md` for flags and replay.

**Planned, not here:** `tide test`/`tide sim`/`tide review`, exercised authority and effects in diffs, packages, fault injection, `--minimize`, simulated network I/O beyond TCP loopback.
