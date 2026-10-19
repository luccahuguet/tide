# Tide consistency review, round 17: confirmation of the round-16 answers

## Verdict

**approve with notes**

Reviewed commit `698b1ddadf83bc549f50a56799f945fb2ed3be6a` (`698b1dd`, "Add
round 16 and answer it: escaping &mut parameters, abort reruns"), dated
2026-10-19 19:15 -0300. It was diffed against `9bac29e`. Scope, as asked:

- whether each round-16 item (N1–N7) is closed as its Resolution claims.
  The owner's decisions on bead `fx-wsd` are taken as given;
- whether the changed lines contradict 01, 02, 03, 04, the reference, or
  the README, and whether stale wording survives elsewhere;
- whether `FX0517`, `FX0802`, and `FX0803` collide with existing codes;
- whether the `FX0803` condition reads the same in 04 CLAUSE-4, 04 rule 4,
  01 D10, and reference §10.1, and whether it agrees with 04 §4.1's split
  between executable and prophetic `final`. Rust claims were checked on
  rustc 1.96 where that was cheap (Validation).

The rest of the design was not re-audited; rounds 12–16 did that.

What holds:

- **Every item is answered where the Resolution says** (table below).
- **The codes do not collide.** Before this commit no file in the
  repository used `FX0517`, `FX0802`, or `FX0803` (`git grep` at `9bac29e`).
  Afterwards they appear only in 01, 03, 04, the reference, and round 16.
  `FX0801` survives only in reviews, as history. No other file uses an
  `FX08xx` code, and nothing under `spikes/` or `trials/` uses any of the
  three.
- **No old wording survives in the changed areas.** No text still says any
  of these:
  - "each for `f32` and `f64`";
  - "the seed whose manifest was in flight";
  - "counted in CI" as a present fact;
  - `FX0801` in a live document.

  02:728–731, 04:633, and 03:888 say only that an overflow aborts, or that
  it fails the seed. D5 (`01:364–366`) defers to PANIC-1. All of these stay
  true.
- **The `FX0803` condition has the same two parts everywhere it is spelled
  out**: the result borrows from the parameter, or the parameter's borrow
  can outlive the call. That is CLAUSE-4 (`04:432–434`), 01 D10
  (`01:636–638`), and reference §10.1 (`reference.md:1181–1182`). Rule 4
  (`04:192–193`) cites it as "where `final(*x)` would be a prophecy". 04
  anchors it to §4.1. The paraphrases in 01 and the reference do not anchor
  it (N2), and the four texts forbid different things (N1).

## Round-16 item status

| Item | Status | Evidence |
| --- | --- | --- |
| N1 | **Resolved**, with N1–N3 below | `FX0803` in rule 4, CLAUSE-4, 01 D10, and the reference (lines above). CLAUSE-7 names the generic helper and the renamed `self` (`04:471–475`) |
| N2 | **Resolved** (recorded) | CLAUSE-7 (`04:473–475`) and D18 (`01:941–943`) |
| N3 | **Resolved**, with N4 below | PANIC-1 (`03:247–260`), REPLAY-1 (`03:1124–1125`), reference §9 (`reference.md:1150–1154`), which now covers PANIC-2's abort |
| N4 | **Resolved** | 03 §3.1 (`03:367–370`), reference 6.6 (`reference.md:657–659`), D18's mapping with `FX0511` and the float-fixture note (`01:943–948`) |
| N5 | **Resolved**, with a nit in N6 | Grammar comment (`04:109–110`), rule 6 (`04:202–205`) |
| N6 | **Resolved** in text; see N5 below | PAR-3 (`03:966–967`) |
| N7 | **Resolved**, with leftovers in N6 | Generic float functions (`01:327`, `03:964–965`, `reference.md:406–407`); reference header method (`reference.md:9–13`); CI planned (`01:205–206`, `:823`); open question 8 (`01:1074–1075`); habit row (`reference.md:1459`); v1 marks (`01:62`, `03:411`, `README.md:147`); `FX0802` |

## New blocking problems

None.

## Non-blocking notes

### N1. The four statements of `FX0803` forbid different things

They agree on the condition but not on what the rule forbids:

- **04 rule 4** (`04:190–193`) makes "a bare `x` or `*x`" `FX0803`. Under
  this reading, `old(v.len())` is allowed.
- **04 CLAUSE-4** (`04:433–434`) makes "naming `x` or `*x` in an `ensures`"
  `FX0803`, and exempts only `old(*x)`. Under this reading, `old(v.len())`
  names `v`, so it is an error.
- **01 D10** (`01:638`) and **reference §10.1** (`reference.md:1182`) say
  "only `old(*x)` may be named". Under this reading, `old(v.len())` and
  `old(v[0])` are errors.

A conformance test for
`#[ensures(*result == old(v[0]))] fn first(v: &mut Vec<u8>) -> &mut u8`
would have to guess which reading holds. The lowering has no trouble with
any `old(e)`. Every `old` runs at entry, before the body takes its
reborrow. With the generic-helper lowering, `first` compiles with
`old(v[0])` and `old(v.len())` and runs (Validation).

**Fix.** Use rule 4's reading in all four places, unless the owner meant
the narrower one:

> … naming `x` or `*x` in an `ensures` outside `old(..)` is `FX0803` in
> v0.

In 01 and the reference, write "`x` may be named only inside `old(..)`".
If only `old(*x)` is meant, change rule 4 to match, and the reference
should then say that `old(v.len())` is rejected too.

### N2. 01 and the reference paraphrase §4.1's condition in terms that §4.1 does not use

§4.1 (`04:739–748`) makes `final(*x)` executable only under syntactic
conditions:

- `x`'s outer origin occurs in no other parameter type;
- it occurs in no bound or where clause, except on the right of an
  outlives bound;
- it is not among the result's origins;
- no bound makes it outlive another origin.

CLAUSE-4 points to §4.1 ("Where §4.1 makes `final(*x)` a prophecy"), so 04
is exact. 01 D10 and reference §10.1 say only "`x`'s borrow can outlive the
call". Read semantically, that sentence gives the wrong verdict for a
common shape:

```rust
#[ensures(*x >= *y)]
fn k<'a>(x: &'a mut i32, y: &'a i32) { *x += *y }
```

Nothing in `k`'s signature can hold `x`'s borrow past the call. Yet `'a`
occurs in `y`'s type, so §4.1 calls `final(*x)` a prophecy, and the clause
is `FX0803`. Sharing one lifetime between parameters is common style
(`fn parse<'a>(buf: &'a mut Buf, src: &'a str)`). An agent that loads only
the reference would predict that the clause is accepted. Validation shows
that the lowering could accept `k` if it erased `'a` from the closure
parameter, so `FX0803` here comes from §4.1's conservative rule, not from
a limit of the lowering.

**Fix.** In reference §10.1 and 01 D10, state the condition as §4.1 does,
for example:

> If the result borrows from `v`, or `v`'s lifetime is named elsewhere in
> the signature (another parameter, a bound; 04 §4.1), …

### N3. The lowering also fails for a named lifetime that §4.1 calls executable

CLAUSE-4 (`04:430–431`) says "the lowering passes the body a reborrow and
reads the caller's reference after it." Round 16 checked this only with
elided lifetimes. Take a parameter whose outer lifetime is named but occurs
nowhere else:

```rust
#[ensures(*x == old(*x) + 1)]
fn g<'r>(x: &'r mut i32) { *x += 1 }
```

`final(*x)` is executable under §4.1. But the lowering keeps the declared
type on the closure parameter, `|x: &'r mut i32| ..`. The reborrow must
then live for `'r`, which outlives the function body, so reading
`*__caller` afterwards is E0503 (Validation). With the lifetime erased
(`|x: &mut i32| ..`), it compiles and the clause holds. The erasure is
sound in exactly §4.1's executable case, because then no other part of the
signature names `'r`. It breaks only if the body itself writes `'r` in a
type that it gives `x`.

**Fix.** Add to CLAUSE-7's list of lowering needs (`04:471–475`): "a fresh
lifetime in place of a named outer lifetime on the reborrowed parameter."

### N4. PANIC-1 gives no verdict when no rerun aborts, and plain tests in flight are not observable

PANIC-1 (`03:254–258`) reruns each in-flight test and seed alone, and
reports each that aborts alone. It also says that "a rerun or replay may
not reproduce" an overflow (`03:259–260`). Neither statement says what the
run reports when the child aborted but no rerun does. Read literally:

- the abort goes unreported;
- each in-flight test gets its rerun's verdict, a pass included;
- `tide test` can then exit 0 after its child aborted.

That also strains PANIC-1's first sentence: an overflow "under the
simulator ... fails the seed".

Separately, the driver must know which tests were in flight. For a seed,
the pre-run manifest records it. A plain test has no manifest. Stable
libtest announces a test only when it finishes, when it runs tests on
several threads. A three-test binary whose third test aborted printed
only `running 3 tests` (Validation). libtest's JSON format does emit
per-test `started` events, but only with `-Z unstable-options`. That flag
was refused on a stable binary and accepted under `RUSTC_BOOTSTRAP=1`.

**Fix.** Add a sentence to PANIC-1, and its gist to reference §9:

> If no rerun aborts, the run still fails: it reports the abort and every
> test and seed that was in flight.

Also say how a plain test's start is recorded. Tide's `#[test]` wrapper
(D6) can write it, as the manifest does for a seed. Alternatively, the
driver can use libtest's JSON events.

### N5. On rustc 1.96, `mul_add` already avoids the C library's `fma` in a linked binary

Round 16's N6 found `jmpq *fma@GOTPCREL`, a call to an `fma` that the
linker resolves later. In a linked `x86_64-unknown-linux-gnu` binary,
built at the baseline CPU in both debug and release, that call reaches a
local `fma` from `compiler_builtins` (`compiler_builtins::math::libm_math::arch::x86::fma`).
That `fma` dispatches at run time to the FMA instruction or to a software
`fma_fallback`. The binary does not import `fma` from glibc (`nm -D`),
and the GOT slot is an `R_X86_64_RELATIVE` relocation to the local symbol
(Validation). A user-defined `#[unsafe(no_mangle)] extern "C" fn fma`
overrides it.

So the new sentence in PAR-3 (`03:966–967`) is true in result. But it
leaves three points unsaid:

- **Which library.** "std's math library" can mean Rust's
  `compiler_builtins` port or D15's single Tide library. The two coincide
  only if Tide's library is that port.
- **How.** It names no mechanism. D3's premise (`01:321–322`) is that
  inherent float methods cannot be overridden.
- **Where.** The choice happens per CPU, at run time, not per target.

Both paths are correctly rounded, so results do not differ. Only the code
being trusted does.

**Fix.** Say which library and how, for example:

> `mul_add` never calls the C library's `fma`: it links std's correctly
> rounded `fma` symbol, which uses the instruction when the CPU has it.

Or record the choice as implementation work in D18.

### N6. Nits

- **"the driver"** (`03:254`). 03 also uses "The driver" for the admission
  driver (ADMIT-2, `03:353`). D5 says "the test driver" (`01:365`). Write
  "they" (`tide test` and `tide sim`) or "the test driver".
- **Unreflowed lines.** `03:370–372` ("that is / authority-free in /
  fact") and `reference.md:659–661` ("and / other / handles").
- **Untiered Rust consumers.** Round 16 listed two places. Two more remain
  without a tier mark:
  - D7's "Rust code that calls a Tide crate is unchecked" (`01:474–476`);
  - D18's "A generated `Cargo.toml` is an export for Rust consumers"
    (`01:933`).

  Add "(v1)" to each, as 03 §3.1 now has.
- **README budget.** `README.md:58` describes the reference as "under a
  25,000-token budget", but the reference header measures 39,302. Write
  "with a 25,000-token budget (currently over; 01 open question 8)".
- **`#[invariants]` row.** 04 §1.1 (`04:72`) places `#[invariants]` "on a
  function". The row above it says "a function or inherent method", so the
  contrast reads as excluding methods. Rule 6 (`04:205`) now allows the
  attribute in trait-impl and default methods. Write "on a function or
  method".
- **Code in 01.** 01 D10 (`01:636–638`) states the rule without
  `FX0803`. D1 cites `FX0512` in the same way, so this is optional.

## Ready to implement v0

**Yes.** Every round-16 item is closed where its Resolution says. The new
codes collide with nothing. The owner's two decisions read the same in 01,
03, 04, and the reference, apart from N1's scope difference and N2's
paraphrase.

Before the checker's test corpus is written, settle:
- N1's reading of `old(e)` for a parameter under `FX0803`;
- N2's wording, so that the reference predicts the same verdicts as §4.1.

Also settle N4's verdict before the corpus has a test that aborts. N3 and N5
are implementation points; record them where CLAUSE-7 and D18 list the
others.

Validation: this was a read-only review of `698b1dd` (diffed against
`9bac29e`) on 2026-10-19. Programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in
`/tmp/claude-1000/r17`. The `&mut` cases used round 16's lowering:
`let __caller = x;`, snapshots, the body as a closure called on
`&mut *__caller`, and the clause reading `*__caller`.

- **N3.** `g<'r>(x: &'r mut i32)`, with the closure parameter typed
  `&'r mut i32`, failed with E0503 ("argument requires that `*__caller` is
  borrowed for `'r`"). With `&mut i32`, it compiled and ran.
- **N2.** `k<'a>(x: &'a mut i32, y: &'a i32)` compiled and ran with the
  closure parameter typed `&mut i32`. So did
  `h<'a>(x: &mut &'a str, y: &'a str)`, whose pointee lifetime is shared
  but whose outer reference is elided, a case §4.1 calls executable.
  Output: `erased 2 q 3`.
- **N1.** `first(v: &mut Vec<u8>) -> &mut u8`, lowered through a generic
  `call(a, |v| &mut v[0])` helper, checked `*result == old(v[0])` and
  `old(v.len()) > 0`, and ran (`first [8, 8]`).
- **N4.** In a libtest binary with three tests and `--test-threads=3`, one
  test called `process::abort()` while two slept. The output was only
  `running 3 tests`, then the abort (exit 134). With
  `-Z unstable-options --format json`, the stable binary refused ("only
  accepted on the nightly compiler"). Under `RUSTC_BOOTSTRAP=1`, it
  printed a `started` event for each of the three tests before the abort.
- **N5.** `x.mul_add(a, b)` with `black_box` inputs, at the baseline CPU,
  with and without `-O`: `main` calls through a GOT slot whose relocation
  is `R_X86_64_RELATIVE` to a local `t fma`. That function loads
  `compiler_builtins::…::fma::FUNC` and calls it. Beside it sit
  `fma_fallback` and `fma_with_fma`. `nm -D` lists no `fma`. A binary that
  defines `#[unsafe(no_mangle)] pub extern "C" fn fma` printed from its own
  function.
- **Codes.** `git grep` for `FX0517`, `FX0802`, and `FX0803` at `9bac29e`
  found nothing. At `698b1dd` they appear only in 01, 03, 04, the
  reference, and round 16.
- **Reference header.** `claude --help` lists the `--model`, `--tools`,
  `--system-prompt`, and `--output-format` flags that the method uses. The
  39,302 count was not re-measured.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the commit that adds this review.

- **N1:** all four places now forbid naming `x` outside `old(..)`, so
  `old(*x)` and `old(x.len())` are allowed (04 CLAUSE-4 and rule 4, 01 D10,
  reference §10.1).
- **N2:** 01 D10 and reference §10.1 now state §4.1's syntactic condition:
  `x`'s origin appears in another parameter's type, the result type, or a
  bound. CLAUSE-4 cites §4.1 and keeps its outlives-bound exception. The
  reference and CLAUSE-4 give the usual fix, which is to give the other
  parameter its own lifetime.
- **N3:** CLAUSE-7's list of lowering needs adds erasing a named lifetime
  in the closure parameter's type.
- **N4:** PANIC-1 now says that Tide's `#[test]` writes a start record for
  every test, and that the run fails even when no rerun aborts. In that
  case the report names the tests that were in flight.
- **N5:** PAR-3 names the toolchain's correctly rounded software `fma`
  (`compiler_builtins` today), and the choice is now per CPU, not per
  target.
- **N6:**
  - PANIC-1 names `tide test` or `tide sim` where it said "the driver";
  - `03:370–372` and `reference.md:659–661` are rewrapped;
  - D7 and D18 mark Rust consumers v1;
  - the README says the reference exceeds its budget (bead `fx-fbq`);
  - the `#[invariants]` row in 04 §1.1 includes methods;
  - D10 gives `FX0803`.
