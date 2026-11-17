Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate ef2cc49

Reviewed `git diff 81dbeed...ef2cc49` (11 files). HEAD is `ef2cc493fa62408e5903ee3f02e0ccf15bc8c8ef`, the worktree is clean, and the diff's SHA-256 matches `4089cff6411bede8b6eadc9a6d7b9749cc3af564f2c62a9555e21f911a1244f0`. I read AGENTS.md, the bead's Contract and Acceptance (`br --no-db show fx-qlek.14`), report.md, r1-codex.md, r1-recheck-codex.md, the builder's manual churn logs, the changed code with its callers (`compile`, `compile_with_library`, the single-crate cache path, `run_test_process`, `trusted_inputs`, `rustc_args`, `fingerprint`), the replay driver test, and every changed doc. I also grepped the rest of the docs for leftovers. I ran no builds or tests, started no subagents, and changed no repository files.

## Verdict

approve

## Blocking findings

None.

## Non-blocking findings

1. **No driver test covers the "build loaded a different set" refusal** (`crates/tide-admit/src/invocation/replay.rs:208`, test loop `crates/tide-admit/tests/build.rs:4677`). The pre-build check (`current_artifacts`) re-hashes only the names the recording lists. A library the new build loads that the recording does not name is caught only by `manifest.build.compare(&receipt.build)` after the build. That line predates this change. Before it, the full-build pre-check made it largely redundant; now it is the only guard for added libraries, and design/03 states the guarantee ("then refuses if the build loaded a different set"). If someone deleted the line, no test would fail. Failure scenario: a later refactor drops the post-build compare, and a recording that omits a loaded library replays without complaint. In practice the risk is low: std's metadata pins its crate graph by SVH, and std has its own full hash. Cheapest fix: one more arm in the existing component loop, e.g. `"dependency" (added)` → `bad.build.dependencies[1].pop_last();`, then expect exit 2, "dependency artifact" and empty stdout. Driver-only stdout (the child compile's stdout is cached and not printed before `run_test_process`) keeps the "candidate not started" assertion valid.

2. **Acceptance 1 is proven at driver level only on the single-crate, cached path.** The churn replay in `replay_manifests_validate_identity_and_reproduce_exact_evaluations` uses `driver()`, without `--tide-test-library`. The demo uses the paired path (`compile_with_library`, `crates/tide-admit/src/invocation.rs:597`). That path has no cache, and its loaded set is the union of two compilations. The shared functions (`loaded_identity`, `current_artifacts`, `compare`) are the same, and the paired path's record/replay agreement is exercised by the paired replay test and by `demo/run.sh`. Its tolerance of churn rests on the builder's manual demo check (churn-replay2.log, exit 101, `replay reproduced`). That check is acceptable evidence. A paired-path churn assertion would be stronger, but its absence does not block.

3. **Some error paths read poorly** (`crates/tide-admit/src/invocation/replay.rs:153`). `path.exists()` returns false on any stat error, so a recorded library in an unsearchable directory (EACCES) is reported as `current missing` rather than as a permission error. For a recorded entry that exists but cannot be opened, or is now a directory, `file_digest` returns a bare io error ("Permission denied (os error 13)", "Is a directory (os error 21)") without the path. Both still refuse, with exit 2 and no candidate launch, so the result is safe. The `file_digest` wording matches the old `artifact_tree`. Nit: map the error to `"{path}: {e}"`.

4. **The bead's demo-user friction is narrowed, not removed** (`docs/owner-guide/06-replay.md:65`). After `demo/run.sh`, a plain whole-workspace `cargo build` in the same target dir rebuilds std into different bytes, and replay refuses (the builder observed this: churn-build.log, churn-replay.log). Under the Contract this refusal is correct, because a loaded library really changed, and the guide says so honestly. The README rewrite (fx-qlek.3) should repeat it wherever it tells users to build, as the builder suggests.

## Reasoning

