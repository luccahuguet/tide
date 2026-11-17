# Answers to the ten hardest questions

One page for funding calls, grant forms, and critics. Each answer is what
`main` supports on 2026-11-17; the chapter in brackets has the detail and the
commands. Say "planned" whenever the answer says planned.

**1. "What is Tide, in one sentence?"**
A capability-safe, replayable subset of Rust: a checker in front of the
standard Rust compiler that admits only code whose authority you can read in
its signatures, whose concurrency bugs replay from a seed, and whose changes
come with a diff of what the code may now do. Every Tide program is a valid
Rust program. [1, 2]

**2. "Isn't 'no ambient authority' just a lint, or cap-std?"**
No. A lint forbids listed calls; Tide's std doesn't have the ambient calls at
all, and from Rust's lower layers the checker admits only an allowlist.
cap-std is opt-in, and every other line of a Rust program can still call
`std::fs`. The precise claim is **no accidental ambient authority** for the
code the checker admits. [1]

**3. "rustc has soundness bugs. Can't malicious code forge a capability?"**
Through a compiler bug, in principle yes, and we say so. We probed it:
rust#25860 got through (fx-6xi2), and `TD0206` now closes that route. A red
team of over 40 routes (fx-qlek.5) found two forms of one more route,
rust#57893, both now closed by `TD0402`; every other escape route it lists
was rejected. Other soundness bugs may remain. Test runs the driver starts
(`--tide-run-test`, `--replay`) are inside a Linux kernel sandbox that
doesn't depend on the compiler; a test binary run directly gets none.
Deployed programs get kernel enforcement in v1. [3, 8]

**4. "Why a new language? Rust plus cackle, Clippy, turmoil and Antithesis
covers it."**
Each covers one piece. cackle is the closest: it lists which kinds of API
(files, network, processes) each dependency may use, per package, and checks
the build against that list. Tide puts authority in each function's
signature instead, so it is read and checked per function while compiling.
Library simulators (turmoil, madsim) work only while every line, dependencies included, uses their simulated APIs; Tide's
checker rejects the ambient call, the bare thread, the unsimulated clock, so
the checker enforces that discipline. Antithesis is different: it
runs whole programs deterministically in a hypervisor, with no code
discipline, as a commercial service. Tide's case against it isn't
determinism; it's the combination: authority readable in signatures, local
modeled tests in the edit loop, and a trust artifact for review. And it's not
really a new language: it's Rust with a
checked subset and a different std, so models' and people's Rust knowledge
carries over. [1, 5]

**5. "Isn't replay just rr?"**
rr records one real run. Tide searches many seeded simulated runs for a
failure, then replays the failing one exactly from a small manifest, and
refuses if the code changed. The demo finds a lost update in 8 of 16 seeds
and replays it with byte-identical traces. Limits: tests only, same machine,
and only what the simulator models. No CPU time, no real kernel behavior.
Production record and replay is v1. [5, 6]

**6. "A widened signature is visible in the code diff anyway. What does the
trust diff add?"**
It finds widening wherever it hides (a field three files away) and states it
in a line a merge gate can act on. v0's diff shows declared authority and
`unsafe` holes only; it can't see new use of authority already held, less
attenuation, or data printed out. Exercised authority is v1. Whether it
helps reviewers is E3's question. [7, 9]

**7. "E1 shows parity with Rust, not a benefit. Why switch?"**
Right: E1 is the cost line. Models write Tide about as reliably as Rust (first
check 56/60 vs 59/60, final 58/60 each, 1.46× the iterations against a 1.5×
limit), on small tasks with one model. The benefit test is E3: do reviewers
catch planted authority widening 1.5× more often with the trust diff? Its
protocol is in review (fx-1vwf.1), half its plants use mechanisms the v0
diff isn't expected to show, by design, and the result is published either way. [9]

**8. "Your world is tiny: one crate, no dependencies, no TLS, no HTTP. Real
programs live outside it."**
True today, and the strongest critique. The answer is graded claims:
authority and replay you can read for the Tide-written part, with every hole
listed. Imported Rust crates (v1, not built) become listed trust holes, unmodeled
runs are marked not replayable, and the kernel sandbox, whatever the
language, bounds them in driver-started test runs and, from v1, in deployed
programs. The effect surface (TLS, processes, Unix sockets) is the next
priority question. [10]

**9. "You depend on a pinned nightly's compiler internals. That's fragile."**
It's a real, chosen cost. The driver links seventeen of rustc's internal
libraries on one pinned nightly, and the pin hasn't moved yet. In exchange,
Tide reuses rustc's type checker, borrow checker and code generator instead
of building its own. 389 conformance cases check that stable rustc and the driver
give the expected verdicts. There's no CI yet: an agent runs them, usually
when a code change lands. A few cases await rules not built yet. A rehearsal pin bump is the recommended next check. [2]

**10. "What actually runs today?"**
`tide-admit` (the checker), Tide's std with files, TCP, clock, environment,
stdin and structured tasks, the simulator and replay for tests, the trust
artifact and its diff, the test sandbox, and a demo that shows all of it in
one command. Not yet: the `tide` command, packages, HTTP, processes,
randomness, fault injection, the fast incremental checker, proofs. [10]

<!-- TODO(tide-lead, fx-1vwf.1): update answer 7 when the E3 protocol lands (E3's status or
result). -->
