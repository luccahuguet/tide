# Tide consistency review, round 41: confirmation of the round-40 fix

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `4a05cc7`
("Require a Codex supervisor pass after clean Opus reviews"):
`git diff -- design/` (SHA-256 `68ae19af…9ebd`), which changes `02-memory.md`
§9, `03-authority-concurrency.md` (ADMIT-1, §3.1, §3.3, the §3.5 table,
LOCK-4, SUSP-1), `04-contracts.md` §4.4, `reference.md` 2.2 and 6.6, and the
companion's core count. It implements bead fx-cze and applies round 40's
fix in 02 §9 and ADMIT-1. This is a narrow confirmation round. Scope, as
asked:

- whether round 40's B1 (routes A and B) and N1 are closed;
- whether any safe route remains that leaves an `io::Error` holding a
  non-`Pure` concrete payload, given that the conversion must reach one of
  alloc's four `From` impls for `Box<dyn Error + Send + Sync>` by identity.
  Candidates tried: `E = Box<dyn Error + Send + Sync>` against the
  `From<E: Error>` impl, `E = io::Error`, `Arc<E>`, `&'static E`, and
  generic code;
- whether the new ADMIT-1 bullet agrees with 02 §9 and with ADMIT-1's
  "keyed on crate identity" rule.

Nothing else was re-audited.

## Round-40 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1, route A (user `From<Key> for Box<dyn Error + Send + Sync>`) | **Resolved** | The rule no longer checks the top-level `Into` impl. "The conversion ... must reach one of alloc's four `From` impls ... by identity", and "Anything else is `FX0517`, including a user `From` or `Into` impl" (`02:876–881`). ADMIT-1 names the same exclusion (`03:393–396`). The user's impl is not one of the four, so it is rejected whichever obligation selects it. |
| B1, route B (reflexive `From<T> for T` on an erased box) | **Resolved** | Excluded by name: "and the reflexive `From<T> for T`" (`02:881`, `03:396`). It is core's impl (`core/src/convert/mod.rs:785`), not one of alloc's four, so the identity rule also rejects it without the name. |
| B1, ADMIT-1 wording | **Resolved** in substance, with a code clash (B1 below) and a wording point (N1 below) | `03:393–396` replaces "a user impl selected for" with "any conversion ... other than alloc's four `From` impls", as round 40 asked. |
| N1, the cost | **Resolved** | `02:896–899`: "an already-boxed `Box<dyn Error + Send + Sync>`, such as `into_inner()`'s result, cannot be wrapped in an `io::Error` again (wrap the concrete error)". |

**Remaining routes.** Read in Rust 1.96.0's rust-src. Each candidate below
either reaches no admitted impl or reaches
`impl<'a, E: Error + Send + Sync + 'a> From<E> for Box<dyn Error + Send + Sync + 'a>`
(`alloc/src/boxed/convert.rs:525`), where 02's "with `E: Pure`"
(`02:879`) decides it. rustc accepts every construction below that reaches
that impl (witness under Validation).

- **`E = Box<dyn Error + Send + Sync>`.** This does not match the
  `From<E: Error>` impl. `impl<E: Error> Error for Box<E>`
  (`alloc/src/boxed.rs:2394`) has an implicit `Sized` bound, so
  `Box<dyn Error + Send + Sync>` is not `Error`. rustc 1.96 confirms this:
  `need::<Box<dyn Error + Send + Sync>>()` with `E: Error` fails with
  E0277 on `Sized`. The only std impl that matches is the reflexive one,
  and the rule excludes it. **Closed.**
- **`E = io::Error`.** `io::Error` is `Error + Send + Sync + 'static`
  (`std/src/io/error.rs:1061`) and `Pure` under 02's erased-payload rule
  (`02:871–873`, `reference.md:92–94`). It reaches `From<E>` with `E: Pure`.
  The inner payload is itself one that the rule admitted, so the verdict is
  a sound greatest fixed point (`02:800`). `downcast::<io::Error>()` returns
  that `Pure` value, and `into_inner()` returns an erased box, which is not
  `Pure` (`02:889`). **Closed.**
- **`Arc<E>`, `Arc<dyn Error + Send + Sync>`.**
  `impl<T: Error + ?Sized> Error for Arc<T>` (`alloc/src/sync.rs:4305`)
  makes both `Error`, and both reach `From<E>`. `Arc` is a synchronization
  handle (`02:827`), so `E: Pure` fails. **Closed.**
