# Tide consistency review, round 8: Rust lifetime spelling

## Verdict

**request_changes**

Reviewed commit `af8a99ce158b37be658e2728c5c940681b5d9e1a` (`af8a99c`,
"Adopt Rust lifetime syntax for origins (draft 3.4)") on 2026-10-19,
diffed against its parent `f1d7a5d`.

What holds:

- **No leftover `from` syntax.** Normative text has no `from`-clause syntax
  or place-named origins left. The remaining `from` spellings are all
  historical: D2's Rejected paragraph, the second paragraph of 02 §2.6,
  02 §13 items 2–3, and 00 H1. 02, 03, 04, the reference, and the README
  all say draft 3.4.
- **The documents agree, with one exception.** D1, D2, 02 §2–§5,
  reference §0–§3 and §14, README, REFERENCES, and 00 H1 give the same
  accepted and rejected forms, elision rules, `dyn` defaults, `impl Trait`
  capture rule, `Owned`/`T: 'static` rewrite, and codes FX0201–FX0204 and
  FX0210. 02 §8.2, 03 §6.1, and reference §7.1 agree on
  `scope<'env, ..>(&'env self, ..)`, `Scope<'s, 'env, E>`, and
  `Task<'s, U, E>`. The exception is D2 and 03 §1, which still show the
  old arity (B3).
- **Cross-references resolve.** The section numbers of 02 did not change.
  Every cross-reference checked (into §2.1–§2.7, §3, §5, §8, §10, §11, §12)
  lands on the intended text.
- **The `'env` encoding is sound.** It gives the higher-ranked `'s` its lower
  bound without a written bound or an explicit `for<>`. The test was a rustc
  mock of 02 §8.2's signatures whose `Scope` links its two lifetimes only
  through a field of type `PhantomData<&'s &'env Tasks>`.
  - These compile: `parallel_sum`, `after_failure`, a grandchild spawn, a
    `join` in the body, `serve<'s>(.., cfg: &'s Config)`, and reference
    §7.1's `fetch_both`.
  - These are rejected: returning a `Scope` or `Task` through `T`, and
    storing a `Scope` in an outer `Option<Scope<..>>`.
  - Replacing the field with `(&'s (), &'env Tasks)` makes every borrowing
    `scope` body fail (12 errors: E0597, E0503, E0521).

  So the field-implied `'env: 's` is necessary and sufficient, as std's
  explicit `'env: 'scope` is.
- **The examples match rustc, except `serve`.** Every verdict in 02
  §10.1–§10.8 and §10.11, reference §3.1–§3.5, and The Feel matches rustc
  1.96 under editions 2018 and 2021, including all ten stated rejections.
  The exception is 02 §10.12's `serve` (B4).
- **04 keeps its old meaning.** 04 §4.1's restated executable-`final`
  condition means what the old one did: sharing a lifetime name takes the
  place of a `from` identification. N7 notes a gap that predates this
  commit.

B1 and B2 come from decisions this commit made beyond the trial's arm B, and
each needs the owner to choose. B3 and B4 are text fixes.

## Blocking problems

### B1. "Type-outlives bounds in every spelling" can spell the rejected `'a: 'b`

**Location:** `design/01-design.md:19–21`, `design/01-design.md:202–210`,
`design/02-memory.md:119–123`, `design/02-memory.md:165–172`,
`design/02-memory.md:190–192`, `design/reference.md:229–240`.

**Problem.** D2 accepts "type-outlives bounds in every spelling". 02 §2.3
and §2.4 item 3 define `X: 'a` as `σ ⊆ ρa` for every `σ` in `orig(X)`, and
nothing restricts `X`. When `X` carries a lifetime of its own, the bound is a
lifetime-outlives bound: `orig(&'a ()) = {ρa}`, so `&'a (): 'r` means
`ρa ⊆ ρr`, which is exactly `'a: 'r`. That contradicts four other
statements:

- FX0201's rejection of written `'a: 'b`;
- 02 §2.1: a body "may rely on `σ ⊆ τ` between two signature origins only
  when the well-formedness of the signature's types implies it";
- the thesis: "a subset without written outlives relations";
- D2's accepted cost for invariant positions.

