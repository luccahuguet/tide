Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate b919d06

# fx-qlek.2, round 2: Opus converge review (report only)

Scope reviewed: `git diff main...b919d06` (merge base 8f75213). That covers the Dockerfile,
.dockerignore, docs/docker.md, the link lines in docs/install.md and README.md,
crates/tide-demo/src/lib.rs and tests/demo.rs, and demo/README.md. I also read the bead and its
comments, GOVERNANCE.md's Merge Rule, target/work/report.md, accept.sh, the accept-r1 logs,
build-r1.log, demo-test-r1.log, r1-codex.md and r1-recheck-codex.md. I ran no builds and no docker
or nix commands, and I did not open trials/e1-checked/.

## Verdict

**approve**

No blocking findings. The non-blocking findings below are text fixes, a test-strength gap, a
proof-record gap, and one runtime cost that I could not confirm without running the image. N1 is a
condition for landing, not a code change: the coordinator's rebase rerun of `cargo test -p
tide-demo` also closes it.

## Blocking findings

None.

## Non-blocking findings

**N1. Proof record: the host `cargo test -p tide-demo` run predates b919d06 and the last write to
lib.rs** (target/work/demo-test-r1.log; report.md "Reruns on b919d06")
- demo-test-r1.log was last written at 21:15:41 -03.
- `crates/tide-demo/src/lib.rs` was last written at 21:15:52.9 -03. The target dir's
  `debug/tide-demo` binary was linked at 21:15:52.85, which matches a mutation build followed by
  the restore.
- b919d06 was committed at 21:16:29 -03.
- So the two new unit tests' pass is tied to b919d06 only by the builder's statement that the
  restore was byte-for-byte identical (`cmp`). No hash or revision was recorded.
- The container checks are not affected. clone-r1 is at b919d06 and clean, and the build and
  accept-r1 runs followed it.
- Fix: the coordinator's post-rebase rerun of `cargo test -p tide-demo` on the frozen tree records
  the revision. main has moved (b3c075e, 3ee4176, 581b052; README.md and crates/tide-admit/tests
  changed), so that rerun is due anyway.
- Rebase note: main's fx-zu4f.2 added two lines to README.md, so check the link line for a
  textual conflict.

**N2. Test gap: nothing tests that `$RUSTC` beats `rustc` on PATH** (crates/tide-demo/src/lib.rs:512,
tests/demo.rs:109)
- The subprocess test clears the environment, so it always takes the `rustc`-on-PATH route.
- The route the container actually uses is the dev shell's `RUSTC=${sysroot}/bin/rustc`. That
  rustc and the joined toolchain's rustc print the same `-vV`. The flake's comment says only the
  sysroot copy gives the driver its REPLAY-2 sysroot identity.
- Failing case: a regression that ignores `RUSTC` and binds the PATH rustc still passes every
  unit test. Only the container demo could catch it, if it catches it at all.
- Minimum fix: add one case to the existing test. Put a fake rustc with a wrong commit-hash on
  PATH, and set `RUSTC` to a pinned fake in another directory. Assert that the build log shows
  `RUSTC=<that directory>/rustc`. Alternatively, swap the two and assert the refusal.

**N3. docs/docker.md:58-61: the openat2 test needed a setup step that the doc does not mention**
- accept.sh's check 4 wrote `~/.cargo/config.toml` with the dev shell's cc as the linker, because
  the test's std build sets `PATH=/usr/bin:/bin` and the image has no `/usr/bin/cc`.
- A reader who repeats `cargo test -p tide-admit --test build real_files_…` in the container gets
  a linker failure.
- Minimum fix (text): after the test name, add "(with Cargo's linker set to the dev shell's `cc`,
  since the image has no `/usr/bin/cc`)".

**N4. Overstated privacy claims** (.dockerignore:1-4, docs/docker.md:22)
- "Nothing else in the clone reaches Docker" and "nothing private or local reaches the builder"
  are true only for tracked files.
- An untracked file without a leading dot under an allowlisted directory is copied into the image.
  Examples: `docs/notes-private.md`, Emacs's `lib.rs~` and `#lib.rs#`, `examples/out/` binaries.
- A fresh clone has none of these, and the image is not published, so the risk is low. Still,
  whoever publishes it (the owner) should build from a fresh clone.
- Minimum fix (text): "Nothing else in a fresh clone reaches Docker; untracked files you add under
  those directories do."

**N5. Dockerfile:10: "a build argument that the image does not keep" is inaccurate**
- `ARG` values used by a later `RUN` appear in the image's history (`docker history`, the
  `|1 NIX_CONFIG=…` prefix). They are kept out of the image's environment only.
