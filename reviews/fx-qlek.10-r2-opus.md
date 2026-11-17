Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 0c98c65

# fx-qlek.10 Opus review (converge round, report only)

## Verdict

request_changes. There is one blocking finding, and it is text-only: the fix is a README sentence, and under AP-PROOF-001 a text-only fix needs no re-check. The code needs no change. Check parity with run.sh holds.

## Blocking findings

### B1. README.md:133 says `cargo test --workspace` now includes "the runner's tests" but does not list the two prerequisites those tests add

**Where:**
- `README.md:133`. The owner's guide setup list, `docs/owner-guide/README.md:77-78`, names only "rustup and the pinned toolchain".
- The prerequisites come from `crates/tide-conformance/src/main.rs:889-938` (`a_stale_float_fixture_fails_the_oracle`) and `crates/tide-conformance/tests/runner.rs:19-28`.

**Failure scenario.** Before this change, the workspace suite needed only rustup and the pinned nightly. Now it also needs:
- (a) the `stable` toolchain. `runner.rs` runs the binary, and the binary runs `rustup run stable cargo build --offline` on the stub std. If `stable` is not installed, the runner exits 2 and the test fails.
- (b) `/usr/bin/python3` that can load `libmpfr.so.6`. Without the interpreter, the unit test panics with `Setup("cannot run /usr/bin/python3: …")`. Without MPFR, it fails with "the unchanged copy fails" and an `OSError` from ctypes.

