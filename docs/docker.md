# Try Tide on macOS or Windows

The alpha runs on x86_64 Linux only. On macOS or Windows, run it in a Linux
container: the repository's `Dockerfile` installs Tide by
[install.md](install.md)'s route, Nix's single-user installer and then the
flake, in a pinned Ubuntu 24.04 image.

You need Docker (Docker Desktop on macOS and Windows), a clone of the
repository ([install.md](install.md), step 2) and about 10 GB of free disk
space: the image is about 6 GB, and Cargo builds in the container add more.
In the clone:

```sh
docker build --platform linux/amd64 -t tide .
docker run --rm -it --platform linux/amd64 tide
```

The image copies the clone, Docker's build context, instead of cloning a URL,
because the repository is not public yet; it holds the revision you built.
`.dockerignore` allowlists what it copies: the Cargo workspace, the flake, the
toolchain pin, `examples/`, `conformance/`, `benchmarks/`, `demo/`, `docs/`,
README, LICENSE and NOTICE. Nothing else in a fresh clone reaches Docker: not
`.git`, the `.beads` ledger, dotfiles such as `.env`, or build outputs.
Untracked files you add under those directories do. Without `.git`, the demo
prints `revision:  unknown` in the container.
The build downloads Nix, the pinned Rust nightly and Nix's C toolchain, and
compiles the driver and Tide's std. On the x86_64 Linux host that checked it,
with Nix capped at 4 cores, it took about 3 minutes.

The container starts a shell in `~/tide` as the user `ubuntu`, with the
driver built at `result/bin/tide-admit`, so [install.md](install.md)'s steps 4
and 5 run as written there. `nix develop` enters the dev shell, where
`cargo run -p tide-demo` runs the [demo](../demo/README.md); Cargo downloads
the workspace's crates on first use.

## What to expect

- Tide's `Fs` confines paths with `openat2`, which needs a Linux 5.6 or later
  kernel. A container uses the kernel of Docker's Linux VM, and current
  Docker Desktop VMs provide one; `uname -r` in the container shows it.
- The image is `linux/amd64` only. Apple-silicon Macs run it under
  emulation, which is slower.
- The image does not make Tide's std build natively on macOS: the driver and
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
  README's RUN-3 paragraph says what each level means.

The image was checked on Linux only: on 2026-11-18, an x86_64 Linux host
(Linux 7.2.6, Docker 29.10.0) built it from a clean clone and, in the
container, under Docker's default seccomp profile, ran the example and its
sandboxed test above, the demo, and the `openat2` confinement test
`real_files_build_and_run_against_production_std` (with Cargo's linker set to
the dev shell's `cc`, since the image has no `/usr/bin/cc`). No macOS or
Windows run has been observed.
