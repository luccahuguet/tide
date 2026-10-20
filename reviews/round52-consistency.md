# Tide consistency review, round 52: confirmation of the round-51 fix (fx-1am)

## Verdict

**approve**

Codex round 51's B2 is closed. A generic sealed type's own impls, such as
`impl<T: Copy> Iterator for Feed<T>`, now apply to it by a syntactic test
on the self type, with no trait selection. Everything rounds 48–50 closed
stays closed under the reading those rounds used. The conservative
"cannot rule out" rule maps onto an operation that driver A already has,
so it needs no unstated symbolic test. Nothing contradicts §11.1,
companion §18, or round 51's accepted text. The edit is the only change
since round 51.

There are two non-blocking notes. The new text narrows round 51's "it
holds for" to "a blanket impl". Under the corpus's own use of "blanket"
(`impl<T> X for T`), that leaves generic impls over `&T`, `Box<T>`, or
`Vec<T>` that do not name the sealed type outside both clauses (N1).
The phrase "any instantiation of a generic type included" also has a wide
reading that would make the blanket clause redundant (N2).

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `03de0e86…061c`, full value
`03de0e863d101f373b5e319c54178158cd703403937669e4577298df449d061c`). This is
a narrow confirmation round. Scope, as asked:

- whether Codex round 51's B2 is closed;
- whether the new "applies to" definition keeps closed what rounds 48–50
  closed, and whether D18's driver A can implement it without an unstated
  symbolic test;
- whether it contradicts §11.1, companion §18, or round 51's accepted text,
  and whether the reflow changed wording beyond this edit.

Nothing else was re-audited.

## Item status

| Item | Status | Evidence |
| --- | --- | --- |
| Codex r51 B2 (generic sealed type: `impl<T: Copy> Iterator for Feed<T>`) | **Resolved** | `04:1285–1287`: an impl applies "when its self type contains the type, any instantiation of a generic type included (... `impl<T: Copy> Iterator for Feed<T>`)". The self type `Feed<T>` names `Feed`, so the impl is in `Feed`'s whole entry. Its header, its `type Item = u8`, and its method signatures are all fixed (`04:1281–1284`). Changing `Item` to `u16`, tightening `T: Copy`, or deleting the impl is `FX0920`. Adding `impl Iterator for Feed<String>` names `Feed`, so the add-exception bars it (`04:1296–1297`, `04:1312–1313`). The test needs no concrete `Feed<X>` and no reasoning under the definition's bounds, which was Codex's objection. |
| r48 B1 (inherent associated constants) | **Still closed** | SEAL-1 is unchanged (`04:1245–1246`). `impl Feed { .. }` has self type `Feed`, so clause (a) covers it. |
| r48 B2 (refined return types) | **Still closed** | "method signatures, refined return types included" is unchanged (`04:1284`). |
| r48 B3 (`impl<T: Marker> Bar for T`; `impl Bar for Vec<Feed>`) | **Still closed** | The first is a blanket impl that holds for `Feed`, so it falls under clause (b) (`04:1287–1289`). The second names `Feed` in its self type, so it falls under clause (a). |
| r49 B1 (`&Feed`, `&mut Feed`, `Box<Feed>` impls; generic impl over `&T`) | **Still closed** for the concrete shapes; the `&T` shape depends on N1 | The concrete shapes fall under clause (a) as "self type contains the type". `impl<T: Marker> Bar for &T` is covered only if "blanket impl" includes an impl over `&T`. Rounds 49 and 50 called it one. The corpus uses the word for `impl<T> X for T` only (N1). |
| r49 N1, N2 | **Still closed** | `04:1296–1297` and `04:1293–1294` are unchanged. |
| r50 B1 (`impl<T: Copy> Bar for T` through `&Feed`) | **Still closed** | `04:1288–1291`: a blanket impl that may hold "for `&` ... of it" applies, and the text names this exact case. |
| r50 B2 (re-export capture), N1, N2 | **Still closed** | SEAL-6 (`04:1348–1356`) and companion §18 (`companion:471–473`) are unchanged. |
| Reflow | **No wording change beyond this edit** | See below. |

