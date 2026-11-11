claude-opus-5-5 · 2026-11-11

# fx-mhd round 5: Opus re-check of the round-4 fixes (read-only)

Candidate `0201144` on base `cd9d47e`. The diff SHA-256 is `4aa1f3606ed81f0dc4ce41634d1e89fc5a1e3b1abc90af47a8d83dab4cfbe7fe`, it matches, and the worktree is clean. I ran the probes against the candidate's `target/debug/tide-admit` and `target/tmp/build/std/artifacts`, one at a time, under nice 19 with CARGO_BUILD_JOBS=4. Load was 1.4 to 4.7, 14 to 15 GiB was available, and no quiet window was active (`quiet-until` 1794414320 is earlier than now, 1794440028). The probes are in `/home/lucca/.cache/eon-agents/tide-fx-mhd-work/r5-opus-probes/`. I made no repository, git or `br` writes and did not open `trials/e1-checked/`.

## Verdict: approve

All 11 round-4 findings are resolved, and none of the fixes introduces a blocking defect. Fix 1, which widens the change, holds up under every adversarial shape I tried. The fixes did introduce three small non-blocking defects, and one instance of finding 4's class is still in the const path (N1–N4 below). Each needs a fix or a rebuttal, but none blocks landing.

## Status of findings 1–11

1. **`track_caller` blame: resolved.**
   - Blame lands on the inner call line in each case: nested requires, recursion (`down(3)` → line 2, inside `down`), after an early `return` and a `?`, a by-value `self` method, a `&mut self` method, and a call inside a body closure (`.map(|x| f(*x))`).
   - A body panic reports the body's own line.
   - Adversarial shapes all compile and give correct results in debug and release:
     - early `return`;
     - `?` on Result and on Option;
     - `<'a, T> … where T: Clone + Debug` returning `&'a T`;
     - `impl Iterator` returns, including ones that borrow from a `&'a mut` input or from `&mut self`;
     - `Result<impl Display, E>` with `?`;
     - `self` by value and `mut self`;
     - `&mut` returns tied to inputs, with named lifetimes, elided lifetimes, pass-through, a `&mut` hidden in a type alias or in a struct, and `Option<&mut _>`;
     - `-> &mut Self` builders;
     - `&mut dyn Write` and `&'a mut dyn Iterator` returns;
     - `T: ?Sized` behind `&mut T`;
     - recursion and `-> !`.
   - `#[requires] #[test] fn(x: u8)` and `#[requires] #[sim_test]` build and run.
   - Release erasure works: violating calls pass in release, and `t_down` reaches the overflow panic.
2. **Struct shorthand: resolved.** `W { n }` and nested `W { n: W { n }.n }` read entry values in both profiles.
3. **Const message used as a format string: resolved.** A const fn with `matches!(p, P { a: 1 })` and a braced `ensures` block compiles. Const evaluation passes, and the runtime violation message keeps the braces literally.
4. **`unused_braces`: resolved on the runtime path.** No warning appears for `#[requires] fn one(x) -> i32 { x }`. The const path still has one instance (N4).
5. **Ensures marker: resolved.**
   - Rustdoc HTML keeps "Runtime doc." and "Const doc." and contains neither `__tide` nor `const_ensures_marker`.
   - The release HIR keeps the `::std::__contract::ensures(...)` call under its gate, and keeps the unconditional `const_ensures_marker()` call in the const fn.
   - `Clause::check` always emits the helper call, sampled clauses included (`contract.rs:94`), so test builds keep the marker.
   - Const requires-only functions return early without the marker (`const_fn.rs:40-43`).
   - The interface is sound for fx-m72.2: it should resolve either definition among the function's own body calls, closures included, and skip nested items. Generated wrappers such as `sim_test` place the user's function as a nested item, so fx-m72.2 has to inspect that inner function.
