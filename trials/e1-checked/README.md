# E1 readiness machinery

This directory implements bead `fx-o8o` against the frozen `protocol.md` and
`tasks.md`. It contains no gate-writer output and reports no E1 gate result.
The suites and renderer are drafts: **not frozen**. `fx-ic2` owns their
pre-session freeze and tree SHA-256.

## Contents

- `references/common/`, `references/rust/`, `references/tide/`: 60 logical
  reference solutions. Eight identical core-language solutions share one file;
  every rendered solution is one standalone source file.
- `suites/common/`, `suites/rust/`, `suites/tide/`: library acceptance tests.
  `suites/binaries.json` supplies the 26 process invocations for B1/B2/B5/B8.
  Multiple assertions/subtests implement the numbered cases; no extra input
  requirements are added. File fixtures and the fixed xorshift generator are
  shared test setup, not given to writers.
- `harness.py`: arm rendering, skeleton preparation, `./check` and `./test`,
  snapshots/budgets, structure searches, acceptance compilation/execution,
  and reference readiness runs. Python 3.11+ standard library only.
  `doctest-runner.sh` applies the per-test limit to both standalone and merged
  Rust 2024 doctests, using the recorded 1.96.0 runner protocol.
- `rust/`: edition-2024 manifest and pinned lockfile for serde 1.0.229,
  serde_json 1.0.154, thiserror 2.0.21 and ten transitive crates. Rust setup
  vendors them offline into `target/e1-checked/rust-vendor-<lock hash prefix>`.
  The generated tree retains licenses/checksums and is mounted read-only.
- `readiness.md`: observed results, prerequisite ownership, and remaining work.
  `reference-review.md` records this draft's concurrency inspection. It does
  not replace the independent governance reviews or per-writer reviews.

## Commands (from the repository root)

```sh
python3 trials/e1-checked/harness_test.py
python3 trials/e1-checked/harness.py render A1 tide
python3 trials/e1-checked/harness.py prepare A1 rust "$PWD/target/e1-checked/writer-rust-A1"
# prepare prints the operator session directory, outside the writer directory.
# The isolated writer gets the prepared directory and logical ./check, ./test tools.
python3 trials/e1-checked/harness.py command SESSION check
python3 trials/e1-checked/harness.py command SESSION test
python3 trials/e1-checked/harness.py grade SESSION --first --compile-only
python3 trials/e1-checked/harness.py grade SESSION
python3 trials/e1-checked/harness.py references rust
python3 trials/e1-checked/harness.py references tide
python3 trials/e1-checked/harness.py references rust --local
python3 trials/e1-checked/harness.py references tide --local
# Optional task subsets follow the arm, e.g. references rust C10 --local.
```

Use the host's configured build limiter for compilation/execution batches.

Exit 0 means the requested checks passed, 1 means failure, and 2 is setup or
budget failure. Reference runs write command/exit/output records to
`target/e1-checked/{rust,tide}-results.json`, including failures. The directory
belongs to the checkout running the harness. A later run of the same arm
replaces that arm's scratch report. Writer snapshots and command records stay
under `target/e1-checked/sessions/`; each command's `run/` tree is removed after
its result is recorded. Reference sessions live in the batch's temporary
directory and are removed with it; their aggregate report remains.

Rust setup runs the pinned Cargo's `vendor --locked --offline` against
`rust/Cargo.lock`, using the operator's default Cargo cache. It verifies the
lockfile is unchanged and publishes the vendor directory atomically. If the
locked crates are missing, the one-time network setup remedy is:

```sh
cargo +1.96.0 fetch --locked --manifest-path trials/e1-checked/rust/Cargo.toml
```

Tide setup likewise vendors the root `Cargo.lock` offline into `target/e1-checked/tide-vendor-<lock hash prefix>` and builds with source replacement; if missing, run `cargo +nightly-2026-08-09 fetch --locked` once from the repository root.

The dependency identity fx-ic2 records under protocol §5 is the full SHA-256
of `rust/Cargo.lock`; its first 16 hex characters select the generated cache.
Cargo verifies vendored files on locked builds. No third-party source is
committed here, and registry/cache paths are never mounted into the jail.

`prepare` creates a new directory, empty source, rendered prompt and arm
package files; only Tide gets byte-identical reference/companion copies.
It creates no `check` or `test` file or executable in the writer directory.
There are no grants. Writers put all code and tests in the one source file.
For real writer sessions, the operator must bind the allowed logical commands
`./check` and `./test` directly to the trusted
`python3 harness.py command SESSION check|test` dispatcher. **Do not execute a
file from a writer-editable directory outside isolation.** Reference runs
also invoke the trusted dispatcher directly, from the operator state directory. The
agent-tool broker that enforces these bindings and file access belongs to
fx-ic2's session runner, not this preparation CLI.

The operator owns the manifest, compiler, flags, vendored source and output
paths; candidate Cargo configuration, manifests, toolchain files and environment
are not used by builds. Each dispatched command stores the source before checking it,
counts both successful and failed calls toward 20, and refuses calls after
40 minutes from preparation. Missing, unreadable, escaping or non-UTF-8 input
is a counted failure with a snapshot/result, so a repaired next attempt can run.
First-check grading preserves an input failure rather than compiling an empty
replacement. `./check` compiles source and writer tests;
`./test` repeats that check then runs writer tests. A test has 10 seconds;
a complete command has 120 seconds, bounded also by the session deadline.
Timeouts kill the child process group. Children get closed unused stdin and
a clean environment. Native test processes must write a completed libtest
log record, so a silent early exit 0 fails.

