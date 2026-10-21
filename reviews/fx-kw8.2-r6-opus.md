Model: Opus 5.5. Date: 2026-10-21. Round 6 (converge), candidate 101fef2.

# fx-kw8.2 round 6 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** Round 5's edits are correct, and I found no false rejections from them. The systematic pass found two more false accepts in the same class, both reachable in safe code with no grant. In each, rustc meets a `Pure` clause through std's blanket impl, and a use then relies on it through a trait bound without selecting an impl. Each fix has a regression in `authority/pure_routes.rs`. The changes are uncommitted in the worktree. This round changed code, so another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- rounds 4 and 5;
- all of `pure.rs`, and all of `walk.rs` at 101fef2;
- `authority/pure_routes.rs`;
- rustc's `compare_impl_item.rs` (`compare_impl_ty`, `compare_type_clause_entailment`, `compare_impl_const`) and `ty/fold.rs` at the pinned nightly.

First I reviewed round 5's three edits: `impl_method_clauses` with `item_env`, `report`, and the `compare_impl_item` guard. Then I enumerated where a local item can assume `X: Pure`, and where rustc discharges each assumption. I read `follow`, `use_generic`, `clause_obligations`, `pure_obligation`, `trait_impl`, `local_drop_impl`, `impl_method_clauses`, and `opaque_bounds` against that list, and probed every route. I finished with a short hunt for false rejections and crashes.

**The assumption sources.** rustc's param-env holds only these, elaborated with supertraits:
- an item's `predicates_of`, including its parents' and the impl's;
- a trait's where-clauses and supertraits, inside the trait;
- associated-type item bounds, through projections.

Implied bounds from argument types are outlives-only, so a struct's `T: Pure` is never assumed by code that names the struct. A `struct S<T: Pure>` gives `impl<T> S<T>` nothing. Every discharge is therefore one of two kinds:
- an obligation `X: Trait` that selects a local impl, and so needs that impl's where-clauses;
- the where-clauses of a used item at its args.

The question for each route is whether the driver sees that obligation, or else checks the assumption at the definition.

## Discharge-site table

Probes are in `scratchpad/k2r6/`. "F" means FX0517 at the expected line.

| Route (where rustc discharges an assumed `X: Pure`) | Covered by | Probe result |
|---|---|---|
| Call or path of a fn, assoc fn, or ctor (own + parent where-clauses) | `visit_path`/`visit_qpath` → `use_generic` → `clause_obligations` | F (`r2` `cnst`, `wc_use`; rounds 4–5) |
| Fn item as a value, fn-pointer coercion | `use_ty` `FnDef` → `use_generic` | F (`r2` `FNP`, `FNS`) |
| Method call, through autoref or UFCS | `visit_expr` `type_dependent_def_id` → `use_generic` (trait ref + clauses) | F (`r1` `default_method`, `to_string`; `r2` `by_ref`) |
| Operators, compound assignment, `==`, `Index`, unary | `type_dependent_def_id` | F (`r1` `eq`; round 5 `o1`) |
| Autoderef, deref coercion | adjustments → `follow(Deref/DerefMut)` | F (`pure_routes` `method`/`field`/`coercion`) |
| Unsizing to `dyn Trait` (`Box`, `&`, `Pin<Box<_>>`) | adjustments → `unsized_trait_ref` → `follow` | F (`r1` `dyn_display`; round 4 `p1`) |
| Supertraits of an impl'd trait (vtable, `dyn Sub`) | `trait_impl` → `where_clauses(trait)` | F (`r2` `impl Sub for C<T>`) |
| Impl where-clauses reached through another impl (`?` via `From`, `Into`, `collect`, `extend`, `HashMap` `K: Hash`) | `pure_obligation` follows `select` recursively | F (`r1` `question`, `into`, `iter_sum`; `r2` `vec_extend`) |
| `for` (`IntoIterator`/`Iterator`), `?` (`Try`/`FromResidual`) | lang-item paths, args from the path's `FnDef` type | F (`r2` `for_loop`; `r1` `question`) |
| `format!`/`Display` through macros | expansion paths → clauses | F (`r1` `display`) |
| `Default`, `Clone`, `to_vec`, `cloned`, `unwrap_or_default` (std generic code calling a local impl) | clause recursion | F (`r1` lines 24–30) |
| Derived impls over `Pure`-bounded impls | derived body checked in the derive's env | F (round 4) |
| Associated consts, in expressions, patterns, and array lengths | path/qpath `node_args` (anon-const bodies are visited) | F (`r1` `assoc_const`, `pat_const`, `arr_len`) |
| Const and static initializers, closures | nested bodies visited with their owner's env | F (`r2` `FNP`, `FNS`, `clos`) |
| Default method bodies | trait-level clauses at the impl (`trait_impl`), method clauses at calls | F (`r1` `default_method`) |
| Trait where-clauses and associated-type bounds (`where Self::A: Pure`, `type A: Pure`) | `trait_impl`: `where_clauses(trait)` and `explicit_item_bounds` | F (`r2` `impl H`, `impl J`) |
| Associated type value relying on impl where-clauses, used through `T: Tr` | the impl's selection at the caller's `T: Tr2` | F (`g1` `h2`) |
| Trait impl method's own where-clauses (calls, `dyn`, `Drop::drop`) | `impl_method_clauses` (round 5) → `impl_item_clauses` | F (`pure_routes` `Logged`) |
| **Trait impl GAT's own where-clauses** (`type Out<U> = W<U> where U: Pure`, used through `T::Out<U>: Show`) | **none at 101fef2**; now `impl_item_clauses` for every impl item | **missed on 101fef2** (`g1` line 12); F after the fix |
| Trait GAT's own where-clauses at a projection | `declared` on signatures → `use_generic` on the projection | F (values reach a body only through a signature) |
| Drop glue | `use_generic` → `local_drop_impl`; `ty.walk()` reaches closure upvars, `FnDef` args, containers | F (round 4 `p4`, round 5 `d1`) |
| `Termination` (plain `main`, `#[tide::main]`, `#[test]`) | `entry_point`; `report` under toolchain expansions | F (`pure_routes`) |
| Opaque type's bounds on itself (`-> impl Show`) | `opaque_bounds` | F (`o4` `pair`) |
| **Opaque type's bounds on its associated types** (`-> impl Iterator<Item: Show>`, in an RPIT or an impl's RPITIT) | **none at 101fef2** (`opaque_bounds` followed only bounds whose self type is the opaque); now every bound, with the hidden type folded in | **missed on 101fef2** (`o1` lines 9 and 19, `o2` lines 8 and 14); F after the fix |
| Refined RPITIT in an impl (`-> IntoIter<W<T>>`) | `trait_impl` synthetic item → `explicit_item_bounds` | F (`o1`/`o2` line 19 or 24) |
| `Fn` traits | builtin for closures, fn items, and fn pointers; user impls are feature-gated | n/a |
| `CoerceUnsized`, `DispatchFromDyn`, `deref_patterns`, specialization, `generic_const_items` | feature-gated (`-Zallow-features=`) | n/a |
| Safe marker impls (`Copy`, `Unpin`, `Eq`) | run no code. `Eq` is still followed when a clause needs it | not a route |
| `Send`/`Sync` user impls | `unsafe impl`, so they need the grant (FX0901); followed when selected | grant-only |

