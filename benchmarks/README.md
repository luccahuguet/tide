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
(`crates/tide-admit/tests/support/admitted.rs` shows both). The rerun script
of bead `fx-6u8m.6` will be the supported way to build and time both arms.
