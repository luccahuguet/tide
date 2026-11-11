Claude Opus 5.5 · 2026-11-11

**Verdict:** R6-1 is fixed and the nit is done. The fix adds no new defect. This tree is clean for Opus to approve landing.

**Status:** Round 7 re-check of the candidate is done, with no findings. I checked HEAD d6edefc: `git diff --binary cd9d47e HEAD | sha256sum` = `dd957317…c41dc006`, which matches. The fix delta `a0a2861..d6edefc` touches 4 files. I made no repository, git or `br` writes. The driver (20:58:12) and the std/macros artifacts (20:59:21) are newer than the changed sources (20:58:01). Each probe ran in both debug and release with `r5-opus-probes/run.sh`, under nice 19 with 4 build jobs, at load 3.6–4.6 with 13 GB available and no quiet window. Probes and logs are in `tide-fx-mhd-work/r7-recheck-opus/` (`builds.log`, `runs2.log`).

1. **R6-1 is fixed.**
   - **`shadow.rs`, unchanged from round 6:** `t_const`, `t_tracked` and `t_plain` now all pass in both profiles (3/3 each). In round 6, the const and tracked cases failed in debug.
   - **The reverse case (`inverse.rs`, with `#![deny(warnings)]`):** the body declares LIMIT as 1000 and the call passes 150.
     - In debug, both `c` and `t` panic with `requires violated: x < LIMIT`, so the check still uses the outer value, 100.
     - The tracked case blames its caller (`probe.rs:9:37`), so `#[track_caller]` still works inside the nested block.
     - Release erases the checks as designed. Both profiles build with no warning, so the nested-block path doesn't trigger `unused_braces`.
   - **Where:** `contract.rs:340-345` and `const_fn.rs:41-47` use the same `Stmt::Item` test. The regression at `build.rs:115` covers both paths in both directions.

2. **The nit is done.** `lib.rs:85-86` now says that, in a user-tracked requires-only function, a failed requires in a function it calls is blamed on the outer caller.

3. **No new defect.**
   - **Macro-expanded items:** the item test only looks at top-level statements, so an item that a macro expands into would be spliced and missed. Admitted code can't produce one, though:
     - A user `macro_rules!` is rejected with FX0101 (my `macro.rs` probe fails to build in both profiles).
     - The only std macros that create items, `thread_local!` and `lazy_static!`, are rejected with FX0502 (`walk.rs:1389`).
     - Every other macro Tide allows (`assert*`, `format`, `vec`, `panic`, `print*`, `write*`, `matches`, `todo`, …) expands to an expression.
   - **`use` and items, at the top level and in a nested block (`usenest.rs`, with `#![deny(warnings)]`):**
     - A top-level `use inner::LIMIT` is a `Stmt::Item`, so the body keeps its own block and the outer LIMIT is the one checked, for both const and tracked.
     - A `use` inside a nested block stays scoped to that block, so it can't reach the clause.
     - A body-local `fn` item works the same way.
     - In debug, all 7 tests pass, including both `_bad` cases, which panic with `requires violated`. In release, the two `_bad` tests fail only because release erases the checks, as designed.
   - **The `unused_braces` hit my first `usenest` draft got:** that came from my own probe code, a user-written `{ x + K }` tail. Plain stable rustc gives the same error on that code without any contract, so it isn't a macro defect, and I removed those cases from the probe.

**New findings:** none.
