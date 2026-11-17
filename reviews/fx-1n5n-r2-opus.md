Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate cf3a8fd

Opus converge round (GOVERNANCE.md Merge Rule) for fx-1n5n. It is read-only and reports without fixing. It reviews `git diff main...cf3a8fd` (merge base c3fe999) on the Codex-clean tree (r1-recheck-codex.md approved).

## Verdict

**request_changes**. Confidence: high on both blocking findings. Each was checked directly: B1 against the builder's own failing log and the flake, and B2 by running the shell construct.

The driver change is sound. All three trust-boundary lenses pass and the non-Nix behaviour equals main's. The two blockers are in the flake and in the proof it documents. Both fixes are small and touch no Rust.

## Blocking findings

### B1. The dev shell builds a driver whose every link build fails, on any host

`flake.nix:174-177` (`packages = [ toolchain ]`), `docs/nix.md:11`.

The dev shell puts rust-overlay's joined toolchain (`rust-default-1.99.0-nightly-2026-08-09`) on PATH and does not set `RUSTC`. A Cargo build of the workspace in that shell runs `crates/tide-admit/build.rs`, which bakes `TIDE_SYSROOT=$(rustc --print sysroot)`. That value is the joined store path. In it, the std rlibs are symlinks into the component store paths.

Because of that:
- `after_analysis` (`invocation.rs:172-184`) resolves each loaded sysroot rlib outside the canonical sysroot and keeps it in `loaded`.
- `loaded_identity` (`replay.rs:116-131`) then fails with "the build loaded …/libaddr2line-….rlib, which is in no --tide-dep-dir". `publish_receipt` calls it on every successful link build (`invocation.rs:648`, `:1266`).

This is the failure the builder already hit: `target/work/nix-build-2.log:63` shows that message with the same joined-toolchain path. The r1 fix routes around it for the packages only (`RUSTC = "${sysroot}/bin/rustc"` in `cargoBuild`). The dev shell was never moved to the copy. The only dev-shell proof recorded is `rustc -vV`.

Concrete failing case: on any x86_64 host, run `nix develop`, then `cargo test --workspace`. Every driver test that links fails. So does `cargo run -p tide-admit -- --emit=link …` with an explicit `--tide-std`.

The bead's Scope asks for "a dev shell with cargo for what the flake does not provide", and fx-qlek.1 builds its install path on that shell. As shipped, the shell cannot produce a working driver.

Minimum fix: in `flake.nix`, give the shell the materialized sysroot's compiler:
```nix
devShells.${system}.default = pkgs.mkShell {
  packages = [ toolchain ];
  env.RUSTC = "${sysroot}/bin/rustc";
};
```
Then state the remaining limit in docs/nix.md (see N4). The proof is one focused link test, or the demo build, run with heavy.sh from `nix develop`.

Recommended in the same edit, but not required for this bead: also set `TIDE_LINKER_BIN`, `TIDE_RUNTIME_LIB_DIRS` (the same `linkerBin` and `runtimeLibDirs` values) in `env`. Without them, a shell-built driver links with `/usr/bin/cc`. That fails on NixOS, and it breaks fx-qlek.1's acceptance ("no step reads /usr/bin/cc"). These variables are still read only at compile time.

### B2. The check's `/usr/bin/cc` absence assertion does nothing in the one case docs say it covers

`flake.nix:189`, `docs/nix.md:105-106`.

`test ! -e /usr/bin/cc && test ! -e /usr/bin/ld` runs under `set -eu`. Bash does not exit when a command that is not the last in an `&&` list fails. If `/usr/bin/cc` exists, the first `test` fails, the list stops, and the script goes on.

Reproduced here:

```
bash -c 'set -eu -o pipefail; test ! -e /bin/sh && test ! -e /nonexistent; echo continued'
```

It prints `continued` and exits 0.

So on a host that has `/usr/bin/cc`, the assertion cannot fail. docs/nix.md:105-106 says the opposite: "evidence of the sandbox only on hosts that have `/usr/bin/cc`". The round-0 report's argument ("an unsandboxed fallback would fail that assertion") was also wrong.

The r1 acceptance evidence now rests on `--option sandbox-fallback false`, and the build log does not echo that flag (Codex noted this). That leaves the check's self-evidence as the only on-tree proof of "no build step reads /usr/bin", and it is void.

