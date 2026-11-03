# Tide

**Code that cannot do more than its signature says, programs that replay bit
for bit, and reviews that show what changed in what code may do, with Rust's
speed and safety.**

Tide is a systems language for software that AI agents write and humans have
to trust.

- **No ambient authority.** No code, in any dependency, can touch files, the
  network, or the clock unless it is handed that authority, so a function's
  signature bounds what it can do.
- **Every program replays.** Any Tide program, dependencies included, runs
  under a seeded simulator and reproduces bit for bit, so concurrency bugs
  become ordinary failing tests.
- **Review what changed in what code may do.** The compiler diffs authority,
  contracts, and `unsafe` holes between revisions, so reviewers see the risk
  before reading the code.
- **As safe and fast as Rust.** The same memory safety and data-race freedom
  without a garbage collector, and generics specialized like Rust's.
  Integer overflow never wraps silently, and no package code runs at build
  time beyond the toolchain's own proc macros and, from v1, the sandboxed
  build scripts of imported Rust crates.
- **Fast feedback.** Edits check in tens of milliseconds: no user proc
  macros, no build scripts, no inference across signatures, no unbounded trait solver.
- **Already familiar.** Rust syntax wherever the meaning is the same,
  including lifetimes, without macros or `async`/`await`. Contracts run as
  checks and tests today and become proofs later.
- **Works with Bend** for massively parallel, proved kernels.

*Tide is in the design phase: the guarantees are what the design commits to,
and the speed figures are targets, not measurements.*

## Status

Tide is in the design phase. The decision record is `design/01-design.md`
(draft 3.7). Syntax is provisional until the E1 learnability experiment
meets its thresholds (D0). D18 makes Tide a checked dialect of Rust,
compiled by a pinned rustc against Tide's std; its feasibility spike
(`spikes/d18/`) passed, and draft 3.7 applies its changes to the other
decisions.

The v0 checker has started, and its first piece is production code:
`crates/tide-admit`, the admission driver of D18's architecture A. It is a
rustc driver that enforces the admission allowlist (ADMIT-1, `FX0520`),
decides `Pure` structurally under the codes of the rules that need it (D18:
statics, `const fn`, `#[pure]`, `Shared`, parallel closures, and `FX0517`),
requires the `unsafe` grant for unsafe code (UNSAFE-1, `FX0901`), requires
`#[tide::main]` on a binary's `main` (ROOT-1, `FX0512`), rejects code that
calls or names `main` (ROOT-5, `FX0506`), and passes rustc
`-Zallow-features=` and `-Zpolonius=no` (ADMIT-2). After rustc's
analysis, a backstop checks the crate's instances that can run for denied
impls and `Pure` where-clauses, whatever route reached them (ADMIT-1). It constructs
every compiler invocation, clears the compiler environment (keeping only
`RUSTC_ICE=0`, so a compiler crash writes no report file), snapshots candidate
Rust sources, rejects file-loading attributes (`FX0911`), user macro definitions
(`FX0101`), `#![no_std]` (`FX0520`), and package build scripts (`FX0915`), and
caches successful metadata checks by source and trusted-input content. A crate
it admits against Tide's std also gets a trust artifact beside its metadata,
`<crate>.interface.json` (D11, 03 §4.1): each exported item's declared
authority, with the access path that carries each kind (`app.db.net: net`, or
`opaque(..)` where a type hides it), and the crate's trust holes, every site
the `unsafe` grant admits. It reports no exercised authority, effects, or proof
status. A crate whose `std` is neither Tide's nor Rust's fails the check; one
built on Rust's std has no capabilities and gets no artifact; and a failed
check leaves no artifact. `tide-admit --tide-diff BASE HEAD` compares two
artifacts: the items added, removed, or whose declared authority widened or
narrowed, with the paths that changed, and the holes added, removed, or
recounted; it prints nothing when neither changed. It is a partial precursor
to `tide review`'s trust diff (03 §4.3): no exercised authority, effects, or
proof status. The root Cargo workspace builds the driver with the nightly
that `rust-toolchain.toml` pins, with the `rustc-dev` component:

