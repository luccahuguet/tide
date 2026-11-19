# Tide demo: a lost update, replayed, and a widened interface

A labelled partial-v0 demonstration (fx-nkv). One command builds a small
concurrent service written in Tide through the admission driver and shows two
of Tide's claims on it:

1. **Tests replay.** A seeded simulation finds a schedule that loses an update,
   records it, and replays it twice with byte-identical event and output
   traces. The fixed revision passes the same seed set.
2. **Review shows what code may do.** A patch that gives the service's public
   function a `Net` parameter shows up in the trust diff as a widened access
   path, before anyone reads the code.

A last step, when gdb is available, runs the failing seed's replay under
`rust-gdb` with breakpoints in the service; [`docs/debugging.md`](../docs/debugging.md)
covers how Tide and debuggers compose.

## Run it

```sh
cargo run -p tide-demo
```

It exits 0 when every step shows what it should, 1 when one does not, and 2 on
a setup error. `crates/tide-demo` holds it, and `cargo test --workspace` runs
it (`crates/tide-demo/tests/demo.rs`), with two more runs that must fail: one
with the buggy patch applied as the fixed revision, which fails step 3, and
one without the widening patch, which fails step 4. It needs what the driver
needs:

- an x86_64 Linux host, since the driver links and runs tests only there, with
  `/usr/bin/cc` and `/usr/bin/ld`;