- **`&'static E`.** `impl<'a, T: Error + ?Sized> Error for &'a T`
  (`core/src/error.rs:1100`) makes `&'static P` (for example from
  `Box::leak`) and `&'static (dyn Error + Send + Sync)` both `Error`, and
  both reach `From<E>`. `&T` is `Pure` exactly when `T` is (`02:838–839`).
  `&'static P` is therefore rejected when `P` is not `Pure`.
  `&'static (dyn Error + Send + Sync)` is rejected by the `dyn` rule
  (`02:834`), even when the payload behind it is `Pure`. That conservative
  rejection is the same shape as the stated "already-boxed" cost.
  `&'static (dyn PureErr + Send + Sync)`, for a companion trait, is `Pure`,
  and its concrete type was checked at the coercion (03 §3.1). **Closed.**
- **`Box<P>`.** `Box<P>: Error` for a sized `P: Error`. It reaches `From<E>`,
  and `Box` is walked (`02:798`), so `E: Pure` fails for a non-`Pure` `P`.
  **Closed.**
- **Generic code.** Round 40's analysis still holds.
  - In `fn w<E: Error + Send + Sync + Pure + 'static>(e: E)`, the
    placeholder `E` unifies only with `From<E>`'s head. The reflexive impl
    needs `E == Box<…>`, and the string impls need `E` to be a string type.
  - A wrapper bounded by `E: Into<Box<…>> + Pure` reaches none of the four
    impls, because the obligation is met by its where-clause. It falls under
    "Anything else" (`02:880`).
  - A wrapper without `Pure` reaches `From<E>` with an unbounded `E`, which
    is not `Pure` (`02:804–805`), and is rejected at its definition
    (`02:885–888`). **Closed.**
- **Other fill points.** The `From` impls for `io::Error` in std are
  unchanged since round 40's list: `NulError`, `TryReserveError` and
  `ErrorKind` (`io/error.rs:111, 119, 529`), `IntoInnerError<W>`
  (`io/buffered/mod.rs:175`), and `TryLockError` (`fs.rs:533`). Only `new`
  and `other` take `Into<Box<dyn Error + Send + Sync>>`
  (`io/error.rs:580, 605`). Both box `error.into()` unchanged into `Custom`
  (`:608–610`). The string impls box a private `StringError(String)`
  (`alloc/src/boxed/convert.rs:578`), which matches "a private string
  error" (`02:875`).
- **Later mutation.** `downcast_mut::<T>` matches the stored `TypeId`
  exactly, so it yields only the admitted concrete type. Every admitted
  concrete type is `Pure`, so nothing written through that reference can
  hold a `Shared`.

This round found no safe route that leaves a non-`Pure` concrete payload in
an `io::Error`.

**Crate identity.** The new bullet names "alloc's four `From` impls"
(`03:394`). 02 says "by identity (ADMIT-1)" (`02:878`). Both key the four
impls to the `alloc` crate, so a user crate named `alloc` cannot supply
one. This agrees with "keyed on crate identity rather than crate name"
(`03:382`). The bullet is the only one in ADMIT-1's list that restricts an
impl only at certain call sites, since the reflexive impl stays admitted
everywhere else. ADMIT-1 covers this: the checker resolves each "impl
selected for an obligation" (`03:380`).

## New blocking problems

### B1. 02 and ADMIT-1 give the rejected conversion different codes

- **02.** 02 §9 says that any conversion other than the four "is
  `FX0517`, including a user `From` or `Into` impl and the reflexive
  `From<T> for T`" (`02:880–881`).
- **ADMIT-1.** The new bullet puts the same conversions in ADMIT-1's
  "Outside the list" list (`03:393–396`). That list's rule is "Anything
  else is `FX0520`" (`03:383`).
- **How the other bullets handle this.** The neighbouring bullets agree
  with 02 on their codes. `OnceLock` is "not admitted (ADMIT-1, `FX0520`)"
  (`02:849`, `03:391`), and `Waker` is `FX0520` (`03:400`).

A checker built from 03 reports route B as `FX0520`. One built from 02
reports it as `FX0517`. Both texts are new in this diff. 03 §3.1 defines
`FX0517` as the code for a failed `Pure` obligation that no rule claims
(`03:427–430`, `01:952–954`). That fits a non-`Pure` `E` that reaches
`From<E>`, and 02 owns the rule.

**Fix** (03 only, no core tokens). End the ADMIT-1 bullet with
"(`FX0517`, `02-memory.md` §9)" in place of "(`02-memory.md` §9)". This
records the bullet as the list's one exception to `FX0520`. The other way,
02 would say `FX0520` for an impl that is not one of the four and keep
`FX0517` for `From<E>` with a non-`Pure` `E`. That is also consistent, but
it splits one rule across two codes.

