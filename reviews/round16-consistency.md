# Tide consistency review, round 16: the fx-wsd pre-corpus notes

## Verdict

**approve with notes**

Reviewed commit `9bac29eddd83461a68b83b6482c09c5246af50c9` (`9bac29e`,
"Settle D18 pre-corpus review notes (fx-wsd)"), dated 2026-10-19 18:55
-0300. It was diffed against `c15790c`. Scope, as asked:

- whether each note listed in bead `fx-wsd` is fixed in the design text, or
  recorded in 01's Open Questions with a reason it can wait past the
  checker's test corpus. The owner's decisions in the bead's comments are
  taken as given;
- whether the changed lines contradict 01, 02, 03, 04, the reference, or
  the README, and whether the old rule survives anywhere the change missed;
- whether `FX0512` and `FX0801` collide with existing codes;
- whether the Rust claims hold on rustc 1.96 (Validation).

The rest of the design was not re-audited; rounds 12–15 did that.

What holds:

- **Every listed note is answered** (table below). N12's budget gap is
  recorded, not fixed, as the owner decided: 01 open question 8 and the
  reference header give the count, and bead `fx-fbq` owns the restructure.
- **The four owner decisions are stated as decided, and they agree across
  documents.**
  - `main`: 01 D1 (`01:189–194`), 03 ROOT-1 (`03:173–177`), and reference
    6.2 and §14 (`reference.md:498–499`, `:1451`).
  - Simulator aborts: 01 D5 (`01:364–366`), 03 PANIC-1 (`03:247–257`), and
    reference §9 (`reference.md:1145–1147`).
  - `&mut` at return: 01 D10 (`01:633–637`), 04 rule 4 and CLAUSE-4
    (`04:189–194`, `:424–428`), and reference §10.1 (`reference.md:1172–1175`).
  - Implicit snapshots: CLAUSE-3 and CLAUSE-4 (`04:400–401`, `:429–438`)
    and reference §10.1 (`reference.md:1181–1185`).
- **No old rule survives.** No text still says any of these:
  - "its value at return" for a `&mut` parameter;
  - "Under `prove` (v1)" for the bare-`*x` error;
  - "cloned at entry if it is `Clone`" without `Pure + Sync`;
  - E0580 as the code for a plain `main`;
  - `sqrt` among the banned float methods;
  - `by = ..` inside a clause.

  02's edit is a reflow with no word changed (`git diff --word-diff`).
  02:728–731 and 04:623 still say only that an overflow aborts, or fails
  the seed. Both remain true under PANIC-1.
- **The codes do not collide.** Before this commit, no file under `design/`
  used `FX0512` or `FX0801`. `FX0512` follows `FX0501`–`FX0516`, the
  authority and `Root` range. `FX0801` is the only `FX08xx` code (but see
  N7).
- **The Rust claims hold** where they could be checked (Validation).
  - `sqrt` and `mul_add` are documented as correctly rounded for `f32` and
    `f64`.
  - `from_bits` is a transmute.
  - The cursor idiom compiles when the body takes a reborrow as a closure
    argument.
  - An imported `test` attribute replaces the built-in one and keeps
    `#[should_panic]`.

The notes below are new. Two of them (N1, N5) are rules a conformance test
would have to guess, and N4 is one the spike's fixtures would get wrong.
Settle those three before the corpus is written.

## fx-wsd item status

