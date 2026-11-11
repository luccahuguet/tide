Model: gpt-6.1-sol · Effort: high · Date: 2026-11-11

Verdict: approve

Numbered findings: none. No blocking, non-blocking, or nit findings. No patch is required for this Codex pass. Confidence: high for the supported subset and recorded bead decisions.

What this change does: exposes Root::sim and process-isolated sim_test entries over the existing seeded scheduler, captures ordered sink output, and checks trace and outcome determinism. Production owns the runtime and macro; conformance includes those definitions.

Reviewed candidate: 419b07d49cfa84769f7f0f5dbb1855468e89f2c0 on 32c5bc7. Diff SHA-256: 134504951f2bb13e31cd332c4a6ffcf4f806c2fbdcb54fce4ee1a1c0088b6213. HEAD, digest, and clean worktree were verified again before this report. The range-diff from 33325d2..4bb58e1 to 32c5bc7..419b07d reports the patch unchanged.

Review evidence:

- Acceptance: conformance/cases/tasks/simulation_runner.rs:9–38 uses a genuine split-critical-section lost update. Both workers use ordinary Shared operations; no injected failure determines the counter. The fixed version makes the read-modify-write one closure. crates/tide-admit/tests/build.rs:1745–1820 exercises both through a driver-built production binary and compares failing traces across fresh processes.
- Process model: crates/tide-std/src/sim/runner.rs:16–107 selects the exact libtest name, compares the complete private child-role value, closes stdin, drains stdout and stderr concurrently, and kills/reaps the child on timeout. Seed-start records begin on a separate line and are flushed before execution. Abort and timeout tests verify last-seed attribution. The expected child path enters the seed loop directly rather than re-executing itself.
- Lifetime: crates/tide-std/src/task.rs:270–281 and :428–510 keep task 0 active for the root lifetime, record its normal/panic exit, diagnose early root destruction in a live scope, and prevent scope unwinding from restoring the ended main context. Existing structured scopes retain ownership of joining children; stale capability use fails its simulation-identity check.
- Trace and effects: crates/tide-std/src/task/scheduler.rs:231–255 and :382–441 append ordered sink events with fixed keys and shared JSON escaping. Scheduler decisions retain sorted task IDs and per-simulation counters. Host time is confined to the runner backstop. root.rs:60–80 diverts sinks before host writes; fs.rs:136–148 preserves simulation identity through attenuation and rejects operations before host I/O; collections.rs:35–42 rejects simulated Rand and real Rand use inside simulation before host entropy is acquired.
- Macro and seams: crates/tide-macros/src/sim_test.rs:5–75 lowers to a parameterless harness entry with fn(SimRoot), validates the seed argument and unsupported signatures, and rejects should_panic. The conformance std includes production sim.rs, task.rs, root.rs, and the production macro; the oracle_macros dependency and lockfile entry are removed. Generated parameters/shrinking, the file-store backend, and replay identity remain explicit successor work. The docs accurately describe this partial surface and its limits.

Validation:

- Inspected AGENTS.md, GOVERNANCE.md, the requested bead records and binding decisions, research/16-deterministic-simulation.md, the requested ROOT/TASK/simulation/reference sections, the complete diff, and affected owners/callers/tests. No trials/e1-checked/ files were opened.
- Inspected the implementation report, suite logs, and saved exit statuses: workspace 149 passed; conformance 220 PASS / 24 AWAITS / 0 FAIL. These are implementation proof, not newly executed supervisor suites. The docs-only rebase leaves the patch unchanged.
- Supervisor probes reused the existing production driver/std artifacts. The driver rejected both #[sim_test] followed by #[cfg_attr(test, should_panic)] and #[should_panic] followed by #[sim_test], with the intended diagnostic. An initial scratch-layout attempt stopped at the driver's package/output separation check before compilation; the corrected layouts were outside the repository.
- Independently executed the existing driver-built simulation fixture: seed 0 failed with counter=1; two fresh runs produced identical trace bytes. All 32 events parsed as JSON, contained the required fields, had consecutive steps, included captured sink output, and ended in task 0's panic exit. Simulated messages did not leak to raw stdout. The fixed revision passed all 20 seeds.
- Each focused probe ran under nice -n 19 and CARGO_BUILD_JOBS=4, with stdin closed and an outer timeout. Launch gates passed: one-minute load below 12, available memory at least 6 GB, and no active quiet window. No full-workspace build or suite was launched. No repository, Git, or br state was changed.

Not checked: full suites were not rerun; Opus review remains the next governance step. The documented limits remain: trusted test processes have no sandbox, the timeout is per child, and replay manifests and unmodeled capability backends are pending.
DONE
