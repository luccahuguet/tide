Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 15e0880

# fx-m4ip, Opus review r2 (docs-only round, with a fresh-reader pass)

Base fcd7758, candidate 15e0880. `git diff fcd7758...15e0880 | sha256sum` =
b06a1923cb84804011bf682b2aa90a7796d7a6aeb5b3728b09064a103ceb110c, which matches the brief. I read
all 17 files under docs/owner-guide/ and checked them against design/01-design.md (D0, D7, D8,
D12, D18), design/02-memory.md §5.4, design/03-authority-concurrency.md (§2.1, §2.6 RUN-3/RUN-4,
§9), README.md Status, conformance/rules.md, crates/tide-admit (policy.rs, walk/reify.rs,
invocation.rs, invocation/sandbox.rs, diag/labels.rs) and crates/tide-std (lib.rs, root.rs).
I also checked demo/ (run.sh, the patches, hits.rs, hits_sim.rs), trials/e1-checked/gate-report.md,
reviews/fx-p44q-audit.md, reviews/2026-11-15-achilles-opus.md, the fx-qlek.5 red-team record in its
worktree (156fcc7), the E3 draft protocol (fx-1vwf.1 worktree, 1a2a1ab), the beads fx-m4ip,
fx-qlek.5 and fx-1vwf, the builder's report.md and runs/*.log, and Codex's r1 and re-check. I ran
no cargo, demo or driver. The one exception was a check that nushell 0.113.1 runs a relative-path
script and passes `--` through, done on a throwaway script in my scratchpad.

## Verdict

**request_changes.** The guide is well built and mostly honest. Codex's nine findings are fixed
and I confirm those fixes. Eight blocking findings remain. Each is a sentence the owner could
repeat in public that is stronger than its source, or a step that contradicts what he will see on
screen. All eight are one-line text fixes, and none needs a rerun.

## Blocking findings

