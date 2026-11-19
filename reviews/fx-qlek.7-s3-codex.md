Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate da0fcaa (code, S3)

## Verdict

Changes required. The normal-path runs are supported, but the output-directory handling and several published measurement or image claims need correction. Confidence: high on the relative-path failure and documentation mismatches; write-confinement conclusions are from inspection, not a traced run.

## Blocking findings

### B1 — A relative CARGO_TARGET_DIR resolves inside the read-only store

**Location:** flake.nix:201, flake.nix:202.

**Failure scenario:** A caller uses the ordinary Cargo setting `CARGO_TARGET_DIR=target nix run .#tide-demo`. The wrapper preserves that relative string, then changes into `demoSrc` in /nix/store. Cargo consequently tries to create its target there and fails on the read-only sources. The same setting works for the README's Cargo route. The successful Nix evidence used an absolute target; the devcontainer used the absolute default.

**Minimum fix:** Resolve a relative target against the invocation directory before changing directories, then export the absolute path. Check one relative override as well as the default when verifying the fix.

### B2 — Setting the target does not confine all application writes

**Location:** flake.nix:198, flake.nix:201; Dockerfile:57.

**Failure scenario:** With an otherwise clean environment, Cargo's home remains `$HOME/.cargo` and compiler/toolchain temporary files use the normal temporary directory. Neither location is moved under the target. In the try image the defaults are therefore /tmp/.cargo and /tmp, while the stated target is /tmp/tide-demo; for the host app they are likewise outside ~/.cache/tide-demo. Vendoring and --offline prevent dependency fetching; they do not establish the requested “writes only to CARGO_TARGET_DIR” contract.

This is a code-inspection finding: the supplied runs report artifact locations, not filesystem writes. Cargo's pinned executable also contains its package-cache/global-cache machinery; no clean-home or temporary-write confinement check was recorded.

**Minimum fix:** After making the target absolute, create application-owned Cargo-home and temporary subdirectories beneath it and direct CARGO_HOME and TMPDIR there for the invocation and its children. Verify the resulting write boundary on a clean run. This concerns the demo process and its children, not Nix's own store/cache activity while obtaining the app.

### B3 — The devcontainer timing is described with CPU/job caps the recorded invocation does not apply

**Location:** README.md:335; docs/docker.md:56.

**Failure scenario:** Both pages say the measurements used four-CPU containers and four Cargo jobs. The devcontainer check's actual `docker run` command in target/work/s3-devcontainer-check.log has no --cpus limit. Neither the committed config nor target/work/capped/devcontainer.json sets runArgs, containerEnv or remoteEnv to supply that CPU limit or CARGO_BUILD_JOBS=4. The recorded exec is simply `bash -c 'id; pwd; nix run .#tide-demo'`. The host heavy.sh environment is not automatically passed through Docker into that exec. The capped config does apply NIX_CONFIG cores=4/max-jobs=4 to the image build, which is a different boundary.

**Minimum fix:** Qualify the existing 360 s / 49 s measurement with the conditions actually applied: Nix build caps, with devcontainer runtime CPU/Cargo-job caps not established. Correct the blanket claims in both pages. A rerun is only needed if retaining the stronger four-CPU/four-job comparison.

### B4 — The rustup “downloads” number is a different measurement

**Location:** README.md:338, README.md:343.

**Failure scenario:** The Downloads column calls the measurement “83 MB of crates.” The log and target/work/rustup-check/user.sh measured `du -sh ~/.cargo`, which reports 83M of installed Cargo-home disk usage after both runs. That includes the rustup installation, registry metadata, compressed crate archives and unpacked sources; it is not a measurement of crate-download bytes. The recorded 1.4G toolchain figure is likewise a disk measurement, already described as such, with binary rather than decimal units.

**Minimum fix:** Label these observations as disk usage (approximately 1.4 GiB for ~/.rustup and 83 MiB for ~/.cargo), and separate them from download-volume claims. No new download benchmark is necessary.

### B5 — The try image does contain a shell

**Location:** docs/docker.md:23.

**Failure scenario:** The guide says the try image has “no shell.” The actual demo launcher is a Bash script: the built launcher at /nix/store/g1asa2ybsf0rfbp31rgja9x1h3v3ijwv-tide-demo/bin/tide-demo starts with `#!/nix/store/gik3rh1vz2jlgnifb9dh6vc6sxwwz9jj-bash-5.3p9/bin/bash`. Dockerfile:49 copies the launcher's full requisite closure, including that Bash. The image has no conventional /bin/sh installation and starts the demo rather than an interactive shell, but it is not shell-free.

**Minimum fix:** Remove “and no shell,” or say it starts the demo and has no /bin/sh. Keep the dev-image instructions for an interactive development environment.

## Non-blocking findings

1. **Inherited build bootstrap is not fully pinned:** Dockerfile:18 and Dockerfile:35 retain moving Ubuntu apt repositories and https://nixos.org/nix/install. These predate S3; the Ubuntu base digest and all flake inputs are pinned, and the final try contents are the pinned Nix runtime closure rather than those bootstrap packages. The whole Docker build recipe is therefore not fully reproducible even though its intended runtime closure is fixed. This is an inherited limitation, not a newly unpinned S3 image/input. Do not describe the entire dev image as bit-for-bit reproducible.

