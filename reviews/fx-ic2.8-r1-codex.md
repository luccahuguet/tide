Model: gpt-6.1-sol · Effort: high · Date: 2026-11-14
Verdict: request_changes

## Scope

Read-only supervisor review of `fx-ic2.8`, HEAD `029404327d2dc30da7ea1186406fd08e885a0e56`, tree `20799218846b7bf955dc355bfbf26ba9ce49c871`. Reviewed `git diff main...HEAD`; merge base is `cc3bb2d719beb400caf4e8467e168874cfceca0f`, and the diff SHA-256 is `2fa7364a16d425c4a6421adf11a36d5048972ed1a03390a27c04f0ff229fac63`. Main advanced during the review, without changing that merge base or diff. Candidate remained clean and HEAD unchanged.

Read AGENTS.md, GOVERNANCE.md, BR_WORKFLOW.md, both requested beads (including the parent’s fx-rpwp r4 finding 5), the author’s rounds 1 and 2, protocol.md, the README changes, changed harness code/tests and connected callers. No held-out files were opened or searched manually. Synthetic controls and harness tests were used without displaying held-out content. No candidate edits, commits or br writes.

Contract reviewed: every writer build/test and grade uses the recorded closure, pinned jobs, private tmpfs and enforced process-tree memory limit; setup and host faults remain infrastructure, while a local cap violation fails the candidate. The store scan may skip genuinely inaccessible content but must not certify readable held-out content or hide other scan failures. The generated Tide import must work at the existing library boundary.

## Blocking

