Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 4f695bc

# fx-rpsb.1: Opus converge round (report only)

Scope: `git diff main...4f695bc`. I read the runner, `tests/support/admitted.rs`, the
`build.rs` change, both examples, and the root README. I checked them against
`crates/tide-admit/src/{main.rs,invocation.rs,synthesis/contracts.rs}`,
`crates/tide-std/src/{time.rs,test/runner.rs}`, `design/01-design.md` (D7, D10, D11,
D18), `design/03-authority-concurrency.md` (CAP-1, §3.2 AFREE-1), and
`design/04-contracts.md` (§2.4, generated tests). I also read the builder's report and Codex's
r1 pass and re-check. Following the brief, I ran no builds or tests, so the
behavioural claims below rest on reading the code.

## Verdict

**request_changes.** One blocking finding (B1). Everything else is non-blocking.

## Blocking findings

### B1. A rejected variant passes even when it also fails for an unrelated reason

`crates/tide-admit/tests/examples.rs:170-183` (the `Outcome::Rejected` arm), and the
contract text at `examples.rs:19-20`.

**Problem.** The check is `exit code == 1 && stderr.contains("error: <code>:")`. The
driver deliberately runs admission "after name resolution and before rustc's analysis,
so a crate with rustc errors still gets its verdicts" (`crates/tide-admit/src/main.rs`,
module doc). So a variant gets the declared TD code *and* any number of other errors,
and the runner still accepts it. Those other errors can be rustc errors (`error[E0599]`,
`E0277`, ...) or a different TD code. Each example README tells the reader that the
declared code is *the* reason the variant fails. For example,
`examples/capabilities/README.md:14` says "Passing the capability is legal, but `parse`
is declared `#[pure]`". The runner does not check that claim. This is lens 1's case
exactly: an unrelated failure that also emits the expected admission code. It matters
for the runner's next callers too. fx-zu4f's buggy revisions use `rejected <code>` as
the evidence that Tide catches the bug, and a revision that is also broken elsewhere
would still pass.

**Concrete failing cases.**
- In `examples/capabilities/pure_reads_clock.rs:28`, change `clock.now()` to
  `clock.now_ms()` (no such method, so rustc reports `error[E0599]`). TD0511 is decided
  from the parameter type `&Clock` before analysis, so stderr holds both
  `error: TD0511:` and `error[E0599]:`. The exit code is 1 and the runner passes. The
  README's "passing the capability is legal" is now false, and nothing notices.
- The same happens if a std rename later breaks a rejected example: for example,
  `Instant` losing `PartialEq`, which `#[derive(PartialEq)] struct Config` at
  `pure_reads_clock.rs:7-12` needs. Keeping examples green at every landing is this
  runner's whole job (bead "Why").
- `pure_reads_clock.rs` could also pick up an ambient call, giving `TD0501` beside
  `TD0511`, and it would pass unchanged.

**Minimum fix.** Require the declared code to be the variant's only error. The driver
always passes `--error-format=short` (`invocation.rs:349`). Every error line therefore
either contains `: error` (`path:l:c: error[E..]:` / `path:l:c: error: TDxxxx:`) or starts
with `error` (`error: aborting due to ...`). Assert that at least one error line exists,
and that each one is `error: <code>:` or the `error: aborting due to` summary. Update the
contract comment at `examples.rs:19-20` to match: "the driver exits 1, and every error it
reports is `<code>`". Add one case to the existing synthetic tests if the check is
factored out as a pure function, as `tested` already is. Otherwise, observe it once by
adding a stray E-code error to a rejected variant without landing it.

## Non-blocking findings

1. **A declared failure can be met by a hand-written test of the same name.**
   `examples.rs:142-154` maps `__tide_contract_tests::insert_sorted_N` to
   `insert_sorted`. It also accepts any `…::insert_sorted`, and `tested` takes the first
   matching failure (`examples.rs:290`). Suppose a variant had a hand-written failing
   `tests::insert_sorted` while its generated contract test passed. Then
   `test fails insert_sorted` would pass, but `examples/contracts/README.md:14-16` says
   "the failing test is the one the test build generated from the contract". No current
   file has hand-written tests, so the claim holds today by inspection. If fx-zu4f needs
   it, a declaration form that names the generated test explicitly would close the gap.
   Methods also collide: a generated test is named by `item_name` only
   (`synthesis/contracts.rs:59-64`), so `A::len` and `B::len` in one module both map to
   `<module>::len`.
2. **`std_dir` is now shared between two test binaries.** `admitted.rs:20-26` builds into
   `$CARGO_TARGET_TMPDIR/build/std` from both `build.rs` and `examples.rs`.
   `build_std` deletes and recreates `artifacts/` (`tide-std/tests/support/mod.rs`).
   `cargo test` runs test binaries one at a time, so the project's commands are safe. A
   runner that runs binaries in parallel (nextest), or two concurrent invocations on one
   target dir, would race on that directory. Before this change only one binary used it.
   A per-binary directory (for example `build/std-examples`) would remove the race, at
   the cost of a second std build. Record the trade-off, or leave it.
3. **Mistyped declaration prefixes are silently ignored.** A line that does not begin
   exactly with `//@` is not a declaration (`examples.rs:40`). This covers an indented
   `  //@`, `// @ expect: …` and `///@ …`. It is caught only when it was the variant's
   sole declaration. For example, `sorted_insert.rs` with a mistyped second line
   `// @expect: tests pass` would silently check only `runs`. A cheap guard would reject
   any line whose `trim_start()` begins with `//@` or `// @` but is not an exact
   declaration.
