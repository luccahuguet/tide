# Tide review, fx-o8o round 4 (Opus): confirmation round

Candidate: 869766d on branch `fx-o8o` (round-3 candidate a1f0b5c, base 3fdc020). I reviewed `git diff a1f0b5c 869766d` (4 files under `trials/e1-checked/`), `test_command` as it stands at 869766d, libtest's logfile code on both pins, the B9 suite, fixtures and reference, CODEX-REPORT.md, and fx-ic2's two comments. The worktree's tracked tree was not modified.

Host results come from the orchestrator (readiness.md): harness_test.py 18/18; Rust references 29/30 in the full run, with B9 case1 timing out once under load, then B9 alone 3/3, one sweep-caused build failure, then 5/5; Tide 30/30 FAIL as mapped; A2/A3/A4/A10 passed 7/5/5/4 acceptance tests through logfiles; no integrity flag.

My own checks, all in the scratchpad:
- I read `library/test/src/console.rs` and `cli.rs` in both pinned sysroots (1.96.0 and nightly-2026-08-09). `write_log_result` is byte-identical in the two. A record is `ok|failed[: msg]|ignored[: msg] NAME`, then ` <time>` only with report-time (unstable), then `\n`. The file is opened once, before any test runs. `--logfile` prints `warning: \`--logfile\` is deprecated` to stderr with no trailing newline.
- I built a 13-test probe with `rustup run 1.96.0 rustc --edition=2024 --test` and ran it through the 869766d `test_command` (not sandboxed), once as writer tests and once as acceptance with the frozen list patched to the probe's names. Results:

  | Probe test | Writer | Acceptance |
  | --- | --- | --- |
  | plain pass | pass | pass |
  | `stdout().lock()` writeln | pass | pass |
  | spawned thread writing `ok\ntest result: FAILED.` to raw stdout | pass | pass |
  | `#[should_panic(expected)]` that matches | pass | pass |
  | `#[should_panic(expected)]` with the wrong message | fail (101) | fail (101) |
  | failed assert | fail (101) | fail (101) |
  | `#[ignore = "why not"]`, plain `#[ignore]` | pass | fail (code 1) |
  | silent `process::exit(0)` | fail (code 1, empty log) | fail (code 1, empty log) |
  | candidate writes `ok NAME` to the logfile path and panics | fail (101, log `failed NAME`) | fail (101) |
  | candidate unlinks and recreates the logfile path, then passes | pass (host read the original inode) | pass |
  | candidate writes `ok NAME` to the logfile path, then `process::exit(0)` | pass (the documented forgery) | pass |
  | `read_dir(".")` in the execution directory | pass | pass |

- I ran the three changed self-tests (`test_uncaptured_stdout_does_not_change_test_verdict`, `test_acceptance_requires_frozen_names_and_consistent_results`, `test_integrity_flags_require_review_for_both_arms`): 3/3 OK.
- I confirmed that `git diff 31a9800 869766d` touches only readiness.md. harness.py (blob e995b7e) and harness_test.py (blob 5c45800) are identical at 31a9800 and 869766d.

Model: claude-opus-5-5 · Date: 2026-11-08

## Verdict

approve. Round-3 B1 and B2 are fixed. No new finding blocks. N1 (deprecation-warning noise in writer output) is a cheap fix I recommend before the harness freezes. The B9 timing exposure only affects the Rust arm, and fx-ic2's constraint comment should say so (see question 3). That is a planning note, not a blocker for this bead.

## Round-3 findings

- **B1 (uncaptured stdout false-fails Tide tests): fixed.**
  - **Mechanism.** Each test now gets a fresh `--logfile` inside its execution directory, and the host keeps its own handle open on it. The test passes only when the bytes equal `ok <name>\n` and the exit code is 0. Writer tests may also log `ignored[: reason] <name>\n`. Stdout no longer affects the verdict, so interleaved output cannot cause a false failure. The real-binary probes above confirm this, including a raw-stdout thread printing a fake `test result: FAILED.`.
  - **False passes: none, apart from the documented in-process forgery.**
    - Empty log (silent exit 0): fails.
    - Extra or duplicate lines, a missing newline, a wrong name, `failed` or `failed: msg`: all fail, because the comparison is exact equality. A `failed: msg` record cannot be made to equal `ok NAME\n`.
    - Missing file: the host creates the file and reads through its own handle, so the path cannot go missing from the host's view. Unlinking or replacing the path does not redirect the read (the "replaced" self-test fault and my unlink probe).
    - A forged `ok NAME` followed by a panic: libtest's own handle is still at offset 0 and overwrites the forgery with `failed NAME`, and the exit is 101 anyway.
    - `should_panic`: a matching panic logs `ok`, a non-matching one logs `failed: …`. Both are correct.
    - Ignored acceptance tests fail. Writer ignores with or without a reason pass.
    - A forgery needs the process to end with code 0 before libtest writes, so `process::exit` stays the relevant flag.
  - **False failures: none found in either arm.**
    - The log names the full test path, so the Tide `acceptance::` prefix and module-nested writer tests match their `--list` names.
    - Libtest's logging code is identical on both pins.
    - Host Tide acceptance (21 cases) graded through logfiles.
    - The only theoretical false failure is in N4.
  - **The new self-tests prove what they claim.**
    - The real-binary test proves that output from a test with an uncaptured stdout interleaves (it asserts `test case1 ... uncaptured output\nok`) and that the test still passes, and that `should_panic` passes. Codex reports it fails on a1f0b5c.
    - The mocked matrix covers both arms: missing, wrong name, duplicate, extra, malformed, failed, ignored, a non-zero exit, a replaced path (proving the read goes through the original handle), and unique per-test paths.
    - The forged-transcript regression now also asserts an empty `test_log`.
    - Small redundancy: "summary-only" and "missing" are now the same fault (N5).