- The value is harmless (`cores = 4`).
- Minimum fix (text): "that the image's environment does not keep (it shows in `docker history`)".

**N6. Unconfirmed: in the container, every `nix develop` may copy `~/tide`, `target/` included,
into /nix/store**
- Without `.git`, `.` is a `path:` flake.
- Upstream Nix has historically copied a path flake's whole directory into the store to
  fingerprint it. The tracked-file filtering that a git flake gets on install.md's route does not
  apply.
- After `cargo run -p tide-demo` fills `~/tide/target`, later `nix develop` calls could copy and
  hash gigabytes. That would erode docker.md's "about 10 GB" budget.
- I could not confirm this. The installed Nix is 2.35.6, which may evaluate lazily, and accept-r1's
  check 4 (a `nix develop` after the demo) finished in 13 s.
- Cheapest check: `du -sh /nix/store/*-source` in the container after the demo and one more
  `nix develop`.
- Fix if confirmed: `ENV CARGO_TARGET_DIR=/home/ubuntu/target` in the Dockerfile. install.md's
  `--out-dir target/quickstart*` outputs are small either way.

**N7. Simplicity and leftovers: concrete deletions or edits**
- `crates/tide-demo/src/lib.rs:320`: drop `pub` from `check_path_toolchain`. Its only caller is
  `Run`, and tests/demo.rs does not import it.
- `crates/tide-demo/src/lib.rs:7-8`: the crate doc says the fallback uses "the rustc … on `PATH`".
  It uses `$RUSTC` first, as demo/README.md says correctly. Line 8 is also about 117 columns, and
  the surrounding text wraps at 100. Reflow it and say "`$RUSTC`, else rustc on `PATH`".
- target/work/image.txt still describes round 0's image (sha256:cebfae…, 23:54Z), not round 1's
  (sha256:8e96c5d4…, from build-r1.log). It is a WORK-only leftover, outside the candidate, so
  delete or refresh it before the report goes on the bead.

**N8. Pre-existing, out of scope (report only): the rustup branch does not bind the compiler
either** (crates/tide-demo/src/lib.rs:299-304)
- `rustup run <channel> cargo` inherits any `RUSTC` or `CARGO_BUILD_RUSTC` from the caller, and
  Cargo prefers those to rustup's toolchain.
- Example: on a host that has rustup, inside `nix develop`, the demo takes the rustup route but
  builds with `$RUSTC`.
- This is main's behaviour, unchanged here, and the bead asks for no change when rustup is
  present. I am not asking for a fix in this bead.

**N9. Process notes (no change to the candidate)**
- The builder reported starting the r1 acceptance run at a load of 49.24, above the limit of 12.
- Before closing, the bead must record (Acceptance 2 and 3):
  - the sandbox line;
  - "no macOS or Windows run observed, by anyone";
  - the openat2-test location decision: tide-admit's build suite, because tide-std's suite has no
    test that makes the real syscall.

## Reasoning

### Lens 1: the pin fallback as a trust point

**Identity.**
- `check_path_toolchain` requires `release` and the full `commit-hash` of both rustc and cargo to
  match `PINNED` exactly.
- Lines are matched with `strip_prefix(key)?.strip_prefix(": ")` and compared with `== Some(wanted)`.
  Missing fields, CRLF output, extra spaces, non-UTF-8 output, a non-zero exit and a failed spawn
  all fail closed with exit 2.
- It refuses a channel that differs from `PINNED.channel`.
- `the_recorded_identity_is_the_pinned_toolchain` ties `PINNED` to rustup's toolchain and to
  rust-toolchain.toml's channel line, so moving the pin alone fails the suite.

**Binding.**
- In `Toolchain::Path`, cargo runs by the absolute path that was checked, with `RUSTC` set to the
  checked rustc's absolute path.
- The builder's real-Cargo check showed that `RUSTC` beats `CARGO_BUILD_RUSTC` and `build.rustc`,
  and the test asserts the bound `RUSTC` against a conflicting `CARGO_BUILD_RUSTC`.
- Every `Command::new` in the crate goes through `pinned()`, or runs the built driver, git or gdb.
  No path reaches an unchecked rustc or cargo.
- `rust-gdb` is looked up on PATH but gets `RUSTC`, which it uses only to find the sysroot's
  pretty-printers. That is not a build trust point.
- Wrappers (`RUSTC_WRAPPER`) are out of reach, as the builder recorded. Like any `-vV` check, a
  script that impersonates the identity can pass. The check is a pin guard against a mistaken
  environment, not a boundary against the user.