1. **[P1] Verify the memory controller before executing the jail.** `trials/e1-checked/jail_limits.py:24`, `:26`, `:27`, `:31`; `trials/e1-checked/harness.py:333`.

   The reporter finds its cgroup, immediately starts the unreviewed command and waits for it, and only then reads the controller files and effective limits. If the user manager lacks the memory controller, or the requested setting was not applied, the command can run uncapped for its entire timeout. Reporting infrastructure failure afterwards does not protect the shared machine. A successful systemd scope start is insufficient: systemd v255 logs controller-enable failures and continues, and applies resource attributes without making that application a fatal return from `unit_update_cgroup`. See [systemd v255 cgroup implementation](https://github.com/systemd/systemd/blob/v255/src/core/cgroup.c#L2296-L2333).

   Probe: `python3 codex-r1/probes.py`, `missing_memory_controller` in `codex-r1/probe-results.json`. With missing controller files simulated and process creation instrumented, `child_launched_before_validation` is true; the first failure occurs at the post-execution read of `memory.events`. This is a control-flow probe, not a claim that delegation was removed from the real host.

   Smallest fix: before `Popen`, read and validate effective `memory.max` and `memory.swap.max`, and require the event/peak interfaces used by the reporter. Missing or mismatched limits must exit as setup failure without launching the command. Keep the post-run report. Add a negative test whose child leaves a marker and prove that missing/mismatched controller state prevents that marker.

2. **[P1] Host OOM kills are misclassified as candidate cap violations.** `trials/e1-checked/jail_limits.py:27`, `:29`; `trials/e1-checked/harness.py:338`.

   `memory.events.oom_kill` counts victims of any OOM killer, including global or ancestor-cgroup OOM. It does not prove that this scope exceeded its 4 GiB cap. The reporter discards the local `max` and `oom` counters, and `run()` turns any kill into `memory_limit_exceeded=True, infrastructure=False`. A modest writer process killed because another workload exhausts the host therefore loses a trial instead of receiving protocol §3’s infrastructure handling. Raising its `oom_score_adj` to 1000 makes it particularly likely to be that host-OOM victim. The distinction between `max`, `oom` and `oom_kill` is explicit in the [kernel cgroup-v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

   Probe: `python3 codex-r1/probes.py`, `host_oom_report`. Injected a valid report representing an OOM victim with a 64 MiB peak beneath a 4 GiB cap and a surviving parent exit of 0. The result is code 137, `memory_limit_exceeded=true`, `infrastructure=false`. This validates the classification path; no host OOM was induced.

   Smallest fix: retain evidence of local limit exhaustion in the trusted report and distinguish it from an external/ambiguous OOM kill. Preserve candidate failure for proven local cap kills, including a killed child whose parent exits 0; classify external host/ancestor kills as infrastructure. Add both classification controls alongside the existing real local-cap test.

3. **[P2] A directory that cannot be listed can still contain readable files.** `trials/e1-checked/gate_checks.py:112`, `:115`; regression-test gap at `trials/e1-checked/harness_test.py:826`.

   The new `os.walk` error callback skips every PermissionError on directory enumeration. Directory read permission permits listing names; execute/search permission permits opening a known child. A mode-0111 directory therefore fails enumeration while its known mode-0644 files remain readable by the same uid. The scan certifies the store without hashing those files. The same-uid justification in the docstring/README does not establish inaccessibility. Closure exclusion remains a separate defense; this finding does not claim an arbitrary outside-closure path is mounted.

   Probe: `python3 codex-r1/probes.py`, `search_only_directory`. An actual filesystem control places synthetic held-out bytes at `<store>/<hash>-unrelated/opaque/known.txt` and sets `opaque` to 0111. Result: `passed=true`, zero files hashed, directory recorded in `unreadable_skipped`, and `same_uid_can_read_skipped_content=true`. No held-out corpus bytes were used. The additional real-jail version is queued and is not counted as proof.

   Smallest fix: keep directory-enumeration errors fatal, or skip only when inability to traverse the directory is independently established for the jailed credentials. Continue skipping PermissionError from opening an individual file. Add the searchable-but-unlistable directory case; the existing mode-000 directory test misses it. Correct the directory claim in README.md:376 and the store-scan docstring.

4. **[P2] File-type checks silently suppress non-permission scan failures on this host.** `trials/e1-checked/gate_checks.py:125`, `:126`.

   This is inherited code in the changed scan, but it fails this review’s explicit requirement that every other error remain fatal. The host uses Python 3.14.8. `Path.is_file()` suppresses all OSError exceptions in Python 3.14; an EIO on a regular store file therefore makes this branch skip it before the explicit `path.stat()` or hashing handler runs. The scan returns passed with no error or unreadable record. See [Python 3.14 file-status documentation](https://docs.python.org/3.14/library/pathlib.html#querying-file-type-and-status).

   Probe: `python3 codex-r1/probes.py`, `regular_file_io_error`. Injected `OSError(EIO)` from `os.stat` only for a synthetic store file containing a held-out control. Result: `passed=true`, `files_sized=0`, `matches=[]`, `unreadable_skipped=[]`. A separate injection at the later explicit `Path.stat()` does fail with errno 5, confirming that the early predicate is the hole.

   Smallest fix: use explicit `lstat`/mode inspection for symlink and regular-file classification, with deliberate PermissionError handling and other errors propagated. Reuse that metadata for size. Add the EIO regression at the first metadata lookup, not only during hashing.

## Non-blocking

No additional actionable non-blocking findings. The author’s remaining Tide reference failures (C3/C6 at L and C7 at execution) are disclosed and tracked separately; this review did not inspect their held-out contents or establish a new wrapper-caused failure.

## Reasoning

The direction and shared owners are appropriate. `harness.run()` is the common boundary for both arms’ compilation, writer tests, acceptance tests and binary cases. It wraps those jails with the same scope and tmpfs constants. `environment()` and the trusted toolchain build pin Cargo jobs to 4; removing the coordinator’s duplicate pin leaves that policy with the harness. Binary-case environments remain the case’s explicit inputs, as the protocol requires.

`g.identity` records jobs, memory/swap/tmpfs limits, CPU weight, helper paths and each arm’s closure. Its machinery hashes include `jail_limits.py`. The systemd-added environment is replaced with the recorded jail environment before bwrap executes the candidate. The inherited XDG_RUNTIME_DIR is used to reach the outside user manager; it is not passed to the writer. No new caller-environment leak was established.

The closure bind removes the whole-store mount. Absolute symlinks resolve inside the jail mount namespace; they do not themselves reopen an unmounted host store path. The namespace options provide private PIDs and procfs, private devices, and no host /sys or /run mount. The limits directory/report is outside the work bind and private /tmp. No new namespace escape was established by source inspection. The author’s real-jail tests, canaries and 30 Rust reference passes provide execution evidence for the recorded toolchain closure; that evidence does not repair the scan’s independent false-pass cases above.

The generated `use tide_candidate as solution;` matches the driver’s external-crate name, and `harness.suite()` consumes the alias with `use solution::*;`. The author ran all 30 Tide references on this HEAD, reporting 27 passes and no FX0912/FX0913 wrapper failure. The exact-wrapper unit test and the shared boundary cover the intended current callers. I did not claim equivalence for arbitrary future suite/module layouts.

Independent checks on this unchanged candidate, Linux host, uid 1000, Python 3.14.8:

- `git diff --check main...HEAD`: pass.
- Five selected lightweight GateTests: 5 run, all pass. These cover identity fields, held-out byte matching versus writer-visible exclusions, revision-independent paths, tmpfs arguments, and canary failure controls. Result: `codex-r1/light-tests.json`.
- `python3 /home/lucca/.cache/eon-agents/tide-fx-ic2.8-work/codex-r1/probes.py`: completed; concrete counterexamples and the fatal explicit-stat control are recorded in `codex-r1/probe-results.json`. All fixtures and writes are outside the candidate.
- An additional real-jail isolation probe and seven focused tests were submitted through `bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 180 python3 /home/lucca/.cache/eon-agents/tide-fx-ic2.8-work/codex-r1/jail-probes.py </dev/null`, with load1 6.75 at submission. The command remains queued under heavy.sh (wrapper PID 1811228); the quiet window was extended to 02:14:31 UTC. It has not been bypassed or counted as completed proof. Its outputs, if subsequently produced, stay in codex-r1. The existing author’s real-jail proof was reviewed, not independently rerun in this pass.

Confidence: high in the four demonstrated source/control-flow defects; independent new jailed behavior remains unproven while the scheduled probe is held. No full reference rerun, full harness suite, full host store rescan, host-OOM experiment or controller reconfiguration was performed by this reviewer. Approve only after the four findings are fixed or rebutted and re-checked; the subsequent Opus round still remains required.

DONE
