Model: Opus 5.5. Date: 2026-10-21. Round 4 (converge), candidate bc6bb45.

# fx-kw8.2 round 4 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** The coordinator's text edit was not the only change. I found one false accept that safe code reaches, with no grant: a local `Drop` impl can bound a type parameter by `Pure` where the type does not, and nothing checked that bound where the type is used. The fix is in `walk.rs`, with a regression in `authority/pure_routes.rs`. The coordinator's ROOT-4 edit is also applied. All changes are uncommitted in the worktree. This round changed code, so another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- round 3's review;
- `git show bc6bb45`: all of `pure.rs`, and `walk.rs`'s obligation, `Pure`, raw-place, parallel-closure, `io::Error`, opaque, item, and `trait_impl` code;
- `authority/root_zeroed.rs`, `pure_routes.rs`, `pure_declared.rs`, and `pure_std_types.rs`;
- 03 ROOT-4, and 02 §9 (PURE-1) and OPAQUE-2.

Round 3's edits:
- FORGES's four new entries and their indices;
- the `root_zeroed.rs` lines;
- the ROOT-4 list.

I then ran a bounded hunt over the whole change, mostly for safe-code false accepts.

## Findings and fixes

### C1. Coordinator's edit: ROOT-4's contract is now a finite list

- **03 ROOT-4:** "by any path to them" now reads: Tide's std bounds by `Pure` the type that "each std operation listed here produces, under any path that names it". The list is unchanged, regrouped with semicolons.
- **Raw-pointer reads:** they are now defined as a deref, a read method of a raw pointer or `NonNull`, or a conversion to a reference or an owner. Before, they were "every route to a `T` from a raw pointer", which also claimed the list was closed.
- **Closing sentence:** it keeps the unsafe-forging sentence and now opens with "The bound is defense in depth over this list, not a closed guarantee".
- **The same wording elsewhere:** `root_zeroed.rs`'s header ("under any path that names them") and `pure::FORGES`'s doc comment ("wherever a use names the function", replacing "wherever the function is reached from").
- **Check:** every operation the text lists is in FORGES or in `raw_place`. FORGES also covers more, which fails closed.

### F1. A `Drop` impl's `Pure` bound was never checked (false accept in safe code)

- **What:** rustc proves a `Drop` impl's where-clauses from the type's own (E0367), and std's blanket impl meets any `T: Pure`. So rustc accepts `impl<T: Pure> Drop for Guard<T>` on `struct Guard<T>(T)`. The same goes for a bound by a trait with `Pure` as a supertrait and a blanket impl.
  - Drop glue runs that impl with no obligation any expression shows, so the drop body could rely on `T: Pure` for any `T`.
  - `walk.rs`'s `use_ty` comment assumed an ADT's where-clauses were "all a `Drop` impl's bounds can be". That is false for `Pure`.
- **Probe:** `k2r4/p3.rs`, with no grant.
  - The program: `impl<T: Pure> Drop for Gen<T> { fn drop(&mut self) { needs(&self.0) } }`, then `let _ = Gen(clock);`.
  - It exited 0 with no FX code.
- **Fix:** `walk.rs` gets a new helper, `local_drop_impl`.
  - It finds a local ADT's `Drop` impl through `adt_destructor`.
  - It maps the use's args onto the impl's parameters through the impl's self type. E0366 makes that self type the ADT at distinct parameters, though not always in order.
  - `use_generic` then runs `clause_obligations` on the impl, with `admit` false, so only the `Pure` part is checked.
  - Because the check is in `use_generic`, it covers types in expressions and patterns (`use_ty`) and types that declarations name (`declared`), as the ADT's own where-clauses are covered.
  - I corrected the `use_ty` comment.
- **Probe after the fix:** `k2r4/p4.rs`. Each of these is FX0517:
  - `Gen(clock)`;
  - `Gen(t)` with `T` unbounded;
  - a permuted impl `impl<'a, B: P, A> Drop for Two<'a, A, B>`, flagged only where `B` is a `Clock`;
  - a const-generic impl;
  - a field `g: Gen<Clock>`.

  `Gen(1u8)` and a `Drop` impl with no bound stay clean.
- **Design:** no text change. PURE-1 already says the checker decides `Pure` "wherever a type meets a `Pure`-implying obligation", and drop glue meets this one.
- **Regression:** `authority/pure_routes.rs`.
  - The header now names the `Drop` route.
  - New items: `Guard<T>` with `impl<T: Pure> Drop`; `Guard(clock)` marked `//~ FX0517`; and an unmarked `Guard(ticks)`.
  - With the mark removed, the case FAILs with the new FX0517 at that line. With the mark, it passes. The case stays `AWAITS CHECKER` only for its existing FX0512.

