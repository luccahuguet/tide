Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-07

## Verdict

**request_changes — high confidence.** The direction is sound, but grading can be falsified, one suite rejects permitted behavior, and two correctness bugs remain.

## Scope

Reviewed `3fdc020..ff2b251`: all 99 changed files, the required instructions, protocol/tasks, bead comments and Opus decisions, and recorded result JSON.

The recorded execution/ownership portion of Acceptance is satisfied: Rust references are 30/30 green; Tide reaches the available driver interfaces, and its recorded failures map to open prerequisites blocking `fx-ic2`. The diff changes no protocol, task, threshold, or suite freeze.

## Blocking

1. **Writer tests can replace the held-out acceptance suite.**  
   [harness.py:376](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/harness.py:376), [harness.py:198](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/harness.py:198)

   `grade()` places acceptance source in the writable package, runs writer tests, then builds the acceptance executable. A Rust unit test can write `#[test] fn case1() {}` into `package/tests/acceptance.rs`. A candidate whose public functions merely type-check can then pass without those functions executing. Writer tests can also read the suite.

   **Fix:** Run writer tests in a separate tree containing no acceptance source. Build acceptance against an immutable candidate artifact, protect trusted sources from writes, and expose only required executables/fixtures during execution. Add a self-test proving candidate code cannot replace the suite or alter the graded candidate.

2. **B10 case 3 removes Rust’s stated timing slack.**  
   [suites/rust/B10.rs:25](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/suites/rust/B10.rs:25)

   The suite requires `elapsed == Duration::ZERO`. The task explicitly permits Rust elapsed times from the stated time up to, but excluding, another 150 ms. A correct implementation reporting a few microseconds before its first probe fails. Tide retains exact virtual zero, creating an unauthorized disadvantage for Rust.

   **Fix:** Check `calls` and `last_ok` separately, and require `elapsed < Duration::from_millis(150)`.

3. **A missing source file permanently breaks the session.**  
   [harness.py:357](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/harness.py:357)

   `command()` creates `snapshot-01` before reading the candidate, but persists the incremented run count afterwards. If the source is missing, invalid UTF-8, or rejected by the containment check, the directory remains while `runs` stays zero. After the writer repairs the source, the next command fails because `snapshot-01` already exists. The failed call is also omitted from iteration accounting.

   An in-memory mocked check confirmed the missing-file path leaves `runs == 0` after snapshot creation.

   **Fix:** Persist the allocated attempt before reading candidate input, record input failures as failed runs, and allow the next attempt to proceed. Cover deletion followed by repair.

4. **Both B9 references silently lose valid stored data.**  
   [references/rust/B9.rs:19](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/references/rust/B9.rs:19), [references/tide/B9.rs:14](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/references/tide/B9.rs:14)

   B9 permits carriage returns in values. Saving `set("a", "x\r")` writes `a\tx\r\n`, but reopening with `str::lines()` removes the trailing `\r`, returning `"x"`.

   **Fix:** Split records on LF without stripping CR, for example with `split_terminator('\n')`, in both references. Verify the round trip outside the held-out acceptance suite; no task change is needed.

## Non-blocking

- [harness.py:442](/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/trials/e1-checked/harness.py:442): final grading reads live writer source and saves only its result. The `fx-ic2` runner must preserve immutable final bytes and grade those bytes for reproducibility.
- The broker and gate-host isolation canaries remain explicit `fx-ic2` obligations. This preparation CLI alone does not restrict the writer agent’s file tools.

## Reasoning

The six Opus decisions are implemented: approved system/store mounts, driver-only Tide execution, explicit runner and library-boundary failures, the corrected doctest fixture, trusted command dispatch without writer-directory wrappers, and setup-time vendoring.

I found no additional host escape through the inspected command path. The mount policy excludes private host trees, other sessions, and external networking, subject to the approved mounts and pending canaries. Finding 1 nevertheless breaks held-out integrity within the grading tree.

Read-only rendering and reference structure checks passed for all 60 task/arm combinations; `git diff --check` passed. The supplied host results support readiness counts, but green references cannot detect the false-pass and false-rejection cases above.

Not checked: nested bwrap, fresh full execution suites, or host isolation canaries. No files or beads were changed.