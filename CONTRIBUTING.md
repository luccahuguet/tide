# Contributing to Tide

Tide is a checked subset of Rust 2024 that runs on rustc. It complements
Rust and is not a fork: a place to move faster on stricter, checkable rules
while staying valid Rust.

## Who writes the code

Tide is AI-led and human-steered. AI agents design, specify, write,
review and validate its production code and its design text: they make the
other technical choices, and each landing needs an AI review's approval. The owner steers,
setting scope, priorities and direction and making the owner calls.
Hand-written production code is an exception that a change marks as one
([GOVERNANCE.md](GOVERNANCE.md), Development Rule).

AI-written contributions are welcome because every change here is checked,
and they meet the same bar as every change: review rounds by models from
different providers, and runnable proof scoped to what the change touches.
[GOVERNANCE.md](GOVERNANCE.md) (Merge Rule) binds the review, and
[AGENTS.md](AGENTS.md) (AP-PROOF-001, AP-TEST-001) binds the proof.

## Soundness bugs in rustc

Tide's checker rests on rustc, so adversarial testing of the checker can
find rustc soundness bugs. Those go upstream to rust-lang/rust through
people: the Rust project's contribution policy limits LLM-written issues and
comments, so an agent never writes text posted there. A human writes the
report in their own words.

## Build and test

The workspace builds with the nightly that `rust-toolchain.toml` pins, with
its `rustc-dev` component; rustup installs it on first use. The tests also
need the `stable` toolchain (`rustup toolchain install stable`) and
`/usr/bin/python3` with MPFR (`libmpfr.so.6`; on Debian/Ubuntu, `python3` and
`libmpfr6`).

```
cargo test --workspace          # the driver's, Tide's std's, the demo's, and the runner's tests
cargo run -p tide-conformance   # the conformance corpus, through stable rustc and the driver
```

The first run installs the pinned nightly with `rustc-dev` (about 1.4 GB)
and compiles a rustc driver, so it is slow. [docs/driver.md](docs/driver.md)
says what each part checks.

## How to contribute

1. Open a GitHub issue to propose work, or a pull request with the change.
   Before larger changes, open an issue first so the owner can place it.
   Maintainers track accepted work as beads, the issues in
   `.beads/issues.jsonl` (JSON lines, readable with `br` from
   [beads_rust](https://github.com/Dicklesworthstone/beads_rust); see
   [BR_WORKFLOW.md](BR_WORKFLOW.md)). Contributors do not edit `.beads/`.
2. Language decisions live in [design/01-design.md](design/01-design.md). A
   substantial language or toolchain addition passes the design admission
   filter ([GOVERNANCE.md](GOVERNANCE.md), Design Admission Rule).
3. Make the change with its proof, and keep the docs it touches current.
4. A maintainer runs the review the Merge Rule requires and decides whether
   the change lands. Expect to fix or rebut each finding; review records may
   be committed under `reviews/`.

Unless you state otherwise, a contribution is licensed under Apache-2.0
([LICENSE](LICENSE), section 5).

## Roles

- **Owner**: sets Tide's direction and scope.
- **Maintainers and co-leads**: own technical areas.
- Maintainers join by the owner's invitation.

## The name

The license covers the code, not the name; see [README.md](README.md),
License and Name.
