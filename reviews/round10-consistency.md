# Tide consistency review, round 10: confirmation

## Verdict

**request_changes**

Reviewed commit `cd370405338cad04897bc22ffd29e211930f66db` (`cd37040`,
"Answer round 9: declared bounds in bodies, final escapes"), dated
2026-10-19 14:34 -0300, diffed against `c846da0`. The review covers only the
round-9 items and the passages the commit changed in `design/01-design.md`,
`design/02-memory.md`, `design/04-contracts.md`, `design/reference.md`, and
`REFERENCES.md`.

What holds:

- **Round 9's B1 is closed for soundness.** Each of round 9's escaping
  programs fails the new executable-`final` test in 04 §4.1. That covers
  `give`, `stash` with `'r: 'q`, `setter`, `bump`, and the older
  `stash_generic`. An ordinary `fn inc(x: &mut i32)` passes it. rustc 1.96
  (edition 2024) confirms both halves: each escape runs and prints 7, and it
  stops compiling once the bound or capture that the new text names is
  removed.
- **Round 9's B2 is closed.** 02 §2.7 now lets a body rely on declared
  outlives bounds.
- **N2 and N3 are resolved, and most of N5 is.**

Two new problems come from the new text. Both trace to wording that round 9
itself proposed, and this commit adopted it as written.

- **B1 (v1).** The clause "no declared or implied outlives bound relates it
  to another origin" catches the `'a: 'r` that every borrowing pointee
  implies. So `final(*self)` becomes a prophecy for every `&mut self` method
  of a borrowing type. That contradicts the paragraph's own summary.
- **B2 (v0).** The new `orig` row for a returned `impl Trait` ignores the
  opaque type's outlives bound. It therefore contradicts §5.2's new rule that
  `impl Trait + Owned` is `Owned`, and rejects programs that rustc accepts.

## Round-9 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** for soundness; its new clause is too broad (new B1) | The test is at `design/04-contracts.md:670–679`. `give` is excluded by the bound `'r: 'q` (`:675–676`) and is named at `:678–679`. `stash` with `'r: 'q`, `bump` with `'r: 's`, and `stash_generic` with `S: Sink<&'r mut i32>` are excluded because `'r` "occurs ... in no bound or where clause" (`:673`). `setter` is excluded by "counting the lifetimes a returned `impl Trait` captures" (`:674–675`). `inc` passes: `x`'s elided origin occurs nowhere else, its result `()` has no origins, and its only implied relation is `i32: 'x`, where `orig(i32) = ∅` (`02-memory.md:137`, `:140–142`). The `orig` row is at `02-memory.md:135`, and the summary sentence is reworded at `04-contracts.md:693–697`. rustc results are in the validation note. |
| B2 | **Resolved** | `design/02-memory.md:263–268` now admits "the signature's declared outlives bounds" and suggests `'a: 'b` as a fix. This agrees with `02-memory.md:121–123`, with `pick` at `:196`, and with `reference.md:230–232`. No copy of the old sentence remains in `design/`. |
| N1 | **Partial** | `design/02-memory.md:456–458` makes an opaque type with `+ Owned` or `+ 'static` `Owned`, which rustc confirms. The auto-trait bullets were not qualified (`02-memory.md:463–465`, `:770`), so the case of a result that captures nothing is now a direct contradiction (new N2). The new `orig` row disagrees with the `+ Owned` case (new B2). |
| N2 | **Resolved** (recommended option) | `design/02-memory.md:438–442` and `design/reference.md:289–293`: inside a body, an elided object lifetime is inferred. This matches D2 (`01-design.md:192–195`), §11.1 (`02-memory.md:1217`), and rustc. Several summaries still say `Box<dyn Trait>` is `Owned` without qualification (new N1). |
| N3 | **Resolved** | D4 lists "lifetime parameters on traits" (`design/01-design.md:297–298`), and reference §5.1 has a row for them (`design/reference.md:405`). The code `FX0201` agrees with `01-design.md:205`, `02-memory.md:175`, `reference.md:237`, and `:1395`. |
| N4 | **Partial** | `Capture` now has its own production, limited to a return-position `impl Trait` (`design/02-memory.md:161`), and `WherePred` is added (`:160`). Not done: "never on a trait" still sits on `Generics` (`:154–155`), which includes type parameters, and `Capture` does not say that it lists every type parameter in scope (new N3). |
| N5 | **Mostly resolved** | `T: 'static` is now among the accepted forms (`01-design.md:204`, `02-memory.md:172–173`, `reference.md:235`). `REFERENCES.md:18` no longer rejects `'a: 'b`, but now lists GATs twice. The scope-passed-down bullet (`reference.md:286–288`) agrees with `reference.md:899–902`, `02-memory.md:669–672`, and `03-authority-concurrency.md:607–609`. The draft number was not changed (new N4). |