```
cargo test --workspace   # the driver's and Tide's std's tests
conformance/run.sh       # the conformance corpus, through stable rustc and the driver
```

The trusted build caller invokes the driver directly, for example:

```
target/debug/tide-admit --crate-type=lib --out-dir /tmp/tide-output path/to/package/src/lib.rs
```

The interface accepts one `.rs` entry, `--edition 2024`, `--crate-type lib|bin`,
`--crate-name NAME`, `--test`, `--emit metadata`, `--error-format short`,
`--out-dir DIR`, and `--tide-grant=unsafe` (value options also accept `=VALUE`).
`--tide-test-backstop`, for the driver's tests, skips the obligation steps of
the checks before analysis, so the backstop alone decides those routes, and
`--tide-test-ice` crashes the compiler after expansion.
Raw rustc flags and response files are rejected. The source directory is the
entry's parent; when it is named `src`, its parent is the package directory.
The package path must not traverse directory symlinks, and a resolved source
directory must stay beneath that package.
Output and `.tide-cache` state must be outside that package. Only canonical
`.rs` descendants of the source directory are served to rustc; committed
symlinks cannot grant access outside it. Each loaded source is checked before
expansion, and the expanded crate after it, which catches attributes and macro
definitions written in macro arguments. The cache records source bytes and
directory membership, driver and pinned compiler identity, grants, normalized
arguments, and trusted artifacts; failed checks never become cache hits.

`--tide-std PATH` and repeatable `--tide-dep-dir DIR` select trusted toolchain
artifacts outside the candidate package. These are operator inputs, like
grants; candidate source, Cargo configuration, toolchain files, and inherited
environment never set them. Without `--tide-std`, Rust's pinned std is used
for driver development tests, and `FX0512` is not checked, since Rust's std
has no `#[tide::main]`. Tide's std is `crates/tide-std`, with its macros in
`crates/tide-macros`: so far `#[tide::main]`, `Root` and the capability types
as identities without operations, `Pure`, and the `out` and `err` sinks
(`print!`, `println!`, `eprint!`, `eprintln!`); its crate docs list what it
does not provide yet. The conformance runner injects a stub std, a
compile-time fixture that shares Tide's std's definitions of those items. The
installed pinned toolchain is trusted and treated as immutable. This slice
provides loader and invocation isolation, with no OS sandbox or protection
against concurrent filesystem mutation; assembler, linker, runtime, and
test-process isolation remain later work.

[conformance/](conformance/README.md) holds the conformance corpus: cases
with expected verdicts per rule ID. Stable rustc checks their rustc marks,
and the driver checks their marks for the codes it decides. `spikes/d18/`
holds the throwaway feasibility spike for D18, and the early Rust scaffold
from the first design, which does not implement Tide, is kept in
`archive/v0-design/scaffold/`.

## Reading Guide

### Design

- [design/00-first-principles.md](design/00-first-principles.md): the problem
  statement, hard constraints, measurable targets, and hypotheses H1–H7 with
  their verdicts.
- [design/01-design.md](design/01-design.md): the decision record (D0–D18)
  and the tier table; it owns every decision.
- [design/reference.md](design/reference.md): the agent-facing reference for
  the v0 core, written as a delta from Rust, within a 25,000-token budget;
  [design/reference-companion.md](design/reference-companion.md) holds its
  on-demand tables, examples, and later tiers.
- [design/02-memory.md](design/02-memory.md): ownership, origins, variance,
  drop check, `Send`, `Sync`, and `Pure`; the definition the checker
  implements.
- [design/03-authority-concurrency.md](design/03-authority-concurrency.md):
  capabilities, structured tasks, parallelism, the simulator, the Bend
  boundary, and Rust import.
