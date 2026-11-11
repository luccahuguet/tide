Opus 5.5 · 2026-11-11

## Verdict: approve

All four notes (N1–N4) are fixed, but the fixes add one new non-blocking defect, R6-1 below: a constant or other item declared inside a function body can change what that function's `requires` clause checks. Fix it or rebut it before landing; a narrow re-check of that fix is enough.

**Candidate checked:** HEAD a0a2861; `git diff --binary cd9d47e HEAD | sha256sum` = `6e84e8e1…bde6`, which matches. The fix delta is `0201144..a0a2861`. The driver and std artifacts are newer than the commit (20:45 vs 20:43).

**How the probes ran:** each probe was compiled and run in both profiles, debug and release, with `r5-opus-probes/run.sh`, under nice 19 with 4 build jobs, at load ≤ 2.0 with 14 GB available and no quiet window. Probes and logs are in `tide-fx-mhd-work/r6-recheck-opus/` (`builds.log`, `runs.log`).

## Notes N1–N4

- **N1, a user-written `#[track_caller]` on a requires-only function: resolved.**
  - In the r5 `blame.rs` probe, `t_user_tracked` now panics at its caller, `probe.rs:30:31`, in both profiles.
  - All other blame lines match round 5.
  - With the attribute order reversed, blame still goes to the caller (`g2` in my new `tracked.rs` probe).
  - Branch check you asked for (`tracked.rs`): `g` is user-tracked, has only a `requires`, and calls `f`, which also has a `requires`.
    - When `f(y)` fails its check, the blame goes to the test's `g(0)` (`19:25`), not to the `f(y)` line inside `g`.
    - With an `ensures` added (`g3`), blame lands on `f(y)` at `17:5`.
    - This is standard Rust behaviour: a `#[track_caller]` function reports panics at its own caller. The user asked for that, and `lib.rs:83-84` documents the split between the two cases. I'm not counting it as a defect.
    - Text-only nit: `lib.rs:84` could say that a failed `requires` in a called function is then blamed on the outer caller. The new regression only covers a bounds panic.
- **N2, the "unreachable expression" warning on `-> !`: resolved.** My new `never.rs` probe has `#![deny(warnings)]` and three diverging functions: requires-only, requires plus ensures, and user-tracked. All three build in both profiles, and the tests panic with the expected messages. Release erases the `requires` as designed. `shapes.rs` builds with no warning, both as a binary and as a test, in both profiles.
- **N3, the order of `take_unchecked()`: resolved.** `contract.rs:60/70/120-123` keeps a `Vec` in first-seen order plus a `HashSet` for dedup, and `seen` is cleared on take. The unit test that asserts the full order passes (`workspace-r6.log:122`).
- **N4, `unused_braces` on const functions with only `requires`: resolved.** `constreq.rs` builds with no warning, both as a binary and as a test, in both profiles.

## New finding

**R6-1 — minor, non-blocking; fix or rebut. Spliced body items capture the names a `requires` clause uses.**

- **Where:** `crates/tide-macros/src/contract.rs:339-341` (the N1 branch) and `crates/tide-macros/src/contract/const_fn.rs:41-42` (N4).
- **Cause:**
  - Both fixes now put the body's statements in the same block as the `requires` checks.
  - In Rust, an item declared in a block (a `const`, `fn`, `use` or `struct`) is visible everywhere in that block, including before its declaration.
  - So a body item with the same name as one the clause refers to now replaces it in the clause.
  - Before this round, both paths kept the body in its own block, and the default path still does.
- **Reproduced (`shadow.rs`):**
  - Setup: an outer `const LIMIT: i32 = 100`, a clause `#[requires(x < LIMIT)]`, and a body declaring `const LIMIT: i32 = 1`.
  - Called with 50, the const function `c` and the user-tracked `t` both fail in debug with `requires violated: x < LIMIT`, because the check compared against 1.
  - The same function with neither change (`plain`) passes, and release passes because it erases the checks.
  - The reverse also happens: a real violation can be hidden. This silently splits debug from release.
- **Suggested fix:**
  - Splice only when `body.stmts` has no `Stmt::Item`; otherwise keep `{ #(#requires)* #body }`.
  - `unused_braces` only fires when the inner block is a single expression, so a block containing an item can't trigger it.
  - Add a debug-profile regression with a body-local item that shadows one the clause names, covering both the const and the user-tracked case.
