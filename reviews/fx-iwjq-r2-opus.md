Model: claude-opus-5-5 · Date: 2026-11-16 · Candidate f093d40

# fx-iwjq Opus review r2 (read-only, reports only)

## Verdict

approve

## Scope

`git diff 3f3d417...f093d40` (SHA-256 4031551d30e7ab85b5d28fd2001d47a5b9f4186e34939d004cd64d44a0feef56 checked), 18 files. I checked it against:

- the bead's Decision, Scope and Acceptance, plus its comments through the 21:50 UTC round-1 note;
- the builder report and the r1 Codex findings;
- AGENTS.md and the GOVERNANCE tier rule.

Read in full:

- `crates/tide-admit/src/walk/reify.rs`;
- the hook in `walk.rs` (`visit_expr` adjustment loop, `visit_nested_body`, `generated_by_toolchain`, `can_select`, `select`);
- `diag.rs`;
- the `driver.rs` fixture swap;
- all 8 CLOSURE-3 cases;
- the design and doc edits: 01 D7 and the tier rows, 02 §2.8, §5.4 and §11.2 R3, 03 RUN-4 hole 5, rules.md, README.md and conformance/README.md.

I checked the rustc internals against the pinned nightly's source (`nightly-2026-08-09` `rustc-src`):

- `implied_bounds_tys` and `implied_outlives_bounds` (`rustc_trait_selection/src/traits/outlives_bounds.rs`);
- `ImpliedOutlivesBounds::perform_query`;
- borrowck's `free_region_relations::create`;
- `assumed_wf_types` (`rustc_ty_utils/src/implied_bounds.rs`);
- borrowck's `ReifyFnPointer` and `ClosureFnPointer` cast handling (the #129021 hack, `type_check/mod.rs`);
- `InferCtxt`'s `Drop`;
- `instantiate_bound_regions_with_erased`, and `TyKind::FnDef`, which in this nightly carries a `Binder` over its args.

I ran no cargo, rustc, tide-admit or conformance command. The suites are the coordinator's. `conformance-2.log` so far shows all 8 CLOSURE-3 cases PASS.

## Blocking findings

None.

## Non-blocking findings

N1. The stated cost leaves out the most common rejections. The design text implies them but never names them.

- **Where:** design/02-memory.md:556-558, conformance/rules.md:58 ("Cost: …"), README.md:69-70 and conformance/README.md:127-128 all name only "generic functions over references such as `first<T>(&[T])`".
- **What is also rejected:**
  - **Every trait method with a reference receiver, on any type.** `reify` checks the trait declaration itself (`items[0]` is the trait item, reify.rs:28). Its `&'s Self` implies `Self: 's`, and `Self` is a type parameter that is not declared `'static`. So `let f: fn(&u8) -> u8 = Clone::clone;` is FX0206 ("implies `Self: '_`"). The same holds for `PartialEq::eq`, `Ord::cmp`, `Iterator::next`, `ToString::to_string` and `Display::fmt`, even though `impl Clone for u8`'s body assumes nothing.
  - **Every `&self` or `&mut self` method of a type with a lifetime parameter**, such as `impl<'a> Lexer<'a> { fn next(&mut self) }`, which implies `'a: 's`.
  - **Non-generic functions over nested references**, such as `fn f(v: &mut Vec<&str>)`.
- **Is it stated?** The rule text ("on a type parameter"; "its argument and return types") covers these literally, so the implementation does not contradict the design. But the one-line cost misleads agents about what will hit FX0206.
- **Recommendation:** name trait methods with `&self` receivers and `&self` methods of lifetime-carrying types in the cost sentence.
- **Later bead (optional):** when selection finds an impl that defines the method, the trait declaration's own bounds are not assumed by the body that runs. Skipping them there is a sound relaxation.
- **Tier:** text only. The rule and the code are correct.

N2. The help text and 02 say that "rustc checks the bounds of a call", which overstates the case for impl-header bounds.

