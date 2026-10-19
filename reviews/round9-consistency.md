# Tide consistency review, round 9: Rust 2024 lifetimes

## Verdict

**request_changes**

Reviewed commit `295731e10599cf4bc80c7ee5cb063ff26919dc5c` (`295731e`,
"Take Rust 2024 lifetimes whole, answering round 8") on 2026-10-19, diffed
against `af8a99c`.

What holds:

- **Round 8 is answered.** B1–B4 and N2–N6 are resolved. N1 and N7 are
  resolved as stated, and N8 is resolved except for one slip. Statuses are
  in the next section.
- **No leftover rules from draft 3.4's narrower subset.** No normative text
  in the reviewed sections describes Rust 2018 capture, cites `FX0204`, bans
  `'a: 'b` or `+ use<..>`, or uses the old `Scope<E>`, `Scope<'_, E>`, or
  `Task<U, E>` arity. The remaining mentions are historical: D2's Rejected
  paragraph (`design/01-design.md:245–248`) and the second paragraph of 02
  §2.6 (`design/02-memory.md:244–247`). Outside the reviewed files,
  `REFERENCES.md:18` still lists `'a: 'b` among the Rust features Tide
  rejects (N5).
- **The declared `'env: 's` is enough on its own.** The rustc mock of 02
  §8.2 makes `'s` and `'env` invariant, as std does
  (`PhantomData<&'s mut &'s ()>`, `PhantomData<&'env mut &'env ()>`).
  No field implies `'env: 's`; only the declaration
  `struct Scope<'s, 'env: 's, E>` states it.
  - Every case the documents accept compiles under editions 2024 and 2021.
  - Every case they reject is rejected.
  - Deleting the declaration gives 20 errors (E0597, E0503, E0521), as
    deleting the implying field did in round 8.

  So N5's `PhantomData` point is moot.
- **The examples match rustc 1.96 under edition 2024.** This covers 02 §2.4's
  `pick`, 02 §5.2's `evens` and `len_of`, reference §3.5's `evens` and
  `over`, every verdict in 02 §10.1–§10.8 and §10.11, and 02 §10.12's four
  functions. The details are in the validation note.

The commit admitted declared outlives bounds and Rust 2024 capture but left
two older rules unchanged. Each now contradicts the new rules:

- **B1** is the executable-`final` test in 04 §4.1. It is unsound, and
  `final` is v1.
- **B2** is the body rule in 02 §2.7. It is v0 text, and the fix is one
  sentence.