- [design/04-contracts.md](design/04-contracts.md): contracts, proof modes,
  and the trust surface.
- [REFERENCES.md](REFERENCES.md): the languages, projects, papers, and data
  Tide draws on.

Chapters 02–04 elaborate draft 3.7 and label which sections are v0 core and
which are later tiers; on any conflict, `01-design.md` and its tier table
win.

### Research

Cited evidence behind the decisions, in [research/](research/):

- `00-old-design-salvage.md`: what to keep, drop, and re-decide from the
  archived first design.
- `01-bend-hvm.md`: Bend, HVM, and interaction nets, measured against native
  code.
- `02-memory-models.md`: memory-safety and aliasing models, from Rust to Hylo
  and Mojo.
- `03-compile-speed.md`: which language decisions set the ceiling on check
  and build speed.
- `04-verification.md`: practical program verification and how well LLMs
  produce verified code.
- `05-llm-code-empirics.md`: which language properties help or hurt LLM
  coding agents.
- `06-agent-native-tooling.md`: agent-first languages and the toolchain
  contract agents need.
- `07-effects-capabilities.md`: effect systems, capabilities, and purity
  tracking.
- `08-type-system-core.md`: types, generics, traits, and metaprogramming.
- `09-adoption-interop-backends.md`: adoption, interop, and backend and
  bootstrap strategy.
- `10-concurrency-parallelism.md`: concurrency, deterministic parallelism, and
  deterministic simulation testing.
- `11-bend-interop.md`: how Tide and Bend 2 divide the work.
- `12-newcomer-sweep.md`: 2025–2026 languages and tools, the ideas Tide
  adopted from them, and lust, a checked Rust subset with fast edit checks.
- `13-agent-fleets.md`: what people running many agents and subagents need
  from a language, ranked, and where Tide meets or misses each need.
- `14-autoresearch.md`: Karpathy's autonomous experiment loop, why it does
  not fit Tide's design work, and where it fits once the checker exists.
- `15-rust-dialect.md`: whether Tide should ship as a checked dialect of
  Rust, compiled by stock rustc against Tide's std; per-feature
  classification, experiments, Eon as a test case, and a recommendation.

### Reviews and Trials

- [reviews/](reviews/): adversarial reviews, one file per reviewer and
  round, named `<bead>-r<N>-opus.md` or `<bead>-r<N>-codex.md`; the
  `round<N>-*.md` files are the earlier, global numbering. Round 1
  (agent usability, performance, soundness, strategy, verification, and a
  second-model review) examined draft 1;
  `round1-resolution.md` audits draft 2 against it. Round 2 (consistency,
  simplicity, and a second-model review) examined draft 2.1 and led to the
  tier split. Rounds 3–7 (consistency and second-model reviews) checked
  drafts 3 to 3.3 until no v0-blocking objection remained. Round 8 reviewed
  the switch to Rust lifetime syntax and led D2 to take Rust 2024's rules
  whole. `dialect-codex.md` is the second-model review of
  `research/15-rust-dialect.md`.
- [trials/](trials/): E1 learnability trials. A fresh agent that saw only a
  reference wrote the programs, and `grading.md` grades them against the
  normative chapters.
  - `e1/`: the pilot. It estimated a 6/10 first-check pass rate against
    about 9/10 for Rust, with no leaked Rust habits; the misses trace to
    gaps in the reference and one design gap.
  - `e1-round2/`: the same tasks after the reference fixes, 9/10. The
    reference now mirrors these tasks, so later rounds use held-out tasks.
  - `e1-origins/`: held-out, borrow-heavy tasks written once with `from`
    clauses and once with Rust lifetime syntax. Both arms passed 10/10 with
    no borrow errors, and Rust syntax needed about 3.5 times the
    annotations; the tie decided D2's Rust spelling.
  - `e1-dialect/`: the first round under D18, on held-out tasks covering
    outlives bounds, `use<..>`, scopes, capabilities, the simulator, and
    attribute contracts. One writer passed 11/11 with no definite errors; a
    rustc oracle against a stub std (`rustc/run.sh`) confirmed every type
    and borrow verdict. It records the writer's token cost.

