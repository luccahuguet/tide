Model: Opus 5.5. Date: 2026-10-21. Round 7 (converge), candidate 43ef2de.

# fx-kw8.2 round 7 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** Round 6's two edits are correct, and I found no false rejections or crashes from them. Auditing the discharge-site table turned up two more false accepts. Both are reachable in safe code without the grant, so both block:

- A trait's associated-type where-clauses were never decided where a use names the type in its generic args.
- A local `Drop` impl's `Pure` bounds were missed in two cases: values a constructor builds when it is passed as a function, and values a method builds from its type argument.

Each fix has a regression in `authority/pure_routes.rs`. The changes are uncommitted in the worktree. This round changed code, so another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- round 6's review;
- all of `pure.rs` and `walk.rs`;
- the diffs to `crates.rs`, `diag.rs`, `main.rs`, `policy.rs`, `tests/driver.rs`, README.md, `conformance/rules.md`, and `design/02-memory.md` / `03-authority-concurrency.md`;
- `authority/pure_routes.rs`.

Probes are in `scratchpad/k2r7/`:
- `p1`–`p3`, `d1`, `d2`: routes;
- `n2`: false rejections;
- `i1`: ill-typed input;
- `probe.sh` and `chk.sh`: runners;
- `sweep.sh`: the E1 sweep;
- `cmp.sh`: the HEAD driver against the fixed driver over every earlier scratch probe.

## Findings and fixes

### F1. Associated-type where-clauses at a use's generic args (false accept, safe code)

**Round 6 got one claim wrong.** Its "Not changed" note said that a value of type `T::Out<Clock>` reaches a body only through a signature or a call that `declared`/`use_generic` decide. That is false.

**The route.** Take `trait Tr { type O<U>: Show + Default where U: Pure; }` and `impl Tr for X { type O<U> = W<U> where U: Pure; }`, with `impl<T: Pure> Show for W<T>`. The impl's where-clause repeats the trait's, so `impl_item_clauses` rightly skips it. Then `fn g<T: Tr>() { <T::O<Clock>>::default().show() }` does the following:
- rustc proves the trait's `Clock: Pure` through the blanket impl, as part of well-formedness;
- `show` goes through the trait's item bound (an alias-bound candidate), so no impl is selected;
- `g::<X>` selects only `X: Tr`, which has no clauses.

So `W<Clock>::show` ran with no check. The projection appeared only in a path's or a method's generic args:
- `<T::O<Clock>>::default()`;
- `let o: T::O<Clock> = Default::default()`;
- `vec![Default::default()]`;
- `s.make::<T::O<Clock>>()`;
- `<T::O<Clock> as Show>::C`.

The type walk never decided the associated type's where-clauses there. Probes `p1.rs` lines 10 and 14 and `p2.rs` lines 11, 14, and 17 were all missed on 43ef2de.

### F2. A local `Drop` impl's `Pure` bounds, for values that no type shows (false accept, safe code)

**First route: a method's type argument.** `maker.make::<Vec<Guard<Clock>>>()`, where `fn make<V: Default>(&self) { let _ = V::default(); }` and `impl<T: Pure> Drop for Guard<T>`, builds and drops a `Guard<Clock>` inside `make`. Method-call generic args never went through a type walk, so `local_drop_impl` was never reached. Earlier scratch probes show no admission miss of this kind. An admission hole would need an impl, and `obligation` follows impls.

**Second route: a constructor as a function.** `local_drop_impl` matched only `DefKind::Struct | Enum | Union`. A constructor `FnDef` (`.map(Guard)`, `.map(E::A)`, `-> impl Fn(Clock) -> Guard<Clock> { Guard }`) therefore skipped its type's `Drop` impl. `clause_obligations` already treats the constructor's where-clauses as the type's. Probes `d1.rs` lines 6–7 and `d2.rs` lines 4–5 were missed on 43ef2de.

### The fix (`walk.rs`)

- **`named`.** `declared`'s walk is split out as `named(tys, span)`: the `Pure` obligations of the ADTs and projections the types name, each through `use_generic(.., admit: false)`. `declared` now only swaps in the owner and calls it. `visit_path`, `visit_qpath`, and `visit_expr`'s type-dependent branch call `named` on the use's generic args. This covers F1, and the first F2 route. It is linear: `use_generic` does not walk its args, so nothing recurses.
- **What I rejected first.** Running `use_ty` on the args moved existing `FX0520` spans from the named item to the enclosing call (`t5/a1.rs` lines 10 and 16, `k2r4/p1.rs` line 29). The cause is that `Diagnostics` keeps the first report per line. So the args get only the `Pure` part. A `use_ty` `Alias` arm for projections turned out redundant once `named` was in place: every probe gave the same results without it, so I removed it.
- **`local_drop_impl`** maps `Ctor(Struct)` to its parent and `Ctor(Variant)` to its grandparent. This covers the second F2 route.
- **Docs.** The module doc and the `local_drop_impl` doc are updated.

