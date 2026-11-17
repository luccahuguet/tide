Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 6fd58bf (re-check of cf3a8fd findings)

## Verdict

**approve.** All eight findings are resolved, and the fixes broke nothing I could find.

Scope: this is a finder re-check of B1, B2 and N1–N6 only. It reviewed `git diff cf3a8fd 6fd58bf` (`invocation.rs`, `flake.nix`, `docs/nix.md`) and the builder's round-2 logs. The worktree is clean at 6fd58bf. I ran no builds, Nix commands or tests.

## Findings

**B1, dev shell driver fails every link build: resolved.**
- What changed: `devShells.default` now sets `env.RUSTC = "${sysroot}/bin/rustc"`, plus `TIDE_LINKER_BIN` and `TIDE_RUNTIME_LIB_DIRS`. They use the same `linkerBin` and `runtimeLibDirs` bindings as the packages, so each value has one owner.
- Compile-time only:
  - The driver reads `TIDE_*` only through `option_env!`. A search of `crates/` finds no runtime `env::var` of any `TIDE_*` variable.
  - `RUSTC` is read at run time only by test harnesses (`tide-admit/tests`, `tide-std/tests`), never by the driver, and there it names the same pinned nightly.
  - The compiler process still re-execs with `env_clear()`, so the shell's variables cannot reach rustc or the linker at run time.
- Proof: `devshell-link-test.log` echoes the three values, all store paths: the sysroot copy, gcc-wrapper, and glibc with gcc-lib. It shows `test result: ok. 2 passed`.
  - The second test, `replay_refusal_preserves_existing_outputs`, links, records a receipt and refuses a replay. That runs `loaded_identity`, the exact path that broke.

**B2, the absence assertion did nothing under `&&`: resolved.**
- The check now runs `test ! -e /usr/bin/cc` and `test ! -e /usr/bin/ld` as two separate commands under `set -eu`, so each one fails the check on its own.
- docs/nix.md:128-130 now describes that accurately, including that it is evidence only on hosts that have those files.
- `nix-build-r2.log` now records the effective daemon settings and the exact `--option sandbox true --option sandbox-fallback false` command, and it ends `exit=0`. That closes the provenance gap Codex noted.

**N1, closure cause double-counted: resolved.**
- docs/nix.md:59-62 now gives the closure as 2.1 GiB: a 1.2 GiB sysroot copy, a 403 MiB rustc component (NAR size) that the copy's RUNPATHs still name, and the C toolchain with the 80 MiB tidelang-std.
- The 1.2 GiB matches my earlier read-only measurement (about 1.24 GiB apparent size).
- Dropping the rustc component from the copy's RUNPATHs stays an acknowledged follow-up, not a blocker.

**N2, std stored twice: resolved.**
- The `cp …/libtide_std.rlib $out/lib/` line is deleted, and `TIDE_DEFAULT_STD` now points at `${tidelang-std}/lib/deps/libtide_std.rlib`.
- docs/nix.md's table names that path.
- The check passed with the new default (`nix-build-r2.log`).

**N3, two owners of the `cc` name: resolved.**
- `rustc_args` now takes `let [cc, _] = linker_tools()` and passes `-Clinker={cc}`.
- With the variable unset, `Path::new("/usr/bin").join("cc")` is `/usr/bin/cc`, byte-identical to main's.

**N4, dev shell limits undocumented: resolved.**
- docs/nix.md has a new "Dev shell" section. It covers the three variables and their reasons, and what the shell does not provide: the stable toolchain, and `/usr/bin/python3` with MPFR.
- Its claim that the shell-built driver reads nothing under `/usr/bin` matches the B1 proof and the builder's `strings` observation.

**N5, compile-time paths not validated: resolved; the defaults are unchanged.** I traced `absolute_entries` by hand:

| Input | Result | Why |
| --- | --- | --- |
| `""` | true | the default, an empty list, passes |
| `"/a"`, `"/a:/b"` | true | every entry is absolute |
| `"a"`, `":/a"` | false | the first entry is not absolute |
| `"/a:"`, `"/a::/b"` | false | trailing or inner empty entry |
| `"/a:rel"` | false | relative entry |

- `LINKER_BIN` must start with `/`, so an empty or relative value fails, and the default `/usr/bin` passes.
- The assertions are in a `const _` block, so a bad value is a compile error, not a run-time check.
- The builder's `cargo check` cases (`bin`, `/a:`, `/a:rel` fail; `/a:/b` compiles) agree with the trace.
- Both default builds compile: `cargo-test-r2.log` shows 370 passed and 0 failed with the variables unset, and `nix-build-r2.log` passes with the Nix values set.

**N6, grep error counted as no match: resolved.**
- `status=0; grep -lF /usr/bin $files || status=$?`, followed by a failure unless the status is 1, fails on a match (0) and on a read error (2).
- The `||` keeps `set -e` from ending the script before the status is read.

No regression found. The check's RUN-3 comment now correctly names the trusted runtime directories.

DONE
