# Nix packaging

`flake.nix` packages Tide for `x86_64-linux` (bead fx-1n5n). The attribute
names follow research/35 §4.

| Output | What it is |
| --- | --- |
| `packages.tidelang-admit` (`default`) | The `tide-admit` driver, built with the pinned nightly, with Nix's C toolchain as its linker and `tidelang-std` as its default std |
| `packages.tidelang-std` | Tide's std, `lib/deps/libtide_std.rlib`, beside its dependencies and `tide-macros` in the same `lib/deps` |
| `packages.tide-demo` | A script that runs the [demo](../demo/README.md) (`cargo run -p tide-demo`) from the flake's sources with the dev shell's settings; [Demo](#demo) says where it writes |
| `apps.tidelang-admit` (`default`) | Runs `tide-admit` |
| `apps.tide-demo` | Runs `packages.tide-demo`: `nix run .#tide-demo`, or `nix run github:luccahuguet/tide#tide-demo` with no clone once the repository is public |
| `devShells.default` | The pinned toolchain, cargo included, set up so a driver Cargo builds there gets the packages' sysroot, linker and runtime directories |
| `checks.link` | In the build sandbox, the driver alone admits, links and runs `examples/contracts/sorted_insert.rs`, and runs its generated contract test through `--tide-run-test` (RUN-3); no executable or shared object it uses names `/usr/bin` |

Nix commands need `--extra-experimental-features 'nix-command flakes'`
unless the machine enables both.

## How the driver is configured

The driver's linker and default std are fixed when it is compiled, never
by the environment it runs in (README, operator inputs):

- `TIDE_LINKER_BIN` names the directory whose `cc` links and whose `ld`
  GCC finds through `-B`. The flake sets it to `stdenv.cc`'s `bin`, which
  holds the cc wrapper and the binutils wrapper's `ld`. Unset, the driver
  uses `/usr/bin`, so a Cargo build of the workspace links as before. The
  link cache key and the replay receipt hash that directory's `cc` and `ld`.
- `TIDE_DEFAULT_STD` and `TIDE_DEFAULT_DEP_DIR` name a std and its
  dependency directory, used when the operator passes no `--tide-std`. The
  flake points them at `tidelang-std`, which puts the std in the driver's
  runtime closure, so `nix run` admits Tide code with nothing else
  installed. An explicit `--tide-std` replaces both. A driver build that
  sets only one of them fails to compile.
- `TIDE_RUNTIME_LIB_DIRS` lists, separated by `:`, the directories where
  RUN-3 (`--tide-run-test`, `--replay`) looks up a test's libraries before
  the system directories. The flake sets it to the glibc and libgcc `lib`
  directories that Nix's linker writes into each executable's
  `DT_RUNPATH`. The test binary is the untrusted child's, so the driver
  never reads its `DT_RUNPATH`: a library that only that search path names
  gets no Landlock grant and fails to load. Unset, the list is empty.

A relative `TIDE_LINKER_BIN`, or a relative or empty entry in
`TIDE_RUNTIME_LIB_DIRS`, fails the driver's compilation: it would resolve
against the operator's working directory, which would then choose the
linker or a grant.

A wrapper script that adds `--tide-std` was rejected: the driver refuses a
repeated `--tide-std`, so the wrapper would have to parse the driver's
arguments to leave an explicit one alone, a second owner of that parser.

The driver is compiled, and the std built, with `tidelang-sysroot`'s rustc.
That sysroot is a copy of the toolchain's `rustc` and `lib` as real files.
rust-overlay joins its components with symbolic links, and the driver
counts a loaded library as the sysroot's only when its resolved path lies
inside the sysroot (REPLAY-2's dependency identity). The copy's ELF search
paths are moved from the joined toolchain to the copy, because rustc finds
its sysroot from the `librustc_driver` it loads. The derivation fails if a
symbolic link or a reference to the joined toolchain remains, or if its
rustc reports another sysroot. The driver's rpath and `--sysroot` name that
store path. `tidelang-admit`'s closure is 2.1 GiB: the 1.2 GiB sysroot copy;
the 403 MiB rustc component, which the copy's search paths still name; and
the C toolchain the driver links with, with the 80 MiB tidelang-std. With
the joined toolchain and its docs, the closure was 2.9 GiB.

## Dev shell

`nix develop` puts the pinned toolchain, cargo included, on `PATH`, and
sets, for Cargo builds of the workspace:

- `RUSTC` to `tidelang-sysroot`'s rustc, so a driver built there gets a
  sysroot whose libraries resolve inside it. With the joined toolchain's
  rustc, every link build would fail its dependency identity;
- `TIDE_LINKER_BIN` and `TIDE_RUNTIME_LIB_DIRS` to the packages' values, so
  that driver links with Nix's C toolchain and RUN-3 grants Nix's glibc
  and libgcc. Its link builds read nothing under `/usr/bin`, on NixOS too.

The shell does not provide what `cargo test --workspace` and the
conformance corpus also need (README): the `stable` toolchain through
rustup, and `/usr/bin/python3` with MPFR for the float oracle. Cargo itself
links the workspace's own binaries with the shell's `cc`.

`tidelang-std` is built with Cargo's dev profile, the one README's demo
build, the driver's tests and the conformance corpus use, and is not
stripped. The driver is built with `--release`. Neither package runs its
tests in the flake; `cargo test --workspace` owns them.

## Demo

`packages.tide-demo` runs the demo the way the dev shell would: `RUSTC`,
`TIDE_LINKER_BIN` and `TIDE_RUNTIME_LIB_DIRS` as above, and Cargo with the
crates `Cargo.lock` names vendored, so it builds offline. Its sources are
the workspace, `demo/` and `rust-toolchain.toml`, read-only in the store,
so the demo prints `revision:  unknown`.

Where the demo writes: it and its children write under `$CARGO_TARGET_DIR` when it is set, else
under `$XDG_CACHE_HOME/tide-demo` (by default `~/.cache/tide-demo`). A
relative `CARGO_TARGET_DIR` is taken from the directory the demo was
started in, as Cargo takes it. The target's `tide-demo-home/` holds their
Cargo home and temporary directory (`CARGO_HOME`, `TMPDIR`), so a caller's
`~/.cargo` and the configuration files in it are neither read nor
written; Cargo's environment configuration, such as `RUSTFLAGS` and
`CARGO_*` variables, still applies. The one exception is the C linker's
temporary files when the driver links: the driver starts its compiler with
an empty environment, so they go to `/tmp`. With a read-only root
filesystem, the target and a writable `/tmp` are all the demo needs.
Nix's own writes while it fetches or builds the app, to the store and its
cache, are Nix's, not the demo's.

Its `PATH` holds only Cargo, the C toolchain, git and coreutils, and
replaces the caller's, so a rustup there is not used. Of the toolchain,
only its cargo component (`toolchain.availableComponents.cargo`) is on
`PATH`, since `RUSTC` names the sysroot's rustc, which keeps the docs,
clippy and rustfmt out of its 2.1 GiB closure (4.3 GiB with the joined
toolchain on `PATH`). Without gdb, the demo skips its debugger step.

## Inputs (AP-DEPENDENCY-001)

**nixpkgs** `567a49d1913c` (2026-06-16), Eon's pin.

- Capability: `stdenv` (the GCC cc wrapper, binutils, glibc) for linking,
  and `makeRustPlatform`'s `importCargoLock` and `cargoSetupHook`, which
  vendor `Cargo.lock`'s crates for an offline Cargo build; for the demo's
  script, `writeShellApplication`, `gitMinimal` and coreutils.
- Alternatives: a newer nixpkgs gains nothing the build uses. Sharing
  Eon's pin keeps one C toolchain in the store for both projects.

**rust-overlay** `49f68bf7cd35` (2026-11-17).

- Capability: `rust-bin.fromRustupToolchainFile ./rust-toolchain.toml`, a
  toolchain for the pinned nightly with `rustc-dev` and `rust-src`, read
  from the file rustup also reads, so the pin has one owner.
- Alternatives: nixpkgs' own rustc is stable and ships no `rustc-dev`.
  fenix also reads a toolchain file, but it needs the manifest's hash in
  Nix, a second value to update with every pin bump. Eon's rust-overlay pin
  (2026-07-04) predates `nightly-2026-08-09`, so this input is newer than
  Eon's.

**Not used:**

- crane: its gain is caching dependency builds between rebuilds. The
  workspace builds in two small Cargo runs, so it would add an input and
  its build conventions for little.
- `buildRustPackage`: it passes `--target`, which puts the std's
  proc-macro in a different directory from its libraries, and installs
  binaries only. A plain `stdenv.mkDerivation` with `cargoSetupHook` runs
  the same `cargo build` README does.

**Pin:** `flake.lock`. When `rust-toolchain.toml` moves to a nightly the
overlay does not list yet, move the rust-overlay input to a newer commit.

**Replacement cost:** `flake.nix` alone. The driver depends on no Nix
input; any packager can set the four compile-time variables above.

**Proof:** `checks.x86_64-linux.link`, run with
`nix build .#checks.x86_64-linux.link`. The build sandbox has no
`/usr/bin` and no `~/.rustup`. Nix may build without it when
`sandbox-fallback` is true and the kernel refuses it, so acceptance builds
pass `--option sandbox true --option sandbox-fallback false`, which only a
trusted user can set. The check also fails if `/usr/bin/cc` or
`/usr/bin/ld` exists, each tested by its own command; that is evidence of
the sandbox only on hosts that have them. Its grep for `/usr/bin` covers
the driver, the std's proc-macro, the linked program and the test binary,
and fails on a match or a read error. It skips the std's rlibs: syn's rlib
carries its docs' `#!/usr/bin/env rustx` shebang example, which nothing
reads as a path.