**Regressions (`pure_routes.rs`):**
- `supplied<T: Supply>` (F1), with the clean `supplied_data` over `u64`;
- `guarded_inside` (method type argument);
- `guard_clocks` (constructor as a function).

Each isolates its mechanism. The header names the routes. Swapping 43ef2de's `walk.rs` back in makes the case FAIL: it misses lines 69, 82, and 134. With the fix it gives AWAITS CHECKER, for FX0512 only.

## Round 6's edits (reviewed, no change)

- **`opaque_bounds` with `BottomUpFolder`.** It is correct:
  - the hidden type replaces only the opaque at its identity args;
  - nested opaques keep their own checks;
  - projection-equality bounds are skipped, and `instantiate_bound_regions_with_erased` removes the outer binder.
  - It also catches a hidden type that is another fn's opaque with `Item = Checked<T>`: `p2.rs` line 23 normalizes through the alias bound.

  `impl Iterator<Item = impl Deref>` and lifetime-capturing `+ 'a` forms are caught (`p2` lines 24–25). Their `T: Pure` twins are clean.

  **No false rejections** (`n2.rs`):
  - RPIT and RPITIT under impl-level `T: Pure`;
  - `impl Fn` over iterator adapters with closures;
  - `chain`, `zip`, and `flatten`;
  - nested `impl Deref<Target = T>`;
  - a lending GAT;
  - a GAT under `U: Pure`.

  The cost is one `type_of` per opaque, and typeck has already computed it.
- **`impl_item_clauses` for every impl item.** The environment matches rustc's. `normalize_param_env_or_error` elaborates, so an entailed clause counts as repeated. Saving and restoring `seen_pure` is right. Its one gap is a repeated clause, which callers must decide. That is exactly F1, now covered at uses.

## Discharge-site table (additions to round 6's)

| Route | Covered by | Probe |
|---|---|---|
| Associated type's where-clauses at a use's generic args (path, qualified path, method turbofish, associated const) | `named` at the visitors (new) | F (`p1`, `p2`) |
| Drop glue for values a method's type argument makes | `named` → `use_generic` → `local_drop_impl` (new) | F (`pure_routes` `guarded_inside`) |
| Drop glue for values a constructor `FnDef` builds | `local_drop_impl` constructor arm (new) | F (`d1`, `d2`) |
| `IndexMut`/`DerefMut` mutable fixups (`w[0] = 1`, `w[0] += 1`, `**w = 1`, `v.x = 1`, `&mut v.x`) | `type_dependent_def_id` after writeback; adjustments | F (`p3`, all seven) |
| Opaque whose hidden type is another opaque with an `Item =` bound | `opaque_bounds` → normalization through the alias bound | F (`p2` line 23) |

## Not changed (reasons)

- **`pub` items used only inside `pure.rs`** (`NOT_PURE`, `MAX_TYPE_DEPTH`, `NotPure::part`, `implied_by`). Unneeded visibility in a binary crate is a nit. It causes no warning or dead code.
- **`#[derive(Default)]` on a type with `impl<T: Pure> Drop`** is FX0517 at the derive. The derived body builds `Self<T>` for an unbounded `T`. This is consistent with the existing drop-impl rule and has been the behaviour since before this round. Users write the impl with the bound.
- **Projections under a binder in a fn-pointer type** (`for<'a> fn(T::O<&'a Clock>)`) are skipped, because they have escaping bound regions. rustc does not check well-formedness there either, and a value still has to come from a use that is checked.
- **The design text and README Status.** PURE-1 ("wherever a type meets a `Pure`-implying obligation") already covers both fixes. README's Status matches the driver.

## Checked

- **No-FX sweep.** The 11 E1 programs, as lib, `--test`, and `--bin`, get no FX code. Lib and `--test` exit 0, except `backup.rs` with the existing stub-std E0600. `--bin` gives E0601 as before. The same holds for the clean probes `k2r6/n1`, `k2r6/g2`, and `k2r7/n2`.
- **Before/after comparison** (`cmp.sh`, lib and `--test`). It runs every scratch probe (`t`–`t5`, `k2r1`–`k2r7`) and the E1 programs. The only differences are the intended new FX0517 lines in `k2r7` probes. There is no crash or timeout on any probe in either mode, including the ill-typed `i1.rs`: E0107, E0220, E0425, and E0283 inside turbofish projections.
- **Tests.** `CARGO_BUILD_JOBS=4 cargo test --workspace`: 2 + 5 passed, no warnings.
- **Conformance.** `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS, 0 FAIL**. `pure_routes.rs` was re-run after the final rename of its trait.
- **Formatting.** `rustfmt +stable --edition 2024 --check` on the crate's src and tests: clean. `git diff --check`: clean.
- **reference.md:** 24,992 tokens (not edited).

Files changed this round:
- `crates/tide-admit/src/walk.rs`: `named` split from `declared` and called at the three visitors; the constructor arm in `local_drop_impl`; the module doc.
- `conformance/cases/authority/pure_routes.rs`: three regressions plus clean twins, and the header.
