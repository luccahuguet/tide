Model: Opus 5.5. Date: 2026-10-21. Round 5 (converge), candidate fe30fa5.

# fx-kw8.2 round 5 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** Round 4's `local_drop_impl` is correct for what it covers. I found two more false accepts in the same class, both in safe code with no grant, and one driver crash. Each fix has a regression. The changes are uncommitted in the worktree. This round changed code, so another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- round 4's review, and rounds 2 and 3 for context;
- all of `pure.rs` and `walk.rs` at fe30fa5;
- `authority/pure_routes.rs`, the stub std's `marker`/`tide`/`prelude` modules, and the `#[tide::main]` and `#[test]` shims;
- rustc's `compare_impl_item.rs` at the pinned nightly;
- 02 PURE-1.

I reviewed `local_drop_impl` first, as asked. I then checked every implicit route the brief lists against `follow()`, and finished with a bounded hunt over the whole change.

## Findings and fixes

### F1. A trait impl method's own `Pure` where-clause was never checked (false accept, safe code)

- **What:** rustc's E0276 check proves an impl method's where-clauses from the impl's where-clauses and the trait method's own. std's blanket impl meets any `T: Pure`, so rustc accepts the following:
  - `impl<T> Show for W<T> { fn show(&self) where T: Pure { needs(&self.0) } }`;
  - `impl<T> Drop for Gen<T> { fn drop(&mut self) where T: Pure { .. } }`.

  A call `W(clock).show()` selects the impl, and `pure_obligation` follows only the impl's where-clauses, never the method's. The same holds for a call through `dyn Show`, and for drop glue. So the method body could rely on `T: Pure` for a `Clock`. The `drop` form also gets past round 4's `local_drop_impl`, which checks only the impl's clauses.
- **Probe:** `k2r5/m1.rs`. On fe30fa5 it exits 0 with no FX code.
- **Fix:** `walk.rs` gets a new `impl_method_clauses`, called from `trait_impl` for each method of a local trait impl. It checks at the definition, as rustc does:
  - It builds the environment that rustc's `compare_method_clause_entailment` builds: the impl's clauses plus the trait method's own clauses, rebased onto the impl. The env is normalized.
  - It decides each of the method's own where-clauses with `pure_obligation` in that environment. A new `item_env` field overrides `pure_env`.
  - A clause that repeats one in the environment holds as written. Without this, an impl that restates the trait's `where Self: Pure` was falsely rejected (`k2r5/n1.rs`).
  - The dedup set is saved and restored around the check, so its results never stand in for the method body's own environment.
  - Associated types are left out, because a use normalizes to the type's value, which `use_ty` checks.
- **Regression:** `authority/pure_routes.rs` gains `Logged<T>`:
  - a `Drop::drop` with `where T: Pure`, marked `//~ FX0517`;
  - a `Show` impl whose `show` adds `where T: Pure`, marked `//~ FX0517`;
  - a `show_pure` that repeats the trait's `where Self: Pure`, unmarked and clean.

### F2. `#[tide::main]` and `#[test]` hid a `Termination` impl's `Pure` bound (false accept, safe code)

- **What:** `pure_obligation` reported through `flag`, and `flag` drops anything under the toolchain's expansions. The code those expansions generate calls `Termination::report` (`#[tide::main]`) and `assert_test_result` (`#[test]`) on the user's result type. With `impl<T: Pure> Debug for W<T>`, both of these passed:
  - `#[tide::main] fn main() -> Result<(), W<Clock>>`;
  - `#[test] fn t() -> Result<(), W<Clock>>`.

  The plain `fn main` form was already caught, by `entry_point`.
- **Probes:** `k2r5/t1.rs` (bin) and `t3.rs` (`--test`) exit 0 on fe30fa5. `t2.rs`, the plain-main control, gives FX0517.
- **Fix:** `pure_obligation` now reports through a new `report`, which `flag` also uses after its toolchain test. This mirrors what `obligation` already does for admission, with the same reason: the user's types choose the impl. The failure is reported at the attribute, the outermost call site. `parallel_closure`, `io_error_payload`, and the item rules keep `flag`.
- **Checked for false rejections:** the 11 E1 programs get no FX code under `--test` or as lib/bin. The only `Pure` obligations in harness code are ones the user's types bring.
- **Regression:** `authority/pure_routes.rs` gains `#[test] fn checked_test() -> Checked<Clock>`, with `//~ FX0517` on the `#[test]` line. It fires only in the `--test` pass.

All three marks in F1 and F2 were confirmed to FAIL on fe30fa5's `walk.rs`. With the fix, the case is `AWAITS CHECKER` for its existing FX0512 only.

### F3. Driver ICE on an impl item whose generics do not match the trait's (E0049)

