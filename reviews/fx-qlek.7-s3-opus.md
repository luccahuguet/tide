Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 43bffd0 (code, S3)

## Verdict

Changes required. The code is sound. The flake app, the try and dev stages and the devcontainer
do what they say, and the rebase is textually clean. Four findings block. Each is a docs or claim
fix in README.md, docs/nix.md or a flake.nix comment, and needs no rebuild. Two of them are
commands or measurements that do not hold for an ordinary rustup user. Separately, tide-lead has
one landing check to rerun, because the rebase changed the demo's store inputs (see "Landing
condition").

## Scope

- Read: AGENTS.md; GOVERNANCE.md's Merge Rule; `br --no-db show/comments fx-qlek.7`, which hold
  the relayed try-it order, the honest-times rule and the owner-only GHCR and Codespaces steps;
  target/work/report.md, sections S3 and S3 r1; s3-codex.md and s3-recheck-codex.md.
- Diff: the whole of `git diff 766eb17 43bffd0`, all seven files. I also compared it with
  `git diff 46297dd 2e7635e`: the two differ only in index lines and one hunk offset, so the
  rebase changed nothing textually. `git diff --stat 46297dd 766eb17` shows what main changed
  under the candidate.
- Read the logs behind every README and docs/docker.md number: rustup, nix run, docker build,
  the docker run timestamps, the devcontainer log and the rustup-check scripts.
- Read the demo's revision, gdb and toolchain detection in crates/tide-demo/src/lib.rs, and
  crates/tide-demo/tests/demo.rs.
- Ran: `nix eval --offline` of `apps.x86_64-linux.tide-demo.program` and of the package's
  drvPath on 43bffd0 (both evaluate), `git diff --check 766eb17 HEAD` (clean), and an FX-code
  grep (none). I built nothing and ran no Docker command.

## Blocking findings

### B1: The flake app honours `$XDG_CACHE_HOME`, and the docs say it does not

**Location:** flake.nix:205 (code); README.md:330, docs/nix.md:10, docs/nix.md:95 and the
flake.nix:183 comment (text).

**Failure scenario:** The wrapper's default target is
`${XDG_CACHE_HOME:-$HOME/.cache}/tide-demo`. Every text says the default is `~/.cache/tide-demo`.
Take a user with `XDG_CACHE_HOME=/data/cache`, a common setting:

- `nix run github:luccahuguet/tide#tide-demo` writes about 1.3 GiB, plus its Cargo home and
  TMPDIR, under `/data/cache/tide-demo`.
- README step 3 says it writes under `~/.cache/tide-demo`.
- So the user looks in the wrong place for the files to delete, and the documented write place
  is wrong.

**Minimum fix:** Pick one, and change all four text sites together.

- Document it: "under `$XDG_CACHE_HOME/tide-demo` (by default `~/.cache/tide-demo`)".
- Or drop the XDG fallback from line 205.

Either is a text or one-line change and needs no new run.

### B2: The rustup command fails with rustup older than 1.28

**Location:** README.md:317-323.

**Failure scenario:** `rustup toolchain install` with no toolchain argument installs the toolchain
that rust-toolchain.toml names only from rustup 1.28.0 on. That version came with the change
that stopped `rustup show` installing it. Before 1.28, the command fails because a required
argument is missing.

- The prerequisites say only "rustup", and step 2 is the path for "Rust developers".
- A distribution rustup can be older. I believe Ubuntu 24.04's `apt install rustup` gives 1.26,
  but I did not check this, since network access was off.
- Such a user hits an error on the second line. With rustup before 1.28, `cargo run` would have
  installed the toolchain by itself.
- The measured run used rustup 1.29.3 (report §3.4).

**Minimum fix:** In the prerequisites, say "rustup 1.28 or later". Or add a comment that older
rustup installs the toolchain on the first `cargo run` and the line can be skipped.

### B3: The rustup row's size and time were measured with rustup's minimal profile, and the README does not say so

**Location:** README.md:348, and README.md:323's comment.

**Failure scenario:**

- rustup-check/user.sh:9 installs rustup with `--profile minimal`. s3-rustup-clean-container.log:8
  confirms it ("profile set to minimal").
- rust-toolchain.toml names no `profile`. So an ordinary rustup.rs install installs the pinned
  nightly with the default profile: rust-docs, clippy and rustfmt, in addition to rustc-dev and
  rust-src.
- That user gets more than "about 1.4 GiB in `~/.rustup`" and more than "19 s to install the
  toolchain". The rust-docs component alone is several hundred MB unpacked.
- The README states its conditions (host, caps, bandwidth) but not this one, so the size and time
  do not hold for the typical reader. That breaks the relayed honest-times rule.

