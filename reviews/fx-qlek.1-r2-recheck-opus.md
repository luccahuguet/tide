Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate ed60c88 (re-check of 895cef7 findings)

This is a finder re-check only. I read `git diff 895cef7 ed60c88` (docs/install.md and
scripts/clean-install-check.sh), the updated target/work/report.md, transcript-ed60c88.log and
host-driver-eval.log. I ran no docker, nix or builds. I made no full re-review.

## Verdict

approve. B1 is resolved, and so are all of N1 to N10 and S1 to S3. The two new failure paths, the
uid precondition and the fence check, fail loudly and cannot produce a false PASS. The fixes broke
nothing that blocks. One small non-blocking regression and two text nits are listed at the end.

## Findings

**B1: resolved.**
- (a) docs/install.md:9-10 now says `sudo apt-get update && sudo apt-get install curl xz-utils git`,
  which removes the "Unable to locate package" failure on images with empty package lists.
- (b) The script header (:5-17) drops "the only substitution" and lists every deviation:
  - the bundle clone;
  - the pre-created, user-owned /nix, with the sudo step not run and no sudo in the image;
  - the root `apt-get update` and `--no-install-recommends ca-certificates …` install;
  - the NIX_CONFIG caps;
  - `nix develop` exiting at once.

  report.md's Acceptance 1 repeats the list for the bead record.
- The uid-1000 assumption is now a checked precondition (:42-43). Analysis of that check:
  - It runs after argument parsing and before RUN_DIR is created and before the EXIT trap. A
    refusal therefore creates nothing, leaves nothing behind, and prints its message to the
    terminal.
  - It is the right condition: the script's own `mkdir` makes RUN_DIR/nix owned by the effective
    uid, and the image's `ubuntu` is uid 1000.
  - One case it cannot catch is rootless Docker or userns-remap, where host uid 1000 does not map
    to container uid 1000. There the installer stops with "not writable by you", so that case
    fails loudly too. No silent pass is possible.

**N1: resolved.** The guide's step 3 now names static.rust-lang.org for the nightly and
cache.nixos.org for the C toolchain.

**N2: resolved, and the new fence check is sound.**
- Allowed fences are unindented ```` ```sh ````, ```` ```text ```` and a bare ```` ``` ```` closer
  inside a block. Any other line starting with optional whitespace and then ```` ``` ```` or `~~~`
  calls `bad()`.
- `bad()` sets `failed` and calls `exit 1`, which still runs `END`. `END` re-exits 1 on `failed`
  before its own checks, so the first error is the one reported.
- `END` also fails on an unclosed fence, on a count other than `expected_blocks=6`, and on a clone
  count other than 1.
- The group `{ echo; echo; git show | awk; } > steps.sh || die` takes the pipeline's status under
  pipefail, so a `git show` failure is caught too.
- awk's stderr goes to the transcript tee, which is already in place by then.
- The current guide has exactly six ```` ```sh ```` fences and one ```` ```text ```` fence (lines
  18-104). The author's mutation tests cover ```` ```bash ````, an indented fence, a wrong count
  and a missing clone line.
- The cost is that the fixed count makes every added or removed step a one-line edit to the
  script. That is intended, and it is the cheaper guard of the two I offered.

**N3: resolved.** Header lines :18-21 say that Nix's build sandbox is off in the container, that
the check proves the steps work without rustup or a system cc, not that they ignore them, and that
fx-1n5n's `checks.link` with `--option sandbox true` is that proof.

**N4: resolved.**
- machine.sh prints `driver: $(readlink /home/ubuntu/tide/result)` under `set -e`, so a missing
  `result` fails the run.
- The transcript shows `driver: /nix/store/ca6rwjjl2pjdvk48nnxklgsv6c0dfyri-tidelang-admit-0.0.0`
  (transcript-ed60c88.log:387).
- host-driver-eval.log records `nix eval --raw .#tidelang-admit.outPath` at ed60c88 giving the same
  path, which is the driver host-run.log's sandbox line came from.

**N5: resolved.**
- `--daemon` is now "the other option Nix documents" rather than "works too".
- The step-6 `cargo build` claim stays in the guide, but report.md records that it rests on
  docs/nix.md alone and that fx-qlek.7 should treat both claims as uncovered. That is the fix I
  asked for.

**N6: resolved.** "Run these in bash or zsh:" now precedes step 1's block. It is prose, so the
fence check is unaffected.

**N7: resolved.** The new wording, "lacks Landlock, its scopes or seccomp, or they are disabled or
blocked", matches README's RUN-3 list and sandbox.rs.

**N8: resolved.** It now reads "with Tide's std as its default".

**N9 and S1: resolved.** The `pulled` flag, the image probe and pull, and the `docker rmi` are
gone. `docker run --platform linux/amd64` pulls the pinned digest when it is missing (transcript
line 9), and the header says the image is kept and how to remove it. The false FAIL from a
concurrent `rmi` can no longer happen.

**N10: resolved.** The header (:28-29) gives the SIGKILL recovery: `docker rm -f` on the cidfile's
ID, then the root-in-image `find … -delete`.

**S2: resolved.** `absent "before setup"` is deleted. The checkout and empty-/nix checks and the
after-prerequisites and after-guide checks stay. The transcript shows them at lines 19, 22 and 386.

**S3: resolved.** blocks.sh is gone, and steps.sh is written by one group under pipefail.

The new proof is transcript-ed60c88.log on ed60c88. It shows all six blocks, `[3, 7, 9, 14, 20]`,
`sandbox: full (Linux 7.2.6-76070206-generic, Landlock ABI 10)`, `test result: ok. 1 passed`, the
`driver:` line, cleanup, and `PASS (ed60c88…)`.

## Non-blocking (new or residual)

- **A failed pull now leaves RUN_DIR/check behind.** The image pull moved from before the bundle
  into `docker run`. If that pull fails (network or registry), RUN_DIR/check already holds the
  bundle, steps.sh and machine.sh. `finish` then tries the root-in-image `find` to empty it, that
  run needs the same missing image, and it fails. The run already exits non-zero, but it now also
  says "cleanup failed" and leaves RUN_DIR/check. A rerun with that RUN_DIR is refused as not
  empty. Before, a failed pull left RUN_DIR clean. Every file in RUN_DIR/check is created by the
  host user (it is mounted read-only in the container), so the minimal fix is a host `rm -rf
  "$run/check"` in `finish`, keeping root-in-image for nix and tmp only. Recovery is a plain
  `rm -r`, and it cannot produce a false PASS.
- **Nit:** docs/install.md:46 was not reflowed after the N1 edit and runs well past the file's
  usual width.
- **Nit:** the header's N10 recovery could end with `rmdir RUN_DIR/{nix,tmp,check}` to match what
  `finish` leaves.

DONE
