# Tide consistency review, round 11: confirmation

## Verdict

**request_changes**

Reviewed commit `173998b94922c0cd778c68d0bc60345adb9d98be` (`173998b`,
"Answer round 10: opaque-type bounds, direction-limited final, draft 3.5"),
dated 2026-10-19 15:10 -0300, diffed against `62355d0`. The review covers
the round-10 items and the passages the commit changed in
`design/01-design.md`, `design/02-memory.md`, `design/03-authority-concurrency.md`,
`design/04-contracts.md`, `design/reference.md`, `README.md`, and
`REFERENCES.md`.

What holds:

- **Round 10's B1 is closed.** The direction-limited test in 04 §4.1 keeps
  every known escape a prophecy. `skip_ws`, `next_token`, `Ring::push`, and
  `inc` are executable again, and the summary paragraph matches the rule.
- **Round 10's B2 is closed** for the cases it named. With the new `orig`
  row, `+ Owned` gives an `Owned` result and `+ 'a` holds only `'a`'s loans.
  rustc 1.96 agrees.
- **N1, N3, and N4 are resolved, and N2 is resolved inside 02.**

Two new problems come from the changed text.

- **B1 (v0).** The new `orig` row says `{ρ}` "if + ρ". It does not say what
  happens when the bounds name two different lifetimes. rustc then ignores
  both bounds and falls back to the captured lifetimes. Any one-`ρ` reading
  accepts programs that rustc rejects, which 02 §2.8 and §11 forbid.
- **B2 (v1).** The new auto-trait bullet says outright that an opaque type is
  `ThreadBound` only if its bounds state it. Nothing requires a body whose
  hidden type is `ThreadBound` to state it. So `impl Trait` hides
  `ThreadBound` from pinning, which is the same hole round 2's B8 closed for
  `dyn`.

## Round-10 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | `design/04-contracts.md:675–677` now asks only that no declared or implied bound make the origin "outlive another origin (`'r: 'q`, that is `ρr ⊆ ρq`)", and says "The `'a: 'r` that a borrowing pointee implies does not count." The direction matches 02 §2.1 (`02-memory.md:115–116`: `'a: 'b` is `ρa ⊆ ρb`). The summary at `04-contracts.md:694–699` now says "lets outlive another origin (a shared name, a bound, or `'r: 'q`, declared or implied)" and matches the rule. Still excluded: `give` (its bound names `'r`, `:672–673`, and states `'r: 'q`), `stash` (shared name, `:678`), `bump` (`'r: 's`), `setter` (captured, `:674–675`), and `stash_generic` (a bound names `'r`). Executable again: `inc`, `skip_ws`, `next_token`, and `Ring::push`/`pop` (`:1189–1216`), whose only implied relations run into `'r`. One wrinkle about declared bounds in that direction is new N3. |
| B2 | **Resolved** for `+ Owned` and a single `+ 'a` | `design/02-memory.md:135–137`. `make(..) -> impl Fn() -> usize + 'static` is now `Owned`, in agreement with §5.2 (`:461–463`) and §3.7 (`:358–359`). `two(..) -> .. + 'a` holds only `{ρa}`, so §2.4 item 3 (`:202–204`) no longer asks for `ρb ⊆ ρa`. Putting `Owned` first matches rustc for `+ 'static + 'a`. The case of several lifetime bounds is new B1. |
| N1 | **Resolved** | All seven passages now say "in a signature or field" or "inside a body it is inferred": `design/01-design.md:218–221`, `02-memory.md:367`, `:435–436`, `:489`, `reference.md:287`, `:290–293`, `:308–309`, and `:1398`. The phrase leaves out positions outside both kinds (new N2). |
| N2 | **Resolved in 02** | `design/02-memory.md:468–471` and `:776–777` say that `Owned` follows `orig`, as Rust's `'static` does. rustc agrees (`count`, `wrap`, and `make` pass a `T: 'static` parameter). Copies of the old wording remain in `reference.md`, 03, and 02 §5.1 (new N1). The `ThreadBound` half of the new bullet is new B2. |
| N3 | **Resolved** | "Never on a trait" now attaches to lifetime parameters (`02-memory.md:158–159`). The second `Generics` alternative (`:157`) derives `fn f<T>`. `Capture` says it "lists every type parameter in scope" (`:165–166`), which matches rustc. Two smaller gaps in `Capture` are new N4. |
| N4 | **Resolved** | `REFERENCES.md:18` lists GATs once. Draft 3.5 is used in every status line: `01-design.md:3`, `02-memory.md:3`, `03-authority-concurrency.md:5`, `04-contracts.md:3`, `reference.md:4`, `:1637`, `:1652`, and `README.md:35`, `:64`. The status line says that 3.4 returned D2 to Rust's spelling and that 3.5 takes Rust 2024's lifetimes whole (`01-design.md:3–7`). The histories in D2 (`01-design.md:248–251`) and 02 §2.6 (`02-memory.md:251–254`) agree with that: 3.4 first kept a narrower subset, and D2 now takes the whole. The remaining "draft 3.4" mentions are correct history (`REFERENCES.md:30`, `02-memory.md:1296`). |

