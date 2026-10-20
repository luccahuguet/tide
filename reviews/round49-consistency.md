# Tide consistency review, round 49: confirmation of the round-48 fix (fx-1am)

## Verdict

**request changes**

Round 48's B1, B2, B3, and N1 are closed as named. The general rule is the
right shape and is implementable: driver A has rustc's trait selection
(`01:950–953`). The reflow changed no wording beyond the listed edits.

One escape of the same family remains. It sits in the rule's undefined
term "applies to". Read naturally, an impl applies to `Feed` when it can
hold for `Feed` itself. Under that reading, an impl for `&Feed` (or a
blanket impl over `&T`) is outside `Feed`'s entry and outside the
add-exception. A delegate can then change or delete
`impl IntoIterator for &Feed`. Worse, a delegate can add one that silently
retargets an existing `feed.go()` call (B1; rustc witness below). One
clause defining the term closes it.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `3e373115…be3589`, full value
`3e37311552390df8ef78f3de22565c54691f3ce65c889467cc47da56b20e3589`). This is
a narrow confirmation round. Scope, as asked:

- the status of round 48's B1–B3 and N1;
- whether the generalized SEAL-2 rule is sound and implementable, what
  "applies to" means for a blanket impl, and whether the type rule and the
  trait rule contradict each other;
- any remaining escape by impl-level items of sealed types or traits;
- any new contradiction with the rest of §11.1, companion §18, or the
  reference pointer, and whether the reflow changed any wording beyond the
  listed edits.

Nothing else was re-audited.

