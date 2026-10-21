Model: Opus 5.5. Date: 2026-10-21. Round 8 (converge), candidate 77afe91.

# fx-kw8.2 round 8 (Opus): the Pure check in tide-admit

## Verdict

**clean, with nits fixed.** Round 7's edits are correct. I found no false rejections, double reports, span moves, crashes, or verdict regressions. No route sits next to round 7's fixes. The two fixes are nits with no behaviour change:
- the `pub` items that only `pure.rs` uses are now private;
- `use_generic`'s doc comment was out of date after round 7.

Both are uncommitted in the worktree.

## Scope

Bounded by the coordinator's decision: a confirmation round, not a route hunt. The route-independent closure belongs to fx-kw8.4.

Read:
- AGENTS.md and GOVERNANCE.md;
- the round 6 and 7 reviews;
- all of `walk.rs` and `pure.rs` at 77afe91;
- `diag.rs`;
- the diffs to `crates.rs`, `main.rs`, `policy.rs`, `Cargo.toml`, and `tests/driver.rs`;
- `authority/pure_routes.rs`;
- the README.md, `conformance/rules.md`, `design/02-memory.md`, and `design/03-authority-concurrency.md` diffs.

Probes are in `scratchpad/k2r8/`:
- `n1.rs`: false rejections;
- `p1.rs`: neighbouring routes;
- `i1.rs`: ill-typed input;
- `probe.sh`, `chk.sh`, `sweep.sh`, `cmp.sh`, and `cmpall.sh`: runners;
- `admit-base`: the driver built from this tree;
- `cmp-*.txt` and `conf.txt`: results.

## Findings and fixes

### N1. `pub` items used only in `pure.rs` (nit, fixed)

`NOT_PURE`, `MAX_TYPE_DEPTH`, `NotPure::part`, and `Pure::implied_by` are now private. `grep` shows no use outside the module. `NotPure` stays `pub` because `Pure::check` returns it, and so does its `message`, which `walk.rs` calls.

### N2. `use_generic`'s doc was stale (nit, fixed)

It said `admit` is false "for a type a declaration names". Since round 7, `named` also passes a use's generic args with `admit: false`. The doc also left out the local `Drop` impl's where-clauses, which the function decides. It now reads: "its trait, if it is a trait item, its where-clauses, and those of its local `Drop` impl (`local_drop_impl`). `admit` is false for a type that is only named (`named`) …".

## Round 7's edits (reviewed, no change)

### `named`, split from `declared`

`named` walks every type the arguments name and calls `use_generic(.., admit: false)` for each ADT and each projection. It is called:
- from `visit_path`, on `node_args` or on the lang-item `FnDef` args;
- from `visit_qpath` (type-relative);
- from `visit_expr`'s type-dependent branch (methods, operators, indexing).

It is correct, and it is linear: `use_generic` does not walk its arguments, so nothing recurses.

**Duplicate reports.** `seen_pure` keys on `(trait ref, owner, need)`, so a failure that `use_generic`, `named`, and `use_ty` all meet on one use is reported once, at the first span visited. `Diagnostics` deduplicates by line as well.

**Spans.**
- In `visit_expr`, the type-dependent `named` runs before `use_ty` on the same `e.span`, so it moves no report.
- Comparing round 7's fixed driver with this tree over every probe gives identical output (below).
- Compared with round 7's pre-fix driver, the only differences are the intended new lines. Every earlier line keeps its span.

**Cost.** `named` adds about one `use_generic` per ADT per use's argument list, which the `use_ty` work on expression types already exceeds. Over the 11 E1 programs plus `pure_routes.rs`, the wall time was 0.53 s with this tree's driver and 0.57 s with round 7's pre-fix driver, which is within noise.

**Ill-typed input** (`k2r8/i1.rs`, plus round 7's `i1.rs`). None of these crashes:
- E0107 in a constructor turbofish;
- E0425 for an unknown constructor or an unknown type in a turbofish;
- E0790;
- the E0366 and E0367 forms: a specialized `Drop` impl, `Drop for Twice<A, A>`, an extra `T: Copy` bound, and an unconstrained impl parameter (E0207).

The specialized, repeated-parameter, and unconstrained-parameter impls all return `None` from `local_drop_impl`'s parameter mapping. Errors in `node_args` stop at `pure_obligation`'s `references_error` guard.

### `local_drop_impl`'s constructor arm

- `Ctor(Struct)` maps to its parent.
- `Ctor(Variant)` maps to its grandparent, the enum.

Both are right. A constructor's generics are its ADT's, so `args` line up with the impl's self-type parameters. The arm covers:
- a path (`.map(Guard)`, `Gate::Open`);
- a `FnDef` in an expression type, including a block's type (`pure_routes` line 69 is reported at the body's `{`);
- `Res::SelfCtor` (`.map(Self)`), through `use_ty`'s `FnDef` arm.

### `pure_routes.rs` regressions

`supplied`, `guarded_inside`, and `guard_clocks` each isolate one mechanism, and each has a clean twin (`supplied_data`, `guarded_data`). The header names the routes. The case gives AWAITS CHECKER, for FX0512 only.

