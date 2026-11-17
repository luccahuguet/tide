# Owner's guide: Tide's ten core ideas

This guide is for Tide's owner. It assumes you can run commands and read code a
little, but not that you know Rust's internals, rustc's, or Tide's. Each
chapter takes about 20 minutes. It explains one idea in plain English, gives
you commands to run, tells you what to look for in their output, and ends with
the hardest question a skeptic would ask and the honest answer.

The guide describes `main` as it is on 2026-11-17. When something is planned
rather than built, the guide says so. Work still in review is named with its
bead and marked "in review"; a hidden `TODO(tide-lead, <bead>)` comment in the
chapter's source tells the coordinator what to update when it lands.

## A weekend plan

| When | Chapter | The idea in one line |
| --- | --- | --- |
| Saturday morning | [0. Setup](#setup) | build the driver and std once (about 10 minutes of machine time) |
| | [1. Capabilities](01-capabilities.md) | code can touch the world only through values it was handed |
| | [2. Admission](02-admission.md) | what Tide's checker checks, and what it leaves to rustc |
| | [3. Soundness bugs](03-soundness-bugs.md) | where the compiler itself has holes, and which ones Tide closed |
| Saturday afternoon | [4. Tasks and `Shared`](04-tasks-and-shared.md) | concurrency with no orphan threads and no lock deadlocks |
| | [5. Seeds and the simulator](05-seeds-and-simulator.md) | a seed picks the schedule, so a race becomes a test |
| | [6. Replay](06-replay.md) | rerun a recorded failure exactly, and what it can't reproduce |
| Sunday morning | [7. The trust diff](07-trust-diff.md) | review what a change may do before reading how it does it |
| | [8. The RUN-3 sandbox](08-sandbox.md) | the kernel fence around test runs |
| Sunday afternoon | [9. E1 and E3](09-experiments.md) | what the experiments measure, and what they showed |
| | [10. v0 and what's planned](10-v0-and-planned.md) | what runs today and what doesn't yet |
| | [Answers sheet](answers.md) | the ten hardest questions, one paragraph each, for calls |

## The code that matters

You don't need to read Tide's code to defend it, but you can honestly say
you've read the parts that matter. Each chapter's "Read the code" step walks
through one of these line by line, in plain English. In weekend order, about
an hour and a quarter in all:

| # | File | Chapter | What it shows | Minutes |
| --- | --- | --- | --- | ---: |
| 1 | `crates/tide-admit/src/policy.rs:64-72, 92-135` | 1 | the table of Rust's ambient calls Tide refuses | 7 |
| 2 | `conformance/cases/authority/cap_ambient.rs` | 2 | one conformance case: how expected verdicts are written | 10 |
| 3 | `conformance/cases/memory/closure_fnptr_capability.rs` | 3 | the 13-line capability forge `TD0206` stops | 3 |
| 4 | `crates/tide-admit/src/walk/reify.rs:1-55` | 3 | the `TD0206` check itself | 10 |
| 5 | `demo/service/src/hits.rs` | 4 | a whole Tide service, 58 lines: tasks, a scope, `Shared` | 15 |
| 6 | `demo/patches/lost-update.patch` | 4 | the one-line-pair change that makes it race | 3 |
| 7 | `demo/tests/src/hits_sim.rs` | 5 | the simulation test that finds the race | 5 |
| 8 | `crates/tide-demo/src/lib.rs:676-797` | 6 | how the demo proves replay is exact | 8 |
| 9 | `demo/patches/metrics-report.patch` | 7 | the change the trust diff flags | 10 |
| 10 | `crates/tide-admit/src/invocation/sandbox.rs:84-101, 549-578` | 8 | the sandbox level, and seccomp's allowed system calls | 8 |

Chapters 1–3 belong together, as do 4–6. Read [the answers sheet](answers.md)
last, and check that you could give each answer yourself.

## Words used everywhere

- **Rust**: the programming language Tide is built on. Tide code is Rust code.
- **rustc**: Rust's compiler, the program that turns Rust source into a program
  you can run.
- **crate**: Rust's unit of compilation, a library or a program built from one
  root file.
- **std**: a language's standard library, the code every program gets for free
  (files, networking, threads, and so on).
- **Tide**: a *checked dialect* of Rust. Every Tide program is a valid Rust
  program, but Tide's checker admits only a subset of Rust, and compiles it
  against Tide's own std instead of Rust's (design decision D18).
- **tide-admit**: the checker, also called *the admission driver* or just *the
  driver*. Today it is Tide's only command. The planned `tide` command (`tide
  check`, `tide test`, `tide sim`, `tide review`) does not exist yet.
- **nightly**: a daily development build of rustc. Tide pins one exact
  nightly (in `rust-toolchain.toml`), so everyone builds with the same
  compiler.
- **v0, v1, later**: Tide's tiers. v0 is what the first release commits to;
  v1 and later come after. `design/01-design.md` has the tier table.

## Setup

You need the x86_64 Linux machine the project lives on, with rustup and the
pinned toolchain (`rustup toolchain install` in the repository installs it).
`cargo test --workspace` and the conformance corpus also need the `stable`
toolchain (`rustup toolchain install stable`) and `/usr/bin/python3` with MPFR
(`libmpfr.so.6`; on Debian/Ubuntu, `python3` and `libmpfr6`), which the float
oracle uses.
Run every command from the repository root.

Build the driver and Tide's std, and run the demo once. The first build takes
several minutes; later runs take about 20 seconds:

```sh
cargo run -p tide-demo
```

It ends with `== Done: every step showed what it should.` and exit code 0.
After it, `target/debug/tide-admit` (the driver) exists, and
`target/tide-demo/` holds the demo's logs and recordings (chapters 5–8).

Then gather Tide's std and its helper libraries into one folder,
`target/guide/std/`, so the guide's commands can name them simply. Cargo,
Rust's build tool, lists the files it built; this copies each one:

```sh
bash -c 'mkdir -p target/guide/std && cargo build --offline --locked -p tide_std --message-format=json | grep -o "\"filenames\":\[[^]]*\]" | grep -o "\"[^\"]*\.\(rlib\|so\)\"" | tr -d "\"" | xargs -I{} cp -f {} target/guide/std/ && ls target/guide/std'
```

It lists about ten files, among them `libtide_std.rlib` (the std; `.rlib` is
a compiled Rust library) and a
`libtide_macros-….so` (its built-in attributes such as `#[tide::main]`). It's
the recipe in the repository `README.md`'s demo build, made to fit one line.
If you rebuild Tide's std later, run it again.

If you set `CARGO_TARGET_DIR`, the build goes there instead of `target/`;
change the paths in the commands to match.

**Sharing the machine with agents.** The agent rules send every command that
compiles through `~/.cache/eon-agents/eon-overseer/heavy.sh`, so builds don't
overload the machine. If agents are running, put
`bash ~/.cache/eon-agents/eon-overseer/heavy.sh` in front of any command that
compiles (`tide-admit`, `rustc`, `cargo`, so the demo too), for example
`bash ~/.cache/eon-agents/eon-overseer/heavy.sh cargo run -p tide-demo`.

**Running the driver on your own file.** Most chapters check a small example
program from [examples/](examples/). Every such command has the same shape:

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/NAME docs/owner-guide/examples/NAME/src/lib.rs
```

- `--crate-type=lib` checks a library; `bin` checks a program with a `main`;
- `--tide-std` and `--tide-dep-dir` name Tide's std and its helper libraries;
- `--out-dir` is where the driver writes its outputs, which must be outside
  the program's own folder;
- the last argument is the program's root file.

When the driver rejects a program it prints the errors and exits with status 1.
nushell may add a line saying the command failed; that is expected. Every
command in this guide is meant to paste into nushell as written; the review
of this guide checked the forms it uses (relative paths, `--` passed through,
`bash -c '…'`) in nushell 0.113.1.

## Where the authority lives

This guide explains; it decides nothing. When it and the design disagree, the
design wins: `design/01-design.md` owns every decision, the chapters in
`design/` elaborate it, and the repository `README.md` states what runs
today. Each chapter ends with links into them.
