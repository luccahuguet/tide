# Research report: executable contract clauses (fx-mhd)
Drafted by Fable on 2026-11-10; decisions are recorded on fx-mhd.

Date 2026-11-10. Read-only research; nothing in the repository or in `br` was changed, no cargo build was run, and no file under `trials/e1-checked/` was opened.

Evidence labels: **[repo]** is a file I read in this checkout; **[web]** is a page fetched today; **[memory]** is recalled, not re-fetched, and should be confirmed before it is relied on.

## 0. Where the work starts

- Production has no contract macros (`crates/tide-macros/src/lib.rs`). The only implementation is the throwaway spike, `spikes/d18/macros/src/lib.rs`, which the conformance stub std re-exports (`conformance/rustc/std/src/lib.rs:47`) **[repo]**.
- 04 §1.8 CLAUSE-7 lists what the spike lacks: clause classes, a panic inside a clause reported as a violation, step budgets, `loop {}` rewriting, the signed binder's lower bound, entry values of by-value parameters, a reassigned `&mut` parameter (E0521), the blamed span, and clause source text **[repo]**.
- The driver already sets `-Cdebug-assertions=on|off` by profile and `-Coverflow-checks=on` always (`crates/tide-admit/src/invocation.rs:208-220`), and it already identifies std attribute macros by definition in `check_expansions` (`walk.rs:1340-1357`, `is_std_attr`) **[repo]**.
- Tide has no `async` (`FX0601`) and no user `macro_rules!` (reference §2.1), which removes two classes of edge case **[repo]**.

Design 04 is specific on most points. The open choices are mostly about lowering, and several places where the design is silent (section 4).

## 1. Design choices

### 1.1 Capturing the return value: wrapper closure vs. in-body rewrite

**Options.**
- (a) Run the body in an immediately called closure (spike; `contracts` crate; Kani).
- (b) Move the body into a sibling function and call it.
- (c) Keep the body inline and rewrite each `return e` and `e?` into a checked exit (what rustc does).

**Evidence.**
- rustc's `core::contracts` uses (c), but in the compiler: AST lowering installs `contract_ensures` "so we will intercept `return` statements" and wraps each returned expression in `contract_check_ensures` **[web]** <https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast_lowering/src/contract.rs>. It runs after macro expansion, so it sees every `return`. A proc macro does not: syn leaves the arguments of `vec![a?, b?]` or `assert!(f()? > 0)` as raw tokens, so a `?` there would skip the `ensures` check silently.
- The `contracts` crate uses (a): `create_body_closure` builds `let mut run = |args| -> Ret { body }; let ret = run(args);` and renames `self` to `this` inside the body **[web]** <https://gitlab.com/karroffel/contracts/-/blob/master/src/implementation/codegen.rs>.
- Kani began with (b) and moved to (a). Its PR 3363 states the reason: "at a proc-macro level, we cannot determine if we are inside a associated function or not, except in cases where `Self` is used", and sibling functions cannot be added inside a trait impl; "we decided to wrap contract logic using closures instead of functions" **[web]** <https://github.com/model-checking/kani/pull/3363>. The same blocks (b) for Tide: an associated function that uses the impl's `T` but never writes `Self` cannot be called correctly, and a nested fn cannot use outer generics (E0401).
- `core::contracts` itself is not usable. It is a lang experiment (tracking issue 128044, stabilization unchecked) **[web]** <https://github.com/rust-lang/rust/issues/128044>; its checks call `panic_nounwind("failed requires check")`, a fixed message that cannot unwind, so `#[should_panic(expected = "requires violated")]` could never pass **[web]** <https://github.com/rust-lang/rust/blob/master/library/core/src/intrinsics/mod.rs>; and the driver passes `-Zallow-features=` to user crates **[repo]**.
- Dafny's `-testContracts` also generates "a wrapper for each function or method ... `expect` statements for each requires clause, a call to the original, and ... `expect` statements for each `ensures` clause" **[web]** <https://github.com/dafny-lang/dafny/blob/master/docs/DafnyRef/UserGuide.md>.

**Recommendation: (a), the closure, shaped as 04 CLAUSE-4 and CLAUSE-7 already describe.** It is the only option that catches every `return` and `?` without reading macro arguments, and it is the choice two Rust proc-macro implementations converged on. Details:

