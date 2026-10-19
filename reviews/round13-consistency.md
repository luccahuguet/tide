# Tide consistency review, round 13: confirmation of the round-12 answers

## Verdict

**request_changes**

Reviewed commit `e84b2e023879616af305ff6143ce8b513cf4fc99` (`e84b2e0`, "Add
round 12 and answer it: consumer std injection, trusted_pure tier, trait
contracts to v1"), dated 2026-10-19 18:04 -0300. It was diffed against
`fc4cb4c`, which applied D18 and was read for context. The review covers
every passage the commit changed:

- `design/00-first-principles.md` and `design/01-design.md`;
- `design/02-memory.md`, `design/03-authority-concurrency.md`, and
  `design/04-contracts.md`;
- `design/reference.md` and `README.md`.

It also sweeps 01, the chapters, and the reference for D18 leftovers:
spellings that are not Rust, tier mismatches, and old backends.

What holds:

- **Round 12's B1, B2, B3, B5, and B6 are closed** consistently in 01, the
  chapters, and the reference (table below).
- **B6's mechanism works in Rust.** On rustc 1.96 a proc-macro attribute
  named `requires` rewrote and removed `requires!(..)` inside a closure and
  `invariant!(..)` inside a `for` loop and a `loop`. Neither `!` form
  resolves to any macro. Output: `3 10`.
- **The D18 sweep found no other v0-blocking contradiction.**
  - No `tide` fence remains in 01, 02, 03, or the reference. The eight in 04
    are v1 proof examples under the draft 3.5 grammar label.
  - `dyn Trait + Pure` and `dyn Trait + ThreadBound` appear only as rejected
    spellings (01:460, :468; 02:462; 03:381, :640; reference:661).
  - Interpreter, Cranelift, and C-backend text appears only as history
    (01:756–760), as a conditional (01:738–741, :798; 03:998), or as the
    lapsed open question 7.
  - `#[tide::main]` is spelled the same in all four documents.
  - `#[trusted_pure]` now has one tier split everywhere: types in v0,
    foreign functions and Rust-import wrappers in v1.
- **The new spellings checked here compile.**
  `f: impl FnOnce(&mut T) -> R + Pure` in argument position (03 §6.5)
  compiles. So does a blanket `unsafe impl<T: ?Sized> Pure for T {}`.

One problem remains:

- **B1 (v0).** Round 12's B4 is only partly closed. The new CLAUSE-4
  sentence snapshots a by-value parameter only when the body "assigns or
  moves" it. A body can also change a parameter without doing either, for
  example `x.push(1)` or a write through a `&mut` field. In that case the
  clause still reads the value at return. When the parameter is not `Clone`,
  its entry value still has no defined meaning, which was the hole B4
  named. The sentence also leaves unclear how a reassigned `&[T]` parameter
  is snapshotted.

## Round-12 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | D13 (`01-design.md:780–785`): "their own manifest and `std` are unchanged. The Tide crate's exported `Cargo.toml` (D18) declares `std = { package = "tide_std" }`, so Cargo builds that crate, and only it, against Tide's std". RUST-6 (`03-authority-concurrency.md:1237–1240`) and `reference.md:1400–1402` say the same. This matches the spike's probe (`spikes/d18/examples/cargo-rename/Cargo.toml`, where the renamed dependency sits in the facade-built crate) and `research/15-rust-dialect.md:640–644`. D18 (`01:919`) still calls the generated `Cargo.toml` "an export for Rust consumers", which agrees. |
| B2 | **Resolved** | The v0 row of the Tiers table (`01:82`) adds "`#[trusted_pure]` on types as a trust hole under an `unsafe` grant". The v1 row (`:83`) keeps "FFI with `#[trusted_pure]` wrappers". 03's tier line (`:14–15`) now says "on foreign functions". These agree with 03 §3.1 (`:355–358`), 02 §9 (`:788–792`), reference §2.3 (`:177`) and §11 (`:1296`), and 01 D7 (`:462–465`). Every v0 mention requires an `unsafe` grant. |
| B3 | **Resolved**, option (a) | 01 D10 (`:659`): "Trait methods may carry contracts (v1)". The v1 tier row (`:83`) lists "trait-method contracts and their inheritance by impls". 04 §1.1 (`:71`) reads "a function or inherent method; v1: a trait method or impl method", and rule 6 (`:196`) is marked v1. The reference drops the v0 line and adds the feature to Appendix A (`:1688–1689`). No reference example puts a clause on a trait method. Two small leftovers are N3. |
| B4 | **Partly resolved** | CLAUSE-4 (`04-contracts.md:422–425`) adds the implicit `old(..)`. The spike-gap lists name it (`04:448–450`; `01:924–927`). The rule's trigger and its treatment of shared references are new B1. |
| B5 | **Resolved** | Thesis: "slow edit loops" (`01:24`) and "a much faster edit loop" (`01:54`). D12 (`01:743–747`) says builds and debug-build runtime are "set by E4 against rustc's debug profile", and gives the earlier targets as history. `00-first-principles.md:126` matches. The 1 s edit-to-test target stays, and only the conditional interpreter (D12, open question 5) can meet it; round 12 accepted that. |
| B6 | **Resolved** | 01 D10 (`:599–602`), 04's Spelling paragraph (`:29–33`), and reference §2.1 (`:97`) all say the `!` clause forms are rewritten by the enclosing attribute and are not exported. Compiled here: see Validation. One wording leftover is in N5. |
| N1 | **Resolved** | `README.md:35`, `:38–39`, `:68`. |
| N2 | **Resolved** | The Feel's `max_index` no longer uses `decreases!`, and its doc comment marks `prove` as v1 (`01:985–1000`). |
| N3 | **Partly resolved** | Channel ends are added in D13 (`01:785`) and in `reference.md:1402`. They are missing from RUST-6's list (`03:1241–1243`). See N1. |
| N5 | **Resolved** | 04 §2.6 (`:590–591`). Counterexample replay (`01:740–741`). D1 cites the reference (`01:175`), but see N5 below. D18's build-script clause (`01:916–917`). |
| N10 | **Resolved in 01** | `01:926` adds `loop {}`. 04's gap lists (`:448–450`, `:532–533`) do not mention it (N5). |
| N11 | **Partly resolved** | Fixed: 01 D4 (`:343`), 02 §2.7 (`:267`), 03's two fences, and open question 7's indentation. Still saying "auto trait" or treating `Owned` as a trait: `02:267`, `02:802`, `reference.md:423` (N2). |
| N4, N6–N9, N12, spike `FX0510` | **Open, as recorded** | None is a contradiction between documents. Each needs a choice or a measurement before the checker's test corpus or E1, as round 12 said. N6 and N9 affect v0 behavior but need only a rule, not a change to any decision, so they do not block. |

## New blocking problems

### B1. CLAUSE-4's implicit snapshot misses by-value parameters changed through a mutable borrow

**Location.**
- `design/04-contracts.md:421–425` (CLAUSE-4): "other parameters denote
  their entry values. A by-value parameter that the body assigns or moves
  is read at entry through an implicit `old(..)`, so it must be `Copy` or
  `Clone`; otherwise naming it in an `ensures` is an error".
- `04-contracts.md:409–410`: "A reference is snapshotted as its referent".
- `04-contracts.md:187–188` (§1.2 rule 4): "A by-value or `&T` parameter
  denotes its value at entry."
- The reference does not state the rule. §10.1 (`reference.md:1162–1171`)
  defines only `&mut` parameters and `old`.

Tier: v0. §1.8 is "the one normative statement of what a v0 clause means"
(`04:12`).

**Problem.** "Assigns or moves" does not cover every way a body changes a
by-value parameter. Two shapes compile on rustc 1.96 without either:

- **A method that takes `&mut self`.**
  `fn bump(mut x: Vec<u8>) -> usize { x.push(1); x.len() }`. No assignment
  and no move. Under `#[ensures(result == x.len() + 1)]`, CLAUSE-4 says `x`
  is the entry value, so the clause holds. The trigger does not fire, so a
  lowering that follows the text reads `x` at return, as the spike's does,
  and reports a violation.
- **A write through a `&mut` the parameter holds.**
  `fn grow(s: Cursor<'_>) -> usize { s.buf.push(1); s.buf.len() }`, where
  `struct Cursor<'a> { buf: &'a mut Vec<u8> }`. The binding is not even
  `mut`. `Cursor` cannot be `Clone`. So `s.buf.len()` in an `ensures`
  "denotes its entry value", which cannot be computed, and the new sentence
  does not make it an error. This is round 12's B4 hole through a different
  door.

The new sentence is also ambiguous about shared references. A reassigned
`mut xs: &[u8]` is a common Rust idiom for advancing a slice
(`xs = &xs[1..]`). The parameter's type is `Copy`, so the new sentence
allows it. But `old(..)` "snapshots a reference as its referent", and `[u8]`
is neither `Copy` nor `Clone`. One reading accepts the clause and the other
rejects it. The cheap and correct snapshot copies the reference itself:
while the call runs, the referent cannot change.

**Proposed fix.** Replace the last sentence of CLAUSE-4
(`04-contracts.md:422–425`) with:

> A by-value parameter that the body assigns, moves, or mutably borrows
> (including any place reached through a `&mut` it holds) is read at entry
> through an implicit snapshot: a `Copy` parameter, a shared reference
> included, is copied as is, and any other is cloned. If it is neither
> `Copy` nor `Clone`, naming it in an `ensures` is an error whose fix
> narrows it, as in `old(x.len())`.

Add one bullet to reference §10.1 after the `&mut` sentence (`:1165`):

> A by-value parameter means its entry value; if the body moves or mutates
> it, it is snapshotted at entry and must be `Copy` or `Clone`, else write
> `old(x.len())`.

## Non-blocking notes

### N1. RUST-6 misses channel ends, and its tier is unstated

- **Channel ends.** `03-authority-concurrency.md:1241–1243` lists the
  Tide-owned types as "`HashMap`, `HashSet`, `Arc`, the capability types".
  D13 (`01:785`) and `reference.md:1402` now include channel ends. Copy
  them into RUST-6.
- **Tier.** `03:1195` says "Imports (RUST-1 to RUST-5) are v1; RUST-6 is
  the other direction", but neither 03's tier line (`:10–17`) nor 01's
  Tiers table places Rust consumers in a tier. The generated `Cargo.toml`
  export is driver work. State whether it ships in v0.

### N2. "Auto trait" leftovers, and `Owned` listed as a trait

- **`02-memory.md:802`.** "Coherence never treats a missing auto trait
  (`Send`, `Sync`, `Pure`, `Owned`)". `Pure` is a marker trait, and under
  D18 `Owned` is `'static`, not a trait (02 §3.7).
- **`02-memory.md:267`.** "the auto and marker traits (`Owned`, `Send`, …)"
  has the same `Owned` problem.
- **`reference.md:423`.** "A missing auto trait never makes impls disjoint:
  `impl<T: Pure> K for T`".

**Fix.** Say "auto or marker trait", and write `Owned` as "`'static`
(`Owned`)" or move it out of the trait list.

### N3. B3 leftovers: the grammar and the reference's v0 rejection

- **The grammar.** 04 §1.2's `trait_fn` and `impl_fn` productions
  (`:109–112`) take `fn_clause` with no tier mark. The table at `:71` does
  carry one. Add `(* v1 *)` to `trait_fn` and to trait-impl methods.
- **The reference.** §10.1's table (`reference.md:1153–1154`) says
  "attribute on a fn". An agent will read a trait method as a fn. State
  that clauses on trait methods and trait-impl methods are rejected in v0,
  and give the FX code.

### N4. Does an implicit snapshot count as `old` when a clause is classified?

CLAUSE-5 (`04:426–433`) calls a clause `checked` if it uses "`old` of
`Copy` values" and std accessors. Classes "depend on syntax, not cost".
Under B1's rule, `#[ensures(result == v.len())]` on a body that consumes
`v: Vec<u64>` contains no `old`, so it is `checked`. Yet it clones the whole
`Vec` at entry on every debug-build call. Two fixes would work:

- say that an implicit snapshot counts as `old(x)` for CLAUSE-5;
- or say that the lowering snapshots only the accessor path the clause
  reads (`v.len()`).

The second keeps the clause cheap and `checked`.

### N5. Small leftovers

- **D1's citation.** `01:175` cites "`design/reference.md` §12.1" for both
  `#[path]` and `extern crate`. `extern crate` (`FX0912`) is in §12.2
  (`reference.md:1325–1326`).
- **04's spike-gap lists omit `loop {}`.** CLAUSE-7 (`04:448–450`) and §2.4
  (`:532–533`) do not mention it, though D18 (`01:926`) does.
- **D18 and D12 give different reasons for an interpreter.** D18
  (`01:921–922`) says an interpreter is built only if "`tide eval` or E4's
  edit-to-test target needs it". D12 (`01:738–740`) adds "v1's spec
  evaluation (`by(compute)`, spec tests)".
- **Reference §2.1's intro.** `reference.md:86` says "Tide's std supplies
  exactly these". The `invariant!` row (`:97`) now says it is "not a macro
  you import". Reword the intro to "Tide's std or the contract attributes
  supply exactly these".
- **The README contradicts D6 and D13.** `README.md:21–22` says "nothing
  runs at build time" and "no proc macros". D6 (`01:384–386`) and D13
  (`01:764–765`) have the toolchain's own proc macros run at build time.
  Say "no user proc macros". The README's review history also stops at
  round 8.

## Ready to implement v0

**Yes, after B1.** B1 replaces one sentence of CLAUSE-4 and adds one bullet
to the reference.

- **B2, B3, B5, and B6** of round 12 are closed.
- **B1** of round 12 is also closed. It was the only one that changed
  interop.
- Spellings, tiers, codes, and clause semantics now agree across 01, 02–04,
  and the reference under D18.
- **N1–N5** are wording and tiering fixes. Settle N3's FX code, N4, and the
  round-12 items still open (N4, N6, N7, N9) before the checker's test
  corpus is written.

Validation: this was a read-only review of `e84b2e0` (diffed against
`fc4cb4c`) on 2026-10-19. Rust programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in a scratch
directory outside the repository (`/tmp/claude-1000/r13`).

- **New B1.** One program compiled and printed `2 3 1`:
  - `fn grow(s: Cursor<'_>)` pushed through `s.buf: &mut Vec<u8>` with a
    non-`mut` binding;
  - `fn bump(mut x: Vec<u8>)` changed `x` only through `x.push(1)`;
  - `with(v, f: impl FnOnce(&mut T) -> R + Pure)`, under a blanket `Pure`,
    was called with a closure.
- **Round 12's B6.**
  - A `proc_macro_attribute` named `requires`, built as a proc-macro crate,
    stripped `requires!(..)` from a closure body and `invariant!(..)` from
    a `for` body. An `#[invariants]` attribute stripped `invariant!` from a
    `loop` body.
  - No macro named `requires` or `invariant` was in scope as a `!` macro.
  - The program compiled and printed `3 10`.
- **Round 12's B1** rests on the committed spike probe
  (`spikes/d18/examples/cargo-rename/Cargo.toml`) and research/15:640–644.
  The probe was not rerun.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only `reviews/round13-consistency.md` was written; nothing was staged
or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** CLAUSE-4 now copies a `Copy` by-value parameter (shared
  references included) as it is, and snapshots a non-`Copy` one that the
  body assigns, moves, or mutably borrows (including through a `&mut` it
  holds) through an implicit `old(..)`, which must be `Clone`; reference
  §10.1 states the same rule.
- **N4:** the implicit snapshot counts as `old` for CLAUSE-5.
- **N1, N2:** RUST-6 lists channel ends; 02 §9 and reference §4 say "auto or
  marker trait" (02 §2.7 already did).
- **N5:** D1 cites reference §12.1–§12.2; 04 §2.4 names the spike's
  `loop {}` gap; reference §2.1's intro no longer says std supplies every
  form; the README no longer claims no proc macros or nothing at build time.
- **Left open:** N3 (tier marks on 04's trait-method grammar rules and a v0
  rejection code), the D18/D12 interpreter wording, RUST-6's tier, and
  round 12's open notes.
