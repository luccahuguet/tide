# Tide review, fx-o8o round 3 (Opus): confirmation round

Candidate: a1f0b5c on branch `fx-o8o` (round-2 candidate bf712d8, base 3fdc020). I reviewed `git diff bf712d8 a1f0b5c` (4 files under `trials/e1-checked/`), harness.py as it stands at a1f0b5c, the suites, and the fx-ic2 decision comment. The worktree's tracked tree was not modified.

Host results come from the orchestrator: harness_test.py 17/17, Rust references sandboxed 30/30 PASS, Tide references 30/30 FAIL as mapped, and no reference record raised an integrity flag.

My own checks ran in the scratchpad only:
- A std-only probe crate built with `rustup run 1.96.0 rustc --edition=2024 --test`. Seven tests: captured `println!` from the test thread and from a spawned thread, `#[should_panic]`, `#[should_panic(expected)]`, `#[ignore = "why"]`, a direct `io::stdout().write_all`, and two forgeries.
- Each probe test ran through the a1f0b5c `test_command`, with `run` replaced by a plain subprocess and the frozen list replaced by the probe's names. I also ran `integrity_flags` on the probe source and on sample lines.

Model: claude-opus-5-5 · Date: 2026-11-08

## Verdict

request_changes. Round-2 B1 is fixed as decided: frozen names, an exact named result per test, symmetric flags, and the limit documented. Two findings block:
- B1 (not text-only): the new named-outcome parser rejects passing tests that print through an uncaptured stdout. Tide's `println!` always prints that way, so the parser fails correct Tide code that Rust passes.
- B2 (text-only): readiness.md says the a1f0b5c proof is still owed. The host has now run it.

## Round-2 findings

- **B1 (forgeable acceptance verdict): fixed as decided, with one regression (new B1 below).**
  - **Frozen list: correct for every task and name.**
    - I compared `ACCEPTANCE_TESTS` with every fn that carries `#[test]` or `#[sim_test]` in `suite(task, arm)`, for all 26 library tasks in both arms. They match exactly, with no extras and no omissions. Helper files (`fs.rs`, `random.rs`) and helper fns (`populated`, `tree`, `replay`) declare no tests.
    - The listed order is libtest's byte order: `case2_drain < case2_new < case2_push`, `case3 < case3_equal_edges`, `case6 < case6_zero`. The host's Rust 30/30 confirms the order for all 26 Rust listings.
    - The `harness_test.py` inventory check (`^fn (case\w*)\(`) would miss a test whose name does not start with `case`. None exists today.
  - **Parser on real 1.96.0 output: no false pass except a complete forgery.**
    - These all pass: ` - should panic` with and without `expected`, a `finished in 0.00s` suffix, and `N filtered out` equal to `len(tests) - 1`.
    - An ignored acceptance test fails, as intended. Writer-test ignores with a reason still pass.
    - `println!` is captured on the test thread and on spawned threads, so it passes.
    - The false failure is new B1.
  - **Integrity flags.** The searches are identical in both arms, the record is code 1 with `review_required`, and the README documents the limit. Each case from the brief:
    - **Cheap escapes: covered by the documented limit, not blocking.** Both escapes below passed the parser with exit 0 and raised no flag (probe cases 6 and 7):
      - a 2024-edition `unsafe extern "C" { safe fn _exit(code: i32) -> !; }`, called with no `unsafe {}` block;
      - a `concat!("test re", "sult: …")` transcript followed by `std::os::unix::process::CommandExt::exec` of `true`, all in safe std.

      `use std::process::{exit}` and `use std::process::*` are also unflagged. These fall under the README's "can miss aliases, generated code or constructed output" and its sentence on Rust-only termination hooks. fx-ic2's comment says "in principle" and requires review only for flagged records.

      Writers see neither harness.py nor the flag list, so an evasion would have to target a detector they don't know exists. Under AP-MINIMAL-001 an honest partial search is acceptable. A cheap hardening: add `\bunsafe\b` (it covers `unsafe extern`, `unsafe {` and `#[unsafe(`) and `\bexec\s*\(`. Tide rejects `unsafe` without the grant and has no `exec`, so neither adds a Tide false positive outside comments.
    - **False positives, all review-only:**
      - `\bctor\b` flags the identifier `ctor` (`let ctor = …`).
      - `test result` flags a comment such as "compare the test result".
      - `#[unsafe(` cannot hit a Tide grant, because grants live in the manifest (`reference.md:1026`). Tide cannot use the attribute without the `unsafe` grant.
      - The one that matters is N1.
- **B2 (readiness.md stale): fixed for bf712d8; the same staleness recurs at a1f0b5c (new B2).**

## New findings

**B1 (not text-only, blocks). The named-outcome parser fails every passing test during which Tide's `println!` runs. Rust's `println!` is captured, so only the Tide arm is hit.**

