# Answers to the ten hardest questions

One page for funding calls, grant forms, and critics. Each answer is what
`main` supports on 2026-11-19; the chapter in brackets has the detail and the
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
helps reviewers was E3's question, and the pilot found no evidence that it
does (question 7), so we present it only as a summary of what a change may
do. [7, 9]

**7. "E1 shows parity with Rust, not a benefit. Why switch?"**
Right: E1 is the cost line. Models write Tide about as reliably as Rust (first
check 56/60 vs 59/60, final 58/60 each, 1.46× the iterations against a 1.5×
limit), on small tasks with one model. E3 tested a benefit of the trust
diff, under a protocol frozen before any plant existed, and it was negative.
As published: "In the E3 pilot, Opus and Sol reviewed 12 planted
authority-widening changes repeatedly, 6 times per change in each arm. With
the trust diff, 66 of 72 review sessions caught the planted widening; with
the plain diff, 67 of 72 (ratio 0.99, 95% interval [0.95, 1.00]). This does
not meet D0's 1.5x threshold on the point estimate. The interval does not
rule out no benefit." Plain review already caught almost every plant, so the
bar couldn't be met; changes too large to read in full were not tested. The
reasons to switch are replay and admission: a seeded test finds a race and
replays it exactly, and admission rejects ambient authority. On real
malicious code: Tide's std had a gap, our screen of all 87 malicious-crate
advisories (0 caught) exposed it, we fixed it, and re-tested under a new
registration; the faithful chrono_anchor port is rejected, as it was in
the screen, and its capability-passing version is now admitted. Every check is consistent with that
registration being anchored before the fix landed, and independent proof of
that order is not established. The re-test ran admission and type-checking
only; no port was executed. Not a first-time catch and not a rate; the fix
also reversed an earlier, deliberate removal of the same conversion
(chapter 9 and research/49 give that history). [9]

**8. "Your world is tiny: one crate, no dependencies, no TLS, no HTTP. Real
programs live outside it."**
Mostly true today, and the strongest critique. The std now has one HTTP
GET with TLS, name lookup and starting processes, but the driver still
admits one crate with no dependencies, and there is no UDP and no Unix
sockets. The answer is graded claims:
authority and replay you can read for the Tide-written part, with every hole
listed. Imported Rust crates (v1, not built) become listed trust holes, unmodeled
runs are marked not replayable, and the kernel sandbox, whatever the
language, bounds them in driver-started test runs and, from v1, in deployed
programs. The rest of the effect surface (UDP, Unix sockets, more of HTTP)
is the next priority question. [10]

**9. "You depend on a pinned nightly's compiler internals. That's fragile."**
It's a real, chosen cost. The driver links seventeen of rustc's internal
libraries on one pinned nightly, and the pin hasn't moved yet. In exchange,
Tide reuses rustc's type checker, borrow checker and code generator instead
of building its own. 392 conformance cases check that stable rustc and the
driver give the expected verdicts; in the latest recorded run, 389 pass and
3 await rules not built yet (`conformance/run-record.txt`). There's no CI
yet: an agent runs them, usually when a code change lands. A rehearsal pin bump is the recommended next check. [2]

**10. "What actually runs today?"**
`tide-admit` (the checker), Tide's std with files, TCP, one HTTP GET, name
lookup, starting processes, clock, environment, stdin and structured tasks,
the simulator and replay for tests, the trust artifact and its diff, the
test sandbox, and a demo that shows all of it in one command. Not yet: the
`tide` command, packages, UDP, randomness, fault injection, the fast
incremental checker, proofs. [10]
