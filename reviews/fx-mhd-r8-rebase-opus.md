Opus 5.5 · 2026-11-11

**Verdict: not clean.** The rebase caused one semantic conflict, and it blocks landing. The README merge reads correctly apart from one nit. I worked read-only, did no build and opened nothing under `trials/e1-checked/`. The blocking finding comes from reading the code; the suite rerun on f643937 should confirm it.

The base is 188705c, a ledger commit on top of 18adfd8. `main` has since moved to 0e86bbb, but that commit touches only `.beads/issues.jsonl`.

**1. README merge**

- Lines 171–176 are accurate. "remaining simulated capabilities" (from fx-byk.2) fits the tree, where `sim.fs` and simulated files now exist. The fx-mhd text about executable clauses is unchanged.
- Lines 300–302 are accurate in substance: the stub std now does use `tide_macros` and includes production's `contract.rs`. One nit is below.

**2. Semantic conflicts**

- **FX codes:** no collision. fx-u65.2 added FX0102, 0103, 0201, 0202, 0203, 0210 and 0301; fx-mhd adds FX0804–0807. The `Code` enum, `ALL` and `as_str` are consistent, so `--tide-print-codes` lists both sets.
- **`rules.md`:** no duplicate rows. FORM-1/2/4 and CLAUSE-1/4/7 stay separate from the new REF-0102/0103/0210/0301 rows.
- **Stub std wiring:** clean. The `__contract` module and the prelude's `tide_macros` re-export sit alongside fx-byk.2's fs/sim changes. `invariant` is in fx-u65.2's list of allowed bang forms (`BANG_FORMS`), and it resolves through the std export.
- **`build.rs` helpers:** no signature changes. fx-mhd's inline sources contain no lossy casts, disallowed bang forms or disallowed derives; `die as fn(i32) -> !` is an address cast, which counts as lossless.
- **Lowering under the new checks:** the `::core::cfg!`, `compile_error!` and `panic!` calls that the contract macros generate carry the macro crate's spans, so the FX0103 check skips them. In `macros_reject.rs`, `core::line!` inside `#[requires]` should still report FX0103 on its line.

**Findings**

1. **Blocking** — `conformance/cases/contracts/executable.rs:146`, `:155`, `:156` and `conformance/cases/contracts/scoped_clauses.rs:93`, `:94`
   - These lines use `cfg!(…)` in code the user writes. fx-u65.2 rejects that: `cfg!` is not an allowed bang form, and `conformance/cases/authority/macros_reject.rs:10` marks `let _ = cfg!(test); //~ FX0103`.
   - Before the rebase FX0103 did not exist, so neither branch alone shows the problem.
   - Effect: `conformance/run.sh` will report FAIL for both cases (an FX0103 diagnostic with no mark). The driver will refuse to admit the package in `crates/tide-admit/tests/build.rs:13` (`executable_contracts_run_in_debug_test_and_erase_in_release`), which compiles both files through the driver.
   - Fix: use `#[cfg(…)]` attributes on the statements or blocks, the way `scoped_clauses.rs:87` already does (for example `#[cfg(all(debug_assertions, test))] { … }` and `#[cfg(not(test))] { … }`). Don't add FX0103 marks; these files are meant to be programs that compile and run.
   - This changes test code, so the reviewers need to re-check the fix, along with both suites on the fixed tree.

2. **Nit** — `README.md:302`
   - The merge brought back "remain planned in production", which fx-byk.2 (a5343a2) had deliberately shortened to "remain planned".
   - The stub shares production's definitions, so "in production" wrongly suggests the stub already has these features. Drop "in production". This is a text-only fix and needs no re-check.

3. **Nit** — `crates/tide-admit/src/diag.rs:73`
   - fx-u65.2 rewrote the comment on `ALL` to "(its own verdicts and its labels on rustc's)". FX0804–0807 are neither: the contract macros emit them as compile errors with no rustc code, and the driver passes them through.
   - Widen the parenthetical, for example "…and toolchain-macro errors it passes through". This is text-only.