**Reflow.** I took round 51's view of §11.1 from Codex's `nl -ba` read of
`04:1233–1359` in its log, and word-diffed it against the current file.
The only difference is the definition sentence, `04:1285–1291`. The other
lines of that paragraph were rewrapped, with the same words. I then put
that snapshot back into a copy of the tree and regenerated
`git diff -- design/ README.md`. Its SHA-256 is
`bf83df00…1c2`, exactly the hash Codex recorded. So nothing else in any
file has changed since round 51. Every §11.1 line fits 79 columns.

**Implementability.** The definition has two clauses, and driver A can
decide each one.

- **Clause (a).** This is a walk of the impl's resolved self type
  (rustc's `type_of`) for the sealed type's definition, with any generic
  arguments. It needs no trait selection.
- **Clause (b).** This is evaluated against a finite set of candidates:
  the type, `&`, `&mut`, and `Box` of it, and the crate-written types that
  contain it.
  - A generic sealed type's own parameters, and any parameters in the
    crate-written types, become fresh inference variables.
  - "Cannot rule ... out" is rustc's "may hold" result, where an ambiguous
    outcome counts as holding. This is the query that coherence and
    diagnostics already use (`InferCtxt::predicate_may_hold`).
  - A `rustc_private` driver on the pinned nightly (`01:950–953`) has that
    query directly.
  - "Needs no solver" (`04:1239`) keeps the sense rounds 49–50 accepted,
    which is no SMT.

  Counting an ambiguous result as applying is conservative. It can only
  freeze more impls. It cannot let one escape.

**Contradictions.** None found.

- **Companion §18.** "impls that apply to sealed types (`impl Bar for &Feed`
  and blanket impls such as `impl<T> Bar for T` included)"
  (`companion:471–473`) matches both clauses.
- **The trait rule and the add-exception.** "new impls of it may be added,
  except ones that apply to a type sealed whole" (`04:1296–1297`,
  `04:1312–1313`) inherits the definition unchanged.
- **SEAL-6.** The definition does not touch SEAL-6's admission that a new
  impl can retarget an unsealed `f.go()` (`04:1352–1354`).
- **The "so" sentence.** The closing example at `04:1290–1291` is a case
  that selection decides positively, since `&Feed: Copy` holds. It is not
  an ambiguous one. "Cannot rule out" includes it, so the "so" is still
  true.

## New blocking problems

None.

## Non-blocking notes

### N1. "Blanket impl" narrows round 51's "it holds", and the corpus uses the word for `impl<T> X for T` only

- **Where.** `04:1287–1290`: "or when it is a blanket impl that may hold
  for the type, for `&`, `&mut`, or `Box` of it, ..." and "cannot rule a
  blanket impl out". Round 51's text said "it holds for the type itself,
  for `&`, ...", for any impl.
- **The gap.** The corpus says "blanket impl" only for impls over a bare
  type parameter: `Pure`'s (`02:788–792`, `01:455–457`), core's blanket
  `Into` (`03:394`), and companion §18's `impl<T> Bar for T`. rustdoc's
  "Blanket Implementations" uses the same sense.
  - Under that reading, a generic impl whose self type is `&T`, `&mut T`,
    `Box<T>`, or `Vec<T>` is not a blanket impl.
  - It also does not name `Feed`, so clause (a) does not reach it either.
  - So `impl<T: Marker> Bar for &T`, with `impl Marker for Feed`, does not
    apply to `Feed`. Round 49 named this shape, and round 50 marked it
    closed.
- **Risk, bounded.** Orphan rules and D4 (`reference.md:316–317`) put such
  an impl in a local trait's module. That limits the damage:
  - **Adding one.** If the trait's module is sealed, its whole entry still
    fixes the impls it had at seal time. A newly added impl's capture of
    an unsealed `f.go()` is admitted by SEAL-6 (`04:1352–1354`). rustc
    1.96 prints `2` where the sealed code printed `1` (Validation). A
    sealed caller's resolution is hashed (`04:1261–1265`).
  - **Changing an existing one in an unsealed trait module.** Changing
    `type Out` breaks a parallel `<&Feed as Bar>::Out` with no `FX0920`.
    Round 51's text froze it through `Feed`'s entry. Now only SEAL-6's
    "the modules that define the ... traits its ... sealed types
    implement" (`04:1351–1352`) covers it, and only if `&Feed: Bar` counts
    as `Feed` implementing `Bar`.

  This is not blocking. The residue depends on how the word is read, and
  SEAL-6 arguably admits it. It is still a loss of coverage from round 51's
  accepted text, in a defining term.