These environments all fail the suite:
- a NixOS host;
- a nix-built image (fx-qlek.2's planned Docker image has no `/usr/bin/python3`);
- a slim container;
- a rustup install with only the pinned nightly.

A new alpha user who follows the README sees `cargo test --workspace` fail with nothing in the docs explaining why, and may read it as Tide being broken. AP-CONTRACT-001 requires docs to state availability. This change edits that README line, so the fix belongs to a tier the change edits.

**Is the requirement acceptable for the alpha? Yes, if it is documented.**
- The alpha targets x86_64 Linux. On mainstream distributions both prerequisites are usually already present: python3 is in the default install, and gcc, which Rust's default linker setup needs, depends on MPFR (Debian/Ubuntu `libmpfr6`, Fedora and Arch `mpfr`).
- The full corpus already needs the same interpreter and the stable toolchain. Any machine that can produce the published "cases pass" claim therefore also passes this test.

**Minimum fix:** one sentence next to the README's command block, for example:

> `cargo test --workspace` and the corpus also need the `stable` toolchain (`rustup toolchain install stable`) and `/usr/bin/python3` with MPFR (`libmpfr.so.6`; on Debian/Ubuntu, `python3` and `libmpfr6`), which the float oracle uses.

Adding the same clause to the owner's guide Setup list is recommended but not required, because the guide never tells readers to run the workspace suite.

**I reject the skip-with-notice alternative.**
- libtest captures the output of passing tests. A runtime "skip" therefore prints nothing anyone sees, and it turns into a test that can silently never run.
- A skip gains nothing: wherever it would apply, the full corpus exits 2 anyway. Keeping the test strict and documenting the need costs less and hides nothing.
- Also keep the corpus run's exit 2 for a missing oracle, as it is now.

## Non-blocking findings

**N1. The missing-oracle setup error comes late** (`main.rs:589-593`, `619-629`).
- The Python/MPFR probe runs only when the run reaches `authority/float_math`. That is after both stub builds, the driver build, and every case that sorts before it.
- It then aborts with exit 2. The cases after it are never checked, and no count line is printed.
- The contract still holds: exit 2, never a pass. It just wastes minutes and leaves partial output.
- Fix: run the probe in `Suite::build` whenever the case list includes the float case.

**N2. The prerequisite failures in the unit test read as test bugs** (`main.rs:911-919`).
- The test's `merged(oracle).unwrap()` and its first `assert_eq!` show a missing prerequisite as a raw panic or an assertion failure.
- Running the same probe first, and panicking with the runner's own message (`… needs /usr/bin/python3 with MPFR (libmpfr.so.6)`), would make the cause obvious. This complements B1 and does not replace it.

**N3. The word "oracle" means two things in `conformance/README.md:21-25`.**
- "`stable` for the oracle" and "the stable oracle stub std" use "oracle" for stable rustc.
- The next section introduces "the independent oracle `oracles/float_math.py`".
- Saying "stable rustc" in the first two would remove the ambiguity. This is a nit.

**N4. The runner does not check stable's version. This predates the change and is out of scope.**
- `conformance/README.md` names "stable rustc and cargo 1.96". The runner, like run.sh before it, uses whatever `stable` resolves to.
- The published "cases pass" claim therefore depends on a toolchain version that nothing records.
- Printing `rustc +stable --version` in the count line, or checking it, would pin the claim. This could be a candidate for the fx-qlek.7 launch-claims audit. I am reporting it, not filing it.

**N5. A run that finds no cases passes.**
- A case directory with no `.rs` files gives `0 cases: 0 PASS, 0 AWAITS CHECKER, 0 FAIL` and exit 0. run.sh also exited 0 in that case.
- A missing `cases/` directory is now a setup error (exit 2), which is stricter than run.sh.
- Treating zero cases as a setup error in the default run would close the remaining gap. This is optional.

**N6. Sharing one target directory between parallel runs can collide on outputs. This is only a note.**
- `runner.rs` checks a case named `cap_ambient` in the same `<target>/conformance/conformance/` output directory as the real corpus run.
- If the workspace suite and a corpus run use the same `CARGO_TARGET_DIR` at the same time, they overwrite each other's `libcap_ambient.rmeta`.
- heavy.sh's single suite slot serializes them, and the README already says to give each checkout its own target directory.

## Reasoning

### Parity with `09e741e:conformance/run.sh`

I compared the deleted script with `main.rs` check by check. I found no expectation that run.sh enforced and the runner drops, and no input that run.sh would FAIL but the runner would PASS.

**Case-file checks:**
- **Rule line:** `rules_of` matches `sed '1s#^//@ rules: *##p' | tr ', '` (split on `,` and space, empty pieces dropped).
- **Rule IDs:** IDs are checked by exact membership, as `grep -vxF` did. A trailing `\r` or a tab fails in both.
- **`rules.md` IDs:** `rule_id` matches the sed pattern. It is the same or stricter: ASCII-only, where sed's `[A-Z]` depends on the locale.
- **Unknown `//@` lines:** the check is the same, with the same exact list of directives, applied from line 2 on.
- **Marks:**
  - `Marks::parse` reproduces the awk program: the first `//~` on a line, tokens split on spaces and tabs, `WARN` only before a final token and only once, `TD####` at the current severity, `E####|error` only while the severity is still error, and everything else malformed.
  - The unit test `marks` covers these edge cases.
- **Binary or library:** `fn main` at column 0, then a character that is not a word character, matching `grep -qw '^fn main'`.
- **Crate name:** derived byte by byte, matching `basename .rs | tr -c`.

**rustc stage:**
- The arguments are the same in both modes, including `RUSTC_BOOTSTRAP=1 -Zallow-features=`.
- Stdout and stderr go to one pipe, as `2>&1` did.
- "Exit 1 if either run fails" holds.
- `rustc_errors` reproduces the awk logic:
  - the leftmost `:L:C: error([E…])?:`;
  - a path prefix kept for diagnostics in other files;
  - `?:error` for an unlocated error;
  - `aborting due to` skipped.
- The fail condition `want != got || (want empty && code != 0)` is the same.

**Driver stage:**
- The flags are the same and in the same order. The grant and profile are found by exact whole-line match.
- The exit status is the maximum over both runs, and death by signal counts as 128 + the signal number.
- **Crash test:** exit status above 1, `internal compiler error`, or `thread '…' panicked`. `panicked()` is equivalent to `thread '.*' panicked`, because it takes the first `thread '` and so searches the widest suffix.
- `driver_diagnostics` matches the `(error|warning): TD####:` regex with severity kept.
- **Final condition:** `driven != dgot || rejects == (dcode == 0)` is algebraically equal to run.sh's `[[ … ]]`, where `&&` binds tighter than `||`.
- **Codes:** the driven/AWAITS split uses the driver's `--tide-print-codes` set. AWAITS is reported only after both checkers agree, as before.

**Strengthened over run.sh, not weakened:**
- Failures to start a tool, a missing `cases/` directory, and an unreadable case are setup errors (exit 2). run.sh turned them into FAIL lines or, for `find` errors, exit 0.
- The float oracle is called. run.sh never called it; the bead and tide-lead's decision 3 ask for it. It runs isolated with `-I`, and Codex's re-check confirmed the `PYTHONOPTIMIZE` fix.
- Pinned-stub build errors are rendered (`json-render-diagnostics`). The JSON decode is strict, and `required` on an `Option` field accepts messages that have no `filenames`.

### Toolchain and environment

- **Stable:** `rustup run stable` with `RUSTUP_TOOLCHAIN` removed is equivalent to `RUSTUP_TOOLCHAIN=stable`.
- **Pinned nightly:** `rustup run <channel>` reads the one owner, `rust-toolchain.toml` (`channel = "nightly-2026-08-09"`). Its missing component auto-install does not matter, because `cargo run`/`cargo test` from the repository already installed the pinned toolchain from that file.
- **Working directory:** commands now run with the repository root as their working directory, where run.sh used the caller's. No `.cargo/config.toml` exists, and every path is absolute, so this changes nothing.
- **`cargo run`'s `LD_LIBRARY_PATH`** includes the nightly's lib directory. Stable rustc's `librustc_driver`/`libLLVM` sonames are version-hashed, so they cannot be picked up from it, and the parity run went through `cargo run`.
- **Python:** `-I` drops `PYTHON*` variables and keeps the script and working directories off `sys.path`. The oracle imports only the standard library.

### Self-tests: races and leftovers

- `runner.rs` uses a per-PID directory under `CARGO_TARGET_TMPDIR` with a Drop guard. Its single test runs alone in its binary, and cargo runs test binaries one after another, so its nested `cargo build -p tide-admit` cannot race other test binaries.
- The suite log shows that nested build was fresh (4.2 s), so it does not overwrite the outer suite's driver.
- The unit test's per-PID directory under `temp_dir()` is cleaned on unwind; only SIGKILL leaves it behind.
- The mutation follows the Acceptance: a copied case, never the corpus, with exit 1 and the case named in the FAIL line.

### Evidence

- `git diff 09e741e...0c98c65 | sha256sum` gives `deda50f5…bcf94`, matching the brief.
- `git diff --check` is clean, and the worktree is clean at `0c98c65f`.
- The stub's `Cargo.lock` pins the same versions as the root lock for every shared crate, so the offline stub build works after a workspace fetch.
- The parity tables, the logs, and the suite logs (356 passed) are the builder's. I ran no builds, suites, or subagents, and I changed no file other than this report.

DONE