Minimum fix: make each assertion a separate command:
```sh
test ! -e /usr/bin/cc
test ! -e /usr/bin/ld
```
Alternatively, delete the line and the docs sentence and rely only on the documented `sandbox-fallback false`. Either way, docs/nix.md:105-106 must match the code.

## Non-blocking findings

N1. **The closure's stated cause double-counts** (`docs/nix.md:54-56`).
- Read-only measurement of `/nix/store/pashzkwa…-tidelang-sysroot-…` gives about 1.24 GiB apparent size: `rustlib/x86_64-unknown-linux-gnu` 804 MiB, libLLVM 202 MiB, librustc_driver 158 MiB, rust-src 70 MiB, rustc-src 34 MiB.
- The "1.7 GiB sysroot" is therefore the sysroot's closure. That closure already includes the 454 MiB rustc component, kept alive by the copy's RUNPATH: `bin/rustc` and `librustc_driver` both still list `…-rustc-1.99.0-nightly-…/lib`.
- The rest of the 2.1 GiB is the C toolchain the driver links with (gcc wrapper, gcc, binutils, glibc) and tidelang-std.
- Suggested rewording: "2.1 GiB: the 1.2 GiB sysroot copy, the 454 MiB rustc component its search paths still name, and the C toolchain".
- The rustc component duplicates librustc_driver and libLLVM. The sysroot derivation could drop it by removing that component's directory from each rewritten RUNPATH, saving about 450 MiB. This is a follow-up, as the report says.

N2. **Unnecessary std copy** (`flake.nix:135`, `:147`).
- `cp $out/lib/deps/libtide_std.rlib $out/lib/` stores the std twice.
- Pointing `TIDE_DEFAULT_STD` at `lib/deps/libtide_std.rlib` would delete one line and one copy. docs/nix.md's table would then name that path.
- This matches README's demo build, whose `--tide-std` lies inside a `--tide-dep-dir`.

N3. **Duplicate owner of the `cc` name** (`invocation.rs:429`).
- `rustc_args` spells `format!("-Clinker={LINKER_BIN}/cc")` itself, while `linker_tools()` owns `DIR/cc` for the cache key and receipt. They agree today.
- Using `linker_tools()[0]` there would leave a single owner for the linker path, as the report claims there is.

N4. **The dev shell's limits are not documented** (`docs/nix.md:11`, `:24-25`).
- docs/nix.md does not say that a driver built in the shell links with `/usr/bin/cc`, so link mode fails on NixOS, unless B1's recommended variables are set.
- It also does not say that `cargo test --workspace` additionally needs stable rustc and `/usr/bin/python3` with MPFR, which the shell does not provide (README:150-153).
- The brief asked for the NixOS limit to be stated. Today it appears only in the builder's report.

N5. **The compile-time paths are not validated** (`invocation.rs:72-99`).
- A relative or empty `TIDE_LINKER_BIN` or `TIDE_RUNTIME_LIB_DIRS` entry would resolve against the operator's working directory at run time. The working directory would then choose the linker or a grant.
- Only a packager can set these values, and the flake passes absolute store paths, so this is not reachable today.
- A `const` assertion that each value starts with `/` would turn a packaging mistake into a compile error.

N6. **The `/usr/bin` grep passes on grep errors** (`flake.nix` check).
- `if grep -lF /usr/bin $files` treats grep's exit status 2 (an unreadable file) as "no match".
- `grep -lF /usr/bin $files && exit 1` with an explicit status check would close that. This is a nit.

## Reasoning, by lens

**1. Trust boundaries.**
- RUN-3: the only new grant source is the compile-time `RUNTIME_LIB_DIRS`, searched before `SYSTEM_DIRS` for bare `DT_NEEDED` names.
  - The candidate binary still supplies only `PT_INTERP` and `DT_NEEDED`, exactly as on main.
  - Its `DT_RUNPATH`, `DT_RPATH` and `DT_SONAME` are not parsed.
  - Lookup depends only on the name and a fixed list, so traversal order no longer matters.
  - Every visited file must parse as ELF64, so the text file `libc.so` in the glibc directory fails closed.
  - The flake's directories are immutable store paths: the glibc that is the linked program's interpreter, and gcc-lib.
  - A candidate can name any ELF in those directories. That is the same class of grant as the system directories on main.
  - The new test `a_library_only_in_the_binarys_runpath_gets_no_grant` guards the reverted mechanism, which counts as a security absence contract under AP-TEST-001.
  - No new steering route.