## New blocking problems

### B1. 04 §4.1's implied-bound clause makes `final` prophetic for every `&mut` to a borrowing type

**Location:** `design/04-contracts.md:675–676`, against
`design/04-contracts.md:693–697`, `design/02-memory.md:130`, and
`design/02-memory.md:140–142`. Tier: v1.

**Problem.** The new test makes `final(*x)` executable only if "no declared
or implied outlives bound relates it to another origin". By 02 §2.2, the
well-formedness of `&ρ T` implies `σ ⊆ ρ` for every `σ` in `orig(T)`. So
`x: &'r mut Lexer<'a>` always implies `'a: 'r`, which relates `x`'s origin
to `'a`.

- **The cases it catches.** The clause catches every `&mut` whose pointee
  holds a borrow:
  - `next_token(&mut self) -> Option<Token<'a>>` in `impl<'a> Lexer<'a>`
    (`reference.md:256–259`);
  - `fn skip_ws(&mut self)` on the same type;
  - `v: &mut Vec<&str>`.
- **What follows.** In each of these, `final(*x)` is a prophecy. Prophetic
  `final` is `proof-only` (`04-contracts.md:686–687`, `:433–434`), so
  `assert` and `test` stop checking those postconditions. `tide fix` also
  stops rewriting v0 clauses there (`04-contracts.md:166–168`).
- **Type parameters.** If `orig(T)` counts as "another origin", the same
  holds for `&mut Vec<T>`. That would include `Ring<T>::push` and `pop`,
  whose `final(*self)` postconditions (`04-contracts.md:1189–1215`) run under
  the default `assert` use.

**Contradiction.** The paragraph's summary (`04-contracts.md:693–697`)
lists where prophecies arise:

- results that carry or capture a `&mut` parameter's origin;
- a `&mut` parameter's origin that the signature relates to another
  parameter "(one name, a bound, or `'r: 'q`)";
- borrowing types with `&mut` fields.

It ends with "ordinary `&mut` arguments need none". `skip_ws` and
`next_token` fit none of these:

- `orig(&'r mut Lexer<'a>)` is `{ρr}` (`02-memory.md:130`), so a result of
  `Token<'a>` does not carry `self`'s origin.
- `Lexer` has no `&mut` field.

**Why the clause is not needed.** `'a: 'r` is the direction in which `x`'s
loans cannot escape. rustc 1.96 accepts `skip_ws`. It rejects a `Lexer<'a>`
that tries to push `&mut self.pos` into a log reachable through `'a`
("lifetime may not live long enough").

The implied relation that can move a borrow is the other direction,
`'r: 'q`. It arises only where `'r` appears in some other signature type.
Another parameter's type and the result are already covered. That leaves
`x`'s own pointee: `x: &'r mut Pair<'r, 'q>` with a field
`&'q RefCell<Option<&'r mut i32>>` implies `'r: 'q`. rustc accepts a body
that stores `&mut x.val` in that cell. It then rejects any later read of the
place (E0502), because `&'r mut Pair<'r, ..>` locks the place for as long as
it is used.

**Proposed fix.**

- In `04-contracts.md:675–676`, replace "and no declared or implied
  outlives bound relates it to another origin" with: "and no declared or
  implied bound makes it outlive another origin (`'r: 'q`, that is
  `ρr ⊆ ρq`). The `'a: 'r` that a borrowing pointee implies does not count."
- In `:694–696`, write: "a `&mut` parameter's origin that the signature lets
  outlive another origin (a shared name, a bound, or `'r: 'q`, declared or
  implied)".

`give`, `stash`, `bump`, `setter`, and `stash_generic` stay excluded, and
`inc`, `skip_ws`, `next_token`, and `Ring::push` become executable again.
Dropping "or implied" altogether would also be sound by the `Pair` result
above, but the direction-limited form is the safer change.

### B2. 02 §2.2's `orig` row for a returned `impl Trait` ignores its outlives bound

**Location:** `design/02-memory.md:135`, against
`design/02-memory.md:456–458`, `design/02-memory.md:353–354`,
`design/02-memory.md:197–199`, and `design/01-design.md:192–195`. Tier: v0.
This is the origin calculus, which D2 makes "the definition the checker
implements" (`01-design.md:234–236`).

**Problem.** The new row reads `orig(returned impl Trait) = its captured
lifetimes ∪ orig of its captured type parameters`. It ignores a stated
`+ Owned` (`+ 'static`) or `+ 'a`.

- **`+ Owned`.** Take `fn make(x: &str) -> impl Fn() -> usize + Owned`,
  which is round 9's N1 example. Under Rust 2024 capture its result captures
  `x`'s lifetime, so the row gives `{ρx}`. Then 02 §3.7 says `Owned` holds
  only when `orig` is empty (`02-memory.md:353–354`), so the result is not
  `Owned` and holds the caller's loan. The commit's new sentence in §5.2
  (`02-memory.md:456–458`) says the same result is `Owned`.
- **`+ 'a`.** Take
  `fn two<'a, 'b>(x: &'a str, y: &'b str) -> impl Fn() -> usize + 'a`. The
  row gives `{ρa, ρb}`. Read with §2.4 item 3 (`02-memory.md:197–199`), that
  demands `ρb ⊆ ρa`, which the signature never gives, and it keeps `y`'s
  loan alive in the caller.

