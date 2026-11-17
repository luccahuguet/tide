Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 6157247

# fx-rpsb.1: Opus finder re-check of r2-opus.md

Scope: `git diff 4f695bc 6157247` (5 files), the updated `report.md` (round 2 section),
`r2-before-fix.log`, `r2-after-fix.log`, `workspace-r3.log`. The worktree is clean at
6157247. Following the brief, I ran no builds or tests, so the evidence on behaviour is the
builder's logs plus my reading of the code.

## Verdict

**approve.** B1 is fixed, and it has a regression test that failed on the old check.
Each non-blocking finding was either fixed or declined with a reason I accept. The fixes
add no blocking bug. I note two nits below, and neither needs a re-check.

## B1. A rejected variant passes when it is also broken for another reason: fixed

- `admission` (`examples.rs`, new) is a pure function of the declared codes, the exit
  code and stderr. It requires exit 1, and every error line in the
  `--error-format=short` output must carry a declared code. The only exception is
  `error: aborting due to ..`. Every declared code must also appear, so at least one
  error exists, because the first code is always a TD code.
- I checked the parse against the short format:
  - `path:l:c: error[E0425]: ..` gives `E0425`.
  - `path:l:c: error: TD0501: ..` gives `TD0501`.
  - An uncoded `error: cannot write ..` is rejected as undeclared.
  - A TD *warning* does not count, because a warning line has no `: error`.
- The paths come from `package()`, so they are absolute and never begin with `error`.
- The `check` arm now delegates to `admission`, and the doc contract
  (`examples.rs:19-22`) states the new rule.
- **Rebuttal of "only the declared TD code": accepted.** `ambient_read.rs` cannot meet
  that rule. Tide's std has no `std::fs::read_to_string`, so rustc's `E0425` accompanies
  `TD0501` at the call. This is the established verdict: `conformance/rules.md`'s CAP-1
  row lists "TD0501; rustc E0425 / E0599 / E0433 / E0432 (unresolved)", and
  `cases/authority/cap_ambient.rs:4` marks `//~ E0425 TD0501`. The builder lets
  `rejected` take extra required `TD`/`E` codes and declares
  `rejected TD0501 E0425`. That keeps my rule's intent: undeclared errors fail, and
  companion codes must be named, appear, and be explained in the variant comment and the
  README row. The README row ("beside rustc's `E0425`: Tide's std has no such function")
  is true of the driver.
- Residual risk, accepted: an author can declare an unrelated E-code to bless a broken
  variant. That is now an explicit, reviewed line, not a silent pass. The doc says
  further codes are those "the rejected construct itself causes", and reviewers check it.
- Regression test: `a_rejected_variant_fails_only_with_its_declared_codes` covers:
  - my `now_ms`/E0599 case;
  - a stray TD0501 beside TD0511;
  - an undeclared E0425;
  - a missing declared code;
  - exit 101;
  - no error line at all;
  - a warning line that is ignored.

  `r2-before-fix.log` shows it FAILED on the old `contains` check: `unwrap_err()` on
  `Ok`, `1 passed; 1 failed`. `r2-after-fix.log` shows all 7 tests passing, with
  `examples_meet_their_declared_outcomes` checking the 5 real variants. `workspace-r3.log`
  shows the workspace suite green on 6157247, with 7 tests in `examples.rs`.
- Nit, false failures only, no re-check: a non-error line is classified with
  `line.find(": error")`. A warning whose message contains `: error` after a colon, such
  as an unquoted identifier starting with `error`, would read as an undeclared error. It
  fails loudly and cannot cause a false pass. rustc usually backticks identifiers, so it
  is unlikely.

## N1. A declared name matching a hand-written test, or two methods: fixed

`tested` now rejects a `test fails` name that matches more than one test in the run,
whatever that test's result. This covers both cases I raised: a hand-written
`tests::insert_sorted` beside the generated contract test, and two methods with one name
in one module. Both now fail loudly, with no new declaration form. The regression test
`a_declared_name_that_matches_two_tests_is_an_error` would have passed `Ok` before the
fix: the only failure was `insert_sorted`, and its report matched. The before-fix log
filtered it out, but this finding was non-blocking and the before/after behaviour is
clear from reading the code. There is no new bug: ignored tests also count toward the
ambiguity, which is the strict reading.

## N2. Shared std directory: declined with a recorded constraint. Accepted.

`std_dir`'s comment in `admitted.rs` now says the two binaries share
`build/std`, that `cargo test` runs binaries one at a time, and that nextest would race.
Building std once matters for the full suite's time, so this trade-off is reasonable.

## N3. Mistyped declaration prefixes: fixed

A line that is not an exact `//@` declaration is now an error when it is:
- an indented `//@`;
- `// @`;
- `///@`, which is any `//…` comment whose text after its slashes and spaces begins
  with `@`.

`declarations_are_read_strictly` has three cases, including the `sorted_insert` scenario
I gave. Side effect: an ordinary comment such as `// @see` in an example now fails loudly.
That is a false failure with a clear message, which is acceptable.

## N4. `timeout -k`: fixed

The argument order is `timeout -k 10 <secs> <program>`, which is correct.

## N5. D11 citation: fixed

`examples/capabilities/README.md:3-5` now cites D7 and 03 CAP-1 and §3.2 AFREE-1. Feature
items 1 and 2 in `report.md` match. No trust-diff link remains, so A1 holds.

## N6. Contracts README wording: fixed

The README cites D10, says "neither generates tests", and narrows the claim to "every
function with an `ensures` whose parameters it can generate", which matches 04 §2.4.

## N7. Weakest acceptance observation: declined. Accepted.

The acceptance text is met. The mismatch paths are now covered by the synthetic tests
for `tested` and `admission`.

## N8. One pass over counts: declined (nit). Accepted.

## New findings

- Nit, text only: the parser's error message (`examples.rs`, `declared`) spells the form
  `rejected TD<4 digits> [E<4 digits>...]`. `error_codes` also accepts further `TD`
  codes, and the doc says `[<code>...]`. Aligning the message would be tidier.
- Outside this bead, for tide-lead only: 03 CAP-1 (`design/03-authority-concurrency.md`
  l.68-71) says the driver reports an ambient path "as `TD0501`, not as rustc's
  unresolved name". The driver and the conformance corpus emit both. The example
  documents the actual behaviour correctly. Whether CAP-1's wording should say "beside"
  is a design-text question this bead does not edit.

DONE
