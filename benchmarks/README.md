# Benchmark workloads: Tide and plain Rust

Four small programs, each written twice: once in Tide (`tide.rs`) and once
in plain Rust (`rust.rs`), with the same observable output. They are the
workloads of the benchmark against plain Rust (bead `fx-6u8m`); this
directory holds the programs and the check that keeps the two arms
equivalent, not measurements.

| Pair | Tide arm | Rust arm | Default size |
| --- | --- | --- | --- |
| `contracts` | a prime sieve, integer square root, and sorted insert, with `#[requires]`, `#[ensures]`, and `invariant!` | the same code without the contracts | limit 20,000 |
| `files` | creates, appends to, lists, reads, renames, and removes files through `Fs` | the same through `std::fs` | 50 files |
| `tcp` | a server task and a client exchange lines on loopback through `Net` | the same through `std::net` and a thread | 200 messages |
| `tasks` | four workers in a `Tasks::scope`; per block of 16 numbers, one update of a `Shared` cell and one message over an `mpsc` channel | `std::thread::scope`, a `Mutex`, and `std::sync::mpsc` | 2,000 numbers |

Each program takes one optional argument, its size, and uses the default
above without one. The default is small, so the check stays fast; timed
runs pass a larger size. `files` works in a new `files-workload` directory
under the working directory, refuses to start if one exists, and removes it
at the end. `tcp` binds `127.0.0.1` on a port the system picks and never
prints it. The Rust arms use no `async` and no `unsafe`, and need nothing
beyond std.

Within a pair, both arms do the same work in the same order: the same
algorithms, calls, paths, buffers, and messages. What differs is the cost of
Tide's implementation, and that cost is what the pairs measure:

- Every pair: the driver builds every Tide program with
  `-Coverflow-checks=on`, `-Zinline-mir=no`,
  `-Zcross-crate-inline-threshold=never`, and `-Zshare-generics=no`, in
  release as in debug (`crates/tide-admit/src/invocation.rs`). So every
  arithmetic operation traps on overflow, including the Collatz steps in
  `tasks` and the counters in `files` and `tcp`; there is no MIR inlining at
  all, no inferred cross-crate inlining, and generics are instantiated per
  crate. A Rust build with the same flags separates this cost from the rest.
- `contracts`: in debug and test builds the Tide arm checks its contract
  clauses; release builds erase them. The cost is concentrated in
  `insert_sorted`, whose `requires(xs.is_sorted())` and
  `ensures(xs.is_sorted())` each scan the vector, on an insert that is itself
  linear, so they roughly triple that function's cost. The clauses on
  `sieve` and `isqrt` cost a constant per call. A debug-build ratio from
  this pair is therefore not a typical contract cost.
- `files`: each Tide call goes through the root `Fs` that `main` receives
  and resolves its path through the capability
  (`crates/tide-std/src/fs/real.rs`), which costs extra system calls. A
  rename opens both parent directories, calls `renameat`, and closes both:
  five calls against std's one `rename`. A removal opens the parent, calls
  `unlinkat`, and closes it: three against one. The `exists` check at the
  start is three against one. Listing reads the whole directory into a
  snapshot and sorts it, and the program then sorts the names again, as the
  Rust arm does, so the Tide arm sorts twice. Every `File` read and write is
  also a cancellation checkpoint. In all, about 18 system calls per file
  against 12. The authority checks themselves (a mode comparison, a NUL-byte
  scan) cost little next to that. The Tide arm does not narrow the
  capability to the workload's directory (`Fs::dir`), which the Rust arm has
  no counterpart for.
- `tcp`: Tide's real sockets are nonblocking
  (`crates/tide-std/src/net/real.rs`). A read that would block returns
  `EAGAIN`, then waits in `poll` on the socket and the task's eventfd, reads
  the eventfd, and reads the socket again. Each wait also takes the
  runtime's lock and registers a waiter. So one request and reply costs
  roughly ten system calls against four for std's blocking sockets.
