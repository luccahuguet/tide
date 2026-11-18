// Tide v0 paper. Typst source; build with typst 0.15.0 (3ae52774) from the
// repository flake's nixpkgs pin 567a49d1913ce81ac6e9582e3553dd90a955875f:
// nix shell github:NixOS/nixpkgs/567a49d1913ce81ac6e9582e3553dd90a955875f#typst \
//   -c typst compile paper/tide-v0.typ <out>.pdf
// Links to in-repository reports target their repository paths.

#set document(title: "Tide v0: a capability-safe, replayable subset of Rust for code that agents write", author: "Lucca Huguet")
#set page(paper: "a4", margin: (x: 2.4cm, y: 2.4cm), numbering: "1")
#set text(size: 10.5pt, lang: "en")
#set par(justify: true)
#set heading(numbering: "1.1")
#show link: set text(fill: rgb("#1a4f8a"))
#show heading: set block(above: 1.4em, below: 0.8em)
#show raw.where(block: false): set text(size: 0.95em)
#show table: set par(justify: false)
#show quote.where(block: true): set block(inset: (left: 1em))

#align(center)[#text(size: 17pt, weight: "bold")[Tide v0: a capability-safe, replayable subset of Rust for code that agents write]]

#align(center)[#text(size: 12pt)[Lucca Huguet]]