Arm B's reference excluded `T: 'a` (arm-B reference line 227; grading §2.2
and RB2). Widening the subset to every spelling is what opened this.

**Scenario.** This is the signature D2's accepted cost calls inexpressible:

```tide
fn pick<'r, 'a, 'b>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str
    where &'a (): 'r, &'b (): 'r
{ v.first().copied().unwrap_or(s) }
```

rustc 1.96 (edition 2021) accepts it, and also accepts a caller that drops
`s`'s owner and then uses `v` again. It also accepts
`struct Two<'a, 'b> where &'a (): 'b { .. }`. The two parts of 02 disagree
on these programs: a checker that follows 02 §2.4 item 3 accepts them, and
one that follows §2.1 rejects them.

**Proposed fix.** Decide in D2 first. Two options:

- **(a) Recommended: restrict the subject.** Allow type-outlives bounds only
  on a type parameter or an associated-type projection of one (`T: 'a`,
  `T: Trait + 'a`, `I::Item: 'a`), plus `impl Trait + 'a` and
  `dyn Trait + 'a`. A bound whose subject names a lifetime of its own
  (`&'a (): 'b`, `Lexer<'a>: 'b`) is FX0201, with the same rewrites.
  - Say this in D2's Accepted bullet, 02 §2.3 and §2.4 item 3, and reference
    §3.2.
  - Change the thesis to "without written lifetime-outlives bounds".
  - The implied-relation restructure in N2 stays the sanctioned route.
- **(b) Accept `'a: 'b` outright.** 02's notation (line 17) already defines
  it as `ρa ⊆ ρb`. Then delete the accepted cost and FX0201's first entry.

Option (a) keeps the decision as recorded, and the trial gives no evidence
about `'a: 'b` either way. Option (b) is the cheaper reversal if the owner
reconsiders.

### B2. Rust 2018 capture without `use<..>` leaves a common `&mut self` case with no Tide spelling

**Location:** `design/01-design.md:207–209`, `design/01-design.md:227–229`,
`design/02-memory.md:167–171`, `design/02-memory.md:226–231`,
`design/02-memory.md:438–444`, `design/02-memory.md:1196`,
`design/reference.md:302–308`, `design/reference.md:1395–1397`.