| Note | Status | Evidence |
| --- | --- | --- |
| R12 N4 floats | **Resolved** | 01 D3 (`01:325–331`), 03 PAR-3 (`03:956–960`), reference §4.3 (`reference.md:400–406`). Rust's guarantees verified. The `f32` spelling and `mul_add`'s libm are N6 and N7 |
| R12 N6 `main` | **Resolved** | `FX0512` with a safe fix, everywhere `main` is specified; the habit row cites it. Reference 12.1's "a binary root if it defines `main`" now holds with or without the attribute |
| R12 N7 `#[test]`, helpers | **Resolved** | 01 D6 (`01:389–395`), reference §2 and 2.3 (`reference.md:79–80`, `:177–181`) |
| R12 N8 Bend `by =` | **Resolved** | `#[bend_law(file, law)]` is its own attribute (`03:1160–1161`, `:1194–1196`). No other text uses `by =` |
| R12 N9 overflow under the simulator | **Resolved**, with N3 | PANIC-1's supervisor (`03:249–257`); D5 and the reference point to it |
| R12 N11 spike `FX0510` | **Resolved**, with N4 | D18 records the mapping (`01:940–944`) |
| R12 N12 reference budget | **Recorded** | Header (`reference.md:9–11`), 01 open question 8 (`01:1068–1071`), bead `fx-fbq`. The corpus does not depend on it |
| R13 N3 trait-method clauses | **Resolved**, with N5 | Grammar comment (`04:109`), rule 6 (`04:200–202`), reference table (`reference.md:1161–1162`) |
| R13 N5 interpreter reasons | **Resolved** | D18 defers to D12 (`01:933–934`); open question 5 adds spec evaluation (`01:1060–1061`) |
| R13 N1 RUST-6 tier | **Resolved**, with N7 | Tiers table v1 row (`01:83`), D13 (`01:792`), 03 tier line and §11 (`03:16–17`, `:1208`), reference Appendix A (`reference.md:1708–1709`) |
| R14 N3 reassigned `&mut` | **Resolved** for the reassigned case; see N1 | Meaning in D10, rule 4, CLAUSE-4, and the reference; spike gap in CLAUSE-7 and D18 (`04:461–465`, `01:937–939`) |
| R14 N4 README qualifier | **Resolved** | `README.md:21–23` |
| R15 N1 implicit clones | **Resolved**, with N2 | CLAUSE-3 covers entry snapshots (`04:400–401`); CLAUSE-4 clones only `Clone + Pure + Sync` (`04:431–433`) |
| R15 N2 tier mark | **Resolved** | 01 D10 says "From v1" (`01:614–616`), as 04 rule 4 does |
| R15 N3 generic fix, reflow | **Resolved** | CLAUSE-4's fix adds `T: Clone + Pure + Sync` (`04:436`); `02:266–269` reflowed |

## New blocking problems

None.

## Non-blocking notes

### N1. A bare `*x` cannot mean `final(*x)` when the borrow outlives the call, and the lowering rejects it there

The owner's rule is stated as "the caller's referent at return, as v1's
`final(*x)` does" (`01:633–637`, `04:424–428`, `reference.md:1172–1175`).
This holds where `final(*x)` is executable. 04 §4.1 (`04:729–749`) makes
`final(*x)` a prophecy in two other cases:

- the result borrows from `x`, as in `fn first(v: &mut Vec<u8>) -> &mut u8`;
- the signature lets `x`'s borrow outlive the call, as in
  `fn stash<'r>(s: &mut Slot<'r>, x: &'r mut i32)`.

There the prophecy is the value when the borrow ends, not the value at
return. Neither case can be lowered as CLAUSE-4 describes, because the
caller's reference is still borrowed at return. With the reborrow passed
as a closure argument (Validation):

- `first` with `#[ensures(v.len() == old(v.len()))]` fails with E0502;
- `stash` with `#[ensures(*x == old(*x))]` fails with E0503;
- without a clause naming `x`, both compile and run.

So in v0, a clause that names such a parameter is ill-defined. It reaches
the user as a rustc error in generated code, not as an FX code. A
conformance test would have to guess the verdict. The same split explains
why rule 4 (`04:193–194`) lets `tide fix` rewrite to `final(*x)` only where
`final` is executable. Two smaller lowering points:

- an immediately called closure with an annotated `&mut` parameter cannot
  return a borrow of it ("lifetime may not live long enough"), though a
  generic helper `call(a, |x| ..)` can;
- `&mut self` cannot be a closure parameter (E0424), so the macro must
  rename `self` or keep capturing it.

**Fix.** In CLAUSE-4 (and in D10's and the reference's sentence), scope
the rule:

> … denotes the caller's referent at return, as v1's executable
> `final(*x)` does. Where §4.1 would make `final(*x)` a prophecy (the
> result's origins include `x`'s, or the signature lets `x`'s borrow
> outlive the call), naming `x` or `*x` in an `ensures` is an error in v0
> (`FX08nn`); `old(*x)` is allowed.

Add the lowering's helper-call and `self` points to CLAUSE-7's list of
spike gaps, or to D18's.

### N2. rustc cannot tell which types are `Pure`, so the macro alone cannot pick a snapshot

