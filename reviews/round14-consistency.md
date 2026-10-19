# Tide consistency review, round 14: confirmation of the round-13 answers

## Verdict

**request_changes**

Reviewed commit `9d1ac8bf3e73175d4fcd98ca55108c6aa12c5301` (`9d1ac8b`, "Add
round 13 and answer it: entry values of by-value parameters"), dated
2026-10-19 18:13 -0300. It was diffed against `e84b2e0`. The review covers
every passage the commit changed:

- `design/01-design.md`, `design/02-memory.md`, and
  `design/03-authority-concurrency.md`;
- `design/04-contracts.md` CLAUSE-4 and §2.4;
- `design/reference.md` §2.1, §4, and §10.1, and `README.md`.

It also skims 01, 02, 03, 04, and the reference for D18 contradictions that
rounds 12 and 13 did not record.

What holds:

- **Round 13's B1 is closed as a statement of meaning.** The changed
  CLAUSE-4 (`04-contracts.md:421–428`) gives every by-value parameter in an
  `ensures` a computable entry value or an error:
  - a `Copy` parameter, a shared reference included, is copied at entry, so
    a reassigned `mut xs: &[u8]` means the entry slice;
  - a non-`Copy` parameter that the body assigns, moves, or mutably borrows
    is cloned at entry, so `x.push(1)` is covered;
  - a write through a `&mut` the parameter holds is covered by name, so
    `Cursor { buf: &mut Vec<u8> }` is an error with a narrowing fix.

  Interior mutability is the one path the trigger leaves out, and no clause
  can observe it. CLAUSE-3 admits only `Pure + Sync` arguments.
  `Cell`, `RefCell`, and `UnsafeCell` are not `Sync`, and atomics and
  handles are not `Pure` (`02-memory.md:816`, `:850`).
- **The rule can be lowered by the D18 attribute.** The lowering needs no
  type information in the macro, and its candidate was compiled
  (Validation).
  - **Snapshot.** The attribute snapshots each by-value parameter named
    outside `old` in an `ensures` with autoref specialization: a `Copy`
    value is copied, a `Clone` value is cloned, and any other is borrowed.
    The clause then reads the snapshot.
  - **Rejection.** rustc's borrow checker rejects the shared-borrow
    fallback exactly when the body assigns, moves, or mutably borrows the
    parameter. A write through a `&mut` field it holds is rejected too.
    The `rustc_private` driver (architecture A) can report those cases as
    the FX error first, using the closure's capture kinds.
- **The other changed lines agree with 01.**
  - 01 D1 now cites reference §12.1–§12.2 (`FX0911`, `FX0912`).
  - 02 §9's coherence list and reference §4 match 01 D4 (`01:343–344`).
    02 adds `'static`, which is additive.
  - RUST-6 lists channel ends, matching D13 (`01:785`).
  - 04 §2.4's `loop {}` gap matches D18 (`01:926`).
  - The README's proc-macro wording matches D6, D13, and D18 for v0.

One problem remains. It was introduced by the answer to round 13's N4:

- **B1 (v0).** An implicit snapshot "counts as `old` for CLAUSE-5", so
  editing a function's body changes which of its clauses debug builds run.
  This contradicts 01 D10's skip rule, CLAUSE-5's own "depends on syntax",
  and the reference's rule.

## Round-13 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** (meaning); new B1 is a side effect | `04-contracts.md:421–428`; `reference.md:1171–1174`. See "What holds" and Validation. The reference omits "including through a `&mut` it holds" (N2). |
| N1 | **Resolved** (channel ends); tier **open, as recorded** | `03-authority-concurrency.md:1242–1243`. |
| N2 | **Partly resolved** | `02-memory.md:802–803` and `reference.md:423` now say "auto or marker". `02-memory.md:267` still lists `Owned` among "the auto and marker traits". The resolution says 02 §2.7 "already did", but the `Owned` half of N2 remains (N4). |
| N3 | **Open, as recorded** | Trait-method grammar tier marks and a v0 FX code. |
| N4 | **Answered with option 1, which creates new B1** | `04-contracts.md:426`. |
| N5 | **Resolved**, apart from the interpreter wording, which is left open as recorded | D1 citation (`01:175`); §2.4's `loop {}` (`04:536`); reference §2.1's intro (`reference.md:86–87`) now separates the last row; README (`README.md:21–24`). |

## New blocking problems

### B1. An implicit snapshot changes a clause's class, so a body edit silently stops debug checks

**Location.**
- `design/04-contracts.md:424–428` (CLAUSE-4): "is snapshotted at entry
  through an implicit `old(..)`, which counts as `old` for CLAUSE-5".
- `04-contracts.md:429–436` (CLAUSE-5): only "`old` of `Copy` values" keep
  a clause `checked`. "The class depends on syntax, not cost".
- `design/01-design.md:628–631` (D10): "clauses built only from operators,
  literals, and standard-library accessors are `checked` in debug and test
  builds".
- `design/reference.md:1194–1197`: the same rule, which lists only explicit
  `old` of `Copy` values. Line 1173's "as if by `old`" leaves open whether
  the snapshot counts.

Tier: v0. This decides which clauses debug builds evaluate.

**Problem.** Take this function:

```rust
#[ensures(result.len() == v.len())]
fn sorted(mut v: Vec<i64>) -> Vec<i64> { v.sort(); v }
```

The clause is built only from accessors and `==`.

- **01 D10 and the reference:** the clause is `checked`, and debug builds
  run it.
- **04:** `v.sort()` mutably borrows `v`. That triggers an implicit
  `old(v)` of a non-`Copy` `Vec`, so the clause is `sampled`. Debug builds
  then skip it, and only `tide test` runs it (CLAUSE-6).

Two consequences follow:

- **A body edit changes the class.** With
  `{ let mut w = v.clone(); w.sort(); w }` as the body, the same clause is
  `checked` again. The class now depends on method resolution in the body,
  which contradicts "depends on syntax".
- **The trigger is not syntax.** Whether `v.f()` mutably borrows `v` is
  decided by autoref. An agent cannot predict it from the clause.

**Proposed fix.** Make the snapshot independent of the body and keep the
class a function of the clause text. This also matches the lowering that
rustc can implement (Validation). Replace the last two sentences of
CLAUSE-4 (`04-contracts.md:422–428`, from "A `Copy` by-value") with:

> A `Copy` by-value parameter, a shared reference included, is copied at
> entry as it is. A non-`Copy` one is cloned at entry if it is `Clone`. If it
> is neither, naming it in an `ensures` is an error when the body assigns,
> moves, or mutably borrows it (including through a `&mut` it holds), and
> the fix narrows it, as in `old(x.len())`. These implicit snapshots follow
> this item's rules for `old` (CLAUSE-3's `Clone` restriction, panics as
> violations) but do not change a clause's class (item 5).

