Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate df118d2 (Codespaces, Opus review)

## Verdict

APPROVE. No blocking findings. Reviewed `git diff main...df118d2 -- . ':!.beads'` (README.md,
crates/tide-demo/src/lib.rs, crates/tide-demo/tests/demo.rs, docs/install.md, docs/nix.md,
flake.nix) against the Dockerfile, .dockerignore, .devcontainer/devcontainer.json, demo/README.md
and docs/docker.md. Read-only. I ran no builds. The proof is the builder's (report.md r1 to r3).

What I checked and found correct:

- **Revision order** (lib.rs:443-469). Git's `HEAD` comes first, then a non-empty
  `TIDE_REVISION`, then `unknown`. Under `nix run`, `root` is `CARGO_MANIFEST_DIR/../..`,
  canonicalized (lib.rs:156), which is the `demoSrc` store path. `/nix/store` holds no repository,
  so git fails and the recorded value is used. In a clone, git wins over any caller-set
  `TIDE_REVISION`, so the variable cannot override a real `HEAD`. An empty value falls through to
  `unknown`.
- **Dirty-state honesty.** Status has three outcomes: clean, `(with uncommitted changes)`, and
  `(uncommitted changes unknown)`. A failed status is no longer read as clean, as it was before.
  `--no-optional-locks` keeps the probe from writing the index. On the Nix side,
  `self.dirtyRev` is `<rev>-dirty`, so a dirty flake is visibly marked. A Nix older than 2.17
  has no `dirtyRev`, so `or null` makes it `unknown` rather than claim a clean rev.
- **`GIT_LOCAL_ENV_VARS`.** It matches `git rev-parse --local-env-vars` on git 2.43 exactly
  (15 names, checked locally). Both child gits clear them.
- **Flake.** `self` is bound in `outputs` (flake.nix:15). `self.rev or self.dirtyRev or null`
  parses right-nested. `revision` reaches only the `demo` launcher text, so `demoSrc`,
  `cargoDeps`, the toolchain and `sysroot` are unchanged. The interpolated value is hex or
  hex-`dirty`, so it is safe unquoted. With no revision the launcher runs `unset TIDE_REVISION`,
  which also drops a caller's value. A `github:` source has `rev`. A `path:` or plain-directory
  source, the Docker build context, has none and prints `unknown`, as docs/nix.md and
  docs/docker.md say.
- **Signature change.** `first_line` now has a single caller (lib.rs:452). No caller is left
  over.
- **Test.** The fixture git uses `env_clear`, a null system/global config, no hooks and no
  template. The re-exec probe with `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` aimed at a
  decoy runs both the writing fixture and the query. It requires "1 passed", so an empty filter
  cannot pass, and it checks that the decoy's refs are unchanged. It also covers an untracked
  file, an unreadable index, and a missing root with `Some`, empty and `None`. The builder's
  falsifiers (report r2/r3) show that the env clearing and the fixture isolation each matter.
  Nothing is built.
- **Docs.** "skipped: no gdb" is the prefix of the real message (lib.rs:1088). `revision:  unknown`
  matches the `println!` (lib.rs:533). The Docker try image builds from a context without `.git`
  (.dockerignore `*` allowlist), so `unknown` is right there. The owner's numbers are copied
  exactly (2m13.715s real; configuration "Tide"; 2-core; US East; `d48b1aa`), and only the time
  to ready is called approximate. The removed "once public" clauses (README, install.md,
  nix.md) fit a public repository. paper/tide-v0.typ:306 rightly stays. No "not yet measured"
  is left outside reviews/.

## Blocking findings

None.

## Non-blocking

1. **The README cites a log file for every number, but the owner's numbers have none there.**
   README.md:389-392 says reviews/fx-qlek.7-s3-measurements.md "gives the conditions and the
   log lines behind each number". The owner's Codespaces numbers are not in that file. The
   added sentence at :398-400 mostly makes up for it.
   - Minimum fix: say "behind each number measured on that host", or add one line about the
     owner's run to the measurements file.
2. **Upward discovery outside `root`.** This is old behavior and the diff does not cause it.
   Suppose the sources sit in a plain directory inside some other git work tree, such as a
   GitHub ZIP unpacked inside another checkout. `git -C root` then finds the outer repository
   and prints its `HEAD`, plus `(with uncommitted changes)`. The recorded value or `unknown`
   would be the honest answer. The new doc comment, "Git runs on `root` alone", is qualified
   only for environment routing. No documented path, whether clone, Nix store or Docker, hits
   this.
   - Cheap fix: set `GIT_CEILING_DIRECTORIES` to `root`'s parent, or require
     `rev-parse --show-toplevel == root`.
3. **The test inherits the caller's global git config.** `source_revision` in the test still
   reads the caller's global git config. A caller with `status.showUntrackedFiles=no` fails the
   untracked-file assertion. In production the same setting hides untracked files from the
   dirty mark.
   - Fix: pass `--untracked-files=normal` to the status call.
4. **The Codespaces launcher is no longer prebuilt.** The dev image prebuilds `.#tide-demo` from
   a plain directory, where the launcher runs `unset`. A codespace evaluates the git flake,
   where it runs `export <rev>`. So the codespace's first `nix run` builds the small launcher
   locally. Its build inputs, shellcheck and stdenv, are already in the image's store, because
   the dev stage never collects garbage.
   - Dockerfile:63-64 ("start at once") and .devcontainer/devcontainer.json:2 ("demo ...
     already built") remain true in substance.
   - The README row is pinned to `d48b1aa`, so it stays honest.
   - The builder noted this in report r1. No fix is needed. A word in the Dockerfile comment
     would be a nit.
5. **The gdb sentence has a second cause it leaves out** (README.md:363-366). Under `nix run`,
   the launcher's `PATH` is fixed (flake.nix:192-199). So a gdb installed in the image would
   still not be found without `RUST_GDB`. "The image has no gdb" is true but not the whole
   reason. Nit.
6. **`REVISION_VAR` is `pub` with no outside user.** No test or other crate uses it, so it could
   be private. Nit.
7. **Proof scope.** The `nix run` log (nixrun-r1.log, `revision:  ccc8e6ba…`) predates r2/r3.
   Those rounds changed only the git branch of `source_revision` and the tests, not the recorded
   path or the flake, so the log still covers the Nix route. The focused test ran at df118d2.
   Full suites are tide-lead's.

DONE