## False-rejection check (`k2r8/n1.rs`, lib and `--test`: no FX code)

- Ordinary turbofish: `parse::<u64>`, `sum::<u64>`, `collect::<Vec<_>>()`, and `collect::<Vec<String>>()` with a closure.
- `Default::default()` and `Bag::<Point>::default()` for local types.
- `.map(Some)` and `.map(Wrapper)`, for a struct with no `Drop` impl, at `u64` and at `Clock`.
- `.map(Closer)` and `m.make::<Vec<Closer<Clock>>>()`, where `impl<T> Drop for Closer<T>` has no `Pure` bound.
- `.map(Shape::Dot)`.
- `HashMap::<String, Vec<Point>>::new()`.
- `PhantomData::<Clock>`.
- `size_of::<Wrapper<Clock>>()`.
- `iter::empty::<Closer<Clock>>()`.
- `Option::<Clock>::None`.
- Generic `Vec::<T>::new()` and `Bag::<T>::default()`.
- `.map(Self)` in `impl<T> Wrapper<T>`.
- A lending GAT, through a method and through `Option::<L::Item<'_>>::is_some`.

## Neighbours of round 7's fixes (`k2r8/p1.rs`: all 12 flagged)

**Method-argument (F2) neighbours:**
- a `Drop` type nested in a tuple, an `Option`, or a `Vec` in a method turbofish;
- the type as a trait's generic argument, through UFCS (`<Mk as Use<Vec<Guard<Clock>>>>::go`) and through a path (`Use::<..>::go`);
- an enum variant constructor as a value (`Gate::Open`);
- a unit variant (`Gate::<Clock>::Shut`);
- a constructor coerced to `fn(Clock) -> Guard<Clock>`.

**Associated-type (F1) neighbours:**
- the GAT projection as the self type of a fully qualified path;
- the GAT projection as a trait's argument;
- inside `Some(..)`;
- as an associated function passed as a value (`repeat_with(<T::S<Clock>>::default)`);
- inside a closure;
- through a `None` whose type a `let` fixes, which is reported at the `None` path.

No neighbour was missed, so I added no regression. HIR at this pin has no `QPath::LangItem`, so every lang-item path goes through `visit_path`.

## Not changed (reasons)

- **Where-clause types in signatures.** `declared` does not walk them, for example `fn h<T: Tr, I: Iterator<Item = T::O<Clock>>>`. A value of such a type reaches the body only through a use whose args or expression type are walked: `x.show()` has `node_args` `[T::O<Clock>]` after normalization. So this is not a route on its own. A route that is in a different class goes to fx-kw8.4.
- **fx-kw8.4's class.** That bead owns any remaining route where rustc discharges a local item's `X: Pure` through the blanket impl and no use shows it. It does this with the post-mono check of instantiated predicates. This round found no new one.
- **`size_of::<Guard<Clock>>()` is FX0517** when `impl<T: Pure> Drop for Guard<T>`. Naming the type is rejected, as it already was in fields and signatures before round 7. This is consistent with the drop-impl rule and conservative.
- **The design text and README Status.** I made no edit. README's Status matches the driver: allowlist, structural `Pure` under the D18 codes, FX0901 for the `unsafe` grant, ADMIT-2's flags, and flag ownership still unbuilt. The 02 §9 and 03 (ROOT-4, UNSAFE-1, §3.1 `#[trusted_pure]`, AFREE-1 and AFREE-2) edits match `pure.rs` and `walk.rs`. PURE-1 already covers round 7's fixes.

## Checked

- **Regression comparison** (`cmpall.sh`, lib and `--test`). It covers 186 files: every probe in `t`–`t6`, `cx`, and `k2r1`–`k2r8`, plus the E1 programs.
  - Against round 7's fixed driver: identical.
  - Against round 7's pre-fix driver (43ef2de): additions only. These are the intended FX0517 lines in `k2r7/d1`, `d2`, `p1`, and `p2`, and the `// F`-marked line 13 of `k2r7/i1`.
  - No line is lost, and there is no timeout, panic, or ICE.
- **E1 sweep** (`sweep.sh`): the 11 E1 programs, plus the clean probes `k2r6/n1`, `k2r6/g2`, and `k2r7/n2`, each as lib, `--test`, and `--bin`, get 0 FX codes.
  - Lib and `--test` exit 0, except `backup.rs`, with the existing stub-std E0600.
  - `--bin` gives E0601 for the library programs, as before.
- **Tests:** `CARGO_BUILD_JOBS=4 cargo test --workspace` passed, 2 + 5, with no warnings.
- **Conformance:** `conformance/run.sh` exits 0, with **180 PASS, 28 AWAITS, 0 FAIL**.
- **Formatting:** `rustfmt +stable --edition 2024 --check` on the crate's `src` and `tests` is clean. `git diff --check` is clean, and no line is over 100 columns.
- **reference.md:** 24,992 tokens (not edited).

Files changed this round:
- `crates/tide-admit/src/pure.rs`: `NOT_PURE`, `MAX_TYPE_DEPTH`, `NotPure::part`, and `implied_by` are private.
- `crates/tide-admit/src/walk.rs`: `use_generic`'s doc comment.