CLAUSE-4 now clones a non-`Copy` parameter only if it is
`Clone + Pure + Sync`, and reads any other in place (`04:431–435`). 03 §3.1
(`03:360–362`) declares `Pure` with a blanket impl, so rustc accepts every
`Pure` bound. Round 14's autoref-specialization lowering therefore clones
every `Clone + Sync` type. A struct with an `Arc` field is one such type
(Validation): `Arc` is not `Pure` (reference 6.6), yet the lowering clones it.
The result differs from CLAUSE-4 in two ways:

- **Missed error.** CLAUSE-4 makes it an error for the body to move such a
  parameter while an `ensures` names it. The lowering clones the parameter
  instead, so rustc accepts the body.
- **Side effects.** The lowering runs the `Clone` impl of a non-`Pure`
  type. A hand-written `Clone` on a struct that holds `Fs` can do I/O. The
  debug build then differs from the erased build by more than the missing
  checks, which breaks CLAUSE-3's promise.

The rule is well defined, and Tide's checker can enforce it. Round 14's
claim that "the proc macro can decide this alone" (round 14, Resolution)
no longer holds, though: the snapshot set needs the checker's `Pure`
decision.

**Fix.** Choose one:

- have the driver feed the macro the snapshot set, round 14 N1's
  "driver-fed" lowering;
- have the admission driver reject a snapshot `Clone` on a non-`Pure` type
  wherever the checker would read it in place.

Either way, add the choice to CLAUSE-7's list of spike gaps.

### N3. PANIC-1 assumes one seed in flight per child

PANIC-1 (`03:250–255`) reports "the seed whose manifest was in flight".
This leaves three questions open.

- **Parallel tests.** Rust's test harness runs tests on several threads in
  one process. One overflow aborts the whole binary while other tests are
  still running (Validation), so several manifests can be in flight. The
  rule must say:
  - whether a child runs one simulated seed at a time;
  - or whether every in-flight seed is rerun and only the culprit is
    reported. Rust's abort message names the overflowing thread.
- **Tests without a seed.** An overflow in a plain or property test
  (`tide test`) has no replay manifest (REPLAY-1 covers simulated runs).
  The rule does not say what is reported for one.
- **What the manifest holds.** REPLAY-1 (`03:1109–1117`) lists "every
  injected input" in the manifest. A manifest written before the seed runs
  cannot hold inputs that arrive during the run. Say that the pre-run
  manifest holds the seed and the build identity, and that the rest is
  appended as the run goes.

The reference (`reference.md:1145–1149`) also omits PANIC-2's
"reported the same way".

**Fix.** Add one sentence to PANIC-1, such as:

> A child runs one test or seed at a time; `tide test` reports an aborted
> test without a seed by name.

Or state the parallel rule. Mirror it in reference §9.

### N4. The `FX0510` mapping covers only rule-specific `Pure` checks, and a spike fixture now contradicts D3

D18 (`01:940–944`) says the design reports a failed `Pure` obligation
under the code of the rule that needs it: `FX0502`, `FX0603`, `FX0606`,
`FX0611`, and `FX0612`. Most of the spike's `FX0510` fixtures are generic
`Pure` obligations:

- an impl of a `Pure` supertrait;
- an unsize coercion to `dyn PureValue`;
- a substitution for a user-written `T: PureValue`;
- a closure passed to a user `Pure` bound;
- an unbounded type parameter.

These are at `spikes/d18/admit/tests/pure/companion.rs:62–89`. None of
the five codes covers them, and 03 §3.1 and reference 6.6 give a failed
generic `Pure` bound no code. A corpus that reuses these fixtures has no
expected code for them.

Separately, `spikes/d18/admit/tests/neg/float.rs:7` expects `f64::sqrt` to
be `FX0520`, while D3 now admits it.

**Fix.**
- Give generic `Pure` obligations a code, for example in 03 §3.1: "a type
  that fails a `Pure` bound or a coercion to a `Pure` trait object is
  `FX05nn`".
- List that code in D18's mapping beside the rule codes, and add `FX0511`
  for `#[pure]`.
- Note in D18 that the spike's float fixture rejects `sqrt`.

### N5. Whether `#[invariants]` is allowed on a trait-impl method in v0 is open

