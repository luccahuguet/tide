# E1 on the checked toolchain: frozen protocol

Status: frozen by bead `fx-gfq` (2026-10-31). Bead `fx-ic2` runs it. This
file and `tasks.md` define the paired learnability experiment that D0
(`design/01-design.md`) gates the month-3 decision on: "E1 on the first
checked toolchain". D0 owns the thresholds; §8 states them exactly and
changes none.

Earlier rounds (`trials/e1`, `e1-round2`, `e1-origins`, `e1-dialect`) had
one writer, no paired Rust arm, and hand grading, at most against a
compile-only stub std. None of them is a gate run, and nothing here
reuses their tasks or results.

## 1. What this protocol can and cannot conclude

- Only a **gate run** (§3 to §8) on a toolchain that meets every
  prerequisite in §10 can report E1 as passed or failed.
- A **metadata-only pilot** (§9) checks admission and type and borrow
  verdicts against a compile-time stub std. It runs nothing, so it
  measures no final pass rate and no iterations to green, and it never
  reports E1 as passed or failed.
- A gate run that fails a threshold is reported as failed. Running it
  completes bead `fx-ic2`; it does not close D0's gate.

## 2. Freeze and held-out rules

- The thirty tasks in `tasks.md` are new. None appears in the reference,
  the companion, the design chapters, the conformance corpus, the spikes,
  or an earlier trial. Some resemble earlier material the reference was
  tuned on, and the report reads their results with that in mind:
  companion 15.1 (arguments and files) for B1, B2, and B5; 15.2 (deadline
  outcomes) for C5; 15.3 (contracts with quantifiers) for C7 and C8; 15.4
  (an append-only log) for B4; reference 6.3's atomic-replace snippet for
  B9; e1-dialect's `ring` for C7 and `scan` for A10's `with_word`; and
  e1-origins' `group_words` for A1's drop case.
- From this freeze until the gate report is published, nobody copies a
  task, a checked case, a reference solution, or a writer's program into
  the reference, the companion, a diagnostic's text, or an example, and no
  change to those documents draws on `trials/e1-checked/`. This covers
  the prerequisite beads, the readiness work, and the reference solutions
  as much as the run itself; a change that breaks it is recorded on
  `fx-ic2` and the gate run then needs a fresh held-out task set.
- Task text, checked cases, the measures, and the thresholds do not change
  once the first gate writer has started. Before then, a change that fixes
  a task's precision is recorded on `fx-ic2` with its reason; making a
  task easier or harder to change the expected outcome is not allowed.
- A later E1 (D0: "E1 reruns on every release") uses a fresh task set.

## 3. Arms, writers, and sample

- **Arms.** *Tide*: the writer gets the preface and its task rendered for
  Tide, `design/reference.md`, and `design/reference-companion.md`, both
  byte-identical to the revision the run records. *Rust*: the writer gets
  the preface and its task rendered for Rust, and no other document.
- **Writer.** Every session uses one model and one agent harness, the
  same for both arms, recorded by exact identifier and version:
  `claude-opus-5-5` at the harness's default effort unless `fx-ic2`
  records another single choice before the first session. Each session is
  fresh: it has seen no task, no other session's output, and no grading.
- **Isolation.** A session runs in its own directory holding only its
  inputs and package skeleton. It may read and write files there and run
  `./check` and `./test`; it has no other shell, no network, no web search,
  and no access to the repository or other sessions.
- **Sample.** Each task is written twice in each arm, by independent
  sessions: 60 trials per arm, 120 in all. The trials run in an order
  fixed before the first session: the 120 `(task, arm, replicate)` triples,
  sorted, shuffled by Python 3's `random.Random(20261031).shuffle`, and
  run in that order in batches that keep the
  coordinating session's own context bounded (raw logs stay on disk).
- **Infrastructure faults.** A session that ends through no act of the
  writer (a model API error, a harness crash, a host fault) is rerun once
  with a fresh session; the report counts such reruns. A second fault in
  the same slot counts that trial as failed.

## 4. Writer procedure

### 4.1 Package and commands

The harness makes the skeleton: a library (`src/lib.rs`) or a binary
(`src/main.rs`), as the task says, empty, plus the arm's package files
(Tide: whatever manifest the toolchain needs, no grants; Rust: a
`Cargo.toml` for edition 2024 with the three crates the preface allows).

- `./check` checks the package and the writer's tests without running
  anything: Tide, the admission driver and rustc on the library or binary
  and on its test build; Rust, `cargo check --all-targets`. It prints the
  toolchain's diagnostics as they are.
- `./test` runs the same check and, when it passes, builds and runs the
  writer's tests (Tide: plain, property, simulation, and generated
  contract tests, as `tide test` does; Rust: `cargo test`), printing the
  results. Each test has a 10-second wall limit, and a whole `./test` run
  120 seconds.