**What rustc does.** rustc 1.96 (edition 2024) uses the opaque type's
outlives bound to decide which captured loans its value holds. It accepts:

```rust
let f = { let s = String::from("abc"); make(&s) };          // f: 'static
let g = { let tmp = String::from("zz"); two(&keep, &tmp) };  // holds only keep's loan
```

It also accepts `f` passed to a `T: 'static` parameter. Without the bound,
the first line is E0597. A checker that follows the new row rejects both
lines. That contradicts §5.2 and D2's "mean what they mean in Rust 2024".

**Proposed fix.** Replace the row with:

> `orig(returned impl Trait)` = `∅` if its bounds state `Owned`; `{ρ}` if
> they state `+ ρ`; otherwise its captured lifetimes ∪ `orig` of its
> captured type parameters (§5.2).

04 §4.1 can stay as written. It counts "the lifetimes a returned `impl
Trait` captures" explicitly, so `final` remains a prophecy whenever `x`'s
origin is captured, even under a bound. That is conservative, and rustc
rejects such a hidden type anyway (`setter` with `+ 'static` fails to
compile).

## Non-blocking notes

### N1. Summaries still say `Box<dyn Trait>` is `Owned` without the new body exception

The new rule infers an elided object lifetime inside a body
(`design/02-memory.md:441–442`, `design/reference.md:292–293`). rustc agrees
for `let` annotations, turbofish type arguments, and `as` casts. But these
passages still state the default without qualification:

- D2: "`Box<dyn Trait>` is `Owned`" (`design/01-design.md:216–218`);
- 02's own lead sentence for §5.1 (`02-memory.md:430–431`);
- "`Owned` is required by `Box<dyn Trait>` contents" (`02-memory.md:362`);
- "Where `Owned` is required (`Box<dyn Fn()>`, ..)" (`02-memory.md:483`);
- reference §3.4's `Owned` bullet (`reference.md:287`);
- reference §3.5's "an `Owned` position (`Box<dyn Fn()>`, ..)"
  (`reference.md:308–310`);
- the habit row "`Box<dyn Fn()>` capturing a borrow ... `FX0203`"
  (`reference.md:1398`).

Read literally, the last two tell an agent that
`let b: Box<dyn Fn() -> usize> = Box::new(|| s.len());` is an error, but 02
§5.1 and rustc accept it.

**Fix.** Add "in a signature or field" to each.

### N2. Round 9's N1 is now a direct contradiction for results that capture nothing

The commit changed §5.2 from "`Owned` only when every captured parameter
is" to "`Owned` when every captured parameter is, or ..."
(`design/02-memory.md:456–458`).

- **Under §5.2.** `fn count() -> impl Iterator<Item = u32>` captures nothing,
  so it is now `Owned`. So is `fn wrap<T: Fn() + Owned>(t: T) -> impl Fn()`.
  §3.7 gives the same answer from `orig = ∅` (`:353`).
- **Under the auto-trait bullets.** They still say an opaque type has "only"
  or "exactly" the auto traits its bounds state (`:463–465`, `:770`). `Owned`
  is one of those auto traits (`:256`, `:773`), so neither result is `Owned`.
- **What rustc does.** It accepts both results as `'static`.

**Fix.** Round 9's second half: say that those bullets cover `Send`, `Sync`,
`Pure`, and `ThreadBound`, and that `Owned` follows `orig` (§3.7), as Rust's
`'static` does.

### N3. Round 9's N4 leftovers in the 02 §2.3 grammar

