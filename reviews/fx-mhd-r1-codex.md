Model: gpt-6.1-sol · Effort: high · Date: 2026-11-11

Verdict: request_changes

Confidence: high. The direction and single production runtime owner are appropriate, but the frozen candidate has observable lowering and spec defects. No override. Opus should wait for the blockers to be resolved and re-checked.

Reviewed HEAD `4bdae8db0701c617e19ba1f63276d3816274b941`, base `cd9d47e`, diff SHA-256 `92150efbb4b362820f1d9056f9f195dd42af88cb0c22954fe63031979bce8e13`. The worktree was clean before and after inspection. Read AGENTS.md, GOVERNANCE.md, the requested beads and comments, the implementation report, the added research report, the relevant specifications, every changed file, and the surrounding macro, driver, test, and task-runtime paths needed for review.

1. **Blocking — the half-open range rewrite shortens endpoint temporary lifetimes, including in release.** Evidence: `crates/tide-macros/src/contract.rs:979` initializes the iterator with a `let`, whereas the other-iterator branch at line 984 uses Rust's lifetime-preserving `match`.

   ```rust
   #[invariants]
   fn f() {
       let values = std::cell::RefCell::new(vec![1]);
       for _ in 0..values.borrow().len() {
           invariant!(true);
           assert!(values.try_borrow_mut().is_err());
       }
   }
   ```

   Driver-built debug and release executables both fail this assertion: the shared borrow is dropped before the loop body. The same loop without the attribute and invariant builds and passes. This changes ordinary program behavior even when release erases every check, contrary to CLAUSE-2 and the erasure contract. **Fix:** preserve the original `for` scrutinee's temporary scope in the range branch too, using the outer `match` principle already used for other iterators. Add this endpoint-guard regression to the debug/release matrix; existing temporary-lifetime tests cover only other iterators and `while let`.

2. **Blocking — admitted macro arguments are not traversed consistently, breaking entry semantics, classification, and body rewriting.** Evidence: `crates/tide-macros/src/contract.rs:621` handles only `vec!` in `Replace`; `Class` at line 648 and `Loops` at line 1017 do not visit expressions inside macro tokens. Conversely, `Receiver::visit_macro_mut` at line 423 uses the unscoped token rewrite at line 438. These are permitted builtins, not user macros. Confirmed through the production driver:

   ```rust
   #[ensures(result == matches!(n, 0))]
   fn f(mut n: u8) -> bool { let r = n == 0; n = 1; r }
   ```

   `f(0)` incorrectly panics: `n` in `matches!` reads its exit binding, rather than its entry value (reference §10.1, CLAUSE-4).

   ```rust
   fn positive(n: u32) -> bool { n > 0 }
   #[ensures(matches!(positive(n), true))]
   fn f(n: u32) {}
   ```

   A debug non-test executable calling `f(0)` panics. The visible user-function call should make this clause sampled and therefore skipped in that build (CLAUSE-5/6). The temporary allowance for operator/Clone classification requiring types does not cover a syntactically visible function call.

   ```rust
   #[invariants]
   fn f() {
       assert!({ for _ in 0..0 { invariant!(false); } true });
   }
   ```

   This valid placement is rejected with FX0806 because the loop is hidden in `assert!`'s tokens. A separate `&mut self` probe with a local `impl` inside an `assert!` block also fails: raw `rename_self` rewrites that nested method's own `&self` into invalid syntax, although the AST receiver visitor deliberately skips nested items.

   **Fix:** traverse the expression-bearing arguments of admitted builtins with binding/item scopes preserved, consistently for parameter discovery/replacement, classification, loop rewriting, and receiver rewriting. Protect these observed cases; token-wide replacement and a special case only for `vec!` do not preserve code semantics.