**B1. The RUN-3 sandbox's coverage is stated without its scope.**
- Where: `01-capabilities.md:187-188` ("test runs sit inside a kernel sandbox"),
  `03-soundness-bugs.md:170-173` ("Test and simulation runs … happen inside a kernel sandbox …
  Even code that forges a capability meets the operating system's fence there"), the one-liner at
  `03-soundness-bugs.md:178-179`, and `answers.md:25-26` ("Test runs … are inside a Linux kernel
  sandbox").
- Problem: README Status says RUN-3 "covers the runs `tide-admit` starts, in part". It also says
  "A test binary run directly gets no OS sandbox". The README's own demo recipe builds
  `target/demo-tests/hello-test` and runs it directly. Chapter 8 (`08-sandbox.md:160-162`) gets
  this right, but the funding sheet and chapter 3 do not. A critic who runs a built test binary
  directly falsifies the sentence.
- Minimum fix: in each place, say "test runs the driver starts (`--tide-run-test`, `--replay`)",
  and in answers 3 add "a test binary run directly gets none".

**B2. "Tide closes the known routes" is false on `main` today.**
- Where: `01-capabilities.md:187` ("Tide closes the known routes it has found") and
  `03-soundness-bugs.md:166` ("It closes the routes it knows").
- Problem: rust#57893 is a known route and `main` still admits it. Chapter 3 says so itself at
  lines 52-53 ("until it lands, `main` still admits that route").
- Minimum fix: "Tide has closed one known route (`FX0206`); the second, rust#57893, is closed in
  review (fx-qlek.5), and `main` admits it until that lands." Say the same in chapter 3's item 1.

**B3. The closure workaround and rust#100051 are overstated.**
- Where: `03-soundness-bugs.md:36-38` (the fix is a closure, "which rustc checks correctly") and
  `03-soundness-bugs.md:168-169` ("#100051 … already rejected").
- Problem: design/02 §5.4 (CLOSURE-3) says a call checks the item's own signature but "does not
  check the bounds an impl header implies (the rust-lang/rust#100051 family), which this rule
  leaves open". `reify.rs:5-6` and its `HELP` text at lines 13-15 say the same. The red-team record
  shows that route 12's witness for #100051 is rejected; it does not show that the family is
  closed. Together the two sentences tell a critic that the closure route is sound and that
  #100051 is dealt with. The design says neither.
- Minimum fix: line 38, "which makes rustc check the call against the function's signature (one
  related family, rust#100051, stays open; design/02 §5.4)". Line 169, "the known reproductions
  of … #100051 are rejected (the #100051 family stays open in the design)". Also mark that list as
  in review: it comes from fx-qlek.5's record, which is not on `main` yet.

**B4. Chapter 1 says the driver prints help text that it never prints.**
- Where: `01-capabilities.md:144-145` ("That is the message you saw in Run it") and
  `01-capabilities.md:161-162` ("into a Tide error that names the capability to use").
- Problem: the driver forces `--error-format=short` (`invocation.rs:332`), so `Rule::help` is
  never shown. The Run it output at lines 98-99 shows only "`std::fs::read_to_string` is not
  admitted". The owner will look back for the message and not find it, and if he demos the second
  claim it fails. Chapter 3 (lines 104-106) and the builder's report both state the right
  behavior.
- Minimum fix: "That help text is in the table, but the driver's short one-line errors don't show
  it (chapter 3 says the same)", and "into a Tide error that names the rule (`FX0501`)".

**B5. The red-team numbers are stale, and "every other route was rejected" is too strong.**
- Where: `03-soundness-bugs.md:46-52`, `10-v0-and-planned.md:35-36` and `answers.md:23-25`.
- Problem: the guide says 42 routes, one more admitted route, and every other route already
  rejected. The current fx-qlek.5 record (`reviews/2026-11-17-red-team-opus.md` at 156fcc7) says
  48 routes. It says two forms of rust#57893 were admitted: the blanket form, and the projection
  form, which Opus r2 found at 6a30b7c. It also says two routes stay admitted on purpose (a stderr
  write under SINK-1, and an inert crate attribute). The TODO marker covers updating at landing,
  but the numbers on the funding sheet are already wrong.
- Minimum fix: drop the counts. Say "a red team of over 40 routes found two forms of one more
  route, rust#57893, both closed by `FX0402` (in review)". Replace "every other route was already
  rejected" with "every other escape route was rejected".

**B6. "E1 shows Tide is cheap to adopt" generalizes past the evidence.**
- Where: `09-experiments.md:46`.
- Problem: the gate report says the result holds "for this task set, writer and checked
  toolchain". It says E1 "provides little separation on difficult maintenance or larger programs"
  and that "Tide never beat Rust". Adoption covers far more than a model writing small
  single-file tasks. Line 120 of the same chapter is worded correctly.
- Minimum fix: "E1 suggests models can write Tide at little extra cost, on small tasks with one
  model."

**B7. Hardest-question point 3 in chapter 10 is unsourced, and the repository's own review
contradicts it.**
- Where: `10-v0-and-planned.md:163-165` ("Most of the value is in the logic, not the I/O. The
  races and overreach that hurt in agent-written code are in the code that agents write").
- Problem: no source supports this. Achilles W1, which this answer replies to, cites the opposite:
  Eon's recurring defect class (lifecycle races) needs Proc and PTY, and fx-zu4f.1 had to exclude
  Eon's supervisor races "because v0 cannot express them". W1 also says the code that does the
  I/O in such programs is hyper, rustls and the like. A hostile follow-up quoting the project's own
  review ends this answer.
- Minimum fix: delete point 3, or restate it as a hypothesis that concedes W1's evidence, for
  example: "Tide's bet is that much of the risk in agent-written code is in its own logic; W1's
  Eon example shows that's not yet true for process-heavy programs, which is why point 4 sets the
  priority."

**B8. Answers 9 claims the conformance suite checks "both compilers' verdicts on every change".**
- Where: `answers.md:80-81`.
- Problem: the repository has no CI. The corpus runs when an agent runs `conformance/run.sh`,
  usually at landing for code changes. Also, the two checkers are stable rustc (against a stub std)
  and the driver, not two compilers. Anyone asking to see the CI falsifies the sentence.
- Minimum fix: "372 conformance cases check that stable rustc and the driver give the expected
  verdicts (369 pass; 3 await rules not built yet)". Drop "on every change", or replace it with
  what the project rules actually require.

## Non-blocking findings

- **N1. The run evidence doesn't match two commands as written.** `runs/examples-1.log` ran
  chapter 2's two `unsafe_hole` commands without `--crate-name=unsafe_hole`, and the artifact it
  lists is `lib.interface.json`. The guide passes `--crate-name=unsafe_hole`, and chapter 7
  (`07-trust-diff.md:81`) cats `target/guide/unsafe/unsafe_hole.interface.json`. report.md says
  the run used `--crate-name`, but no log shows it. The run is low risk because the flag is
  supported (`invocation.rs:241`). Still, the Acceptance item "each run recorded in the bead" is
  not met yet (Codex noted this too). Before closing, the coordinator should rerun those two
  commands and the chapter 7 `cat`, and record all the runs on the bead.
- **N2. The nushell claim is unrecorded.** `README.md:126-127` says "Every command in this guide
  pastes into nushell as written", but no run was made in nushell. I checked the risky parts in nu
  0.113.1 on a scratch script: relative paths such as `target/debug/...` and `demo/run.sh`, `--`
  passed through to an external command, and raw single-quoted `bash -c '…'`. All work. In nu,
  `ls target/debug` prints nushell's table, which is fine.
- **N3. `execve` is missing from the seccomp walkthrough.** `08-sandbox.md:131-136` and line 35
  ("no starting other programs") don't mention that `SYS_execve` is in `ALLOWED` (sandbox.rs:574).
  The owner will see it in the excerpt. Add one sentence: `execve` is allowed, but Landlock lets
  the child execute only its own binary, as the comment at lines 552-556 says.
- **N4. Error names are inconsistent.** `08-sandbox.md:17` says a forbidden call fails with
  "permission denied", but seccomp's `EPERM` is "operation not permitted" (line 125 says so).
  Landlock denials are `EACCES`. The diagram at line 47 ("kernel says EPERM") has the same
  conflation. This is a nit.
- **N5. `opaque(..)` in the artifact is unexplained.** `07-trust-diff.md:67` shows `Debug::fmt`
  with authority `{"f": ["opaque(Formatter<'_>)"]}`. A reader or critic will ask why a formatter
  holds authority. Add a "What to look at" bullet: `opaque(..)` marks a type that hides what it
  carries, listed conservatively (README Status).
- **N6. "Coverage" can be misheard.** `05-seeds-and-simulator.md:133` says "the simulator's
  coverage is a property the compiler enforces", which can be heard as "it explores every
  interleaving". Say "that nothing bypasses the simulator is a property the checker enforces".
- **N7. Missing soundness caveat.** `05-seeds-and-simulator.md:37-39` says admitted code "has no
  way to read the real clock …". Add "barring compiler soundness bugs (chapter 3)", as chapter 1
  does for the same kind of claim.
- **N8. The seed diagram isn't from a trace.** `05-seeds-and-simulator.md:27-32` attributes
  specific interleavings to "the demo's seed 0" and "seed 1", but no trace shows them. Either label
  the diagram "illustrative" or check it against seed 0's `trace.ndjson`.
- **N9. Wrong count in an example comment.** `examples/ambient/src/main.rs:2` says "all three
  marked lines", but four lines are marked and the run shows four `FX` errors.
- **N10. Wrong line count.** `03-soundness-bugs.md:107` ("eleven lines") and `README.md:42`
  ("11-line") describe a 13-line file. This is a nit.
- **N11. Inconsistent date.** `03-soundness-bugs.md:40` puts the RUN-3 move "the same day" as the
  fx-iwjq landing (2026-11-16), but D7 and the chapter's own Going deeper say it was decided on
  2026-11-15.
- **N12. Answers 4 names cackle and never explains it or answers it.** The owner won't know that
  cackle gives a Rust build per-crate API access lists. A critic will press that comparison.
  Either add one honest line on the difference or take cackle out of the question.
- **N13. The W5 paraphrase adds a detail.** `02-admission.md:173-174` ("a rehearsal bump while the
  driver is small"): W5's cheapest test is a bump and its mitigation is monthly bumps. "While the
  driver is small" isn't in the source.
- **N14. The "plain rustc" in conformance is a different compiler.** `02-admission.md:152` says
  "Plain rustc", but the conformance run uses stable rustc against a stub std. Chapter 3's "plain
  pinned rustc" is a different compiler. Name it "stable rustc, with a stub of Tide's std" so the
  owner doesn't conflate them.
- **N15. Sandbox claims need their tier.** `answers.md:72-73` ("the kernel sandbox bounds them
  regardless of language") and `10-v0-and-planned.md:161-162` ("RUN-3 today … bound imported code
  too"): today there is no imported code, and RUN-3 covers only driver-started test runs. Add the
  tier to each claim.
- **N16. The failure-precedence sentence reads as a contradiction.**
  `04-tasks-and-shared.md:28-30` ("A rule decides which failure is reported, so independent
  failures are never hidden") reads that way to a fresh reader. Add D8's "echo" idea in one
  clause: a sibling's error that is only a reaction to the cancellation never masks the real
  failure.

## Fresh-reader notes, chapter by chapter

I read the guide in order, playing an owner who knows terminals and agents but not Rust. None of
these notes would stop him finishing a chapter. B4 is the one place where he would look for
something the text promises and not find it.

- **Index (README.md).** The plan and the glossary are good. Setup is clear. "nightly" (used in
  chapter 2) and "rlib" (in the setup output) are undefined, which is harmless. The reading list
  totals about 79 minutes, which is close to the hour asked for.
- **Ch1 (about 20-25 min).** Clear, and the diagram helps. Sticking points: B4 (the help message
  he's told he saw), N9 (three marked lines against four), and `..deny(pattern, …)` in the excerpt
  (struct-update syntax), which isn't explained but can be skipped.
- **Ch2 (about 20 min).** "A second program that the checker gets run against"
  (`02-admission.md:139-140`) is unclear; say the case file is itself the program that both
  checkers are run on. "D12 (checker architecture A, then B)" names A and B without explaining
  them. "Achilles" is used before it's introduced; one clause would fix that ("the 2026-11-15
  adversarial review, code-named Achilles").
- **Ch3 (about 25 min).** The lifetimes introduction is good. "Generic function" (line 22) is
  never defined, and the #57893 paragraph (blanket impl, trait objects, `dyn`) is
  impenetrable without Rust. Add "you don't need the mechanism, only that it forges `&Fs`". The
  reify walkthrough works if he skims lines 28-48 as advised.
- **Ch4 (about 25-30 min, likely over 20).** At 1,614 words plus a 15-minute walkthrough of a
  58-line file, this is the longest chapter. The walkthrough is accurate line by line (I checked
  every range). `Ok(())` and `()` aren't explained, and LOCK-4 and SINK-5 appear without being
  explained. N16 applies here. Consider moving the hardest-question speed paragraph to the answers
  sheet to save time.
- **Ch5 (about 20 min).** Clear. Tokio, PCT and `research/10` are named without explanation,
  which is acceptable. N6, N7 and N8 apply.
- **Ch6 (about 20 min).** It's very good that the steps are runnable. The warning to replay right
  after `demo/run.sh` is clear. The run.sh walkthrough's line ranges all match.
- **Ch7 (about 15-20 min).** Clear. N5 is the one stumbling point.
- **Ch8 (about 20 min, plus the first `cargo test` compile, "several minutes").** The level and
  seccomp walkthrough's line ranges all match. N3 will puzzle him when he sees `SYS_execve` in the
  list.
- **Ch9 (about 15-20 min).** The E1 reproduction one-liner is good evidence and runs in seconds.
  "Sol" (line 64) is fine for this owner. B6 applies.
- **Ch10 (about 30+ min with the audit, so over 20).** "197 lines, 20 minutes; skim the tables"
  plus the chapter itself goes past 20 minutes. Either cut the audit step to the header and one
  table, or budget it separately in the index.
- **Answers sheet.** Good one-page shape. Fix B1, B5 and B8 here first, since this is the page he
  will read aloud.

## Hardest-question answers

Most survive a hostile follow-up: chapter 2 (coupling), chapter 4 (thread per task), chapter 6
(rr), chapter 7 (blind spots listed plainly), chapter 8 (two fences, with RUN-3 scoped correctly)
and chapter 9 (E3 framing, now correctly separating a missed 1.5× bar from absence of benefit).
Four need the fixes above:
- Chapter 1 and chapter 3 (B1, B2, B3): "closes the known routes" and "test runs are sandboxed"
  fall to "rust#57893 is still admitted on main" and "run the test binary yourself".
- Chapter 10 (B7): point 3 falls to the project's own Achilles W1 evidence.
- Answers 3 and 9 (B1, B5, B8): wrong counts, an unscoped sandbox claim, and "every change" with
  no CI.

## Reasoning

I treated this as a guide the owner will quote, so the test for each sentence was whether a hostile
reader could falsify it from the repository. Codex's round fixed the large conceptual errors
(unsafe, preemption, cancellation, Landlock, Antithesis, E3), and I confirm those fixes against
the sources. What remains is narrower: four claims that drop a qualifier their sources keep (B1,
B2, B3, B6), one that cites stale in-review numbers (B5), one that contradicts the printed output
(B4), one unsourced argument (B7), and one process claim with no mechanism behind it (B8). Every
command path, flag and excerpt line range I checked exists and matches its walkthrough: policy.rs
64-72 and 92-135, reify.rs 1-55, hits.rs, run.sh 107-141, sandbox.rs 84-101 and 549-578,
cap_ambient.rs, the 56-line patch, 57 codes, 12 `(planned)` labels, 13 v0-gap beads, and the
audit's 54/14/34/8. The recorded outputs match the builder's logs, apart from N1. The fixes are
text-only and change no command, so per AP-PROOF-001 a finder re-check of B1-B8 is enough; a
full rerun isn't needed.

DONE