## Round-8 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** (option b: accept `'a: 'b`) | `design/01-design.md:199–201`, `design/02-memory.md:121–123`, `:168–171`, `:188–197`, `design/reference.md:230–232`. The invariant-position cost and FX0201's `'a: 'b` entry are gone (`01-design.md:254–260`, `reference.md:1392`). The thesis reads "keeps Rust 2024's lifetimes" (`01-design.md:19–20`). rustc accepts `pick` and round 8's caller. New B2 is text this change missed. |
| B2 | **Resolved** (`+ use<..>` and Rust 2024 capture) | `01-design.md:201–203`, `02-memory.md:448–455`, `:1210–1211`, `reference.md:301–305`. FX0204 and its habit row are gone. Round 8's `Lexer::lens` compiles with no annotation under 2024; under 2021 it is E0700. |
| B3 | **Resolved** | D2 (`01-design.md:219–222`), 03 §0 and the §1.2 table (`03-authority-concurrency.md:36`, `:113–114`), reference §7.1 (`reference.md:874`). The mock confirms the arity and the bound. |
| B4 | **Resolved** | `02-memory.md:1156–1163`, `:1184–1187`. rustc reports E0373 for `buf` and for `conn` in `serve`, and accepts `serve_owned`. |
| N1 | **Resolved** | (a) The `Fn` output clause is gone from §2.4 item 3 (`02-memory.md:195–197`), and `spawn` has `U: Send + 's` (`02-memory.md:637`, `03-authority-concurrency.md:595`, `reference.md:846`). `E: 's` was not added. It is not needed, because `E` is fixed by `scope`'s caller and outlives the call. (b) Rust's container default is adopted (`02-memory.md:435–436`, `reference.md:290–291`), and rustc accepts round 8's `W` example. (c) `use<..>` is now Tide's own form. (d) §2.6 says "except the rows §11.1 marks" (`02-memory.md:235–237`). D12's "nearly the identity" (`01-design.md:653`) is compatible. New N2 covers object defaults in bodies, which round 8 missed. |
| N2 | **Resolved** by B1's option (b) | The cost is deleted from D2, from 02 §12 item 6 (`02-memory.md:1264–1266`), and from reference §3.2, which shows `pick` with declared bounds. |
| N3 | **Resolved** | "Owned only when every captured parameter is" and "Type parameters are captured as in Rust" (`02-memory.md:450–451`, `:455`). D2 no longer says "with none it is `Owned`". New N1 is a gap in the replacement sentence. |
| N4 | **Resolved** | "signature types" (`01-design.md:201`, `reference.md:233`). |
| N5 | **Resolved; (c) moot** | (a) `02-memory.md:203–204`. (b) `02-memory.md:254`. (c) `Scope` declares `'env: 's` (`02-memory.md:635`, `01-design.md:220–221`, `02-memory.md:491–492`), as std's `Scope<'scope, 'env: 'scope>` does. No `PhantomData` field is needed, and §11.2 R2's "the encoding that `std::thread::scope` relies on" (`02-memory.md:1231–1232`) is now accurate. The variance is stated (`02-memory.md:635`). |
| N6 | **Resolved** | `02-memory.md:165–167`, `:226–228`, `reference.md:188–190`, `:1393`. rustc 2024 rejects a bare `impl Lexer` with E0726 and accepts `impl Collector<'_, '_>`. |
| N7 | **Resolved as stated; B1 reopens it** | `04-contracts.md:671–674`, `:688–691`. Round 8's `RefCell` `stash` now fails the second test. |
| N8 | **Partly resolved** | Resolved: the sources of higher-ranked lifetimes (`02-memory.md:174–175`, `:482–483`, `01-design.md:298–299`), `Shared::with` (`02-memory.md:710–711`), "Rejected" (`02-memory.md:888`), and untested forms (`01-design.md:259–260`, `02-memory.md:1275–1276`). Left: "`T: 'static` is accepted" still sits in the rejected-forms bullet (`01-design.md:207–208`, also `02-memory.md:176–177`, `reference.md:237–238`). |

## New blocking problems

### B1. 04 §4.1's executable-`final` test misses borrows that escape through `'r: 'q` or a captured lifetime

**Location:** `design/04-contracts.md:669–674`, `design/04-contracts.md:688–691`;
against `design/01-design.md:199–203`, `design/02-memory.md:168–171`,
`design/02-memory.md:195–197`, `design/02-memory.md:448–454`.

**Problem.** `final(*x)` is executable "only if the origin of `x`'s outer
reference, named or given by elision, occurs neither in the result type nor
in the type of any other parameter". The test checks only where that lifetime is
written. This commit admits two ways for `x`'s borrow to outlive the call
without the lifetime being written in either place:

- **A declared bound.** With `'r: 'q`, `x: &'r mut i32` coerces to
  `&'q mut i32`, and `'q` may appear in the result or in another parameter.
- **Rust 2024 capture.** A returned `impl Trait` captures `x`'s elided
  lifetime without naming it (02 §5.2).

The closing claim, "Prophecies arise only from results that carry a `&mut`
parameter's origin, a `&mut` parameter's origin shared with another
parameter's type, and borrowing types with `&mut` fields", fails for both.

**Scenario.** Each of these passes both tests, so `final(*x)` is executable
and equals 5. rustc 1.96 (edition 2024) accepts all four.

```tide
fn give<'q, 'r: 'q>(x: &'r mut i32) -> &'q mut i32 { *x = 5; x }

fn stash<'q, 'r: 'q>(s: &RefCell<Option<&'q mut i32>>, x: &'r mut i32) {
    *x = 5;
    *s.borrow_mut() = Some(x);
}

fn setter(x: &mut i32) -> impl FnMut(i32) { *x = 5; move |v| *x = v }

fn bump<'s, 'r: 's>(bg: Scope<'s, '_, TaskError>, x: &'r mut i32) {
    *x = 5;
    bg.spawn(move || { *x = 7; Ok(()) });
}
```