- Wrap only when the function has an `ensures`. With `requires` only, prepend the check and leave the body alone, as rustc and the `contracts` crate both do. With `#[invariants]` only, rewrite loops and nothing else, so it stays legal on trait-impl methods.
- Pass each `&mut` parameter as a closure parameter fed a reborrow (`&mut *v`). That is CLAUSE-4's rule ("the lowering passes the body a reborrow and reads the caller's reference after it") and it removes the spike's E0521 when the body reassigns the parameter. Erase only the outer reference's named lifetime in the closure parameter type (CLAUSE-7 records E0503 otherwise).
- Let the closure capture by-value and `&T` parameters and `&self`. A capturing, non-`move` closure borrows what the body only reads, so the clause can still read them afterwards, and rustc reports a body that moves one (E0382) without help.
- Route the call through a generic std helper (`__contract::call1(|v: &mut T| -> R { .. }, &mut *v)`) wherever the result may borrow from a reborrowed parameter; the helper's `for<'a> FnOnce(&'a mut T) -> R` bound gives the closure the higher-ranked signature that inference alone does not. CLAUSE-7 already names this need.
- Emit the same code shape in every profile and gate only execution, with a constant. rustc does this deliberately: checks sit under `if contract_checks` "so that contracts can be type checked, even when they are disabled, which avoids them becoming stale" **[web]** (contract.rs, above). 04 §2.6 requires release builds to resolve and type-check clauses, and a `#[cfg]`-removed lowering would make borrow checking differ by profile.

### 1.2 `old(e)`: what is captured, and how

**Options.** Clone implicitly (design); require the user to write the copy (Kani, `contracts`, Prusti); restrict to `Copy`.

**Evidence.**
- Ada: `X'Old` "denotes a constant that is implicitly declared at the beginning of the subprogram body", requires a nonlimited (copyable) type, and its prefix may not mention `'Result` or an enclosing quantifier's loop parameter **[web]** <http://www.ada-auth.org/standards/22rm/html/RM-6-1-1.html>. Tide's CLAUSE-4 has the same three properties.
- The `contracts` crate emits `let __contract_old_N = <expr>;` with no clone; the user writes one **[web]** (codegen.rs). Kani: `old` "is syntax, not a function and implemented as an extraction"; "If you wish to return borrowed content from `old`, make a copy instead (using e.g. `clone()`)"; nested `old` is an error **[web]** <https://model-checking.github.io/kani/rfc/rfcs/0009-function-contracts.html>.
- Prusti: "`old(...)` cannot move a value out from behind a reference", so non-`Copy` needs `*old(&*x)`; that works only because Prusti is static **[web]** <https://viperproject.github.io/prusti-dev/user-guide/syntax.html>. Creusot's snapshots are zero-sized and proof-only **[web]** <https://github.com/creusot-rs/creusot/blob/master/guide/src/snapshots.md>. Neither gives a runtime model for non-`Copy` values.

**Recommendation: keep CLAUSE-4 and lower `old(e)` to `let __tide_old_n = (e).clone();` with method-call syntax, as the spike does.** Method resolution then implements the design's three rules without type information in the macro: a `Copy` value is copied; `old(s)` on `s: &T` with `T: Clone` yields a `T` ("a reference snapshots its referent"); a type that is neither is rejected with E0599, which the conformance case already expects (`clause_old_reject.rs`). Do not write `Clone::clone(&(e))`: that form clones the reference, not the referent.

- Evaluate snapshots after every `requires` check, in clause order (the spike's order), so a precondition guards the snapshot expressions.
- Reject nested `old` and an `old` argument that names `result` or a binder. The latter falls out as E0425 because the snapshot is hoisted out of the binder's scope, which is how Kani's extraction behaves too.
- Scan tokens for `old` not preceded by `.`, as the spike does, so a method named `old` is untouched.

### 1.3 Entry values of other parameters in `ensures`

CLAUSE-4: a `Copy` parameter is copied at entry, a `Clone + Pure + Sync` one cloned, any other is read in place and is `FX0809` if the body assigns, moves or mutably borrows it. The macro has no types.

**Options.** (i) Snapshot nothing and read in place (the spike; wrong when the body changes a `Copy` parameter such as `mut n: u32`). (ii) Clone every named parameter (rejects non-`Clone` types the design allows). (iii) Choose by trait at type-check time, with autoref-based dispatch in a std helper.

**Evidence.** Autoref-based specialization selects among `Copy`, `Clone` and a by-reference fallback on stable Rust, and in generic code it selects by the bounds in scope at the definition **[memory]** <https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md>. That matches the design's "in generic code, if the definition bounds it so (D4)".

**Recommendation: (iii), limited to parameters the `ensures` clauses name outside `old(..)`.** The fallback arm keeps `&p`; rustc then reports a body that moves or mutates `p` as E0505 or E0502, which is the `FX0809` situation. One residue stays with the checker: a `Clone + Sync` type that is not `Pure` gets cloned where the design says "read in place"; the driver's `Pure` decision must flag that clone (CLAUSE-7 already says snapshot choice needs it). If the autoref helper proves fragile, fall back to (i) plus a syntactic copy for primitive and `&T` parameter types, and record the gap on the bead.

### 1.4 `result`

**Evidence.** Kani and rustc pass the return value to the postcondition by reference (`|result: &T|`; `C: Fn(&Ret) -> bool`) **[web]** (Kani RFC; `core/src/contracts.rs` <https://github.com/rust-lang/rust/blob/master/library/core/src/contracts.rs>). Ada's `F'Result` "denotes the return object" **[web]** (RM 6.1.1). The `contracts` crate binds a local named `ret`.

**Recommendation.** Bind `let result = <closure call>;`, evaluate each `ensures` in a non-`move` closure that captures `result` by reference, then return `result`. This gives the reference's rule ("a place the clause cannot move out of: match on `&result` unless it is `Copy`") through rustc's own E0507, and `result == 3` works for `Copy` types with no `*`. Give the closure an explicit `-> R` from the signature, and omit it when the type mentions `impl` (the spike's `mentions_impl`). Create the `result` identifier with call-site hygiene so the clause's token resolves to it. Keep FORM-2 (`FX0805`) for a parameter named `result`.

### 1.5 Quantifiers and `==>`

**Evidence.** Ada defines executable bounded quantifiers: evaluation iterates, a `for all` "is False if the evaluation of any predicate yields False; evaluation ... stops at that point", and an exception from the predicate propagates **[web]** <http://www.ada-auth.org/standards/22rm/html/RM-4-5-8.html>. Creusot and Prusti spell implication `==>` and quantifiers `forall`/`exists`, proof-only **[web]** <https://github.com/creusot-rs/creusot/blob/master/guide/src/pearlite.md>. The spike lowers to `(lo..hi).all(|i: T| ..)` and `.any(..)` **[repo]**.

**Recommendation.** Keep the spike's token lowering (`a ==> b` to `!a || b`, right-associative, lowest precedence; body extends to the end of the group), and change four things:

1. Call std helpers (`__contract::forall(lo..hi, |i: T| p)`, `exists`, and `forall_in`/`exists_in` for `c.contains(&i)`) instead of `Iterator::all`. The helper short-circuits like Ada, counts steps for the budget (1.8), and marks the clause as quantified for the driver.
2. Require `lo <= i` for a signed binder (CLAUSE-1; the spike assumes `0`). The macro can read signedness from the binder's type token.
3. Accept exactly CLAUSE-1's guard shapes. The spike also accepts `i <= hi`; the design does not list it (section 4).
4. Classify syntactically in the macro: a clause with a quantifier, a closure literal, or a call that is not one of the listed std accessors is `sampled`. The macro cannot see that `==` reaches a user `PartialEq`; that case needs the driver (section 4).

### 1.6 `invariant!` and loop rewriting

**Evidence.**
- Dafny defines `for i := lo to hi` as a `while` whose invariant holds at every head with `lo <= i <= hi`, so the index equals `hi` at exit **[web]** <https://dafny.org/latest/DafnyRef/DafnyRef#sec-for-statement>; 04 §2.4 cites this as the source of the next-index rule.
- SPARK lets `Loop_Invariant` appear "anywhere at the top level of a loop" and reasons on the virtual loop that starts and ends at the invariant **[web]** <https://docs.adacore.com/spark2014-docs/html/ug/en/source/assertion_pragmas.html>. Tide fixes the place to the head (FORM-4), which is simpler and already decided.
- Rust defines `for pat in e { body }` as `match IntoIterator::into_iter(e) { mut iter => loop { match Iterator::next(&mut iter) { None => break, Some(pat) => body } } }` **[memory]** <https://doc.rust-lang.org/reference/expressions/loop-expr.html#iterator-loops>.

**Recommendation.** One visitor over the body, skipping nested items and entering closures, as the spike's `Loops` does, with these corrections:

- **`for x in a..b`**: keep the real `Range` and read its public `start` field as the next index: `let mut r = a..b; loop { { let x = r.start; check } match Iterator::next(&mut r) { Some(x) => body, None => break } }`. `Range::next` advances `start` only while `start < end`, so `start` is `a` before the first iteration and `max(a, b)` at exit, exactly the reference's rule, for any step type and with no overflow. The spike's hand-rolled `i += 1` works only for integers.
- **Other `for` loops**: use rustc's `match`-based desugaring. The spike's `let mut it = into_iter(e);` drops temporaries of `e` at the `let`, so `for x in m.lock().iter()` would stop compiling once an invariant is added.
- **`while let P = e`**: lower to `loop { check; match e { P => { body } _ => break } }`, not the spike's `let P = e else { break };`. The `match` keeps the scrutinee's temporaries alive through the body, as `while let` does; `let`-`else` drops them early, which changes when a guard is released.
- **`loop`**: `loop { check; body }`, no exit check (CLAUSE-6). The spike leaves it unrewritten.
- **Debug builds** check a `for` over `a..b` at entry and exit only: guard the head check with `cfg!(test) || first || r.start >= r.end`.
- Keep the unexported `invariant!` form that fails with FORM-4's fix text for an unrewritten use (`FX0806`).

### 1.7 Violations: message, blame, and a panic inside a clause

**Evidence.**
- C++26 (P2900) counts both as violations: "The value that would result from evaluating the predicate is `false`" and "The evaluation of the predicate exits via an exception" **[web]** <https://en.cppreference.com/w/cpp/language/contracts.html>. That is CLAUSE-7's rule. Ada instead lets the predicate's own exception propagate (RM 4.5.8, above).
- Eiffel blames by clause kind: a precondition violation is the client's bug, a postcondition or invariant violation the supplier's, and labelled clauses name which one failed **[web]** <https://www.eiffel.org/doc/eiffel/ET-_Design_by_Contract_(tm),_Assertions_and_Exceptions>.
- Rust's tracking issue lists "Improve error message (include `#[track_caller]`)" as unfinished work **[web]** (issue 128044), which points at the mechanism for blaming the caller.
- `#[should_panic(expected = "..")]` matches a substring of a `&str` or `String` payload **[memory]** <https://doc.rust-lang.org/reference/attributes/testing.html#the-should_panic-attribute>.

**Recommendation.** One std function per kind, for example `::std::__contract::requires(text, function, blamed, || clause)`, that:

- evaluates the closure under `catch_unwind(AssertUnwindSafe(..))`, which std already uses internally (`sync.rs`, `task.rs`); users have none (`FX0701`), so the wrapper must live in std;
- on `false` or a caught panic, panics with a `String` payload: `requires violated: <clause source text>`, then the function name and the blamed span, then `(clause panicked: <message>)` when that is the cause;
- records a structured copy (kind, clause index, function) in a thread-local before panicking (see 1.10).

Blame: put `#[track_caller]` on a function that has a `requires` and make the std helper `#[track_caller]`, so the panic location is the call site, as `Option::unwrap` reports its caller. For `ensures` and invariants, embed the clause's own file, line and column as literals read from the clause span in the macro; do not use `Location::caller()` there, because in a `#[track_caller]` function it would report the function's caller.

Clause text: take the source text of the attribute's parenthesized group (`Span::source_text`), strip the delimiters, and fall back to the token string when it is unavailable. The spike prints tokens, which CLAUSE-7 lists as a difference.

### 1.8 When clauses run: profiles, test builds, classes, budgets

**Evidence.**
- The `contracts` crate has the same three tiers: `debug_*` uses `debug_assert!`, and `test_*` variants "guard the `assert!` with an `if cfg!(test)`" **[web]** <https://docs.rs/contracts>.
- Eiffel's monitoring levels (`no`, `require`, `ensure`, `invariant`, `all`) are compilation options, legitimate because "for a correct system ... assertions will always hold" **[web]** (eiffel.org, above). Ada's `Assertion_Policy` is `Check` or `Ignore` **[web]** <http://www.ada-auth.org/standards/22rm/html/RM-11-4-2.html>. C++26 has `ignore`, `observe`, `enforce`, `quick-enforce`; under `ignore` the predicate is not evaluated, and even a checking semantic may elide side effects ("Increment of `num` might not occur") **[web]** (cppreference, above). Tide's two-level rule is a subset, and CLAUSE-3 removes the side-effect question.
- rustc PR 144438 moved the `contract_checks` test inside the lowered body so that "no contract code is executed when contract-checks are disabled", and reported that disabled contracts then optimize out **[web]** <https://github.com/rust-lang/rust/pull/144438>.

**Recommendation.**
- `checked` clauses: `if cfg!(debug_assertions) { .. }`. The driver's flag already tracks the profile.
- `sampled` clauses: `if cfg!(debug_assertions) && cfg!(test) { .. }`. This is the only test-build signal a proc macro has without a new driver flag; its limit is that a dependency's sampled clauses do not run in another crate's tests (section 4).
- In release, snapshots must not run: bind them as `Option` (`Some` under the constant, else `None`), as the spike does, or compute them inside the gated block.
- Step budget: count in the std quantifier helpers (one step per binder value), reset per clause evaluation. On exhaustion the helper stops, the clause is recorded as unchecked, and no panic follows. 01 says "steps count evaluated operations"; a macro cannot count operations inside a called function, so binder values are the nearest deterministic proxy (section 4).
- Confirm that the production driver admits a toolchain-generated `::core::cfg!` in user crates; `walk.rs` `check_expansions` passes every expansion's macro through `use_def`, and I did not find `cfg!` in the reference's `!` set. The fallback is a pair of `#[cfg(debug_assertions)]` and `#[cfg(not(..))]` `const` items in the generated block, which keeps one code shape.

### 1.9 Clause purity and what admission must see

CLAUSE-3 (`FX0808`) is a checker rule; rustc accepts each violating clause (`conformance/cases/contracts/clause_pure_reject.rs`) **[repo]**. fx-mhd does not own that check, but its lowering decides whether the driver can find clause code after expansion. Clause tokens keep the user's spans, so expansion data alone cannot mark them.

**Evidence.** rustc marks contract locals with `LocalSource::Contract` and calls lang items **[web]** (contract.rs); Kani tags its generated closures with `#[kanitool::..]` attributes **[web]** (PR 3363). Tide cannot register a tool attribute in user crates (`-Zallow-features=`).

**Recommendation.** Make the std helper functions the markers. Every clause body is a closure literal passed to a `::std::__contract::*` function, and every snapshot is a closure passed to `__contract::old` or `__contract::entry`. The driver can then:

- find clause code as the closure argument of a call whose callee is one of those std items, the same way it recognizes std macros by definition today, and apply `Pure + Sync`, authority and `&mut` rules to that body only (fx-kw8.16's clause normalization can hash the same closures);
- find a clause on a trait or trait-impl method (`FX0802`, fx-u65.6) from the expansion record of std's `requires`/`ensures` attribute at that call site. Because the first contract attribute consumes its siblings, only one expansion is recorded per function, which is enough for placement.

For a bodiless trait method the macro should return the item unchanged plus nothing else, and leave the diagnostic to the driver; a `compile_error!` there would carry no `FX` code.

### 1.10 Visibility to generated tests (fx-m72.2)

fx-m72.2 needs three things from this lowering: a `requires` it can evaluate before a call, per-conjunct rejection counts, and a way to tell a rejected input from a failure.

**Evidence.** Kani keeps check and replace variants of each contract inside the function and selects one per harness (PR 3363). 04 §2.4 makes the generator "their caller: it evaluates each call's `requires` first".

**Recommendation.**
- Put clause parsing in one module of `tide-macros` (parse, split top-level `&&` conjuncts, lower, classify), used by the contract attributes now and by the test macros later. The spike's `conjuncts` function is the seed.
- Keep the spike's rule that the first toolchain attribute to expand collects the function's other contract attributes, and state it as the contract between macros: whichever of `#[test]` or `#[requires]` runs first sees the rest as tokens.
- Evaluate each `requires` attribute, and each top-level conjunct inside it, as its own helper call with its own text, so the violation record names the rejecting conjunct. Conjunct order and short-circuiting preserve `&&` semantics.
- Expose the thread-local violation record through a hidden std accessor. A runner can then classify a caught panic as "requires rejected" without parsing the message, while the `String` payload keeps `#[should_panic]` working.
- Emit no companion predicate functions in fx-mhd. Nothing calls them yet, and they would enter the interface the driver hashes.

## 2. Pitfalls

1. **Double evaluation.** Each clause must be evaluated once per check point; evaluating it again to build a message would double the cost and the budget. A parameter that appears in both `old(..)` and bare form gets two snapshots; that is correct, only costly.
2. **Borrow conflicts from `old`.** `old(v.get(0))` holds a borrow across the body and fails (E0500 in the spike, likely E0502 with reborrow parameters). `old(*s)` on a slice does not clone; method resolution copies the `&[T]`, so the user gets a borrow error, not "neither `Copy` nor `Clone`". The conformance cases pin spike error codes (`clause_old_reject.rs`); switching the stub std to production macros can change them.
3. **Reborrow limits.** Reading `*v` after the body fails when the result borrows from `v`. That is the `FX0803` condition, so rustc's error is the fallback until fx-u65.6 lands.
4. **Unwinding.** A body panic skips `ensures` (no result exists); do not add a drop guard that checks on unwind, since a second panic aborts. `catch_unwind` inside the helper prints the inner panic through the hook before the violation message, so stderr shows two messages. A violation inside `Shared::with` poisons as any panic does.
5. **Profile drift.** Any `#[cfg]` that removes generated code changes type and borrow checking by profile. Gate execution only.
6. **`self`.** A closure captures `self`; it cannot take it as a parameter. A `&mut self` method whose result borrows `self` needs the generic-helper form with a renamed receiver, and renaming `self` is unreliable inside macro arguments (`println!("{}", self.x)` is opaque tokens). Parse the fixed `!` forms' arguments as expressions for this rename, or reject that combination with a clear error.
7. **Generics.** The closure sees the function's generics and `Self`, which is why it beats a sibling function. `impl Trait` in return position needs the omitted annotation; `impl Trait` arguments work because captured parameters need no annotation.
8. **`#[track_caller]`** is rejected on `main` and on non-Rust ABIs, and it propagates through nested tracked calls; add it only for `requires`, and never read `Location::caller()` for callee blame.
9. **Attributes and hygiene.** Keep every non-contract attribute in place (`#[test]`, `#[should_panic]`, `#[tide::main]`, docs). Generated locals need mixed-site spans; `result` and the rebound loop index need call-site spans.
10. **Async.** Not applicable: Tide rejects `async` (`FX0601`).

## 3. Cheapest falsifying tests

Each is a toolchain test in the style of `crates/tide-admit/tests/build.rs` (build through the driver, run the binary or test executable) unless noted.

| Recommendation | Test that would falsify it |
| --- | --- |
| 1.1 closure wrapper | An `ensures` function exits through `return`, through `?`, and through `vec![f()?]`; each exit violates the clause, and each must panic with `ensures violated`. |
| 1.1 reborrow parameter | `fn last(mut cur: &mut Node)` reassigns `cur` in a loop with `#[ensures(old(cur.len()) == cur.len())]`: it must compile, and the clause must read the caller's referent. |
| 1.1 one code shape | The same source builds in debug and release; the release binary runs a violating call to completion and prints a marker after it. |
| 1.2 `old` | `push_all` from reference 10.1 passes; a wrong body fails with the clause text; `old(*t)` on a non-`Clone` type fails to build; a `requires` that guards `old(v[0])` on an empty `v` reports `requires`, not `ensures`. |
| 1.3 entry values | `fn f(mut n: u32) -> u32 { n += 1; n }` with `#[ensures(result == n + 1)]` passes (entry value); a body that moves a non-`Clone` parameter named in `ensures` is rejected. |
| 1.4 `result` | `#[ensures(result.len() > 0)]` on a `Vec` result compiles and returns the value; a clause that moves `result` is rejected; `-> impl Iterator` compiles. |
| 1.5 quantifiers | `max_index` from reference 10.1 passes under `--test`; an off-by-one body fails with the `forall` clause text; a signed binder without a lower bound is rejected; an empty range is true for `forall` and false for `exists`. |
| 1.6 loops | `for _ in 0..0 { invariant!(false); }` panics; `for i in 5..2 { invariant!(i == 5); }` passes; a false-at-exit `while` panics; `loop` with a false invariant panics on the first head; `for x in m.lock().iter()` and a `while let` over a guard still compile and run with an invariant added; in a debug non-test build an invariant false only at a middle index of `a..b` does not panic, and under `--test` it does. |
| 1.7 violations | `#[should_panic(expected = "requires violated: xs.len() > 0")]` passes for odd source spacing preserved verbatim; a clause that indexes out of bounds panics with `ensures violated`, not the index message alone; the reported location of a `requires` violation is the caller's line. |
| 1.8 classes and budget | A sampled clause (`forall`) that is false does not panic in a debug binary and does panic in a test executable; a quantifier over a range above the budget neither panics nor passes silently: the unchecked record lists it. |
| 1.9 markers | Driver unit check: for one function with each clause kind, the driver enumerates exactly the clause closures by callee and reports the user's clause line as the span. |
| 1.10 conjuncts | A test calls a function with `#[requires(a > 0 && b > 0)]` violating only `b > 0`; the violation record names the second conjunct, and the payload is still a `String`. |

## 4. Where design/04 and the reference are silent or ambiguous

Each needs a decision recorded on the bead and in 04; none needs a new language rule.

1. **What "test build" means for dependencies.** Reference 10.1 says `tide test` checks sampled clauses "on every call". With `cfg!(test)`, only the crate under test does. Recommend `cfg!(test)` for fx-mhd and listing a dependency's sampled clauses as unchecked; a driver-set switch can widen it later without changing source.
2. **Step unit and default budget.** 01 D10 says steps "count evaluated operations"; no default number appears in 01, 04 or the reference, and 04 §2.4 says only that packages can change it. Recommend one step per quantifier binder value, counted in std, with a fixed default constant recorded in 04, and no manifest key in fx-mhd.
3. **The unchecked report's owner.** CLAUSE-7 says "the report lists every clause a build or test run did not evaluate" but names no producer. Recommend fx-mhd records over-budget clauses in std and exposes them; the runner beads print them.
4. **Sampled invariant schedule.** 04 §2.4 gives iterations 1–64, 128, 256, ...; reference 10.1 does not mention it. Recommend implementing §2.4's schedule, since 04 is the normative chapter.
5. **`a..=b`.** CLAUSE-6 gives the next-index binding to `a..b` only and says any other iterator "cannot name the loop pattern's bindings"; §2.4 calls the missing `a..=b` binding a spike gap. Recommend treating `a..=b` as "any other iterator" in v0: its exit index `b + 1` can overflow.
6. **Guard shapes.** CLAUSE-1 lists `lo <= x && x < hi` and `c.contains(&x)`; the spike also takes `x <= hi`. Recommend rejecting what CLAUSE-1 does not list.
7. **Eager snapshot failure.** CLAUSE-4 makes a panic while snapshotting a violation even when the clause would not have used the value, which is why it advises `old(v.get(10).cloned())`. Ada instead restricts `'Old` in conditionally evaluated positions (RM 6.1.1). The text is clear enough; recommend eager failure at entry, reported as `ensures violated` with the clause text, and one sentence in 04 saying it fires at entry.
8. **Message layout after the prefix.** CLAUSE-7 fixes the prefix and the content (text, function, blamed span), not the layout. Recommend one fixed layout stated in 04, since tests match on the prefix only.
9. **Classes the macro cannot decide.** CLAUSE-5 makes a clause `sampled` when an operator or `Clone` "reaches a user type's impl". That needs types. Recommend the macro classify by syntax and the driver later report a clause it classified too low; until then such a clause runs as `checked`, which evaluates more, never less.
10. **Conformance stub.** `conformance/rustc/std` uses the spike macros and its cases pin spike error codes. fx-mhd's acceptance refers to fx-nkv's production/conformance equivalence; whether the stub switches to `tide-macros` in this bead is not stated. Recommend switching it here and updating the pinned codes in the same change, so one lowering is tested.

## Verification status

Every URL is cited where it is used. Three were recalled, not re-fetched (marked **[memory]**): the autoref-specialization case study, the Rust reference's `for` desugaring, and its `should_panic` rule.

Not verified by building: the `Range.start` lowering, the autoref snapshot helper, `Span::source_text` on the attribute group, and admission of a generated `cfg!`. Each has a test in section 3 or a fallback named where it is recommended.

DONE