**PATH edge cases.**
- `resolve` follows shell semantics: a bare name is looked up on PATH, an empty PATH entry means
  the working directory, and a path is made absolute.
- A missing cargo or rustc gives "there is no rustup, and no cargo on PATH".
- A `RUSTC` naming a nonexistent path gives "… cannot run …".
- An empty `RUSTC` is ignored.

**Rustup present.**
- The branch is byte-equivalent to main's. It keeps `env_remove("RUSTUP_TOOLCHAIN")`,
  `rustup run`, `current_dir` and `stdin(null)`, and its error texts still say "cannot run rustup".
- `find_gdb`'s refactor onto `resolve` preserves its old relative-path and bare-name behaviour.

**Tests.**
- They would fail if the `RUSTC` binding were removed, or if the rustc or cargo commit-hash check
  were dropped. The builder's mutation runs support this.
- The gap is N2.

### Lens 2: image correctness

**Route.**
- The Dockerfile follows install.md's route: step 1 verbatim under bash, flakes on in nix.conf as
  step 1 allows, and step 3's `nix build .#tidelang-admit`.
- It copies the context instead of cloning, which is decision (3) on the bead, and documents
  that. The dev shell is realized at build time. There is no second install route: no rustup and
  no system cc (the env log shows `/usr/bin/cc` absent and rustup absent).

**Base image.**
- The base is pinned by the multi-arch index digest. build-r1.log resolves
  `ubuntu:24.04@sha256:73b9…`, a 6.69 kB index, to its amd64 manifest.
- The image is non-root: `USER ubuntu`, uid 1000, with no sudo binary.

**Context.**
- The allowlist matches the tracked files. No tracked file under an allowed path is a dotfile or
  matches a re-exclude pattern.
- No crate reads design/, trials/, spikes/, scripts/, research/ or reviews/. code_prefix.rs only
  skips those directories.
- The flake's fileset (Cargo.toml, Cargo.lock, crates, rust-toolchain.toml, examples/contracts) is
  inside the allowlist. The planted-context export and Codex's byte comparison back this up.

**Secrets.**
- No secrets are present (see N4 and N5 for wording).
- The documented commands match what was run, apart from resource and mount flags that do not
  change security: `--builder default`, `--progress=plain`, the `NIX_CONFIG` cap (which the doc
  mentions), `--cpus 4`, `-e CARGO_BUILD_JOBS=4`, and the `/work` mount for the logs.

### Lens 3: honesty of docs/docker.md

**The four Scope points are present.**
- openat2 needs Linux 5.6 or later, and Docker Desktop VMs provide it.
- The image is amd64 only, so Apple silicon runs it under slower emulation.
- The image does not make the std build natively on macOS.
- The sandbox line is quoted exactly as observed (`sandbox: full (Linux 7.2.6-76070206-generic,
  Landlock ABI 10)`), on this Linux host under Docker's default seccomp. It makes no promise for
  Docker Desktop VM kernels, and it points the reader to their own line.

**Other required statements.**
- Linux-only checking is stated, with the date, kernel and Docker version, and "No macOS or
  Windows run has been observed".
- No `--privileged` or `seccomp=unconfined` appears. The env log shows `Seccomp: 2`.
- The doc says the image copies the context because the repository is not public yet.
- "About 3 minutes" matches build-r1.log: the build steps sum to about 189 s, with tidelang-std
  and tidelang-admit built from source. "About 6 GB" matches 5.77 GB.

### Lens 4: proof

**Container checks.**
- clone-r1 is at b919d06 (`git rev-parse`) with a clean status and no diff from the candidate.
- Its `.git/HEAD` (21:16:37) postdates the commit (21:16:29).
- build-r1.log (exit 0, image 8e96c5d4…) ends at 00:20Z.
- accept-r1 ran at 00:21:53-00:22:44Z against the tag that build produced.
- The run log does not name the image id. The tag and the timestamps tie them.

**Acceptance checks.** All appear verbatim in accept-r1, with status 0:
- the example printing `[3, 7, 9, 14, 20]`;
- the `--tide-run-test` run with its `sandbox:` line;
- `nix develop --command cargo run -p tide-demo` with no rustup, the checked toolchain line, steps
  0-4 passing and step 5 skipped for lack of gdb;
- the openat2 symlink-confinement test `real_files_build_and_run_against_production_std`, which
  passed.

The host unit-test proof is N1.

### Lens 5: simplicity

The change is small and has a single owner: `PINNED` lives with its only consumer, and `resolve`
replaces the closure that `find_gdb` had. Deletions are listed in N7. Nothing is orphaned:
`find_gdb`'s old closure is gone, and every caller of `pinned()` takes `&Toolchain`.

DONE