- **What callers do.** Each caller writes 7 through the result, the cell, the
  closure, or the child before the borrow ends.
- **What the program prints.** The built program reads `v` after each borrow
  ends and prints `7 7 7 7`. (The `bump` case was checked in the `Scope`
  mock; the run covers the first three plus an
  `impl DerefMut<Target = i32>` variant.)
- **Why this is unsound.** A `prove`-mode caller that takes `final(*x) == 5`
  as the value of `v` once the call returns believes something false.
- **Why it is new.** Under `af8a99c`, `give`, `stash`, and `bump` could not
  be written (FX0201). `setter` needed `+ '_`, which put the lifetime in the
  result type (rustc 2021 reports E0700 without it).

A related case predates this commit: the lifetime can occur only in a type
parameter's bound. rustc accepts
`fn stash_generic<'r, S: Sink<&'r mut i32>>(s: &S, x: &'r mut i32) { *x = 5; s.put(x); }`
with a `RefCell`-backed `Sink`.

`final` is v1 (`design/01-design.md:77`), so this does not block the v0
checker. But the normative text is unsound as of this commit.

**Proposed fix.** Test the origin, not where it is spelled. Replace the
condition at 04:671–674 with:

> `final(*x)` is executable only if nothing else in the signature can hold
> `x`'s borrow past the call: the origin of `x`'s outer reference occurs in
> no other parameter type and in no bound or where clause; it is not in the
> result type's origins, counting the lifetimes a returned `impl Trait`
> captures (`02-memory.md` §5.2); and no declared or implied outlives bound
> relates it to another origin (`'r: 'q`).

Then:

- Add `give` beside `stash` as a failing signature.
- Reword 04:688–691: "Prophecies arise only from results that carry or
  capture a `&mut` parameter's origin, a `&mut` parameter's origin that the
  signature relates to another parameter (one name, a bound, or `'r: 'q`),
  and borrowing types with `&mut` fields."
- Give 02 §2.2's `orig` table (`design/02-memory.md:129–137`) a row that the
  new test can cite: the origins of a returned `impl Trait` are its captured
  lifetimes plus the `orig` of its captured type parameters.

The fix also closes the older `stash_generic` case.

### B2. 02 §2.7 still lets a body rely only on implied bounds

**Location:** `design/02-memory.md:261–264`; against
`design/02-memory.md:121–123`, `design/02-memory.md:188–197`,
`design/01-design.md:199–201`, `design/reference.md:230–232`.

**Problem.** The Bodies bullet reads: "Implied bounds come from the
well-formedness of the parameter and return types. Any other inclusion the
body needs between placeholders is an error, which names the lifetime to add
or share." This commit left it unchanged.

- **What §2.7 says.** A declared `'a: 'r` is "other", so a checker that
  follows §2.7 rejects any body that relies on one.
- **What the rest says.** 02 §2.1 allows inclusions "through a declared
  outlives bound or the well-formedness of its types". 02 §2.4 item 2 and
  reference §3.2 give `pick` as the way to write a result that borrows
  through an invariant position.

**Scenario.**
`fn pick<'r, 'a: 'r, 'b: 'r>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str { v.first().copied().unwrap_or(s) }`
needs `ρa ⊆ ρr` and `ρb ⊆ ρr`, which only its declared bounds give.

- rustc 1.96 accepts it under editions 2024 and 2021, together with round
  8's caller, which drops `s`'s owner and then pushes to `v`.
- With the bounds removed, rustc reports "lifetime may not live long enough"
  twice.

Read literally, §2.7 gives the second verdict for the first program.

**Proposed fix.** Replace the two sentences with:

> Inclusions between placeholders come from the signature's declared
> outlives bounds and from the well-formedness of its parameter and return
> types (§2.1, §2.4). Any other inclusion the body needs is an error; its
> fix suggests the bound (`'a: 'b`) or a shared lifetime, as rustc does.

## Non-blocking notes

### N1. Is a returned `impl Trait + Owned` `Owned`? §5.2's two bullets disagree

`design/02-memory.md:450–451` against `design/02-memory.md:456–458`,
`design/02-memory.md:254`, and `design/02-memory.md:763–766`.

**The disagreement.** Under Rust 2024 capture,
`fn make(x: &str) -> impl Fn() -> usize + Owned` captures `x`'s lifetime.

