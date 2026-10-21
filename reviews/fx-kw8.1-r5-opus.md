# Tide review, fx-kw8.1 round 5 (Opus): converge round

Candidate: HEAD 0d10f52 on 187a22b (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This round judged the shared cause behind rounds 2-4's admission holes, then
searched for more routes. It fixed what it found in the working tree and did not commit.
Probes are in this scratchpad's `t5/` and run through `adm5.sh` (round 3's `adm3.sh`, writing
to `t5/`). `sweep5.sh <name>` runs every probe from rounds 1-5 and every E1 program as lib,
`--test`, and `--bin`, and writes `sweep5-<name>.txt`. `admb.sh` runs HEAD's driver, built from
`git archive HEAD` in `base-admit/`.

## Verdict

fixed

## Shared cause

**Judgment.** Every hole from round 2 to this round has one cause. A generic item is
instantiated, or an obligation is proven, somewhere the walk did not list, and the walk
rebuilds that list from HIR and the typeck tables. Rounds 2-4 each added sites: generated code,
`?`/`for`/autoderef/ADT where-clauses, then supertraits, associated-type bounds, and builtin
impls. This round found two more (F1, F3).

**Would MIR or the mono collector own it better?** I weighed a walk over each local body's MIR
at concrete instances: `Call` and `Drop` terminators, fn-item constants, `Instance::resolve` on
the concrete args. It would see desugarings, autoderef, operators, and drops in local code with
no per-route code. It does not remove the hard part, though:
- **The calls that matter happen inside std's generic code.** Examples are `fmt`'s
  `new_debug::<T>`, `Vec::<T>::drop`, the drops in `Map::count`, and the unsizing inside
  `Box<dyn Error>::from`. A walk that stops at the std boundary still has to follow the
  callee's where-clauses into impls. That is today's `obligation`, with its supertrait and
  builtin-impl rules. A walk that goes into std's MIR sees std's internals: raw pointers,
  atomics, and `core::ptr` everywhere. The allowlist says what user code may name, not what std
  may run, so it cannot apply there.
- **What it would miss or cost:**
  - MIR exists only for crates that type- and borrow-check. Today's gate also gives verdicts on
    crates with rustc errors.
  - An instance's spans point into the generic body, not at the instantiation.
  - Drop glue and shims reach std's private types (hashbrown's tables are `Role::Other`).
  - A vtable built during const evaluation needs the collector's allocation walk.
  - Check builds would build optimized MIR for every reachable item.
- **Fit with D18.** ADMIT-1 states admission at the obligation level ("every path, method call,
  operator, deref, impl selected for an obligation, and macro expansion"). Architecture B can
  implement that and be tested against A. A MIR or monomorphization check is not something B
  can mirror.

**Decision.** The HIR walk with obligation following stays the owner. This round fixes each
route at the shared cause inside the walk. F1 does not special-case `.map(Logged)`. It closes
the whole class "an instantiation whose only trace is a fn item's type", which covers
constructors and generic fns stored in adapter types. No design text changes, because admission
is still checked the same way.

**Follow-up (not started; for the caller to file under fx-kw8):** a route-independent backstop
for denied impls.
- *Evidence.* Every hole from round 2 to round 5 ends in one of a few denied impls:
  - core's `impl<F: FnPtr> Debug/Hash/PartialOrd/Ord/Pointer for F`;
  - `impl Default for RandomState`;
  - the array `From` impls of `HashMap` and `HashSet`.

  All of them are generic or `#[inline]`. So when one can run, its instance is among the local
  crate's mono items, however the code reached it.
- *Design.* In an `after_analysis` callback, which runs only when rustc and the walk found no
  errors, call `tcx.collect_and_partition_mono_items(())`. For each `MonoItem::Fn` that is a
  method of a non-local trait impl, build the impl's key the way `admit_impl` does. Reject it
  only when an explicit `Deny` rule or an ungranted `Unsafe` rule matches. Unlisted keys must
  not count, since std's generic internals instantiate unlisted impls.
  - Report at the use in the nearest local mono item. Find it by walking `mentioned_items`
    backward from the instance, only when there is a rejection, so the common case costs
    nothing. If none is found, report at the crate root and name the instance.
  - Record in ADMIT-1 that this part of the gate runs after type checking.
- *What it subsumes, for code reachable from the mono roots:* round 2's F1 (`Termination` under
  `#[tide::main]` and `#[test]`), round 3's F1, round 4's F1 and F2, and this round's F1 and F3.
  Reading the collector (`rustc_monomorphize/src/collector.rs`, `should_codegen_locally` and
  `collect_roots`), the instance would exist for F1 and F3. I did not build the backstop to
  observe it.
