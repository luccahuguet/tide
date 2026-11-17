Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 0d6b566

# fx-m4ip, Opus r2 finder re-check

Scope: my own findings B1-B8, N1-N16 and the fresh-reader notes. I checked them against
`git diff 919107b 0d6b566`, plus the new FX0402 and red-team text and the new chapter-3 command,
which I checked against `main`. `git diff main...0d6b566 | sha256sum` =
0b40871c33dd3c5ac76a924070efdf0ea6b45c6d9b5c8b3d285fe9196d8514ba, which matches the brief. HEAD is
0d6b566 and its parent is fe73625 on `main`. Everything was read-only: no builds, no subagents.

I also checked that every excerpt anchor still holds after the rebase. Since 15e0880, `main`
changed only `policy.rs` (hunks at line 345 and below) and `README.md`. The policy.rs ranges 64-72
and 92-135 still match line for line. sandbox.rs, reify.rs, hits.rs, run.sh, the patches,
cap_ambient.rs, closure_fnptr_capability.rs, reference.md (12 `(planned)` labels) and the fx-p44q
audit are unchanged. The builder's new logs (`runs/r2-*.log`) show every changed or new command run
on the rebased tree, with outputs matching the guide.

## Verdict

**request_changes.** Exactly one item blocks: a sentence the N5 fix introduced (new finding X1
below), which misstates the design. All eight original blocking findings are fixed at the strength
of their sources. All sixteen non-blocking findings and the fresh-reader stalls are resolved. X1 is
a text-only fix, so under AP-PROOF-001 it needs no further re-check once applied. With X1 applied,
I approve.

## B1, the RUN-3 sandbox's scope: fixed

Chapter 1 (lines 191-193), chapter 3's item 2 and its one-liner, and answers 3 all now say "test
runs the driver starts (`--tide-run-test`, `--replay`)" and that "a test binary run directly gets
none". That matches the README ("covers the runs `tide-admit` starts, in part") and its sentence on
directly run test binaries.

## B2, "closes the known routes": fixed

fx-qlek.5 has landed (dc699ce, closed in 95be653), so FX0402 is on `main`. Chapter 1 now says Tide
"has closed two known compiler routes … others may remain", which matches README Status on `main`.
Chapter 3's item 1 names both routes and their codes.

## B3, the closure workaround and rust#100051: fixed

`03-soundness-bugs.md:38-40` says the closure "makes rustc check the call against the function's
signature", and that the rust#100051 family "stays open even there; design/02 §5.4". Item 1 now
says the red team checked the *known reproductions* and that "the #100051 family stays open in the
design". Both match design/02 §5.4 and `reify.rs:5-6`.

## B4, help text the driver never prints: fixed

Chapter 1 now says the help text is in the table, but the one-line error format leaves it out.
That matches `invocation.rs:332`, and the Run it output is as quoted. The take-away now says the
error "names the rule (`FX0501`)".

## B5, stale red-team numbers: fixed

The guide now says "over 40 escape routes" and "two forms of one more admitted route,
rust#57893", and that every other route the record lists was "rejected or denied". It names the two
routes admitted on purpose (the stderr sink and the inert crate attribute), cites
`reviews/2026-11-17-red-team-opus.md`, and gives a landing date of 2026-11-17. All of this matches
the record on `main` (48 routes counting controls, routes 6 and 6b, routes 22 and 24) and the close
reason (45 routes).

Nit: answers 3 says "every other escape route it lists was rejected". Chapter 3's "rejected or
denied, two admitted on purpose" is more exact, and copying it into answers 3 would be better. I
proposed nearly this wording myself, so it is not blocking.

## B6, the E1 overreach: fixed

`09-experiments.md:46-47` now says "E1 suggests models can write Tide at little extra cost, on
small tasks with one model". That is as strong as the gate report.

## B7, the unsourced "value is in the logic" point: fixed

