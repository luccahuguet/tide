# Tide review, fx-kw8.10 round 7 (Opus): converge confirmation round

Candidate: branch fx-kw8.10 at ecfc039 on base 0141db4, diff SHA-256
`cacdbc62c7a37106844b536797dad4717ce8302a13bac5e72d1543e643455126`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Clean: 0 blocking, 0 non-blocking, 1 nit. Round 6's B1, N1, and nit are all
fixed. `cargo test --workspace` exits 0 (33 tests pass) and
`conformance/run.sh` exits 0 at ecfc039. Stable `rustfmt --check` is clean on
mono.rs and interface.rs, and the tree stays clean.

## New findings

- **Nit** (`walk/mono.rs:154`, and the pre-existing `:168-170`): `root`'s doc
  says "The function that `def` is or whose closure it is". The
  `instantiators` doc says "Each is a function, not one of its closures".
  Both are wrong for a closure whose typeck root is not a function. Input
  1: `pub static F: fn(Vec<fn()>) -> Vec<fn()> = |mut v| { v.sort(); v };`
  holds both impls at `lib::F`, kind `static`. Input 2: a closure in
  `impl<T> W<T> { pub const F: .. }` holds them at `lib::W::<T>::F`, kind
  `associated constant`. In both cases the behaviour is right. Suggested
  wording: "The item whose body `def` is, or whose closure it is: a
  function, `const`, or `static`."

## Checked

- B1: `is_generic` returns false for `CRATE_DEF_ID` before it calls
  `generics_of`. The holders test reaches the crate-root path: its expected
  output has a `lib`/`module` entry, and it passes. No other owner without
  generics reaches `generics_of` unsafely. A `static` owner (q4, q8) and an
  associated-const closure (q3) are both fine. A `global_asm` def is never
  an owner, because `uses(GlobalAsm)` is empty, so `local_uses` never yields
  it.
- `global_asm!` with `sym gsort::<fn()>` on a generic local fn (q1): the
  walk holds `Ord` at `{global_asm#0}`, and `gsort` holds `PartialOrd`. With
  a `top` caller as well (q2), the shape is the same as it is with only a
  normal caller, so it is consistent. A `naked_asm!` `sym` is held by its
  naked fn. `global_asm!` with `sym <[fn()]>::sort` beside a static that
  also sorts adds no crate-root entry.
- N1: `instantiators` starts from `root(owner)` only, so keying the memo by
  root is exact. Two closures in one generic fn, and a generic method
  (q5), give the expected holders. Chains of a closure in a generic within
  a generic, with an inline `const` (q7): `g` holds both impls. `by_root` is
  built only when `generic` is non-empty, as the old scan effectively was,
  so no new `root` calls on new defs. Timing with the debug driver: c_600_4
  takes 3.42 s and u_600_4 takes 3.23 s, so the gap has closed.
- Nit fix: the interface.rs:23-28 doc matches the code's exception, and it
  matches the crate-root holder (the `lib` entry). The driver test doc names
  the `global_asm!` case.
- Leftovers: there are none. The old closure is gone, `FxIndexSet` is still
  used, and `CRATE_DEF_ID` is imported once.