**Minimum fix:** State the condition in the row or the intro, for example "with rustup's minimal
profile; the default profile adds the docs, clippy and rustfmt". Alternatively, re-measure with
the default profile, or pin `profile = "minimal"` in rust-toolchain.toml. The pin changes what
every rustup user installs, so it is not my recommendation here. While editing, the line-323
comment can say "with rustc-dev and rust-src", as the pin does.

### B4: The README cites logs that the bead does not hold and that disappear at landing

**Location:** README.md:338-339 ("bead fx-qlek.7's stage S3 records the logs").

**Failure scenario:**

- The logs are only in target/work/ of this worktree. That directory is git-ignored and is
  deleted when the worktree is removed after landing.
- The bead's comments so far hold summaries only, with no numbers and no logs.
- So once the change lands, the README's only citation for its published timing and size claims
  points at nothing. AP-PROOF-001 says a temporary path is not proof, and says to keep logs for
  performance-sensitive claims. fx-qlek.7's Acceptance asks that every claim cite a landed report.

**Minimum fix:**

- Reword to "records the measurements and their conditions".
- Before removing the worktree, tide-lead records report.md's S3 sections on the bead. They hold
  every number and condition.
- Better: also land the small timestamp logs beside the S3 review files under reviews/:
  rustup-clean-container, nix-run, docker-run-try-timestamps, the docker build step timings,
  devcontainer `up`/`exec` lines, and the gzip size.

## Landing condition (coordinator, not an author finding)

The rebase changed inputs of `packages.tide-demo`:

- Main's fx-qlek.20 changed crates/tide-std: sources, the scheduler's rand sub-stream, and
  rustix's `rand` feature in tide-std/Cargo.toml. `demoSrc` includes `./crates`.
- On 43bffd0 the demo's output path (/nix/store/j1fmsapb…-tide-demo) is not in the store. So no
  `nix run .#tide-demo` has run on the rebased tree.

The risk is low:

- Cargo.lock, flake.lock, demo/ and the toolchain pin are unchanged, so the vendored crates are
  the same, and rustix's `rand` feature adds no crate.
- The workspace suite's crates/tide-demo/tests/demo.rs runs the demo through the Cargo route, and
  it passed on fx-qlek.20's tree.

Still, under AGENTS.md's Landing rule, rerun the cheapest check of the store-source route on
43bffd0 (or on the fixed candidate): `nix run .#tide-demo` through heavy.sh, about 50 s. The
README's times describe 46297dd's std, which is fine because they are dated, and need no rerun.

## Non-blocking findings