- §5.2's new sentence says an opaque type "is `Owned` only when every
  captured parameter is", so `make`'s result is not `Owned`.
- The next bullet says an opaque type has the auto traits its bounds state.
  §2.7 and §9 list `Owned` among the auto traits, so the result is `Owned`.

**What Rust does.** `+ 'static` makes the result `'static`. rustc 2024
accepts passing `make(&s)` to a `T: 'static` parameter after `s` has died.

**Why it is new.** Under `af8a99c`, `+ Owned` named no lifetime, so nothing
was captured and the two bullets agreed.

**Fix.** Write: "It is `Owned` when every captured parameter is, or when its
bounds state `Owned` (Rust's `+ 'static`); `+ use<>` captures nothing."
Also say that the auto-trait bullet covers `Send`, `Sync`, `Pure`, and
`ThreadBound`, and that `Owned` follows captures, as Rust's `'static` does.
That also settles a question that predates this commit: whether
`fn count() -> impl Iterator<Item = u32>`, which captures nothing and states
no `Owned`, is `Owned`.

### N2. Object-lifetime defaults inside a body are not Rust's

`design/02-memory.md:434–436`, `design/reference.md:288–291`; against
`design/01-design.md:192–195`, `design/01-design.md:216–218`,
`design/02-memory.md:1210`, `design/reference.md:175–177`.

**The contradiction.** D2 and reference §3 now say that object-lifetime
defaults "mean what they mean in Rust 2024", and §11.1 calls them "the
same". But Tide still defaults some in-body positions to `dyn Trait + Owned`:

- 02 §5.1 does so for "annotations on `let`" and "type arguments";
- reference §3.4 does so for "type arguments".

**What rustc does.** Inside a body, Rust infers the bound. rustc 1.96
(editions 2024 and 2021) accepts each of these when the closure borrows the
local `s`:

- `let b: Box<dyn Fn() -> usize> = Box::new(|| s.len());`
- the same closure in a `Vec<Box<dyn ..>>` or an `Rc<dyn ..>` annotation;
- the same closure in an `as` cast.

It rejects only the struct-field form (E0597).

**Why it matters.** Tide is stricter here, so it stays sound, and the
wording predates this commit. The commit's "the same" turns it into a
contradiction. It also breaks a common Rust habit, a `let` vector of boxed
callbacks that borrow locals.

**Fix.** Two options:

- **Recommended (D2).** In §5.1 and reference §3.4, write "in a body (`let`
  annotations, casts) the bound is inferred, as in Rust".
- **Alternative.** Keep `Owned` in bodies, list the difference in §11.2, and
  qualify D2.

### N3. D4 does not list the trait-lifetime exclusion that D2 attributes to it

`design/01-design.md:204–208`, `design/02-memory.md:172–177`,
`design/02-memory.md:237–239`, `design/reference.md:235`; against
`design/01-design.md:297–300` and `design/reference.md:399–407`.

**The gap.** These texts exclude lifetime parameters on traits "because they
lie beyond D4's power ceiling":

- D2;
- 02 §2.3 and §2.6;
- reference §3.2, which cites its §5.1.

But D4's list and reference §5.1's table name only GATs and
rank-N/`for<'a>`, and neither shows FX0201.

**Fix.**

- Add "lifetime parameters on traits" to D4's "No ..." bullet.
- Add a row to reference §5.1's table: `trait Parse<'a>`, `FX0201`, "a
  lifetime on the method or impl".

Trait type parameters such as `Parse<&'a str>` stay allowed, so the
exclusion is a simplicity choice, and D4 is where to record it.

### N4. 02 §2.3's grammar places `use<..>` where Rust rejects it

`design/02-memory.md:151–160`.

The grammar has `Bound ::= TraitBound | Lifetime | 'use' '<' .. '>'`, valid
"in `T: ..`, `impl ..`, `dyn ..`, where clauses". The text says these forms
mean what they mean in Rust 2024, but rustc 1.96 disagrees in three ways:

- It rejects `use<..>` in `dyn` bounds, in type-parameter bounds, and in
  argument-position `impl Trait`.
- It requires `use<..>` to list every type parameter in scope.
- The grammar has no production for the where-clause form `where 'a: 'b`.

