Model: claude-opus-5-5 · Date: 2026-11-16
Verdict: approve

Candidate: 84cd497 on fx-abv5. I reviewed `git diff main...HEAD` (conformance/run.sh, conformance/README.md) against bead fx-abv5's Scope and Acceptance, and formed this view before reading report.md and r1-codex.md. I also read tide-admit's path handling in invocation.rs (package_root, trusted_inputs, destination, cache key) and its build.rs and tests, and grepped scripts/, trials/ and the docs for other consumers of the conformance target paths.

Checks run (none compile anything): `bash -n conformance/run.sh` passes. I copied lines 12-22 into a scratch script and resolved the paths with CARGO_TARGET_DIR unset, empty, `rel dir` (from a cwd that contains a space), `/mnt/build/x/`, `./t` and `../t`. Unset and empty give main's paths. Relative values are anchored to `pwd -P` before `pinned()` cds to $ROOT. Spaces survive. A trailing slash or `..` leaves non-normalized strings, and they are harmless: the driver canonicalizes `--tide-std`, `--tide-dep-dir` and `--out-dir` (invocation.rs:426, 432, 594-609), `ADMIT_DEPS` comes from Cargo's own JSON, and no path is compared as a string in the script.

Contract points checked:
- Unset or empty behaves as on main: TARGET=$HERE/target, the driver uses `--target-dir $ROOT/target`, and DRIVER=$ROOT/target/debug/tide-admit (run.sh:15-16, 51-52).
- When the variable is set, every runner-controlled output moves under it: the driver build (:51), the oracle stub (:47), the pinned stub (:58), case outputs and the driver's `.tide-cache`, which lives under `$OUT/admit` (:46, 103, 128). The driver writes no ICE files (env holds only RUSTC_ICE=0). The author's before/after snapshots match this.
- The toolchains stay apart. Pinned-nightly artifacts sit in $CTD, stable artifacts in $CTD/conformance, and pinned-stub artifacts in $CTD/conformance/admit, all separate Cargo roots. They nest the same way the default `conformance/target/admit` does. The name `conformance` does not clash with anything Cargo creates.
- The executed binary is the one that was built: DRIVER follows DRIVER_TARGET, so the fx-kw8.1 stale-binary class does not come back. Moving between target dirs cannot reuse stale cache entries, because the driver's cache key includes a fingerprint of the driver executable, the std and the dependency directories (invocation.rs:1017-1050).
- Sharing with `cargo test --workspace`: with the same CTD, the driver build reuses the pinned workspace target, which is the point. The tide-admit tests write only under CARGO_TARGET_TMPDIR (`$CTD/tmp`). No test or script reads conformance/target.
- The README matches the script: the default paths, the redirected `conformance/` subtree, separate toolchains, relative and empty values, and the cache location (README.md:17-24, 66-68).
- Leftovers: none. No other file names the old paths.

Findings:

1. conformance/README.md:20-24 (and run.sh:51-52). Non-blocking. Failure scenario: two worktrees point CARGO_TARGET_DIR at the same directory. Worktree B's `cargo build`/`cargo test` uplifts B's `debug/tide-admit` after run.sh in worktree A has built A's driver but while A is still running cases. A's remaining cases then run B's driver, and their output files in `$CTD/conformance/conformance` collide too. On main this could not happen with CARGO_TARGET_DIR set, because conformance ignored the variable. The build-disk convention uses one target dir per bead (`/mnt/build/tide-agents/<bead>-target`), and heavy.sh gives full suites a single slot, so this is outside the contract, not a defect in it. One README clause ("use a target directory per checkout") would close it.

2. conformance/run.sh:17. Nit, non-blocking. Cargo's other target-dir sources (`CARGO_BUILD_TARGET_DIR`, `build.target-dir` in config) are still ignored, because the explicit `--target-dir` overrides them. That is the same as main, and the bead names only CARGO_TARGET_DIR, so no change is needed.

Acceptance: the author's set and unset full runs (report.md) both exit 0 with 361 PASS, 4 AWAITS CHECKER, 0 FAIL, and identical ordered verdicts. The worktree's target/ and conformance/target/ stayed absent across the set run. I did not rerun them (per the brief). Their candidate diff hash covers the reviewed commit.

DONE
