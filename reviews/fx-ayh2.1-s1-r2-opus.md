Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 107688e (design, docs)

# fx-ayh2.1 stage 1, Opus review round s1-r2

- **Candidate:** `107688e82ddc95e082392902491d9cf0b73525f5` on branch `fx-ayh2.1`, base main `91a2f36`.
- **Diff:** `git diff 91a2f36..HEAD`, SHA-256 `9701b4714bc3511e02950383b1f1384b576112d62eeed30e77263d164f1eaf28` (7 files, +638 −33).
- **Budget:** `scripts/count-tokens.sh design/reference.md` returns **24,985** (base 24,961, so +24), within D14's 25,000. Passes.

## Verdict

**Changes required.** The design is sound, minimal for A1, and consistent in its main owners. Three blocking findings remain. All are small. B2 and B3 are text-only (no re-check needed under the Merge Rule). B1 changes a stage-2 instruction, so I re-check it.

## Scope

I read the bead and its comments, AGENTS.md, GOVERNANCE.md (Merge Rule, admission filter), the whole of research/40, the full diff of 01, 03, reference, companion, conformance/rules.md and research/30, the rest of 03 §9.9 and §1.1 for context, research/30 §8, the red-team re-check 2, and the builder's report. I grepped `design/`, `conformance/` and README.md for leftover signal and process tier text. I checked tide-std and tide-admit for what exists today (`Proc` identity only; `NOT_PURE` already lists the handles; `tide_std::process` already re-exports `ExitCode` and `Termination`). As the brief asked, I did not repeat the Codex pass or the red team's attack on the authority surface. Nothing was compiled.

## Blocking findings

### B1. Stage 2's label instruction would make the reference false for `Proc::exit`, and it misses other current-state text

- research/40-proc-command-v0.md:460-463 tells stage 2 to drop "the `planned` label from `reference.md`'s `Proc` row when the code lands". research/40:83-84 says `Proc::exit` "is not part of this profile (it stays planned)". tide-std has no `exit` (crates/tide-std/src/lib.rs:33, identity only). Followed as written, design/reference.md:343 would list `exit(code: i32) -> !` as built when it isn't.
- §12 also leaves out other text that describes the current state and has to change when the profile ships. The builder's report names one of these, but §12 doesn't: reference.md:511 ("partial: not yet `Rand` or `Proc`"). The others: the companion §20 heading "(planned)", 03 §9.9's "None of it is implemented yet" (03:2300), and README.md:230, :247 and :431 (`Proc` operations listed as not built; "real child processes" listed as not simulated).
- **Minimum fix:** in §12, say that stage 2 moves `planned` from the row onto `exit` (as in "planned: `exit(..)`", the same pattern as the `Net` row) unless stage 2 also builds `exit`. List the other labels stage 2 flips: reference 6.8, the companion §20 heading, 03 §9.9's "not implemented" sentence, and the README rows. Stage 2 recounts the reference after that edit.

### B2. `Signals` is still listed untiered in v0 text, so 03 disagrees with itself and with the companion

The change moves `Proc::signals` and `Signals` to Later. It marks them Later in the 03 §1.1 table (03:98) and the Process handles bullet (03:352), and it removes `Signals::recv` from the companion's receivers list (companion:672-674). Three places still list `Signals` the old way:
- design/03-authority-concurrency.md:43, §0 Terms: `Signals` is in the list of resource handles with no tier. The same list marks "from v1" items, so an unmarked item reads as v0.
- design/03-authority-concurrency.md:389, §1.1: "`File::sync`, `read_owned`, `write_owned`, and `Signals::recv` take `&self`". This is the same list the companion was changed to drop it from, so the two documents now differ.
- design/02-memory.md:883: the same resource-handle list as 03 §0.
- **Minimum fix (text-only):** mark `Signals` and `Signals::recv` "(Later)" in all three places, or drop them as the companion did. `NOT_PURE` can keep `Signals`.

### B3. research/40 calls the owner's decision A1 a "desk decision"

- research/40-proc-command-v0.md:419: "**Keep Proc out of v0** (the 2026-11-18 desk decision A1 reversed)". research/40 §1, the bead, 01:1020, 03:2287 and research/30:10 all record A1 as an **owner** decision that tide-desk relayed. The admission record should not attribute it to the desk.
- **Minimum fix (text-only):** "(what owner decision A1 reversed on 2026-11-18)".

## Non-blocking findings