_Disclosure._ Tide's code, its design text, and this paper were produced by AI agents (Claude and GPT models: mainly Claude Opus and GPT Sol, with some earlier production code by Claude Fable) under the author's supervision. The author set scope and direction and approved decisions. Human-written production code is allowed only as an explicitly marked exception. Under the project's merge rule, a change lands only after an AI supervisor pass, normally a pass by GPT Sol through OpenAI's Codex CLI and then an Opus review; §8 gives the exceptions (#link("GOVERNANCE.md")[GOVERNANCE.md]).

_Status._ Draft for the v0 alpha, kept in the Tide repository. Design decisions are cited by their IDs in the decision record, #link("design/01-design.md")[`design/01-design.md`] (D0 to D18). Every measurement in this paper comes from a report in the repository and is cited where it appears. Features of v1 and later tiers are labelled planned.

#heading(level: 1, numbering: none)[Abstract]

When models write most of the code and humans review it, a language should make it cheap to check what a change may do and to reproduce what a program did. Tide is a checked subset of valid Rust 2024 built for that setting. Code reaches files, the network, the clock, the environment or randomness only through capability values it is handed, so a function's signature bounds its authority. Because every source of nondeterminism is such a capability, a test can run under a seeded simulator and a failing run can be replayed from its recorded manifest. Tide v0 is not a new compiler: an admission driver built on a pinned rustc compiles each crate against Tide's own standard library and checks it against Tide's rules before rustc writes any artifact.

We report what v0 shows and what it does not. A seeded test finds a lost update in a small concurrent service, and its replay reproduces the recorded traces byte for byte. Admission has closed three admitted routes from two open rustc soundness bugs, the second found by a red team that tried 48 routes, controls included; other rustc soundness bugs remain open. A pre-registered pilot of the trust diff as a review aid did not meet its threshold: plain review already caught almost every planted change, which left no room to show a benefit. A paired learnability trial found that one model writes Tide about as well as Rust on 30 small tasks: parity within the protocol's margins, not a benefit. Benchmarks against plain Rust show only ties and losses. Two case studies show that admission catches none of the real bugs studied; what Tide adds there is a deterministic test of the fix.

= Motivation

When code-writing models produce changes faster than people can read them, the bottleneck moves from writing to checking: deciding whether a change does what its description says and nothing more, and reproducing a failure well enough to fix it. A language cannot make review free, but it can decide how much of a change a reviewer must read to answer those two questions.

Tide starts from four properties of the agent's edit loop (#link("design/00-first-principles.md")[`design/00-first-principles.md`]): latency per check, the signal quality of each diagnostic, locality (a function is understood from its body and the signatures it uses), and convergence (one obvious way to write a common thing, so independent attempts produce similar code). It adds a learnability constraint. A new language has no training data, so its reference must fit in an agent's context, and its surface should borrow what models already know.

Prior evidence shaped how close to Rust Tide stays (summarized in #link("design/01-design.md")[`design/01-design.md`], "Evidence That Shaped The Design"). In RustAssistant, giving GPT-3.5 a localized error with a fix description instead of a changelog-only prompt raised its repair rate for Rust compile errors from 10.7% to 73.7% (#link("research/05-llm-code-empirics.md")[`research/05-llm-code-empirics.md`]). And Tide's first draft would have rejected or respelled constructs, despite their identical meaning, in 76.8% of files of a 6.08M-line Rust sample (#link("reviews/round1-agent-usability.md")[`reviews/round1-agent-usability.md`]). Tide therefore keeps Rust's spelling wherever the meaning is the same (D1), and under D18 every Tide program is a valid Rust crate.

What Rust does not give is a bound on authority. Any Rust function, in any dependency, can open a file, read the clock or connect to the network through the standard library. Tools layered on Rust cannot remove that, because the code can always reach `std`. A reviewer of a Rust change must read every body to learn what the change may touch, and a test of concurrent Rust code cannot in general be rerun deterministically from a seed, because its dependencies reach real time, real threads and real randomness. Record-and-replay tools such as rr can record one real run and replay it; rr's chaos mode randomizes schedules, but no seed reproduces a run. Tide's thesis (#link("design/01-design.md")[`design/01-design.md`], "Thesis") is that a language can close both gaps by construction and keep Rust's guarantees: memory safety and data-race freedom without a garbage collector.

= Design principles

== Low entropy

Tide removes what costs agents the most and adds as little as it can. It has no user macros (D6), no `async`/`await` (D8), and a trait system with a hard power ceiling (D4). D14 specifies a deterministic formatter with no options; it is planned. GOVERNANCE.md applies an admission filter to substantial language changes: "A feature should be rejected by default if it increases semantic entropy more than it improves safe change." The core reference, #link("design/reference.md")[`design/reference.md`], is written for agents to load whole and has a budget of 25,000 tokens (D14).

== Capabilities bound authority

Code touches the world only through capability values it is handed (D7): `Fs`, `Net`, `Clock`, `Env`, `Tasks` and others. A program's `main` receives a `Root` and passes narrower capabilities down; a test can take a simulated root instead. A function that takes no capability cannot read the clock, the network, files, the environment or randomness, in any dependency, by design. Capabilities can be narrowed: a directory capability can be cut down to a subdirectory or made read-only. Write-only diagnostic output (`print!`, `eprintln!`, logging) goes to sinks, which are not capabilities and cannot read anything back (D7). The design rejected effect rows, effect variables and implicit capability resolution in favor of plain values (D7, "Rejected").

== Deterministic concurrency through the simulator

Concurrency is structured (D8). Tasks are spawned into scopes from the `Tasks` capability and cannot outlive them; the first failure cancels the siblings; cancellation is level-triggered with deadlines that every wait in a scope shares. Because time, scheduling, files and the network are all capabilities, the same program can run against a simulated root, `Root::sim(seed)`, in which the seed decides the schedule and virtual time advances only when every task is blocked (D15). A simulated test run started through the driver writes a replay manifest that records the seed, the content hashes of the sources, a semantic build identity and the injected inputs; replay refuses to run against a mismatched identity (D15). Running the test binary directly records no manifest.

== Contracts

Contracts use toolchain attributes, `#[requires(..)]` and `#[ensures(..)]`, plus loop `invariant!(..)` (D10). In v0, debug and test builds evaluate _checked_ clauses, those built from operators and the standard library's accessors. _Sampled_ clauses, such as one with a quantifier or a call to a user function, run only in test builds, each under a step budget (D10; #link("design/reference.md")[`design/reference.md`]). Contracts also drive property tests, and release builds erase them. Proof of contracts is planned for v1 (the tier table in #link("design/01-design.md")[`design/01-design.md`]); v0 claims no proofs.

== Admission: a checked dialect of Rust

Tide v0 is a dialect, not a separate compiler (D18). Every Tide program is a Rust 2024 crate that a pinned, unmodified nightly rustc compiles against Tide's standard library, which the driver passes as `std`, so the real standard library's paths cannot be named. Tide's checker runs inside rustc as a driver and gates every crate before rustc writes any artifact for it. It owns the subset, authority, the `Pure` trait (a marker for types from which no capability, synchronization handle or foreign handle is reachable), contracts, the TD diagnostics (the driver's own error codes, such as `TD0206`) and the trust diff, and it enforces an admission allowlist over `core`, `alloc` and std's inherent methods. Because the checker is rustc itself, with rustc's own name resolution, no separate argument is needed that Tide and rustc read a program the same way.

== A reviewable trust surface

The compiler computes each public item's signature and declared authority and lists trust holes, such as every site an `unsafe` grant admits (D11). The *trust diff* compares this surface between two revisions. In v0 it shows the declared authority of exported items only: which exported functions, methods and types gained or lost a capability. It does not show exercised authority, effects, or narrowing inside a body. We describe it as a summary of what a change may do. §4.2 reports a pilot that found no evidence it helps reviewers catch more.

= Implementation

The v0 production code is one Cargo workspace under `crates/`, built with the nightly that `rust-toolchain.toml` pins (#link("AGENTS.md")[AGENTS.md], "Verification").

- *`tide-admit`*, the admission driver, is a `rustc_private` driver on the pinned nightly. It owns every rustc invocation and adds no user-controlled `--extern`, `-L`, `--sysroot`, `--cfg` or semantic `-C` flags (D18). It checks resolved uses before rustc's analysis, then a backstop checks the instances that can run. Today it admits a single crate, or one test suite paired with one library, against Tide's std (#link("demo/README.md")[`demo/README.md`], "What runs today, and what is planned"). It also runs tests (`--tide-run-test`), replays manifests (`--replay`) and prints the trust diff (`--tide-diff`). The planned `tide` command (`tide test`, `tide sim`, `tide review`) does not exist yet; the driver's modes are its partial precursors.
- *`tide-std`* is Tide's standard library, with its macros in `tide-macros`. It provides the capabilities, structured tasks, channels, `Shared` cells, and the simulator of tasks, channels, `Shared`, files, TCP loopback, the clock, the environment and standard input.
- *The simulator and replay.* In v0 the simulator runs each task on its own OS thread and passes a baton so that one runs at a time (#link("design/03-authority-concurrency.md")[`design/03-authority-concurrency.md`] §9.1). A `#[sim_test]` runs a test over a fixed set of seeds; a run through the driver records a manifest and a trace, and `--replay` checks the identity and reproduces the run.
- *The RUN-3 test sandbox.* The type system alone cannot bound what admitted code reaches, because rustc has open soundness bugs that safe code can exploit to forge a capability (D7; §4.1). Test and simulation runs that the driver starts therefore run as a child process inside a kernel sandbox, Landlock and seccomp on Linux, installed between fork and exec (#link("design/03-authority-concurrency.md")[`design/03-authority-concurrency.md`] RUN-3). The driver prints the level before the child starts: `full`, or `partial, not enforced`, naming what the kernel lacks. A test binary run directly gets no sandbox, and the build itself (rustc and the linker) has none yet (RUN-2) (#link("docs/owner-guide/08-sandbox.md")[`docs/owner-guide/08-sandbox.md`]).
- *`tide-conformance`* checks a conformance corpus: every normative rule in the design chapters carries an ID and is traced to cases that an implementation must pass (D0).

= Evidence

We report the evidence in the order chosen for the alpha: replay and admission first, then the pre-registered trust-diff pilot (E3), then the learning-cost trial (E1), then benchmarks, then case studies. Each subsection states its negatives with its positives.

== Replay and admission

*Replay.* The demonstration in #link("demo/README.md")[`demo/`] builds a small concurrent service through the admission driver, with a patch that introduces a lost update. Its suite's `#[sim_test(seeds = 16)]` runs the same code on the same input under a seeded scheduler for seeds 0 to 15. On the buggy revision 8 seeds pass and 8 fail (seeds 0, 2, 4, 8, 9, 10, 12 and 14), each failing on the `lost hits` assertion; only the schedule differs between seeds. The demo records seed 0 and replays the recording twice: the recording and both replays have 40 events and identical hashes of the event trace and the output trace. The fixed revision passes all 16 seeds. A further step runs the replay under `rust-gdb` with breakpoints in the service, when gdb is available (#link("demo/README.md")[`demo/README.md`], "Run it" and "What the demo does"). `cargo run -p tide-demo` runs it, and the workspace test suite runs it too, with two runs that must fail.

The same demo shows the trust diff on a patch that gives the service's public function a `Net` parameter: `hits::serve` is listed as widened, with `+ metrics: net`. This is the trust diff working as a summary. It is not evidence about whether reviewers need it (§4.2).

Replay is bit for bit on the same machine, for the model the simulator has. It reproduces what the simulator models, not every behavior of the real program (§6).

*Admission.* On 2026-11-15 a probe (`fx-6xi2`; IDs of this form name issues in the project's tracker) showed that the driver admitted rustc's own test case for rust-lang/rust\#25860, an issue open since 2015, and a variant that turns a borrowed `&'a Fs` into a `&'static Fs` with no `unsafe` and no grant. Two things followed: `TD0206` now rejects the function-pointer route of \#25860, and the kernel sandbox for test runs moved into v0 (D7) (#link("docs/owner-guide/03-soundness-bugs.md")[`docs/owner-guide/03-soundness-bugs.md`]).

A red team on 2026-11-17 (`fx-qlek.5`) then tried 48 routes, controls included, against admission and the RUN-3 sandbox (#link("reviews/2026-11-17-red-team-opus.md")[`reviews/2026-11-17-red-team-opus.md`], "Result"). Two forms of one more admission route were admitted: the trait-object overlap of rust-lang/rust\#57893, which gives safe code a generic `transmute`, in a blanket form and in a projection form found by the review of the fix. `TD0402` now rejects both. Two routes stay admitted on purpose: one writes only to standard error, which every program holds as a sink, and one is an inert crate attribute. Every other admission route the record lists is rejected, and every RUN-3 route gets the denial RUN-3 specifies.

In all, admission has closed three admitted routes from two rustc soundness bugs: the function-pointer route of \#25860 and both forms of \#57893. The red team also checked that the known reproductions of other routes (\#25860 through closures and trait objects, \#84591, \#100051) are rejected; that covers those reproductions, not every variant, and the \#100051 family stays open in the design (#link("docs/owner-guide/03-soundness-bugs.md")[`docs/owner-guide/03-soundness-bugs.md`], "The hardest question"). rustc has other open `I-unsound` issues. Tide's claim is therefore exact: no _accidental_ ambient authority. Against a malicious dependency, safety rests on rustc's soundness, on the routes closed so far, and, for test runs the driver starts, on the kernel sandbox.

== E3: a pre-registered negative pilot of the trust diff

E3 asks whether reviewers catch more planted authority-widening changes with the trust diff than with the plain code diff (D0). D0's threshold is 1.5 times as many catches. The protocol was frozen before the run, and the report's first paragraph is the sentence the protocol's §12 requires. We quote it unchanged (#link("trials/e3-pilot/report.md")[`trials/e3-pilot/report.md`]):

#quote(block: true)[In the E3 pilot, Opus and Sol reviewed 12 planted authority-widening changes repeatedly, 6 times per change in each arm. With the trust diff, 66 of 72 review sessions caught the planted widening; with the plain diff, 67 of 72 (ratio 0.99, 95% interval [0.95, 1.00]). This does not meet D0's 1.5x threshold on the point estimate. The interval does not rule out no benefit.]

The ceiling explains the result. The plain arm caught more than two thirds of its plant sessions, and at that rate the gate could not be met whatever the trust arm did: 1.5 × 2S/3 = S, where S is the number of plant sessions per arm, and no arm can catch more than S. The interval is narrow because 11 of the 12 pairs tie at 6 of 6 in both arms; its upper end of 1.00 reflects that ceiling, not evidence about harder changes (report, opening and "Pairs").

Two further observations are described, not tested. Every miss in either arm is on one plant, which keeps a writable directory capability instead of narrowing it with `read_only()`; eleven of its twelve sessions approved it. And the trust arm had 3 false positives, the plain arm none: all three are Sol sessions on one clean change, objecting to write authority on a function that only reads, which v0's trust diff lists as `fs.read, fs.write` as it lists every `&Fs` parameter (report, "What the data shows, and what it cannot").

What the pilot does not test matters as much: changes too large to read in full, human reviewers, and the trust diff as a mechanical record that does not depend on anyone reading it. Its items were changes of 10 to 120 lines to small single-file programs, reviewed by two models in sessions of one prompt, one reply and no tools (report, "Limits"). We draw one conclusion: this pilot gives no evidence that the trust diff helps model reviewers catch planted authority widening. We present the trust diff only as a summary of what a change may do. Work after the alpha is planned on a follow-up pilot with harder cases and on per-operation file authority in the diff.

== E1: the learning cost

E1 asks what a model pays to write Tide instead of Rust (#link("trials/e1-checked/gate-report.md")[`trials/e1-checked/gate-report.md`]). Each arm had 30 tasks with 2 independent replicates, 60 trials per arm. One fresh writer session ran per trial, all `claude-opus-5-5`. The Tide arm received the frozen reference and its companion; the Rust arm received neither. The protocol was frozen on 2026-10-31.

#figure(table(columns: 5, align: left, inset: 4pt, stroke: 0.4pt,
  table.header([*Measure*], [*Tide*], [*Rust*], [*Protocol §8 threshold*], [*Ratio (Tide/Rust)*]),
  [First-check pass F], [56/60], [59/60], [F(Tide) ≥ 0.8 × F(Rust)], [56/59 = 0.949153],
  [Final pass P], [58/60], [58/60], [P(Tide) ≥ P(Rust)], [1],
  [Iterations I, passing trials], [86/58 = 1.482759], [59/58 = 1.017241], [I(Tide) ≤ 1.5 × I(Rust)], [86/59 = 1.457627],
))

All three thresholds are met on the point estimates (report, "Measures and verdict"). The result is parity within the protocol's margins, not a benefit. Rust passed first check more often, final pass tied, and Tide needed 1.457627 times Rust's iterations against a 1.5 limit. The information-only bootstrap interval for the iteration ratio, 1.29 to 1.64, crosses 1.5, so this is not a confidence-bound claim (report, "Information-only uncertainty").

Both arms worked near ceiling. Median sessions took 40.3 seconds for Tide and 30.6 seconds for Rust, so the trial mostly measures rapid, small, single-file solutions; it says little about difficult maintenance or larger programs. Every final failure in both arms was the same literal structure-check slip on one task. Both arms used one writer model; the author declined a second-model run, so this is not evidence of learnability independent of the model (report, "Interpretation, coverage and limitations"). The reviewed failure classification found three first-check failures in the Tide arm where a Rust habit leaked, and no toolchain defects, reference gaps or design gaps among the observed failures (report, "Failure classification").

== Benchmarks against plain Rust

#link("research/36-benchmarks.md")[`research/36-benchmarks.md`] measures what Tide's additions cost against plain Rust on four small programs, each written to isolate one cost, on one shared laptop. It is not D0's E4 or E5 gate run. Its main hedge comes first:

#quote(block: true)[*Stability: almost no series met the tool's criterion.* All 60 series of the main run ran to the 30-round cap with a CV above 1% (from 1.04% to 46.7%), so the run marks every one unstable (§1.6). [...] The medians and ratios below are descriptive observations from these unstable runs, not settled measurements.]

The note reads ties and losses from paired rounds: an arm is slower when it is slower in at least 20 of 30 rounds. Its summary (§1.1):

- *Building.* A cold debug build takes 3.8 to 4.6 times plain Rust's time; a rebuild after a one-line edit takes 5.2 to 6.3 times incremental Rust's, because the driver has no incremental build. Compiler peak memory is 17% to 39% higher. Release binaries load 1.4 to 2.0 times the bytes of plain Rust built with Tide's profile flags.
- *Contracts in release.* "No measurable cost in this run": a tie, 0.99x.
- *Contracts in debug.* The plain debug build checks only the cheap clauses; the program's two sampled sortedness clauses run only in test builds, so these numbers exclude them. At the timed size of 1,200,000, a tie (1.05x). At 200,000, in a separate run outside a quiet window, 1.19x, slower in all 30 rounds: a loss.
- *Contracts in a test build.* In that same separate run at 200,000, the debug test build, which also checks the sampled clauses, took 7.7x the plain Tide debug build and 9.2x Rust's debug build. Its sampled clauses scan a vector on every insert, which the note calls unrepresentative of typical contract cost, and the test build also differs by the test harness and its `cfg(test)` code, which the run does not separate.
- *Capability file I/O.* 1.04x by medians, slower in 26 of 30 rounds, paired median ratio 1.07x: "a small loss, of a few percent".
- *Capability TCP.* 1.18x, slower in all 30 rounds.
- *Arithmetic-heavy tasks.* 2.29x plain release Rust, of which 1.74x comes from Tide's profile flags alone; the run does not separate which flag costs what.
- *Simulator.* One seed under the simulator takes 12.8 times the real run in release builds.

We adopt the note's verdict: "Every comparison above is a loss for Tide or a tie. None is a win." Its other limits include different linkers for the two arms and a Tide std built in each arm's profile, so the debug comparisons also compare an unoptimized std with Rust's optimized one (§3). A fast edit loop remains a design target (D12), not a measurement.

== Case studies

Two case studies rewrite four real bug reports in Tide. Admission catches none of them.

*Eon: one slow close blocks every request* (#link("examples/case-studies/eon-slow-close/README.md")[`examples/case-studies/eon-slow-close/`]). Tide's admission would not have rejected this bug: the buggy loop is ordinary Tide. Eon [17] is a terminal workspace whose supervisor answers every control request on one loop thread. In bug eon-8wh7h, closing a tab stopped its terminals one after another on that thread, and each terminal slow to exit cost a 500 ms stop grace. Eon's own reproduction closed a 14-terminal tab in 6992 ms under IO and CPU stress, and the window's Inspect request timed out at 6011 ms against its 6 s deadline. Eon fixed it by stopping a close's terminals together under one shared deadline. The Tide rewrite models the loop's structure, not Eon's code. A `#[sim_test]` sends an Inspect every 250 ms during the close and requires an answer within 6 s. It fails on the buggy revision and passes on the fixed one, in which the bound is a scope deadline that every wait inside shares. Rust can write the same test. What Tide adds is that the test runs in virtual time, deterministically, with no fake clock written for it: the 7 s close takes milliseconds, where Eon's real-time red/green test fails at 3.78 s. Async Rust gets the same from tokio's paused clock; Eon's thread-based supervisor would need a clock it is handed. A third file encodes the still-open eon-pimct, a terminal start that waits on the loop, as a failing test.

*hyper: the pool reaches for an ambient runtime* (#link("examples/case-studies/hyper-ambient-runtime/README.md")[`examples/case-studies/hyper-ambient-runtime/`]). No Tide admission code in this case study catches either reported failure. hyper 0.14's client pool [18] runs its idle-connection reaper on whichever tokio runtime is ambient. One report is a panic in tokio's timer; in the rewrite that timer gets only rustc's `E0433`, because Tide admits no external crate. The other, hyper\#2808, is a reaper that never starts. Tide does not prevent it: a pool whose optional clock is left unset is admitted, and the reaping test fails on it. What Tide gives is the fix's shape: a pool can start its reaper only in a scope its caller hands it and read time only from a `Clock` it is handed. The rewritten pool's `#[sim_test]` shows it closing idle connections on each of 20 seeds.

Both studies point the same way. Tide's contribution to these bugs is a cheap, deterministic test of a property someone had to think of, plus a structure that makes the fix's bound explicit. It does not find the bugs.

= Related work

*Rust.* Tide depends on Rust [12] and is meant to complement it. Every Tide program is a valid Rust 2024 crate compiled by an unmodified rustc (D18), and Tide's memory safety is Rust's. Tide is a restricted subset with its own standard library, not a fork or a replacement. Its capability claims inherit rustc's soundness bugs (§4.1).

*Faster Rust toolchains.* lust [13] and Blaze aim at the edit loop, which Tide has not yet built (D12) (#link("research/28-positioning-and-name.md")[`research/28-positioning-and-name.md`] §2). lust checks a subset of valid Rust with its own front end, keeping rustc as the oracle; it has no authority bounds or replay. Blaze is a second Rust compiler with a warm daemon; it has partial overlap with Tide's thesis through attenuable `bstd` capabilities and a simulator, but does not admit a whole program against a capability standard library. This alpha does not benchmark either.

*Capabilities and effects.* Object-capability languages and libraries, among them Joe-E, Pony, Wyvern, Austral and the `cap-std` crate, and runtime permission models such as Deno's and WASI's, are surveyed in #link("research/07-effects-capabilities.md")[`research/07-effects-capabilities.md`]. Effect systems such as Koka and Effekt track effects in types; Tide's design rejected effect rows, effect variables and, in v1, user-defined handlers, in favor of plain capability values (D7). Effect [14], a TypeScript library, is the closest prior art for requirements in types: its `Effect<A, E, R>` type carries requirements, services are provided through layers, and fibers' lifetimes are tied to their parent. Its `R` is a convention, though: nothing stops a dependency from calling `fetch` or reading the clock directly. Tide enforces the equivalent at admission, and by design in dependencies too (v0 admits a single crate), and its structured tasks forbid the detached fibers that Effect's `forkDaemon` creates (D8).

*Deterministic simulation.* FoundationDB-style simulation testing, TigerBeetle's VOPR, and Rust tools such as loom, Shuttle and madsim test concurrent code under controlled schedules (#link("research/16-deterministic-simulation.md")[`research/16-deterministic-simulation.md`]). rr, the closest tool to Tide's replay, records a run of an unmodified Linux program and replays it deterministically under gdb; it needs the CPU's hardware performance counters and finds a failing run by recording runs, where Tide fixes the schedule from a seed and searches seeds, and a seed is a number that reruns on the same source (#link("docs/debugging.md")[`docs/debugging.md`], "rr compared"). Library-level simulation in Rust is limited by dependencies that reach ambient time, randomness and threads (#link("design/01-design.md")[`design/01-design.md`], "Evidence That Shaped The Design"). Tide's difference is that admission makes those reaches impossible, so simulation covers the whole program wherever the simulator has a model for every effect reached. Rivet [15], a Rust-core actor runtime whose workflows resume after a crash by skipping finished steps, does not record or replay nondeterminism; it is complementary, and with Temporal it is prior art for any future resume-after-crash feature in Tide. Independently, plusthree, an MIT-licensed Rust port of XState v6 statecharts, keeps its `no_std` kernel free of a clock: the host fires timers, so tests can run timers instantly [19].

*Verification.* Verus, Prusti, Creusot and Kani verify Rust programs (#link("research/04-verification.md")[`research/04-verification.md`]). Tide borrows their contract names and spellings (D10) but proves nothing in v0. Bend 2 [16] targets proved, massively parallel pure code; Tide plans to call it for such kernels rather than compete (D16, Later tier). As context, Anthropic and Theorem reported in October 2026 on models carrying out formal verification at a much larger scale than before [11]. That work is independent of Tide; it is one sign that agents can do large amounts of checked work, and Tide is not formally verified.

= Limitations

- *Linux on x86\_64 only.* The driver links and runs tests only on x86\_64 Linux. Elsewhere it builds with the sandbox compiled out and its run modes are unavailable (#link("docs/owner-guide/08-sandbox.md")[`docs/owner-guide/08-sandbox.md`]).
- *A single crate.* The driver admits one crate, or one suite paired with one library. Packages and dependencies are planned. Until they exist, the guarantees cover a closed world that grows weaker as programs reach more of the outside world, the review's weakness W1 (#link("reviews/2026-11-15-achilles-opus.md")[`reviews/2026-11-15-achilles-opus.md`]).
- *Replay reproduces the model, not the program* (W7). The simulator does not model CPU time, memory exhaustion, stack depth, parallel speed-up, signals, real child processes, or kernel behavior beyond its models. Simulated HTTP, randomness and processes, fault injection, crashable nodes, and `tide sim` for whole programs are planned, not built (#link("docs/owner-guide/10-v0-and-planned.md")[`docs/owner-guide/10-v0-and-planned.md`]).
- *Capability safety rests on rustc's soundness* (W3, `fx-6xi2`). Three admitted routes are closed; rustc has other open `I-unsound` issues; until a toolchain pin bump is audited, `core::*` globs such as `core::mem::*` admit whatever a later rustc adds under them; and the red team's record lists what it tried and what it left open (§4.1).
- *The sandbox is partial.* The level `full` requires a Linux kernel with Landlock ABI 6 or later and seccomp. RUN-3 does not confine metadata reads such as `stat` and `access`. A test binary run directly gets no sandbox, the build has none yet (RUN-2), and deployed programs get kernel enforcement only in v1 (D7).
- *E1 is near ceiling and uses one writer model.* Both arms passed almost every task, on small single-file programs, with `claude-opus-5-5` alone.
- *E3 missed its threshold.* The pilot did not meet D0's 1.5x threshold, and its plain arm left no room to show a benefit.
- *The benchmarks are unstable and show only costs*, on four small programs on one shared machine (§4.4).
- *The `tide` command is planned.* The driver's flags are its precursors; typed holes and machine-readable fix plans (D14) are not delivered, which the E1 report counts against the Tide arm.

= Roadmap

The tier table in #link("design/01-design.md")[`design/01-design.md`] ("Tiers") owns what comes next. Planned for v1: proofs of contracts (`prove` mode), exercised-authority tracking, the evented runtime, direct import of Rust crates behind capability wrappers, production record and replay (D17), and kernel enforcement of authority for deployed binaries (D7). Later: the Bend tandem (D16), automatic parallelization, and modeled child processes and signals. Planned evidence includes a follow-up to E3 on harder cases, E2 (productivity on multi-file tasks), and E4 to E6 (D0). The fast edit loop of D12 needs Tide's own incremental front end, which is not built.

= How it was built

Tide is agent-produced under a written governance (#link("GOVERNANCE.md")[GOVERNANCE.md]). Its primary rule: "No production Tide code should originate from direct human coding." Human-written production code is forbidden by default and must be explicitly marked if it occurs as an exception. The owner defines architecture and specifications, reviews, and steers; agents implement compiler components, write tooling and tests, and apply fixes.

No change lands on `main` without an AI supervisor pass. Review runs in sequence, each reviewer read-only on a frozen candidate: first a Codex pass (`gpt-6.1-sol`, reasoning effort high), whose findings the author fixes or rebuts until the supervisor reports the tree clean, then an Opus review on that tree. When the provider refuses the Codex pass, an Opus stand-in review takes its place and records the refusal. Planning-only changes (bead filings) skip the Codex pass, and a protocol-import update that only regenerates `AGENTS.md` and `.agent-protocols.json` from a pinned source commit skips both reviews when its source-backed check passes. The human maintainer remains final merge authority. Each review is recorded in the pull request's discussion or, for a direct commit, as a file under #link("reviews/")[`reviews/`]. Work is tracked as beads in a committed ledger, and the design is a decision record whose language decisions are owned by #link("design/01-design.md")[`design/01-design.md`].

This is why we call Tide AI-welcome: not because agents wrote it, but because what they wrote is checkable. Changes are agent-written unless explicitly marked otherwise, reviewed under that merge rule, and backed by proof that can be rerun: a demo, a conformance corpus, and trial reports with their supporting records. Not every raw record is in the repository: E1's raw writer and broker streams and its grading logs are kept outside it, and its report says so. The same habit shaped this paper. Both trials ran under protocols frozen before the first session, and both reports, the negative one included, are in the repository.

#heading(level: 1, numbering: none)[References]

In-repository reports and design records (relative links):

1. Tide decision record: #link("design/01-design.md")[`design/01-design.md`]; first principles: #link("design/00-first-principles.md")[`design/00-first-principles.md`]; core reference: #link("design/reference.md")[`design/reference.md`]; authority and concurrency chapter: #link("design/03-authority-concurrency.md")[`design/03-authority-concurrency.md`].
2. Replay and trust-diff demonstration: #link("demo/README.md")[`demo/README.md`].
3. Red team of admission and the RUN-3 sandbox: #link("reviews/2026-11-17-red-team-opus.md")[`reviews/2026-11-17-red-team-opus.md`].
4. rustc's soundness bugs and what Tide closed: #link("docs/owner-guide/03-soundness-bugs.md")[`docs/owner-guide/03-soundness-bugs.md`]; the RUN-3 sandbox: #link("docs/owner-guide/08-sandbox.md")[`docs/owner-guide/08-sandbox.md`].
5. E3 pilot report: #link("trials/e3-pilot/report.md")[`trials/e3-pilot/report.md`].
6. E1 gate report: #link("trials/e1-checked/gate-report.md")[`trials/e1-checked/gate-report.md`].
7. Benchmarks against plain Rust: #link("research/36-benchmarks.md")[`research/36-benchmarks.md`].
8. Case studies: #link("examples/case-studies/eon-slow-close/README.md")[`examples/case-studies/eon-slow-close/README.md`] and #link("examples/case-studies/hyper-ambient-runtime/README.md")[`examples/case-studies/hyper-ambient-runtime/README.md`].
9. Adversarial review (weaknesses W1 to W8): #link("reviews/2026-11-15-achilles-opus.md")[`reviews/2026-11-15-achilles-opus.md`]; round-1 agent-usability review: #link("reviews/round1-agent-usability.md")[`reviews/round1-agent-usability.md`].
10. Research notes: #link("research/04-verification.md")[`research/04-verification.md`], #link("research/05-llm-code-empirics.md")[`research/05-llm-code-empirics.md`], #link("research/07-effects-capabilities.md")[`research/07-effects-capabilities.md`], #link("research/16-deterministic-simulation.md")[`research/16-deterministic-simulation.md`], #link("research/28-positioning-and-name.md")[`research/28-positioning-and-name.md`].

External:

11. Anthropic and Theorem, "LLMs are making formal software verification feasible at a much larger scale", 2026-10-22. #link("https://www.anthropic.com/research/formal-verification")
12. The Rust programming language and rustc. #link("https://www.rust-lang.org/"); soundness issues rust-lang/rust\#25860 and \#57893: #link("https://github.com/rust-lang/rust/issues/25860"), #link("https://github.com/rust-lang/rust/issues/57893").
13. lust: #link("https://github.com/t3dotgg/lust-lang").
14. Effect: #link("https://github.com/Effect-TS/effect").
15. Rivet: #link("https://github.com/rivet-dev/rivet").
16. Bend 2: #link("https://github.com/bendlang/bend").
17. Eon: #link("https://github.com/Yazelix/eon").
18. hyper: #link("https://github.com/hyperium/hyper"); issue \#2808: #link("https://github.com/hyperium/hyper/issues/2808").
19. plusthree: #link("https://tangled.org/rockorager.dev/plusthree").
