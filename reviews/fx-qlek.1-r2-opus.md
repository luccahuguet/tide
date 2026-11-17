Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 895cef7

Converge round (reports, fixes nothing) under GOVERNANCE.md's Merge Rule, after Codex r1 and its
re-check. Read: AGENTS.md, `br --no-db show fx-qlek.1` with comments, `git diff main...895cef7`
(README.md, docs/install.md, scripts/clean-install-check.sh), docs/nix.md, flake.nix,
examples/contracts/sorted_insert.rs, examples/capabilities/ambient_read.rs, README's build and
RUN-3 sections, crates/tide-admit/src/invocation/{sandbox.rs,replay.rs} for the `sandbox:` line,
and target/work/{brief.md,report.md,r1-codex.md,r1-recheck-codex.md,transcript-895cef7.log,
host-run.log}. No docker, nix, builds, tests or subagents; runtime conclusions rest on the
transcripts.

## Verdict

request_changes: one blocking finding, with a text-only fix of three lines and one shell line
added to the guide. Everything else holds up. The example, its test, the attribute names, the
per-command flake flags, the sandbox line and the dev-shell limits all match the flake, docs/nix.md,
README and the transcript. The four Codex r1 fixes are sound.

## Blocking findings

### B1. The check's record says the clone URL is "the only substitution", but there are more, and one hides a literal guide failure

`scripts/clean-install-check.sh:4-5` ("that is the only substitution"), with
`scripts/clean-install-check.sh:107-108,131-135,142-145` and `docs/install.md:8-9,20`. report.md
repeats the claim ("Every other guide line runs as written").

The guide's lines do run as written. But the machine they run on differs from a newcomer's in ways
that skip parts of the guide, and the record does not say so:

1. **/nix exists before step 1.** It is a bind mount of `RUN_DIR/nix`, owned by the host user
   (`:144`). The Nix installer runs `sudo mkdir /nix && chown` only when `/nix` is missing. So the
   step that docs/install.md:20 describes ("it asks for `sudo` only to create `/nix`") never runs
   in the check, and the image has no sudo at all. It also means the check passes only when the
   host user is uid 1000, the container's `ubuntu`. The comment at :108 asserts this but nothing
   tests it. On another uid the installer fails loudly ("exists, but is not writable by you"), so
   this is a portability gap for fx-qlek.7's rerun, not a silent pass.