## Round-48 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (inherent associated constants unsealed) | **Resolved** | SEAL-1 now reads "the methods and associated constants of its inherent impls included" (`04:1245–1246`). Constants are always sealed whole (`04:1286`), so `impl Feed { pub const MAX: usize = 10; }` changing to `0` is `FX0920`. The whole-type entry also covers it: it fixes "every impl in the crate that applies to it, inherent, trait, or blanket ... associated types and constants" (`04:1278–1280`). |
| B2 (refined return type of a trait impl method) | **Resolved** | The whole-type entry now fixes "method signatures, refined return types included" (`04:1280–1281`). So `fn items(&self) -> std::vec::IntoIter<u8>` cannot go back to `impl Iterator`. For impls of a sealed trait on unsealed types, the trait entry fixes them "in the same way" (`04:1284–1285`). |
| B3 (blanket impls' associated items) | **Resolved** for both of round 48's shapes | `impl<T: Marker> Bar for T` with `impl Marker for Feed` holds for `Feed`, so it "applies to" `Feed` and is fixed with it (`04:1278–1279`). Tightening its bound to `T: Marker + Debug` removes it from the set, which changes `Feed`'s hash. Deleting `impl Marker for Feed` does the same. `impl Bar for Vec<Feed>` in `Bar`'s module is now fixed by `Bar`'s whole entry, which covers "the impls of it in the crate at seal time" (`04:1284–1285`). Open variant: B1 below. |
| N1 (SEAL-6 names types only) | **Resolved** | "the modules that define the types and traits its interfaces name or its sealed types implement" (`04:1339–1340`), the wording round 48 proposed. |
| Reflow of SEAL-1..6 | **No wording change beyond the listed edits** | See below. |

**Reflow.** I word-diffed the current `04-contracts.md` against round 48's
saved round-47 snapshot. Round 48 had checked that snapshot against Codex's
recorded hash. The diff, `git diff --no-index --word-diff`, touches only
§11.1. It holds exactly round 47's three edits and this round's listed
ones:

- SEAL-1 gains "the" and "and associated constants".
- SEAL-2's type and trait clauses are rewritten as described. "derives"
  becomes "and derives", which belongs to the same rewrite.
- The add-exception becomes "impls that apply to".
- SEAL-4 gains round 47's "recomputing every hash ...".
- SEAL-6 gains "and traits" and "or its sealed types implement".

Every §11.1 line fits 79 columns.

**"Applies to" for blanket impls.** The rule has a consistent reading,
which is the one used above. A blanket impl applies to a type when, in
that configuration, the type meets its bounds. This is trait selection,
not SMT. D18's driver A runs on rustc's own resolution (`01:950–953`), so
"needs no solver" (`04:1239`) still holds in its intended sense. Under
that reading:

- **A new blanket impl.** If it holds for a whole-sealed type, it is
  rejected twice: by the add-exception (`04:1300–1301`), and because the
  type's impl set, and so its hash, changes (`04:1278–1279`, SEAL-3). If it
  holds for no sealed type, it is allowed. An unbounded `impl<T> X for T`
  holds for every type, so a crate with any type sealed cannot add one.
  That is strict but consistent, and companion §18 tells delegates so
  ("add items other than impls that apply to sealed types",
  `companion:469–470`).
- **The type rule and the trait rule.** There is no contradiction in
  effect. A new `impl SealedTrait for SealedType`, or a blanket impl of a
  sealed trait that holds for a sealed type, leaves the trait's hash
  unchanged, because new impls "may be added" (`04:1285`). It changes the
  type's hash, and the add-exception rejects it. The trait sentence's "may
  be added" reads as unconditional, though. It is limited only by the
  separate add-exception sentence. N1 below.

**Other impl-level routes checked, closed or already admitted.**

- **Inherited defaults.** Changing which trait items an impl overrides
  changes the impl's method set, and so its hash (`04:1280–1281`). If the
  trait is unsealed, its defaults are the dependency limit that SEAL-6 now
  names (`04:1339–1340`).
- **Supertrait or where-clause edits on a sealed trait.** These are among
  the trait's items and headers (`04:1283–1284`).
- **Impls of a sealed trait added for unsealed types.** These are allowed
  (`04:1285`). With no overlap or specialization (D4, `01:342`), they
  cannot change which impl an existing obligation selects.
- **An impl that names a sealed type only as a trait argument on a foreign
  self type** (`impl Add<Feed> for u32`). It has no legal home: 5.2 puts an
  impl in the module of its type or its trait (`reference.md:316–317`,
  `FX0401`). With a local trait, it lives in that trait's module, and the
  trait's whole entry covers it when that module is sealed.

## New blocking problems

### B1. "Applies to" is undefined, and the natural reading leaves impls for `&Feed` open

- **Where.**
  - `04:1278–1279`: "every impl in the crate that applies to it".
  - `04:1300–1301`: "except impls that apply to a type sealed whole".
  - Companion §18 (`companion:469–470`).
- **The gap.** Nothing says whether an impl whose self type is `&Feed`,
  `&mut Feed`, or `Box<Feed>` applies to `Feed`. The same holds for a
  blanket `impl<T: Marker> Bar for &T`, which holds for `&Feed`. An
  implementer deciding "can this impl hold for `Feed`" by trait selection
  answers no. Then:
  - No entry fixes such an impl. It is not in `Feed`'s set. A std trait
    such as `IntoIterator` has no entry. A module seal fixes public items
    and inherent impls only (`04:1245–1246`), not trait impls.
  - The add-exception does not stop a new one.
- **Risk.**
  - **Changing or deleting one.** `impl<'a> IntoIterator for &'a Feed`
    (`type Item = &'a u8`) is the usual way to support `for x in &feed`. A
    delegate can change `Item` to `&u16`, or delete the impl. Either
    breaks parallel callers with no `FX0920`. This is round 47's
    `type Item = u8` escape again, on the reference type.
  - **Silently retargeting a call.** Take a sealed `Feed` with
    `impl Foo for Feed { fn go(&mut self) }` and a parallel caller that
    writes `use crate::feed::*; f.go()`. A delegate adds a new trait `Bar`
    to the sealed module (adding items is allowed) and
    `impl Bar for &Feed { fn go(self) }`. Method probing reaches the `&Feed`
    step before the `&mut Feed` step, so `f.go()` now calls the delegate's
    `Bar::go`. It still compiles and gives no `FX0920`. rustc 1.96 prints
    `2` where the sealed code printed `1` (Validation).

  SEAL-6 does not admit this (`04:1336–1346`). Its limits are unsealed
  callers, unsealed dependencies, test-only behavior, and trust holes.
  This is an impl in the sealed type's own module, changing what a call on
  the sealed type does.
- **Smallest fix.** Define the term once in SEAL-2, for example: "an impl
  applies to a type when it can hold for the type or for a reference or
  `Box` of it, a blanket impl when the type meets its bounds". The
  add-exception and companion §18 then inherit the definition unchanged.
  An alternative is "every impl in the crate whose header names it or
  that can hold for it or a reference to it". That also covers
  `impl LocalTrait for Vec<Feed>` when the trait's module is unsealed, but
  it widens what delegates cannot add.

## Non-blocking notes

### N1. The trait sentence's "new impls of it may be added" reads as unconditional

- **Where.** `04:1285`, against the type rule (`04:1278–1279`) and the
  add-exception (`04:1300–1301`).
- **Risk.** The text says a new `impl SealedTrait for SealedType` "may be
  added", yet adding it is `FX0920`. Read per entry (it does not change
  the trait's hash), the two agree. Read as permission, which is how the
  sentence is phrased, they conflict. The later sentence resolves it, so
  this is not blocking.
- **Smallest fix.** Write "new impls of it may be added, except ones that
  apply to a type sealed whole", or "adding one does not change the
  trait's entry".

### N2. The whole-type entry freezes private inherent helpers, and the text does not say so

- **Where.** `04:1278–1281`: "every impl ... inherent ... method
  signatures".
- **Risk.** A delegate implementing a sealed `Feed::fetch` cannot add a
  private `fn parse(&self)` to `impl Feed`, or change one's signature. Any
  inherent impl applies to `Feed`, and private signatures are in the hash.
  This is deliberate, since round 48 proposed covering inherent impls. It
  is a real cost on the most common way of implementing a method, though.
  The companion conveys it only through "impls that apply to sealed types"
  (`companion:469–470`).
- **Smallest fix.** Optional. Add half a sentence to SEAL-2 or companion
  §18: "write helpers as free functions or in an unsealed type". The other
  route is to limit the inherent part to items visible outside the
  module, which needs care, because parallel callers in the same module
  see private items too.

### N3. One prose line in the companion header exceeds 79 columns

- **Where.** `companion:9` is 109 columns: "17, 18, and Appendix A hold
  ... budget (D14)."
- **Smallest fix.** Reflow the paragraph.

## Ready to merge

**No, but the remaining fix is one defining clause.** Round 48's three
escapes and its SEAL-6 note are closed. The type and trait rules agree in
effect, and N1 tidies their wording. The rule is implementable with
driver A's trait selection. The reflow is clean. B1 remains: "applies to"
must say whether impls for references to a sealed type count. Otherwise
an impl added for `&Feed` can silently retarget a call on a sealed type.
The reference pointer (`reference.md:964–967`) and 01's D11 bullet
(`01:733–753`) agree with §11.1 as revised.

## Validation

Only this file was written; nothing was staged or committed.

- **Read:**
  - AGENTS.md;
  - `reviews/round48-consistency.md`, and `reviews/round41-consistency.md`
    for the format;
  - `04:1225–1395` (§11.1 and the worked example);
  - `01:337–350` (D4) and `01:936–962` (D18);
  - reference 5.1–5.2 and the diff's reference 11 pointer;
  - companion header and §18 (`companion:1–14`, `companion:466–489`);
  - the full `git diff -- design/ README.md`.
- **Reflow:** `git diff --no-index --word-diff=plain` of round 48's saved
  round-47 snapshot (`scratchpad/r48/04-old.md`, SHA-256 `dcc98612…3de5`)
  against the current `design/04-contracts.md`. Results are under Reflow
  above. Column widths were checked with `awk`.
- **Witness:** compiled and run with
  `rustc +1.96.0 --edition=2024`, stdin closed, 60 s timeout. The program
  has:
  - a module `feed` with `Feed`, `impl Foo for Feed { fn go(&mut self) }`
    returning 1, and `impl IntoIterator for &Feed`;
  - an added `trait Bar` and `impl Bar for &Feed { fn go(self) }`
    returning 2;
  - a caller with `use feed::*; f.go()`.

  It printed `2 7`: the call bound to the added impl. This proves only what
  rustc accepts. Tide's verdicts are deductions from the text, since no
  checker exists.
