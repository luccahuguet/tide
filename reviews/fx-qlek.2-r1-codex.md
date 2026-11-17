Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 0d6baa3

Verdict: request_changes

Confidence: high. Read-only supervisor review of `git diff main...0d6baa3`, the affected callers and tests, AGENTS.md, GOVERNANCE.md, bead fx-qlek.2 and its comments, and the builder's report and logs. No override.

What this change does: Builds an amd64 Ubuntu image through the existing Nix installation route and documents trying Tide in it. It also lets tide-demo use tools on PATH when rustup is absent.

The required contract is that this fallback still selects the pinned toolchain, and that the image contains the source needed to try Tide without private checkout state. The ordinary single-user interactive demo is the expected workload.

1. **Blocking — Private checkout state enters the image.** `.dockerignore:3` and `Dockerfile:37`.

   The ignore file excludes only root `target`, `result`, `result-*`, and `.worktrees`. `COPY .` therefore includes the tracked `.beads/` ledger and any local `.beads/` database, despite the explicit exclusion requirement. It also deliberately copies `.git`, including local Git configuration, and does not exclude local credentials such as `.env` or agent-run caches such as `.autorun/`. A clean clone still contains `.beads/`; a user's working clone can additionally expose local credentials to the builder and image. Docker does not apply `.gitignore` to this copy. The recorded context transfer was 84.66 MB, not an allowlisted source export. [Docker's context rules](https://docs.docker.com/build/concepts/context/#dockerignore-files) confirm that exclusions must happen before sending the context.

   Minimum fix: exclude `.beads/` and private local files/caches before context transfer, retaining the existing output/worktree exclusions. Preserve only the Git metadata actually required by the flake, without shipping credential-bearing local configuration, or use a safe source context that does not need the original `.git`. Deleting these files in a later image layer would not fix disclosure. Check the resulting context as well as the image.

2. **Blocking — The fallback does not verify the exact pinned Rust/Cargo pair.** `crates/tide-demo/src/lib.rs:313`, `crates/tide-demo/src/lib.rs:280`, and `crates/tide-demo/tests/demo.rs:104`.

   The compiler predicate checks only a release suffix of `-nightly` and a commit date equal to channel-date minus one day. It never compares the release version or commit hash. For the current pin, any executable reporting a nightly release and `commit-date: 2026-08-08` passes, even with a different compiler commit from `771916f90` recorded in rust-toolchain.toml. A calendar date does not identify a compiler commit. Moreover, `Toolchain::Path` launches whatever `cargo` PATH selects without checking its version at all; a pinned rustc paired with an unrelated stable Cargo passes this gate. A mismatched toolchain can proceed into rustc-private builds or fail later with a generic build error instead of a pin diagnostic.

   The new test positively accepts an arbitrary fake hash and checks only an older date, stable rustc, and an undated channel. It proves the implemented date heuristic, not exact pin enforcement or Cargo selection.

   Minimum fix: validate both tools against the authoritative identities for the pinned channel, including rustc's version and commit hash and Cargo's own identity. Remove the inferred day-before identity rule. Add negative cases for a wrong rustc hash with the same date and a mismatched Cargo, with clear setup failures before either inner build. Keep the existing rustup branch unchanged.

3. **Blocking — Cargo can compile with a different rustc from the one checked.** `crates/tide-demo/src/lib.rs:500` and `crates/tide-demo/src/lib.rs:280`.

   The fallback checks `$RUSTC`, otherwise bare `rustc`, then discards that selection: `Toolchain::Path` stores no compiler path, and the inner Cargo commands do not set one. With rustup absent, RUSTC unset, a pinned rustc on PATH, and `CARGO_BUILD_RUSTC=/other/toolchain/bin/rustc` (or Cargo's `[build] rustc` configuration), the check succeeds but Cargo selects the other compiler. These are supported [Cargo compiler settings](https://doc.rust-lang.org/cargo/reference/config.html#buildrustc). Thus the comment claiming to check “the rustc Cargo runs” is not generally true. The flake's explicit RUSTC makes the recorded happy-path run work, but does not close this fallback gap.

   Minimum fix: retain the resolved, validated compiler path in the fallback and explicitly bind inner Cargo builds to it, overriding conflicting compiler-selection settings. Add a no-rustup regression case with a conflicting Cargo compiler setting; assert that the checked compiler is the one invoked, or that setup refuses the conflict. The current helper-only test never exercises fallback dispatch or inner command construction.

Other review results:

- The Ubuntu digest matches `scripts/clean-install-check.sh:34`. Installation follows docs/install.md's prerequisite, single-user Nix, flake-build and dev-shell route; there is no rustup or second compiler installation. `USER ubuntu` and the recorded uid 1000 confirm non-root runtime use. Apt uses no recommends and removes its lists in the same layer. The recorded 5,849,661,585-byte image is reasonable for the included development toolchain and consistent with the roughly 6 GB disclosure; no additional image-size blocker found.
- docs/docker.md states all four Scope points. Its sandbox line exactly matches `accept/2-sandbox.log`, explicitly describes this Linux host under Docker's default seccomp, and does not promise the same level on Docker Desktop kernels. It discloses Linux-only verification and no observed macOS/Windows run. The documented commands contain neither `--privileged` nor `seccomp=unconfined`. The Apple-silicon emulation/slowness caveat agrees with [Docker's emulation documentation](https://docs.docker.com/build/building/multi-platform/#qemu).
- `git diff 12f03b4 0d6baa3 --stat` reports only `docs/docker.md`, 24 insertions and 15 deletions. The code, Dockerfile, ignore file, flake and runtime examples therefore match the recorded container candidate. `accept/0-env.log` identifies revision `12f03b496f8d1c7720273f460b966be399a4bf1d`, rustup absent, uid 1000, and one active seccomp filter. The report records the container command without a security-profile override.
- All four acceptance logs end with exit 0, and `accept/status` agrees: the sorted-insert example, its `--tide-run-test` sandbox check, `cargo run -p tide-demo`, and the real confinement test. `real_files_build_and_run_against_production_std` lives in tide-admit's integration suite rather than tide-std's own suite, but it builds against production tide-std, exercises real Fs/openat2 confinement and escaping symlinks, and verifies the outside file remains untouched. This supports the builder's stated technical correction to the suite location. The temporary Nix-linker Cargo configuration is disclosed. Demo step 5 explicitly skips gdb; the demo permits that skip.
- The rustup branch still removes RUSTUP_TOOLCHAIN and invokes `rustup run <channel> <program>` as before. The recorded host tide-demo suite reports eight passing tests on that route. The fallback's existing spawn/version failures provide clear setup diagnostics; findings 2 and 3 concern mismatches it fails to detect.
- README/install links resolve, and the Docker guide's driver and dev-shell commands agree with docs/install.md and docs/nix.md. The fallback wording in demo/README.md must be made true by fixes 2 and 3. `git diff main...0d6baa3 --check` passes. No other blocking leftover found in the reviewed scope.

Review identity: HEAD `0d6baa318b6e8bf9e739cbbf3082064cf345cc70`; main `581b0526cc4cc96f899963f539bee9a7fe482f72`; merge base `8f752139b432e5557f8bcb814490253c2d127e19`. SHA-256 of `git diff main...0d6baa3`: `4afffcd6ae2eaaee5217176eddf861ff37199589d6ef6e5265a218bae51e0c4e`. Tracked worktree was clean during review.

Not checked: no builds, Docker/Nix commands, or tests were rerun; runtime conclusions use the supplied logs. No macOS/Windows execution was observed, no subagents were started, and trials/e1-checked/ was not opened. Remaining risks are private context disclosure and incomplete toolchain enforcement described above.

DONE