2. **The prerequisites line is not the guide's.** The guide says `sudo apt-get install curl
   xz-utils git` (docs/install.md:9). The check runs, as root, `apt-get update` and then
   `apt-get install -y --no-install-recommends ca-certificates curl xz-utils git` (:131-133).
   Concrete failing case: in the check's own pinned `ubuntu:24.04` image, which has empty apt
   lists, the guide's literal line fails with `E: Unable to locate package curl`. The check passes
   only because it adds the `apt-get update` the guide lacks. A newcomer on a fresh minimal VM or
   container image hits the same failure. Separately, the guide's line installs recommended
   packages, so the "no compiler after the prerequisites" absence check covers the check's
   variant, not the guide's.
3. **The run conditions differ** in two smaller ways. Both are harmless to the result, but both
   belong in the same list. `NIX_CONFIG` caps cores and jobs; the script states this at :140, but
   not as a deviation. Step 6's `nix develop` gets stdin from /dev/null, so it exits at once;
   report.md states this, the script does not.

Minimum fix:
- (a) In docs/install.md:9, write `sudo apt-get update && sudo apt-get install curl xz-utils git`.
- (b) In the script header (:4-5), and in the bead record when the coordinator writes it, replace
  "that is the only substitution" with the actual list:
  - the clone URL;
  - /nix pre-created and owned by the user, so the installer's sudo step is not exercised (and the
    host user must be uid 1000);
  - the prerequisites installed as root with `--no-install-recommends` plus `ca-certificates`;
  - the NIX_CONFIG caps;
  - `nix develop` exits at once without stdin.

Fix (a) is a one-token guide change that the check does not execute (prerequisites are prose), so
it needs no rerun. Fix (b) is text. Under AP-PROOF-001 neither needs a re-check by this reviewer
beyond confirming the text landed. Exercising the sudo path instead would mean putting the store on
Docker's root disk (Nix refuses a symlinked /nix), which the builder rightly avoided. Stating the
deviation is the right fix.

## Non-blocking findings

N1. **Wrong download source** (docs/install.md:44-45). "The first build downloads the pinned Rust
nightly and Nix's C toolchain from cache.nixos.org". The nightly is not on cache.nixos.org:
rust-overlay's `rustc-nightly-…tar.xz.drv`, `cargo-nightly-…`, `rustc-dev-nightly-…` and the other
fetches are built locally. The transcript lists them under "these 28 derivations will be built",
and they come from static.rust-lang.org. Only the 200.7 MiB of C toolchain and tools comes from
cache.nixos.org. This matters for a newcomer behind a firewall, and for fx-qlek.7's claims audit.
Fix: "downloads the pinned Rust nightly from static.rust-lang.org and Nix's C toolchain from
cache.nixos.org".

N2. **The block extraction can silently drop a step** (scripts/clean-install-check.sh:93,
`/^```sh$/`). The check runs only blocks fenced exactly ```` ```sh ````. If a later edit fences a
step as ```` ```bash ````, ```` ```shell ```` or ```` ```console ````, or indents the fence, that
step drops out of the check without a sound. Step 6 (`nix develop`) is the likely victim, because
none of the final greps depends on it. The candidate itself is fine: all six steps are ```sh. The
cheapest guard is for awk to exit 1 on any fence whose info string is a shell (`bash`, `shell`,
`console`, `zsh`). Another is to assert the block count printed in the transcript.

N3. **The Nix build sandbox is off in the container, and only report.md says so.** The transcript
shows `warning: auto-disabling sandboxing` (line 33). report.md says the no-/usr/bin/cc property
rests on absence, not on Nix's sandbox. That is honest. But the script header (:2-6) is what
fx-qlek.7 reads when it reruns the check, and it does not say so. Two different claims are in play:
- docs/install.md:10-11 claims "no step uses rustup or the system's C compiler" on any machine,
  including one that has /usr/bin/cc;
- the check proves the steps work on a machine without them.

The first claim rests on the flake's design and on fx-1n5n's `checks.link` under
`--option sandbox true`. This check does not prove it. Add one header line stating that, so the
launch-claims audit cites the right proof.

N4. **The host's sandbox line is from the right driver; tie the container's driver to it.**
host-run.log shows the flake's driver
`/nix/store/ca6rwjjl2pjdvk48nnxklgsv6c0dfyri-tidelang-admit-0.0.0` and the `sandbox: full (Linux
7.2.6-76070206-generic, Landlock ABI 10)` line from `--tide-run-test`. That satisfies the second
Acceptance item.

The clone was main's f1c6c99. The same-store-path argument holds: the diff touches none of
flake.nix, flake.lock, rust-toolchain.toml, Cargo.*, crates/ or examples/contracts. But the
`nix eval` that showed it is not in any log, and the container transcript never prints its own
driver path. One line after the guide in machine.sh (`readlink /home/ubuntu/tide/result`) would
put the identity in every transcript, at no cost.

N5. **"Works there" and "works too" are unproven.** docs/install.md:102-104 says "so `cargo build`
works there". The check's `nix develop` exits at once (report.md says so), so the claim rests on
docs/nix.md alone. docs/install.md:22 says the multi-user install "works too", which nothing
exercised. Both are Nix and flake claims that are probably true. Mark them as unproven in the bead
record so fx-qlek.7 knows they are not covered.

N6. **Step 1 needs a POSIX shell** (docs/install.md:16-17). Process substitution `sh <(…)` and
`. file` need bash or zsh. A fish or nushell user fails at step 1; the owner uses nushell. Nix's
own instructions use the same line, so a short "(in bash or zsh)" is enough.

N7. **"On an older kernel" is too narrow** (docs/install.md:92). README (and `sandbox.rs:100`) give
`partial` for any missing Landlock, Landlock scope or seccomp. That includes Landlock disabled at
boot and a container profile that blocks seccomp or Landlock, not only older kernels. "When the
kernel lacks Landlock or seccomp" would be accurate; the README pointer that follows covers the
rest.

N8. **"With Tide's std built in"** (docs/install.md:6) matches the flake's app description. But the
std is a separate store path that the driver points at (`TIDE_DEFAULT_STD`), not part of the
binary. "With Tide's std as its default" is exact. This is a nit.

N9. **The image removal adds a false FAIL** (:38,:59,:84-87). Run A pulls the image while run B
already uses it, so B does not pull. A's `docker rmi` then fails because B's container is using
the image, and A reports FAIL after a passing guide. The image is 78 MB and pinned by digest. See
simplicity item S1.

N10. **SIGKILL leaves the container running.** If a run is killed with SIGKILL (for example
`timeout -k`), `finish` does not run and the container (no `--rm`) keeps running with about 7 GB
in RUN_DIR. Only `container.id` says what was left. A TERM delivered to the script's PID alone
(not its process group) waits until `docker run` returns, because bash defers traps during a
foreground pipeline. Both cases are inherent to a shell wrapper. One header sentence on recovery
(`docker rm -f $(cat RUN_DIR/container.id)`, then the root-in-image `find … -delete`) would help
the next maintainer.

The builder already lists two items: the untracked `result` link (fx-qlek.3 or a `.gitignore`
line) and the unpinned installer (rerun at the RC). I agree with both and do not repeat them.

## Simplicity

The script is close to minimal for what it checks. Concrete deletions:

- S1. Delete the `pulled` flag and the `docker rmi` (:38, :59, :86, and the `if` at :84-87, which
  becomes a plain `docker pull`, or goes away entirely because `docker run` pulls by digest). This
  is 4 to 5 lines, and it removes N9's false FAIL.
- S2. Delete `absent "before setup"` (:125). `apt-get install` only adds files, so the
  `absent "after the prerequisites"` check, the state the guide starts from, implies it. Keep the
  checkout and /nix checks. The root-path half of `absent "after the guide"` cannot fail either:
  the guide runs as `ubuntu` without sudo. It is cheap, though, so keeping it is fine.
- S3. Write `steps.sh` directly from the awk pipeline, in a `{ echo …; echo …; git show … | awk …;
  } > steps.sh || die …` group under pipefail, and drop `blocks.sh` (:104-105). This saves one
  file and one line.
- Keep the `tee` signal trap, the cidfile, the cleanup ordering and the root-in-image delete. Each
  carries a failure that Codex r1 or the builder's interrupt tests found.

## Reasoning

**Lens 1: the newcomer.**
- The steps run in order in the transcript. Block 1 installs Nix 2.35.6 single-user. Block 2
  clones the bundle (detached at 895cef7). Block 3 runs `nix … build .#tidelang-admit`, building
  28 derivations in 104 s. Block 4 prints `[3, 7, 9, 14, 20]`. Block 5 prints `sandbox: full …`,
  `100/100 cases passed` and `test result: ok. 1 passed`. Block 6 realizes the dev shell.