- Both commands run the writer's code inside the harness sandbox (§10.3).

### 4.2 Rendering and budget

- The writer's prompt is the preface and its one task from `tasks.md`.
  A line starting `- Tide:` or `- Rust:` goes only to that arm, together
  with its continuation lines (lines indented by two spaces); the arm
  label itself is removed. Nothing else is changed.
- The preface also tells the writer that its first run is measured, so it
  writes the whole program and its tests before the first run.
- Budget per session: 20 runs of `./check` or `./test` in total and 40
  minutes of wall time. The harness refuses runs past the budget and ends
  the session at the time limit.
- The session ends when the writer says it is done or the budget runs
  out. Its last saved files are the **final snapshot**; the files at its
  first run are the **first snapshot**.

## 5. Toolchains

Each identity is recorded before the first session and held for the whole
run; a toolchain change mid-run voids the run.

- **Tide**: the repository revision of `crates/` (the `tide-admit` driver
  and the production std, macros, and test runner it builds against), the
  pinned nightly in `rust-toolchain.toml`, and the harness revision. Tide
  code builds only through the driver; no step falls back to stable rustc
  or the conformance stub. Diagnostics are what the driver and rustc
  print. If they lack D14's fix plans or typed holes, or `./test` reports a
  failing seed without the replay manifest of companion 17 (`fx-nkv.7`,
  not a prerequisite), the report says so; such a limitation counts
  against Tide, as it would for a user.
- **Rust**: stable rustc and Cargo 1.96.0, edition 2024, offline, with
  `serde` 1.0.229 (`derive`), `serde_json` 1.0.154, and `thiserror` 2.0.21,
  and their lockfile, vendored.
- Both arms build and test in debug mode, where overflow checks are on in
  both languages.

## 6. Acceptance suites and grading

- **Suites.** For each task and arm, the grader writes an acceptance suite
  that does exactly what the task's checked cases say and nothing more:
  Tide library suites use the simulator as the preface describes, Rust
  library suites use temporary directories and real time with the stated
  slack, and binary suites run the built program as a process with only
  the case's files, arguments, standard input, and environment variables.
  Each test has a 10-second wall limit.
- **Structure checks.** Where a task states a structure requirement for an
  arm, a fixed text search over the final snapshot checks it, as the task
  words it. A mention in a comment satisfies it; the report notes such
  cases.
- **Concurrency.** Only C10 has a checked case that a serial program
  fails. For C1 to C6 and C9 a grader reads each final snapshot and fails
  a trial whose work the task says runs concurrently or in parallel does
  not, citing the lines: in particular a C3 trial without one producer per
  input running concurrently with the merge, a C5 trial whose jobs do not
  all start at once, and a C6 trial whose searches do not check for the
  stop at least every 4096 bytes. The review is recorded per trial, and
  the preface tells writers it happens. Simulated suites run 20 seeds unless a
  case says otherwise.
- **Reference solutions.** For each task and arm the grader also writes a
  solution, kept outside every writer's directory. Each must pass `./check`,
  its structure check, and its acceptance suite on the recorded toolchains;
  this is the executable readiness proof of §10.2.
- **Freeze.** The suites, structure checks, and render script are frozen
  before the first gate session, and the SHA-256 of their tree is recorded
  on `fx-ic2`. After that a suite may only lose a test that a disputed
  result shows checks something the task text leaves open; the test goes
  from both arms' suites, and the report records it. No test is added or
  tightened.
- **Grading is mechanical** for the measures in §7, apart from the
  concurrency review. Classifying errors
  (§7.4) follows, by a grader who records the evidence for each class.

## 7. Measures

A trial is one session. Every measure is computed per arm over its 60
trials.

### 7.1 First-check pass

A trial passes first check when its first snapshot passes `./check` with
the writer's own tests and, for a library, also compiles with the
acceptance suite (so the public items the task names exist with usable
shapes). A trial with no run has no first snapshot and fails. The rate is
passing trials over 60.

### 7.2 Final pass

A trial passes when its final snapshot passes `./check`, all of the
writer's own tests (for Tide, with the generated contract tests and the
vacuity floor that `./test` runs), its structure checks, its concurrency
review, and every test of its acceptance suite. The rate is passing
trials over 60.

### 7.3 Iterations

A trial's iterations are all its runs of `./check` and `./test`,
including any after its first green run. D0 measures
iterations to green, so the arm's iteration figure is the mean over the
trials that pass final; failed trials already count against the arm in
§7.2. If either arm has no passing trial, threshold 3 is not met. The
mean over all 60 trials with each failure counted as 20 is reported as
information.

### 7.4 Recorded but not gated