- *Open points for that bead:*
  - forcing mono collection in `--emit metadata` builds costs time, and post-mono const errors
    would start to appear in check builds;
  - `-Zshare-generics` is on by default at opt-level 0. A downstream crate can then link an
    upstream Tide crate's instance instead of instantiating it. Passing `-Zshare-generics=no`
    would need an ADMIT-2 line;
  - in a lib, the roots are only the reachable non-generic items;
  - testing: once the walk is right, no program triggers the backstop alone. The bead has to
    choose between a test that disables the walk's obligation step and the probe evidence.

  I did not start it, for two reasons. It moves part of the gate after analysis, which D18 and
  ADMIT-1 should record first. And no runnable check could show it working this round.

## Findings and fixes

**F1. A constructor passed as a function skipped its type's where-clauses (admission hole).**
`[noop as fn()].into_iter().map(Logged).count()`, with `struct Logged<T: Debug>` and a `Drop`
impl that formats `T`, was admitted at HEAD (`t5/a4.rs:14`). std builds and drops each
`Logged<fn()>`, so a code address is formatted. No expression has that type: `Logged` is a
`FnDef` value, and the `Map` type holds the `FnDef`, not `Logged<fn()>`. `use_generic` skipped
constructors' where-clauses, and `use_ty` checked only a fn item's def. The same held for enum
variant constructors (`Ev::V`) and for `filter_map(f).map(Logged)` (`t5/a5.rs`).
*Fix (`walk.rs`):* `use_ty` runs `use_generic` on every `FnDef` it meets, at the fn item's
args, and `use_generic` no longer skips constructors (fields stay skipped). A fn item's
where-clauses are thus checked wherever its type appears, which is the shared cause, not this
one route. `admit_obligation_routes.rs` gains `mapped()`, marked at line 68. HEAD's driver
misses that mark (`admb.sh`); the fixed one reports it.

**F2. Formatting through a `&mut` reference was rejected (false rejection).** The raw-pointer
pseudo-item patterns `*const T` and `*mut T` begin with `*`, which the glob read as a
wildcard. So they matched every key that ends in `const T` or `mut T`, including
`impl core::fmt::Debug for &mut T`. With `x: &mut u32`, `format!("{:?}", x)` was FX0520 "needs
the `unsafe` grant", and so was `{}` of a `&mut String` (`t5/a6.rs`). By their keys, `&mut T`'s
`Hash`, `Deref`, and `AsRef` impls matched too (not probed). A Rust programmer
writes this constantly. Raw-pointer impls themselves keep the grant through `admit_impl`'s
raw-pointer head check.
*Fix (`policy.rs`):* a leading `*` in a pattern is literal, so `*mut T` matches only the
pseudo-item. The module doc says so. The glob is now `glob`, and `matches` handles the
leading star. The unit test asserts both directions, and its end-anchor check
(`core::*::new` against `..::new_in`) replaces the old leading-star one. `admit_accept.rs` gains
`show(n: &mut u32, s: &mut String)`. HEAD's driver rejects it at 41:5; the fixed one admits it.

**F3. `main`'s return type's `Termination` impl went unchecked (admission hole).**
`fn main() -> Result<(), fn()> { Err(noop) }` was admitted as a binary (`t5/a8.rs`, exit 0).
rustc's `lang_start` reports the error through `Termination::report`, which formats `fn()`.
rustc proves that obligation in its entry-point check, and no expression shows it. Round 2's F1
covered the same route under `#[tide::main]` and `#[test]`, whose expansions call `report`. A
plain `main` is FX0512 (ROOT-1, a later slice), but admission should not depend on another rule.
*Fix (`walk.rs`):* `entry_point` checks `ret: Termination` for a user-written entry fn, in its
typing environment. A `main` the toolchain generates is skipped, since its return type is fixed:
`()` for the test harness, `ExitCode` for `#[tide::main]`, whose body is still walked. Without
the skip, a `#![no_std]` crate under `--test` got a spanless FX0520 for the harness's
`(): Termination` (`t/nostd.rs`). The new case `conformance/cases/authority/admit_main_return.rs`
(ADMIT-1, GLOBAL-4, ROOT-1) marks `fn main` with `FX0512 FX0520`. It awaits the checker for
FX0512, and its FX0520 is enforced. HEAD's driver reports nothing there. The module doc names
the route.

