Model: Opus 5.5. Date: 2026-10-21. Round 2 (converge), candidate 9359593.

# fx-kw8.2 round 2 (Opus): the Pure check in tide-admit

## Verdict

**fixed.** All twelve round-1 findings are fixed, and each one's probe now gets the expected verdict. This round implemented the coordinator's two decisions. It also fixed one new false accept, union type punning, which forges a capability as `transmute` does, and one wrong code, FX0610 instead of FX0611 for a `Box<&F>` adapter. Each change has a regression in a conformance case. The changes are uncommitted in the worktree. Because this round changed code, another round is required.

## Scope

Read:
- AGENTS.md and GOVERNANCE.md;
- `git show 9359593` in full: `pure.rs`, all of `walk.rs`, `crates.rs`, `diag.rs`, `main.rs`, and `policy.rs`;
- the new and extended cases, and the stub std's `lib.rs` (its re-exports and every type it defines);
- 02 §9 in full, and 03 ROOT-4, UNSAFE-1, §3.1, §3.2, AFREE-1/2, and FFI-1..6;
- both round-1 reviews.

Ran (on the final tree unless noted):
- Baseline on 9359593: `conformance/run.sh` exit 0, 180 PASS, 28 AWAITS, 0 FAIL, which matches the author's result.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 4 passed, no warnings.
- `conformance/run.sh`: exit 0, **180 PASS, 28 AWAITS CHECKER, 0 FAIL**. The total is unchanged because the regressions extend existing cases.
- `rustfmt +stable --edition 2024 --check crates/tide-admit/src/*.rs`: clean. `git diff --check`: clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992, unchanged, since reference.md was not edited.
- The 11 E1 trial programs, as a library and with `--test`: no FX code at all. Twenty runs exit 0. `backup.rs` keeps its pre-existing rustc E0600, which is a stub-std gap.
- Every round-1 probe (`scratchpad/k2r1/*.rs`, `scratchpad/cx/*.rs`) on the candidate. Each now gets the verdict its finding asked for:
  - autoderef, field, deref coercion, and `Termination`: FX0517;
  - the default-body RPITIT: FX0517;
  - the debug builders in `with`: FX0603;
  - changing recursion: FX0517 "too deep", with no hang or overflow;
  - `&f`, `Box::new(f)`, and `&mut f`: FX0611 or FX0610 as PAR-7 says;
  - declaration-only `NeedsPure<Clock>`: FX0517;
  - `Wrapped<'_>(&mut u32)`: FX0518;
  - required trait methods and foreign fns: FX0901 and FX0511;
  - a nested helper: no FX0511;
  - forges by every path: FX0517.