- `tasks`: tasks are OS threads in both arms. In the Tide arm, spawning goes
  through `Tasks::scope`, and every `Shared::with`, send, and receive goes
  through the runtime. Receives are also cancellation checkpoints; sends on
  this unbounded channel are not. `Shared::with` acquires its cell in FIFO
  order, where std's `Mutex` is not fair. Each worker makes one cell update
  and one send per block of 16 numbers, so this cost grows with the size, as
  does the scheduling work when the pair runs under the simulator.

## The equivalence check

`crates/tide-admit/tests/benchmarks.rs`, part of `cargo test --workspace`,
builds each Tide arm through the admission driver against production std and
each Rust arm with plain `rustc --edition=2024` (the toolchain
`rust-toolchain.toml` pins, unless `RUSTC` is set), runs both in an empty
directory, once without an argument and once with a small explicit size, and
requires byte-identical standard output, exit code 0 from both arms, and an empty directory
afterwards. Output holds nothing nondeterministic: no timings, ports, or
arrival order (`tasks` folds its blocks into per-worker summaries, in worker
order, whatever order the blocks arrive in). The explicit size is 3, so one
`tasks` worker gets no numbers.

Every directory in `benchmarks/` is a pair, and the check fails rather than
skip anything it does not expect: a pair directory must hold exactly
`tide.rs` and `rust.rs`, the only other file allowed here is this README, and
the four pairs above must all be present.

The Tide arm of `tasks` also runs under the simulator: its test runs the
program's `run` on eight seeds, requires the same output from each, and
prints it; the check requires it to equal the Rust arm's output.

To run only this check:

```sh
cargo test -p tide-admit --test benchmarks
```

To build a Rust arm by hand:

```sh
rustc --edition=2024 -O -o tcp-rust benchmarks/tcp/rust.rs
./tcp-rust 10000
```

A Tide arm also needs the production std built first and the driver's flags
(`crates/tide-admit/tests/support/admitted.rs` shows both). The rerun tool
below is the supported way to build and time both arms.

## Rerunning the benchmark

```sh
cargo bench -p tide-admit --bench pairs -- [--quick] [--out DIR] [--pair NAME]...
```

The rerun tool (`crates/tide-admit/benches/pairs.rs`) builds every pair under
each configuration below, runs each built program, and writes one raw record
per measurement. It is a `harness = false` bench target of the driver's
package, so it builds Tide arms with the equivalence check's helpers and Cargo
builds the driver optimized, as rustc itself is. It runs only when named with
`--bench pairs`: a plain `cargo bench`, `--benches`, `--all-targets`, and
`cargo test` never start a measurement (`cargo test` type-checks it as a
module of the equivalence check and tests its output check). It needs rustup
with the `stable` toolchain installed, for the `stable-release` arm, and
checks for it before it creates anything.

| Configuration | Compiler | Flags |
| --- | --- | --- |
| `tide-debug` | the driver, std built in Cargo's dev profile | `--tide-profile=debug` |
| `tide-release` | the driver, std built in Cargo's release profile | `--tide-profile=release` |
| `rust-debug` | the pinned nightly's rustc | Cargo's dev profile's code generation: `-Copt-level=0 -Cdebuginfo=2`, debug assertions and overflow checks on |
| `rust-release` | the pinned nightly's rustc | Cargo's release profile's code generation: `-Copt-level=3`, debug assertions and overflow checks off |
| `rust-release-tide-flags` | the pinned nightly's rustc | `rust-release` with the flags the driver adds in every profile: `-Coverflow-checks=on -Zinline-mir=no -Zcross-crate-inline-threshold=never -Zshare-generics=no` |
| `stable-release` | the stable toolchain's rustc (`rustup run stable`) | as `rust-release`; for information only |

Every Rust arm also passes `--edition=2024`, and the tool runs each rustc from
its toolchain's sysroot, not through rustup's proxy. Cargo's own codegen-unit
count and incremental mode do not apply: like the driver, rustc runs
non-incrementally with its default codegen units. Some differences stay
inside the Tide arms' numbers, since no configuration separates them:

- the linker: the driver links with `/usr/bin/cc` and the system `ld`, with
  rustc's bundled linker disabled (`invocation.rs`), while rustc on the
  current pins links through `cc` with its bundled `rust-lld`;
- the std: Tide's std is built by Cargo in the arm's profile and its
  non-generic code runs at that optimization level and with that profile's
  checks, and in `tide-debug` its debug information adds to the link, while
  Rust's std is always the toolchain's optimized build.

What it measures, for each pair and configuration:

- build: the wall time and peak resident memory of the compiler process. A
  build runs in a new directory, so the driver's build cache never hits; a Tide
  build's time still includes the driver's re-executed compile child and its
  writes to that cache, Tide's real cost for a cold build. Peak
  memory comes from `wait4` (`benches/support/peak_rss.rs`) and is the
  largest peak among the process and the descendants it waits for: the
  driver's compile child and the linker, not their sum.
- run: the wall time, including `timeout`'s own start, of the built program
  at its timed size (the table in
  `pairs.rs`, `TIMED`: chosen so the release arms run for about 0.5-2 s), in
  a new empty directory, under `timeout` with a 120 s limit. Its peak memory
  is recorded too and includes `timeout`'s own, about 2.4 MiB. Every
  configuration's output must equal `tide-debug`'s, or the tool writes an
  `error` record and stops.

The method is fx-ayxr's: rounds interleave the configurations (each round
measures each configuration once, in table order, so two arms run A B A B).
A full run measures a pair's builds, then its runs, in 5 rounds each, and adds
5 rounds while any configuration's wall times have a coefficient of variation
over 1%, up to 30 rounds. It reports the median, the CV, and the median peak
memory of each series. A series still over 1% at 30 rounds keeps its records
and is marked unstable in its summary record and in the closing line.

`--quick` measures one round at each program's small default size, for
checking the tool only. `--out` names a new or empty directory, as an absolute
path (Cargo runs the tool in the package's directory); by default each run
writes to a new directory under `<target dir>/tmp/bench-pairs/results/`
(`target/` unless `CARGO_TARGET_DIR` is set), which is
never committed. `--pair` limits a run to the pairs it names. A failed build
or run, or a run whose output differs, stops the tool with exit status 1,
after its record is written.

The results directory holds `records.jsonl`, one JSON object per line, and
`bin/<pair>/<configuration>`, the built programs. Each object's `record` field
names its kind:

- `header`, first: mode, start time, git revision and whether the tree is
  dirty, CPU model and count, kernel, load average, the process's nice value,
  the directory the runs use, the path, `rustc -vV`, and default linker (from
  the target spec) of the pinned and the stable compiler, the driver's path,
  `cc` and `ld` versions, the round settings, the sizes, and each configuration's compiler,
  std profile, and flags;
- `build` and `run`, one per measurement: pair, configuration, round, size
  (runs), `wall_s`, `peak_rss_kib`, and `exit`, plus the tail of standard
  error when the process failed;
- `summary`, per pair, phase, and configuration: `n`, `median_s`, `cv`,
  `median_peak_rss_kib`, and `stable` (null in a quick run);
- `error`, on an output mismatch: pair, configuration, round, the run's
  `exit`, the reference configuration, and the tails of the expected and the
  actual output;
- `end`: finish time and load average, written only when every measurement
  succeeded. A dataset without `end` is incomplete (a failure or an
  interruption) and is not a result.

A full run on the 2026-11-17 machine (16 threads) takes about 50 s per round
over all four pairs, so about 5 minutes when every series settles at 5 rounds
and about 25 minutes at the cap, plus the std builds and, the first time, the
optimized driver build. In this repository's agent setup it runs through
`heavy.sh` in a booked quiet slot (`AGENTS.md`, Heavy runs), for example:

```sh
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=... TMPDIR=.../tmp timeout 3600 \
  cargo bench -p tide-admit --bench pairs -- --out /abs/results/dir </dev/null
```