In `reference.md:1171–1174`, write:

> Other parameters mean their entry values: a `Copy` one is copied and a
> `Clone` one cloned at entry; one that is neither cannot be named if the
> body assigns, moves, or mutably borrows it, through a `&mut` it holds
> included (narrow it: `old(x.len())`). This does not change the clause's
> class.

The clone costs time proportional to the value's size. CLAUSE-5 already
accepts the same cost as `checked` for `a == b` on two `Vec<i64>`, so the
cost needs no special treatment.

**Minimal alternative.** Delete ", which counts as `old` for CLAUSE-5," and
add the reference's last sentence. The snapshot stays body-dependent. Then
the macro alone cannot decide which values to clone, and the driver must
pass it each closure's capture kinds (N1).

## Non-blocking notes

### N1. If the body-dependent trigger stays, the macro needs the driver's analysis

A proc macro cannot tell `v.len()` from `v.push(1)`, nor whether `f(v)`
moves `v`. Those are type and borrow facts. Two lowerings work:

- **Body-independent.** Clone every `Clone` parameter named in an
  `ensures`, as B1's fix does. Validation shows this lowering.
- **Driver-fed.** The driver passes the snapshot set to the macro, for
  example from the closure's capture kinds in an analysis pass. That costs
  a second expansion.

If B1 takes the minimal alternative, record which lowering v0 uses in
CLAUSE-7's list of spike gaps.

### N2. The reference omits the held-`&mut` case

`reference.md:1172–1173` says "assigns, moves, or mutably borrows" without
"including through a `&mut` it holds". In
`fn grow(s: Cursor<'_>) { s.buf.push(1) }`, the binding `s` is not `mut` and
is not mutably borrowed in the usual sense, so an agent reading only the
reference would not expect the error. B1's proposed text includes the case.

### N3. A reassigned `&mut` parameter: meaning undefined, and the spike's lowering rejects the idiom

The cursor idiom reassigns a `&mut` parameter:

```rust
fn push_back(mut cur: &mut Option<Box<Node>>) {
    while let Some(n) = cur { cur = &mut n.next; }
    *cur = Some(..);
}
```

It compiles as plain Rust. Two problems follow:

- **The spike's lowering rejects it.** The spike runs the body as
  `let result = (|| body)();`. Once the function carries any
  `#[ensures]`, even one that does not name `cur`, rustc rejects it with
  `E0521` and `E0597`.
- **The meaning is undefined.** For such a parameter, CLAUSE-4's "a bare
  `x` or `*x` of a `&mut` parameter denotes its value at return"
  (`04:420–421`) can mean two things:
  - **the place `cur` points to at return**, which is the tail;
  - **the caller's referent**, which is what v1's `final(*x)` means.

  So `tide fix`'s v0-to-v1 rewrite to `final(*x)` (`04:190–192`) would
  change the meaning.

**Fix.** Either rule should be stated before the checker's test corpus:

- say that a bare `x` or `*x` of a `&mut` parameter means the caller's
  referent at return;
- or make naming a `&mut` parameter that the body reassigns an error, whose
  fix rebinds it (`let mut cur = &mut *cur;`).

Record the closure lowering's rejection among the spike gaps. Related: the
words "by-value parameter" in CLAUSE-4's new sentences could be read to
include `&mut` parameters, which are non-`Copy`, non-`Clone`, and mutably
borrowed. "Of the other parameters, a `Copy` one …" would exclude them.

### N4. Leftovers

- **`02-memory.md:267`** still lists `Owned` among "the auto and marker
  traits". Under D18, `Owned` is `'static` (02 §3.7). This is the remaining
  half of round 13's N2.
- **01 D10's first bullet** (`01:603–606`) states the v1 rule, "A bare `*x`
  of a `&mut` parameter in `ensures` is an error", without a tier mark. The
  later bullet, "In v0 …" (`01:617–625`), overrides it, so this is not a
  contradiction. Mark the first bullet "(v1)", as 04 rule 4 does.
- **The README** (`README.md:21–22`) says "no package code runs at build
  time beyond the toolchain's own proc macros". That holds in v0. Imported
  Rust crates' build scripts run sandboxed (D13, D18, `01:916–917`), so
  qualify the sentence when imports land in v1.

## Ready to implement v0

**Yes, after B1.** B1 replaces two sentences of CLAUSE-4 and one bullet
sentence of the reference, or deletes one phrase under the minimal
alternative.

- **Round 13's B1** is closed as a statement of meaning, and its rule can
  be lowered with rustc's own borrow checker as the enforcement.
- **No other v0-blocking D18 contradiction** was found among 01, 02, 03,
  04, and the reference, beyond those rounds 12–13 recorded. Contract
  spellings, `#[invariants]`, `#[trusted_pure]` tiers, consumer std
  injection, and the release backend agree.
- **Settle N3's rule** before the checker's test corpus is written. Also
  settle round 13's open N3 and the round-12 notes still open.

Validation: this was a read-only review of `9d1ac8b` (diffed against
`e84b2e0`) on 2026-10-19. Rust programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in
`/tmp/claude-1000/r14`. Each one hand-writes the output of a candidate
attribute lowering, in the spike's shape:
`let result = (|| body)(); checks; result`. Each named parameter gets
`let __p_x = (&&&Probe(&x)).snap();`, and the clause reads `(*__p_x)`.

- **Accepted cases.** The program compiled and every clause held:
  - `bump(mut x: Vec<u8>)` with `x.push(1)` (cloned);
  - `adv(mut xs: &[u8])` with `xs = &xs[1..]` (copied);
  - `consume(v: Vec<u64>)`, which moves `v` (cloned);
  - `peek(n: NoClone)`, which only reads (borrowed);
  - generic `glen<T>(v: Vec<T>)` (borrowed) and
    `gpush<T: Clone>(mut v: Vec<T>, t: T)` (cloned);
  - `look(v: Vec<u8>)`, which only reads but is still cloned (B1's fix
    makes this the rule).

  Output: `bump 3`, `adv 2`, `consume 3`, `peek 5`, `glen 1`, `gpush 2`,
  `look 1`.
- **Rejected cases** (non-`Clone`), each by rustc's borrow checker:
  - `grow(s: Cursor<'_>)` writing through `s.buf: &mut Vec<u8>`: `E0500`;
  - `eat(n: NoClone)`, which moves `n`: `E0505`;
  - `reset(mut n: NoClone)`, which assigns `n`: `E0502`.
- **N3.** `push_back(mut cur: &mut Option<Box<Node>>)` under the spike's
  closure lowering, with no clause naming `cur`, failed with `E0521` and
  `E0597`. The same body compiles unlowered, and so does a version that
  first rebinds `let mut cur = &mut *cur;`.

The spike macro (`spikes/d18/macros/src/lib.rs`) was read, not rebuilt.
No Tide checker exists, so every Tide verdict above is an inference from the
text. Only `reviews/round14-consistency.md` was written; nothing was staged
or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** the main fix, as proposed: every `Clone` by-value parameter named
  in an `ensures` is cloned at entry whatever the body does; a parameter
  that is neither `Copy` nor `Clone` is an error only when the body assigns,
  moves, or mutably borrows it; implicit snapshots do not change a clause's
  class (04 CLAUSE-4, reference §10.1). The proc macro can decide this
  alone, which also answers N1.
- **N2:** the reference's sentence now includes "through a `&mut` it holds".
- **N4:** 02 §2.7 lists `'static` beside the auto and marker traits; 01
  D10 marks the bare-`*x` error as `prove` (v1), as 04 §1.3 does.
- **Left open:** N3 (reassigning a `&mut` parameter: the spike's lowering
  and the meaning of "value at return"; settle before the checker's test
  corpus), and N4's README qualifier for v1 imports.
