Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Re-check ccc8e6b -> 2da2c84 (Codespaces)

Scope: only the fixes for B1, B2, B3 and the taken public-repository wording item, using `git diff ccc8e6b 2da2c84` and target/work/report.md, section r2. HEAD is 2da2c84cc4b9de8903fc85bcda06452670a3a139; tracked worktree clean at inspection.

B1 — NOT RESOLVED (remaining regression-test gap; implementation mitigation is present).
- Fixture isolation is implemented at crates/tide-demo/tests/demo.rs:94-106: env_clear, explicit PATH/HOME, system/global config disabled and hooks disabled; init disables templates at :115. Both source_revision queries remove the local Git environment through crates/tide-demo/src/lib.rs:421-449; the list matches this machines `git rev-parse --local-env-vars` output.
- However, the poisoned subprocess sets PROBE_ROOT at crates/tide-demo/tests/demo.rs:130 and GIT_DIR/GIT_WORK_TREE/GIT_INDEX_FILE at :132-134. Its branch at :85-88 calls only source_revision and returns before the writing fixture at :94-120. The unchanged decoy HEAD assertion at :140 therefore checks only the read-only query. It does not protect the original hazard: fixture init/commit writing to the callers repository. Removing the fixtures env_clear at :96 would leave this regression undetected in an ordinary unpoisoned test run.
- Minimum remaining fix: exercise the writing fixture in the poisoned subprocess too, then assert that the decoy HEAD remains unchanged. Keep the existing source_revision routing assertion and avoid recursive subprocess launches. This is the original B1 regression requirement, not an additional scope request.

B2 — RESOLVED.
- crates/tide-demo/src/lib.rs:453-461 uses --no-optional-locks and distinguishes successful clean status, successful dirty status, and failure; failure now prints ` (uncommitted changes unknown)` with HEAD.
- crates/tide-demo/tests/demo.rs:142-153 covers an untracked file and an invalid real index, asserting the dirty and unknown-status outputs respectively. Clean and fallback assertions remain at :120-121 and :158-161.

B3 — RESOLVED.
- README.md:398-400 distinguishes the owners approximate readiness estimate from the timed run. README.md:407 names configuration "Tide", default 2-core machine, US East and d48b1aa, preserves about 10 minutes to readiness, and states exact real runtime 2m13.715s with its rounded equivalent and owner attribution.

Taken non-blocking public-repository wording item — RESOLVED.
- The stale qualification is removed from README.md:357 and docs/nix.md:12, and consistently removed from docs/install.md:35-40. No live availability check was attempted; the reviews PUBLIC premise is retained.

Fix-introduced regressions: none identified. The remaining B1 issue is incomplete protection of the original fixture-safety regression.

Checks: `git diff --check ccc8e6b 2da2c84` passed; read-only inspection and Git local-environment-list comparison completed. No Cargo, Nix, builds, test binaries, external contacts, commits or ledger writes. The fixers reported test/falsifier results were read, not independently rerun. Only this report was written.

Verdict: changes required (B1 regression-test gap only).

DONE
