Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: request_changes

Reviewed frozen HEAD `3c5788ceaf25146bfbf56808315d4247c6a72d30`, tree `295b261aa1f80b0af1d80f9f6d9160b12875ff2e`, as `git diff 206b4ac 3c5788c`. The worktree was clean and I left it unchanged. I made no edits, commits or br writes, and opened no held-out material. C3 case 2's timeout is with the owner and is not a finding.

1. **blocking: a `#[sim_test]` body that only returns `Err` satisfies `#[should_panic]`.** See `crates/tide-std/src/test/runner.rs:427` (`failure.report()`) and `crates/tide-std/src/test/runner.rs:174-176` (`Failure::Exit` → `panic!("property returned a failing exit status")`).
   - **Failure scenario.** Take `#[sim_test(seeds = 2)] #[should_panic] fn returns_err(sim: SimRoot) -> Result<(), String> { let _ = sim; Err("no panic here".into()) }`. No seed panics.
     - The child records `Failure::Exit`. `report()` panics with the runner's own message, and the child's libtest accepts that as the expected panic, so the child exits 0.
     - The parent sees exit 0 and a failed-seed summary at `crates/tide-std/src/sim/runner.rs:52-55`, so it panics too.
     - The test reports `should panic ... ok`.
   - **Probe.** I admitted this exact source through the candidate driver, using Codex's HEAD std artifacts, with heavy.sh, load1 3.97 at start. Result: `test returns_err - should panic ... ok`, exit 0. Files: `opus-r2/probe.sh`, `opus-r2/err-probe.log`. Codex's recheck shows the same at runner level (`codex-r1/recheck/returned_error_without_user_panic.log`) but kept it as a control.
   - **Why it blocks.** It is the same false-pass class as r1 finding 1: a runner verdict counts as the body's panic. The change's own contract rules it out. The new test is named `sim_test_should_panic_passes_only_when_a_seed_panics_as_expected`. The comment at `sim/runner.rs:45-51` says only a seed's panic counts. Rust itself refuses `#[should_panic]` on a test that returns non-`()`; reference §2 keeps that attribute.
   - **E1 impact.** On a grading surface, a wrong candidate whose error surfaces as `Err` through `?` would pass a `should_panic` acceptance case.
   - **Suggested fix: at the shared runner, the owner.** Under `config.should_panic`, a `Failure::Exit` never raises.
     - Sim path: choose the first seed whose failure is a panic as the payload to re-raise. If no seed panicked, return after the summary line with a `should_panic not satisfied` note, as the self-check branch at :419 does. libtest then fails the test as "did not panic".
     - Non-sim path (:409): handle it the same way, so sim_test and property semantics stay equal, as the bead's decision requires.
     - Prefer this over a macro rejection: a rejection would be another uncoded `syn::Error`.
     - Add a `returns_err` case (expect failure) to `build.rs:3620`, or a unit case beside `should_panic_never_accepts_a_failed_determinism_self_check`.

2. **nit: the render loop is duplicated.** `crates/tide-std/src/task/scheduler.rs:343` (`Scheduler::trace`) repeats `Trace::render` at :197 line for line.
   - The test then has to prove the two copies agree (`assert_eq!(base.render(), text)` in `trace_difference_names_the_first_changed_or_missing_line`).
   - Fix: have one `fn render(records: &[Record]) -> String` that both call, and drop that assertion.

3. **nit: no regression test covers the chosen "first failing seed" semantics.** `crates/tide-admit/tests/build.rs:3620`: its four bodies panic on every seed or on none.
   - Two mutations would pass the committed tests:
     - requiring every seed to panic;
     - re-raising the last failing seed's payload instead of the first.
   - Codex's probes covered these cases (`quiet_first_then_matching`, `wrong_first_then_matching`), but they are not in the tree.
   - Fix: add one unit case in `sim/runner.rs` tests, using `run_config` with `simulation` and `should_panic`. Its body depends on a static run counter: quiet on runs 0-1 (seed 0 and its self-check), then `panic!("expected")`, to assert the re-raise. Add a second body that panics "wrong" on seed 1's runs before "expected", to assert that "wrong" is re-raised.