- **B2 (readiness.md calls proof outstanding): fixed.**
  - The text names the revision it proved (31a9800). It says that the later commit differs only in readiness.md, which I verified (the harness blobs are identical). It drops both "proof still owed" sentences.
  - It is honest about B9. It reports 29/30 in the full run instead of rounding up, names the cause and the per-test limit, and reports the 3/3 isolated runs, the sweep-caused failure and the 5/5 after the sweep.
  - Taken together, every Rust reference has a passing run on this code: 29 in the full run and B9 in isolation.
  - N3 has two small text gaps.

## Questions from the brief

**3. Can a correct solution miss the 10-second limit, and is recording it on fx-ic2 enough?**

Under quiet conditions, no:
- B9 case1 makes 2 saves, each with a file `sync_all` and a directory `sync_all`. On an idle ext4 disk that takes milliseconds.
- No B9 test makes more than about 4 durable writes. A writer who fsyncs more often still stays far below 10 s.

Under heavy shared write-back, yes. An ext4 fsync can stall for seconds behind journal commits, which is what the full run hit.

Two facts the fx-ic2 comment omits:
- **The exposure is Rust-only.** Rust file suites use real temporary directories (`suites/rust/fs.rs`, `TMPDIR` = the execution directory under `target/e1-checked`, on ext4). Tide file suites use `Root::sim` (`suites/tide/fs.rs`), as protocol §6 requires. A load-induced timeout therefore lowers only the Rust baseline. In final grading that moves "final pass >= Rust" in Tide's favor. In writer `./test` runs it gives Rust writers false failure feedback.
- **The harness has a cheap mitigation that stays inside the protocol.** It could run Rust test executions with `TMPDIR` on tmpfs (`/dev/shm` is tmpfs here), for example a bwrap `--tmpfs` at a fixed path, kept separate from the logfile's directory. §6 asks only for "temporary directories". B9's durability is checked structurally (`sync_all(`), never behaviorally, so no verdict would change except the timing exposure.

Recording the constraint on fx-ic2 is enough for fx-o8o. Gate operations and the pre-session freeze belong to fx-ic2, and this run is not a gate run. I recommend appending to the fx-ic2 comment the direction of the bias (Rust-only, favors Tide), that it covers writer sessions as well as grading, and the tmpfs option, so fx-ic2 can choose between enforcing the constraint and removing the exposure. The fix would not change the 10 s limit or any protocol threshold.

**4. Leftovers, regressions, arm asymmetry.** The listing, per-test and flag rules are identical in both arms, and the logfile mechanism adds no asymmetry. The B9 exposure above comes from §6's fixtures, not from this diff. The leftovers and regressions are N1 and N2.

## New findings

**N1 (nit, recommended before freeze). Every test now adds a deprecation warning, without a newline, to the stderr that writers see.**

Where: `harness.py` `test_command`, whose per-test results `command()` echoes to the writer.

Libtest writes `warning: \`--logfile\` is deprecated` with no newline, once per test. My probe's writer stderr was one line: `warning: \`--logfile\` is deprecatedwarning: \`--logfile\` is deprecated…`, followed by `missing or inconsistent libtest log record`.
- Writers in both arms see, on every `./test`, a warning about an option they never passed.
- Uncaptured stderr from a test (Tide's `eprintln!`) is glued onto that line.

The noise is the same in both arms, so it biases nothing, but it is new writer-visible noise in an experiment that measures iterations.

Fix: strip that one exact prefix from each per-test `stderr`. `observed_exit` and `test_log` already keep the facts needed for diagnosis.

**N2 (nit). The `test result` integrity search no longer guards anything.**

Stdout no longer affects any verdict, so a printed transcript cannot forge a result. The remaining forgery writes the logfile and then exits, and `process::exit`/`abort` still flag that. The `test result` pattern now produces only false positives, and fx-ic2's 04:03 comment still says "printed result lines". The decision kept the flags, so this does not block. Options:
- swap the pattern for `logfile`, which flags candidate code that names the log file (it cannot hit a correct solution);
- or keep it and note in the README and the fx-ic2 comment that the forgery channel is now the log file.

**N3 (nit, text-only). Two gaps in readiness.md.**
- **The proof's identity will not survive the squash.** Both 31a9800 and 869766d become unreachable after "squash before landing". Recording the harness.py and harness_test.py blob ids (e995b7e, 5c45800) would keep the proof's identity checkable after landing.
- **Artifact hashes are silently dropped.** r3 B2 asked for hashes "where available". Either give the 31a9800 results.json hashes or say they were not kept.

**N4 (nit). A writer ignore reason that contains a newline false-fails.**

The regex `ignored(?:: [^\r\n]*)? NAME\n` rejects `#[ignore = "a\nb"]`, which libtest logs verbatim. This is rare and affects writer tests only. `re.DOTALL` with `.*` would fix it.

**N5 (nit). Two remaining notes on the self-tests and README.**
- **Duplicate fault.** "summary-only" and "missing" now inject the same fault. Drop one.
- **Revalidation trigger.** README says to revalidate `--logfile` when changing either pin. It should also say to revalidate when Tide's test runner changes (fx-nkv.6). A runner without libtest's CLI would fail loudly at fx-ic2's reference rerun, so this is wording only.

**Carried over, unchanged and still non-blocking:** r3 N1 (integrity flags on binary tasks), N2 (Tide listing will include writer tests, under fx-dsa) and N3 (`#[sim_test]` name expansion, at fx-ic2's freeze).