- Per trial: every run's verdict, each diagnostic code (`FX`, rustc `E`
  codes, lints), each failing test's name, the wall time, the agent turns,
  the input, cache-read, cache-creation, and output tokens, the peak
  context, for Tide which reference sections the writer read, and the
  first snapshot's size and number of `todo!`/`hole!` holes, so the report
  can show first runs made on unfinished programs.
- Every first-check failure and every final failure, classified as in
  `trials/e1-dialect/grading.md` (Rust habit leaked, reference gap, design
  gap or contradiction, ordinary bug) plus **toolchain defect** (a
  reference-taught item that does not execute, or a toolchain bug), with
  the evidence. A toolchain defect still counts as a failure in §7.1 to
  §7.3; it is flagged, never excluded.

## 8. The gate, as D0 states it

With F, P, and I for an arm's first-check rate, final pass rate, and mean
iterations, E1 meets its thresholds when all three hold:

1. F(Tide) >= 0.8 × F(Rust);
2. P(Tide) >= P(Rust);
3. I(Tide) <= 1.5 × I(Rust).

These are D0's initial thresholds on its 30-task suite, applied to the
point estimates of the fixed sample above. The report also gives, for
information only, 95% intervals for each ratio and difference from 10,000
bootstrap resamples of the 30 tasks (both replicates and both arms of a
task resampled together, percentile intervals, tasks drawn with Python
3's `random.Random(20261031).choices`); with 60 trials per arm these
intervals are wide, and the report says how wide. The report states each
threshold as met or not met, publishes every trial's record (§7.4), and
gives the commands that reproduce its grading from the stored snapshots.

## 9. Metadata-only pilot

A pilot may run before the prerequisites are met, to test the harness and
spot gross reference gaps.

- It uses the driver on the conformance stub std, `--emit metadata` only.
  `./test` does not exist; the writer is told that only `./check` is
  available and that its tests will not run.
- It measures the first-check rate (without the acceptance-suite compile,
  where the stub lacks an item) and the check verdict of the final
  snapshot, labelled **pilot** everywhere they appear.
- Its sessions never count toward a gate sample, and its results never
  report E1 as passed or failed.
- If pilot output leads to a change in the reference, the companion, or a
  diagnostic's text before the gate run, the change is recorded on
  `fx-ic2`, and the gate run then needs a fresh held-out task set, since
  this one has shaped the documents it tests.

## 10. Executable prerequisites

### 10.1 What the tasks use

The Tide arm's tasks and acceptance suites use these parts of the v0
reference. Each must execute on the checked toolchain, not just
type-check. A writer may also reach for any other reference-taught item;
one that does not execute is a toolchain defect (§7.4).

| Surface | Tasks | Owner |
| --- | --- | --- |
| Every `FX` code the reference names, at the reference's severity: errors (`FX0301`, `FX0702`, `FX0913`, ...) rejected, lints (`FX0508`, `FX0607`, `FX0608`) reported | all | `fx-u65` (E1-P14) |
| Driver-built binaries and libraries; plain `#[test]` and `#[should_panic]` runs; overflow traps | all | `fx-kw8.9` |
| `#[tide::main]`, `Root`, `print!`/`println!`/`eprintln!` sinks, minimal std | all binaries | `fx-nkv.1` |
| `Tasks::scope`, `spawn`, `join`, scoped borrows, panic propagation | C3–C6, C9, C10 | `fx-nkv.2` |
| Scope failure precedence, level-triggered cancellation | C3–C6, C10 | `fx-nkv.3` |
| `Shared::new`, `Shared::with`, poisoning | C4 | `fx-nkv.4` |
| Seeded scheduler | all simulated suites | `fx-nkv.5` |
| `#[sim_test]`, `Root::sim`, multi-seed runner | all simulated suites | `fx-nkv.6` |
| `Tasks::limit`, `set_deadline`, `cancel`, `checkpoint`, `shield`, `spawn_isolated`, `Panicked` | C4–C6, C9, C10 | `fx-zc1` (E1-P1) |
| `Clock` (`now`, `sleep`, `sleep_until`), `Instant`, virtual time | B10, C5 | `fx-o8j` (E1-P2) |
| `Shared::with2`, `with_all`, `same`, `reset`, `LockError::Aliased` | C4 | `fx-ugm` (E1-P3) |
| Channels: `sync_channel`, `channel`, `send`, `recv`, `recv_deadline`, iteration, error enums | C3 | `fx-o4h` (E1-P4) |
| `std::par::join`, `par_map`, `par_for`, `par_sort` | C1, C2 | `fx-iob` (E1-P5) |
| `Fs`, `File`, `OpenOptions`, `Mode`, `dir`, `read_only`, `write_synced`, `sync_dir`, `rename`, `read_dir`, `metadata`; `sim.world.file` | B1, B3–B7, B9 | `fx-byk` (E1-P6) |
| `Env`, `Stdin`, `stdout`/`stderr`, log sinks, `main` result and `ExitCode` exits; `sim.world.args`, `env_var`, `stdin`, `sim.records` | B1, B2, B5, B8 | `fx-5vj` (E1-P7) |
| `Net::bind`, `Net::connect`, `TcpListener`, `TcpStream`, simulated loopback | C10 | `fx-hxw` (E1-P8) |
| `derive(Error)` with `#[error]`, `#[from]`, `#[source]` | any (A3, A8, A9, B6–B8 invite it) | `fx-3iu` (E1-P9) |
| `derive(Encode, Decode)`, `#[codec]`, `std::json` | A9, B8 | `fx-08m` (E1-P10) |
| `#[requires]`, `#[ensures]`, `invariant!`, `old`, `result`, quantifiers, violation panics | C7, C8 | `fx-mhd` (E1-P11) |
| Property tests, generated contract tests, vacuity | C7, C8, any writer property test | `fx-m72` (E1-P12) |
| `#[pure]`, `Arena`/`Id`, `std::float`, `std::math`, `truncate`, `cast_signed`/`cast_unsigned`, `round_from`, float `TryFrom`, fixed-key `HashMap`, and the allowlist entries for the `core`/`alloc` items the reference admits | A2, A5, A6, B7, any | `fx-yg5` (E1-P13) |

Every bead in the table blocks `fx-ic2`. The ones filed for E1 carry the
label `e1-prereq` and cite their row by its E1-P number.

### 10.2 Readiness checklist

`fx-ic2` starts its gate run only when all of these hold, each recorded on
the bead with its evidence:

1. Every bead named in §10.1 is closed.
2. Every reference solution (§6) passes `./check`, its structure check,
   and its acceptance suite in both arms on the recorded toolchains, and
   the Tide solutions do so through the driver with the production std,
   never the stub.
3. For every `FX` code the reference names, a program violating it gets
   that code from the driver at the reference's severity (E1-P14's
   conformance cases): rejected for an error, reported for a lint. So a
   Tide writer cannot pass with code the reference forbids, and lint-level
   code is not rejected.
