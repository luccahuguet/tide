Model: gpt-6.1-sol · Effort: high · Date: 2026-11-14
Verdict: approve

Scope: re-checked ONLY r1 findings 1–4 and regressions introduced by their fixes, on HEAD b139dd792db7cb436c4409ade2f402630f5ca069 (tree 9a7a90f53fb5a9c44bd5aad8e8ff9a9ef9c68428). Reviewed the delta from 029404327d2dc30da7ea1186406fd08e885a0e56 and the author’s “Round 3 (Codex r1 fixes)” account. Fix-diff SHA-256: c607811eecc3f2690e409adacf849a5cb68695379eb1a355e24bc114d76b197e. Candidate remained clean and HEAD unchanged. No candidate edits, commits, br writes or held-out file inspection.

1. Finding 1 — RESOLVED: verify the memory controller before executing the jail.

   trials/e1-checked/jail_limits.py:42–55 now reads the expected caps, checks effective memory.max and memory.swap.max, reads memory.peak and the required local event counters, and refuses launch on missing or mismatched state. The setup report returns 125; harness.py:335 preserves code 2/infrastructure when the cap report is absent or invalid. The command starts only after those checks.

   Adapted independent probe: missing memory controller → child_launched_before_validation=false, code 125, setup_error naming memory.max. GateTests.test_reporter_launches_nothing_without_the_recorded_limits passed its missing/mismatched/applied cases, including the child marker’s absence in both negative cases.

2. Finding 2 — RESOLVED: distinguish host OOM from local cap exhaustion.

   trials/e1-checked/jail_limits.py:47 and :56–58 record the run’s deltas of local max, oom and oom_kill counters. harness.py:340–349 requires local max and oom evidence with the kill for candidate failure; a kill without that evidence is code 2/infrastructure with external_oom_kill. A surviving parent cannot conceal a proven local cap kill.

   Adapted independent host-OOM control now returns code 2, infrastructure=true, external_oom_kill=true, observed_exit=0, with a 64 MiB peak under the 4 GiB cap. GateTests.test_only_local_limit_exhaustion_is_a_candidate_memory_failure passed its local, external and ambiguous controls. GateTests.test_jail_memory_cap_kills_and_the_verdict_records_it also passed in a real systemd/bwrap jail: the allocating child is killed under the 64 MiB cap, its parent exits 0, and the harness records candidate failure 137 rather than infrastructure. No host OOM was induced.

3. Finding 3 — RESOLVED: an unlistable directory may still have readable children.

   trials/e1-checked/gate_checks.py:115–117 now raises directory traversal errors instead of treating them as unreadable files. README.md and the scan docstring describe that distinction. Individual unreadable-file skips remain, and writer-visible exclusions and revision-independent path matching were not weakened by this fix.

   The requested unchanged codex-r1/probes.py was rerun on this HEAD. It exits 1 at its first scan with the expected PermissionError from the mode-0111 directory; this is the corrected fail-closed behavior, not a new regression. An adapted copy catches that expected exception so the remaining controls can execute: fatal_errno=13 while the same uid can still read the known child. The updated store-scan regression test passed both mode-0111 and mode-000 directory rejection, individual mode-000 file skipping, writer-visible exclusions and readable held-out-byte detection.

4. Finding 4 — RESOLVED: file-type checks must not swallow other I/O errors.

   trials/e1-checked/gate_checks.py:129–140 uses explicit os.lstat and mode inspection, reuses the returned size, and propagates non-permission metadata/read failures. A directory misclassified by os.walk is rejected explicitly. The old Path.is_file predicate is gone.

   Adapted independent first-metadata probe injects EIO at os.lstat and receives fatal_errno=5. The former Path.stat/os.stat injection no longer causes a false pass: the control file is hashed and its held-out-byte match fails the scan. The updated store-scan regression test also passed its first-lstat EIO assertion.

Verification:

- Exact requested rerun: python3 /home/lucca/.cache/eon-agents/tide-fx-ic2.8-work/codex-r1/probes.py. Expected early PermissionError as described above. Original probe preserved unchanged.
- Completed remainder using codex-r1/recheck/probes-adapted.py. Changes to the probe only catch the now-expected directory failure, inject at the replacement metadata call, and supply the new internal limits/event report fields. Results: codex-r1/recheck/probe-results.json. All four counterexamples now produce the required outcomes.
- Four focused GateTests: 4 run, all passed, no failures or errors. These are the three regression tests identified above plus the real local-cap kill test. Executed on this HEAD through:

  bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 180 python3 /home/lucca/.cache/eon-agents/tide-fx-ic2.8-work/codex-r1/recheck/focused-tests.py </dev/null

  Submission load1 was 4.78, below 12. heavy.sh granted build.lock after queueing; command exited 0. Python 3.14.8, uid 1000, Linux host. Results: codex-r1/recheck/focused-results.json. No review command remains queued from this re-check.
- git diff --check 029404327d2dc30da7ea1186406fd08e885a0e56..HEAD: pass.

No blocking findings remain from r1 findings 1–4, and no regression was established in their fix delta. Confidence: high. This scoped approval clears the Codex findings for the subsequent required Opus review; it does not replace that review.

Not re-run: the full harness suite, reference corpus, canaries, dry run or host store scan. The author’s new-candidate proof for those applicable surfaces was reviewed; independent checking here stayed focused on the four fixes. Missing-controller and host-OOM classification used controlled fixtures, without changing host delegation or exhausting host memory.

DONE