- **What:** `trait_impl` rebases the impl item's identity args onto the trait item and instantiates the trait item's bounds with them. This is the associated-type loop that this bead added, and F1's loop does the same. When the impl item has fewer generics than the trait's (`type A = u8;` for `type A<V>`), the instantiation panics with "type parameter out of range" and exit 101. The driver runs on crates with rustc errors, so this is reachable.
- **Probe:** `k2r5/i3.rs` panics on fe30fa5. `i1.rs`/`i2.rs` cover the method forms after F1.
- **Fix:** `trait_impl` skips an item when `tcx.compare_impl_item(local)` is `Err`. That is rustc's own cached verdict, and rustc has already reported the mismatch.
- **Regression:** a new driver test, `impl_items_that_do_not_match_their_trait_are_left_to_rustc`, in `crates/tide-admit/tests/driver.rs`. It runs on Rust's std, where the crash also happens. It fails on fe30fa5 with the ICE and passes with the fix.

## Not changed (reasons)

`local_drop_impl` (round 4) is correct for what it covers. Probes `k2r5/d1.rs` (enum and union `Drop` impls), round 4's `p4.rs`, and the items below each get FX0517:
- a reordered impl;
- a const-generic impl;
- `Vec<Gen<Clock>>` and `Option<Box<Gen<Clock>>>` (`ty.walk()` reaches them);
- `fn mk<T>(t: T) -> Gen<T>`, and a `T: Pure` helper called with a `Clock`;
- `struct Outer<T> { g: Gen<T> }` (by `declared`);
- a type alias.

The ordinary forms stay clean. E0366 and E0207 forms return `None` without a crash. Its cost is one cached `adt_destructor` query per local ADT use.

The rest of the same class is already covered:
- **Deref:** `follow()` covers the autoderef and deref-coercion forms.
- **Recorded method calls:** `use_generic` covers `Index`/`IndexMut`, `DerefMut`, operators, compound assignment, and unary `-`.
- **Desugarings:** the lang-item paths cover `for`/`Iterator` and `?`.
- **`Clone` and derives:** method and path uses cover these. A derived impl over a `Pure`-bounded impl is flagged at the derive.
- **Coercions:** `follow` on the coercion covers `dyn` methods.
- **`Fn`:** closures' impls are builtin, and implementing `Fn` for user types is feature-gated.
- **`Termination`:** `entry_point` covers a plain `main`, and F2 covers the generated entry points.

Probe `k2r5/o1.rs` gets FX0517 for each of the `Index`, deref, and operator routes above.

The cost of F1's check is one fail-closed restriction. A where-clause such as `W<T>: Pure` on an ADT type is not used as an assumption, because `Pure::check` honours bounds only on parameters and projections (PURE-1's last bullet). The check applies that rule to impl methods as the rest of the walk does.

No design text changed. PURE-1 already says the checker decides `Pure` "wherever a type meets a `Pure`-implying obligation". An impl method's where-clauses and a generated entry point's `Termination` are such obligations.

## Checked

- **Tests:** `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 5 passed (the new driver test included), no warnings.
- **Conformance:** `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS, 0 FAIL**. The total is unchanged because the regressions extend `pure_routes.rs`.
- **Formatting:** `rustfmt +stable --edition 2024 --check` on `crates/tide-admit/src/*.rs` and `tests/*.rs`: clean. `git diff --check`: clean.
- **reference.md:** `scripts/count-tokens.sh design/reference.md` gives 24,992 (not edited).
- **E1 trial programs:** all 11, as lib/bin and with `--test`, show no FX code. Twenty runs exit 0. `backup.rs` exits 1 with the stub-std E0600 that was already there.
- **No false rejections** (`k2r5/n1.rs`, clean):
  - impl methods whose clauses match the trait method's (`U: Pure`, `U: P` with `P: Pure`);
  - a restated `where Self: Pure`;
  - `Drop::drop` with `where T: Pure` on a `T: Pure` impl;
  - a trait-level parameter's clause.
- **No crash** on erroneous impls (`k2r5/i1.rs`, `i2.rs`, `i4.rs`): E0049, E0405 in where-clauses, and E0367 on `Drop`.

Files changed this round:
- `crates/tide-admit/src/walk.rs`: `impl_method_clauses`, `item_env`, `report`, the `compare_impl_item` guard in `trait_impl`, and the module and `trait_impl`/`pure_obligation` doc comments;
- `crates/tide-admit/tests/driver.rs`: the F3 test;
- `conformance/cases/authority/pure_routes.rs`: the F1 and F2 regressions and the header.

Probes: `scratchpad/k2r5/` (`t1`–`t3`, `m1`, `n1`, `d1`, `o1`, `i1`–`i4`, `probe.sh`, `conf2.txt`).