Where: `harness.py:333-337` (`outcomes` / `completed`), used for writer tests and acceptance tests in both arms.

Mechanism, from real libtest output:
- With `--test-threads=1`, libtest prints `test NAME ... ` before the test runs.
- Output that bypasses libtest's capture lands inside that line. The probe's case 4 printed `test case4 ... direct` then `ok` on its own line. Exit was 0 and the summary was `ok. 1 passed`, yet `test_command` recorded code 1, "missing or inconsistent named test result".
- Rust's `print!`/`println!` are captured, on spawned threads too. Only a deliberate `io::stdout().write_all` bypasses capture.
- Tide's `print!`/`println!` expand to `::std::__sink::out` (`tide-macros/src/lib.rs:83-95`), which writes through `io::stdout().lock()` (`tide-std/src/root.rs:36-42`). That path bypasses libtest's capture every time.

Impact:
- Any Tide writer test that prints, and any Tide library code that prints while an acceptance test runs, gets a mechanical failure. The same program in Rust passes.
- On writer tests this is `./test` feedback that wrongly says a passing test failed. That costs iterations, which are an E1 measure.
- The asymmetry is not among §11's kept asymmetries, and it makes the Tide arm's job harder.
- It is masked today only because Tide's `./test` and library acceptance also carry the code-2 runner and library-boundary markers (fx-nkv.6, fx-m72, fx-dsa). The parser freezes with the gate harness, so it would fire once those land.
- bf712d8's substring check did not have this problem.

Fix (keeps the decided integrity): the counts line plus `--exact` already prove that exactly one test ran and passed. So identify the named result without requiring the status to share the start line. For example, require exactly one `^test NAME( - should panic)? \.\.\. ` start marker. Then take the status from the last `(ok|FAILED|ignored)(, …)?` line before the single summary, or drop the outcome-line requirement for `ok` and keep the counts and the single-summary rule.

Add one regression case to `test_acceptance_requires_frozen_names_and_consistent_results`: interleaved stdout such as `test case1 ... hello\nok\n` must pass.

**B2 (text-only, blocks). readiness.md calls the a1f0b5c proof outstanding, but the host has run it.**

Where:
- `readiness.md:6`: "The round-2 result-integrity changes after bf712d8 require rerunning that proof."
- `readiness.md:145-147`: "Remaining work is rerunning proof after the round-2 result-integrity changes".

The orchestrator has run a1f0b5c on the host: 17/17, Rust 30/30 PASS, Tide 30/30 FAIL as mapped, no integrity flag. B1's fix changes harness.py, so record the final candidate's host proof, with its artifact hashes where available. Then drop both sentences. This is the same fix as round-2 B2, applied to the landing revision.

**N1 (nit, recommended with B1's fix). Integrity flags on binary tasks guard nothing and fall mostly on Rust.**

Where: `harness.py:471`. `integrity_flags` runs for B1, B2, B5 and B8 too.
- Binary acceptance (`binary_cases`) judges the program's own exit code and output, which the candidate legitimately controls. No libtest transcript is parsed, so there is nothing to forge.
- These tasks require exits 1 and 2. `std::process::exit(2)` is idiomatic Rust (the references happen to use `ExitCode`) and is FX0501 in Tide.
- So correct Rust binary trials will often get a code-1 review record, and Tide trials never will.
- Verdicts survive a faithful review. The review load is one-sided and buys no protection.

Fix: emit the integrity record only when `config["task"] not in BINS`, and say so in the README.

**N2 (nit). Tide acceptance listing will include the writer's own tests.**

Where: `package()`'s Tide sibling-module probe (`harness.py:241-246`).
- The candidate is compiled as `mod solution` under `--test`, so any `#[cfg(test)]` tests in it are listed as `solution::…`. The listing then differs from the frozen list, and acceptance fails.
- This sits under the code-2 library-boundary marker and the existing "test selection" note, so it is not live.
- Record it on fx-dsa, or filter the Tide listing to the `acceptance::` prefix before comparing. That filter is safe because candidate tests cannot be named `acceptance::…`.

**N3 (nit). Tide frozen names assume `#[sim_test]` expands to a plain test of the same name.**

`#[sim_test]` is not implemented yet (`tide-std/src/lib.rs:21`). If its expansion renames or nests tests, fx-ic2's suite freeze must refresh `ACCEPTANCE_TESTS` for the seven Tide suites that use it: B10, C3, C4, C5, C6, C9 and C10. The README's "current Tide adapter" wording covers this, so no change is needed now.

**Arm symmetry otherwise holds.** The listing rule, the per-test rule and the flag rule are identical in both arms. The only new asymmetries are B1, which blocks, and N1, which is review load only.
