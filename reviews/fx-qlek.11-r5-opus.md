Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate d077152

# fx-qlek.11 r5: Opus converge round (report only)

Scope: `git diff main...d077152` (10 files), read in the worktree at d077152 (HEAD has the same tree). Also read: bead fx-qlek.11 with all comments, fx-qlek's A1 record, GOVERNANCE.md's Merge Rule, `report.md`, `r1-codex.md` to `r4-recheck-codex.md`, `by-hand.log`, the generated `logs/record.log` and `search.log`, the article condensation `eon-desk/shared/debugging-rust-2026-jetbrains.md`, `trials/e3-pilot/report.md`, and the driver sources that step 5 depends on (`invocation.rs` re-exec and `rustc_args`, `invocation/replay.rs` `supervise`, `invocation/sandbox.rs` `install`). I ran no builds, tests or debuggers.

## Verdict

**approve_with_alternatives**, conditional on B1, which is a text-only fix and needs no re-check (GOVERNANCE.md: "text-only fixes need no re-check"). Confidence: high on the code, medium-high on the claims.

The code is correct for the cases the brief names. Step 5 cannot pass when gdb follows the wrong process, and it cannot pass without the replay marker. The bounded run kills what it promises to kill, and it cannot signal a reused group id. One claim on the page is false today, and it sits in the paragraph the bead singles out.

## Blocking findings

### B1. "Every Rust debugging skill applies" is false today, and the page says so itself

- **Where:** `docs/debugging.md:155-156` ("Tide programs are Rust, and every Rust debugging skill applies to them"). Also, more mildly, `docs/debugging.md:4-5` ("gdb, lldb, and the IDEs built on them see an ordinary Rust executable").
- **What is wrong:** the driver passes no `-g` or `-Cdebuginfo`. In `crates/tide-admit/src/invocation.rs` `rustc_args`, the only profile switches are opt-level and debug-assertions; `--tide-profile=debug` does not add debuginfo. The page's own gaps section (`:112-117`) says gdb gets "no source lines and no named locals". Many core Rust debugging skills fail on Tide code today:
  - breaking on `file:line`;
  - stepping by line;
  - `print seen`;
  - RustRover's gutter breakpoints;
  - the article's own rust-gdb technique of inspecting `a` and `b` in the frame.
  Step 5 works only because it reads x86_64 registers at closure entry.
- **Failing case:** a Rust developer reads the "just learn debugging" answer, the quotable summary that fx-qlek.7's launch-claims audit will check. They put `break hits.rs:57` on a Tide replay. gdb reports no line information, which contradicts the sentence they just read.
- **Minimum fix (text only):** say what is true now. For example, at `:155-156`: "Learn it. Tide programs are Rust, so Rust's debuggers run on them. Today the driver builds Tide code without debug information, so a debugger sees function names and backtraces but no source lines or locals (see the gaps above)." At `:4-5`, change "see an ordinary Rust executable" to "see an ordinary Rust executable, built today without debug information". The paragraph stays honest and not defensive, and it stays consistent with the gaps section.

## Non-blocking findings

### N1. A smaller correct shape for `run_bounded` (AP-MINIMAL-001)

- **Where:** `crates/tide-demo/src/lib.rs:1169-1300`.
- **Current shape:** about 130 lines: `run_bounded`, `exited_unreaped` (waitid WNOWAIT), `group_members` (a scan of `/proc` every 20 ms), `kill_group` (a 5 s poll), and `End::NotCleaned`. Four tests cover it.
- **Why it can be smaller:** the WNOWAIT dance exists only so that the code can know the leader exited without reaping it. That is not needed. If the leader is reaped only after the group is signalled, the unreaped leader (a zombie at worst) holds the pgid on every path.
- **Smaller shape:**
  1. Spawn in its own group.
  2. Forward the pipe in a thread.
  3. Call `recv_timeout` until `Disconnected` (EOF) or the deadline.
  4. On both paths, call `kill(-pgid, SIGKILL)`, accepting ESRCH. After EOF it reaches only a member that closed its pipe end, or the zombie.
  5. Then call `child.wait()`, which cannot block now.
  6. Drain for 2 s when the deadline passed.
- **What it keeps:** a complete log (EOF), no running survivors (SIGKILL cannot be ignored, ptrace-stopped tasks included), and no signal to a reused id. It needs only one libc call.
- **Cost:** about 40 lines. `exited_unreaped`, `group_members`, the 5 s poll and `NotCleaned` go away, and the four tests stay as they are, with a one-word change to the variant they match.
- **What it gives up:** a `/proc`-verified "nothing left running" check after SIGKILL. That matters only for processes in uninterruptible sleep, which a demo step need not prove.
- **Status:** the current code is correct. This is an alternative, not a block. Four review rounds converged on the current shape.

