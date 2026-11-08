# Tide review, fx-o8o round 2 (Opus): confirmation round

Candidate: bf712d8 on branch `fx-o8o` (round-1 candidate ff2b251, base 3fdc020). I reviewed `git diff ff2b251 bf712d8` (11 files under `trials/e1-checked/`) and harness.py as it stands at bf712d8. The worktree's tracked tree was not modified, and nothing heavy was run. Host results come from the orchestrator: harness_test.py 14/14, Rust references sandboxed 30/30, Tide references sandboxed and `--local` 30/30 failing as mapped on the bead. `git diff --check ff2b251 bf712d8` passes.

Model: claude-opus-5-5 · Date: 2026-11-08

## Verdict

request_changes. Eight of the nine round-1 blockers are fixed. Codex Blocking 1 is fixed for files but not for result parsing. Two findings block:
- B1 is not text-only: the acceptance verdict is still parsed from output that candidate code writes.
- B2 is text-only: readiness.md is stale against the bf712d8 host proof.

## Round-1 findings

- **Opus B1 / Codex 2 (B10 case 3 timing): fixed.**
  - `suites/rust/B10.rs:25-29` checks `(calls, last_ok)` and `elapsed < 150 ms`, which is the task's slack.
  - The Tide suite still asserts exact virtual zero, and its behaviour is unchanged.
  - The Rust reference drops its call-0 special case, so the reference itself proves the slack.
  - The `reference_test.rs` wrapper adds 1 ns to every report and still passes the suite.
- **Opus B2 (A4 case 3 owned operators): fixed.**
  - `suites/common/A4.rs:14-18` builds `(x.clone() + one.clone()) * (x - one)`, which uses owned `+`, `-` and `*`.
  - The suite is shared by both arms, and the common reference runs all five cases in `reference_test.rs`.
- **Opus B3 / Codex 3 (bricked session): fixed.**
  - `command()` persists `runs` before `snapshot.mkdir()` (`harness.py:369-372`), so a crash at any later point cannot collide with the next attempt's snapshot.
  - Every input fault is caught and recorded as a code-1 `stage: "candidate"` result. That covers `FileNotFoundError`, `IsADirectoryError`, ELOOP/`RuntimeError`, `relative_to`'s `ValueError` and `UnicodeDecodeError` (also a `ValueError`). The raw or empty bytes are snapshotted.
  - `grade --first` keeps the candidate failure instead of compiling an empty file (`harness.py:475-486`).
  - The test covers five broken forms, two failures followed by a repair, and a recorded setup error.
- **Opus B4 (unbounded disk): fixed.**
  - `run/` is removed in `finally` after `results.json` is written (`harness.py:393-396`).
  - Reference sessions live under the batch's `TemporaryDirectory` (`harness.py:343`).
  - Per-test scratch is a `TemporaryDirectory`. The native-wrapper test now asserts through results.
  - Within one batch, grade trees still accumulate until the batch ends. That is bounded and acceptable.
- **Opus B5 (stale and narrative docs): fixed.**
  - The operator paths, heavy.sh policy, session history, absolute proof links and false "no bead"/"no commit" claims are gone.
  - A new staleness, created by the later host run, is B2 below.
- **Codex 4 (B9 CR loss): fixed.**
  - Both references split on `'\n'` with `split_terminator`.
  - `reference_test.rs:26-44` round-trips `"x\r"`, `"\r\t\r"` and an empty key and value, outside the held-out suite.
- **Codex 1 (writer tests replace the suite): fixed for writer-controlled files. Open for result parsing; see B1.**

  I checked every channel the brief lists:
  - **Tests, build.rs, Cargo.toml, .cargo/config, toolchain files: closed.**
    - `package()` copies only the captured entry bytes. The manifest, lock and `.cargo/config.toml` come from the operator.
    - Writer and acceptance packages are separate trees (`harness.py:403-415`). The acceptance package is never written by writer code.
  - **Module paths, `include!`: closed for writes.**
    - A candidate `#[path]`/`include!` can read files inside the jailed acceptance tree at compile time, but it cannot change the integration target.
    - Rust acceptance selects only the target named `acceptance`, of kind `["test"]`, at the operator's `src_path`. It fails closed when there is none (`harness.py:253-258`).
  - **Symlinks: closed at the package level.** `candidate()` resolves the path and confines it to the directory, and the package receives bytes, not links. A TOCTOU remains; see N5.
  - **Target-dir reuse: closed for writers.**
    - Sandboxed builds use `work/build` in a fresh tree.
    - The shared `STATE/rust-build` exists only when `sandbox` is false. That happens only through `references --local`, because `prepare` always records `sandbox: true` and `session.json` lives outside the writer directory.
  - **Env vars: closed.** Every jailed child gets `environment()` or the case's env only.
  - **Runtime writes: closed.**
    - `test_command` mounts only a fresh scratch tree writable and the test executable read-only (`harness.py:276-285`).
    - Binary cases get only their case directory and the read-only binary.
    - Writer doctests can write the writer tree, but that tree holds no acceptance input.
  - **Result parsing: open (B1).** The verdict is still derived from the test process's stdout and exit status, and candidate code runs inside that process.
  - **Self-test: discriminating for the file channel.**
    - `test_jailed_grading_rejects_replaced_or_shadowed_suite` runs a real jailed grade. A writer test writes a shadow suite and a correct candidate into `package/` relative to its working directory.
    - On ff2b251 that working directory was the shared grade tree, so the test would have failed there.
    - On bf712d8 the wrong candidate must still fail the genuine suite.
    - Two mocked tests cover the shadow-target selection and the absence of acceptance source in the writer package.
    - None of these tests covers the parsing channel, and none runs on the Tide arm. Tide acceptance is blocked on fx-dsa anyway.

