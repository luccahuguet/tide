# Tide consistency review, round 18: confirmation of the round-17 answers

## Verdict

**approve with notes**

Reviewed commit `1b01701703013572e746c17caaee2ae8126e4727` (`1b01701`, "Add
round 17 and answer it: FX0803 scope, abort verdict"), dated 2026-10-19
19:24 -0300. It was diffed against `698b1dd`. Scope, as asked:

- whether each round-17 item (N1–N6) is closed as its Resolution claims.
  The owner's decisions on bead `fx-wsd` are taken as given;
- whether the changed lines contradict 01, 02, 03, 04, the reference, or
  the README;
- whether the `FX0803` rule reads the same in 04 CLAUSE-4, 04 §1.2 rule 4,
  01 D10, and reference §10.1, and agrees with 04 §4.1;
- the new Rust and libtest claims, which were checked on rustc 1.96
  (Validation).

The rest of the design was not re-audited; rounds 12–17 did that.

What holds:

- **The four texts now forbid the same thing.** Rule 4 (`04:192–193`),
  CLAUSE-4 (`04:434–435`), 01 D10 (`01:638–639`), and reference §10.1
  (`reference.md:1182`) all make naming `x` outside `old(..)` `FX0803`.
  So `old(*x)`, `old(x.len())`, and `old(x[0])` are allowed everywhere.
- **"`x`'s origin" means the outer one.** 02 says "A reference lists only
  its outer origin" (`02-memory.md:146`). So 01's and CLAUSE-4's wording
  matches §4.1's "the origin of `x`'s outer reference" (`04:744–745`).
  Round 17's `h<'a>(x: &mut &'a str, y: &'a str)` stays executable in both
  texts.
- **CLAUSE-4 is exact.** Its parenthetical keeps §4.1's outlives-bound
  exception, and it cites §4.1.
- **PAR-3's new sentence is true on rustc 1.96** (Validation).

## Round-17 item status

| Item | Status | Evidence |
| --- | --- | --- |
| N1 | **Resolved** | The four places cited above |
| N2 | **Resolved** for the shared-lifetime case; see N1 below | `01:637–638`, `reference.md:1181–1183`, `04:432–436` |
| N3 | **Resolved**, but the wording is too broad; see N2 below | CLAUSE-7 (`04:477–479`) |
| N4 | **Resolved** in 03; reference §9 not updated, see N3 below | PANIC-1 (`03:247–263`) |
| N5 | **Resolved** | PAR-3 (`03:967–969`) |
| N6 | **Resolved**, apart from one reflow and one tier mark (N4) | PANIC-1 names `tide test` or `tide sim` (`03:255`); `03:372–373` rewrapped; D7 and D18 marked v1 (`01:475`, `01:934`); README (`README.md:58–59`, bead `fx-fbq` exists); `04:72`; D10 cites `FX0803` |

## New blocking problems

None.

## Non-blocking notes

### N1. 01 D10 and the reference drop §4.1's outlives-bound exception

§4.1 (`04:744–748`) does not count `x`'s outer origin when it appears
only on the right of an outlives bound (`'a: 'r`, `T: 'r`). CLAUSE-4 keeps
this exception (`04:433–434`). 01 D10 (`01:638`) and reference §10.1
(`reference.md:1182`) say only "or a bound". 01 at least cites 04 §4.1;
the reference does not. The verdicts differ for this shape:

```rust
#[ensures(v.len() == old(v.len()))]
fn fill<'r, T: Clone + 'r>(v: &'r mut [T], x: T) { /* ... */ }
```

- §4.1 and CLAUSE-4: `'r` appears only on the right of `T: 'r`, so
  `final(*v)` is executable and the clause is accepted.
- The reference: `'r` appears in a bound, so the clause is `FX0803`.

The reference errs on the safe side, but round 17 asked that it predict
§4.1's verdicts before the corpus is written. Also, the reference's "the
result type" does not say that a returned `impl Trait` captures `v`'s
elided lifetime (02 §5.2; `04:748`). `fn evens(v: &mut Vec<u32>) -> impl
Iterator<Item = u32>` is therefore `FX0803`, though no lifetime appears in
the result type.

**Fix.**
- In reference §10.1, write: "If `v`'s lifetime appears in another
  parameter's type, the result type (a returned `impl Trait` captures it),
  or a bound other than `T: 'v` or `'a: 'v` (04 §4.1), …".
- In 01 D10, write "or a bound other than on the right of an outlives
  bound".

### N2. CLAUSE-7 erases "a named lifetime", but only the outer one may be erased

CLAUSE-7 (`04:477–479`) says the lowering "erases a named lifetime in the
closure parameter's type". Erasing a named lifetime in the pointee breaks
a case that §4.1 calls executable:

```rust
#[ensures(*x == y)]
fn h<'a>(x: &mut &'a str, y: &'a str) { *x = y }
```

- With the closure typed `|x: &mut &'a str|`, it compiles and the clause
  holds.
- With `|x: &mut &str|`, rustc rejects `*x = y` ("assignment requires that
  `'a` must outlive `'static`").

Validation shows both. Round 17's proposed text ("a fresh lifetime in place
of a named outer lifetime") avoided this. Also, "It also erases" reads as
something the lowering does now, but the sentence sits in a list of what
the spike still lacks.

**Fix.** "It also needs to erase a named outer lifetime in the closure
parameter's type, keeping the pointee's: `|x: &'r mut i32|` is rejected
(E0503) where `|x: &mut i32|` compiles."

### N3. Reference §9 does not carry PANIC-1's new verdict

Round 17's N4 fix asked for its gist in reference §9 as well. §9
(`reference.md:1151–1154`) still says only that `tide test` and `tide sim`
"rerun each test and seed in flight alone, report each that aborts …, and
go on in a fresh process". An agent that loads only the reference would
predict that a run whose reruns all pass is not reported as failed.
PANIC-1 (`03:258–260`) now says that it fails.

**Fix.** Add to §9: "If none aborts alone, the run still fails and names
the tests and seeds that were in flight."

### N4. Nits

- **"since libtest reports none"** (`03:254–255`) overstates it. With
  `--test-threads=1`, libtest prints `test b_abort ...` before the test
  runs (Validation). It reports no starts only when it runs tests
  concurrently. That is the case PANIC-1 needs, so the rule stands. Write
  "since libtest does not report a test's start when it runs tests
  concurrently".
- **Seeds left out** (`03:259–260`): "reports the abort with the tests that
  were in flight". Write "tests and seeds", as the sentences before it do.
- **Run-on sentence** (`03:251–255`). It joins three clauses with "and".
  Split it before "Tide's `#[test]`".
- **Reflow** (`reference.md:660–661`). "ends," still sits alone on a line.
- **Tier mark placement** (`01:474–475`). "a Tide crate (v1)" reads as if
  the crate were v1. Write "In v1, Rust code that calls a Tide crate is
  unchecked".
- **Header count.** The reference header's new figure, 39,330, was not
  re-measured here.

## Ready to implement v0

**Yes.** Every round-17 item is closed where its Resolution says. The four
statements of `FX0803` forbid the same thing, and CLAUSE-4 states §4.1's
condition exactly.

Before the checker's test corpus is written, settle:
- N1, so that the reference predicts §4.1's verdict for outlives bounds
  and captured lifetimes;
- N3, before the corpus has a test that aborts.

N2 is an implementation point for the D18 lowering.

Validation: this was a read-only review of `1b01701` (diffed against
`698b1dd`) on 2026-10-19. Programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in
`/tmp/claude-1000/r18`. The `&mut` cases used round 16's lowering:
`let __caller = x;`, the body as a closure called on `&mut *__caller`, and
the clause reading `*__caller`.

- **CLAUSE-7's E0503 claim.** `g<'r>(x: &'r mut i32)` with the closure
  typed `&'r mut i32` failed with E0503. With `&mut i32` it compiled and
  ran.
- **N2.** `h<'a>(x: &mut &'a str, y: &'a str)` with `*x = y`. Typed
  `|x: &mut &'a str|`, it compiled and ran (output `r`). Typed
  `|x: &mut &str|`, it was rejected: "lifetime may not live long enough …
  `'a` must outlive `'static`".
- **libtest.** In a three-test binary whose second test calls
  `process::abort()`:
  - with `--test-threads=1`, the output was `test a_ok ... ok`, then
    `test b_abort ...`, then the abort (exit 134);
  - with `--test-threads=3`, it was only `running 3 tests` before the
    abort.
- **PAR-3.** `x.mul_add(a, b)` with `black_box` inputs, built with `-O` at
  the baseline CPU, printed `3.25`. `nm -D` lists no `fma`. `nm` shows a
  local `t fma` beside `compiler_builtins`'s `fma_fallback` and
  `fma_with_fma`.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the commit that adds this review. It closes bead fx-wsd: rounds
16–18 found no v0-blocking contradiction, and this answer is text-only, so
no further round follows (AP-PROOF-001's limit on text-only restarts).

- **N1:** 01 D10 and reference §10.1 now keep §4.1's exception for the
  right side of an outlives bound. The reference cites §4.1 and says a
  returned `impl Trait` captures `v`'s lifetime.
- **N2:** CLAUSE-7 says the lowering must erase only the outer reference's
  named lifetime and keep a pointee's.
- **N3:** reference §9 states that the run fails even if no rerun aborts.
- **N4:**
  - PANIC-1 limits "libtest reports none" to tests run concurrently, and
    names seeds too;
  - the run-on sentence is split;
  - `reference.md:660` is rewrapped;
  - D7 reads "a Tide crate (RUST-6, v1)";
  - the header's count is remeasured.
