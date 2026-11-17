# 3. rustc's soundness bugs, and what Tide closed

## The idea

A compiler's type rules are promises. In Rust, one promise is that a
**reference** (a pointer that borrows a value, written `&T`) never outlives the
value it points to. The compiler tracks this with **lifetimes**: labels such
as `'a` that say how long a borrow lasts. `'static` means "for the rest of the
program".

A **soundness bug** is a case where the compiler accepts a program that breaks
its own promises. rustc has some. Its issue tracker labels them `I-unsound`,
and several have been open for years. Most are obscure. But they matter to
Tide, because Tide's capability rules (chapter 1) rest on rustc's type
system. If safe code can trick rustc into treating a short borrow as
`'static`, or into reading one type's bytes as another (called a
**transmute**), it could keep a capability longer than it was lent, or forge
one.

### The first hole: rust#25860, closed by TD0206

rust-lang/rust issue #25860 has been open since 2015. When a **generic**
function (one written once for many types or lifetimes, filled in where it's
used) is turned into a **function pointer** (a plain value that holds
"which function to call"), rustc forgets some of the lifetime facts the
function relied on. A few lines of safe code then turn any borrow into a
`'static` one. The `cve-rs` crate packages this trick.

- **fx-6xi2 (2026-11-15)** was a probe: does Tide's checker stop it? It did
  not. The driver admitted rustc's own test case for #25860, and a variant
  that turns a borrowed `&'a Fs` into a `&'static Fs`, with no `unsafe` and no
  grant.
- **fx-iwjq (landed 2026-11-16)** added `TD0206`: the driver rejects turning a
  function into a function pointer when the function's signature implies a
  lifetime relationship that isn't trivially true. The rule is deliberately
  **conservative**: it rejects some harmless code too, such as a function
  pointer to a generic helper over references. The fix for that code is
  easy: wrap the call in a closure (a small inline function), which makes
  rustc check the call against the function's signature. (One related family,
  rust#100051, stays open even there; `design/02-memory.md` §5.4.) A precise rule would need changes inside rustc's borrow
  checker, and rustc's own partial fix still misses this variant.
- On 2026-11-15, after the probe, the design moved **the kernel sandbox for
  test runs (RUN-3) into v0** (D7), because the probe showed the type system alone can't bound
  what admitted code reaches. Chapter 8 covers it.

### The second hole: rust#57893, closed by TD0402

