# Try Tide on macOS or Windows

The alpha runs on x86_64 Linux only. On macOS or Windows, run it in a Linux
container. The repository's `Dockerfile` makes two images from one build
stage, which installs Tide by [install.md](install.md)'s route, Nix's
single-user installer and then the flake, in a pinned Ubuntu 24.04 image:

| Image | Target | What it holds | Size |
| --- | --- | --- | --- |
| try | `--target try` | Only the closure of the flake's `tide-demo` ([nix.md](nix.md#demo)): Cargo, the pinned rustc's sysroot, Nix's C toolchain, git and the demo's sources, with no distribution and no Nix. It runs the [demo](../demo/README.md) and exits. | 2.42 GB; 769 MB compressed with gzip |
| dev | `--target dev` | Ubuntu, Nix and the clone, with the driver, the demo and the dev shell built. It starts a shell. [.devcontainer](../.devcontainer/devcontainer.json) builds it for GitHub Codespaces. | 6.09 GB |

You need Docker (Docker Desktop on macOS and Windows) and a clone of the
repository ([install.md](install.md), step 2). In the clone:

```sh
docker build --platform linux/amd64 --target try -t tide-try .
docker run --rm --platform linux/amd64 tide-try
```

The try image starts the demo, which builds the driver and Tide's std,
runs its steps, and exits 0 when every step shows what it should. Its
target is `/tmp/tide-demo`, so the demo writes only under `/tmp`
([nix.md](nix.md#demo) gives the boundary). It has no gdb, so the demo skips
its debugger step, and no `/bin/sh`: the demo's launcher runs with the
closure's own bash. For a shell with Nix, the driver and the dev shell,
build the dev image instead, which needs about 10 GB of free disk space:

```sh
docker build --platform linux/amd64 --target dev -t tide .
docker run --rm -it --platform linux/amd64 tide
```

A published try image would let `docker run` start without a clone or a
build; pushing one to a registry is the owner's step, and until then both
images are built from a clone.

The build stage copies the clone, Docker's build context, instead of
cloning a URL, so an image holds the revision you built. `.dockerignore`
allowlists what it copies: the Cargo workspace, the flake, the toolchain
pin, `examples/`, `conformance/`, `benchmarks/`, `demo/`, `docs/`, README,
LICENSE and NOTICE. Nothing else in a fresh clone reaches Docker: not
`.git`, the `.beads` ledger, dotfiles such as `.env`, or build outputs.
Untracked files you add under those directories do. Without `.git`, the demo
prints `revision:  unknown` in the container.

The build downloads Nix, the pinned Rust nightly and Nix's C toolchain. The
dev image also compiles the driver, with `--release`, and Tide's std. The
try image's contents are the flake's pinned closure, but the build's
bootstrap, Ubuntu's apt repositories and the Nix installer from nixos.org,
is not pinned, so the recipe does not rebuild either image bit for bit.

The dev container starts a shell in `~/tide` as the user `ubuntu`, with the
driver built at `result/bin/tide-admit`, so [install.md](install.md)'s steps 4
and 5 run as written there. `nix run .#tide-demo` runs the demo, and
`nix develop` enters the dev shell, where `cargo run -p tide-demo` runs it
too; Cargo downloads the workspace's crates on first use there.

## Measured times

README's [Try it](../README.md#try-it) table gives the measured times and
sizes, with their conditions, and
[reviews/fx-qlek.7-s3-measurements.md](../reviews/fx-qlek.7-s3-measurements.md)
the log lines. What the table leaves out:

- the try image's 3-minute build, from a clean clone with `--no-cache`, was
  69 s of `apt-get`, 12 s to install Nix, 80 s to fetch the closure from the
  binary cache and build the rest, and 7 s to copy and export it; its 36-s
  run spent 21 s building the driver and std;
- 3.5 of the dev container's 6 minutes went to exporting the image;
- the build leaves its `nix` and `try-closure` stages in Docker's build
  cache, the second holding the closure twice, in its Nix store and in the
  copy the try image takes; their size was not measured, and
  `docker builder prune` removes them;
- a published image's download time depends on the connection; the
  measured compressed size is 769 MB.

## What to expect

- Tide's `Fs` confines paths with `openat2`, which needs a Linux 5.6 or later
  kernel. A container uses the kernel of Docker's Linux VM, and current
  Docker Desktop VMs provide one; `uname -r` in the container shows it.
- The images are `linux/amd64` only. Apple-silicon Macs run them under
  emulation, which is slower.
- The images do not make Tide's std build natively on macOS: the driver and
  the std run only inside the container.
- RUN-3's sandbox: on the Linux host, under Docker's default seccomp profile,
  `--tide-run-test` printed

  ```text
  sandbox: full (Linux 7.2.6-76070206-generic, Landlock ABI 10)
  ```

  so Docker's default profile let the driver install Landlock and seccomp.
  The level depends on the kernel the container runs on: a Docker Desktop VM
  whose kernel lacks Landlock or its scopes prints
  `sandbox: partial, not enforced: …` instead. Read the line your run prints;
  [driver.md](driver.md)'s RUN-3 paragraph says what each level means.

The images were checked on Linux only. On 2026-11-18, an x86_64 Linux host
(Linux 7.2.6, Docker 29.10.0) built the dev image from a clean clone and, in
the container, under Docker's default seccomp profile, ran the example and
its sandboxed test above, the demo, and the `openat2` confinement test
`real_files_build_and_run_against_production_std` (with Cargo's linker set to
the dev shell's `cc`, since the image has no `/usr/bin/cc`). On 2026-11-19,
the same host built the try image and ran the demo in it. No macOS or
Windows run has been observed.
