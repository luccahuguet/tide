Opus 5.5 · 2026-11-11

**Verdict: clean.** The blocking finding is fixed, both nits are fixed, the sweep found nothing else that fx-u65.2 rejects, and the fix adds no new defect. Nothing blocks landing.

I worked read-only: I made no edits, no git or `br` writes, no builds and no suite runs, and didn't open `trials/e1-checked/`.

**Candidate identity.** HEAD is 1455fb1. `git diff --binary 87e60d3 HEAD | sha256sum` gives `eef6049b…ca099`, which matches the report. Main is now 7449564, a descendant of 87e60d3 that changes only `.beads/issues.jsonl` ("Record fx-mhd fix round 8"). So `git diff main HEAD` hashes to a different value, but only because of that ledger line. The fix delta `f643937..1455fb1` touches four files and 13 lines, all within scope.

**1. Blocking finding (bare `cfg!`): fixed.** Each rewrite keeps the original condition, and so which branch runs in each build:
- `executable.rs:146`: `if cfg!(debug_assertions) && cfg!(test)` became `#[cfg(all(debug_assertions, test))] { … }`. It runs only in debug test builds, as before.
- `executable.rs:156`: `if !cfg!(test)` became `#[cfg(not(test))]`. `middle(); sampled_false();` still run in debug and release non-test builds.
- `executable.rs:158`: `if !cfg!(debug_assertions)` became `#[cfg(not(debug_assertions))]`. The erased-violation calls and the `take_unchecked().is_empty()` assertion still run only in release.
- `scoped_clauses.rs:93` and `:95`: `not(test)` and `not(debug_assertions)`, mapped the same way. They follow the existing block at `:87`.
- None of the blocks is a tail expression, so `#[cfg]` is allowed in each position. No FX0103 marks were added.
- The logs agree with the report:
  - `workspace-r8.log`: 169 passed (5+45+14+17+2+78+6+2), including `executable_contracts_run_in_debug_test_and_erase_in_release`.
  - `conformance-r8.log`: 265 PASS, 15 AWAITS and 0 FAIL, with `contracts/executable.rs` and `contracts/scoped_clauses.rs` both PASS.
  - Both logs were written after the 21:14 amend.

**2. Sweep for other rejected constructs: nothing missed.** I checked every bang macro and derive in the lines fx-mhd adds against the lists in `walk/surface.rs`:
- **Fixtures and inline sources:** the macros used are `assert`, `assert_eq`, `invariant`, `format`, `vec`, `matches`, `println`, `panic`, `unreachable`, `unimplemented`, `todo` and `debug_assert`. All are allowed. The derives are `Clone`, `Debug` and `PartialEq`, all in the closed set.
- **`include_str!`** (`build.rs:13`ff) is in the test harness, not in Tide user code.
- **`collision::clause!()`** (`build.rs:184`) is compiled with plain `rustc` (`:225`), not through the driver.
- **Docs:** the remaining `cfg!`, `macro_rules!` and `compile_error!` mentions are prose in `design/04-contracts.md` and `research/18-executable-contracts.md` that describe the toolchain's lowering. None is a user code example.
- **Other codes:**
  - FX0203 and FX0210 only label rustc errors, so a fixture that compiles cannot trigger them.
  - FX0201, FX0202 and FX0301 would appear as unmarked diagnostics, and the conformance run shows 0 FAIL.
- **The sweep probe change** at `r8-sweep.py:107` (`format_args!` → `format!`) keeps the nested entry-value capture it tests. Its negative checks still require the exact message `old cannot name a bound variable` (`:38`), so a negative case rejected only for FX0103 could not pass by accident.

**3. Nits: both done.**
- `README.md:302` now reads "remain planned."
- `diag.rs:79–80` now says the `ALL` list also covers "toolchain-macro errors it passes through."

**4. New defects: none.** One side effect, which is not a finding: in builds where a guarded block is compiled out, its only callees (`over_budget`, and in test builds `sampled_false`) may get rustc `dead_code` warnings. The runner only matches `FX` diagnostics and no test checks stderr for warnings, so this doesn't affect proof or behavior.

**Findings:** none.