### N2. Step 5 throws away the read values, so it checks less than it prints

- **Where:** `lib.rs:1088-1089` parses `reads hits = N` and discards N. `lib.rs:996` concludes a lost update from `over > seen` alone.
- **Why it matters:** with the pinned toolchain the registers are right, and the evidence supports that. But the report's claim that a codegen change "fails loudly" holds only when the misread values are unparseable. If a toolchain bump moved `&seen` or `&mut Stats` to other registers that still point at readable memory, the lines would parse. The counts would still match, and a stray `over > seen` could pass the step with a made-up interleaving.
- **Cheap fix:** keep the read value in `Kind::Read`, and require each thread's write `seen` to equal that thread's last read. Optionally also require the first write's `over` to be 0. This ties the printed sentence "it read the count before another task wrote it" to observed data.

### N3. `cargo test --workspace` now depends on the host's gdb when one is on PATH

- **Where:** `the_demo_passes` runs step 5 whenever `find_gdb` finds something (`lib.rs:926`).
- **Failing cases:** gdb on PATH but unusable:
  - a build without Python, such as Fedora's gdb-minimal;
  - Yama `ptrace_scope` 2 or 3, which blocks gdb's `run`;
  - a container that denies ptrace;
  - an older gdb that rejects the script.
  Each of these now fails the workspace suite where it used to pass. The failure is loud, and the docs say a gdb with Python is needed. It is still a new environmental failure mode for the suite that AGENTS.md calls the production check.
- **Suggestion:** record it where the suite is described (README Verification), or skip with a printed line when a probe such as `gdb -nx -batch -ex 'python print(1)'` fails, except when `RUST_GDB` was set explicitly.

### N4. Error paths after spawn can leave the group running

- **Where:** `lib.rs:1203` (`exited_unreaped(group)?`) and `:1207` (`child.wait()?`).
- **What happens:** an `Err` from either returns from `run_bounded` without killing the group. In practice `waitid` on an unreaped child with WNOHANG fails only on ECHILD, for example if SIGCHLD were set to SIG_IGN. So this is theoretical.
- **Fix:** run `kill_group` before propagating the error. N1's shape removes the first site.

### N5. The by-hand path needs a demo run in which step 5 did not skip

- **Where:** `lib.rs:926-933`.
- **What happens:** `debug.gdb` is written only after `find_gdb` succeeds. `docs/debugging.md:83-92` says "after a demo run". Take a reader who ran the demo first without gdb, as most will, then installs gdb and pastes the by-hand block: gdb fails on a missing `debug.gdb`.
- **Fix:** write the script before the gdb check (move one line), or say "after a demo run with gdb". The page also never shows the script. A sentence pointing at `GDB_SCRIPT` in `crates/tide-demo/src/lib.rs` would help a reader who wants to adapt it to their own code.

### N6. Stale count in the test module doc

`crates/tide-demo/tests/demo.rs:4-5` says "Two more cases check step 5's bounded debugger run". There are four. This is a leftover from round 1.

### N7. Paraphrase of E3

`docs/debugging.md:24-25`: "the E3 pilot found no catch benefit on its items". The E3 report says the pilot "gives no evidence that the trust diff helps model reviewers catch planted authority widening". It adds that at the ceiling "a benefit and its absence give nearly the same counts". "Found no benefit" reads as evidence of absence. Suggested wording: "the E3 pilot, with model reviewers on small changes, gave no evidence of a catch benefit". The A1 condition itself is met: the previous sentence makes no catch claim, and the bead Position's "catches authority changes" does not appear.

### N8. "Adds little" and "adds nothing" disagree

`docs/debugging.md:32` says Tide "adds little" for the comparator, and `:162-164` says it "adds nothing that a debugger does not already give you". Pick one. "Adds little" fits better, since the earlier sentence admits the seed is irrelevant but the tooling still runs.

### N9. Smaller nits

- `lib.rs:193`: "Done: every step showed what it should" also prints after step 5 skipped. The skip line precedes it, so this is minor.
- `docs/debugging.md:20-22`: the trust diff covers `unsafe` holes as well as declared authority (README "What runs today"). It is fine to leave out, but "the authority each function's signature declares" is the narrower half.
- rr row: rr runs all of a program's threads on one core. That is why chaos mode exists, and it bears on how rr finds races. One clause would make the comparison fairer to both tools; it is optional.

## Reasoning

### Lens 1: correctness

