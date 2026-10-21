Model: Opus 5.5. Date: 2026-10-21. Round 3 (converge), candidate c17b2db.

# fx-kw8.2 round 3 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** I found one false accept. ROOT-4's forging list (`pure::FORGES`) missed four stable, admitted routes that turn raw memory or reinterpreted bytes into a `T`, so each one minted an `Fs` under the grant with no diagnostic. The fix adds them to FORGES, names them in 03 ROOT-4, and adds a regression to `authority/root_zeroed.rs`. I also rewrapped one overlong line in 03 §3.1, a text-only nit. The changes are uncommitted in the worktree. This round changed code, so another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- all three earlier reviews;
- `git show c17b2db`: all of `pure.rs` and `walk.rs`, plus `policy.rs`, the `key` function in `crates.rs`, and the design and README diffs;
- the new and extended cases: `root_zeroed`, `unsafe_trusted_pure_granted`, `pure_routes`, `pure_std_types`, `pure_declared`, `pure_recursive_changing`, `afree_const_mut`, `afree_pure_nested`, and `lock_closure_formatter`;
- 03 ROOT-4, §3.1, §3.2 AFREE-1/2, and 02 §9;
- the pinned nightly's sources for `*const T`/`*mut T`/`NonNull` (`as_*`, `read*`, `replace`), `MaybeUninit`, `Box`/`Rc`'s `MaybeUninit` impls, `slice::align_to`, `Vec::set_len`, and the `vec!` macro.

Round 2's edits got the most attention:
- FORGES and its type-argument indexing;
- `raw_place` with its assign and raw-borrow exemptions;
- union field reads by expression and by pattern;
- the trusted-type instantiation rule;
- the ROOT-4 and §3.1 text.

## Findings and fixes

### F1. ROOT-4 missed four forging routes (false accept)

- **What:** ROOT-4 says Tide's std bounds what `zeroed`, `transmute`, and raw-pointer reads make by `Pure`, "by any path to them". All of the following are stable and admitted (`core::slice::*`, `alloc::boxed::*`, `alloc::rc::*`, `alloc::vec::*`), and FORGES missed each one:
  - `Box::<MaybeUninit<T>>::assume_init` and `Box::<[MaybeUninit<T>]>::assume_init`;
  - `Rc`'s two `assume_init` forms. These are `MaybeUninit::assume_init` through an owner, and are implemented as `from_raw`.
  - `<[T]>::align_to::<U>` and `align_to_mut`, which reinterpret a slice's bytes as `U`s, as `transmute` does;
  - `Vec::set_len`, which turns the buffer's raw spare capacity into elements, as `assume_init` does.
- **Probe:** `k2r3/p1.rs` under the grant. `align_to::<Fs>`, `align_to_mut::<Fs>`, `Box::<Fs>::new_uninit().assume_init()`, `new_zeroed().assume_init()`, `Box::<[Fs]>::new_uninit_slice(1).assume_init()`, `Rc::<Fs>::new_uninit().assume_init()`, and `Vec::<Fs>::with_capacity(1)` + `set_len(1)` were all accepted with no FX code.
- **Fix:** I added five FORGES entries to `pure.rs`, with indices checked against the pinned sources:
  - `alloc::boxed::Box::<*>::assume_init` (0);
  - `alloc::rc::*::assume_init` (0);
  - `alloc::vec::Vec::<T*>::set_len` (0);
  - `core::slice::<impl [T]>::align_to*` (1, the method's `U` after the impl's `T`).

  The `Box` pattern matches only the method. It does not match `alloc::boxed::box_assume_init_into_vec_unsafe`, which `vec![a, b]` expands to, so `vec!` of capabilities stays clean.
- **Checked and left out:**
  - `<[MaybeUninit<T>]>::assume_init_ref`/`_mut` are stable, but the allowlist does not admit them (FX0520), so they are unreachable. A pattern I tried for them was removed.
  - `Option`/`Result::unwrap_unchecked` make a `T` only on a path that is UB, never by reinterpreting bytes.