**Soundness of the identity.** The loaded set comes from `tcx.crates(())` and `crate_extern_paths` in `after_analysis`. That covers every CrateSource path (rlib, rmeta and dylib) of every crate rustc registered, transitive crates included, and the proc-macro dylib. I checked each class of input in the brief:
- **Transitive dependencies, features, rustc flags and build-script outputs.** Each changes the bytes or the file name of a loaded rlib or rmeta. A change in bytes refuses before the build. A change in name changes std's bytes (std's metadata pins its dependencies by SVH), or else changes the loaded set, which the post-build compare refuses.
- **Proc-macro dependencies.** syn, quote and proc-macro2 are statically linked into the `.so`, whose bytes are hashed. rustc does not register a proc-macro's own dependencies in the crate store, so hashing the dylib is the right granularity.
- **tide-std and the driver.** Each keeps its own full digest (`std`, `driver`).
- **The sysroot and the C toolchain.** These stay under the existing documented assumption that they are immutable, with the compiler id string standing for the sysroot.
- **Linker search.** rustc passes only `Native` and `Framework` search paths to the linker, not `-L dependency=` paths, so unloaded files in a dependency directory cannot reach the link.
- **Unloaded candidates.** A stale candidate of the same crate either fails the SVH match (not loaded) or makes rustc error on multiple candidates. It cannot be substituted silently.

I found no input that can change replayed behavior and escape both the pre-build re-hash and the post-build set compare. Hashes are taken after the build. A swap during the build leaves a receipt that differs from the manifest, so replay refuses. A swap that is restored within one build is out of scope, and the report says so.

**Attribution.** After the round-1 fix, rustc's own spelling is used: the search path, canonicalized by `trusted_inputs`, joined with the entry name. Matching is done by `path.parent() == dep`, with no alias scan. The driver test's symlinked library would hit "in no --tide-dep-dir" if rustc canonicalized the path, so the passing test also proves the locator behavior empirically. Names are validated as single file names before joining (`..`, `a/b`, an absolute path and an empty name are all rejected). Duplicate dependency directories go to the first match, the same way on both sides.

**False refusals the bead targets.** Incremental caches, depfiles, unloaded rlibs, the driver binary in `target/debug`, and lock or cache files no longer enter the identity. In the single-crate path, the cache key still fingerprints whole directories, so churn costs a recompile but no refusal, which is correct. With Cargo's per-unit `build/<pkg>/<hash>/out` directories (visible in the demo's replay command), each directory holds one crate, which also keeps the diff lines unambiguous.

**Error paths.** A missing dependency directory still fails in `trusted_inputs`. An empty one produces empty maps on both sides. The paired path no longer walks directories, so a symlink loop inside a dependency directory no longer matters there. The single-crate `fingerprint` walk is unchanged and ran before the old `artifact_tree` anyway, so this change neither adds nor fixes a loop hazard there. Missing or changed recorded libraries refuse with short lines.

**v1.** `tide-replay/1` decodes (same schema) and is refused first in `validate`, with a re-record message whose seed is printed only as a canonical decimal. Other formats get `expected tide-replay/2`. Old `tide-build/1` cache entries cannot be reused, because the cache key fingerprints the driver executable and the receipt carries the driver digest. Refusing v1 rather than supporting it is a reasonable decision before the alpha ships, and design/03 records it.

**Refusal format.** `Build::compare` reports the first differing component, one escaped line per entry, digests cut to 12 hex digits, at most 8 lines plus `and N more`, and no JSON. The unit test pins the format, the cap and the escaping. The driver test pins the default-path output for a changed and a missing library (≤3 lines, no `{`, no full digest).

**Acceptance.** Item 1 (churn replays reproduce) and item 2 (a real change refuses with the short diff) are both proven by driver tests in the workspace suite, on the single-crate path (see finding 2). The builder reports workspace 349/0, conformance 386 PASS / 3 AWAITS CHECKER, and `demo/run.sh` exit 0 on ef2cc49. I did not rerun these.

**Docs and leftovers.** These now describe the new identity and refusal: design/03's driver precursor paragraph, owner-guide/06 (including a real refusal example and the removed "no other build in between" warning), README.md, demo/README.md, demo/agent-card.md, conformance/README.md and the tide-std module doc. No unhistorical `tide-replay/1` mention remains outside `research/` and `reviews/`, and no `artifact_tree` caller is left. REPLAY-2's "before executing anything" still holds: the post-build set check runs before the candidate launches, and design/03 states the order.

**Complexity.** The change is proportionate. Deleting `artifact_tree` and the alias scan leaves the identity as "what rustc loaded, by name, with hashes". `compare`'s if/else chain replaces the macro to produce per-entry lines, and it is no larger than it needs to be.

DONE