## Findings and fixes

### F1. An opaque type's bounds on its associated types were never checked (false accept, safe code)

- **What:** `opaque_bounds` decided only the bounds whose self type is the opaque itself. For `fn it<T>(t: T) -> impl Iterator<Item: Show>` with hidden type `option::IntoIter<W<T>>` and `impl<T: Pure> Show for W<T>`, rustc proves `W<T>: Show` at the definition through the blanket `Pure` impl. A caller's `x.show()` meets `<opaque as Iterator>::Item: Show` through the opaque's own bound. That is an alias-bound candidate, so `select` returns `None`, and `W<Clock>::show` ran with no `Pure` check anywhere.
- **Scope of the miss:** a plain RPIT, and an impl's RPITIT that is itself opaque. The refined RPITIT form was already caught through `trait_impl`.
- **Probes on 101fef2:** `o1.rs`, with a `Clock` in a concrete fn, and `o2.rs`, with an unbounded `T`. Both missed every opaque line.
- **Fix:** `opaque_bounds` now decides every trait bound of the opaque. It folds the hidden type in for the opaque type (`ty::BottomUpFolder`), as rustc's `check_opaque_meets_bounds` does. `<IntoIter<W<T>> as Iterator>::Item: Show` then normalizes in `pure_obligation` to `W<T>: Show`, which selects the impl. The fold replaces the old `on_itself` special case, so the function is shorter.
- **No false rejections** (`o3.rs`, `n1.rs`, `g2.rs`):
  - `impl Iterator<Item: Show>` over a `T: Pure`;
  - `Item: Display + '_` and `Item: Debug`;
  - `Item = &'a T`;
  - `Item: AsRef<str>`;
  - nested `impl Show`;
  - `impl Fn() -> T + Pure`;
  - RPITIT with `T: Pure` in either position.
- **Regression:** `pure_routes.rs` `checked_all<T>(..) -> impl Iterator<Item: Deref>`, marked `//~ FX0517`, and the unmarked `checked_data` over `u64`.

### F2. A trait impl's associated type could bound a parameter by `Pure` beyond the trait (false accept, safe code)

- **What:** round 5's `impl_method_clauses` ran for methods only, and its comment said an associated type's clauses "need no check: a use normalizes to its value". That holds for concrete uses, not for generic ones. Take `trait Tr { type O<U>: Show; fn mk<U>(u: U) -> Self::O<U>; }` with `impl Tr for X { type O<U> = W<U> where U: Pure; fn mk<U>(u: U) -> W<U> { W(u) } }`:
  - rustc's `compare_type_clause_entailment` (E0276) proves `U: Pure` through the blanket impl;
  - `trait_impl` checks the `Show` bound in the GAT's own environment, where `U: Pure` holds;
  - `fn g<T: Tr>(c: Clock) { T::mk(c).show() }` meets `T::O<Clock>: Show` through the trait's item bound, and `g::<X>` selects only `X: Tr`, which has no clauses.

  So `W<Clock>::show` ran unchecked.