The grader checks first-snapshot library acceptance compilation separately,
without running tests. Final grading runs writer tests, the acceptance suite,
and the task's literal structure searches. Writer tests and held-out acceptance
use separate packages built from the same captured candidate bytes. Writer
packages contain no acceptance source, and writer-created files are not copied
to the acceptance package. Rust acceptance selects only the fixed integration
target and fails if it produces no matching test executable.

Acceptance test names live in harness.py's `ACCEPTANCE_TESTS`, outside writer
directories, and freeze with the suites. Both arms must list exactly those
names. Every expected test runs with `--logfile` at a fresh path the harness
chooses inside its execution directory. After exit, the harness reads the file
through its original open handle and records its contents. A pass requires
exit 0 and a log containing exactly `ok <expected name>` followed by a newline.
Writer tests may instead log
`ignored <name>` or `ignored: <reason> <name>`; acceptance tests may not.
Stdout/stderr are retained for diagnostics and do not determine per-test
outcomes, so uncaptured output may interleave with libtest's console output.
Both pinned toolchains support this stable option and emit a deprecation
warning. Revalidate it when changing either pin.

Candidate code shares the acceptance process and can discover and forge the
log file. It is not an authenticated result channel. Final grading applies
identical source searches
in both arms for `process::exit`/`abort`, unsafe attributes, `link_section`,
`export_name`, `no_mangle`, `ctor` and libtest's `test result` text. Matches,
including comments and string literals, produce a `stage: integrity` result
with `review_required: true` and the matched categories. Its code 1 withholds
a mechanical pass pending grader review; it does not establish misconduct.
The searches are not a Rust parser and can miss aliases, generated code or
constructed output. They do not authenticate in-process results. Rust allows
termination and pre-main hooks that Tide's admission rules/grants reject;
fx-ic2 must account for that remaining asymmetry in the per-trial review.

Libtest executable execution mounts only a fresh scratch tree writable and the
selected executable read-only, alongside toolchain/system inputs. Package
sources and compiler output trees are not exposed to those test processes.
Writer doctests run in their writable writer package/output tree, which holds
no acceptance input. Binary cases receive only their fixture directory and
read-only executable.

Mentions in comments intentionally
count under protocol §6; a per-writer reviewer must record such cases and
confirm concurrency for C1–C6/C9. Structure searches cannot establish that work
actually runs in parallel. A `grade` exit 0 is a mechanical verdict; fx-ic2
must combine it with the recorded per-writer concurrency review and resolve
any integrity flags. No reference solution relies on a comment to meet
its structure requirement.

## Isolation and current boundaries

Normal writer commands and grading require Bubblewrap. Compilation mounts a
fresh source copy/output tree writable. Every jail mounts the selected Rust toolchain,
vendored crates, `/usr`, `/etc/alternatives`, `/nix/store` when present,
the small trusted doctest wrapper, and (for Tide) trusted built artifacts
readable. These mounts are read-only. The host's `/bin`,
`/lib` and `/lib64` symlinks are recreated; real directories are bound
read-only instead. It unshares namespaces, including networking (isolated
loopback), clears inherited environment, and never mounts the repository root,
other sessions or the repository's suite/reference directories. Setup failure is fatal; there
is no automatic unsandboxed fallback. The backend uses
[Bubblewrap's namespace and bind-mount interface](https://github.com/containers/bubblewrap/blob/main/README.md).
The system/store trees preserve compiler helpers, wrapper scripts and linker
symlink chains. Debian's `cc` resolves through `/etc/alternatives`. The jail does
not bind `/etc` itself, `/home`, the repository root or host `/tmp`.
Boundary tests cover the
mount policy, merged-usr and real-directory hosts, and an absent Nix store.
The real self-test requires jailed Cargo exec, rustc link and execution of
the resulting program; namespace failure fails rather than silently skipping.

`--local` exists only on the reference-readiness command. It runs these known
reference files outside the namespace sandbox, using a shared *worktree-local*
Rust build cache. Its results prove only the recorded source checks/cases;
they are neither an isolation proof nor a gate run. Never use that mode for
writer code. Ordinary writer builds keep their cache inside their fresh tree.

Tide uses the pinned driver and production std. Metadata checks, linked
writer tests, linked acceptance tests and binary builds each have a separate
output directory. `--emit=link --test` builds `solution-test` for writer tests.
The L stage uses `--tide-test-library` to admit the library and compile the
acceptance suite as a second crate that imports its public API. First-check
grading uses metadata mode. Full grading builds `acceptance-test` and runs
each listed test with the same time and result checks as Rust. Binary cases
run the driver's linked `solution` through the shared process-case loop.
Writer tests run plain, property, simulation and generated contract entries,
with the vacuity floor and visible allowed-vacuity reports. Acceptance listings
exclude the exact reserved `__tide_contract_tests` module segment; writer listings include it.

This is not the agent-session runner: `fx-ic2` must restrict writer tools and
reads, terminate the agent session at 40 minutes even when no command is run,
fix/shuffle the 120 slots, record model/harness/toolchain identities and tokens,
perform each final concurrency review and compute the paired measures.

The doctest adapter depends on the pinned compiler's internal merged-runner
[case selector](https://github.com/rust-lang/rust/blob/1.96.0/src/librustdoc/doctest/runner.rs)
and [bundle artifact](https://github.com/rust-lang/rust/blob/1.96.0/src/librustdoc/doctest.rs).
The self-check runs two six-second doctests (together longer than ten seconds)
plus a `standalone_crate` and a panic doctest. The self-test verifies all four
doctests pass and no unknown-attribute warning occurs. It also rejects a timed-out
`should_panic` doctest even when rustdoc reports the panic as expected. Revalidate this adapter when changing
the Rust-arm pin. No unstable Rust flags or source rewriting are used.