## Not changed (reasons)

- **Unsafe-only forging routes, non-blocking per the coordinator's decision.** Each needs the `unsafe` grant. Each duplicates or moves an existing value, or is UB, rather than reading data as a new `T`.
  - `ManuallyDrop::take`: it reads out a value the `ManuallyDrop` still holds.
  - `core::ptr::copy_nonoverlapping`, `ptr::copy`, and `ptr::swap`: these write raw memory, which ROOT-4 already names as the residual hole.
  - `Option::unwrap_unchecked`: round 3 covered it; it is UB-only.

  I found none reachable without the grant. Rust's `alloc::sync::Arc::assume_init` is not reachable: `std::sync::Arc` is Tide's type, which has no `new_uninit`, and Tide's `Arc` is not `Pure` anyway.
- **An associated-type bound on an opaque type (`-> impl Iterator<Item: Pure>`) is rejected at its uses, even with a `Pure` hidden item (`k2r4/p9.rs`).**
  - The definition side does not check the hidden item type. This is not a false accept: every consumer decides `<opaque as Iterator>::Item: Pure` through `Pure::check`, which never trusts the opaque's item bounds and so rejects.
  - This is a fail-closed false rejection of an uncommon form. `Item = u8` and `Item = T` normalize and work.
  - It agrees with PURE-1's text: a projection is `Pure` through a bound in scope, and an opaque type's bounds give `Pure` to the opaque type itself.
  - Supporting it needs coordinated changes on both sides (the opaque's item bounds in `walk`'s projection arm, and the hidden type in `opaque_bounds`). That belongs to a later change if needed.
- **Admission of a `Drop` impl's extra non-`Pure` bounds.** I left these out (the fix passes `admit` false):
  - the only bounds that can go beyond a type's are ones blanket impls meet (`Borrow<T>`, `From<T>`, `Pure`);
  - the admission rule is outside this bead.

## Checked

Safe-code false-accept hunt, all flagged correctly (`k2r4/p1.rs`, `p2.rs`, `p10.rs`):
- unsizing to a `Pure`-supertrait `dyn`: `Box`, `Rc`, `&`, `as`, and array elements;
- inherent-impl where-clauses: method, constructor, and function-pointer value;
- fn items as values (`needs::<Clock>`, and `.map(needs)`);
- HRTB clauses (`for<'a> &'a T: Pure`);
- associated-type bounds in a trait and on a where-clause;
- a method-level `T: Pure`;
- an `impl Fn + Pure` return;
- closures capturing by reference and by move;
- nested tuples, arrays, and options;
- a blanket `impl<T> Q for W<T>` with `Q: Pure`;
- `Show::show` through autoref, UFCS, and fn value;
- struct, tuple-struct, and enum constructors, including unit variants;
- `Index`, `Add`, `==`, `format!`, `for`, `clone`, and a derived `Clone`.

Gates:
- `#![feature(unsafe_binders)]` is refused (E0725).
- `#[trusted_pure]` without the grant is FX0901.

Round 3's edit (`k2r4/p7.rs`, `p8.rs` under the grant):
- **FX0517:** `read_unaligned`, `read_volatile`, `as_ref_unchecked`, `Vec::from_raw_parts` of a cast pointer, `assume_init_read`, `ptr::replace`, and `(*q)[0]` of a `*const [Fs; 1]`.
- **FORGES:** the patterns and indices match the pinned sources as round 3 recorded.
- **`root_zeroed`:** the case passes, its `vec!` and `align_to::<u64>` lines stay clean, and its unmarked lines are unchanged.

E1 trial programs:
- All 11, as a library and with `--test`, on the fixed driver show no FX code.
- Twenty runs exit 0. `backup.rs` exits 1 with the existing stub-std E0600.

Final tree:
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 4 passed, 0 warnings (after a `touch`, to force a rebuild).
- `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS, 0 FAIL**.
- `rustfmt +stable --edition 2024 --check crates/tide-admit/src/*.rs`: clean. `git diff --check`: clean.
- `design/reference.md`: 24,992 tokens (not edited).

Files changed this round:
- `design/03-authority-concurrency.md` (ROOT-4, the coordinator's edit);
- `conformance/cases/authority/root_zeroed.rs` (header wording);
- `crates/tide-admit/src/pure.rs` (FORGES doc comment);
- `crates/tide-admit/src/walk.rs` (`local_drop_impl`, its call in `use_generic`, and the `use_ty` comment);
- `conformance/cases/authority/pure_routes.rs` (the F1 regression).

Probes: `scratchpad/k2r4/p1.rs` to `p10.rs`, `probe.sh`, and `run.txt`.