4. The suites, structure checks, and render script are frozen (§6).
5. The harness sandbox (§10.3) is in place for both arms.
6. The toolchain identities (§5), the model and harness identity (§3), and
   the trial order are recorded.

A gate run that starts with any item missing cannot report E1 as passed
or failed.

### 10.3 Running unreviewed code

Writer programs are unreviewed code, and `fx-kw8.9`'s build mode is for
reviewed code only until D13's test sandbox exists. The E1 harness
therefore runs every build, test, and binary of both arms inside its own
operating-system sandbox: a fresh copy of the trial directory as the only
writable tree; read access only to that copy, the recorded toolchains and
vendored crates, and the system libraries they load, so no repository
file, other trial, suite, or reference solution is readable; no network
beyond the sandbox's own loopback; a clean environment; and the time
limits of §4.1. This sandbox belongs to the
harness, not to Tide, and the report does not cite it as D13's
unreviewed-branch guarantee.

## 11. Coverage and known limits

- **Covered**: lifetimes and borrowing views (A1, A8, A10), integers and
  overflow (A2, A3, C9), floats (A5), traits, operators, and trait objects
  (A4, A5), generics (A6, C2, C7), errors and derives (A3, A8, A9, B6–B8),
  capabilities and attenuation (B1–B10), `Pure` (B7, C1, C2), sinks and
  exit codes (B1, B2, B5, B8), durability (B9), time (B10, C5), structured
  tasks, cancellation, and isolation (C3–C6, C9, C10), `Shared` (C4),
  channels (C3), parallel regions (C1, C2), contracts (C7, C8), and TCP
  (C10).
- **Not covered**: `Rand` (Rust's std has no generator, so a paired task
  would hand the Rust arm a crate or a hand-written generator); `Net::get`,
  `Url`, and `sim.world.http` (Rust's std has no HTTP client, so the pair
  would not be the same task); `Proc`; simulated faults, nodes, and
  crashes (the Rust arm cannot be tested the same way); multi-file modules
  and manifests with grants; and `tide review`, `tide fix`, `tide eval`,
  `tide explain`, and `tide fmt`. E2, E3, and E6 test the last group's
  purposes.
- **Asymmetries kept on purpose**: the Tide arm reads a 25,000-token
  reference and the Rust arm reads nothing; Tide's generated contract
  tests and its `FX` checks run in `./test` and `./check`; Tide-only
  structure requirements make Tide writers use the feature the task
  probes. Each makes the Tide arm's job no easier. One runs the other way:
  Tide builds on the pinned 1.99 nightly while the Rust arm uses stable
  1.96.0, so Tide writers may use language and `core` APIs stabilized in
  between; the report counts any final snapshot that does.
