# Tide consistency review, round 50: confirmation of the round-49 fix (fx-1am)

## Verdict

**request changes**

Round 49's B1 is closed for the shapes it named: an impl for `&Feed`,
`&mut Feed`, `Box<Feed>`, or `Vec<Feed>`, and a blanket impl over `&T`.
N1–N3 are closed, and the reflow changed no wording beyond the listed
edits.

The new definition's third clause still leaves a gap. It counts a type
parameter only when the sealed type itself can be the argument, not when
the argument merely contains it. `impl<T: Copy> Bar for T` holds for
`&Feed` but not for `Feed`, so under the text it does not apply to `Feed`.
It still captures an existing `f.go()` call through `&Feed`, which the
text now says cannot happen (B1, rustc witness below). A second capture
route needs no impl at all: a `pub use` of an existing trait, added to the
sealed module, retargets the same call at a glob-importing caller (B2).

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `cd742858…606`, full value
`cd742858e1e31f175a72adfac86cf4e64995f2ff78620cdcb68d1b769fb82606`). This is
a narrow confirmation round. Scope, as asked:

- the status of round 49's B1 and N1–N3;
- whether the definition of "applies to" is sound, implementable by D18's
  driver A, and free of over-reach that contradicts other text;
- any remaining escape of the same family;
- any new contradiction with the rest of §11.1, companion §18, or the
  reference pointer, and whether the reflow changed wording beyond the
  listed edits.

Nothing else was re-audited.

## Round-49 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 ("applies to" undefined; `&Feed` impls escape) | **Resolved for the named shapes**, open for one variant (B1 below) | `04:1281–1285` defines the term: the type occurs "as that type, inside it (`&Feed`, `&mut Feed`, `Box<Feed>`, `Vec<Feed>`), or as an argument for a type parameter there whose bounds it can meet (`impl<T: Marker> Bar for &T`)". `impl IntoIterator for &Feed` is now in `Feed`'s set, so changing `Item` or deleting the impl is `FX0920`. Round 49's witness, `impl Bar for &Feed { fn go(self) }`, is now an added impl that applies to a whole-sealed type, and the add-exception rejects it (`04:1307–1308`). The add-exception and companion §18 (`companion:471–472`) inherit the definition. |
| N1 ("new impls of it may be added" unconditional) | **Resolved** | "new impls of it may be added, except ones that apply to a type sealed whole" (`04:1291–1292`). |
| N2 (private inherent helpers frozen, unstated) | **Resolved** | "Private inherent methods are fixed too, so a delegate's helpers are free functions" (`04:1288–1289`). Companion: "write helpers as free functions" (`companion:472`). |
| N3 (companion header over 79 columns) | **Resolved** | `companion:1–16` reflowed; no line exceeds 79. A word diff against HEAD shows only the additions "what a sealed interface allows," and "17, 18,", plus the core count. |
| Reflow | **No wording change beyond the listed edits** | Word diff of round 48's saved round-47 snapshot (SHA-256 `dcc98612…3de5`) against the current `04-contracts.md`. Outside SEAL-2's whole-entry bullet, the diff holds exactly the edits round 49 listed (SEAL-1, the add-exception, SEAL-4, SEAL-6). Inside it, the changes are the definition, the two "So …" consequences, the private-method sentence, and N1's clause. Every §11.1 line fits 79 columns. |

**Implementability.** Driver A runs on rustc's own resolution
(`01:950–953`), so it can decide each clause. It walks the self type for
the first two clauses. For the third, it substitutes the sealed type for
one parameter, makes the others inference variables, and evaluates the
impl's predicates. "Can meet" then has to mean "may hold", with an
ambiguous result counted as applying. Read the other way, as "could be
made to hold by a future impl", every bounded blanket impl would apply to
every type. The B1 fix below removes the inference variables, and with
them the ambiguity.

**Over-reach.** "Every impl in the crate" (`04:1278`) excludes std and
dependency impls. Those are pinned by the toolchain version and the
lockfile hash (`04:1263–1264`), so this agrees with the rest of SEAL-2.
Inside the crate, every unbounded `impl<T> X for T`, and every
`impl<T> X for Vec<T>`, `Option<T>`, and so on, applies to every
whole-sealed type. Once any type is sealed, those impls are frozen and new
ones are rejected. Round 49 accepted this cost for `impl<T> X for T`. It
contradicts no other text, but companion §18 gives agents only the
`&Feed` example (N1).