## New findings

**B1 (not text-only, blocks). Acceptance pass/fail is parsed from candidate-writable output, and only the Rust arm can write it.**

Where: `harness.py:278-290` (`test_command`).
- A test counts as passed when its exit code is 0 and its stdout contains `test result: ok. 1 passed; 0 failed;`.
- The listing that decides which tests run is parsed from the same process's stdout.

Scenario:
- A Rust library function called by an acceptance test writes libtest's success summary to stdout and ends the process with status 0, before the test's assertions run. The harness records that test as passed.
- With a life-before-main static, which edition 2024 allows in safe code via `#[unsafe(link_section)]` and needs no dependency, candidate code can also rewrite the `--list` output. The harness then requires only one listed test.

Impact:
- Either route makes the suite's verdict something the candidate chooses. That is the same integrity failure Codex Blocking 1 named, by a channel the file isolation cannot reach.
- The Tide arm cannot do either, because `process::exit` is FX0501 and `#[unsafe(..)]` needs the `unsafe` grant, which `grants=[]` withholds (reference.md:551, :954). So this is an asymmetry favouring Rust that §11 does not list.
- I do not expect writers to find it unprompted. They see neither harness.py nor the suites. But the grade is mechanical (§6), and nothing would catch it.

No stdout or exit-code protocol can be made unforgeable while candidate code shares the test process. Two cheap mechanical steps close the selection channel and make the forgery visible:
1. Freeze each task's and arm's expected acceptance test list with the suites, taken from the reference run's `--list`. Fail when a listing differs from it. Candidate code then cannot choose which tests run without also forging every one of them.
2. In `grade`, record a literal-search integrity flag over the final snapshot and route flagged trials to the per-trial grader review that §6 already requires. Search for:
   - process termination (`process::exit`, `process::abort`);
   - unsafe attributes (`#[unsafe(`, `link_section`, `export_name`, `no_mangle`);
   - the libtest summary text (`test result`).

   Record this limit and the Rust-only asymmetry on fx-ic2, as the bead's Scope requires for protocol-level gaps.

Add one self-test: a Rust candidate that prints the summary and exits 0 must yield a failing or flagged grade.

**B2 (text-only, blocks). readiness.md records only the ff2b251 proof and calls the bf712d8 proof outstanding.**

Where:
- `readiness.md:3-11` says "Review baseline: ff2b251 … require fresh host proof" and gives 10/10 harness tests.
- `readiness.md:134` says "Remaining work is fresh host validation of the round-1 corrections".

The orchestrator has since run bf712d8 on the host: 14/14, Rust 30/30, Tide 30/30 failing as mapped. AP-PROOF-001 needs the proof recorded against the revision that lands.

Fix:
- Record bf712d8's results, or the final candidate's, with the revision and the Tide artifact hashes.
- Drop the "requires fresh host proof" and "Remaining work is fresh host validation" sentences.
- If B1's fix changes harness.py, record the proof of the final candidate instead.

**N1 (nit). A writer doctest can turn `./test` into a harness crash.**

Where: `harness.py:395-396`.
- `shutil.rmtree(work)` has no error handler.
- Writer doctests run with the whole `snapshot-NN/run` tree writable (`harness.py:262-266`). A doctest that leaves a directory at mode 000 makes the rmtree raise `PermissionError` from the `finally`. `main` then prints "E1 setup error" and exits 2.
- The run is counted and `results.json` is written, so the session survives. But the writer's own output is not printed, `run/` leaks, and §3 would read a writer-caused exit as an infrastructure fault.

Fix: pass an `onexc` handler that restores owner permissions and retries, as `TemporaryDirectory` does.

**N2 (nit, text-only). README over-promises for doctests.**

Where: `README.md:109-111`.
- The README says "Package sources and compiler output trees are not exposed to the test process."
- That holds for `test_command`, but writer doctests run inside the writer package's work tree, writable.

Fix: say that libtest executables run in fresh scratch, and that doctests run in the writer package's tree, which holds no acceptance input.

**N3 (nit). One assertion in the jailed self-test is vacuous.**

Where: `harness_test.py:179`, `assert!(!source.join("tests/acceptance.rs").exists())`.
- `CARGO_MANIFEST_DIR` is not mounted in the test jail, so the check is true whatever the writer package contains.
- The mocked test already covers what it was meant to prove. Drop it, or move the check to the mocked test.

**N4 (nit). The B9 round-trip test can fail on the leftovers of an earlier failed run.**

Where: `reference_test.rs:28-29`.
- It uses the host `temp_dir()` plus a PID and `create_dir`, so a leftover from a failed run with a reused PID fails the next run spuriously.

Fix: call `remove_dir_all` first and ignore NotFound, or use a directory under the caller's `target/` tempdir.

**N5 (nit, pre-existing). `candidate()` checks one path and reads another, and a FIFO hangs the command.**

Where: `harness.py:350-354`.
- `candidate()` resolves and confines the path, then reads the unresolved path. A concurrent swap between those two steps can read outside the writer directory.
- If `src/lib.rs` is a FIFO, the command blocks indefinitely while holding the session lock.
- fx-ic2's broker limits the writer's tools, so this cannot fire today.

Fix: open the resolved path with `O_NOFOLLOW | O_NONBLOCK`, require `S_ISREG` from `fstat`, and read from that descriptor.

**Arm symmetry otherwise holds.**
- The new grading layout, the cleanup and the input-failure handling are identical for both arms.
- The B10 change applies the slack the task text gives each arm. The A4 change is shared.
- The only new asymmetry found is B1's.
