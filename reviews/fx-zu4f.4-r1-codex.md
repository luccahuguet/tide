Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate cd41434 (code+docs)

## Verdict

Changes required. High confidence. The code and declared outcomes pass; one factual comparison with Eon's existing test seam overstates the work Rust would require. No governance override.

## Scope

- Bead fx-zu4f.4, its comments, parent fx-zu4f and its comments, AGENTS.md, GOVERNANCE.md, and research/33-case-study-candidates.md's rank-3 brief.
- Frozen candidate HEAD: `cd41434a10082b6cc275ebdb1c0a6217755ea78d`, branch `fx-zu4f.4`.
- Base: `33085fd851224ec835230c3f7f068ee9cd5e40d2`.
- SHA-256 of the bytes from `git diff 33085fd..cd41434`: `25035155176c8ced9ead3305ee80dc41d7aa80181b0c0d31772d7737d9338451`.
- All five changed files, the examples runner and build helpers, simulator filesystem/scheduling, failure reruns, driver replay, and reference §§6.8 and 10.1. The builder report was treated as claims, not proof.
- Eon source read only through `git -C /home/lucca/pjs/eon/eon show` at `27b39095f92b544c870d8d60354bab5c35ae3300` and parent `040cb567257fc22a039b9ec64c39795857bd69c7`, including the commit message, sessions.rs, CHANGELOG.md, LICENSE, and the relevant bead record in the commit. Eon desk was notified before these reads. No Eon changes or builds; no commits or .beads/ writes. Only this requested report and generated check artifacts were written.

## Blocking findings

1. **The existing Eon hook can test the reversed ordering.** `examples/case-studies/eon-discovery-race/README.md:140` says checking the swapped host steps in Eon "would mean a new hook placement". That is false for the interleaving this model demonstrates. At `27b39095`, `runtime/crates/eon-runtime/src/sessions.rs:3127`, `discovery_fails_closed_when_a_live_host_has_no_valid_endpoint` already uses the existing `before_endpoint_check` hook to remove the endpoint while leaving the record Live. Discovery then checks and rereads before returning an error. Publishing the same run's tombstone after that return completes the reversed exit order, with the same final-state assertion as this model, without moving or adding a hook. This overstates Tide's advantage over the source's existing test infrastructure.

   **Minimum fix:** delete the sentence beginning "In Eon, checking that would mean a new hook placement" (lines 140–142). Keep the supported statement that swapping the host operations makes the same Tide test fail. No code change is needed.

## Non-blocking findings

None.

## Reasoning

**Source faithfulness.** The parent discovery reads the record, checks host liveness, then validates the endpoints without recovery from that last failure. The fix introduces `reconcile_if_ended`: one reread accepts an equal-identity tombstone, or the same Live identity and record object with a now-dead host. The new private discovery function supplies `before_endpoint_check`, and the commit adds deterministic tests using it. The modeled stale-Live-read/missing-endpoint window and the tombstone reread therefore match the source. The write-up identifies the omitted identity checks, host-liveness branch, reconciliation, and residual races. Eon's LICENSE at the cited commit is Apache-2.0. Its commit title, local commit date, error, ORB-C13 explanation, and quoted acceptance are supported by that revision. Current public branch reachability was not independently checked: the permitted Eon reads were confined to the cited commit and its parent.

**Three variants and property.** Setup uses `sim.world.file` to create the endpoint and Live record before either task runs. Buggy discovery can read Live, then observe the endpoint removed after the ordered host exit. Fixed discovery rereads and accepts the tombstone; its additional missing-endpoint test preserves failure when there has been no exit. Both tasks are joined before the final tombstone/no-discovery-error assertion. Allowing either `Run::Live` or `Run::Ended` is correct: discovery may have completed before the exit. This is a temporal test property, not an `#[ensures]` checked at discovery's return (§10.1). `out_of_order.rs` keeps fixed discovery and reverses only the two host operations; its failures demonstrate the ordering assumption is necessary for this modeled guarantee, without proving that Orbit obeys it.

**Independent focused check.** On this candidate, x86_64 Linux with pinned `nightly-2026-08-09`, the first load field was below 12 before starting. Command:

```sh
mkdir -p /mnt/build/tide-agents/fx-zu4f.4-r1-target/tmp
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.4-r1-target TMPDIR=/mnt/build/tide-agents/fx-zu4f.4-r1-target/tmp timeout 1800 cargo test -p tide-admit --test examples -- --test-threads=4 </dev/null
```

Exit 0: **8 passed, 0 failed**, including `examples_meet_their_declared_outcomes`. That runner discovers all three new files, builds each through admission against production std, and verifies completed test runs against their declarations. Its captured-output checks enforce buggy's `87 passed; 13 failed (100 seeds)` and out_of_order's `89 passed; 11 failed (100 seeds)` as well as the stated error. Fixed's tests pass. No runner modifications are needed. `git diff --check 33085fd..cd41434` also passes.

**Rerun and replay honesty.** `crates/tide-std/src/test/runner.rs:400` reruns each failing simulation seed, compares its trace and outcome, and substitutes a determinism-self-check failure on mismatch. This supports the README's description of the runner's checks. The examples suite itself launches libtest directly; it does not exercise driver manifests. Those are nevertheless implemented in the current driver and std, unlike the older research brief's "still planned" statement. The case study correctly limits its manifest claim to `tide-admit --tide-run-test` and `--replay`, rather than claiming the planned `tide` frontend or production recording.

I additionally used the required heavy.sh/env/target/TMPDIR/timeout wrapper, again below the load threshold, to invoke the built driver on the committed buggy.rs with production std, `--test --crate-type=lib --emit=link --crate-name=demo --tide-run-test tests::discovery_settles_a_run_whose_host_exits --seed 1`, then `--replay` on its generated manifest twice. Recording and both replays exited 101 with the expected discovery error. Both replays reported `replay reproduced` and produced byte-identical 38-line traces. Trace SHA-256 for this source/build invocation: `cac0688c8c082334101ff891572437c4db6b863923bc254fa351d146bf40f08b`. This is identity within the reviewed invocation; no cross-build trace-hash equivalence with the builder's different source invocation is claimed.

**Presentation and limits.** Admission's negative result and Eon's existing deterministic test precede the benefit claim. The root README's addition and pluralized summary agree with the existing case studies and their declared-outcome conventions. The limitations cover every item in research/33's brief: no Unix socket stat, no dead-host retirement branch, no remaining record-read/listing-to-lease/failed-host races, and no proof of ORB-C13. Crash faults remain explicitly planned. Apart from finding 1, the gain is appropriately limited to exploring this handwritten model with a supplied test property.

The full workspace suite and the subsequent Opus round were not run in this supervisor pass; landed-tree workspace acceptance remains the coordinator's responsibility. Minimum patch for approval is the single factual sentence deletion above; no alternative implementation is needed.

DONE