## New blocking problems

### B1. The type-parameter clause misses arguments that contain the sealed type

- **Where.** `04:1284–1285`: "or as an argument for a type parameter there
  whose bounds it can meet". The claim at `04:1286–1287`: "a new trait
  method cannot capture an existing `f.go()` call through `&Feed`".
- **The gap.** The clause requires the sealed type itself to be the
  argument. `impl<T: Copy> Bar for T` has the bare self type `T`.
  - `Feed` is not `T`'s argument, because a typical `Feed` is not `Copy`.
  - `Feed` does not occur inside `T` as written.
  - `&Feed` meets `T: Copy`, since every shared reference is `Copy`, but
    `&Feed` is not `Feed`.

  So the impl does not apply to `Feed`. A delegate may add it, with a new
  trait `Bar { fn go(self) -> u32; }`, in the sealed module (adding items
  is allowed). Method probing reaches the `&Feed` step before the
  `&mut Feed` step, so a parallel caller's `use crate::feed::*; f.go()`
  moves from the sealed `Foo::go(&mut self)` to `Bar::go`. rustc 1.96
  prints `2` where the sealed code printed `1` (Validation). This needs no
  existing impl. Std's `impl Clone for &T` and `impl Pointer for &T` give
  the same route with `T: Clone` or `T: fmt::Pointer`. A crate trait with
  an existing `impl Marker for &Feed` (for example `IntoIterator`) does
  too.
- **Risk.** This is round 49's B1 in another shape, and `04:1286–1287` now
  states it cannot happen, so the text is also false. SEAL-6 does not
  admit it (`04:1343–1353`).
- **Smallest fix.** Define the third clause by a finite set of concrete
  types, which also removes the "can meet" ambiguity. For example: "An
  impl applies to a type when the type occurs in the impl's self type as
  written (`&Feed`, `Box<Feed>`, `Vec<Feed>`), or when the impl holds for
  the type, `&`, `&mut`, or `Box` of it, or a type containing it that the
  crate writes (`impl<T: Marker> Bar for &T` with `impl Marker for Feed`;
  `impl<T: Copy> Bar for T`, which holds for `&Feed`)." Driver A decides
  "holds for" by trait selection on concrete types. Method probing on a
  `Feed` receiver visits `Feed`, `&Feed`, and `&mut Feed`, so this set
  covers every capture step that is not a deref step (N2). The
  alternative is "any instance of the self type that contains the type
  and meets the bounds". That is complete, but it is an open search over
  types, and the driver has no stated bound for it.

### B2. A re-export added to a sealed module captures the same call

- **Where.** `04:1306–1308`: "a delegate may add items, except impls that
  apply to a type sealed whole". The add-exception is the only
  restriction on additions.
- **The gap.** Take an existing trait `Baz` outside the sealed module,
  with `impl Baz for &Feed { fn go(self) }`. That impl is fixed with
  `Feed`, and it is legal. A caller of the sealed module does not import
  `Baz`. The delegate adds `pub use crate::other::Baz;` to the sealed
  module. That is a `use` item, not an impl, so the add-exception does not
  reach it, and no entry's hash changes. The caller's
  `use crate::feed::*; f.go()` now brings `Baz` into scope, and the call
  moves from `Foo::go(&mut self)` to `Baz::go` at the `&Feed` step. rustc
  1.96 prints `3` where the sealed code printed `1` (Validation). The same
  glob path lets an added `pub fn` shadow a prelude name at the caller,
  for example `pub use std::mem::forget as drop;`. That is not a call on a
  sealed type, but it is the same mechanism.
- **Risk.** An item added to the sealed module silently changes what an
  unchanged call on a sealed type does. That is the effect round 49's B1
  named, and SEAL-6 does not admit it. It needs an existing trait with a
  same-named method on an earlier probe step, so it is rarer than B1.