4. **nit: two pieces of duplicate proof (AP-TEST-001).**
   - `crates/tide-macros/src/sim_test.rs:59-69` asserts token strings of the expansion. That is implementation detail, and the build test at `build.rs:3620` already proves the behaviour end to end.
   - The `#[cfg(test)] mod writer { compile_error!(..) }` in the new test at `crates/tide-admit/tests/build.rs:621` repeats the existing check at :560. Every paired library now gets `--cfg __tide_test_library`, so :560 already proves that `cfg(test)` stays off.
   - Fix: delete both.

5. **nit (text): one doc bullet is attributed to the wrong bead.** `design/04-contracts.md:498-500`: the amended bullet sits under "Implementation choices recorded on fx-mhd (research/18)", but the paired-library extension is fx-ic2.12's choice.
   - Fix: name fx-ic2.12 in the bullet.
   - Optional: the `--tide-test-library` paragraph in `README.md:211-220` could say that the library's contracts, sampled clauses included, run as in a test build. Today only the std crate docs say so.

6. **nit (planning): the bead's record of the semantics is stale.** Round 1's bead comment says a should_panic sim_test passes "when some seed panics and the first such payload matches". Since round 3, the test also fails when any seed's determinism self-check fails, when the child cannot launch, times out or dies by signal, and when a deadlock aborts the child. After finding 1's fix, a returned `Err` fails it too.
   - Fix: record the final rule on fx-ic2.12 when the coordinator records round 3.

Checked, with no finding:
- **Design fit, should_panic.** Accepting `#[should_panic]` on `#[sim_test]` matches reference §2, which keeps the attribute. §6.8 and 10.2 make sim_test a per-seed property shorthand. 04 §2.4 item 5 says "`should_panic` never discards", and `evaluate` skips requires discards under `should_panic`. `expand_property` moves `should_panic` (and `cfg_attr`, which rustc expands before the macro sees the item) onto the harness.
- **Design fit, sampled clauses.** `--cfg __tide_test_library` matches reference "`tide test` also checks sampled clauses on every call" and CLAUSE-6 ("`tide test` checks every head"): the paired library is the package under test, not a dependency. The 04 edit is accurate and is a one-line change (finding 5 aside).
- **Ownership.** `contract.rs::test_build()` is the only owner of "test build" for clauses: the sampled `enabled`, the unchecked `skip` and the range-loop schedule. The remaining `cfg!(test)` reads have other meanings:
  - `tide-macros/src/test.rs:63,85` is the proc-macro crate's own test mode.
  - `tide-admit/src/synthesis/contracts.rs:55` gates generated tests (writer test code), which correctly stay off in the tested library.

  The should_panic verdict is split across the child's `run_config`, the parent's `verdict` and libtest. The re-exec design makes that split inherent, and the parent needs its summary parse to reject vacuous runs and seed-parse panics.
- **Trace refactor.**
  - Completeness:
    - nothing still reads a string `trace` field;
    - the string `first_difference` and its test are gone;
    - the conformance stub std includes the production `task.rs`, `sim.rs` and `test/mod.rs`, so it has no copy to update.
  - Equivalence: `Record::render` is a pure function of the record, and every record renders as exactly one line, because `quoted` escapes newlines. So comparing records first and falling back to text gives the same first difference. The `{a:?}` output of an owned `String` matches the old `&str` output.
  - Behaviour of the trace API:
    - `Sim::trace()` renders on each call, O(n) like the old clone;
    - the replay and failure output use `render()`;
    - the deadlock fatal path renders through `Scheduler::trace()`.
  - Memory: by estimate, a `Record` is about 180 bytes plus the heap of the `Sched` runnable `Vec`, against text lines of about 150-200 bytes; sink records are smaller than the old escaped-bytes text. `sim_take_trace` moves the trace out instead of cloning it, so a checked seed holds about two traces at its peak instead of a transient three strings. I see no growth regression. The trace is still unbounded per evaluation, as before.
- **Leftovers.** The tide-admit crate doc (`main.rs:11-12`) and the tide-std crate doc (`lib.rs:9-10`) are updated. No doc still states the old sim_test restriction.

DONE