3. **Blocking — parameter bindings are missed and incorrectly shadowed.** Evidence: `crates/tide-macros/src/contract.rs:266` skips input patterns other than `Pat::Ident`; `scoped` at line 549 and block handling at line 615 use `names` (line 405), which treats every identifier in a pattern as a binding. Two driver-built valid programs incorrectly panic:

   ```rust
   #[ensures(result == n)]
   fn f((mut n,): (u8,)) -> u8 { let r = n; n += 1; r }
   ```

   `f((0,))` never snapshots `n`. Separately:

   ```rust
   struct S { n: u8 }
   #[ensures(match result { S { n: x } => x == n })]
   fn f(mut n: u8) -> S { let value = n; n += 1; S { n: value } }
   ```

   `f(0)` also never snapshots the free `n`: field label `n` in `S { n: x }` is mistaken for a binding shadowing the parameter. That pattern binds only `x`. Both violate the entry-value rule. **Fix:** collect actual bound identifiers from patterns, use that collection for input snapshots and scope exclusion, and cover these two regressions. Do not infer bindings from all token names.

4. **Blocking — implication lowering does not respect Rust expression boundaries.** Evidence: `crates/tide-macros/src/contract.rs:776` recursively lowers a whole token group; `implication` at line 827 treats everything before/after the first arrow as its operands. Match arms have a special boundary, but argument lists and block statements do not.

   ```rust
   fn both(a: bool, b: bool) -> bool { a && b }
   #[ensures(both(true ==> result, true))]
   fn f() -> bool { true }
   ```

   The driver rejects this with E0308/E0061: the comma and second argument become part of the implication's RHS. `#[ensures({ let p = true ==> result; p })]` also fails to parse. Lowest precedence does not permit the operator to absorb an argument separator or a `let` statement. **Fix:** lower within actual expression boundaries, including argument lists and block initializers, retaining the decided right associativity. Add both examples; existing tests cover a whole-clause implication and a match arm but miss these boundaries.

5. **Blocking — mutable-reference aliases receive by-value entry treatment.** Evidence: `crates/tide-macros/src/contract.rs:269` and line 343 recognize mutable inputs only as literal `Type::Reference`; an alias is snapshotted as an ordinary input and omitted from reborrow arguments. This driver probe fails with E0500:

   ```rust
   type Mut<'a> = &'a mut u32;
   #[ensures(*x == 4)]
   fn f(x: Mut<'_>) { *x = 4; }
   ```

   Spelling the parameter directly as `&mut u32` works. Aliases are transparent; §10.1/CLAUSE-4 assigns return-referent semantics according to type, not spelling. The generated live entry borrow prevents a valid mutation. **Fix:** make mutable-reference classification/reborrowing correct for resolved alias types and transparent parenthesized types. Add a driver-level alias regression with a mutation and a postcondition reading the return referent.

6. **Blocking — `old(&T)` silently clones the reference when `T` is not Clone.** Evidence: `crates/tide-macros/src/contract.rs:324` relies solely on `(expr).clone()`. Method lookup falls back to the reference's own Clone implementation when the referent has none. This builds and runs successfully:

   ```rust
   struct Token(u32);
   #[ensures(old(t).0 == result)]
   fn f(t: &Token) -> u32 { t.0 }
   ```

   `Token` is neither Copy nor Clone. Reference §10.1 and CLAUSE-4 require a reference snapshot to snapshot its referent, so this must be rejected, rather than storing `&Token` as an apparent snapshot. The chosen method-syntax mechanism does not meet the chosen observable contract here; the research report's claim that it always yields E0599 is disproved by the probe. **Fix:** enforce referent snapshotting and its Copy/Clone requirement without reference-Clone fallback. Add a non-Clone shared-reference rejection beside the existing non-Clone dereferenced-value case, and retain positive shared-reference/Clone coverage.