## v0 Core and Later Tiers

- **v0 core**: Rust surface and ownership, with lifetimes checked as origins; traits with a power
  ceiling; `Result` and `?`; built-in `!` forms and derives; capabilities and
  `Root`; the `Pure` marker trait; structured tasks on blocking and thread
  runtimes; channels, `Shared::with`, `join`, `par_map`, and `par_for`;
  executable contracts as runtime checks and property-test drivers; the
  simulator with crashes and durability; the declared-authority trust diff;
  sealed delegation boundaries; unreviewed branches tested without a VM
  per run, in a fixed OS sandbox on Linux; per-dependency grants; the admission
  allowlist; the agent toolchain on rustc builds gated by the admission
  driver, then Tide's incremental front end (D18).
- **v1**: `prove` mode with SMT proofs, exercised-authority tracking, the
  evented runtime, FFI, direct Rust crate import and Rust consumers of Tide
  crates,
  `par_reduce`, field-borrow declarations, and production record and replay.
- **Later**: the Bend tandem, deferred overflow traps, automatic
  parallelization, `Shape` reflection, Polonius-precision borrows, and
  interactive-proof export.

## Project Process

- [AGENTS.md](AGENTS.md): rules for agents working here, generated from
  pinned protocol profiles plus `.agent-protocols.local.md`.
- [GOVERNANCE.md](GOVERNANCE.md): production code is agent-produced; humans
  set architecture, write specifications, and review; every change to
  `main` except a generated Starcompass import update (no review) or a
  planning-only one (no Codex pass) gets a Codex supervisor pass beside
  its first Opus review and a narrow Codex re-check of any fixes once the
  Opus reviews come clean, and only findings in the tiers a change edits, or earlier
  ones, block it.
- [BR_WORKFLOW.md](BR_WORKFLOW.md): how `br` tracks work and which `.beads/`
  files are committed.
- Open work lives in beads (`.beads/issues.jsonl`), managed with `br`.
- `scripts/autorun.sh` runs fresh headless sessions back to back: it
  continues a handed-off bead, else works the top ready one, judges each
  outcome from `br` and `git`, logs it to `.autorun/log`, stops on no work,
  repeated handoffs, repeated failures, or its caps, and never pushes. After
  every 10th session of each run, and when it stops with no work left, it
  runs the `correct` skill in a session of its own and logs the beads it
  filed and its cost; those beads stay uncommitted in `.beads/`.
  `scripts/autorun-test.sh` tests it offline with a stub agent.
- `scripts/handoff-guard.sh` is a Claude Code Stop hook for attended
  sessions: under 350k tokens of context, with a pickable bead ready, it
  blocks a stop that proposes a handoff or asks whether to go on, once.
  It runs once the owner adds its Stop entry to `.claude/settings.json`,
  which agents may not edit. `scripts/handoff-guard-test.sh` tests it
  offline.

## History

- [archive/v0-design/](archive/v0-design/): the first design, superseded by
  `design/`; its name predates the v0 core tier and is unrelated to it.
  `research/00-old-design-salvage.md` records what carried over.
- Naming: the language is named Tide (tidelang). Ferox was its working name;
  documents in `research/`, `reviews/`, `trials/`, `tasks/`, and `archive/`
  keep that name as written. [PUBLIC_NAME_REVIEW.md](PUBLIC_NAME_REVIEW.md),
  the `FEROX_*` documents, and the `ORBIS_*` documents record earlier naming
  reviews.
- [TASKS.md](TASKS.md) and `tasks/`: tasks from the first design's bootstrap
  work.
