Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: request_changes

Scope: full converge round (reports, does not fix) on fx-ic2.8, HEAD b139dd792db7cb436c4409ade2f402630f5ca069, `git diff main...HEAD` (merge base cc3bb2d). Read: the bead contract, fx-rpwp's fault list, protocol §2, §3, §4, §6 and §10, the author's report rounds 1-3, both Codex files, and every changed file plus its callers: broker.py, writer_guard.py, and coordinator.py's guard, grading and reclaim paths. No held-out file was opened or searched. The only rust/ access was `stat -c %s rust/src/lib.rs` (size 0), a metadata read, not a content read. No edits, commits or br writes.

Probes (all synthetic, run directory opus-r2/):
- `orphan_nojail.py`, run directly. It compiles nothing and starts no bwrap jail: it is systemd-run plus jail_limits.py plus /bin/sleep. Result: survives=true (finding 1).
- Host probes: cgroup placement of a `systemd-run --user --scope` unit, app.slice's cpu controller, and unprivileged oom_score_adj lowering from 1000 to 0 (findings 2 and 9).
- `probe.py` (jailed: lowered OOM score plus a tmpfs fill under a 64 MiB cap, and jail visibility of /nix/store, /sys, /run, /proc and pid 1's environment) and `orphan_probe.py` (the jailed version of finding 1) were queued through heavy.sh at load 3.4. heavy.sh held them for over 30 minutes behind the booked 03:00Z quiet-window benchmark, so I stopped the queued wrapper before it could start unattended. Neither ran, and neither counts as proof. Both scripts are ready for the re-check.

## Findings

1. **Blocking. A jail now outlives the harness process: no time limit, and the coordinator cannot see it.**
   Code: `trials/e1-checked/jail_limits.py:38-55` and `trials/e1-checked/harness.py:310-319`.
   - On main, bwrap is the harness's direct child, and `--die-with-parent` (bwrap sets PR_SET_PDEATHSIG for its parent) kills the sandbox when the harness dies. Now the chain runs harness → systemd-run, which execs `jail_limits.py` → bwrap. bwrap's parent is the reporter. The reporter has no parent-death signal and runs in its own session (`start_new_session=True`), so a harness death no longer reaches the jail.
   - Probe: `opus-r2/orphan_nojail.py` builds that exact scope argv, with the stand-in harness in its own group as the grader is. After the stand-in is SIGKILLed, the reporter (pid 2884970) and its child (pid 2884979) are both still alive 3 s later: `"survives": true`. Under bwrap, the outer bwrap's parent is that live reporter, so `--die-with-parent` never fires.
   - Paths that kill the harness mid-run:
     - operator cancellation and coordinator shutdown: `end_guard`/`stop_active`, then writer_guard `killpg` of the guard group that holds `harness.py grade`;
     - coordinator death: PDEATHSIG to the guard;
     - the broker's 150 s `killpg` of `harness.py command`;
     - the grader's 600 s backstop;
     - a harness crash.
   - In each case the writer's unreviewed process keeps running with no 10 s or 120 s limit. Only the 4 GiB scope cap holds; §10.3's time limits no longer apply. It also writes into the abandoned snapshot work tree.
   - `wait_quiescent` (coordinator.py:208) checks only the guard group, and the session.json flock is released when the harness dies. So recovery declares the slot quiescent and re-grades or reruns while the orphan still runs. This regresses fx-rpwp's recovery contract ("relies on the parent-death guard, waits for provable quiescence").
   - Fix:
     - In `jail_limits.py`, call `prctl(PR_SET_PDEATHSIG, SIGKILL)` first.
     - Then check `os.getppid()` against a harness pid passed on argv, as writer_guard.py does. Exit as a setup failure on a mismatch, which closes the race where the harness dies before the prctl.
     - The reporter then dies with the harness, and bwrap's `--die-with-parent` follows.
     - PDEATHSIG fires when the parent *thread* exits. Today `run()` runs on the main thread of harness.py processes; keep it there or note the constraint.
     - Add a real-scope test: start `h.run(["/bin/sleep", N], …)` in a child process, SIGKILL that child, and assert that no jailed `sleep N` remains. `opus-r2/orphan_probe.py` is that test in probe form.

2. **Non-blocking. `CPUWeight=1` is unrequested scheduling policy. The README misstates where the scope sits, and starvation timeouts count as candidate failures.**
   Code: `harness.py:313`, `README.md:181`.
   - Placement probe: the caller runs in `app.slice/app-cosmic-eon-502687.scope/init.scope`. The jail scope lands at `app.slice/run-….scope`, a sibling of that Eon scope and of every other app unit (browsers, other Eon windows, heavy builds in their own scopes), each at weight 100. systemd turns on app.slice's cpu controller for it. It is not "a sibling of its caller's cgroup".
   - On main, heavy.sh's nice 19 ranked the jail only inside the caller's cgroup. Now the jail gets about 1/101 of any contended CPU against each busy sibling. When all 16 CPUs are busy (an Eon `nix build --cores 6 --max-jobs 2` plus another slot is enough), a 10 s test or a 120 s command can time out.
   - That timeout is code 124, a candidate failure, never infrastructure. §3 counts host faults as infrastructure, but this one cannot be told apart.
   - Exposure is uneven between the arms. Each sandboxed Rust command rebuilds the vendored crates in a fresh `work/build` (harness.py:263-264), so it spends far more CPU per command than the Tide driver with its prebuilt std.
   - The bead's Scope does not list CPU policy. Some choice was forced, since a separate scope at the default weight 100 would rank jails above the old setup.
   - Fix:
     - Correct the README sentence.
     - Have `jail_limits.py` report the scope's `cpu.stat` `usage_usec`, and have `run()` record it next to `memory_peak`, so the grader can audit any timeout with low CPU use.
     - The coordinator should record the CPUWeight=1 choice and its starvation risk on fx-ic2, since it sets how host load turns into trial failures.

3. **Non-blocking. The real-jail store test would pass even if the jail bound all of /nix/store again.**
   Code: `harness_test.py:924-945`.
   - The fake store and its "older revision" copy sit inside the test's TemporaryDirectory under host /tmp. Every jail hides host /tmp behind its own tmpfs, whatever the store bind is. The `[True, False, False]` result therefore proves that unbound paths are invisible, which was already true. It does not prove that only the closure is bound.
   - Today only the structural `assertNotIn(["/nix/store", "/nix/store"])` (harness_test.py:508) and the canary's single `store_outside_closure` entry (gate_checks.py:203-206) catch that regression.
   - Fix: when /nix/store exists, have the real-jail test assert that `os.listdir('/nix/store')` inside the jail equals the closure basenames. That discriminates, and it is one line.

4. **Non-blocking. The writer-visible exclusion is a `rust/` prefix, not the list of files writers actually get.**
   Code: `gate_checks.py:88-93`, against `harness.py:524` and `:361`.
   - Writers and acceptance packages receive only `rust/Cargo.toml` and `rust/Cargo.lock`. Today the rule is exact: rust/ has three tracked files, and `rust/src/lib.rs` is 0 bytes, so the size rule already covers it.
   - Any held-out file added under rust/ later would go unscanned with no signal. That is the same duplicated-owner problem that `TIDE_WRITER_DOCS` solved for the Tide documents.
   - Fix: add a `RUST_WRITER_FILES = ["Cargo.toml", "Cargo.lock"]` constant, used by `prepare`, `package` and `writer_visible`.

5. **Non-blocking. Proof gaps for landing.**
   - The cap rule says "above every reference solution's measured peak". The C3 and C6 acceptance builds and runs never ran, since both stop at L, so those peaks are unmeasured. Add a re-measure against `MEMORY_MAX` to fx-ic2.12's acceptance.
   - The `references tide` (and Rust) results come from 0294043 and 08c735d. Round 3 changed `run()`, which every reference stage passes through. The Acceptance says "on the landed tree", so the landing tree needs a fresh `references tide` run under AP-PROOF-001. That run is the coordinator's step at landing in any case.

6. **Nit. The timeout path ignores the scope report.**
   Code: `harness.py:322-326`.
   - On a timeout the run returns 124 before it reads `report.json`. The reporter is SIGKILLed with its group, so no report exists anyway.
   - A timed-out jail therefore records no `memory_peak`. A host OOM followed by a hung parent is graded as a candidate timeout.
   - That is rare. Note it in the README or record `memory_events: null` on timeouts.

7. **Nit. readiness.md:135 is stale.** It still says "optional /nix/store mounts". The jail now binds only the toolchain closure.

8. **Nit. One variable name means two things in the canary.**
   Code: `gate_checks.py:204-206` and `:233`.
   - `outside` holds the store path before the shape loop.
   - Inside the loop it is rebound to the negative-control record.
   - That is harmless, but confusing. Rename one of them.

9. **Nit. The `jail_first` comment overstates what protects the reporter.**
   Code: `jail_limits.py:20-23`.
   - On this host an unprivileged process lowered its own `oom_score_adj` from 1000 back to 0. I did not probe this inside a jail, but bwrap's user namespace grants no CAP_SYS_RESOURCE, so the same should hold there.
   - So a jail can lower its own score. What keeps the reporter from being chosen in a scope OOM is the outer bwrap process. It stays at 1000 outside the pid namespace, out of the jail's reach, and its death takes the jail with it through `--die-with-parent`.
   - State that in the comment, so a later change does not drop the outer process's score or move bwrap out of the scope.

## Checked without a finding
- **Verdicts.**
  - A local cap kill requires local `max`, `oom` and `oom_kill` together. It gives 137 and a candidate failure, even when a surviving parent exits 0.
  - An external or ambiguous kill, a missing or mismatched report, and a reporter crash each give code 2 with infrastructure set. That maps to `harness_setup` in coordinator.py's closed `FAULTS`, which matches fx-rpwp's list and §3.
  - The writer cannot reach the reporter or the limits directory (separate pid namespace, private /tmp).
  - The writer cannot get the reporter OOM-killed while the outer bwrap, at score 1000, is alive.
  - A local OOM cannot pass for an external one.
- **Isolation.**
  - The environment comes only from env.json, so `INVOCATION_ID` and `XDG_RUNTIME_DIR` never reach the jail.
  - /sys and /run are not mounted. /proc is a new pid namespace's procfs, and the cgroup namespace is unshared.
  - The reporter runs `-I` (it never imports from the writable cwd), and the canary's `os.open` probe covers directories.
- **Store scan.**
  - The unreadable-file skip is sound here: the store is root-owned, and the walk stays fatal on listing errors.
  - The revision-independent path rule and its `.links` handling are correct.
- **Identity and jobs.**
  - Jobs, caps, CPU weight, the closure and `jail_limits.py` (through `revision()`) all feed the identity record and the canary key.
  - No `CARGO_BUILD_JOBS` comes from the caller anywhere. Only `nix-store` (on PATH), `XDG_RUNTIME_DIR` and `sys.executable` come from the caller's environment; none of them changes jail behaviour.
- **TMPDIR.** /tmp is a 1 GiB tmpfs in every shape of both arms, with one rule for both.
- **L wrapper.** `use tide_candidate as solution;` is pinned, and the test asserts that no `extern crate` appears.

DONE

## Round 4 re-check

Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: request_changes

Scope: this re-checks only r2 findings 1-4 and 6-9 and any regressions their fixes introduce. Candidate: HEAD cbe31447f79c2cac951ce35fec44b4cd12c09db8 (one commit on cc3bb2d); the tree was clean throughout. I reviewed `git diff b139dd7 cbe3144` and the author's "Round 4" account.

The coordinator's decisions are recorded as given:
- Finding 2 keeps CPUWeight=1 (README corrected, `cpu_usage_usec` recorded).
- Finding 5 is the coordinator's.

The r2 probes queued earlier have now run through heavy.sh, at load 8.8-10.3. Outputs are in `opus-r2/r4-*.out`.

### Fixes
1. **RESOLVED: the jail outlived the harness.**
   - Code: `jail_limits.py` now sets PR_SET_PDEATHSIG to SIGKILL before anything else, then checks `os.getppid()` against the harness pid passed in `harness.py:315`. A mismatch writes setup_error and exits 125.
   - `orphan_diag.py` (jailed) shows the ancestry is harness → reporter (`-I jail_limits.py`) → bwrap → bwrap → sleep. After a SIGKILL of the harness's group, the jailed sleep is gone by 0.5 s, 3 s and 12 s.
   - The original `orphan_probe.py` ran 3 more times: survives=false each time.
   - `test_jail_dies_with_a_killed_harness` passes on this HEAD. It is a real scope and a real jail, it discriminates, and the author's mutation check confirms that.
   - The main-thread constraint is documented, and it holds: coordinator and broker reach `run()` only through harness.py subprocesses.
   - **One observation.** The very first `orphan_probe.py` run on cbe3144 (right after `probe.py`, load about 9) still saw the jailed sleep 3 s after the kill. All 4 later runs did not, and the probe killed its process by exact pid. I could not reproduce it. Teardown inside a CPUWeight=1 scope under contention is the likely cause. The fix is sound. The test's 10 s wait is the right margin; do not shorten it.
2. **RESOLVED (as decided).**
   - The README now places the scope correctly and states the starvation risk.
   - `cpu.stat` is required before launch, and `usage_usec` is recorded on every jailed result. `cpu.stat` is a core cgroup-v2 file, so it does not depend on the cpu controller.
3. **RESOLVED.** The real-jail test now asserts that the jail's `/nix/store` listing equals the real closure's basenames exactly. It passes, and it would fail under a full-store bind.
   - Independent check: `probe.py` "visibility" under the recorded Rust config shows 27 store entries (the closure is 27), no /sys, no /run, cgroup `0::/`, and only pids 1 and 2 in /proc.
   - Pid 1's environment holds only the 7 jail variables: no `INVOCATION_ID`, `XDG_RUNTIME_DIR` or caller variables.
4. **RESOLVED.** `RUST_WRITER_FILES` is the one owner for `prepare`, `package` and `writer_visible`. The test now shows that a non-empty held-out file under rust/ is byte-matched.
6. **RESOLVED.** Timeouts record `memory_events: null`, and the README says so. Nothing reads `memory_events` as a dict.
7. **RESOLVED.**
8. **RESOLVED.** The rename's own regression (a Path in `negative_control`) was caught by the author's canary run and fixed. The canary test now JSON round-trips its record.
9. **RESOLVED.** The comment and README now name the outer bwrap as the reporter's protection. The probe below shows that protection is not sufficient.

Focused tests on this HEAD, through heavy.sh: test_jail_dies_with_a_killed_harness, test_jail_sees_only_the_store_closure_not_an_older_trial_copy, test_reporter_launches_nothing_without_the_recorded_limits, test_jail_memory_cap_kills_and_the_verdict_records_it, test_store_scan_skips_unreadable_and_writer_visible_but_fails_held_out_bytes and test_only_local_limit_exhaustion_is_a_candidate_memory_failure. Result: 6 run, OK.

### New finding from the r2 probe that could not run in r2

10. **Blocking. Jail code can get the in-scope reporter OOM-killed, which turns its own cap failure into infrastructure.**
    Code: `jail_limits.py:20-30` and `:55-58`, `harness.py:333-337`.
    - `opus-r2/swarm_diag.py` runs a real scope and jail under a 96 MiB cap. The jailed parent lowers its `oom_score_adj` to 0, then forks 40 children of 3 MiB each.
    - Over the first probe plus this run, 2 of 4 lowered-score runs came back `reporter_exit -9`: the reporter was SIGKILLed while the harness lived. The result was code 2, `infrastructure: true`, "E1 jail memory limit was not applied", with no traceback on stderr. The other 2 were correctly `137 memory_limit_exceeded`. The 2 control runs without lowering were both correctly 137.
    - Mechanism (inference; consistent with the kernel's victim selection):
      - The first scope OOM picks the outer bwrap, at score 1000.
      - Its death tears down the pid namespace, but the 40 dying children are not OOM victims and still hold memory.
      - The next charge in the scope, including the reporter's own wake-up, OOMs again. At equal score 0, the reporter's RSS (about 10 MiB) is the largest.
    - A jail can lower its own score (shown in r2, and the `lowered_adj_tmpfs_fill` probe printed `lowered=0` and `pid1_lowered`). So the protection in the round-4 comment holds only for the first victim.
    - Consequences:
      - A writer's `./test` that does this gets `harness_setup` and a free session rerun.
      - In grading, the frozen snapshot is re-graded at most twice, and then the coordinator stops the whole gate run. One final snapshot can therefore halt E1.
      - An honest writer is unlikely to hit it: it needs many processes with lowered scores. But the brief's question was whether a writer can do it, and the Rust arm can.
    - Fix: keep the reporter out of the capped cgroup, so a cap OOM can never select it. For example:
      - Run the scope with `Delegate=yes`.
      - The reporter moves itself into a `reporter/` leaf, enables memory in `subtree_control`, and creates a `jail/` leaf with the effective `memory.max`/`memory.swap.max` checks applied there.
      - bwrap's preexec writes its pid into `jail/cgroup.procs`, and the reporter reads `jail/memory.events.local`, `memory.peak` and `cpu.stat`.
      - The author rejected delegation in round 1 as "more machinery for the same data". This probe shows it is not the same data.
    - Add `swarm_diag.py`'s lowered-score swarm as a real-scope test. Repeat it about 5 times, or make it deterministic, and assert 137 with `memory_limit_exceeded` and never infrastructure.
    - By the Merge Rule it blocks: the fix changes harness code that this change edits.

### Note for the coordinator (not a finding against these fixes)
- The author's round-4 store scan failed once on a root-owned 0700 `*.drv.chroot` left by a concurrent Nix build. The uid has no search permission on such a directory, so nothing beneath it is reachable by the operator or by a jail.
- Record-and-skip is sound for a directory the uid cannot search (`os.access(dir, os.X_OK)` is false). Directories the uid can search but not list, which was Codex's 0111 case, must stay fatal.
- Without that, the gate's pre-run and post-run scans fail whenever a Nix build is running on the host.

DONE

## Round 5 re-check

Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: approve

Scope: this re-checks only finding 10's fix and any regressions it introduces. Candidate: HEAD 2d85521946a61d680dfce10bf6c9ad92b4d03cc8 (one commit on cc3bb2d); the tree was clean throughout. I reviewed `git diff cbe3144 2d85521` and the author's "Round 5" account. Item (B) is Codex's to re-check; I flag one observation on it below. All probes ran through heavy.sh at load 2.7-3.9. Outputs are in `opus-r2/r5-probes.out`.

10. **RESOLVED: jail code could get the reporter OOM-killed.**
    - Code:
      - `harness.py:312-315`: the scope is `Delegate=yes` with no scope-level memory cap.
      - `jail_limits.py:68-95`:
        - The reporter moves itself into `reporter/` before it enables `+memory` in the scope's `subtree_control`, as the no-internal-process rule requires.
        - It refuses to launch unless `jail/` lists the memory controller.
        - It writes `memory.max` and `memory.swap.max` on `jail/`, reads them back and compares them with limits.json.
        - It requires `memory.peak`, `memory.events.local` and `cpu.stat` on `jail/`. Only then does it start bwrap.
      - `jail_limits.py:33-38`: bwrap's preexec writes `jail/cgroup.procs` before exec, so no jail code ever runs outside `jail/`. A failed move raises as `SubprocessError`, which the new `except` catches as a setup error. The reporter's own cgroup is not an ancestor of `jail/`, so a cap OOM cannot select it.
    - Probe: `swarm_diag.py`, run twice: 10 runs, 6 of them lowered-score swarms. All 10 came back 137 with `memory_limit_exceeded` and infrastructure false. `reporter_exit` was 137 every time and never -9.
      - Local events always show max > 0, oom > 0 and oom_kill of 1-7.
      - `probe.py`'s `lowered_adj_swarm` is also 137, not infrastructure, and `lowered_adj_tmpfs_fill` is 137. The jail still lowered its own score and pid 1's to 0.
      - On cbe3144 the same probes failed 2 of 4.
    - The new test `test_a_lowered_score_swarm_cannot_turn_its_cap_failure_into_infrastructure` (5 real-scope repetitions) passes.

### Regression checks on the fix
- **PDEATHSIG and parent pid.** Unchanged and still first, before any cgroup write. `orphan_probe.py` ×3 gave survives=false each time.
  - `orphan_diag.py`: the ancestry is harness → reporter → bwrap → bwrap → sleep, and the sleep is gone at 0.5 s, 3 s and 12 s after the harness's group is SIGKILLed.
  - `test_jail_dies_with_a_killed_harness` passes.
- **Pre-launch validation.** The unit test's missing, uncontrolled, refused (kernel keeps "max"), orphaned and applied cases each behave as required: exit 125 and no marker, except applied. The applied case checks that the reporter, the cap and the jail pid all land in the right leaves.
  - The harness still rejects a report whose `memory_max`/`memory_swap_max` differ, as infrastructure.
- **Accounting.** `memory.peak`, `cpu.stat usage_usec` and the `max`/`oom`/`oom_kill` deltas now all come from `jail/`. That is cleaner than the scope-level figures, which counted the reporter. The cap test still asserts a peak below the cap and `cpu_usage_usec` above 0.
  - Leftover pages the forked child had charged in `reporter/` before its exec are negligible.
- **Classification.** The paths are unchanged. "Local" now means the `jail/` leaf, so a scope-level or ancestor OOM still reads oom=0 and is infrastructure, with `external_oom_kill`.
  - `test_only_local_limit_exhaustion_is_a_candidate_memory_failure` passes.
- **Isolation unchanged.** The cgroup namespace is created after the move, so the jail sees `0::/` rooted at `jail/`: no cgroupfs, no /sys, no /run, 27 store entries, pid 1's environment holds only the 7 jail variables.
  - The jail therefore cannot write `reporter/` or the scope's files.
- **Docs.** README:180-185, the jail_limits docstring and the harness comment describe the leaves. No stale scope `MemoryMax` text remains.
- **Focused tests on 2d85521, through heavy.sh:** test_a_lowered_score_swarm_cannot_turn_its_cap_failure_into_infrastructure, test_jail_dies_with_a_killed_harness, test_reporter_launches_nothing_without_the_recorded_limits, test_jail_memory_cap_kills_and_the_verdict_records_it, test_only_local_limit_exhaustion_is_a_candidate_memory_failure, test_jail_sees_only_the_store_closure_not_an_older_trial_copy and test_store_scan_skips_unreadable_and_writer_visible_but_fails_held_out_bytes. Result: 7 run, OK.
  - The author's full proof on this HEAD (harness_test 78 OK, both canaries, the four-slot dry run and the host store scan) is recorded in report.md Round 5. I did not rerun it.

### Item (B), flagged for Codex (not re-checked by me)
I read the `X_OK` skip and saw no problem.
- Listing errors stay fatal unless the uid lacks search permission. A 0111 directory therefore still raises.
- A readable but unsearchable directory has its names checked against the holder rules before its contents are skipped.
- `os.access` uses the real uid, which here equals the jail's uid, and it honours ACLs.

DONE
