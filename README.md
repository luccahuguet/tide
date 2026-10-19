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
  time beyond the toolchain's own proc macros.
- **Fast feedback.** Edits check in tens of milliseconds: no user proc
  macros, no build scripts, no inference across signatures, no unbounded trait solver.
- **Already familiar.** Rust syntax wherever the meaning is the same,
  including lifetimes, without macros or `async`/`await`. Contracts run as
  checks and tests today and become proofs later.
- **Works with Bend** for massively parallel, proved kernels.

*Tide is in the design phase: the guarantees are what the design commits to,
and the speed figures are targets, not measurements.*

## Status

Tide is in the design phase; there is no compiler for this design. The
decision record is `design/01-design.md` (draft 3.7). Syntax is provisional
until the E1 learnability experiment meets its thresholds (D0). D18 makes
Tide a checked dialect of Rust, compiled by a pinned rustc against Tide's
std; its feasibility spike (`spikes/d18/`) passed, and draft 3.7 applies
its changes to the other decisions.

The repository holds no production code yet. `spikes/d18/` holds the
throwaway feasibility spike for D18, and the early Rust scaffold from the
first design, which does not implement Tide, is kept in
`archive/v0-design/scaffold/`.

## Reading Guide

### Design

- [design/00-first-principles.md](design/00-first-principles.md): the problem
  statement, hard constraints, measurable targets, and hypotheses H1–H7 with
  their verdicts.
- [design/01-design.md](design/01-design.md): the decision record (D0–D18)
  and the tier table; it owns every decision.
- [design/reference.md](design/reference.md): the agent-facing reference for
  the v0 core, written as a delta from Rust, under a 25,000-token budget.
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
  round. Round 1 (agent usability, performance, soundness, strategy,
  verification, and a second-model review) examined draft 1;
  `round1-resolution.md` audits draft 2 against it. Round 2 (consistency,
  simplicity, and a second-model review) examined draft 2.1 and led to the
  tier split. Rounds 3–7 (consistency and second-model reviews) checked
  drafts 3 to 3.3 until no v0-blocking objection remained. Round 8 reviewed
  the switch to Rust lifetime syntax and led D2 to take Rust 2024's rules
  whole. `dialect-codex.md` is the second-model review of
  `research/15-rust-dialect.md`.
- [trials/](trials/): E1 learnability trials. A fresh agent that saw only a
  reference wrote the programs, and `grading.md` grades them without a
  compiler against the normative chapters.
  - `e1/`: the pilot. It estimated a 6/10 first-check pass rate against
    about 9/10 for Rust, with no leaked Rust habits; the misses trace to
    gaps in the reference and one design gap.
  - `e1-round2/`: the same tasks after the reference fixes, 9/10. The
    reference now mirrors these tasks, so later rounds use held-out tasks.
  - `e1-origins/`: held-out, borrow-heavy tasks written once with `from`
    clauses and once with Rust lifetime syntax. Both arms passed 10/10 with
    no borrow errors, and Rust syntax needed about 3.5 times the
    annotations; the tie decided D2's Rust spelling.

## v0 Core and Later Tiers

- **v0 core**: Rust surface and ownership, with lifetimes checked as origins; traits with a power
  ceiling; `Result` and `?`; built-in `!` forms and derives; capabilities and
  `Root`; the `Pure` marker trait; structured tasks on blocking and thread
  runtimes; channels, `Shared::with`, `join`, `par_map`, and `par_for`;
  executable contracts as runtime checks and property-test drivers; the
  simulator with crashes and durability; the declared-authority trust diff;
  per-dependency grants; the admission allowlist; the agent toolchain on
  rustc builds gated by the admission driver, then Tide's incremental front
  end (D18).
- **v1**: `prove` mode with SMT proofs, exercised-authority tracking, the
  evented runtime, FFI, direct Rust crate import,
  `par_reduce`, field-borrow declarations, and production record and replay.
- **Later**: the Bend tandem, deferred overflow traps, automatic
  parallelization, `Shape` reflection, Polonius-precision borrows, and
  interactive-proof export.

## Project Process

- [AGENTS.md](AGENTS.md): rules for agents working here, generated from a
  pinned protocol profile plus `.agent-protocols.local.md`.
- [GOVERNANCE.md](GOVERNANCE.md): production code is agent-produced; humans
  set architecture, write specifications, and review; every PR needs an AI
  supervisor verdict before it merges.
- [BR_WORKFLOW.md](BR_WORKFLOW.md): how `br` tracks work and which `.beads/`
  files are committed.
- Open work lives in beads (`.beads/issues.jsonl`), managed with `br`.

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