2. **One historical FX code remains outside S3's edited docs:** docs/owner-guide/10-v0-and-planned.md:87 quotes FX0516 in an archived bead-title listing, explicitly explained later as an old spelling. It already exists at 46297dd. The four reviewed S3 documents contain no FX#### diagnostic codes. If “only TD-prefixed codes in live docs” is meant literally across all docs, this existing quotation remains for the broader claims pass.

## Reasoning

What this change does: It provides the four agreed try-it routes in order, adds a Nix-owned demo launcher, reuses the dev image for Codespaces, and copies the demo's runtime closure into a smaller scratch-based Docker target.

### Evidence and identity

- Read AGENTS.md, GOVERNANCE.md, `br --no-db show fx-qlek.7`, `br --no-db comments fx-qlek.7`, the S3 brief/account, checklist rows 3.1–3.6, all seven changed files, the demo's relevant implementation and the recorded S3 logs.
- HEAD was da0fcaade6e43f9cbcca778bbef334f3e74415e5, on the supplied candidate, with a clean tracked worktree. `git diff --check 46297dd da0fcaa` passed.
- The devcontainer proof used 3e0eb6a. `git diff --stat 3e0eb6a da0fcaa` shows only README/docs wording changes. Compared flake.nix, the demo implementation, manifests, lockfile, toolchain pin and service against the saved devcontainer clone: all match.
- Offline evaluation of `apps.x86_64-linux.tide-demo.program` succeeded and selected /nix/store/6787lg49k7x8jc48i4dfzwnanfq973xw-tide-demo/bin/tide-demo. Evaluation did not build it.

### Paths and behavior

- The normal Nix route has recorded exit 0 in 48.090 s, from store sources with an absolute cold Cargo target. The source's .cargo/config.toml replaces crates.io with cargo-vendor-dir, which points at importCargoLock's vendored dependencies. The top invocation uses --offline --locked; nested demo Cargo builds use --locked and the same source configuration. RUSTC, the linker and runtime library directories match the dev-shell settings. PATH excludes the caller's rustup, and the no-rustup identity check matches the pinned compiler/Cargo. No new flake input was added.
- The rustup check used a clean pinned Ubuntu container without Nix, recorded the expected missing-cc failure, then succeeded after build-essential. The explicit rustup step, git and /usr/bin/cc plus /usr/bin/ld prerequisites agree with the implementation. The logged successful runs are 51 s cold-target and 16 s warm-target; the toolchain installation is 19 s. The earlier failed run had already populated some Cargo cache, so these are not proof of a completely untouched crate/index cache.
- The try build completed with exit 0 in 3m17.673s. Its closure stage took 80.4 s; apt took 69.0 s and Nix installation 12.0 s. The timestamped demo log runs from 724 s, when heavy.sh acquired its slot, to 760 s: 36 s of execution, including 21 s between the toolchain line and driver/std readiness. Queue delay is correctly excluded.
- Docker image inspection confirmed linux/amd64, uid/gid 1000:1000, /tmp working directory, HOME=/tmp, CARGO_TARGET_DIR=/tmp/tide-demo and CMD /bin/tide-demo. Its size is exactly 2,424,731,888 bytes, matching 2.42 GB. The saved gzip measurement is 769,099,206 bytes, matching 769 MB; this is a compressed docker-save measurement, not a measured registry pull. History shows the closure COPY and configuration, without Ubuntu/Nix build layers in the try image. No secret or host workspace path is introduced by the reviewed recipe.
- The dev target remains the last/default target, with its prior Ubuntu user, working directory, driver result link, realized dev shell and bash command. Its added demo realization does not replace those behaviors.
- The committed devcontainer selects Dockerfile target dev, context .., remoteUser ubuntu, and adds no base image or feature. The local CLI logged successful up in 360 s and demo exec in 49 s. Its additional build-argument cap is disclosed accurately in the builder's account; the stronger runtime-cap claim is B3.

### Documentation and dependencies

The README order is Codespaces, rustup, Nix, Docker. The public-repository dependency, unmeasured Codespaces end-to-end step and owner-only registry push are labelled. No Docker-one-liner or “60 seconds” claim remains in the reviewed live documents. The README commands match the app/target names. The closure sizes and dev-image size are recorded in the builder's account; the try size is also independently inspectable as above. Linux-only testing and macOS/Windows emulation limits are disclosed.

The new launcher reuses the locked nixpkgs and rust-overlay inputs; docs/nix.md records its use of writeShellApplication, gitMinimal and coreutils. Scratch adds no image dependency. The existing Ubuntu digest is unchanged. Devcontainer CLI 0.90.0 was an exactly versioned verification tool, with its purpose and lack of a repository dependency recorded in the S3 account.

No builds, Rust execution, image runs, publication, commits, bead writes or contact with other sessions were performed. Codespaces end-to-end, GHCR, macOS/Windows execution and an actual filesystem-write trace remain unchecked; the report relies on the supplied runs and bounded read-only inspection.

DONE