- Cache key and receipt: both hash `linker_tools()` (`DIR/cc`, `DIR/ld`).
  - Under Nix these are wrapper scripts that embed the store paths of gcc and binutils, so they identify the toolchain at least as well as `/usr/bin/cc` does.
  - The default std and dep dir are inserted into `Invocation` before canonicalization. They are therefore fingerprinted and receipted exactly like explicit ones.
  - The driver digest covers the baked constants.
- Compile-time only: all four variables are `option_env!` values.
  - The compiler process re-execs with `env_clear()` (`invocation.rs:1176`, `:1277`). rustc, and the linker it spawns, therefore see no `NIX_*` variables, so the cc wrapper's runtime environment hooks cannot steer the link.
  - Cargo records `option_env!` in dep-info, so changing a value rebuilds the driver.
- Explicit `--tide-std`: the default is inserted only when `std.is_none()` after parsing, so an explicit std replaces both values.
  - With only `--tide-dep-dir` given, the operator's directories come first and the default directory is appended. That is deterministic, and the receipt records the order.
  - Replay against a recording made with another std fails at the std digest or at the option comparison, so it fails closed.

**2. Non-Nix equivalence.** With every variable unset:
- `-Clinker=/usr/bin/cc` and `-Clink-arg=-B/usr/bin` are byte-identical to main's.
- `linker_tools()` gives `/usr/bin/cc` and `/usr/bin/ld` in main's order.
- `"".split(':')` is filtered to nothing, so `SYSTEM_DIRS` is searched in main's order.
- `DEFAULT_STD` is `None`, and a half-set pair is a compile error.
- `after_analysis` matches main's (the spelled-path exemption is reverted).

Equivalent.

**3. Nix correctness.**
- Names: `tidelang-admit` (default package and app) and `tidelang-std`. There is no `tidelang` attribute and no exposed macros, as fx-gbq4 requires.
- Pins: both inputs are pinned by rev in the URL and by narHash in the lock. rust-overlay follows nixpkgs. `fromRustupToolchainFile ./rust-toolchain.toml` owns the nightly and its components. `importCargoLock` vendors the crates for an `--offline --locked` build.
- The sysroot copy:
  - The RUNPATH rewrite targets exactly the joined path.
  - Its self-checks (no symlinks remain, no reference to the joined path remains, `rustc --print sysroot` equals `$out`) are the right falsifiers.
  - Read-only inspection confirms that the driver's RUNPATH and embedded `--sysroot` name `pashzkwa…-tidelang-sysroot` and that the copy's rustc lists itself first.
  - The checks do not catch component paths left in RUNPATH (see N1). That costs closure size, not correctness: the copy's `lib` comes first.
- The link inside the sandbox and the run under RUN-3 are shown in `nix-build-r1.log`.
- "Nothing from /usr/bin or ~/.rustup" holds for the packages, given the sandbox. The check's own evidence for it is void (B2).
- The dev shell fails its purpose (B1).

**4. Simplicity and ownership.**
- One compile-time owner exists for the linker directory and one for the runtime directories.
- `cargoBuild` has two callers.
- Rejecting crane, `buildRustPackage` and a wrapper script is justified in docs/nix.md.
- Concrete deletions: the extra std copy (N2), the second spelling of `cc` (N3), and, as a follow-up, the rustc component in RUNPATH (N1).
- Otherwise the flake is close to the minimum the bead needs.

**5. Docs.**
- The RUN-3 paragraph in README matches `libraries()`.
- The linker and default-std paragraphs match the code.
- The `main.rs` module docs are updated.
- design/03 RUN-3 needs no change.
- Wrong or missing: the absence-assertion sentence (B2), the closure cause (N1), and the dev-shell limits (N4).

Not run, as briefed: no builds, Nix commands or tests. The only reads of the store were `readelf`, `ls`, `du` and `strings` on the builder's recorded output paths.

DONE
