Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 4dc26c7 (stage 2 re-check)

## Scope

This re-check covers only my B1 and the non-blocking fixes the builder took (N1, N2, N3, N5, N7a). I read "Stage 2 round 2 fixes" in target/work/report.md and `git diff b586c19 4dc26c7` (5 files). I wrote nothing except this file and did no Rust builds. For the tamper checks, I extracted `crates/tide-std/src/rand.rs` at 4dc26c7 into the scratchpad with `git archive`. On each tampered copy I ran a `python3 -I` copy of the guard's source assertions (`build.rs:4288-4293`, string for string).

## B1. Flags 0 had no check that could fail: resolved

**What changed:**
- One constant, `FLAGS = GetRandomFlags::empty()` (`rand.rs:225`), now goes to both places:
  - the seam, as `seam::call(dest, FLAGS)`;
  - the single system call, `rustix::rand::getrandom(dest, FLAGS)`.
- The seam records `(len, flags)` for each request.
- In `real_draws_read_getrandom_through_the_seam`, every request goes through `lens()`, which asserts `flags.is_empty()`.
- The guard now pins three things:
  - the call site text;
  - the constant's exact definition;
  - exactly 3 occurrences of `FLAGS`.

**Tamper results** (python copy of the guard, scratch copy only):

| Tamper | Guard result |
| --- | --- |
| none (baseline) | PASS |
| constant changed to `INSECURE` | fails (`const`); the builder also reports the seam test failing on it in a git-archive copy |
| call passes a literal `INSECURE` | fails (`callsite`, `FLAGS==3`) |
| call passes `FLAGS \| INSECURE` | fails (`callsite`) |
| seam gets `FLAGS`, call gets a second constant | fails (`callsite`, `FLAGS==3`) |
| a second literal `rustix::rand::getrandom(…, INSECURE)` call | fails (`count`) |

So the check you asked about holds: a call site that passes flags other than the shared constant fails at least one check. The same is true when the seam and the system call are given different flags. The RAND-4 row now names checks that can fail.

**Remaining limit (non-blocking): the guard is textual.** Two edits still pass it:
- An aliased call (`use rustix::rand as r; r::getrandom(dest, INSECURE)`), but only when the exact text `rustix::rand::getrandom(dest, FLAGS)` is also kept in a comment. That is deliberate evasion, not an ordinary edit.
- A second OS call through a different API, such as `rustix::rand::getrandom_uninit(…, INSECURE)`. This is the same "one OS call site" spelling limit the guard had before this round.

Neither is an ordinary refactor that would slip through by accident. If you want the guard tighter, one cheap option is to also pin the number of occurrences of `getrandom` (or of `rustix::`) in `rand.rs`.

## The non-blocking fixes: none introduces a defect

- **N1, resolved.** The new block in `simulated_values_follow_the_stated_methods` uses seed 11 and draws in a fixed order from `reference(11, 64)`:
  - `random_range(-5i8..5)` equals `-5 + below(10)`. I checked this against the offset mapping: `word(-5)` is 123, and `from_word(123 + r)` is `-5 + r`, so the expectation is right.
  - The shuffle that follows equals Fisher-Yates from the end, using the next reference words.

  Now a forward shuffle, a different word order, or a different signed mapping would each fail. 64 reference words is far more than the call can take.
- **N2, resolved.** `random_bool(0.0)` is false, `random_bool(1.0)` is true, and each records `"words":1`. Reordering the code to `p == 1.0 || next() < …` now fails the test.
- **N3, resolved.** The RAND-4 row now says the guard reads only `rand.rs`, the same limit as the NET-HTTP-4 and PROC-3 guards.
- **N5, resolved.** The build test no longer duplicates the expected line.
  - The case's own `assert_eq!(seen, EXPECTED)` panics on a mismatch, and `checked` then fails on the non-zero exit, so nothing is lost.
  - The simulated values are still compared to `real`.
- **N7a, resolved.** The `capability!` comment is now accurate.
  - `#[allow(dead_code)]` is gone. Both users read the field: `Rand` in `rand.rs`, and `Stdin` in `io/stdio.rs:17` and nearby. They are the only two `capability!` users, so the build cannot pick up a new dead-code warning from this.
  - The fixture std keeps its crate-wide allow.

**Nit (non-blocking):** in `rand/tests.rs`, the `lens` helper sits between the test's doc comment and the test. The `/// RAND-4: …` comment at line 153 now documents `fn lens` (line 157) instead of `real_draws_read_getrandom_through_the_seam` (line 163). Fix: move `lens` above the doc comment. This is cosmetic only.

I took the builder's focused runs on 4dc26c7 as their report, not as my own proof. They are:
- tide_std `rand::` and `sim::`: 22 passed;
- tide-admit build `rand`: 4 passed; code_prefix: 2 passed;
- conformance `authority/rand` and `authority/process_inputs`: 2 PASS.

Full suites on the final candidate remain tide-lead's.

## Verdict

**Approve.** B1 is resolved, and the fixes for N1, N2, N3, N5 and N7a introduce no defect. Two non-blocking items remain: the guard's textual limit described above, and the misplaced doc comment.

DONE