4. **`timeout` without `-k`.** `examples.rs:128-138`: a child that ignores SIGTERM keeps
   the suite waiting. Adding `-k 10` costs nothing. The driver invocations are unbounded,
   as in `build.rs`, which is acceptable.
5. **Wrong citation in `examples/capabilities/README.md:4`.** It cites "D7 and D11", but
   D11 is the trust surface. `#[pure]` and authority-free functions are decided in D7
   (`01-design.md:529-541`) and 03 §3.2 AFREE-1. Citing D11 also links the example to
   the trust diff, which owner decision A1 keeps out of the examples. Replace D11 with
   "03 §3.2". Feature item 1/2 in the report carries the same citation onto the epic.
   This is text-only.
6. **The contracts README overstates coverage and omits the decision number.**
   - `examples/contracts/README.md:41-42`: "for every function with an `ensures`".
     04 §2.4 says "Ungeneratable functions are ignored as unchecked", and the driver emits
     those as ignored tests (`synthesis/contracts.rs:43-47`). Suggested wording: "for
     every function with an `ensures` whose parameters it can generate".
   - `README.md:4` cites `design/01-design.md` with no decision; D10 is the owner.
   - `README.md:30-32`: "none generates tests from them" is a universal claim about
     "crates such as `contracts`". "Neither generates tests from them" is modest and
     verifiable.

   All three are text-only.
7. **The acceptance observation is the weakest form.** Deleting the only declaration
   (`deleted-outcome*.log`) trips the parser's "declares no expected outcome" check.
   It never tests an outcome mismatch. Deleting `tests pass` from `sorted_insert.rs`,
   or deleting a whole example directory, still passes. The builder disclosed the
   first case. The acceptance text ("deleting one example's expected outcome makes the
   runner test fail") is literally met, so this note is for the record only.
8. **The runner is not over-built.** All four outcome kinds have a current caller
   (`config.rs`, `ambient_read.rs`/`pure_reads_clock.rs`, `sorted_insert.rs`,
   `off_by_one.rs`). The ` - should panic` strip has no current caller, but it is three
   lines Codex required for fx-zu4f. The helper move from `build.rs` is complete: all
   five moved functions are used in both test crates, and `build.rs`'s remaining
   `Output`/`PathBuf` imports are still used (for example `build.rs:3560`). No orphans
   found. Optional nit: the three counts in `declared` (`examples.rs:107-124`) could be
   one pass.

## Reasoning

**Lens 1 (correctness).** I checked the other cases the brief names:
- *Timeouts.* A timeout yields exit 124 for runs, which fails `success()`. For test runs
  it is rejected by the summary/exit-code pairing (`suite`, `examples.rs:224-243`).
  Codex's r1 fix is sound. A missing `timeout` binary panics at `unwrap()`, which is a
  failure, never a pass.
- *Non-UTF-8 output.* It is read lossily. A false pass would need a declared line
  containing U+FFFD. Negligible.
- *An empty or missing `examples/`.* Both are caught: by `checked_variants > 0`, and by
  `read_dir` panicking. A stray non-directory entry fails loudly.
- *Parallel temp dirs.* Only `examples_meet_their_declared_outcomes` touches the
  filesystem in its binary, and `package()` wipes the variant dir, so no stale binary
  can be run. The driver also removes a previous executable. The shared `build/std`
  is the only exposure (N2).
- *Seeds.* `TIDE_TEST_SEED`, `TIDE_SIM_SEED` and `RUST_TEST_NOCAPTURE` are cleared.
  The contract runner's report lines are `eprintln!`, which libtest captures into the
  test's own section (`tide-std/src/test/runner.rs:431-440`). So the per-section report
  check is reading the right place.
- *Repeated flags.* `--crate-type`/`--emit` repeat, and the driver lets the last one win
  (`invocation.rs:259-260`), so the `lib`/`metadata` overrides behave as intended.

What remains is B1. It is the one path where a broken example passes. It is cheap to
close, and closing it makes the declared code a real claim for fx-zu4f.

**Lens 3 (claims).**
- Codes match the design: TD0501 is CAP-1 (03 l.66-73), and TD0511 is AFREE-1
  (03 l.1043-1047, 01 D18 l.1114).
- "Tide's std has no free function that reads files, the clock, the environment, or the
  network" restates D7 (l.485-487).
- The Clippy scope claim is accurate: lints apply only to linted crates.
- The contracts README matches 04 §2.4 on requires-guided generation (`sorted` is a
  generator shape) and on shrinking that keeps `requires` true. The off-by-one analysis
  is right: `xs=[0], x=1` inserts at 0. The declared reports pin the shrunk input.
- The Rust comparisons are modest after Codex r1, apart from the wording in N6.
- A1 holds: no README or the root paragraph claims the trust diff catches anything. The
  only trust-adjacent item is the D11 citation (N5).

**Lens 4 (acceptance).**
- Runner and two examples: met.
- Workspace suite: the builder ran it on 4f695bc (361 passed, 5 in `examples.rs`). The
  landing suite is tide-lead's, and the bead must record the names and counts.
- Deleted-outcome observation: met (N7).
- Feature list on fx-rpsb: **not yet recorded**. `br --no-db show fx-rpsb` shows only
  the epic body. tide-lead must post it, with N5's citation fix, before closing.
- Merge Rule: Codex r1 is done, and this is the Opus round. After B1's fix, the Opus
  finder re-checks only that fix.

DONE