- rustup with the toolchain `rust-toolchain.toml` pins, nightly-2026-08-09
  with `rustc-dev` (`rustup toolchain install` in the repository installs
  it). The flake's dev shell (`nix develop`, [docs/nix.md](../docs/nix.md))
  stands in for rustup and `/usr/bin`'s linker: it provides the pinned
  toolchain and Nix's linker. When `PATH` has no rustup, the demo checks that
  the rustc (`$RUSTC`, else `rustc` on `PATH`) and the cargo on `PATH` report
  the pinned toolchain's release and commit hashes, and fails if either does
  not; its builds then run that cargo with `RUSTC` set to that rustc.
  `nix run .#tide-demo` runs the demo in that environment with nothing
  else installed ([docs/nix.md](../docs/nix.md#demo));
- `git`, to apply the demo's patches;
- for step 5 only, a gdb built with Python: `$RUST_GDB`, else `gdb` on
  `PATH`. Without one, or when the gdb on `PATH` cannot run Python, step 5
  prints a `skipped: …` line and the demo still passes; a `RUST_GDB` that
  does not work fails the step.

It builds the driver and Tide's std into `$CARGO_TARGET_DIR`, by default
`target/` in the repository, and works in its `tide-demo/` directory, which
each run replaces. The logs of every build and test run are in
`tide-demo/logs/`; the trust diff prints to the terminal. Once the driver and
std are built, a run takes about 20 seconds.

Its output, with paths shortened (the trace hashes change whenever the
service or the suite does):

```text
== 0. Toolchain and build
revision:  <commit>
toolchain: rustc 1.99.0-nightly (771916f90 2026-08-08)
driver:    target/debug/tide-admit
std:       target/debug/libtide_std.rlib

== 1. Find a seed that exposes the bug (buggy revision, demo/patches/lost-update.patch)
simulation every_view_is_counted: 8 passed; 8 failed (16 seeds)
failing seeds: 0 2 4 8 9 10 12 14 (each cause: lost hits)
same code and input on every seed; only the schedule differs
first failing seed: 0

== 2. Record seed 0, then replay the recording twice
cause: assertion `left == right` failed: lost hits
                 events  event trace       output trace
recording            40  9d66e1613fc69090  db1996770f19511f
replay-1             40  9d66e1613fc69090  db1996770f19511f
replay-2             40  9d66e1613fc69090  db1996770f19511f
output: views served: 2, hits recorded: 1
both replays reproduce the recording: identical event and output traces
recording: tide-demo/out/record/target/sim/run-<id>/run.manifest.json

== 3. Run the fixed revision (demo/service) over the fixed seed set
seed set: the suite's #[sim_test(seeds = 16)], seeds 0 to 15, the set step 1 searched
simulation every_view_is_counted: 16 passed; 0 failed (16 seeds); 16/16 cases passed (accepted/generated)
seed 0 on its own: passed

== 4. Authority diff of demo/patches/metrics-report.patch
-pub fn serve(tasks: &Tasks, load: &Load) -> Result<u64, ServeError> {
+pub fn serve(tasks: &Tasks, metrics: &Net, load: &Load) -> Result<u64, ServeError> {
declared authority (no exercised authority, effects, or proof status)
  + <hits::ServeError as core::convert::From<tide_std::io::Error>>::from (associated function)
  ~ hits::serve (function) widened
      + metrics: net

== 5. Debug the replay of seed 0 under rust-gdb, with breakpoints in record_hit
sandbox: full (Linux 7.2.6-…, Landlock ABI 10)
gdb: thread 4 enters record_hit at tide-library/hits.rs:57
gdb: thread 3 enters record_hit at tide-library/hits.rs:57
gdb: thread 4 reads hits = 0 at tide-library/hits.rs:57
gdb: thread 3 reads hits = 0 at tide-library/hits.rs:57
gdb: thread 4 writes hits = 0 + 1 over hits = 0 at tide-library/hits.rs:58
gdb: thread 3 writes hits = 0 + 1 over hits = 1 at tide-library/hits.rs:58
thread 3 wrote 0 + 1 over hits = 1: it read the count before another task wrote it, so that task's hit is lost
the replay under gdb reproduced the recording: the same schedule and trace

== Done: every step showed what it should. Logs and artifacts: target/tide-demo
```

To rerun a step by hand, open `logs/search.log` or `logs/record.log`: each
failing seed there prints its driver commands, shell-quoted and ready to
paste, as `fresh rerun:` (that seed, run afresh with `--seed`) and
`replay:` (the identity-checked replay of its manifest).

## The service

`service/src/hits.rs` is a page-hit counter. `serve` answers a batch of
traffic with one worker task per client, inside one task scope. Each worker
records a hit for every page view in a shared `Stats` cell, and `serve`
returns the hits recorded, which must equal the views served:

```rust
pub fn serve(tasks: &Tasks, load: &Load) -> Result<u64, ServeError> {
    let stats = Shared::new(Stats { hits: 0 }, tasks);
    tasks.scope::<_, ServeError, _>(|scope| {
        for _ in 0..load.clients {
            scope.spawn(|| {
                for _ in 0..load.views {
                    record_hit(&stats)?;
                }
                Ok(())
            });
        }
        Ok(())
    })?;
    Ok(stats.with(|s| s.hits)?)
}

fn record_hit(stats: &Shared<Stats>) -> Result<(), LockError> {
    stats.with(|s| s.hits += 1)
}
```

`&Tasks` is `serve`'s only capability, so it can neither read files nor open
sockets; like all Tide code, it can still write to the ambient `out` and
`err` sinks (`println!`, `eprintln!`). `Shared::with` runs its closure
holding the cell, and the closure must be `Pure`.

### The bug

`patches/lost-update.patch` splits the update into a read and a write, each
in its own critical section:

```rust
fn record_hit(stats: &Shared<Stats>) -> Result<(), LockError> {
    let seen = stats.with(|s| s.hits)?;
    stats.with(|s| s.hits = seen + 1)
}
```

Admission accepts it: there is no data race, since every access holds the
cell. The bug is logical. When the scheduler runs the second worker's read
between the first worker's read and write, both read the same count, both
write it plus one, and a hit is lost. Each `with` is a scheduling point, so
whether that happens depends on the schedule alone. The suite,
`tests/src/hits_sim.rs`, serves two clients one view each under a simulated
root and asserts that both hits were recorded:

```rust
#[sim_test(seeds = 16)]
fn every_view_is_counted(sim: SimRoot) {
    let load = Load { clients: 2, views: 1 };
    let hits = serve(&sim.tasks, &load).unwrap();
    let views = u64::from(load.clients * load.views);
    println!("views served: {views}, hits recorded: {hits}");
    assert_eq!(hits, views, "lost hits");
}
```

Each seed of `#[sim_test(seeds = 16)]` runs the same code on the same input
under a seeded scheduler, which picks the next task to run at each scheduling
point. About half the seeds interleave the two read-write pairs and lose a
hit; the others do not.

### What the demo does

The driver pairs the suite with one revision of the service at a time
(`--tide-test-library`); the suite sees it as the crate `tide_candidate`.
Every Tide crate here is built only by the driver, against production std.

1. **Find a seed.** It admits, links, and runs the suite against the buggy
   revision over seeds 0 to 15 and takes the first failing seed. Some seeds
   must pass and some fail: same code, same input, different schedule. Every
   failing seed's cause must be the `lost hits` assertion, so a seed that
   panics otherwise or fails the determinism self-check fails the demo. It
   reads each seed's outcome from the manifest the run recorded for it, and
   each failing seed's cause from the log, where the self-check reports.
2. **Record and replay.** It runs that seed alone (`--seed`), which records a
   `tide-replay/2` manifest and its `trace.ndjson`, then replays the manifest
   twice (`--replay`). The recorded seed's cause must be `lost hits` and its
   output must show fewer hits than views. Each replay must exit 101, as a
   reproduced failing test does, and say `replay reproduced`. Replay first checks that the sources, build options,
   compiler, std, the libraries the build loaded, and test configuration
   match the recording, and then
   compares the run's trace and outcome with it. The demo also compares the
   three traces byte for byte and prints each one's event count, the hash of
   its event trace, and the hash of its output trace: the `sink` events,
   which hold what the test printed.
3. **Fixed revision.** It runs the suite against `service/` over the same
   seeds 0 to 15, then the failing seed on its own. Both pass.
4. **Authority diff.** `patches/metrics-report.patch` makes `serve` report
   its total to a metrics collector, so `serve` now takes `metrics: &Net`.
   The demo admits both revisions, which writes each crate's trust
   artifact, `hits.interface.json`, and compares them with
   `tide-admit --tide-diff`. The diff lists `hits::serve` as widened, with
   the path that gained `net`. The widened revision is only admitted, never
   run, and the demo does no network I/O.
5. **Debug the failing seed.** It runs step 2's `replay:` command under
   `rust-gdb -batch` with the script it writes to `debug.gdb`. gdb follows
   the driver into its sandboxed test child and prints, at `record_hit` and
   its two `Shared::with` closures, each read and write of the count, from
   the named locals `s` and `seen`, and the source line gdb stopped at. The
   debug profile's debug information supplies both. Every stop must be on
   its line of the buggy `hits.rs`: the entry and the read on
   `let seen = …`, the write on `stats.with(|s| s.hits = seen + 1)`. Every
   `record_hit` breakpoint must be matched by one read and one write, from
   more than one thread, and each write must write back its thread's last
   read plus one. One write must land on a count higher than the one its
   task read, so it overwrites the other task's hit. The replay must
   still say `replay reproduced`. gdb, and the driver it detaches, must
   finish within 120 seconds; otherwise the demo kills every process still
   in gdb's process group and the step fails. gdb's output is in
   `logs/debug.log`.

The driver runs each test child in RUN-3's kernel sandbox and prints its
level (`sandbox: full`, …) at the top of each log.

## Files

| Path | What it is |
| --- | --- |
| `../crates/tide-demo/` | the one command, `cargo run -p tide-demo`, and its workspace test |
| `service/src/hits.rs` | the service, fixed revision |
| `patches/lost-update.patch` | the buggy revision, as a patch to the service |
| `patches/metrics-report.patch` | the `Net` widening, as a patch to the service |
| `tests/src/hits_sim.rs` | the simulation suite run against each revision |
| `agent-card.md` | the same, condensed for coding agents |
| `../docs/debugging.md` | how Tide composes with debuggers, and rr compared |

## What runs today, and what is planned

Runs today, and this demo uses it:

- `tide-admit`, the admission driver, building a library, a paired test
  suite, and linked test executables against Tide's std;
- `#[sim_test]` with seeded scheduling of `Tasks` scopes and `Shared` cells;
- `--tide-run-test NAME [--seed N]` and `--replay MANIFEST`, with the
  identity check and the trace comparison;
- the trust artifact and `--tide-diff` on declared authority and `unsafe`
  holes.

Planned, and not shown here:

- the `tide` command: `tide test`, `tide sim`, and `tide review`; the
  driver's run and replay modes are their partial precursors, and the
  `tide test ... --seed N` line in the failure output names that planned
  frontend;
- exercised authority, effects, and contracts in the review diff, and the
  attenuation in `main` that reaches a changed path (03 §4.3);
- packages and dependencies (`tide.toml`, `tide.lock`): the driver admits one
  crate, or one suite paired with one library;
- simulated network I/O beyond TCP loopback, fault injection, crashable
  nodes, `--minimize`, and other scheduler strategies (03 §9);
- disk crashes, production record and replay, and any claim about running
  unreviewed code safely: this demo runs reviewed code only.

## One std, one fixture

Production std (`crates/tide-std`) is the runtime owner. The conformance
corpus's stub std (`conformance/rustc/std`) is a compile-time fixture, and for
every std item this demo uses it compiles production's own source:

| Items the demo uses | Production source the stub includes |
| --- | --- |
| `Tasks`, `Scope`, `Task`, `TaskError` | `task.rs` |
| `Shared`, `LockError` | `sync.rs` |
| `SimRoot`, `#[sim_test]` | `sim.rs`, `test/` |
| `Net`, `Net::connect` | `net.rs` |
| `std::io::Write`, `std::io::Error` | `io/` |

So each item has one definition, and both surfaces give it the same signature
and verdicts. The stub's own `Net` additions (`get`, `drop_all`, `NetError`,
`Url`) have no production counterpart, and the demo uses none of them.

## Where the rules live

- `design/01-design.md` D11 (trust surface), D15 (simulation), D18 (the
  checked dialect);
- `design/03-authority-concurrency.md` §4.3 (trust diff), §6.1 and §6.5
  (scopes, `Shared::with`), §9.6 (REPLAY-1, REPLAY-2);
- [`docs/driver.md`](../docs/driver.md) for the driver's flags, the replay manifest, and
  the RUN-3 sandbox.