- **Smallest fix.** Drop the word twice. Write "or when it may hold for
  the type, for `&`, `&mut`, or `Box` of it, ...". Then write "Where the
  driver's trait selection cannot rule an impl out, it counts as applying".
  Concrete impls that do not name the type are ruled out by unification,
  so this adds no over-reach. The alternative is to define the term once:
  "a blanket impl, one whose self type is generic and does not name the
  type (`impl<T: Copy> Bar for T`, `impl<T: Marker> Bar for &T`)".

### N2. "Any instantiation of a generic type included" has a reading that makes clause (b) redundant

- **Where.** `04:1285–1287`.
- **Risk.** The examples (`&Feed`, `Vec<Feed>`, `Feed<T>`) show the
  intended sense: the self type names `Feed`, with any generic arguments.
  But "any instantiation" can also be read as instantiating the impl's own
  parameters.
  - Under that reading, `Vec<T>` with `T := Feed`, or a bare `T`, "contains"
    `Feed`.
  - Every generic impl would then apply to every sealed type, whatever its
    bounds, and the blanket clause and its "cannot rule out" rule would
    never matter.
  - The reading is conservative, so nothing escapes. It does make two
    implementations disagree on which impls a delegate may add.
- **Smallest fix.** Write "when its self type names the type, with any
  generic arguments (`&Feed`, `Vec<Feed>`, or `Feed<T>` in
  `impl<T: Copy> Iterator for Feed<T>`)".

## Ready to merge

**Yes, from this round's scope.** Codex's B2 is closed by a syntactic
self-type test that needs no selection under a parameter environment. The
items closed by rounds 48–50 stay closed, with N1's caveat on the `&T`
shape. The "cannot rule out" rule is driver A's ordinary "may hold"
query. The edit is the only change since round 51, as shown by
reconstructing round 51's diff hash. N1's one-word fix is recommended
before the Codex re-check, because it restores round 51's scope exactly.

## Validation

Only this file was written, plus scratch files; nothing was staged or
committed.

- **Read:**
  - AGENTS.md;
  - `reviews/round51-codex.md`, `round48`–`round50-consistency.md`, and
    `round41-consistency.md` for the format;
  - `04:1233–1362` (§11.1);
  - `01:945–957` (D18);
  - companion §18 (`companion:467–492`);
  - every use of "blanket" in `design/`.
- **Reflow and hash:**
  - Round 51's §11.1 came from Codex's `nl -ba` read in
    `scratchpad/codex2.log`. I word-diffed it against the current file with
    `git diff --no-index --word-diff=plain`.
  - I put it back into a copy of `design/` and ran
    `git --work-tree=<copy> diff -- design/ README.md | sha256sum`. The
    result was `bf83df00d30d15b9c39b15723c660f61d507daf23a7ff339597807eaa71091c2`,
    round 51's recorded hash.
  - Widths were checked with `awk`.
- **Witness:** `scratchpad/r52/w/w.rs`, compiled and run with
  `rustc 1.96.0 (ac68faa20) --edition=2024`, stdin closed, 60 s timeout.
  - The sealed side has `Feed`, `impl Marker for Feed`, and
    `impl Foo for Feed { fn go(&mut self) }` returning 1. A caller writes
    `use feed::*; f.go()`.
  - Baseline: prints `1`.
  - `--cfg added` adds `trait Bar { fn go(self) }` and
    `impl<T: Marker> Bar for &T` returning 2: prints `2`. This is the N1
    shape, and SEAL-6 admits it for an unsealed caller.
  - The same file holds Codex's generic `Feed<T>` with
    `impl<T: Copy> Iterator for Feed<T> { type Item = u8 }` and a generic
    caller returning `Option<u8>`. It compiles. That is the B2 baseline,
    which clause (a) now seals.

  This proves only what rustc accepts. Tide's verdicts are deductions from
  the text, since no checker exists.
