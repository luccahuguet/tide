# Install Tide on Linux

This quickstart installs Tide's admission driver, `tide-admit`, on x86_64
Linux through the repository's Nix flake, builds and runs one example, and
runs its generated test in the kernel sandbox. The alpha has no `tide`
command; `tide-admit` is the whole toolchain, with Tide's std as its default.

You need an x86_64 Linux machine with `curl`, `xz` and `git` (on Debian or
Ubuntu, `sudo apt-get update && sudo apt-get install curl xz-utils git`) and
about 10 GB of free disk space, 7 GB of it for `/nix`. Nothing else: Nix
provides the Rust toolchain and the C linker, so no step uses rustup or the
system's C compiler. On macOS or Windows, [docker.md](docker.md) runs this
route in a Linux container.

## 1. Install Nix

Run these in bash or zsh:

```sh
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --no-daemon
. ~/.nix-profile/etc/profile.d/nix.sh
```

This is Nix's single-user install; it asks for `sudo` only to create `/nix`.
The second line puts `nix` on this shell's `PATH`, as new shells get it.
Nix's multi-user install (`--daemon`) is the other option Nix documents; open
a new shell after it instead of running the second line.

The commands below turn on flakes for that command alone with
`--extra-experimental-features`. To turn them on for good, add
`experimental-features = nix-command flakes` to `~/.config/nix/nix.conf`.

## 2. Get the repository

```sh
git clone <repository-url> tide
cd tide
```

<!-- The owner fills in <repository-url> at release. -->

## 3. Build the driver

```sh
nix --extra-experimental-features 'nix-command flakes' build .#tidelang-admit
```

The first build downloads the pinned Rust nightly from static.rust-lang.org
and Nix's C toolchain from cache.nixos.org, and compiles the driver and Tide's std from source, which
takes a few minutes. `result/bin/tide-admit` is the driver.
`nix --extra-experimental-features 'nix-command flakes' run .#tidelang-admit -- ARGS`
runs it without the `result` link.

## 4. Build and run an example

[`examples/contracts/sorted_insert.rs`](../examples/contracts/sorted_insert.rs)
states a function's contract once, with `#[requires]` and `#[ensures]`, and
has no hand-written test.

```sh
result/bin/tide-admit --crate-type=bin --emit=link --crate-name=sorted_insert \
  --out-dir target/quickstart examples/contracts/sorted_insert.rs
target/quickstart/sorted_insert
```

The driver admits the file, compiles it against Tide's std and links it with
Nix's linker. The program prints `[3, 7, 9, 14, 20]`. Code that breaks a Tide
rule fails admission with a named code instead, such as
[`examples/capabilities/ambient_read.rs`](../examples/capabilities/ambient_read.rs)
with `TD0501`.

## 5. Run its test in the sandbox

```sh
result/bin/tide-admit --test --emit=link --crate-name=sorted_insert \
  --out-dir target/quickstart-tests examples/contracts/sorted_insert.rs \
  --tide-run-test __tide_contract_tests::insert_sorted_3
```

The test build generates one test from `insert_sorted`'s contract, which feeds
it sorted vectors and checks every `#[ensures]`. `--tide-run-test` runs that
test in the kernel sandbox (RUN-3). Before the test starts, the driver prints
the sandbox level to standard error; the output ends:

```text
sandbox: full (Linux 7.2.6-…, Landlock ABI 10)

running 1 test
property __tide_contract_tests::r#insert_sorted_3: 100/100 cases passed (accepted/generated)
test __tide_contract_tests::insert_sorted_3 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; …
```

`full` means Landlock and seccomp both confine the test. When the kernel lacks
Landlock, its scopes or seccomp, or they are disabled or blocked, the line
reads `sandbox: partial, not enforced: …` and names what is missing; [driver.md](driver.md)'s
RUN-3 paragraph says what each level means.
`target/quickstart-tests/sorted_insert-test --list` lists the generated tests.

## 6. Work on Tide with cargo

```sh
nix --extra-experimental-features 'nix-command flakes' develop
```

The shell has the pinned toolchain, cargo included, and builds the driver
with the flake's sysroot, linker and runtime library directories, so
`cargo build` works there. It does not provide
what `cargo test --workspace` and the conformance corpus also need: the
`stable` toolchain through rustup, and `/usr/bin/python3` with MPFR for the
float oracle ([nix.md](nix.md), Dev shell).