6. **Unchecked-record dedup: resolved.** Records are now keyed by identity. The unit test passes in `workspace-r5.log`. The new storage causes N3.
7. **Lexical adapter: resolved.**
   - `if !(a ==> b)` and `match !(a ==> b)` parse and evaluate correctly.
   - A real `=` sharing an arrow's position is rejected: `contract_rejects_colliding_generated_operator_spans` passes.
   - `=` after a joint punctuation token (`==`, `<=`, `..=`) is correctly excluded from the collision check.
8. **`todo!`, `unimplemented!`, `unreachable!` captures: resolved.** `unreachable!("{self:?}")` in a `&mut self` method that has ensures compiles and panics with `M(1)` in both profiles.
9. **Const requires blame docs: resolved** (`README.md:172`, `crates/tide-std/src/lib.rs:80-82`).
10. **Design text trim: resolved, and every normative statement survives.**
    - All 10 decisions remain, one sentence each, with no round labels or mechanism narrative.
    - What I checked was dropped from 04 is either covered elsewhere or was not normative:
      - "every return" is in CLAUSE-6;
      - release resolve/type-check is in §2.6 (`04:642`);
      - entry meaning is in CLAUSE-4;
      - blame is in CLAUSE-7;
      - the removed "closure only when the function has an ensures" sentence was mechanism, and fix 1 made it stale.
    - The const and alias limits now live only in the crate docs (`lib.rs:71-88`). README and `conformance/README.md` point there.
11. **Tests: resolved.**
    - `build.rs:13` is one 2×2 matrix looping over three fixtures.
    - `scoped_clauses.rs` and `clause_expressions.rs` replace the round-named fixtures, and no live references to the old names remain.
    - `rules.md:185` is updated.

## New findings

- **N1 (non-blocking): a user-written `#[track_caller]` is now ignored for the body of a requires-only function.**
  - Where: `crates/tide-macros/src/contract.rs:340` and `:396`. The `:244-248` check that skips adding a second `#[track_caller]` shows that user-written ones were anticipated.
  - Failure: for `#[track_caller] #[requires(true)] fn user_tracked() { panic!(..) }`, the panic is now reported at the body (`probe.rs:17:21`), not at the caller. Before fix 1 the body stayed inline, so the user's attribute applied.
  - Ensures-bearing functions already had this behaviour. Tide std uses `#[track_caller]` in `root.rs`, `time.rs` and `fs.rs`, so adding a requires to one of those would silently move where its panics are reported.
  - Fix: when the user already wrote `#[track_caller]`, keep the requires-only body inline, since propagating the caller is what they asked for. Otherwise, state the limit in the crate docs.
- **N2 (non-blocking): every requires-bearing `-> !` function now warns "unreachable expression" at the user's attribute.**
  - Where: `contract.rs:396-398`.
  - Cause: after `let result = call(..)` of type `!`, the trailing `result` is unreachable. Reproduced in both profiles (`probe.rs:38:1` in `shapes`). This is the same class as finding 4.
  - Fix: for a `!` return, emit the `call(..)` as the tail expression, or put an `allow(unreachable_code)` on the generated statements only.
- **N3 (non-blocking): `take_unchecked()` now returns records in a different order on each run.**
  - Where: `crates/tide-std/src/contract.rs:60` and `:70`.
  - Cause: `std::collections::HashMap` with random `RandomState` keys, and `into_values()` follows that order. Before, records came back in insertion order.
  - Tide relies on deterministic replay and gives its own maps fixed keys (`lib.rs:55-56`). The planned printing of unchecked records will consume this order directly.
  - Fix: use a `BTreeMap<ClauseId, _>`, or keep the `Vec` and add a `HashSet<ClauseId>` for the duplicate check.
- **N4 (non-blocking): the const requires-only path still triggers finding 4's `unused_braces` warning.**
  - Where: `crates/tide-macros/src/contract/const_fn.rs:42`.
  - Cause: the body is nested as `{ #(#requires)* #body }`. `#[requires(x > 0)] const fn c(x: i32) -> i32 { x }` warns "unnecessary braces around block return value" at `probe.rs:2:27`.
  - This predates the fix, but it is the same defect finding 4 asked to remove at the user's span.
  - Fix: splice `body.stmts` instead of nesting the block.

DONE