Point 3 is now framed as "Tide's bet, a hypothesis". It concedes Achilles W1's Eon example
(lifecycle races need processes and terminals, which v0 can't express) and points to point 4 for
priority, which matches W1's evidence. Point 2 now puts RUN-3's coverage of imported crates in v1
and says "Today there is no imported code".

## B8, conformance "on every change": fixed

Answers 9 now says "389 conformance cases check that stable rustc and the driver give the expected
verdicts. There's no CI yet: an agent runs them, usually when a code change lands." I counted 389
case files on `main` (372 + 17). Chapter 2 keeps the recorded fx-ip4x run (369 PASS, 3 AWAITS, 372)
and adds the 17 red-team cases, "both suites passed at its landing". The fx-qlek.5 close reason
says "suites pass on 156fcc7", which was the frozen candidate landed as dc699ce, so that sentence
is supported.

## New FX0402 text and the chapter-3 command: they match `main`

- The command runs `tide-admit` on `conformance/cases/memory/escape_dyn_overlap_transmute.rs`,
  which exists on `main` with `//~ FX0402` at lines 11 and 50. Line 50 is inside `pub mod
  projection` (line 38).
- The quoted message matches `walk/traits.rs:145` word for word, and positions 11:1 and 50:5
  match `runs/r2-examples.log` (exit 1, two errors).
- The "What to look at" note is accurate: the suggested fix shows because it is part of the
  message itself.
- RUN-3: `runs/r2-run3.log` shows 2 tests passed with 104 filtered out, as quoted. The second
  test's description (exec, fork, io_uring, ptrace, before-`main` constructors) matches record
  routes 26-41 and 45.
- Nit (not blocking): the chapter 3 heading and the chapter 10 table say "FX0402 closes
  rust#57893", while README Status on `main` says "the trait-object route of rust-lang/rust#57893"
  and the body says "both forms". "Closes the two known forms of rust#57893" would match the README
  exactly. The heading also needs care because fx-qlek.13, still open, asks whether the projection
  form is new upstream.

## Non-blocking findings N1-N16: all resolved

- **N1:** `runs/r2-examples.log` runs both `unsafe_hole` commands with `--crate-name=unsafe_hole`.
  It writes `unsafe_hole.interface.json`, and the `cat` of it matches chapter 7 byte for byte.
  Recording the runs on the bead is still the coordinator's step.
- **N2:** the index now says the review checked those forms in nushell 0.113.1. That is what I
  did, and the claim is no longer phrased as a guarantee.
- **N3:** the execve sentence is added, and matches `sandbox.rs:552-556`.
- **N4:** `EPERM` "operation not permitted" and `EACCES` "permission denied" are now distinguished,
  in the text and in the diagram.
- **N5:** the bullet is added, but it introduces X1 below.
- **N6:** "that nothing bypasses the simulator", plus the seeds caveat.
- **N7:** "barring compiler soundness bugs (chapter 3)".
- **N8:** the diagram is labelled a sketch, not copied from traces.
- **N9:** "four marked lines". `runs/r2-ambient.log` shows the same six errors after the edit.
- **N10:** 13 lines, in chapter 3 and the index.
- **N11:** "On 2026-11-15, after the probe", which matches D7.
- **N12:** cackle is described as per-package API-kind lists checked against the build, against
  Tide's per-function signatures. Fair, and not overstated.
- **N13:** W5 is paraphrased as "cheapest test is a rehearsal bump; mitigation is bumping
  monthly", which matches W5.
- **N14:** "Stable rustc, compiling against a stub of Tide's std … not the pinned nightly".
- **N15:** answers 8 now ties the sandbox claim to driver-started test runs and puts deployed
  programs in v1. Chapter 10's point 2 is fixed as in B7.
- **N16:** the echo rule is explained and matches D8's "Scope results".

## Fresh-reader stalls: gone

Each of these is now addressed:
- "nightly" is in the glossary, and `.rlib` is explained.
- `..deny(..)` is explained.
- Chapter 2's case-file sentence is rewritten.
- Achilles is introduced as the 2026-11-15 adversarial review.
- D12's "A, then B" is explained, matching D12's text.
- "generic" is defined.
- The #57893 mechanism is marked as skippable.
- `Ok(())` is explained, and LOCK-4 and SINK-5 are named as design rules.
- Chapter 10's audit step is cut to its first 60 lines (10 minutes).

The builder chose to keep chapter 4's speed paragraph. That's fine, since it answers the hardest
question. Chapter 3 grew by one command and a short output, so it is still about 25 minutes, the
same as before. No new term goes undefined.

## New finding X1 (blocking, text only): "A formatter holds none" contradicts the design

- **Where:** `07-trust-diff.md:100-103`: "`opaque(..)` marks a type that hides what it carries, so
  the artifact lists it as possible authority, to be safe. A formatter holds none; this is the
  conservative listing …"
- **Problem:** design/03 §4.1 says `opaque(..)` comes from a `dyn Trait` whose trait lacks a `Pure`
  supertrait. A `Formatter` wraps a `&mut dyn fmt::Write`, which is whatever the caller is
  formatting into. With `write!(stream, "{x:?}")` on a `TcpStream`, the `Debug::fmt` call can write
  through `f` to the network. So "holds none" is false in general. It also teaches the owner that
  `opaque` entries are noise, when they are exactly the places the declared-authority computation
  can't see through. A critic who knows Rust's `fmt` machinery can falsify the sentence.
- **Minimum fix:** replace "A formatter holds none; this is the conservative listing …" with
  "A formatter wraps whatever it is writing to, which the artifact can't see, so it lists it as
  opaque, the conservative listing the repository README.md Status describes." Optionally, also
  join the bullet to the list that follows it: a blank line currently separates it from the next
  bullet.

DONE