- **"never on a trait".** The note still sits on `Generics`
  (`design/02-memory.md:154–155`), whose production includes type
  parameters. Traits take type parameters: `E: From<TaskError>` at `:638`,
  and rustc accepts `trait Sink<T>`. Attach the note to the lifetime
  parameters only.
- **Type parameters in `use<..>`.** `Capture` (`:161`) does not say what
  rustc 1.96 enforces: "`impl Trait` must mention all type parameters in
  scope in `use<...>`".
- **A leading lifetime** (predates this commit). `Generics` requires a
  leading `LtParam`, so `fn f<T>` has no derivation.

### N4. Slips

- **REFERENCES.md.** `REFERENCES.md:18` lists "GATs" twice: "..., explicit
  `for<'a>`, GATs, ..., GATs and specialization".
- **Draft number.** It stays 3.4.
  - The status line now says 3.4 holds Rust 2024's lifetimes after rounds 8
    and 9 (`design/01-design.md:3–6`).
  - The history in D2 and 02 §2.6 still says "Draft 3.4 first kept a narrower
    subset" (`01-design.md:245–246`, `02-memory.md:246–247`).

  So "draft 3.4" still names two rule sets. That is acceptable if the owner
  keeps the number. Otherwise use 3.5.

## Ready to implement v0

**Yes, after B2.** B2 is one row of the v0 origin calculus.

- **B1** concerns `final`, which is v1 (`design/01-design.md:77`). Fix it
  before `prove` mode and before v1 classes clauses, not before v0.
- **N1 and N2** touch v0 acceptance (`Box<dyn ..>` in bodies, and `Owned`
  for opaque results). The normative bullets already decide N1. Settle both
  before the checker's test corpus is written.

This review did not reassess the rest of the v0 tier.

Validation: this was a read-only review of `cd37040` (diffed against
`c846da0`) on 2026-10-19. Rust programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), under edition 2024 unless noted, in a scratch
directory outside the repository:

- **Round 9's B1 escapes.** `give`, `stash` with `'r: 'q` over a
  `RefCell<Option<&'q mut i32>>`, `setter -> impl FnMut(i32)`, `bump`
  spawning into a scope, and `stash_generic` with a `RefCell`-backed `Sink`.
  - `bump` used a mock of 02 §8.2: `Scope<'s, 'env: 's, E>` wrapping
    `&'s std::thread::Scope<'s, 'env>`, with `spawn`
    (`F: .. + Send + 's`, `U: Send + 's`) and `Tasks::scope` built on
    `std::thread::scope`.
  - All five compile and print `7 7 7 7 7`, each after the callee left 5.
- **Controls with the named channel removed.**
  - `give`, `stash`, `bump`, and `stash_generic` without the bound: each
    fails with "lifetime may not live long enough".
  - `setter` with `+ use<>`: E0700.
  - `setter` under edition 2021 with no capture: E0700.
- **`inc`.** `fn inc(x: &mut i32)` compiles and runs. Three attempts to let
  `x` escape from that signature are rejected:
  - a `thread::spawn`: E0521;
  - a thread-local slot: E0521;
  - a `Box<dyn FnMut(i32)>` field: "lifetime may not live long enough".
- **New B1.**
  - `skip_ws(&mut self)` on `Lexer<'a>` compiles.
  - Pushing `&mut self.pos` into a log reachable through `'a` is rejected.
  - `f(x: &'r mut Pair<'r, 'q>)`, whose pointee implies `'r: 'q`, compiles,
    and the write through the cell runs. A later read of the place is E0502.
- **New B2.** `make(..) -> impl Fn() -> usize + 'static` and
  `two(..) -> impl Fn() -> usize + 'a` compile, with callers that drop the
  other owner first; the program prints `3 5`. Without the bound, the first
  is E0597. `setter -> impl FnMut(i32) + 'static` is rejected.
- **Notes.**
  - `count()`, `wrap`, and `make` pass a `T: 'static` parameter.
  - These body forms with a borrowing closure compile: a `let` annotation
    `Box<dyn Fn() -> usize>`, a turbofish
    `Vec::<Box<dyn Fn() -> usize>>::new()`, and an `as` cast. A signature
    `-> Box<dyn Fn() -> usize>` is rejected.
  - `use<>` that omits a type parameter is rejected, and so is `use<>` in a
    `dyn` bound. `trait Parse<T>` with `impl<'a> Parse<&'a str>` compiles.

No Tide checker exists, so every Tide verdict above is an inference from the
text and its claim to match rustc. Only `reviews/round10-consistency.md` was
written; nothing was staged or committed.