- **Smallest fix.** One of these, in SEAL-6 or the add-exception.
  - Admit it in SEAL-6: "a caller that glob-imports a sealed module also
    sees items a delegate adds there, such as a re-exported trait, which
    can change the method a call selects; seal such callers or import by
    name."
  - Or bar it: "except impls that apply to a type sealed whole and public
    `use` declarations". This still leaves child modules' `use super::*`
    to the SEAL-6 caller limit.

  The first is a one-sentence change that matches SEAL-6's existing
  stance on unsealed callers.

## Non-blocking notes

### N1. Companion §18 shows only the reference case of "applies to"

- **Where.** `companion:471–472`: "add items other than impls that apply
  to sealed types (`impl Bar for &Feed` included)".
- **Risk.** An agent reading only the companion would not infer that
  adding `impl<T> Tag for T`, `impl<T: Clone> Pretty for Vec<T>`, or (after
  B1's fix) `impl<T: Copy> Bar for T` anywhere in the crate is `FX0920`
  once any type is sealed. It would find out from the diagnostic. That is
  safe, but it wastes a loop.
- **Smallest fix.** Write "(`impl Bar for &Feed` and blanket impls such as
  `impl<T: Copy> Bar for T` included)".

### N2. Deref targets are not named in SEAL-6's list

- **Where.** `04:1345–1347`: "the modules that define the types and traits
  its interfaces name or its sealed types implement".
- **Risk.** Suppose a sealed `Feed` has `impl Deref for Feed`, with
  `Target = Inner` and `Inner: Copy` in an unsealed module. Then
  `impl Bar for Inner { fn go(self) }` beats `Inner`'s inherent
  `go(&self)` on an `f.go()` that reached it through deref. Neither
  `Feed`'s entry nor B1's fixed set covers `Inner`. This reduces to
  sealing `Inner`'s module, because `Inner`'s own methods are unsealed
  anyway. Whether `Inner` counts as "named by its interfaces" is left to
  the reader.
- **Smallest fix.** Optional. Write "name, deref to, or implement".

## Ready to merge

**No.** Round 49's B1 shapes and N1–N3 are closed, the reflow is clean,
and the definition creates no contradiction with §11.1, companion §18,
the reference pointer (`reference.md:964–967`), or 01's D11 bullet
(`01:733–753`). Two capture routes remain:

- **B1.** A blanket impl holds for `&Feed` but not for `Feed`, so the text
  does not count it. This makes the stated guarantee at `04:1286–1287`
  false. The fix is one redefined clause.
- **B2.** A re-exported trait reaches glob-importing callers. The fix is
  one SEAL-6 sentence.

## Validation

Only this file was written, plus scratch files; nothing was staged or
committed.

- **Read:**
  - AGENTS.md;
  - `reviews/round49-consistency.md`, and `reviews/round41-consistency.md`
    for the format;
  - `04:1225–1410` (§11.1 and the worked example);
  - `01:945–957` (D18), and 01's diff (the tier table and D11 bullet);
  - reference 5.1–5.2 and 12.2 (glob imports are as in Rust;
    `use super::*` is named);
  - the reference 11 pointer (`reference.md:960–967`);
  - companion header and §18;
  - the full `git diff -- design/ README.md`.
- **Reflow:** `git diff --no-index --word-diff=plain` of
  `scratchpad/r48/04-old.md` (SHA-256 `dcc98612…3de5`) against the current
  `04-contracts.md`, and `git diff --word-diff=plain` for the companion.
  Widths were checked with `awk`; the remaining long companion lines
  (129–234) are pre-existing tables.
- **Witness:** `scratchpad/r50/w1.rs`, compiled and run with
  `rustc +1.96.0 --edition=2024` (rustc `ac68faa20`), stdin closed, 60 s
  timeout. The sealed code has `Feed`, `impl Foo for Feed { fn go(&mut self) }`
  returning 1, and a caller with `use crate::feed::*;`. The output is
  `f.go()` then `g.go()`.
  - Baseline: `1 1`.
  - `--cfg added` adds a new `trait Bar` and `impl<T: Copy> Bar for T`
    returning 2 to `feed`: `2 2`. This is B1. `Feed` is not `Copy`, and
    `&Feed` is.
  - `--cfg reexp` adds `pub use crate::other2::Baz2;` to `feed2`, where an
    existing `impl Baz2 for &Feed2` returns 3: `1 3`. This is B2.

  This proves only what rustc accepts. Tide's verdicts are deductions from
  the text, since no checker exists.