- **Following the right process.** The script follows the re-exec (`invocation.rs:1131`, `Command::new(exe)...status()`, which matches `Command>::status`) and the sandboxed spawn (`replay.rs:383` `supervise`). It stays with the parent at the `--tide-compile` child, which uses `.output()` and does not match. `by-hand.log` confirms the sequence: vfork, re-exec, then fork into the test child.
- **Wrong process or renamed symbols.** If gdb stays with the driver, the pending dprintfs never resolve. Then `entries == 0` and the step fails with "the breakpoints did not all hit". If gdb follows the linker, the same happens.
- **Failure without the interleaving.** If the replay mismatches under gdb, the driver prints no `replay reproduced` marker and the step fails. A lone task, or a run with no `over > seen`, fails too. So the step cannot pass without showing an interleaving whose values gdb read from memory. N2 covers the residual risk that those reads come from the wrong addresses.
- **gdb absent:** a printed skip.
- **gdb broken:** for example `RUST_GDB` naming a missing file: rust-gdb exits nonzero, giving "rust-gdb failed".
- **gdb slow:** the 120 s deadline, then `kill(-pgid, SIGKILL)`.
- **The driver gdb detaches:** it stays in the group, so success waits for it and a timeout kills it.
- **The sandboxed child:** it called `setsid` (`sandbox.rs:122`) and has `PR_SET_PDEATHSIG` (`:126`), so it dies with the driver thread. gdb's exit-kill on tracees is a second path. Its output reaches `run_bounded` only through the driver's forwarders, so it never holds the pipe.
- **Unsafe code.**
  - `waitid` gets a zeroed `siginfo_t`. Zero is a valid bit pattern, and POSIX leaves `si_pid` at 0 under WNOHANG when nothing is waitable.
  - `si_pid()` is read inside the unsafe block.
  - `kill(-(group as pid_t))`: group is a child pid, never 0 or 1, so it cannot become `kill(0)` or `kill(-1)`.
  - The leader stays unreaped until the group is empty or killed, so the pgid cannot be reused while it is signalled.
  - The `/proc/PID/stat` parse uses `rsplit_once(") ")`, so a comm containing spaces or parentheses is safe. It reads state, skips ppid, then reads pgrp, which is correct.
  - The deadline is re-checked before success, and the post-kill drain is clock-bounded.
- **Dependency.** I found no error. The libc decision on the bead, an exact pin shared with tide-admit and no new crate in the lock, satisfies AP-DEPENDENCY-001 as recorded.

### Lens 2: simplicity and leftovers

- `suite_command` is split out of `suite` cleanly, and `record_and_replay` returns the manifest path.
- I found no dead code.
- The owner-guide range `676-797` matches `record_and_replay` at d077152.
- `run_bounded` is larger than a demo step needs: see N1.
- The leftovers I found are N6 and the report's own stale "No dependency added" line, which `report.md` already marks superseded.

### Lens 3: claims

- **The article.** The survey figures (53.2%, 28.1%, 23.0%), `b.cmp(a)` farthest-first, and RustRover 2026.3's MCP debugger for Codex and Claude Code (start, breakpoints, step, evaluate) all match the condensed article. The author, date and URL match too.
- **rr.** Record and replay under gdb with reverse execution [2], unmodified programs and hardware performance counters [3], chaos mode [4], and `rr pack` with CPU compatibility [5] are each the claim their source makes. The bead's unsourced "large traces, tied to one machine" was correctly dropped.
- **Tide's side.**
  - `--replay` being same-machine for now matches README:27.
  - Half the seeds failing matches the 8 of 16 in `search.log`.
  - The 13 KB manifest is the builder's measurement.
  - "Tide does not replace a debugger" opens the page, which meets acceptance.
- **A1** is met, with N7's wording note.
- **The "just learn debugging" paragraph** opens with "Learn it." It concedes the survey's pain points unchanged and does not argue that Tide fixes them. It names the one thing Tide changes, a reproducible failing run under the debugger's timing. It concedes the deterministic case. That is honest and not defensive, apart from the B1 sentence, which overstates what works today.

### Lens 4: a fresh reader

- A Rust developer on x86_64 Linux with a Python-enabled gdb can run `RUST_GDB=… cargo run -p tide-demo` and get step 5.
- The by-hand block is correct: the `eval "set -- …"` form handles the shell-quoted `replay:` line, which is single-quoted in `record.log`. `by-hand.log` shows it ran verbatim to `replay reproduced`.
- Two things trip a reader. The script exists only after a run in which step 5 did not skip (N5). The page also never shows or points to the script's contents (N5).
- The gaps section prepares the reader for the register reads and the internal function names.

DONE