- **Recorded:** 03 ROOT-4 now names the `Box` and `Rc` forms of `assume_init`, `slice::align_to` (it reinterprets bytes as `transmute` does), and `Vec::set_len` (it makes a buffer's raw memory into elements). It also states the cost: generic unsafe code that sets a `Vec<T>`'s length bounds `T` by `Pure`, as code that reads through `*const T` already does.
- **Regression:** `authority/root_zeroed.rs`:
  - `//~ FX0517` on `Box::new_zeroed().assume_init()`, `Rc::<[Fs]>::new_uninit_slice(1).assume_init()`, `v.set_len(1)`, and `align_to::<Fs>()`;
  - unmarked lines that must stay clean: `align_to::<u64>()` of bytes, and `vec![fs, other]`.

  Each marked line was accepted before the fix (p1). The case passes after it.

### N1 (nit). An overlong prose line in 03 §3.1

Round 2 left the sentence "...as wide as what the reviewer approved. `Pure` is deep: it looks through every" on one unwrapped line. I rewrapped the paragraph and changed no words.

## Not changed (reasons)

- **A `const fn` type parameter instantiated with `&mut` passes AFREE-2. This is correct per the design.**
  - AFREE-2 states its conditions on the `const fn`'s parameter types at the definition: `Pure + Sync`, and no `&mut`.
  - It says a type parameter "needs the bound at the definition", and "Memoization rejects nothing: the compiler memoizes only calls that meet the same conditions."
  - `const fn f<T: Pure + Sync>(t: T)` meets every condition at its definition. `&mut u8` is `Pure + Sync`, so the call is well-bounded.
  - The no-`&mut` condition cannot be stated as a bound. By the memoization sentence, such a call is simply not memoized, and nothing is rejected.
- **Fail-closed rejections, all reachable only under the grant:**
  - reading a `Pure` field through `(*p).field` of a non-`Pure` pointee (bounded by the whole pointee, as ROOT-4's `(*p).field` says);
  - `&raw const (*p)[i]`, since the index is not a field projection;
  - a `ref` binding in a union pattern, which is reported with the reference type (`&ManuallyDrop<Fs>`), with the same verdict.
- **`NonNull`/`*const T`'s `as_uninit_*`** are in FORGES although they return `MaybeUninit<T>`. They are redundant with the `assume_init` bound but harmless, and they fail closed.

## Checked

- **Baseline on c17b2db:** `conformance/run.sh` exit 0, 180 PASS, 28 AWAITS, 0 FAIL.
- **Final tree:**
  - `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 4 passed, no warnings.
  - `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS, 0 FAIL**. The total is unchanged because the regression extends an existing case.
  - `rustfmt +stable --edition 2024 --check crates/tide-admit/src/*.rs`: clean. `git diff --check`: clean.
  - `scripts/count-tokens.sh design/reference.md`: 24,992 (not edited).
- **E1 trial programs:** all 11, as a library and with `--test` on the final driver, show no FX code at all. Twenty runs exit 0. `backup.rs` exits 1 with the stub-std rustc E0600 that was already there. No ordinary safe code hits `Need::Forged`.
- **Round 2's raw and union routes** (`k2r3/p2.rs`): FX0517 for each of these:
  - a union pattern through `&` (default binding mode), and a `ref` binding;
  - a union nested in a struct field, a union read inside a closure, a `match` arm, and a method call on a union field.

  These stay clean:
  - union `bits` reads and `bits` literal patterns, a union field write, and `&raw const` of a union field;
  - raw writes `(*p).0 = 1`, a raw read of a `Pure` pointee, a raw slice index of `u64`, `p.write(..)`, and `addr_of!((*p).0)`.
- **FORGES indexing:** type arguments only, with the impl's arguments first. That is correct for every entry, including the new `align_to` (index 1). The patterns match only the methods intended in the pinned sources. The `<impl *const T>`/`<impl *mut T>` blocks' `as_*` methods are all `unsafe` reference conversions. The `*const [T]`/`[T; N]` `as_ptr` family is in other impl blocks and is excluded.
- **Trusted instantiation rule:** structural descent over type arguments, so it terminates. A trusted type nested in a local type keeps the depth bound. AFREE-2's `&mut` search still looks through a trusted type's fields.
- **Design agreement:** every verdict the cases enforce is stated:
  - ROOT-4 for the forging routes, including the new ones;
  - 03 §3.1 and 02 PURE-1 for trusted generics;
  - 02 §9 for the std table, the builders, markers, and the depth bound;
  - AFREE-1/2 for the nested-helper and wrapper cases;
  - PAR-7 for the adapters.

Files changed this round:
- `crates/tide-admit/src/pure.rs` (FORGES)
- `conformance/cases/authority/root_zeroed.rs`
- `design/03-authority-concurrency.md` (ROOT-4; the §3.1 rewrap)

Probes: `scratchpad/k2r3/p1.rs`, `p2.rs`, `p3.rs`, and `probe.sh`.