## New blocking problems

### B1. The `orig` row for a returned `impl Trait` is undefined, or wrong, when its bounds name two lifetimes

**Location:** `design/02-memory.md:135–137`, against
`design/02-memory.md:283–284` (§2.8), `design/02-memory.md:1217–1218`
(§11), and `design/01-design.md:193–195`. Tier: v0, because this is the
origin calculus that D2 calls "the definition the checker implements"
(`01-design.md:237–239`).

**Problem.** The row reads "`{ρ}` if `+ ρ`". Rust allows several lifetime
bounds on an opaque type, such as
`fn three<'a: 'b, 'b, 'c>(..) -> impl Fn() -> usize + 'a + 'b`. Neither
`{ρa}` nor `{ρb}` is singled out. A checker that takes either one:

- lets a caller drop the owner of `'c`'s data while it keeps the result;
- proves the result outlives `'b` from the `+ 'b` bound.

**What rustc does.** rustc 1.96 (edition 2024) uses an opaque type's
lifetime bound only when the bounds name exactly one lifetime, or include
`'static`. With two distinct lifetimes it falls back to the captured
lifetimes, both for the loans the value holds and for outlives checks:

- the caller that drops `tmp` (`three(&keep, &keep2, &tmp)`) is E0597;
- `Box::new(three(x, y, z))` as a `Box<dyn Fn() -> usize + 'b>` is E0309,
  "the opaque type ... may not live long enough";
- with the single bound `+ 'b`, both programs compile and run.

So the row's natural readings accept programs that rustc rejects. 02 §2.8
says v0 "accepts a body exactly when Rust's NLL borrow checker accepts its
translation". §11 says "A program that Tide accepts and rustc rejects blocks
the release". Choosing `{ρa}` would be sound, because the value's loans lie
in `ρa ∩ ρb`, but it is not Rust 2024's meaning, which D2 promises. The
case is rare, and the fix is one row.

**Proposed fix.** Replace `02-memory.md:135–137` with:

```
orig(returned impl Trait)   = ∅ if its bounds state Owned (+ 'static); else {ρ} if
                              they state exactly one lifetime ρ, written once or
                              more; otherwise, including bounds that name two or
                              more distinct lifetimes, its captured lifetimes ∪
                              orig of its captured type parameters (§5.2)
```

### B2. A returned `impl Trait` hides `ThreadBound` from pinning

**Location:** `design/02-memory.md:468–471` (new text) and `:776–777`,
against `design/02-memory.md:624–627` (§8.1 condition 2),
`design/03-authority-concurrency.md:564–576` (§5.4), and
`design/01-design.md:477–478` (D8). Tier: v1, because `ThreadBound` and the
evented runtime are v1 (`01-design.md:78`, `02-memory.md:468–469`).

**Problem.** The new bullet says: "An opaque type is `Send`, `Sync`,
`Pure`, or in v1 `ThreadBound` only if its bounds state it." §9 repeats
this: opaque types "have exactly the auto traits their bounds state". For
`Send`, `Sync`, and `Pure` that is the safe direction. A caller assumes a
property only when the bounds promise it, and the hidden type must satisfy
the bounds.