- New probes under `scratchpad/k2r2/`: d1 (generic trusted types), d2 (25 raw-pointer routes), u1 (unions), h1 (an impl's RPITIT that drops `Pure`, `Box<&F>`, and an associated type that drops `Pure`), h2 (casts, array and return coercions to `dyn Pure`), e1 (ill-typed raw, union, and trusted code: no ICE), and x1 (branching changing recursion: answers in 0.03 s).

## Findings and fixes

### Decision 1. `#[trusted_pure]` vouches for the type's own fields only

- **What:** `pure.rs` returned `Ok` for any instantiation of a trusted type, so `Tagged<Clock>` was `Pure`. No design text said otherwise. 02 PURE-1 and 03 §3.1 were silent on generics.
- **Probe:** `k2r2/d1.rs`. `Tag<Clock>`, `Mark<Clock>` (a `PhantomData` marker), and `Tag<T>` with `T` unbounded were all accepted.
- **Fix:** a trusted ADT is now `Pure` exactly when its type arguments are, as a std type is. Lifetime-only trusted types are unaffected.
- **Recorded:** in 03 §3.1, with the reason: the author vouches for what they wrote, not for a caller's argument, and failing closed keeps the trust hole as wide as what the reviewer approved. A clause in 02 PURE-1 points to it.
- **Regression:** `authority/unsafe_trusted_pure_granted.rs` gains `Tagged<T>`. `Tagged<u64>` passes, and `Tagged<Clock>` is marked `//~ FX0517`.

### Decision 2. ROOT-4's raw-pointer reads are every route to a `T` from a raw pointer

- **What:** only the `read*` methods and `ptr::read` were bounded. The candidate admitted `&*p`, `&mut *p`, `(*p).f`, `p.as_ref()`, `p.as_mut()`, `NonNull::as_ref/as_mut`, `slice::from_raw_parts(_mut)`, `Box::from_raw`, `Rc::from_raw`, `Vec::from_raw_parts`, and `ptr::replace` for a pointee of `Fs`. FORGES also matched only `core` items, and it indexed generic arguments counting lifetimes. That index would have been wrong for `slice::from_raw_parts<'a, T>`, whose `'a` is early-bound.
- **Probe:** `k2r2/d2.rs`, 25 routes.
- **Fix:**
  - FORGES now covers the conversions as well as the reads, with patterns checked against the pinned nightly's sources:
    - `<*const T>::as_*` and `<*mut T>::as_*`, which match only the `impl *const T` and `impl *mut T` blocks, so the `*const [T]::as_ptr` family is excluded;
    - `NonNull::as_ref`, `as_mut`, and `as_uninit_*`, but not `as_ptr`;
    - `replace`, `slice::raw::from_raw_parts*` and `from_*ptr_range`;
    - `Box::from_raw*` and `from_non_null*`, `rc::*::from_raw*`, and `Vec::from_raw_parts*` and `from_parts*`.
  - FORGES applies to every std crate and indexes type arguments only.
  - `walk.rs` gains `raw_place`, which bounds a raw pointer's deref by its pointee. A place that is only assigned to (`*p = v`), or only borrowed raw (`&raw const (*p).f`, which rustc deems safe and which reads nothing), is exempt.
  - Both routes go through one helper, `forged`, under a new `Need::Forged`. It is FX0517 with the message "what raw bytes or a raw pointer make must be `Pure`", so the routes share one owner.
- **Recorded:** in 03 ROOT-4, with the reason (each route hands safe code a value no constructor made) and the cost (generic unsafe code that reads through `*const T` bounds `T` by `Pure`).
- **Regression:** `authority/root_zeroed.rs` adds `&*p`, `(*p).0`, `as_ref`, `NonNull::as_mut`, `slice::from_raw_parts`, `Box::from_raw`, and an unbounded `T` deref, each marked `//~ FX0517`. It also adds unmarked lines that must stay clean: a raw field borrow, a `T: Pure` deref, and a `*slot = other` write.

### N1. Union type punning forges a capability (false accept, ROOT-4)

- **What:** reading a union field reinterprets another field's bytes, which is `transmute` by another name. Writing `union Pun { bits: u64, fs: ManuallyDrop<Fs> }` and then `ManuallyDrop::into_inner(unsafe { Pun { bits: 0 }.fs })` mints an `Fs` under the grant with no diagnostic. `ManuallyDrop` is admitted through `core::mem::*`. The same holds for `&p.fs` and for the pattern `let Pun { fs } = p`.
- **Probe:** `k2r2/u1.rs`. All three forms were accepted.
- **Fix:** `raw_place` also bounds a union field expression by the field's type, with the same assign and raw-borrow exemptions. `visit_pat` bounds each field of a union struct pattern. Both go through `forged`.
- **Recorded:** 03 ROOT-4 now names union field reads, by expression or pattern. The conformance `rules.md` ROOT-4 row says the same.
- **Regression:** `authority/root_zeroed.rs`: `&p.fs` and `let Pun { fs } = p` are marked `//~ FX0517`, and the `p.fs = ..` write stays clean.

### N2. `Box<&F>` passed to a slice method gets FX0610 instead of PAR-7's FX0611 (wrong code)

- **What:** the adapter peeling ran two loops, references first and then boxes, so `Box<&F>` stopped at `&F` and never reached the closure's captures.
- **Probe:** `k2r2/h1.rs` line 32 (`xs.par_map(Box::new(&f))`).
- **Fix:** one loop peels `&`, `&mut`, and `Box` in any order. It is shorter than the two loops it replaces.
- **Regression:** `tasks/par_capture_reject.rs` gains `boxed_reference`, marked `//~ FX0611`.

### Smaller edits

- The doc comments of `pure.rs` (module, FORGES, `Need::Forged`) and of `walk.rs` (module, `raw_place`) now describe the new behavior.

## Not changed

- **Exponentially branching finite types.** A chain of about 30 distinct generic structs, each with two fields whose arguments differ, has 2^30 distinct instantiations. The walk visits each one, as rustc's own layout computation would. A recursive family fails fast instead: the depth-first walk reaches depth 64 along its first branch in 64 steps (`x1.rs` answers in 0.03 s). Such a program is contrived and no real program writes one, so no budget was added (AP-MINIMAL).
- **A `const fn` type parameter instantiated with `&mut`.** `const fn f<T: Pure + Sync>(t: T)` called with `&mut u8` passes AFREE-2, because the ban is checked at the definition and cannot be stated as a bound. The rule and its form predate this change, and the design's memoization note covers calls. If it matters, it belongs on its own bead.
- **A crate with E0072 gets no FX verdicts** (round-1 non-blocking 7). `main.rs` says "a crate with rustc errors still gets its verdicts", but some errors abort first. This predates the change, and the sentence was only reworded.
- **Fail-closed rejections kept on purpose:** `(*p).len()` on a `*const [Fs]`, a union pattern field bound to `_`, and `MaybeUninit::assume_init_drop`. Each makes a reference to, or runs code on, the read type, and each is reachable only under the grant.
- **`Pure::new`, a pass-through constructor, and NOT_PURE entries for items the stub lacks** (round-1 nits): harmless.
- **The `rules.md` line references (`03:283`)** were already stale before this round. Fixing them is a separate task.

## Checked

- **Ownership:** FORGES calls and raw places reach `Pure` through one helper (`forged`) under one `Need`. `follow` pairs admission and `Pure` at every obligation site except the `Pure` impl itself, which is E0119. Trusted and std types share the type-argument rule.
- **Design agreement:** every verdict the new and extended cases enforce is stated in 02 §9 or 03:
  - generic trusted types: 03 §3.1 and 02 PURE-1;
  - raw routes and union reads: 03 ROOT-4;
  - `Box<&F>`: PAR-7's "in place of", with the case comment updated;
  - FX0611 for adapters: the case comment.
- **No ICE or hang** on ill-typed raw, union, and trusted code (`e1.rs`), on changing recursion (`a7`, `a8`, `b7`, `c2`), or on branching recursion (`x1`).
- **NOT_PURE coverage:** every type the stub std defines and every Rust std item it re-exports was audited. None is a handle without a type argument that the list misses.
- **No false rejections:** the E1 programs, `&raw const`, raw writes, `T: Pure` derefs, `NonNull::as_ptr`, and `Box::from_raw` and `slice::from_raw_parts` of a `Pure` type are all clean.
- **reference.md:** 24,992 tokens, not edited.

Files changed this round:
- `crates/tide-admit/src/pure.rs`
- `crates/tide-admit/src/walk.rs`
- `conformance/cases/authority/root_zeroed.rs`
- `conformance/cases/authority/unsafe_trusted_pure_granted.rs`
- `conformance/cases/tasks/par_capture_reject.rs`
- `conformance/rules.md`
- `design/02-memory.md`
- `design/03-authority-concurrency.md`
