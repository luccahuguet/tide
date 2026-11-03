# Tide review, fx-kw8.10 round 6 (Opus): converge confirmation round

Candidate: branch fx-kw8.10 at 950c0b7 on base 0141db4, diff SHA-256
`027a39c5735660b60a9829768f0f0ea897857d968afefbe7f122025a30f2fa71`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 1 blocking, 1 non-blocking, 1 nit. Round 5's B1 is fixed, and so is
its N1. The new crate-root path crashes the driver when the `unsafe` grant
admits an impl that no local item reaches. `cargo test --workspace` and
`conformance/run.sh` both exit 0 at 950c0b7. Stable `rustfmt --check` is clean
on mono.rs and invocation.rs. The tree stays clean.

## New findings

- **B1, blocking** (`walk/mono.rs:62,70,135-138`): the crate-root fallback
  ICEs. When `local_uses` is empty, `sites` gets `CRATE_DEF_ID`. The match
  guard then calls `is_generic(tcx, CRATE_DEF_ID)`, which runs
  `generics_of` on the crate root. The result is "generics_of: unexpected
  node kind Crate" and exit 101. Input, with `--tide-std ...
  --tide-grant=unsafe`:
  `core::arch::global_asm!(".quad {}", sym <fn() as core::cmp::PartialOrd>::lt);`
  It also crashes with `pub fn lt(x: fn(), y: fn()) -> bool { x < y }`
  beside it. Without the grant, the walk rejects the input with FX0520,
  as it should. The empty `sites` comes from `uses` (mono.rs:220), which
  returns nothing for `MonoItem::GlobalAsm`, so a `sym` operand's instance
  has no local user. Fix: guard the arm with
  `owner != CRATE_DEF_ID && is_generic(..)`, or have `is_generic` return
  false for a def without generics (a `Mod`). Add the input to a driver
  test. Optionally, `uses` could record a `global_asm!`'s `sym` operands, so
  that such a use is held by the `{global_asm#N}` item and not by the
  crate.
- **N1, non-blocking** (`walk/mono.rs:110-114,152-175`): the generic merge
  is quadratic in crate size. `instantiators` runs once per
  `generic` entry, which means once per impl method instance and site, not
  once per owner. Each def it visits scans every mono item. Input: the
  generator `scratchpad/r6/gen.py K 4`. It emits K chains
  `top_i -> s_i_3<T> -> .. -> s_i_0<T>`, where `s_i_0` sorts and dedups a
  `Vec<fn()>`, and each `top_i` instantiates its chain at 8 integer types.
  Debug driver timings, the build that `conformance/run.sh` uses:

  | K   | with `fn()` | same crate with `u8`, no holes |
  | --- | ----------- | ------------------------------ |
  | 300 | 3.6 s       | 0.7 s                          |
  | 600 | 14.1 s      | 1.6 s                          |

  The output is correct; this costs time only. Fix: build a
  `LocalDefId -> Vec<MonoItem>` index (by typeck root) once, and memoize
  `path` per owner root, or group `generic` by owner before the merge.
- **Nit** (`interface.rs:24-26`): the module doc says a generic item holds
  an impl it reaches through std's internals "when a local instance of it
  does". It omits the exception that the code applies (mono.rs:108-109).
  Take `pub fn mid<T>(_: T) -> String { debug(f as fn()) }` with `top`
  calling `mid(0u8)`. There `debug::<fn()>` runs `Debug for fn()` through
  `fmt`, yet `mid` holds the impl and `debug` holds none. The accepted
  residual is another such case. Suggest adding: "unless an item that
  instantiates it, whose types may have chosen the impl, holds it already".

## Checked

- r5 B1 inputs: `gsort` with `g1`, plus either `sorted` or `lt`. `gsort`
  holds both `Ord` and `PartialOrd`, and `sorted`/`lt` hold their own. The
  holders test now covers this: it fails under r5's crate-wide filter
  because `lt` holds `PartialOrd`.
- `debug<T>` with `shown`: only `shown` holds `Debug`. A nested chain
  top -> mid<T: Debug> -> debug<T> with `fn()`: only `top` holds it. A
  generic `mid<T>` that passes a concrete `fn()` to `debug`: `mid` holds it
  and `debug` holds none.
- A closure inside a generic (`gsort<T>`'s closure sorts) holds both
  impls at `gsort`. A closure in a non-generic `top` that calls
  `gsort::<u8>` gives the same result. So does a closure in a generic
  impl method that calls `gsort`.
- Mutual recursion `a<T>` <-> `b<T>`: it terminates, and `a` holds both
  impls.
- A chain through generic `mid<T>`: `gsort` keeps both impls next to an
  unrelated `lt`.
- Instantiators that are a `static` holding a fn pointer, or a `const`
  closure: there was no ICE, and `gsort` holds both impls.
- Order independence: `kept` is filtered against the walk's holes and the
  non-generic backstop holes only, and it is merged at the end.
- r5 N1 docs (interface.rs:19, interface.rs:235-239): they match the
  `P`/`Poly`/`M1` probes, which still give the same output.
- Leftovers: there are none in mono.rs or invocation.rs. The `local`
  closure is gone, and `Holes` is passed by `&mut`.