- **Where:** reify.rs:13-14 (`HELP`); 02-memory.md:558-560 ("since rustc checks the bounds of a call").
- **Accurate part:** a call proves the well-formedness that a fn signature's own implied bounds depend on.
- **Overstated part:** a direct call of a method whose impl header implies a bound, such as `|v| <Pair<'x,'y> as Select>::select(v)`, is the rust#100051 family. The builder's report leaves that family open, and RUN-4 hole 5 says such holes stay open.
- **Recommendation:** qualify the sentence, e.g. "rustc checks a call's argument types". Or note in CLOSURE-3 that the closure fix does not close the impl-header direct-call route.
- **Tier:** text only.

N3. Closures coerced to `fn` pointers are not covered. Today the route depends on FX0404, which no case pins.

- **Mechanism:** a closure whose signature is deduced from a higher-ranked expected `fn` type gets that signature's implied bounds in borrowck. `DefiningTy::Closure` inputs feed `free_region_relations::create`. rustc's #129021 WF hack runs only for `ReifyFnPointer`, not `ClosureFnPointer`. A later fn-pointer to fn-pointer subtyping is a plain subtype check.
- **Likely exploit, by reasoning only (not probed):** `let f: for<'a, 'b> fn(&'a &'b (), &'b T) -> &'a T = |_, v| v; let g: fn(&'static &'static (), &'x T) -> &'static T = f;`.
- **Why Tide is safe today:** that shape needs one bound region shared by two parameters. Elision cannot express that, so it needs an explicit `for<…>`, which FX0404 rejects (`walk/traits.rs` `ceiling_parameter`).
- **Recommendation:** pick one of these:
  - add a one-line note to CLOSURE-3 or R3 that the closure route stays closed only while FX0404 holds;
  - add a reject case with that closure, expecting FX0404, so that relaxing FX0404 later trips it.
- **Tier:** outside this bead's scope. It belongs on fx-djp or as a follow-up note.

N4. Two soundness-relevant branches have no conformance case.

- **`RegionSubAlias` (reify.rs:151-153):** this is the only arm that rejects `fn foo<'a, T: Tr>(_: &'a T::Assoc, v: T::Assoc) -> Box<dyn Debug + 'a>`. A rigid projection yields `RegionSubAlias`, not `RegionSubParam`, and the type-parameter exploit carries over at `T::Assoc = &'x u32`. Changing that arm to `None` would pass the corpus.
- **`Selected::Builtin → Ok(None)`:** this branch is untested but harmless.
- **Recommendation:** add an alias variant to `closure_fnptr_type_param_reject.rs`.

N5. The design text says "between two different origins", but `'static: 'a` is exempt.

- **Where:** 02-memory.md:541-546 and conformance/README.md:122-124.
- **What the code does:** it skips `RegionSubRegion(_, 'static)` (reify.rs:146). For example, `fn f<'a>(x: &'a &'static ())` coerces. That is correct, since the bound always holds, and the module doc says so.
- **Gap:** the design text lists only concrete types and `'static`-declared type parameters as the trivial cases.
- **Recommendation:** add "or `'static: 'a`". This is a nit.

N6. Minor points:

- For a trait's default method, the selected impl's header is still checked (reify.rs:89, `map_or(impl_id, …)`). The trait's default body does not assume that header, so this is extra conservatism. It is stated and harmless.
- `f as usize` and `f as *const ()` on such an item are FX0206 too, since the cast reifies first. conformance/README's "an `as` cast" covers this.
- README.md:70 runs well past 100 columns, as line 64 already does.

## Reasoning

**Coverage of the coercion sites.** In this nightly every fn-item to fn-pointer coercion is recorded as `Adjust::Pointer(ReifyFnPointer)` on some HIR expression:

- implicit coercions;
- `as` casts, which `check_cast` runs through `coerce`;
- `if`, `match` and array LUBs, through `CoerceMany`;
- returns, struct fields and arguments.

`Walk` uses `nested_filter::All`, and `visit_nested_body` installs typeck for every body: fns, consts, statics, assoc consts and anon consts. Closures and inline consts share the root's results. So the hook sees every site. `source` tracks each adjustment's input type, so a reify that follows another adjustment still sees the `FnDef`.

