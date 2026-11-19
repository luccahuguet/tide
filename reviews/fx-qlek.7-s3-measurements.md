# fx-qlek.7 S3: try-it measurements

The raw lines behind every time and size in README.md's "Try it" table and
docs/docker.md, quoted from the S3 builder's logs (home directories shown
as `~`). The commands and log names are on bead fx-qlek.7; the full logs
were not kept, and this file quotes the lines behind each number.

## Conditions

- Host: one x86_64 Linux machine, Linux 7.2.6, 16 CPUs, Docker 29.10.0.
  Connection: the host's own; its bandwidth was not measured.
- Date: 2026-11-19, 10:06Z to 10:58Z.
- Revision: The rustup, first `nix run`, Docker and devcontainer runs used
  the first S3 change on 46297dd (devcontainer: 3e0eb6a), before round 1's
  wrapper changes and the rebase. The 12:06Z `nix run` ran on the rebased
  tree with the final wrapper.
- Caps: Nix builds at `cores = 4`, `max-jobs = 4`. The rustup, `nix run`
  and Docker try runs at 4 Cargo jobs (`CARGO_BUILD_JOBS=4`), their
  containers at `--cpus 4`. The devcontainer run's runtime had no CPU or
  Cargo-job cap; only its image build had the Nix caps.
- Every run went through the fleet's heavy-run queue; times are taken from
  inside each run, after the queue released it.
- rustup: 1.29.3, installed with `--profile minimal` and no default
  toolchain. rust-toolchain.toml names no profile, so rustup's default
  profile would also install rust-docs, clippy and rustfmt; those are not in
  these numbers.

## rustup path, clean container

`ubuntu:24.04` (the Dockerfile's pinned digest), run as the user `ubuntu`,
`git clone` of a bundle of 46297dd. The script installs rustup with

```text
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
```

Log lines (`[phase +seconds]`):

```text
[1 +0s] install rustup (minimal profile, no default toolchain)
info: profile set to minimal
[1 +3s] rustup toolchain install (reads rust-toolchain.toml)
[1 +22s] toolchain installed; ~/.rustup is 1.4G
[1 +22s] cargo run -p tide-demo without a C toolchain
error: linker `cc` not found
error: could not compile `libc` (build script) due to 1 previous error
exit=101
[root +305s] apt-get install build-essential
[2 +0s] cargo run -p tide-demo with build-essential (cold target)
== Done: every step showed what it should. Logs and artifacts: target/tide-demo
exit=0
[2 +51s] cargo run -p tide-demo again (warm target)
exit=0
[2 +67s] phase 2 done; target is 1.3G, ~/.cargo is 83M
```

The failed run without `cc` had already fetched the crates, so the cold
run's 51 s does not include a crate download from an empty cache.
`du -sh` reports binary units: 1.4G is about 1.4 GiB, 83M is 83 MiB.

## `nix run .#tide-demo`, closure already in the store

```text
rev: 46297dd36a047a8cbe7427f9bc5af38e5ceb45bf + uncommitted flake.nix; date: 2026-11-19T10:06:54Z
== Done: every step showed what it should.
real	0m48.090s
exit=0
```

The Cargo target was a fresh, empty directory. After the rebase onto main
766eb17, which changed Tide's std and so the demo's store sources, the same
check on the rebased S3 tree (app
`/nix/store/j1fmsapbh7jfva9ba09w0sh6hymf2g6j-tide-demo`), ending 2026-11-19 12:06Z:

```text
exit=0 after 56s
== Done: every step showed what it should.
target size: 1.3G (du -sh), 1277 MiB
```

The README's 48 s is the first run's. The app's closure, from
`nix path-info -Sh --closure-size`: 2.1 GiB (4.3 GiB before the app put
only Cargo, not the joined toolchain, on `PATH`).

## Docker try image

Build, `docker build --no-cache --progress=plain --target try`, step lines:

```text
#6 [nix 2/6] RUN apt-get update ...
#6 DONE 69.0s
#8 [nix 4/6] RUN sh <(curl ... https://nixos.org/nix/install) --no-daemon ...
#8 DONE 12.0s
#11 [try-closure 1/1] RUN nix build --out-link ~/demo .#tide-demo ...
#11 DONE 80.4s
#12 [try 1/2] COPY --from=try-closure ... /
#12 DONE 2.4s
#14 DONE 4.1s
real	3m17.673s
```

Step #11 is the `nix run` route's cost in an empty Nix store: fetching the
closure from the binary cache and building the rest.

Size: `docker image inspect` gave 2,424,731,888 bytes;
`docker save tide-try | gzip -6 | wc -c` gave 769,099,206 bytes. No
registry pull was measured.

Run, `docker run --rm --cpus 4 -e CARGO_BUILD_JOBS=4`, each line stamped
with seconds since the command was queued:

```text
724s heavy.sh: slot build2.lock
726s == 0. Toolchain and build
747s driver:    /tmp/tide-demo/debug/tide-admit
760s == Done: every step showed what it should. Logs and artifacts: /tmp/tide-demo/tide-demo
```

So the run took 36 s, 21 s of it building the driver and std.

## Dev container (devcontainer CLI 0.90.0)

From a fresh clone of 3e0eb6a, `devcontainer up` with the committed
devcontainer.json plus the build argument `NIX_CONFIG` (`cores = 4`,
`max-jobs = 4`), then `devcontainer exec ... nix run .#tide-demo`:

```text
#6 DONE 45.1s          (apt-get)
#8 DONE 12.3s          (Nix installer)
#11 DONE 84.7s         (dev stage: driver, demo, dev shell)
#12 exporting layers 126.2s done
#12 DONE 208.9s
{"outcome":"success", ... "remoteUser":"ubuntu","remoteWorkspaceFolder":"/workspaces/devcontainer-clone", ...}
up exit=0 after 360s
== Done: every step showed what it should. Logs and artifacts: ~/.cache/tide-demo/tide-demo
exec exit=0 after 49s
```

The dev image's size, from `docker images`: 6.09GB. Codespaces itself was
not measured.