**fx-qlek.5** (landed 2026-11-17) was a red team: an agent tried over 40
escape routes against admission and the sandbox. It found two forms of one
more admitted route, rust#57893. Through it, a blanket implementation of a
trait (Rust's interfaces) can disagree with the built-in one for trait
objects (`dyn` values), and that gives safe code a general transmute, aimed
in the probe at `&Fs`. You don't need the mechanism; what matters is that
safe code could forge a capability. The fix extends `TD0402` to reject both
forms. Every other escape route the record lists was rejected or denied. Two
routes stay admitted on purpose: writing to standard error, which every
program may do (a sink, chapter 1), and a crate attribute that has no effect.
The record is `reviews/2026-11-17-red-team-opus.md`.

## Run it

rustc's own test for #25860, kept in Tide's conformance corpus:
[conformance/cases/memory/closure_fnptr_rustc_known_bug.rs](../../conformance/cases/memory/closure_fnptr_rustc_known_bug.rs).
First, the plain pinned rustc, the same compiler Tide uses, on its own:

```sh
rustc --edition 2024 --crate-type lib --emit metadata --out-dir target/guide/rustc-only conformance/cases/memory/closure_fnptr_rustc_known_bug.rs
```

It prints nothing and exits 0: rustc accepts the program.

Then the same file through Tide's driver:

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/known-bug conformance/cases/memory/closure_fnptr_rustc_known_bug.rs
```

Output (paths shortened):

```text
…/closure_fnptr_rustc_known_bug.rs:11:46: error: TD0206: `foo` cannot become a `fn` pointer: it implies `'b: 'a`, which a `fn` pointer does not keep (CLOSURE-3)
error: aborting due to 1 previous error
```

And the capability variant from fx-6xi2, which turns a lent `&Fs` into a
`&'static Fs`:
[conformance/cases/memory/closure_fnptr_capability.rs](../../conformance/cases/memory/closure_fnptr_capability.rs).

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/capability-forge conformance/cases/memory/closure_fnptr_capability.rs
```

Output (paths shortened):

```text
…/closure_fnptr_capability.rs:11:48: error: TD0206: `foo` cannot become a `fn` pointer: it implies `'b: 'a`, which a `fn` pointer does not keep (CLOSURE-3)
error: aborting due to 1 previous error
```

And the red team's case for rust#57893, both forms:
[conformance/cases/memory/escape_dyn_overlap_transmute.rs](../../conformance/cases/memory/escape_dyn_overlap_transmute.rs).

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/dyn-overlap conformance/cases/memory/escape_dyn_overlap_transmute.rs
```

Output (paths shortened):

```text
…/escape_dyn_overlap_transmute.rs:11:1: error: TD0402: this impl for a type parameter or projection that may be unsized may overlap the built-in impl of the trait's own `dyn` type, which may choose another associated type (rust-lang/rust#57893); drop `?Sized`, or implement the trait for each type
…/escape_dyn_overlap_transmute.rs:50:5: error: TD0402: this impl for a type parameter or projection that may be unsized may overlap the built-in impl of the trait's own `dyn` type, which may choose another associated type (rust-lang/rust#57893); drop `?Sized`, or implement the trait for each type
error: aborting due to 2 previous errors
```

## What to look at

- rustc alone prints nothing and exits 0: it accepts the unsound program.
  The driver rejects it with `TD0206`, at the line that makes the function
  pointer, and names the lifetime fact (`'b: 'a`, "`'b` lasts at least as
  long as `'a`") that the pointer would lose. The driver prints errors in a
  short one-line format, so the rule's suggested fix, a closure, isn't
  shown; it is the `HELP` text you'll read below.
- Read the capability file. It is 13 lines, and it would let code keep a
  file-system capability forever after being lent it for one call. That is
  why the probe mattered.
- The red team's case gets two `TD0402` errors, one per form: line 11 is
  the blanket form, and line 50, inside `mod projection`, the second form.
  The message's suggested fix is shown here because it is part of the
  one-line error itself.
- The `//~ TD0206` comments in the files are the corpus's expected verdicts.
  The conformance runner (`cargo run -p tide-conformance`) checks every case
  against them.

## Read the code

**`crates/tide-admit/src/walk/reify.rs:1-55`** (10 minutes). The check behind
`TD0206`. "Reify" is the compiler's word for turning a function into a
function pointer.

```sh
bash -c 'sed -n "1,55p" crates/tide-admit/src/walk/reify.rs'
```

- Lines 1–6, the `//!` comment: the rule in one paragraph. A function pointer
  doesn't carry the lifetime facts (*outlives bounds*) the function's body
  may assume, and rustc doesn't check them at that moment (#25860), so the
  driver rejects the conversion whenever the function implies such a fact,
  "whatever the instantiation". That last phrase is what *conservative*
  means: it doesn't try to work out whether this particular use is harmful.
- Lines 13–15, `HELP`: the fix the error suggests, a closure.
- Line 19, `pub(super) fn reify(..)`: called once for every place in the
  program where a function becomes a function pointer. `e` is that
  expression.
- Lines 20–22: if what's being converted isn't a named function, there's
  nothing to check. `return` stops here.
- Lines 23–25: skip code the toolchain itself generated.
- Lines 28–48: a wrinkle for **trait methods** (functions that belong to an
  interface): the code that actually runs is the chosen implementation, and
  it can assume more, so the driver adds it to the list of things to check.
  If it can't tell which implementation will run, it rejects (lines 38–45).
  You can skim this part.
- Lines 49–55, the heart: for each function on the list, `implied_bound`
  looks for a lifetime fact that doesn't hold trivially. If it finds one, the
  driver reports `TD0206` with a message naming the fact.

What to take away: the rule is short and blunt on purpose. It needs no
reasoning about lifetimes inside rustc's borrow checker, which is where
rustc's own fix attempts have struggled. The price is rejecting some
harmless code, and the help line tells the writer how to get around it.

## The hardest question

> "rustc has many open soundness bugs. You closed one or two. Isn't
> 'capability safety' just marketing?"

**Be exact about the claim.** Tide claims **no accidental ambient authority**:
code can't call the world's operations (other than printing) without being
handed a capability. That
claim, like every type-based guarantee, rests on the compiler being sound,
whoever wrote the code and whatever they intended. The README says it plainly:
safety against a malicious dependency rests on rustc's soundness, which has
open `I-unsound` issues. Tide has closed particular known routes; other
soundness bugs may remain.

**What Tide does about it:**

1. It closes the routes it finds. `TD0206` closes the function-pointer route
   of #25860; `TD0402` closes rust#57893 (fx-qlek.5). The red team also
   checked that the known reproductions of the other famous routes (#25860
   through closures and trait objects, #84591, #100051) are rejected. That
   covers those reproductions, not every variant: the #100051 family stays
   open in the design (`design/02-memory.md` §5.4).
2. It doesn't rely on the type system alone. Test and simulation runs that
   the driver starts (`--tide-run-test`, `--replay`) happen inside a kernel
   sandbox (chapter 8). Even code that forges a capability meets the
   operating system's fence there. A test binary run directly gets no
   sandbox.
3. It is honest about the gap. Deployed programs get kernel enforcement only
   in v1. Until then, a malicious dependency in a deployed binary is bounded
   by rustc's soundness, Tide's closures of the known routes, and review.

**A good one-liner:** "The type system explains what code may do; the kernel
sandbox enforces it for the test runs the driver starts today, and for
deployed programs in v1."

## Going deeper

- `design/01-design.md` D7, the "Kernel enforcement" bullet (the decision of
  2026-11-15).
- `design/02-memory.md` §5.4 (CLOSURE-3, the `TD0206` rule and its cost).
- `design/03-authority-concurrency.md` §2.6, RUN-4 hole 5 (the toolchain's
  own bugs).
- `reviews/2026-11-15-achilles-opus.md` W3 (the critique that started it).
- Beads: `br show fx-6xi2`, `br show fx-iwjq`, `br show fx-qlek.5`.