`generated_by_toolchain` skips only:

- the test harness;
- `#[tide::main]`;
- `AstPass` expansions;
- the run's own insertions.

User tokens passed through macros keep their own syntax context, so they are not skipped. A `FnDef` reified inside a toolchain-generated block cannot name a user item with a dangerous signature in any std macro I know of.

**Bound computation.** `implied_bound` gathers the same types that borrowck assumes for the body:

- the liberated `fn_sig` inputs and output;
- for an impl method, `assumed_wf_types(impl)`: the self type, or the trait-ref arg types, with duplicates harmless.

It runs them through the same query, with `disable_implied_bounds_hack = false` as `ImpliedOutlivesBounds::perform_query` uses, and that query includes both the unnormalized and the normalized WF args. Borrowck's set is therefore contained in the checker's, so nothing the body may assume is missed.

When the query fails it yields no bounds, but borrowck then gets none for that type either. So the "permissive fallback" in the builder's report is not a hole.

**Late-bound and early-bound regions.** Explicit where-clauses make regions early-bound, and rustc proves them at the path. Only implied bounds can relate late-bound regions, and every non-trivial implied bound is rejected. The `T: 'static` exemption is sound: rustc proves `Tinst: 'static` from the item's predicates (or, for an impl, from selection), and that implies `Tinst: 'a`. The `'static` long-side and reflexive skips are trivially sound.

**Trait methods.** The selected impl's method, or for a default method the impl itself, is checked under identity parameters. When selection finds no impl or several, `select` returns `None` and the coercion is rejected: a param candidate, an object candidate, ambiguity, or a normalization failure. The `Builtin` case checks only the trait signature, which is fine because builtin impls have no header.

**rustc API misuse and panics.**

- `instantiate_bound_regions_with_erased(args)` is correct here: `FnDef`'s args are a `Binder` in this nightly.
- `item_name` on constructors resolves to the parent's name.
- `fn_sig`, `param_env` and `opt_parent` on constructors, foreign fns, trait items and external items all work.
- `implied_outlives_bounds` asserts that there are no non-region inference variables, and identity types have none.
- The local `InferCtxt`'s `Drop` bomb concerns opaque storage only in `PostTypeckUntilBorrowck`, which this code does not use.
- Leftover region constraints registered in the throwaway infcx are dropped harmlessly.

**Cases.** Each reject case carries `//~ FX0206` at the coercion. Removing the check, or the specific branch a case exercises, fails it:

- `routes` covers each route;
- `trait_impl` covers the `selected_impl` Impl branch and the Err branch;
- `header` covers the impl-header types;
- `ctor` covers constructors.

`closure_fnptr_accept.rs::kept` pins the `'static` exemption, and `same` pins the closure fix. The `driver.rs` swap to `std::cmp::max` keeps the backstop's purpose: a generic std item reified at `fn()` that needs `Ord for fn()`. It now implies no bound.

**Leftovers.**

- `Code::ALL` includes FX0206, so `--tide-print-codes` and `run.sh`'s CODES list pick it up.
- No other code list needs it.
- No `AWAITS CHECKER` is left for FX0206.
- The two stale rules.md line references are corrected. Other rows' 02 line references were already stale before this change and are not its leftover.
- reference.md lists only a few FX codes, so leaving it untouched is consistent.

**Design text.**

- D7 records the bead's decision: RUN-3 test and sim sandbox in v0 (it was already in the v0 row via D13), deployed-binary kernel enforcement in v1, and the interim public-claim wording.
- README carries that claim (Codex B2).
- 02 R3 and 03 RUN-4 hole 5 drop the claims this change made stale.
- Nothing goes beyond the bead's scope.
- The widening to type parameters and aliases strengthens the check, and the coordinator recorded and accepted it.

Every finding above is either text-only (N1, N2, N5), outside scope (N3), or a test gap or nit (N4, N6). None shows a wrong verdict for a program the design text decides. So none blocks under the tier rule.

DONE
