Model: gpt-6.1-sol · Effort: high · Date: 2026-11-11

Verdict: request_changes

1. **Blocking — scalar guidance breaks valid borrowed-parameter properties.** Evidence: `crates/tide-macros/src/test_guidance.rs:58`, `:204`, `:278`, `:316`; `crates/tide-macros/src/test.rs:143`. Type recognition strips references, so `lo: &u16` and `hi: &u16` qualify for scalar endpoint guidance. However, bound substitution retains `__tide_arg_lo` as a reference while the borrowed target is generated into owned `u16` storage. This produces a `range::<u16>` lower bound of type `&u16` and prevents compilation. The runtime filter cannot protect against an ill-typed generation plan.

   Reproduced through the frozen candidate's driver and production std:

   ```rust
   #[test]
   #[requires(lo <= hi)]
   fn borrowed(lo: &u16, hi: &u16) { assert!(lo <= hi); }
   ```

   Build exits 1 with `E0308: expected u16, found &u16`. Moving the identical comparison into `fn ordered(lo: &u16, hi: &u16) -> bool` and using `#[requires(ordered(lo, hi))]` builds and passes, reporting `100/170 cases passed`. This violates reference §10.2's borrowed-input support and the recorded decision that a missed shape should cost only acceptance rate.

   **Fix wanted:** distinguish the borrowed argument from its owned scalar value when constructing dependent bounds. Emit an appropriate copied/dereferenced scalar bound, or conservatively leave a shape as a filter when its operands cannot be translated. Add a driver-built regression for the borrowed endpoint relation; retain the existing borrowed-slice/index coverage.

2. **Blocking — the plain-property seed exception also captures simulation parameters.** Evidence: `crates/tide-macros/src/test.rs:140`. The `seed: u64` special case has no `simulation.is_none()` condition. Therefore an ordinary extra parameter named `seed` on `#[sim_test]` receives the simulator's case seed, bypasses its guidance plan, and has no shrinkable choice. The binding decision explicitly limits this spelling exception to `#[test]`; simulation parameters are supposed to draw from the derived parameter stream while only the simulator seed stays fixed.

   Reproduced through the same driver and std:

   ```rust
   #[sim_test(seeds = 2)]
   #[requires(seed == 54321)]
   fn guided_seed(sim: std::sim::SimRoot, seed: u64) { let _ = (sim, seed); }
   ```

   This fails as `vacuous: 0 of 200`, rejecting `seed == 54321` every time. Renaming only the extra parameter and its clause to `value` passes `2/2` cases. A second probe without requires, asserting that the extra parameter is `< 2`, passes for the name `seed` but fails and shrinks to `value = 2` for the name `value`. Thus the leak both defeats recognized equality guidance and restricts test coverage based on a parameter name.

   **Fix wanted:** apply the fixed `seed: u64` exception only when `simulation.is_none()`. Generate and shrink every extra simulation parameter normally. Add a driver regression with a simulation parameter named `seed`, and preserve the plain-property fixed-seed and simulator-seed replay checks.

The shared runner and shared clause parser are the right ownership boundaries; no redesign or dependency is needed. Approval requires the two focused fixes and their regression checks, followed by the required re-check. Confidence in both findings: high.

Reviewed the complete diff, touched implementations and relevant callers, governance, the named beads and binding decisions, the named property research, reference §§10.1–10.2, contract semantics, the new execution fixture/build test, existing property/simulation tests, and production/stub sharing. By inspection, accepted/generated counting uses the recorded N/100*N stopping rule, excludes shrink evaluations, counts first rejections, and breaks ties by source order. The release-property path retains its own requires while the ordinary-function path retains release erasure. No additional blocking issue was found in those paths. These observations are not a replacement for the coordinator's recorded suite proof.

Candidate verified before and after review: HEAD `f8576f73013f3b57b3cfe5957fc0cf958bd076e1`, base `9989c16`, binary diff SHA-256 `a682fc4a9ab74812f3c598b0e9565c305d3db0c9e32d72ee359eb6445c8fe6c3`; clean worktree; `git diff --check 9989c16..HEAD` passed.

Focused probe evidence: Linux x86_64, existing pinned-nightly driver `target/debug/tide-admit` and production artifacts in `target/tmp/build/std/artifacts`; no Cargo rebuild. Sources/results are under `/home/lucca/.cache/eon-agents/tide-fx-m72.2-work/review-probes/` (`borrowed`, `borrowed-filter`, `sim-seed`, `sim-seed-guided`). Each compilation used `nice -n 19 env CARGO_BUILD_JOBS=4 timeout 120 target/debug/tide-admit --crate-type=lib --test --emit=link --tide-std <artifacts>/libtide_std.rlib --tide-dep-dir <artifacts> --out-dir <probe>/out <probe>/package/src/demo.rs`, with stdin closed. Successful builds were run with `nice -n 19 env CARGO_BUILD_JOBS=4 timeout 30 <probe>/out/demo-test --nocapture`, also with stdin closed. Every probe was gated on load <12, available memory >=6 GB, and no active quiet window. Observed load was 3.44–4.24 and available memory 15 GB. Borrowed-guidance build failed as described; its opaque-filter control passed; each simulation comparison binary had one expected pass and one expected failure.

No source, ledger, staging, or commit changes. No full-workspace build or suite rerun; the reported 172 workspace passes and 266 PASS / 15 AWAITS / 0 FAIL remain implementer/coordinator proof. Held-out files were not opened. The candidate is not clean for the following Opus review until findings 1–2 are resolved. DONE