- `tidelang-admit` matches `packages`, `apps` and docs/nix.md.
- The per-command flake flags appear on every nix command, `nix run` included (r1 #4).
- Step 4 and step 5's command lines are exactly `checks.link`'s.
- The test name matches the generated `__tide_contract_tests::insert_sorted_3`, and `--list`
  confirms it (host-run.log).
- The sandbox line's text, its stream (stderr, replay.rs:318), its timing (before the child) and
  the `full` and `partial, not enforced` wording match sandbox.rs and README's RUN-3 paragraph,
  which the guide cites.
- `ambient_read.rs` expects `TD0501`.
- Step 6's limits (no rustup `stable`, no python3 with MPFR) are docs/nix.md's, verbatim in
  substance.
- Disk: about 10 GB, 7 GB of it for /nix, matches the 6 GB observed on /mnt/build. Prerequisites
  are stated. The literal-following gaps are B1(a), N1 and N6.

**Lens 2: is the check sound?**
- PASS needs three things:
  - the containerized pipeline exits 0, with `set -euo pipefail` in machine.sh, `set -eo pipefail`
    in steps.sh and `pipefail` on `docker run | tee`;
  - `finish` succeeds at cleanup;
  - the completed guide log holds the exact stdout line, a `sandbox:` line and
    `test result: ok. 1 passed`.

  None of those strings can come from the echoed blocks or from the prose. That is what Acceptance
  item 1 asks for. `^sandbox: ` also accepts `partial`, which is correct: Acceptance asks for the
  line, not for `full`.
- The absence checks now fail for real (r1 #2), and the transcript shows all three of them.
- Extraction holds for this guide. Its weakness is N2.
- The deviations are B1.
- Host safety: deletion is confined to fresh subdirectories of a canonicalized, empty-at-start
  RUN_DIR, done by root inside a container that mounts only RUN_DIR. `find -delete` does not
  follow links. Removal goes through the cidfile ID only, and the store is emptied only after its
  container is gone. A cleanup failure turns PASS into FAIL. This is all sound.

**Lens 3: is the evidence honest?**
- report.md states the transcript's Nix-sandbox warning and its consequence plainly. N3 asks that
  the script's header say it too.
- The host sandbox line is from the flake's store-path driver, and the revision gap is disclosed
  and argued correctly (N4).

**Lens 4: claims and the README boundary.**
- Apart from N1, N5, N7 and N8, every guide claim traces to flake.nix, docs/nix.md, README, the
  driver source or the transcript.
- The README edit is two lines, a link placed after the existing cargo-prerequisites paragraph,
  with no other wording changed. It stays inside fx-qlek.3's boundary.

**Lens 5: simplicity.** See S1 to S3. None of them is blocking.

DONE