The note "never on a trait" also sits on a production that includes type
parameters, which traits may have.

**Fix.**

- Give `use<..>` its own production, valid only on a return-position
  `impl Trait` and "listing every type parameter in scope".
- Add `WherePred ::= Lifetime ':' Lifetime ('+' Lifetime)*`.
- Attach "never on a trait" to the lifetime parameters only.

### N5. Small slips

- **`T: 'static`** still sits in the rejected-forms bullet (round 8's N8):
  `design/01-design.md:207–208`, `design/02-memory.md:176–177`,
  `design/reference.md:237–238`.
- **REFERENCES.md.** `REFERENCES.md:18` lists "Written `'a: 'b` bounds"
  among the Rust features Tide rejects.
- **Draft number.** D2's accepted set changed, but 01 still says draft 3.4,
  and D2's own history says "Draft 3.4 first kept a narrower subset"
  (`design/01-design.md:245–246`). A citation of "draft 3.4" (README,
  REFERENCES, the trials, round 8) no longer names one set of lifetime
  rules. Consider 3.5.
- **Children of a scope passed down** (predates this commit). Reference
  §3.4 (`design/reference.md:286–287`) says they require `Owned`. Reference
  §7.1 (`design/reference.md:896–899`) and 02 §8.2
  (`design/02-memory.md:662–668`) let them borrow what the signature ties to
  the scope (`cfg: &'s Config`), and the mock accepts that.

## Ready to implement v0

**Yes, after B2.** B2 is a one-sentence fix to text that the v0 checker
implements.

- **B1** concerns `final`, which is v1. It must be fixed before `prove`
  mode, not before v0.
- **N1 and N2** decide two v0 acceptance questions: an `impl Trait + Owned`
  result, and a `let`-annotated `Box<dyn ..>` that borrows. In both, 02 is
  stricter than D2's "as in Rust 2024". Settle them before the checker's
  test corpus is written.

This review did not assess the rest of the v0 tier.

Validation: this was a read-only analysis of `295731e` (diffed against
`af8a99c`) on 2026-10-19. As supporting evidence, Rust transcriptions were
checked with rustc 1.96.0 in a scratch directory outside the repository:

- **Lifetime examples, editions 2024 and 2021.**
  - Compiled: 02 §2.4's `pick` with round 8's caller, plus a no-bounds
    control; 02 §5.2's `evens` and `len_of ... + use<'a>`, with a caller
    that drops `key` first; reference §3.5's `evens` and `over`; 02 §10.1's
    `adults`; round 8's `Lexer::lens` (both forms).
  - Negative cases: `len_of` without `use<'a>`; `len_of` capturing `key`;
    `evens` and `over` escaping a local.
  - Results: under 2024 every stated verdict matches. Under 2021, `evens`,
    `over`, and `lens` fail with E0700, as Rust 2024's capture rule
    predicts.
- **02 §10.1–§10.8 and §10.11, edition 2024.** Every accepted example
  compiles, and every stated rejection is rejected (E0373, E0499, E0515,
  E0726, E0621, E0505, or "lifetime may not live long enough").
- **A mock of 02 §8.2's `Tasks::scope`, `Scope`, `Task`, and `spawn`,
  editions 2024 and 2021.** It has invariant `'s` and `'env` and only the
  declared `'env: 's`, with stubbed bodies.
  - These compile: `parallel_sum`, `after_failure`, `serve_owned`, a scope
    passed down with `cfg: &'s Config`, a grandchild, `join` in the body, a
    child result that borrows outer data (`U: 's`), reference §7.1's
    `fetch_both`, and `Task<'_, Result<Vec<u8>, NetError>, E>`.
  - These are rejected: `serve` (E0373 on `buf` and `conn`), a `Scope` or
    `Task` returned through `T`, a `Scope` stored outside, and a parent
    mutating a lent place (E0502).
  - The control without the declared bound fails as described in the
    Verdict.
- **B1, edition 2024.** The scenarios were compiled and run. N1, N2, and N4
  were each checked with small programs.

No Tide checker exists, so every Tide verdict above is an inference from
the text and its claim to match rustc. Only
`reviews/round9-consistency.md` was written; nothing was staged or
committed.