1. **Cost figure.** research/40:101 estimates "about 40 tokens" in reference.md. The measured cost is 24 (24,961 to 24,985). Give the measured number.
2. **Kernel floor not stated where readers look.** The descriptor sweep fails closed on Linux before 5.11 (`close_range` returns `EINVAL`; research/40:283). So `Proc` never spawns on those kernels, for example RHEL 8 (4.18) or Ubuntu 20.04 GA (5.4). Only the research/40 §5 table says so. Neither 03 PROC-ONLY nor companion 20 nor research/40 §9 Cost mentions it. I recommend stage 2 add an async-signal-safe fallback: loop `fcntl(fd, F_SETFD, FD_CLOEXEC)` from 3 to the soft `RLIMIT_NOFILE`, failing closed only if that loop fails. Otherwise, state "the real backend needs Linux 5.11" in 03 and companion 20.
3. **Unset start-time `PATH`.** PROC-ONLY (research/40:165-172; 03 PROC-ONLY) doesn't say what a bare name resolves to when the process started with no `PATH`. glibc's `execvp` falls back to its default path in that case. Stage 2 would have to choose. I recommend `NotFound`, which follows the "only absolute elements" rule and adds no hidden path. Record that in 03 PROC-ONLY.
4. **Statuses with no code in v0.** research/40:306 and 03:2298 put "signal death" in Later. Yet v0 produces statuses with no code from `kill` (research/40:321-322) and from a model's default broken-pipe death (research/40:328-329). Two things are left for stage 2 to guess. First, what a v0 model may declare for a write to a closed parent end, given that signal handling is Later ("as its model declares"). Second, whether simulated `ExitStatus` `Display` must match the real backend's for these statuses. One sentence would settle both: in v0, statuses with no code come only from `kill` and the default broken-pipe death; a v0 model can only choose between that death and continuing with `BrokenPipe`; and `Display` matches the real backend's.
5. **Seed-witness outcome set.** The bead's acceptance asks for "every permitted finite outcome in the lifecycle subset" with a seed witness and a broken control. Neither research/40 §12 nor PROC-1 names that set. I recommend naming the research/30 §8 rows that fall in v0, with their controls: try_wait stall guard; spawn failure creates no instance; Kill ends the child; Child drop neither kills nor reaps; buffered output after exit; equal-time deadline before a process timer; missing-model coverage; byte-identical replay. Add the outcomes §6 creates: exit before the caller resumes, the kill-versus-exit race, and short I/O sizes.
6. **§12 wording.** research/40:460 says "Stage 2 adds the conformance rows". Stage 1 already added PROC-1 to PROC-3. It should say stage 2 fills their cases.
7. **Option 2's third rejection reason.** research/40:141 rejects Deno-style resolution partly because it "still runs whatever later sits at that path". The chosen design has the same stated limit (research/40:190-191). The first two reasons stand on their own, so drop the third or note that both options share it.
8. **Writable `PATH` directory.** research/40 §5's limit, that whoever can write a directory on the start-time `PATH` chooses the program behind a bare-name entry, has no owner text in 03 PROC-ONLY or companion 20. A clause beside "Whatever is at the canonical path when the child execs runs" would cover it.
9. **ProcessModel spelling.** Model semantics are decided, but apart from `missing()` the builder spelling is provisional. From reference and companion 20 alone, an agent can't yet write a simulated `Proc` test. Stage 2 should record the spelling it picks in 03 §9.9 and add one short simulated-test example to companion 20.

## Reasoning

**1. Admission filter.** The profile is Rust's `std::process` surface that 03 §1.1 already specified for v0. It has 03's two changes: no ambient `Command::new`, and `wait_with_output(&mut self)`. A1 adds no syntax, checker rule or authority kind. `kill` is needed because cancelled `output` and `status` already kill the child (03 §1.1). `try_wait` and `id` keep Rust's shape. `ProcessModel::missing()` is required by the acceptance's missing-program case. The loader-variable refusal and canonical exec come from red-team findings and close real routes. Nothing A1 doesn't need is added. The cost section states the real divergences: the lookup rule, inherited empty and relative `PATH` elements, and the simulator's default stdin. Apart from non-blocking 1 and 2, it is honest. The rejected alternatives hold, with non-blocking 7. In particular, a real backend with no simulated model would leave reference 6.8 with a hole, and argument filtering would read as a sandbox.

**2. Ownership and consistency.** 01 owns the decision: the v0 tier row, the D7 bullet, and D15's v0 and Later split. 03's header, §1.1 table, PROC-ONLY, §4.3 trust-diff line and §9.9 header agree with it, as do the companion's §20 and Appendix A, PROC-1 to PROC-3, and research/30's status line. The PROC-ONLY text in 03 and research/40 matches: check order, the list of 34 glibc names, canonical exec, and argv[0]. The leftover places where `Signals` reads as v0 are B2. "Planned" stays on reference.md's `Proc` row, which is correct until stage 2 ships (AP-CONTRACT-001). B1 makes sure stage 2 removes it correctly.

**3. Learnability.** The reference row points to companion 20 and says the two things that matter most: the child has ambient authority, and an allowed shell is full authority. Companion 20 gives a correct piped example and the default-stdin rule for simulation. It says plainly that `only` "picks the program, not what it does", and lists the steering routes. 03 PROC-ONLY's "What `only` does not bound" and §4.3's "not a sandbox" line make it hard to read `only` as a sandbox. An agent can write correct real-backend code from these two documents. Simulated tests wait on non-blocking 9.

**4. Stage-2 readiness.** PROC-1 to PROC-3 and research/40 §12 can be implemented and checked. Each requirement has a falsifying case: the canonical-exec `/proc/self/cwd` case, the three-descriptor check with an extra inherited descriptor, `TD0501` versus `E0599`, and the item-list test that covers `pub use`. Rust's `CommandExt::arg0` and `pre_exec`, which runs after the standard descriptors are set up and fails the spawn through its error pipe, can implement canonical exec and the sweep with no new mechanism. The gaps stage 2 would otherwise have to fill are B1 and non-blocking 3, 4, 5 and 9.

**5. Budget.** 24,985 of 25,000 tokens. The fix for B1 is a stage-2 edit, recounted then.

DONE