`ThreadBound` works the other way. It is a restriction, and safety needs it
to be seen. Pinning is computed "from liveness" of values whose type is
`ThreadBound`. §8.1 condition 2 and 03 §5.4 rely on "erasure cannot hide
it", but they close only the `dyn` route ("a `dyn` may hold it only as
`dyn Trait + ThreadBound`"). No rule makes a body whose hidden type is
`ThreadBound` state `+ ThreadBound`. Under the new text, this program's
`held` is not `ThreadBound`:

```tide
fn open_ctx() -> impl Debug { GlContext::new() }   // GlContext is #[thread_bound]

fn render(net: &Net, url: &Url, t: Instant) -> Result<(), NetError> {
    let held = open_ctx();          // opaque type: not ThreadBound (02 §5.2)
    let frame = net.get(url, t)?;   // suspends; no pin, so the task may migrate
    drop(held);                     // GlContext's destructor runs on another OS thread
    Ok(())
}
```

This is round 2's B8 (`reviews/round2-consistency.md:450–484`) through
`impl Trait` instead of `dyn`.

- **D12 rules out seeing through the opaque type.** The caller's pinning
  ranges would then depend on the callee's body.
- **Rust catches it differently.** Rust lets `!Send` leak through an opaque
  type, so a future that holds the value across an `.await` is not `Send`.
  rustc 1.96 rejects `assert_send(render())` with "future cannot be sent
  between threads safely". Tide drops that leak on purpose, so pinning is
  the only defence.

**Earlier wording.** The older text, "Auto traits of an opaque type are
only those its bounds state", together with `ThreadBound` in 02 §2.7's list
of auto traits, already allowed this reading. 173998b names `ThreadBound`
explicitly, so it now states the hole.

**Proposed fix.**

- Replace `02-memory.md:468–471` with:

  > - **Auto traits.** An opaque type is `Send`, `Sync`, or `Pure` only if
  >   its bounds state it, and the hidden type must then satisfy them. Rust's
  >   auto-trait leakage would make dependents depend on the function's body,
  >   which D12 forbids. In v1 a hidden type that is `ThreadBound` must be
  >   returned as `impl Trait + ThreadBound`, as a `dyn` must name it
  >   (`03-authority-concurrency.md` §5.4), so erasure cannot hide it from
  >   pinning; the opaque type is then `ThreadBound`. `Owned` follows `orig`
  >   (§2.2, §3.7), as Rust's `'static` does.

- In `03-authority-concurrency.md:568–570`, replace "A `ThreadBound` type
  may be coerced to `dyn Trait` only if the object type names
  `+ ThreadBound` (`Box<dyn Debug + ThreadBound>`); such an object pins like
  the concrete type" with:

  > A `ThreadBound` type may be coerced to `dyn Trait`, or be the hidden type
  > of a returned `impl Trait`, only if the object or opaque type names
  > `+ ThreadBound` (`Box<dyn Debug + ThreadBound>`,
  > `-> impl Debug + ThreadBound`); such a value pins like the concrete type

- In `02-memory.md:626–627`, replace "a `dyn` may hold it only as
  `dyn Trait + ThreadBound`" with "a `dyn` or a returned `impl Trait` may
  hold it only as `dyn Trait + ThreadBound` or `impl Trait + ThreadBound`".

## Non-blocking notes

### N1. Copies of the old opaque and `dyn` auto-trait wording remain outside 02 §5.2 and §9

Round 10's N2 is fixed inside 02 §5.2 and §9. Three other passages still
say that an opaque or `dyn` type has only the auto traits it names, and two
of them call `Owned` an auto trait:

- **Reference §5.1.** `design/reference.md:415–416`: "A returned
  `impl Trait` exposes only the auto traits it names". `reference.md:284`
  calls `Owned` "a compiler-known auto trait". So the agent-facing reference
  says that `fn count() -> impl Iterator<Item = u32>` is not `Owned`, and 02
  §5.2 and rustc say it is.
- **03 §3.1.** `design/03-authority-concurrency.md:313–315`: "`dyn` and
  `impl Trait` types have exactly the auto traits their bounds name".
- **02 §5.1.** `design/02-memory.md:452–453`: "A `dyn` type has no auto
  traits unless it names them". `Box<dyn Fn()>` in a signature is `Owned`
  without naming it (`:435–436`).

Only over-annotation follows, and 02 is normative.

**Fix.**

- `reference.md:415–416`:

  > A returned `impl Trait` is `Send`, `Sync`, or `Pure` only if it names
  > them: write `-> impl Iterator<Item = u8> + Send` if callers need `Send`.
  > It is `Owned` when it captures no borrow, or when it states `+ Owned`.

- `03-authority-concurrency.md:313–315`:

  > `dyn` and `impl Trait` types are `Pure`, `Send`, or `Sync` only if their
  > bounds name it (`Owned` follows origins, `02-memory.md` §3.7)

- `02-memory.md:452–453`:

  > A `dyn` type is `Send`, `Sync`, `Pure`, or `ThreadBound` only if it
  > names them (`dyn Fn() + Send + Pure`); it is `Owned` when its object
  > lifetime is `'static`, stated or by default.

