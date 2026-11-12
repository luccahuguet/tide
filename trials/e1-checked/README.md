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

Exit 0 means the requested checks passed, 1 means failure, 2 is setup
failure, and 3 is budget refusal. Reference runs write command/exit/output records to
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
owned `broker.py` exposes only argument-free `check` and `test` tools;
`coordinator.py` configures the writer and verifies its tool surface.

The operator owns the manifest, compiler, flags, vendored source and output
paths; candidate Cargo configuration, manifests, toolchain files and environment
are not used by builds. Each dispatched command stores the source before checking it,
counts both successful and failed calls toward 20, and refuses calls after
40 minutes from writer launch in coordinated sessions. Standalone `prepare`
starts a provisional clock; the coordinator resets it under the session lock
immediately before launching the writer, recording wall and monotonic time. Missing, unreadable, escaping or non-UTF-8 input
is a counted failure with a snapshot/result, so a repaired next attempt can run.
First-check grading preserves an input failure rather than compiling an empty
replacement. `./check` compiles source and writer tests;
`./test` repeats that check then runs writer tests. A test has 10 seconds;
a complete command has 120 seconds, bounded also by the session deadline.
Timeouts kill the child process group. Bubblewrap's separate JSON status FD
distinguishes setup failure from a candidate's exit or stderr. Cargo respects
the operator's `CARGO_BUILD_JOBS` (default 6). Children get closed unused stdin and
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

`coordinator.py` runs the agent sessions, records identities and tokens, and
ends a writer at 40 minutes even if it never calls a tool. `fx-ic2` still owns
the readiness evidence, suite freeze, tool-surface probe execution, final
concurrency/integrity reviews and paired measures.