Rule 6 (`04:200–202`) makes "a clause" on a trait method or trait-impl
method `FX0801`. The grammar comment (`04:109`) applies to the whole
`fn_clause` production, which includes `#[invariants]` (`04:108`). Yet
`#[invariants]` adds no clause (`04:72`), and loop invariants have nothing
to do with inheritance, which was round 12 B3's reason for v1. As written,
`invariant!` is impossible in v0 inside any trait-impl method, for example
a loop in `Iterator::next`, because the enclosing function needs
`#[invariants]` or a contract attribute (`reference.md:1165–1166`). The
comment also sits above `impl_fn`, which has `vis?` and so seems to cover
inherent methods too, though those are v0.

**Fix.** Decide whether `#[invariants]` and loop `invariant!` are allowed
in v0 trait-impl and default methods. Allowing them is the smaller rule.
Then reword the comment, for example:

> (* trait_fn, and impl_fn in a trait impl: requires/ensures are v1,
> FX0801 in v0; inherent methods are v0 *)

### N6. `mul_add` uses the platform's libm on targets without hardware FMA

D3 says "math functions use one libm on every target" (`01:318–320`).
PAR-3 keeps "never the platform's libm" for functions beyond IEEE's basic
operations (`03:955–957`). It now admits `mul_add` as a basic operation.
On baseline `x86_64-unknown-linux-gnu`, rustc 1.96 compiles `x.mul_add(a, b)`
to a tail call to the C library's `fma` (`jmpq *fma@GOTPCREL`). With
`-C target-cpu=haswell` it becomes `vfmadd213sd`. Rust guarantees the
correctly rounded result, so the text is right in letter. But on such
targets, cross-target reproducibility rests on each platform's `fma` being
correct, which D15 otherwise avoids relying on. `sqrt` compiles to an
instruction (`sqrtsd`) and has no such issue.

**Fix.** Either say in PAR-3 that `mul_add` relies on the target's
correctly rounded `fma`, or route it through std's math library on targets
without FMA.

### N7. Nits

- **`f32` spelling.** "Each for `f32` and `f64`" (`01:327`, `03:957`,
  `reference.md:404`) needs one name for two types. D3 rejects overloading
  (`01:333`), so the functions must be generic, for example
  `float::bits<F: Float>(x: F) -> F::Bits`. Say so, or mark it provisional
  in reference §4.3. `float::bits` of an `f32` returns `u32`.
- **Reference header.** The method (`reference.md:10–11`) omits three
  details that bead `fx-fbq` records: the model, the fixed system prompt,
  and running with no tools. Without them the count cannot be repeated.
  The header also does not name the revision that was measured: 39,140
  tokens is at `9bac29e`, while the bead's 38,797 is at `c15790c`. Also,
  "counted in CI" (`reference.md:9`, `01:205`, `01:821`) describes a check
  that does not exist yet. Mark it as planned.
- **Open question 8** (`01:1070–1071`): "E1's held-out round does (bead
  `fx-fbq`)" reads as if `fx-fbq` were the E1 bead, which is `fx-jv9`.
  Write "the restructure is bead `fx-fbq`".
- **Habit row** (`reference.md:1451`): the fix column shows
  `#[tide::main] fn main(root: Root)` for both inputs. The safe fix for
  `fn main()` adds only the attribute.
- **Untiered Rust consumers.** These still mention Rust consumers without
  a tier:
  - 01's Positioning (`01:61–63`, "Rust code depends on it directly");
  - 03 §3.1's "Unchecked callers" (`03:403–406`).

  Neither has a tier mark, and the README's v1 list (`README.md:146–147`)
  omits them. Add "(v1)".
- **`FX0801`'s history.** Until draft 3 (`23b6040`), the reference used
  `FX0801` for the bare-`*x` error. That rule returns in v1 (04 rule 4)
  with no code. Nothing live collides, but a different number for the
  trait-method rule would keep old reviews readable.

## Ready to implement v0

**Yes.** Every fx-wsd note is fixed or, for N12, recorded with its reason.
The owner's four decisions read the same in 01, 03, 04, and the reference,
and no old rule survives in another document.

Before the checker's test corpus is written, settle:
- N1's v0 verdict for a `&mut` parameter whose borrow outlives the call;
- N4's code for generic `Pure` obligations;
- N5's rule for `#[invariants]` on trait-impl methods.

Also settle N3's reporting rule before the corpus has a test for an abort.
N2 and N6 are implementation choices.

Validation: this was a read-only review of `9bac29e` (diffed against
`c15790c`) on 2026-10-19. Programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in
`/tmp/claude-1000/r16`.