- **Probe on 101fef2:** `g1.rs` line 12 was missed.
- **Fix:** `trait_impl` now calls the check for every impl item. rustc's `compare_impl_item` checks types and consts with the same hybrid environment as methods (impl clauses plus the trait item's own clauses). The function is renamed `impl_item_clauses`, and its doc comment, the `item_env` doc, the `trait_impl` doc, and the module doc are updated. Consts have no own clauses on stable (`generic_const_items` is gated), and RPITIT synthetic items go through it without error. Rustc applies `compare_impl_ty` to those items too.
- **No false rejections** (`g2.rs`):
  - a GAT whose `where U: Pure` repeats the trait's;
  - `where Self: 'a` lending GATs;
  - a `U: Pure` bound in the GAT's generics on both sides;
  - impl-level `T: Pure` used in the GAT's value.
- **Regression:** `pure_routes.rs` `trait Wrap { type Out<U>: Deref; }` with `impl Wrap for Holder { type Out<U> = Checked<U> where U: Pure; //~ FX0517 }`.

**Both regressions FAIL on 101fef2's `walk.rs`:** I swapped in HEAD's `walk.rs` from a scratch copy, rebuilt, and ran the case, which reported FAIL. Both pass with the fix. The case header names the two new routes.

### Round 5's edits (reviewed, no change)

- **`impl_method_clauses` (now `impl_item_clauses`):** its environment matches rustc's `compare_method_clause_entailment`/`compare_type_clause_entailment`, normalized and elaborated. The repeated-clause skip compares erased clauses against the normalized environment. An unnormalized own clause that does not match falls through to `pure_obligation`, which normalizes it and decides correctly (`<X as Tr>::A: Pure` with `A = T`, `T: Pure`). Saving and restoring `seen_pure` keeps its results from leaking into the item's own checks.
- **`report`:** this is right for `pure_obligation`. The `#[test]` and `#[tide::main]` glue only meets obligations on the user's result type, and the E1 programs stay clean under `--test`.
- **`compare_impl_item` guard:** it is right. With F2, `impl_item_clauses` now runs for associated types too, and the guard also covers E0049 on GATs (`i1.rs`).

## Not changed (reasons)

- **Consumers still reject `-> impl Iterator<Item: Pure>` (round 4, fail-closed).** With F1, the definition now checks the hidden item type. A later change could therefore let `Pure::check` trust an opaque's associated-type `Pure` bounds. That is a feature, not a fix for this bead.
- **Expression types in bodies: `use_ty`'s `Alias` arm only admits the projection.** It does not decide the trait GAT's where-clauses at that projection. A value of such a type reaches a body only through a signature or a call, and `declared` and `use_generic` decide those, so no route is open. I kept the arm as it is.
- **Safe marker impls with `Pure` bounds (`Copy`, `Unpin`):** they run no code, so nothing relies on the bound.

## Checked

- **False-rejection hunt:**
  - the 11 E1 programs as lib, bin, and `--test` get no FX code;
  - lib and `--test` exit 0 except `backup.rs`, which has the existing stub-std E0600;
  - `--bin` exits 1 with E0601 (no `main`) for the library programs, as before;
  - ordinary generic code (`n1.rs`: iterator-returning fns, `impl Fn`, `HashMap` counting, `Display` impls, lending GATs, a GAT mapper, an RPITIT `impl Display`, `Box<dyn Iterator>`) is clean.
- **No crash or hang** on ill-typed code:
  - `i1.rs`: E0049 on GATs, E0405 in an opaque bound and in a GAT where-clause;
  - `i2.rs`: an opaque whose hidden type is not an iterator, E0277 hidden item, E0282 recursive opaque, E0277/E0276 GAT clauses;
  - round 5's `k2r5/i1`–`i4.rs`.
- **Tests:** `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 5 passed, no warnings (after `touch`, then again on the final tree).
- **Conformance:** `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS, 0 FAIL**. The total is unchanged because the regressions extend `pure_routes.rs`.
- **Formatting:** `rustfmt +stable --edition 2024 --check` on `crates/tide-admit/src/*.rs` and `tests/*.rs`: clean. `git diff --check`: clean.
- **reference.md:** `scripts/count-tokens.sh design/reference.md` gives 24,992 (not edited).
- **Design text:** unchanged. PURE-1 already says the checker decides `Pure` wherever a type meets a `Pure`-implying obligation, and says an opaque type's bounds hold of its hidden type.

Files changed this round:
- `crates/tide-admit/src/walk.rs`:
  - `opaque_bounds` folds the hidden type into every bound;
  - `impl_method_clauses` is renamed `impl_item_clauses` and runs for every impl item;
  - the doc comments are updated, and the over-long module-doc line is rewrapped.
- `conformance/cases/authority/pure_routes.rs`: the F1 and F2 regressions and the header.

Probes: `scratchpad/k2r6/` (`o1`–`o4`, `g1`, `g2`, `r1`–`r3`, `i1`, `i2`, `n1`, `probe.sh`, `chk.sh`, `conf-final.txt`).