## Not changed

- **The MIR/mono backstop:** see the follow-up above.
- **A where-clause on a concrete type that names a parameter**
  (`fn g<T>(t: fn(T)) -> String where fn(T): Debug`) is admitted where it is defined. Like any
  where-clause, it is checked where `g` is instantiated (`fn(u8): Debug`), so every run of the
  body is checked in some crate. Global ones (`where fn(): Debug`, `where RandomState:
  Default`, on fns and on impls) are rejected in the body, since rustc selects the impl
  (`t5/a7.rs`).
- **Where F1's diagnostic points.** It falls at the start of the method chain (14:5), not at
  `Logged`. This is the existing per-line dedup, which keeps the first report, and the outer
  expression is visited first. `Some(AtomicU64::new(0))` already behaves this way.
- **E0601 order.** `tide-admit` now calls `entry_fn` during the walk, so `main function not found`
  comes before the FX lines when a lib-shaped file is built as a bin. The set of diagnostics is
  unchanged.
- **Routes probed and found closed at HEAD.** Each gave FX0520 at the use:
  - fn items and pointers from denied fns: `let f = AtomicU64::new;`, a typed `fn` pointer,
    `f64::to_bits` and `f64::sin` as values, `[1.0].map(f64::sin)`, and
    `Box<dyn Fn(f64) -> u64>` (`t5/a1.rs`);
  - method references: `<fn() as Debug>::fmt`, `ToString::to_string` mapped over `W<fn()>`;
  - `Default::default()` through `T: Default`, `run(Default::default)`, `unwrap_or_default`,
    `mem::take`, `resize_with`, `collect` into `HashMap`, a derived `Default` holding a
    `RandomState`, and `Keys::default` as a value (`a1`, `a9`, `b1`);
  - `PartialOrd` and `Ord` through `Option`, arrays, `max`, and `cmp` on `fn()`. `==` stays
    admitted;
  - higher-ranked and `extern "C"` fn pointers, `&dyn fmt::Pointer`, and struct-tail unsizing
    `&Two<u8, fn()>` to `&Two<u8, dyn Debug>` (`a5`);
  - autoderef through field access, indexing, and deref coercion at a call (`a4`);
  - generic fns stored as `fn` pointers, and closures coerced to `fn` pointers (`b1`);
  - a const generic argument, a `const`, and a `static` formatted with `{:?}` (`b1`);
  - transitive supertraits, generic supertraits, and a GAT's where-clause (`a9`);
  - `#[ignore = core::include_str!(..)]` and `#[doc = core::env!(..)]` on tests (`a3`);
  - a method with `where Self: Debug` called through `dyn` is not dyn-compatible: E0038
    (`a2`).
- `design/reference.md` is untouched, at 24,992 tokens. ADMIT-1 already says "impl selected
  for an obligation", and the fixes make the walk match it.

## Checked

- Read AGENTS.md, GOVERNANCE.md's Merge Rule, the bead, D18, 03 ADMIT-1 and ADMIT-2, the five
  earlier reviews, and all of `crates/tide-admit/src`. Also read the pinned nightly's
  `rustc_driver_impl` `run_compiler` (when `after_analysis` runs), `rustc_interface` `analysis`
  (it aborts on earlier errors), `rustc_monomorphize` `collector.rs` (roots and local codegen),
  and `rustc_hir_analysis` `check/entry.rs`.
- Probes (`adm5.sh`):
  - at HEAD, F1 (`a4`, `a5`), F2 (`a6`), and F3 (`a8 --bin`) reproduced;
  - after the fixes, each gives the verdict above;
  - round 2's `mainok`/`mainhole` and round 3's `m1`-`m3` and `gen --test` keep their
    verdicts.
- `sweep5.sh final`, 225 runs: rounds 1-4's probes and every E1 program in three modes, plus
  `t5/`. Compared section by section with round 4's `sweep4-final.txt`, the old sections are
  identical once sorted. The only difference is F3's E0601 order. No ICE or panic.
- With HEAD's driver (`admb.sh`), the three changed cases fail as described in F1-F3; with the
  fixed driver they pass.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: 2 + 4 passed.
- `conformance/run.sh`: exit 0, 140 PASS, 58 AWAITS CHECKER (the new case awaits FX0512),
  0 FAIL.
- `rustfmt --edition 2024 --check` (stable) on the crate's sources and tests is clean.
  `git diff --check` is clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992.
- Load stayed below 1, builds ran one at a time, and `CARGO_BUILD_JOBS=4` was used throughout.
