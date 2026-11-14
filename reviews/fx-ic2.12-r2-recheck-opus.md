Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: approve

Re-check of Opus r2 finding 1, plus a quick look at the fixes for nits 2-5.
- **Candidate:** HEAD `be3dde0e86d3041a242e816693c8ac480b1abf04`, tree `c43e31076635353fe1e689249988d30debd8f1b2`, one commit on 206b4ac. I reviewed `git diff 3c5788c be3dde0`.
- **Read-only:** the worktree is clean and unchanged, and I opened no held-out material.

1. **Finding 1 (blocking, a returned `Err` satisfied `should_panic`): resolved.**
   - **The fix.** It is in `crates/tide-std/src/test/runner.rs`, the shared runner, which owns the rule.
     - Sim path: `unpanicked |= matches!(failure, Failure::Exit(_))`, and the runner returns after the summary line.
     - Property path: under `should_panic`, a `Failure::Exit` prints its cause and returns instead of calling `report()`.
     - The flag reads the shrunk failure. That is sound, because `Shrinker` accepts a candidate only when `same_kind` holds (runner.rs:505), so shrinking never turns an `Exit` into a `Panic` or back.
   - **Probe, rerun on the new HEAD.**
     - Setup: tide_std built from `git archive be3dde0` in `opus-r2/copy4`, flattened to `opus-r2/art4`. The candidate driver binary was used; `crates/tide-admit/src` is unchanged since 3c5788c. Everything ran through heavy.sh, with load1 3.90 and 4.14 at the starts. Files: `opus-r2/probe4.sh`, `opus-r2/probe4.log`.
     - `#[sim_test(seeds = 2)] #[should_panic]`, body returns `Err`: `FAILED`, exit 101. The child prints `a failing seed did not panic; should_panic not satisfied`, the parent prints `should_panic not satisfied ... exit status: 101`, and libtest prints `did not panic as expected`. At 3c5788c the same source passed.
     - `#[test] #[should_panic] fn property_err(x: u8) -> Result<(), String>`, returning `Err`: `FAILED`, exit 101, with `the failing case did not panic`.
     - Controls: a sim_test and a property, each panicking `"boom"` under `expected = "boom"`, both pass with exit 0.
   - **The author's deliberate difference is right.** Under it, any seed whose failure did not panic fails the test, even after another seed's expected panic.
     - It is the stricter verdict: it cannot pass a test that the suggested rule would fail.
     - It extends round 3's self-check rule, where a later seed's failed self-check is not excused, to returned errors. A single rule, "a failure that is not a panic never satisfies `should_panic`", is easier to state than a carve-out.
     - It costs a correct candidate nothing. A body that returns `Err` in a `should_panic` test has hit a real failure, and Rust rejects that combination outright.
     - It leaves one asymmetry, which I accept: a later seed that panics with the wrong message is still tolerated, since only the first failing seed's payload meets `expected`. That is a panic, unlike an `Err`, and it keeps the first-failing-case parallel with properties.
   - **The property-path change matches 04 §2.4.**
     - Item 5's "`should_panic` never discards" is untouched: `evaluate` still skips requires discards under `should_panic`.
     - The first failing case is still reported at once, as before.
     - §2.4 says nothing about returned errors. Reference §1 and §2 keep Rust's testing, and 04 CLAUSE-7 expects a violation as a panic, so treating `Err` as "did not panic" matches Rust's rule that `should_panic` tests return `()`.
   - **Tests.** Both new std tests cover the rule: `should_panic_never_accepts_a_failure_that_did_not_panic` (returns `Err`, `Err` then a panic, the self-check cases, property `Err` against the panic control) and `should_panic_reraises_the_first_failing_seeds_panic`. The driver test's new `returns_err` row covers the end-to-end path. The author reports that a mutation check fails the std test; I did not repeat it.

2. **Nit 2 (duplicated render loop): fixed.** One `fn render(&[Record])` in scheduler.rs serves `Trace::render` and `Scheduler::trace`, and the cross-check assertion is gone.
3. **Nit 3 (first-failing-seed rule untested): fixed.** `should_panic_reraises_the_first_failing_seeds_panic` covers three cases:
   - steady panics;
   - quiet then expected, so not every seed must panic;
   - wrong then expected, where the first failing seed's payload is re-raised.
4. **Nit 4 (duplicate proof): fixed.** The token-string assertions in `sim_test.rs` and the duplicate writer `compile_error!` in the paired-library test are deleted.
5. **Nit 5 (text): fixed.**
   - The 04-contracts.md bullet now marks the paired-library part "(fx-ic2.12)".
   - The README `--tide-test-library` paragraph states that the library is built without `cfg(test)` and that its contracts, sampled clauses included, run as in a test build.

Open, and not this reviewer's: finding 6 belongs to the coordinator. The bead should record the final `should_panic` rule for sim_test:
- every seed runs;
- the first failing seed's panic must match `expected`;
- the test fails when:
  - any failing seed did not panic (a returned `Err` or a failed determinism self-check);
  - the child cannot launch, times out or dies by signal;
  - a deadlock aborts the child.

DONE