The doctest adapter depends on the pinned compiler's internal merged-runner
[case selector](https://github.com/rust-lang/rust/blob/1.96.0/src/librustdoc/doctest/runner.rs)
and [bundle artifact](https://github.com/rust-lang/rust/blob/1.96.0/src/librustdoc/doctest.rs).
The self-check runs two six-second doctests (together longer than ten seconds)
plus a `standalone_crate` and a panic doctest. The self-test verifies all four
doctests pass and no unknown-attribute warning occurs. It also rejects a timed-out
`should_panic` doctest even when rustdoc reports the panic as expected. Revalidate this adapter when changing
the Rust-arm pin. No unstable Rust flags or source rewriting are used.


## Gate machinery (fx-rpwp)

`broker.py`, `coordinator.py`, `gate_checks.py` and `standin.py` use only
Python's standard library. No gate writer has been run by this implementation.
The following commands are for the gate coordinator, after its prerequisites
and machine scheduling checks. Compilation, canaries and dry runs require a
cleared machine slot. Do not use `--local` for any of them.

Choose a fresh absolute external run directory, for example
`RUN="$HOME/.cache/tide-e1/gate-001"`. Raw prompts, logs, snapshots and grading
outputs stay there. Keep them out of agent context. Only opaque slot numbers
appear in writer directory names. A real run requires a clean tree in crates,
trial machinery, toolchain/lockfiles and the two reference documents.

```sh
# Builds/records both arm configurations. Pin the CLI binary and its version;
# --claude takes an absolute executable path. This launches no writer.
python3 trials/e1-checked/coordinator.py init "$RUN" --claude "$(command -v claude)" --jobs 1
# state.json contains all 120 triples, their SHA-256 and the seed. Record the
# full order and hash on fx-ic2 before running any gate slot.
python3 trials/e1-checked/harness.py canary rust "$RUN/rust-canary.json"
python3 trials/e1-checked/harness.py canary tide "$RUN/tide-canary.json"
python3 trials/e1-checked/coordinator.py store-scan "$RUN"
# This is a REAL, non-task Claude session. Only the coordinator runs it,
# before the gate. Supply ANTHROPIC_API_KEY or CLAUDE_CODE_OAUTH_TOKEN in the
# environment; credentials are never copied into the identity or logs.
python3 trials/e1-checked/coordinator.py probe "$RUN"
# After fx-ic2 records readiness, freeze, identities, probe and order:
python3 trials/e1-checked/coordinator.py run "$RUN" --batch 4
```

The writer gets the pinned CLI in print/restricted/safe mode, exactly
Read/Write/Edit plus the strict e1 MCP server, dontAsk permissions, no slash
commands or session persistence, a fresh config home, no effort override and
`DISABLE_AUTOUPDATER=1`. The init line must report precisely those five tools,
only e1 connected, no plugins and the recorded model. Every tool use and file
path is audited after exit, as are assistant model identifiers. The probe
requires successful inside read/write controls and complete, valid outside
requests. Only matching pinned-CLI `system/permission_denied` frames with the
restricted-directory reason and matching error results establish refusal;
validation errors, missing fields and unrelated tool errors fail the probe.
Repository/system reads and outside writes must all be refused; the exact init surface establishes shell and web
tools are unavailable. A failed probe blocks the run; an outer writer jail is
an unimplemented fallback requiring another change.

A writer's structured denied outside request is recorded as information, not
a fault or a fresh writer draw. An outside access without a matching denial
stops the whole run for the operator. The rule for a tool outside the allowed
set remains an infrastructure fault. A budget/time kill may drop one
unterminated final JSON log line, recording `truncated_tail`; other malformed
lines remain faults. Normal CLI completion lets its broker close for up to
three seconds before the guard kills remaining group members.

The broker forwards stdout then stderr and an `exit: N` line, and logs mode,
exit, elapsed seconds and truncation to `broker.jsonl`, with lifecycle records
that detect an unexpectedly lost broker. Its fixed 1,000,000-byte
cap applies identically to both arms and every truncation is counted. The
CLI transport ceiling is raised to 2,000,000 tokens so it sits above the byte
cap rather than silently truncating diagnostics first. The harness alone enforces 20 runs and 2,400
seconds, using a monotonic clock in coordinated sessions. At the end of the
last permitted command it captures the final source and publishes an atomic
`budget.json` marker. The coordinator waits for the broker to record that
command's completion, then kills the writer group. Edits after
exhaustion cannot replace that saved final; the last command results remain
on disk. Broker children
have closed stdin, a separate process group and a 150-second guard.

The closed fault list is init failure, CLI error/nonzero, broker crash,
harness setup/bwrap failure, host interruption and malformed/forbidden-tool logs. A slot
gets one immediate rerun in a fresh directory. Both attempts remain recorded;
writer faults prevent grading, and two genuine writer-session faults fail both
measures. Cancellation starts no further attempt. The attempt placeholder is
written only after a guard is granted; partial preparation and an ungranted
guard are never counted as faults. Budget
refusal, the time-limit kill, early completion and wrong/empty output do not
cause a rerun. A stopped coordinator resumes an unfinished attempt as a host
interruption. A Linux parent-death guard terminates the writer process group
if its coordinator dies. The writer starts only after its PID, process-group
ID, boot ID and process start time have been saved. Recovery sends no signals:
it waits up to ten seconds for the original identity/group to have no live
non-zombie members, then waits for any locked command before rerunning.
A missing PID or a start/boot mismatch proves the old group is gone: Linux
keeps a PGID number allocated while any member lives. An ungranted guard is
not a started attempt. Only a matching live group that outlasts the wait
blocks recovery. Active
cancellation signals only a guard through its pidfd; signalling and handle
close share a lock, so concurrent reaping cannot redirect a signal. Graders
use the same guard. The coordinator never signals a numeric PID or PGID.
Shutdown cancels guards before waiting for workers.
`--jobs` defaults to one, is fixed at init, and is recorded;
parallel launches preserve slot order. Gate operators must still avoid the
shared-I/O exposure described on fx-ic2.

Before grading, the coordinator waits on the command lock, stores
`final/source.rs` and its SHA-256, and removes write permissions from the
writer directory. `harness.py grade` uses this saved final thereafter. Both
`grade SESSION --first --compile-only` and `grade SESSION` write records, even
when the generic dry-run program fails. `trial.json` records the end reason,
fault evidence, counts, elapsed time, token categories, peak context, API retry
events, denied-outside-request counts by arm, truncated-tail status, reference
Read offsets/sections, model identifiers, grades and integrity flags. Snapshot
results retain per-command diagnostics and test results. The slot record
counts reruns and fails both measures after a second infrastructure fault.
The ended writer result and final hash are saved before grading. Grader setup
flags, unexpected exits, missing/malformed records and the outer guard timeout
retry the frozen snapshot, with at most two retries across resumes. They never
launch another writer. Exhaustion stops the run for the operator, without
scoring a solution failure. `grading.json` tracks attempts and `grading-N/`
retains their output. A resumed session with a frozen final reuses that writer.
If its writer result was not saved, resume applies the same complete session
classification as a live writer before grading. A final hash that differs from
the saved writer result stops the run for the operator without rewriting it.
The harness gives each grade an aggregate 480-second deadline below the
600-second hang guard; exhausting the aggregate deadline is a solution timeout
with grade exit 1, not infrastructure.
Integrity and concurrency still require fx-ic2's human/agent grading review.

Canaries use a Python probe from `/usr` through the same `run`/jail builder,
in compile, test-runtime and binary-case shapes in both arms. They only open
held-out paths, never read their contents. Inside file/loopback controls must
work; repository, design, suite, reference, task, other-session, host-loopback,
non-loopback, DNS and inherited-environment probes must fail. The unsandboxed
negative control must reach each path/listener and environment sentinel; no
probe is skipped. Every jail mounts tmpfs `/tmp` before binding its read-only inputs, so
executables beneath an external `/tmp` run directory remain visible.
`TMPDIR=/tmp` applies to all shapes, including
writer doctests and both arms' B9-shaped runtime fixtures. The Rust canary
checks the mount type and creates a fixture directory on it.

Gate runs require both arm canaries for the current machinery hashes,
Bubblewrap version and boot ID, plus a matching writer probe and a successful
pre-run store scan. The scan sizes every regular store file, hashing only size
matches against tracked trial/design files; any identical content fails.
Symlinked store files resolve to other immutable store paths and are visited
at their regular-file location. A complete run automatically writes and checks
`store-after.json`. Store errors are fatal. The store-bind experiment is:

```sh
E1_WITHOUT_STORE_TEST=1 python3 trials/e1-checked/harness_test.py HarnessTests.test_jailed_toolchain_executes_and_links
```

It affects this test only; runtime mounts retain the approved store bind.
`harness.py identity OUTPUT --claude /absolute/claude --jobs 1` writes a
standalone identity. Run state includes toolchain versions, crates tree,
lockfile/document/driver/std/machinery hashes, measured suite-tree SHA-256,
kernel, Python, CPU count, jobs and CLI argv template. The suite-tree measure
is not a freeze. Before each writer the clean tree, HEAD, driver/std hashes,
CLI path/hash/version and model must still match; a change stops the run.

For a task-blind dry run in a separate fresh directory:

```sh
DRY="$HOME/.cache/tide-e1/dry-001"
python3 trials/e1-checked/coordinator.py init "$DRY" --dry-run
python3 trials/e1-checked/coordinator.py run "$DRY" --batch 4
python3 trials/e1-checked/coordinator.py dry-clock "$DRY"
```

The four dry slots are A2/A3 in each arm. The harness renders real prompts,
but the stand-in never reads them or any reference solution. It writes only
generic controls, calls check/test through the same broker config, and is terminated after call 20. Its trusted post-session controller calls
the same broker against that exhausted session and asserts call 21 returns 3. Both grades are produced; these controls are expected
to fail grading. The separate clock exercise shortens the limit to one second,
kills a sleeping stand-in, then verifies a broker call is refused with 3.
Only `--dry-run` permits `--seconds` to shorten the 40-minute session limit.
All dry state, slot, attempt, launch and clock records are marked `dry_run` and
are excluded from gate reporting; there is no automatic gate-score publisher.