### N2. "In a signature or field" leaves out the other positions outside a body

D2 (`design/01-design.md:218–221`), 02 §5.1 (`02-memory.md:435–436`,
`:443–447`), and reference §3.4 (`reference.md:290–293`) now set the
default in "signatures and fields" and infer it "inside a body". That
leaves out associated types in impls, impl headers, and the types of consts
and statics. Rust applies the `'static` default in all of them. rustc 1.96
rejects `type Item = Box<dyn Fn() -> usize>;` in `impl<'a> Iterator for
Words<'a>` when `next` boxes a closure that borrows `self.src` ("lifetime
may not live long enough"), and accepts it with `+ 'a`. D2's "mean what
they mean in Rust 2024" settles the question, but the new qualifier reads
as exhaustive.

**Fix.** In `01-design.md:218` replace "In a signature or field," with
"Outside a body (signatures, fields, impl headers, associated types, consts
and statics),". In `02-memory.md:435` replace "in a signature or field," with
"outside a body,". In `02-memory.md:443` replace "Everywhere else in
signatures and fields" with "Everywhere else outside a body". In
`reference.md:290` replace "Elsewhere in signatures and fields" with
"Elsewhere outside a body". The shorter "in a signature or field" in the
example passages can stay.

### N3. 04 §4.1 still makes `final` prophetic for a declared bound in the safe direction

The direction limit applies only to implied relations. Clause one
(`design/04-contracts.md:672–673`) still fails any `'r` that occurs "in no
bound or where clause". In Rust, these pairs of signatures mean the same
thing:

- `fn f<'r, 'a: 'r>(x: &'r mut Lexer<'a>)` and
  `fn f<'r, 'a>(x: &'r mut Lexer<'a>)`;
- `fn push<'r, T: 'r>(v: &'r mut Vec<T>, t: T)` and
  `fn push<'r, T>(v: &'r mut Vec<T>, t: T)`.

`T: 'a` is a common Rust 2015 habit. In each pair the first signature gets
a prophetic, `proof-only` `final(*x)`. The second gets an executable one.
`tide fix` then rewrites v0 clauses differently (`04-contracts.md:166–168`).
The summary (`:694–699`) says prophecies arise only where the signature
"lets outlive another origin", and these bounds do not.

The rule is sound, only conservative. rustc rejects pushing `x` into a
`&RefCell<Vec<&'a mut i32>>` under a declared `'a: 'r`, and accepts it
under `'r: 'a`.

**Fix.**

- In `04-contracts.md:672–673`, replace "occurs in no other parameter type
  and in no bound or where clause;" with "occurs in no other parameter type,
  and in no bound or where clause except on the right of an outlives bound
  (`'a: 'r`, `T: 'r`);".
- In `:676–677`, replace "The `'a: 'r` that a borrowing pointee implies
  does not count." with "A relation in the other direction (`'a: 'r`,
  `T: 'r`), declared or implied by a borrowing pointee, does not count."

### N4. Two `Capture` rules that rustc enforces are unstated

`design/02-memory.md:165–166` gains "lists every type parameter in scope".
Two related rules are still missing:

- **Bound lifetimes must be listed.** A lifetime named in the opaque type's
  bounds must be listed too. rustc rejects
  `-> impl Fn() -> usize + 'a + use<>`: "`impl Trait` captures lifetime
  parameter, but it is not mentioned in `use<...>`". B1's row depends on
  this. Without it, §5.2 would call a `+ 'a + use<>` result `Owned`, while
  the row gives `{ρa}`.
- **Argument-position `impl Trait`.** It is an unnameable type parameter
  (§5.2), so `use<..>` cannot be written while one is in scope. rustc
  rejects `fn f(x: impl Fn() -> usize) -> impl Fn() -> usize + use<>`.

**Fix.** Make the `Capture` annotation read: "only on a return-position
`impl Trait`; lists every type parameter in scope and every lifetime its
bounds name, so it cannot be written while an argument-position
`impl Trait` is in scope".

## Ready to implement v0

**Yes, after B1.** B1 is one row of the v0 origin calculus.

- **B2** concerns `ThreadBound` and the evented runtime, which are v1. Fix
  it before either is specified for implementation.
- **N1, N2, and N4** touch v0 text. The normative 02 rules and D2's "mean
  what they mean in Rust 2024" already decide each one, so they need not
  block. Settle them before the checker's test corpus is written.
- **N3** is v1.

This review did not reassess the rest of the v0 tier.

Validation: this was a read-only review of `173998b` (diffed against
`62355d0`) on 2026-10-19. Rust programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed and a timeout, in a
scratch directory outside the repository:

- **New B1.**
  - `three<'a: 'b, 'b, 'c>(..) -> impl Fn() -> usize + 'a + 'b`, with a
    caller that drops the `'c` owner: E0597.
  - Control with the single bound `+ 'b`: compiles and prints 3.
  - Control with `+ 'a + 'a`: compiles and prints 3.
  - `user` boxing `three(x, y, z)` as `Box<dyn Fn() -> usize + 'b>`: E0309
    with two bounds. With `+ 'b` alone it compiles and prints 3. With no
    bound: "lifetime may not live long enough".
  - `mk(..) -> impl Fn() -> usize + 'static + 'a` passed to a `T: 'static`
    parameter after its source is dropped: compiles and prints 3.
- **Round 10's B2, rechecked.**
  - `make(..) -> .. + 'static` through `need_static`, and `two(..) -> .. +
    'a` with the other owner dropped: compiles and prints `3 5`.
  - Control `make` without the bound: E0597.
  - `two(KEEP, &tmp)` with `'a = 'static`, passed to `need_static`:
    compiles and prints 4.
- **Round 10's B1 and new N3.**
  - These compile and print `2 [1, 2, 3]`: `skip_ws(&mut self)` on
    `Lexer<'a>`, `skip_decl` with a declared `where 'a: 'r`, and `push_t` and
    `push_i` with and without a declared `T: 'r`.
  - Control: pushing `&mut self.pos` into a log reachable through `'a`:
    "lifetime may not live long enough".
  - With a declared `'a: 'r`, pushing `x: &'r mut i32` into
    `&RefCell<Vec<&'a mut i32>>`: "lifetime may not live long enough".
  - With `'r: 'a`, the same push compiles, and the write through the log
    prints 7 after the callee returns.
- **New B2.**
  - `open_ctx() -> impl Debug` hiding an `Rc`, held across an `.await` in
    an `async fn` passed to `assert_send`: "future cannot be sent between
    threads safely".
  - Control with a `u32` hidden type: compiles and prints `ok`.
- **New N2.**
  - `type Item = Box<dyn Fn() -> usize>` in `impl<'a> Iterator for
    Words<'a>`, with a borrowing closure: "lifetime may not live long
    enough".
  - With `+ 'a`: compiles and prints 3.
- **New N4.**
  - `-> impl Fn() -> usize + 'a + use<>`: "captures lifetime parameter, but
    it is not mentioned in `use<...>`".
  - `fn f(x: impl Fn() -> usize) -> impl Fn() -> usize + use<>`: "must
    mention all type parameters in scope in `use<...>`".

No Tide checker exists, so every Tide verdict above is an inference from the
text and its claim to match rustc. Only `reviews/round11-consistency.md` was
written; nothing was staged or committed.
