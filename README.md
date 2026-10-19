# Tide

Tide is a systems language design for software that coding agents write and
humans must trust. Its core is three guarantees that a language can make only
if it is designed for them from the start. First, there is no ambient
authority: code reaches files, the network, the clock, the environment, or
randomness only through capability values it is handed, so a function's
signature bounds what it can do, in every dependency. Second, whole programs
run under deterministic simulation: any program, dependencies included,
replays bit for bit from a seed when its foreign effects have simulator
models. Third, the compiler computes a reviewable
trust surface (signatures, authority, contract status, and trust holes such as
`unsafe` and FFI) and diffs it between revisions. Tide delivers these on a
Rust-familiar surface, with Rust spelling wherever the meaning is the same,
ownership with origins instead of lifetime parameters, no async, and no user
macros, and it aims for much faster builds than Rust. Contracts (`requires`,
`ensures`) are part of the language and serve as runtime checks, property
tests, and optional SMT proofs; they support the trust surface rather than
lead it. Tide works in tandem with Bend 2, not in competition: Tide is the
effectful, authority-checked shell, and Bend owns proved, massively parallel
pure kernels.

## Status

Tide is in the design phase; there is no compiler for this design. The
decision record is `design/01-design.md` (draft 3). Syntax is provisional
until the E1 learnability experiment meets its thresholds (D0).

`crates/` holds an early Rust scaffold (seven crates, about 700 lines) from
the archived first design; it does not implement the current design.
`scripts/test/e2e_workspace_smoke.sh` builds and tests it.

## Reading Guide

### Design

- [design/00-first-principles.md](design/00-first-principles.md): the problem
  statement, hard constraints, measurable targets, and hypotheses H1–H7 with
  their verdicts.
- [design/01-design.md](design/01-design.md): the decision record (D0–D17)
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

Chapters 02–04 elaborate draft 2.1 and still describe some later-tier
features; on any conflict, `01-design.md` and its tier table win.

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

### Reviews and Trials

- [reviews/](reviews/): adversarial reviews. Round 1 (agent usability,
  performance, soundness, strategy, verification, and a second-model review)
  examined draft 1; `round1-resolution.md` audits draft 2 against it. Round 2
  (simplicity and a second-model review) examined draft 2.1 and led to the
  tier split.
- [trials/e1/](trials/e1/): the first E1 learnability pilot. An agent that saw
  only `reference.md` wrote ten programs, which `grading.md` grades without a
  compiler against the normative chapters. It estimated a 6/10 first-check
  pass rate against about 9/10 for Rust, with no leaked Rust habits; the
  misses trace to gaps in the reference and one design gap.

## v0 Core and Later Tiers

- **v0 core**: Rust surface and ownership with origins; traits with a power
  ceiling; `Result` and `?`; built-in `!` forms and derives; capabilities and
  `Root`; the `Pure` auto trait; structured tasks on blocking and thread
  runtimes; channels, `Shared::with`, `join`, `par_map`, and `par_for`;
  executable contracts as runtime checks and property-test drivers; the
  simulator with crashes and durability; the declared-authority trust diff;
  per-dependency grants; the agent toolchain on an interpreter, then a
  Cranelift-class backend.
- **v1**: `prove` mode with SMT proofs, exercised-authority tracking, the
  evented runtime, the C-emitting release backend, FFI, Rust crate import,
  `par_reduce`, field-borrow declarations, and production record and replay.
- **Later**: the Bend tandem, deferred overflow traps, automatic
  parallelization, `Shape` reflection, Polonius-precision borrows, and
  interactive-proof export.

## Project Process

- [AGENTS.md](AGENTS.md): rules for agents working here, generated from a
  pinned protocol profile plus `.agent-protocols.local.md`.
- [GOVERNANCE.md](GOVERNANCE.md): production code is agent-produced; humans
  set architecture, write specifications, and review.
- [AI_SUPERVISOR.md](AI_SUPERVISOR.md): the AI supervisor review every PR
  needs before it merges.
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
- [ZERO_LANGUAGE_FINDINGS.md](ZERO_LANGUAGE_FINDINGS.md): findings from a
  review of Zerolang (Zero), an agent-first language.
- [TASKS.md](TASKS.md) and `tasks/`: tasks from the first design's bootstrap
  work.