1. **Sizes leave out build output.** The Size column gives ~/.rustup and ~/.cargo, the Nix
   closure and the image, but not what each run writes:
   - target/ is 1.3G after the rustup runs (s3-rustup-clean-container.log:195), and
     ~/.cache/tide-demo holds about the same.
   - Building the try image leaves its nix and try-closure stages in Docker's build cache, with
     the closure twice (store, then ~/try).

   docs/docker.md gives a free-disk figure for the dev image only. One clause per row ("plus
   about 1.3 GiB of build output in …") and a build-cache note would make "on disk afterwards"
   complete. The build-cache size was not measured.
2. **The same text is kept in several places (AP-MINIMAL-001).**
   - The write boundary is described four times: the flake.nix comment, docs/nix.md (table and
     §Demo), README step 3 and docs/docker.md. B1 is a drift between them.
   - The try-image and devcontainer times appear in both the README table and docs/docker.md
     §Measured times.

   One owner each would be cheaper to maintain: docs/nix.md §Demo for the boundary and the
   README table for the times, with the others linking to them.
3. **README step 3's "It … writes under …" leaves out Nix's own writes**, in /nix/store and
   ~/.cache/nix. docs/nix.md scopes the same sentence to the package. Saying "the demo writes
   under …" would fix it.
4. **The caller's environment passes through, apart from PATH, CARGO_HOME and TMPDIR.** Examples
   are RUSTFLAGS, CARGO_BUILD_*/CARGO_PROFILE_* config variables, RUST_GDB and GIT_DIR. "A
   caller's ~/.cargo and its configuration are neither read nor written" is true of the files,
   but Cargo's environment configuration still applies. This is acceptable for a try-it command,
   and a word in docs/nix.md would make it exact.
5. **Line endings on Windows (pre-existing, from fx-qlek.2).** A clone made with Git for
   Windows' default `core.autocrlf=true` gives the build context CRLF files, including
   demo/patches/*.patch, which the demo `git apply`s. No Windows run has been observed, as the
   docs say. A `.gitattributes` with `* text=auto eol=lf`, or a clone hint in docs/docker.md,
   would remove the risk.
6. **Codespaces specifics are unverified, and are labelled as such.**
   - In a codespace the flake is a git checkout at /workspaces/tide, not the image's
     .dockerignore'd copy.
   - Nix's git fetcher there depends on how the workspace is owned. The local devcontainer CLI
     remapped the UID, and Codespaces may not.
   - The prebuilt demo closure is reused only when the store paths match. demoSrc has no tracked
     dotfiles, so they should.

   These belong to the owner's row 3.7, after the repository is public.
7. **Nits.** docs/nix.md's "Only Cargo comes from the joined toolchain" means the separate cargo
   component (`toolchain.availableComponents.cargo`). The flake comment at flake.nix:187-188 has
   a stray line break ("on the\n# caller's PATH").

## Reasoning

### Paths, as a first-time user

1. **Codespaces.**
   - The badge and `codespaces.new/luccahuguet/tide` URLs are in the standard form.
   - devcontainer.json builds the `dev` target from the repository root, with remoteUser ubuntu
     (uid 1000 in ubuntu:24.04).
   - The dev stage keeps PATH=~/.nix-profile/bin and nix.conf's experimental features, so
     `nix run .#tide-demo` works in a terminal as written.
   - The recorded check (devcontainer CLI 0.90.0 `up`, then `exec … nix run .#tide-demo`, exit 0
     in 49 s, as uid 1000 in /workspaces/…) proves what docs/docker.md and the README claim: a
     local devcontainer build. Both texts label Codespaces itself as not yet measured, and the
     NIX_CONFIG build-arg deviation is disclosed.
2. **rustup.** The commands and the cc/ld prerequisite match the logged run, which includes the
   expected failure without build-essential. B2 and B3 are the gaps.
3. **nix run.**
   - The app evaluates offline on 43bffd0.
   - The sources come from a store fileset with no tracked dotfiles. The vendored config is
     copied and `cargo-vendor-dir` is linked beside it. There are no git dependencies in
     Cargo.lock, and the build uses `--offline --locked`.
   - PATH is replaced, so the demo takes its non-rustup branch and checks RUSTC (the sysroot's
     rustc) and cargo against the pin.
   - A relative CARGO_TARGET_DIR is resolved against $PWD before the `cd`.
   - CARGO_HOME and TMPDIR are moved under the target. The /tmp exception (the driver's
     `env_clear` for the linker) is real, and the read-only-root runs show it.
   - When no dev shell is present, nothing breaks, because nothing is taken from the caller but
     the environment (non-blocking 4). With HOME unset and no CARGO_TARGET_DIR,
     writeShellApplication's `set -u` stops with a clear error.
   - B1 is the one contradiction.
4. **Docker try.**
   - try-closure copies exactly `nix-store --query --requisites` of the app. /bin/tide-demo is an
     absolute symlink into the copied store, and /tmp is 1777.
   - The final stage is `FROM scratch`, so there is no base image to pin. It runs as 1000:1000
     with HOME and CARGO_TARGET_DIR under /tmp, and nothing secret or host-specific is copied.
     NIX_CONFIG is used only in the intermediate stage's RUN.
   - I checked that the `NIX_CONFIG` build argument still reaches the child stages
     (`FROM nix AS …`). Nix's default max-jobs is 1, yet the logged closure stage started
     several large fetches and unpacks at once: rustc-dev and rust-std tarballs 0.4 s apart, and
     three unpacks at 26.6 s. So the documented `--build-arg` cap applies, and "Nix builds were
     capped" holds.

### Dev target

- It is still the last stage, so a plain `docker build .` gives it.
- It inherits USER, WORKDIR, SHELL, PATH and the ARG from `nix`.
- It runs the same `nix build .#tidelang-admit && nix develop --command true` as before. The
  added `nix build --no-link .#tide-demo` does not touch `result`.
- Its behavior is unchanged apart from the pre-built demo closure. The 6.09 GB measured from the
  devcontainer build is the new image's size.

### Claims against the logs

These numbers match the logs:

- 19 s toolchain install (+3 → +22 s).
- 51 s and 16 s Cargo runs.
- 48.09 s nix run.
- 80.4 s closure stage; 69/12/7 s build steps.
- 36 s try run (724 → 760 s, queue excluded).
- 360 s `up` (209 s export ≈ 3.5 min) and 49 s exec.
- 2,424,731,888 bytes for the image, and 769,099,206 bytes gzipped.

The caps sentence matches the scripts. The remaining gaps are B3's unstated profile, B4's
citation and non-blocking 1's omissions.

### Docs agreement and relayed rules

- The order is Codespaces, rustup, nix run, Docker, as relayed.
- There is no "60 seconds" claim and no Docker one-liner.
- GHCR is labelled as the owner's step, and the public-URL dependency is stated.
- No FX codes appear.
- demo/README.md, docs/install.md and docs/tide-or-rust.md still point at docker.md correctly.
- The rebased README reads coherently with fx-qlek.20's Rand edits, which sit in later sections
  that S3 does not touch.

The four blocking fixes are text-only, so under AP-PROOF-001 they need no finder re-check beyond
reading them. The landing rerun above is a separate requirement.

DONE