- **Floats.** In rustc 1.96's `rust-src`, `std/src/num/f64.rs` documents
  both `sqrt` and `mul_add` as "guaranteed to be the rounded
  infinite-precision result", specified by IEEE 754 as `squareRoot` and
  `fusedMultiplyAdd`. `f32.rs` says the same. `core/src/num/f64.rs`
  documents `from_bits` as identical to `transmute::<u64, f64>`. The
  compiled code is N6:
  - `mul_add` is `jmpq *fma@GOTPCREL` at baseline and `vfmadd213sd` with
    `target-cpu=haswell`;
  - `sqrt` is `sqrtsd`.
- **Reborrow lowering** (the owner's R14 N3 check, repeated):
  - `push_back(mut cur: &mut Option<Box<Node>>, v)` was lowered as
    `let __caller = cur; (|mut cur: &mut _| body)(&mut *__caller)`, with
    the clause reading `len(&*__caller)` against an entry snapshot. It
    compiled and the clause held.
  - So did a variant with an early `return` in the body.
  - Output: `len 4 false true`.
- **N1.**
  - `first(v: &mut Vec<u8>) -> &mut u8` with an annotated closure: "lifetime
    may not live long enough". With a generic `call(a, |v| ..)` helper it
    compiled (`7`). With the clause reading `__caller.len()`: E0502.
  - `stash<'r>(s: &mut Slot<'r>, x: &'r mut i32)`: compiled (`2`). With the
    clause reading `*__caller`: E0503.
  - A closure parameter named `self`: E0424.
- **N2.** Under a blanket `unsafe impl<T: ?Sized> Pure for T {}`, the
  autoref probe with `T: Clone + Pure + Sync` chose clone for a struct
  with an `Arc<Vec<u8>>` field. It chose the borrow for one with an `Rc`
  field.
- **N3.** In a three-test libtest binary, one test overflowed its stack
  while the other two slept. The process aborted ("thread 'b_overflow' has
  overflowed its stack") before the other two finished.
- **R12 N7.** A proc-macro attribute named `test`, imported with
  `use tmac::test;`, replaced the built-in. It forwarded
  `#[should_panic(expected = "boom")]` to `::core::prelude::v1::test`, and
  it turned a test with parameters into a wrapper test. Result:
  `3 passed`.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the commit that adds this review. The owner decided N1 and N3
(2026-10-19, recorded on bead fx-wsd).

- **N1:** where 04 §4.1 makes `final(*x)` a prophecy, naming `x` or `*x`
  in an `ensures` is `FX0803` in v0, and `old(*x)` is allowed (04 CLAUSE-4
  and rule 4, 01 D10, reference §10.1). CLAUSE-7's spike gaps name the
  generic helper call and the renamed `self`.
- **N2:** recorded as implementation work in CLAUSE-7's spike gaps and
  D18's list: the lowering needs the checker's `Pure` decision to choose
  snapshots.
- **N3:** a child may run tests and seeds in parallel. After an abort the
  driver reruns each one in flight alone and reports each that aborts alone,
  by seed and replay command or by test name. The pre-run manifest holds the
  seed and build identity, and REPLAY-1's other contents are appended during
  the run (03 PANIC-1, REPLAY-1, reference §9, which now covers PANIC-2).
- **N4:** a failed `Pure` obligation that no rule gives its own code is
  `FX0517` (03 §3.1, reference 6.6). D18's mapping lists it and `FX0511`,
  and notes that the spike's float fixture rejects `sqrt`.
- **N5:** `#[invariants]` and loop `invariant!` are v0 in trait-impl and
  default methods. Only `requires` and `ensures` there are rejected (04
  rule 6 and the grammar comment).
- **N6:** on a target without hardware FMA, `mul_add` calls std's math
  library, never the platform's `fma` (03 PAR-3).
- **N7:**
  - the float functions are generic over `f32` and `f64`, with signatures
    provisional;
  - the reference header records the full method and measures the current
    text;
  - CI counting is marked as planned;
  - open question 8 names `fx-fbq` as the restructure;
  - the habit row's fix adds only the attribute;
  - Rust consumers are marked v1 in 01 Positioning, 03 §3.1, and the
    README;
  - the trait-method code is `FX0802`, leaving draft 2's `FX0801` unused.