7. **Blocking — contract attributes make supported `const fn` definitions ill-formed.** Evidence: `crates/tide-macros/src/contract.rs:77` emits the runtime helper call; `crates/tide-std/src/contract.rs:137` is not const. The smallest checked clause fails through the driver with E0015:

   ```rust
   #[requires(n > 0)]
   const fn f(n: u32) -> u32 { n }
   ```

   Reference §2.3 retains Rust `const fn`, and §10.1 permits these attributes on functions without a const exclusion. This example needs no mutable arguments, quantifiers, or allocation. **Fix:** provide const-compatible lowering for the admitted const surface and protect a contracted const definition used at compile time and runtime. The report/docs must not claim unrestricted function support while this legal combination is rejected; no explicit deferral is recorded in the binding decisions.

8. **Blocking — parentheses change the range invariant's variable scope.** Evidence: `crates/tide-macros/src/contract.rs:970` matches only an immediate `Expr::Range`.

   ```rust
   #[invariants]
   fn f() { for i in (0..3) { invariant!(i <= 3); } }
   ```

   The driver rejects this with E0425 for `i`; removing the parentheses enables the next-index binding. Parentheses do not change this iterator's kind, and §10.1's next-index rule still applies. **Fix:** peel transparent parenthesis/group expressions when recognizing a half-open range; test identical entry/exit behavior with and without parentheses.

9. **Non-blocking — the TLS runner interface loses child-task metadata at scope propagation.** Evidence: `crates/tide-std/src/contract.rs:55` stores violation/unchecked records on the current OS thread. `crates/tide-std/src/task.rs:866` transfers only the panic payload, then line 668 resumes it on the parent. A production `Tasks::scope` probe whose child calls `#[ensures(false)] fn fail()` produces the correct String panic, but a parent unwind guard prints `RECORD None` from `take_violation()`. TLS isolates concurrent libtest threads and follows the explicit bead decision; it does not carry metadata across the existing runtime's panic transport. **Fix wanted:** document the current-thread-only accessor limit for fx-m72.2/.3, and preserve selected violation/unchecked metadata when integrating task outcomes with runners. This does not block this slice: missing metadata can conservatively remain an ordinary failed case, child failures must not be discarded, and runner integration/unchecked printing are explicitly deferred. A global last-violation slot would introduce attribution races and is not an adequate replacement.

Other results: the later fx-mhd decision explicitly supersedes separate predicate functions with per-conjunct helper calls; the implementation follows it. Attribute reordering exposes remaining contracts to test/sim_test. The ensures doc marker and definition-identifiable helper calls provide the recorded consumer hooks. FX0804–0807 are macro-owned diagnostics advertised by the driver; this does not take FX0802/0803 placement ownership from fx-u65.6. Conformance imports production macros and the exact production runtime file, preserving one runtime owner. String panic layout and requires caller blame match the recorded contract on inspected/tested paths. The return wrapper catches normal exits without a Drop-based unwind check; an independent body-panic probe produced only BODY_PANIC, with no ensures violation. A release probe with observable Clone implementations printed only BODY_ONLY: snapshots did not execute. Supplied tests cover useful return, borrowed-result/reborrow, profile, source/blame, quantifier-budget, and sampled-head boundaries, but miss the confirmed failures above.

Validation: focused probes reused the frozen candidate's `target/debug/tide-admit` and `target/tmp/build/std/artifacts`, compiled small programs serially, and ran under nice -n 19, CARGO_BUILD_JOBS=4, timeouts, and closed stdin. Before each compile, checked load < 12, free -g available >= 6 GB, and no active quiet window. Sources and build/run outputs are retained under `/home/lucca/.cache/eon-agents/tide-fx-mhd-work/r1-probes/`; findings state actual results above. The uninstrumented range-loop baseline and release-clone probe passed. All probes were outside the repository; no repository source, ledger, stage, commit, or br write occurred. Final diff identity remained unchanged.

Skipped: no full-workspace build or conformance rerun; the implementer's reported 156 workspace tests and 225 PASS / 20 AWAITS / 0 FAIL are prior proof, not new reviewer runs. No held-out file was opened. Risk: the blockers permit incorrect clause outcomes or reject valid code, and the range-temporary defect changes release body behavior. DONE