## Non-blocking notes

### N1. ADMIT-1 read literally also excludes core's blanket `Into`

ADMIT-1 excludes "any conversion ... other than alloc's four `From` impls"
(`03:393–394`). Every admitted call reaches those impls through
`impl<T, U: From<T>> Into<U> for T` (`core/src/convert/mod.rs:767`), which is
core's, not one of the four. 02's "must reach" (`02:877`) allows that step,
and ADMIT-1 cites 02, so the intent is clear. Round 40 proposed "through
core's blanket `Into`". **Fix** (optional, no core tokens): in ADMIT-1,
write "any conversion ... that does not reach, through core's blanket
`Into`, one of alloc's four `From` impls".

### N2. One reason given for the identity rule does not support it

02 says that "a bound on the input alone is not enough" for three reasons.
The third is that "an already-boxed `Box<dyn Error + Send + Sync>` can hold
any payload" (`02:881–885`). A `Pure` bound on the input already rejects
that box: by the `dyn` rule it is not `Pure` (`02:834`, `02:889`). It is
also outside the input list, because it is not `Error` (above). The first
reason is the one the identity rule needs: a user conversion turns a `Pure`
key into a non-`Pure` payload. The second matters only together with the
first. The third explains why the reflexive impl is named, not why a bound
is insufficient. The rule is unaffected. **Fix** (optional): drop the third
clause there. The exclusion of the reflexive impl (`02:881`) and the cost
sentence (`02:896–899`) already cover the case.

## Ready to merge

**No, but the remaining fix is one parenthesis.** Round 40's B1 is closed.
Route A fails because a user `From` or `Into` impl is not one of alloc's
four, whichever obligation selects it. Route B fails because the reflexive
impl is excluded by name and by identity. Round 40's N1 is closed. This
round tried a boxed `dyn Error` as `E`, a nested `io::Error`, `Arc`,
`&'static` references, `Box<P>`, and generic wrappers. Each either reaches
no admitted impl or reaches `From<E: Error + Send + Sync>`, where `E: Pure`
decides it as intended. The crate-identity keying is consistent. The only
contradiction left is the diagnostic code (B1): 02 says `FX0517`, and
ADMIT-1's list implies `FX0520`. N1 and N2 are optional wording fixes.

Validation: this was a read-only review of the working tree at `4a05cc7`,
with `.beads/issues.jsonl` and the five `design/` files modified and
`reviews/round36`–`round40` untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-cze (`br show`);
  - `reviews/round40-consistency.md` and `reviews/round33-consistency.md`;
  - 02 §9 (`02:790–902`);
  - 03 ADMIT-1 and §3.1 (`03:372–456`), and §3.3's argument (`03:512–518`);
  - reference 2.2 and 6.9;
  - 01 D7's admission bullet and the `FX0517` note (`01:419–430`,
    `01:948–956`).
- **Rust 1.96.0's rust-src:**
  - `alloc/src/boxed/convert.rs`: the eight `From` impls for
    `Box<dyn Error …>`, and `StringError`;
  - `alloc/src/boxed.rs:2394` and `alloc/src/sync.rs:4305`;
  - `core/src/error.rs:1100`;
  - `core/src/convert/mod.rs:767, 785`;
  - `std/src/io/error.rs`: `new`, `other`, `_new`, and the `From` impls;
  - a grep of `std/src` for every `From<…> for io::Error` and every
    `Into<Box<dyn` bound.
- **Witness:** compiled with
  `rustc +1.96.0 --edition=2024 --crate-type=lib --emit=metadata`, stdin
  closed, 60 s timeout, with `Mutex<Vec<u32>>` standing in for `Shared`.
  - `io::Error::other` accepts `&'static (dyn Error + Send + Sync)`,
    `Arc<P>`, `Arc<dyn Error + Send + Sync>`, `io::Error`, and `Box<P>`:
    exit 0, with only a dead-code warning.
  - `Box<dyn Error + Send + Sync>: Error` fails with E0277 (`Sized`).

  This proves only what rustc accepts. Tide's verdicts are deductions from
  the text, since no checker exists.

Only this file was written; nothing was staged or committed.

## Resolution

Applied before the Codex re-check (round 42, approve).

- **B1:** the ADMIT-1 bullet ends "(`FX0517`, `02-memory.md` §9)".
- **N1:** it says the conversion passes through core's blanket `Into`.
- **N2:** the unsupporting clause is dropped from 02 §9.

That was the second text-only restart, so no further Opus round is run.