**Problem.** A returned `impl Trait` captures only the lifetimes its bounds
name (Rust 2018's rule). Rust has two ways around that rule, and Tide rejects
both under FX0201: `+ use<..>`, and the `Captures<'a>` idiom, which needs a
trait lifetime parameter.

When the hidden type holds a lifetime at an invariant position that the
bounds do not name, rustc 2018 and 2021 report E0700. A checker that
matches rustc (02 §2.8, §11.1's `use<'p…>` row) must therefore reject.
Adding `+ 'a` does not help, because it demands that the whole hidden type
outlive `'a`.

02 §2.6 justifies rejecting precise-capture lists as forms "whose meaning is
not an origin of a parameter, a field, or a result". But `use<'a>` says
exactly that the result borrows `'a`.

**Scenario.**

```tide
impl<'a> Lexer<'a> {
    pub fn lens(&mut self) -> impl Iterator<Item = usize> + '_ {
        std::iter::from_fn(move || self.next_word().map(str::len))
    }
}
```

- **As written:** rustc 1.96 under editions 2018 and 2021 reports E0700
  (the hidden type captures `'a`).
- **With `+ '_ + 'a`** (adding `+ 'a`): "lifetime may not live long
  enough".
- **With `+ use<'_, 'a>`:** rustc compiles it under both editions, but it is
  FX0201 in Tide.
- **What compiles in Tide:** only `Box<dyn Iterator<Item = usize> + '_>`,
  or an undocumented trick: a type-parameter trait
  `trait Captures<T: ?Sized> {}` with a blanket impl and the bound
  `+ Captures<&'a ()>`, which rustc accepts.

The reference has no row or diagnostic for this case. Its FX0201 rewrites
(one shared lifetime, `+ 'a`, restructure) and FX0204's "safe" fix
(`+ '_`, already present here) all fail.

The trial never hit this case, because arm B's iterators named `'a` in
`Item`. But a `&mut self` method yielding owned items from a borrowing type
is ordinary lexer and cursor code.

**Proposed fix.** Two options:

- **Recommended: accept `+ use<..>` with Rust's meaning.** It replaces the
  default capture set. Make it FX0204's fix when a hidden type needs a
  lifetime its bounds do not name. Update D2's Rejected bullet, 02 §2.3,
  §2.6, §5.2 and §11.1, and reference §3.2, §3.5 and §14.
- **Alternative: keep the rejection.** Add this case to D2's accepted costs
  and to 02 §12 item 6, and make FX0204 suggest `Box<dyn Trait + '_>` when
  `+ 'a` cannot hold.

The first is current Rust spelling with the same meaning (D1). The second
leaves agents with an allocation or an obscure idiom.

### B3. D2 still gives `Scope` the old arity, which has no `'env` lower bound

**Location:** `design/01-design.md:222–224`; also
`design/03-authority-concurrency.md:36`,
`design/03-authority-concurrency.md:113–114`, `design/reference.md:877`.

**Problem.** D2 says `Tasks::scope` is typed
`impl FnOnce(Scope<'_, E>) -> Result<T, E>`. 02 §8.2, 03 §6.1, and reference
§7.1 type it with `Scope<'s, 'env, E>` and `FnOnce(Scope<'_, 'env, E>)`.
The decision record owns D2, so as written the chapters contradict it.

D2's form is also not enough on its own. With no `'env`, the higher-ranked
`'s` has no lower bound (02 §5.4), so a child could borrow only `'static`
data. 03 §1.1 and the §1.2 table still list `Scope<E>` and `Task<U, E>`,
and reference §7.1 writes `Task<Result<Vec<u8>, NetError>, E>`.

**Scenario.** An implementer types `scope` from D2 as `FnOnce(Scope<'_, E>)`.
In the rustc mock, removing the field that implies `'env: 's` has the same
effect: 02 §10.12's `parallel_sum`, which 02 says is accepted, then fails
(E0597 on `sums` and `mid`, E0503, E0521), and so does every other
borrowing scope body.

**Proposed fix.** In D2, write
`` (`impl FnOnce(Scope<'_, 'env, E>) -> Result<T, E>`, where `Scope`'s fields imply `'env: 's`) ``.
In 03 §1.1 and the §1.2 table, write `Scope<'s, 'env, E>` and
`Task<'s, U, E>`. In reference §7.1, write
`Task<'_, Result<Vec<u8>, NetError>, E>`.

### B4. 02 §10.12's `serve` verdicts do not match rustc's

**Location:** `design/02-memory.md:1142–1146`, `design/02-memory.md:1167–1169`;
claim at `design/02-memory.md:814–816`.

**Problem.** 02 §10 now says "Each verdict matches rustc's". In `serve`, the
first spawn borrows `conn` as well as `buf`. That loan is in the scope's
placeholder origin, which is live throughout `serve` (§8.2). The second
spawn's `move` of `conn` therefore conflicts with a live loan ([Loan], §1.4).
By Tide's own rules the second spawn is rejected, but the text says it is
accepted.

`read_into` also cannot accept both calls unless it takes `&[u8]`:

- against `&mut [u8]`, the first call's `&buf` is a type error;
- with a `&mut self` receiver, both calls need `mut conn`.

**Scenario.** Transcribed with `read_into(&self, &[u8])`, rustc reports
three errors:

- E0373 for `buf`;
- E0373 for `conn`;
- E0505 "cannot move out of `conn` because it is borrowed", at the second
  spawn.

With the first spawn removed, the second compiles. A checker test built from
§10.12 expects one error and gets three.

**Proposed fix.** Split it into two functions (checked with rustc: the first
gives the two E0373 errors, the second compiles):

```tide
fn serve(bg: Scope<'_, '_, ServeError>, conn: Conn) {   // read_into(&self, &mut [u8])
    let mut buf = [0u8; 4096];
    bg.spawn(|| conn.read_into(&mut buf));             // error: borrows locals `conn` and `buf`
}
fn serve_owned(bg: Scope<'_, '_, ServeError>, conn: Conn) {
    bg.spawn(move || { let mut buf = [0u8; 4096]; conn.read_into(&mut buf) });   // fine
}
```

Then reword the verdict: the first is rejected because the child borrows
`conn` and `buf`, neither of which outlives `bg`'s scope.

## Non-blocking notes

### N1. 02 §2.6 overstates "the identity on lifetimes"

`design/02-memory.md:226–227`, `design/02-memory.md:190–192`,
`design/02-memory.md:429–430`, `design/02-memory.md:1192–1202`,
`design/01-design.md:654–655`.

02 §2.6 says every accepted signature "is a Rust signature with the same
meaning", so the translation is the identity on lifetimes. D12 says "nearly
the identity". In four places the same spelling means something else, or
the translation must add text:

- **(a) The `Fn` output clause.** §2.4 item 3 makes `F: Fn..() -> R + 'a`
  and `dyn Fn() -> R + 'a` also require `R: 'a`, which Rust does not.
  rustc accepts both of these, and Tide rejects both under item 3:
  - `fn make<'a>(_: &'a str) -> Box<dyn Fn() -> &'a str> { Box::new(|| "lit") }`;
  - a `'static`-bounded `FnOnce() -> &'a str`.

  Under `from`, this rule had no Rust spelling to collide with. Now it does.
- **(b) The container object default.** §5.1 keeps `Owned` where Rust takes
  the object default from a container's `T: 'a` bound. This commit lets user
  structs write `T: 'a`, so the difference now reaches user code. Given
  `struct W<'a, T: ?Sized + 'a> { r: &'a T }`, rustc accepts
  `fn f<'a>(x: &'a (dyn Debug + 'a)) -> W<'a, dyn Debug> { W { r: x } }`.
  With Tide's default it reads `+ 'static` and is rejected.
- **(c) `use<..>`.** §11.1's own row adds `+ use<..>` when the target is
  Rust 2024.
- **(d) Mapped types.** `Owned` becomes `'static`, and `Scope` and `Task`
  become std types.

**Fix.**

- Drop item 3's output clause, so that `F: 'a` means what it means in Rust,
  and write `U: 's` and `E: 's` on `Scope::spawn`, as
  `std::thread::Scope::spawn` writes `T: 'scope`.
- List (b) in §11.2. It is stricter than Rust, so it is still sound.
- In both §2.6 and D12, say "the identity on lifetimes except the rows
  §11.1 marks".

### N2. The invariant-position cost is stated too strongly

`design/01-design.md:260–262`, `design/02-memory.md:186–189`,
`design/02-memory.md:1249–1252`, `design/reference.md:237–239`.

Checked with rustc:

- **One shared name:** `fn pick<'a>(v: &mut Vec<&'a str>, s: &'a str) -> &'a str`
  is itself accepted. A caller then fails, because `s`'s loan enters `v`'s
  element lifetime: E0597, "`short` does not live long enough", when `v` is
  used after `short` dies.
- **Implied relation, no written bound:**
  `fn pick<'r, 'a>(v: &'r mut Vec<&'a str>, s: &'r str) -> &'r str` is
  accepted, and so is the same caller. The cost is that `v` stays mutably
  borrowed while the result lives.

So the result can get a lifetime of its own. The reference's "cannot get a
lifetime of its own" and 02 §12's "cannot express" are too strong.

**Fix.** State the cost as: "one name pulls the other inputs into the
invariant lifetime; the alternative ties the result to the `&mut` borrow".
Show both forms. If B1 is resolved by accepting `'a: 'b`, delete the cost
instead.

### N3. "With none it is `Owned`" ignores captured type parameters

`design/01-design.md:227–229`, `design/02-memory.md:438–444`,
`design/reference.md:302–303`.

`fn g<T: Clone>(x: T) -> impl Fn() -> T` names no lifetime, yet `g(s)` with
`s: &str` is not `'static`: rustc reports E0521 under editions 2018 and 2024.
02's own "Type parameters are captured as in Rust" implies this.

**Fix.** Write "with no lifetimes and no captured type parameter that
carries origins, it is `Owned`".

### N4. Where implied relations come from: arguments versus the whole signature

`design/01-design.md:205`, `design/reference.md:232` against
`design/02-memory.md:121–122`, `design/02-memory.md:195–196`,
`design/02-memory.md:251`.

D2 and the reference derive implied relations from "well-formed argument
types". 02 derives them from parameter and return types, as rustc does.
rustc accepts `fn h<'a, 'b>(x: &'b u8) -> (Option<&'a &'b ()>, &'a u8) { (None, x) }`
on the strength of the return type alone.

**Fix.** Say "signature types" in D2 and the reference.

### N5. The `Scope` encoding depends on rules 02 leaves implicit

`design/02-memory.md:240–245`, `design/02-memory.md:477–481`,
`design/02-memory.md:622–633`, `design/02-memory.md:1216–1217`.

The mechanism works (see the Verdict), but three things it rests on are
unstated:

- **(a) Struct well-formedness.** 02 never says that the relations a
  struct's fields imply are part of its well-formedness wherever the type
  is used. §3.4 and §2.4 item 5 together suggest it, and the scope lower
  bound depends on it.
- **(b) The interface record.** §2.7's record for a public type lists
  variances, auto traits, and a drop-check summary, but not the outlives
  relations its fields imply. A private-field edit could therefore add or
  remove an implied bound in dependents without counting as an API change.
- **(c) std's field.** std's Rust `Scope` declares `'env: 'scope`
  explicitly, which FX0201 forbids. Tide's std must therefore use a field
  such as `PhantomData<&'s &'env ()>`. §11.2 R2 should say that, not cite
  "the encoding that `std::thread::scope` relies on".

The variance of `'env` is also unstated.

**Fix.** Add the struct-WF sentence to §2.4 item 5. Add "implied outlives
relations" to §2.7's per-type record. Name the field and the variance of
`'env` in §8.2.

### N6. Impl-header elision is undefined, and the bare form is a Rust error

`design/02-memory.md:160–164`, `design/02-memory.md:200–222`,
`design/02-memory.md:1017`, `design/03-authority-concurrency.md:600`,
`design/reference.md:853`.

§10.8 writes `impl Collector<'_, '_>`, and 03 and the reference write
`impl<U, E> Task<'_, U, E>`. But §2.5 defines elision only for function
signatures, fields, and associated types.

§2.3 says a borrowing type written without lifetime arguments "elides them,
as in Rust". In impl headers that is false: for `struct Lexer<'a>`, rustc
rejects `impl Lexer { .. }` with E0726 under editions 2018 and 2021. That
was draft 3's habitual form, so agents and older examples will produce it.

**Fix.** Add to §2.5: "In an impl header each `'_` is a fresh lifetime
parameter of the impl; the bare name is an error (E0726) whose fix adds
`<'_>`." Limit §2.3's bare-name sentence to signatures.

### N7. 04 §4.1's executable-`final` test misses stores through interior mutability (predates this commit; v1)

`design/04-contracts.md:670–674`, `design/04-contracts.md:688–691`.

The old and new texts both look only for the lifetime "under another `&mut`
parameter". This function passes both tests:

```tide
fn stash<'r>(s: &RefCell<Option<&'r mut i32>>, x: &'r mut i32) {
    *x = 5;
    *s.borrow_mut() = Some(x);
}
```

So `final(*x)` would be executable, with the value 5. rustc accepts
`stash`, and a caller can then write 7 through the cell before the borrow
ends. A by-value `Sender<&'r mut i32>` behaves the same way. `final` is in
the v1 row of the Tiers table.

**Fix.** Make the second test "occurs in the type of any other parameter".

### N8. Small wording slips

- **Where higher-ranked lifetimes come from.** 02 §2.3 (`design/02-memory.md:170`)
  and D4 (`design/01-design.md:300–301`) say only from `Fn` sugar. 02 §5.4
  (`design/02-memory.md:470–471`) also makes elided lifetimes in `fn`
  pointer types higher-ranked; name both.
- **`Shared::with`.** 02 §8.4 (`design/02-memory.md:696`) writes its bound
  as `for<'σ> FnOnce(&'σ mut T1, …) -> R`, a form FX0201 rejects in user
  code. The Tide spelling is `FnOnce(&mut T1, …) -> R`.
- **`T: 'static` in D2.** D2 (`design/01-design.md:209–210`) puts "`T:
  'static` is accepted and rewritten" inside the Rejected bullet; move it
  to Accepted.
- **"Rejected in v1".** 02 §10.2 (`design/02-memory.md:874`) says
  "Rejected in v1", but v0 rejects the example too (§2.8). Only the later
  Polonius tier could accept it.
- **Untested forms.** No trial program exercised the type-outlives family or
  `Scope<'s, 'env, E>`. D2 cites the trial as evidence for the subset, so
  E1 should cover them.

## Ready to implement v0

**No.** B1 and B2 leave open which lifetime forms the v0 checker accepts:
whether `&'a (): 'b` is legal, and whether `+ use<..>` exists or what FX0204
suggests. Those choices feed the parser, the interface artifact, and the
FX0201 and FX0204 diagnostics. B3 and B4 are text fixes.

Once B1–B4 are resolved, nothing in this review's scope blocks v0. This
review did not assess the rest of the v0 tier.

Validation: read-only document analysis of `af8a99c` (diffed against
`f1d7a5d`) on 2026-10-19. As supporting evidence, Rust transcriptions were
checked with rustc 1.96.0 in a scratch directory outside the repository:
the worked examples, and a mock of 02 §8.2's `Scope`/`Task` signatures with
stubbed bodies. Editions 2018 and 2021 were used throughout, and 2024 where
stated. No Tide checker exists, so every Tide verdict above is an inference
from the text and from its claim to match rustc. Only
`reviews/round8-consistency.md` was written; nothing was staged or
committed.
