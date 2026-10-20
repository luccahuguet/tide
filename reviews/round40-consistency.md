# Tide consistency review, round 40: confirmation of the round-39 fix

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `4a05cc7`
("Require a Codex supervisor pass after clean Opus reviews"):
`git diff -- design/` (SHA-256 `c4b2394c…e678`), which changes `02-memory.md`
§9, `03-authority-concurrency.md` (ADMIT-1, §3.1, §3.3, the §3.5 table,
LOCK-4, SUSP-1), `04-contracts.md` §4.4, `reference.md` 2.2 and 6.6, and the
companion's core count. It implements bead fx-cze and applies round 39's
proposed fix. This is a narrow confirmation round. Scope, as asked:

- whether round 39's B1 is closed by 02 §9's admitted-conversion rule, the
  new ADMIT-1 bullet, 03 §3.3's premise, and reference 2.2, including any
  other route by which an `io::Error` ends up holding a non-`Pure` concrete
  payload through safe code;
- whether the generic-definition rule agrees with D4 and with impl
  selection;
- whether round 39's N1 fix in 04 is correct and consistent with LOCK-4;
- whether the companion's "Core count" line is present and well formed.

Nothing else was re-audited.

## Round-39 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1, the input list | **Resolved** | `02:873–876` admits only `E: Error + Send + Sync + Pure + 'static` or a `String`, `&str`, or `Cow<'_, str>`. Round 39's `Key` is not `Error`, so it is outside the list. Reference 2.2 agrees: "take only a `Pure` error or a string" (`reference.md:94–95`). 03 §3.3's premise now rests on this rule (`03:511–514`). |
| B1, the enforcement clause | **Open** (B1 below) | The clause that says how the list is enforced checks "the impl selected for their `E: Into<Box<dyn Error + Send + Sync>>` obligation" against "one of std's" (`02:876–878`). ADMIT-1 excludes only "a user impl" (`03:393–395`). Neither wording rejects two routes that rustc accepts. |
| B1, generic wrappers | **Resolved** | `02:880–883`. Agrees with D4 and with rustc's selection; see "Generic definitions" below. |
| B1, mutation after construction | **Resolved** for the listed inputs | `get_mut` yields `&mut (dyn Error + Send + Sync)`. `Error` has no `&mut self` method, an unsized place cannot be assigned or swapped, and `downcast_mut` yields only the concrete type, which the list makes `Pure`. `downcast` and `into_inner` return that same type or an erased box that is not `Pure` (`02:884`). |
| N1 | **Resolved** | `04:887–889`: "neither they nor their `Pure` payloads reach authority; they cannot call `with` or suspend". This matches LOCK-4 (`03:919–923`) and the §3.5 row (`03:550`). A grep for "call only authority-free" and "every call in" finds only CLAUSE-3 and its D-record and §3.5 summaries (`01:633`, `04:401`, `03:551`), which are about contract clauses. Those are out of scope here. |
| Core count | **Present and well formed** | `reference-companion.md:12–14`: "Core count: 24,996 tokens on 2026-10-20, measured by `scripts/count-tokens.sh design/reference.md`", under the 25,000 budget (`:10`). Not re-measured. |

**Std's fill points for `io::Error`.** Read in Rust 1.96.0's rust-src.
Outside the two admitted constructors, every public route stores no payload
or one std already built:

- `From<ErrorKind>`, `From<NulError>`, and `From<TryReserveError>` build a
  simple or static repr (`std/src/io/error.rs:111–128, 529`);
- `From<IntoInnerError<W>>` and `From<TryLockError>` return an `io::Error`
  they already hold (`io/buffered/mod.rs:175–179`, `fs.rs:533–540`);
- `from_raw_os_error` stores a code; `const_error!` and
  `from_static_message` are unstable (`-Zallow-features=`, ADMIT-2);
- std's internal `new` calls pass strings or plain errors such as
  `TryFromIntError` (`sys/fs/unix.rs:1699`).

`?` into `io::Result` needs a `From<X> for io::Error`. A user's impl of that
can only call these constructors, because `io::Error`'s fields are private.
`From<E: Error + Send + Sync>` boxes generic wrappers (`Box<T>`, `&T`) unchanged.
They are `Pure` only when `T` is. So the input list, enforced as written, closes
every route this round found. The hole is in how the enforcement is worded.

**Generic definitions.** The rule agrees with D4 ("Generics are
definition-checked", `01:347`) and with how rustc resolves the obligation:

- In `fn wrap<E: Error + Send + Sync + Pure + 'static>(e: E)`, the bound
  `E: Into<Box<…>>` resolves to core's blanket `Into` and then to alloc's
  `From<E: Error + Send + Sync>` (`alloc/src/boxed/convert.rs:557`). No
  other head unifies with a placeholder `E`.
- At an instance, coherence keeps that impl. A user's `From<K>` or
  `Into<…>` impl for a `K` that is `Error + Send + Sync` overlaps std's
  impls. rustc rejects that overlap with E0119, and D4 forbids overlapping
  impls. Definition and instance therefore select the same impl, so
  instantiation cannot add an error.
- A wrapper bounded only by `Into<Box<…>> + Pure` has its obligation met by
  a where-clause, not by an impl. That falls under "anything else"
  (`02:878`) at the definition, as round 39 asked.

## New blocking problems

### B1. The identity clause names the wrong obligation and admits std's reflexive `From`

`02:876–878` says that "the impl selected for their
`E: Into<Box<dyn Error + Send + Sync>>` obligation must be one of std's,
by identity (ADMIT-1), and anything else is `FX0517`". ADMIT-1 excludes "a
user impl selected for the conversion" (`03:393–395`). A checker built from
either sentence accepts two constructions that rebuild round 39's B1. Both
compile under `rustc 1.96.0 --edition=2024`, with `Arc<Mutex<Vec<u32>>>`
standing in for `Shared`:

```rust
// Route A: a user From impl. The impl selected for `Key: Into<…>` is
// core's blanket `impl<T, U: From<T>> Into<U> for T`, which is std's.
impl From<Key> for Box<dyn Error + Send + Sync> {
    fn from(k: Key) -> Self { Box::new(Payload { id: k.0, log: None }) }
}
let mut a = io::Error::other(Key(1));
a.get_mut().unwrap().downcast_mut::<Payload>().unwrap().log = Some(log);

// Route B: std's reflexive `impl<T> From<T> for T` on an erased box.
let b: Box<dyn Error + Send + Sync> =
    Box::new(Payload { id: 2, log: Some(log) });
let b = io::Error::new(io::ErrorKind::Other, b);
```

- **Route A, against 02.** `From` is the idiomatic way to write a
  conversion, and it is the one coherence permits. With a `From` impl, the
  impl selected for the `Into` obligation is always core's blanket impl,
  so the clause as worded passes it. The user's impl is selected only for
  the nested `Box<…>: From<Key>` obligation. ADMIT-1's "selected for the
  conversion" covers the nested impl, but 02's text and its example ("a
  user's `Into` impl", `02:878`) do not. 02 owns the rule.
- **Route B, against 02 and ADMIT-1.** The reflexive `From<T> for T` is
  std's own impl, so it passes 02's "one of std's". It is not "a user
  impl", so ADMIT-1 does not exclude it either. ADMIT-1's carve-out, "std's own
  conversions from a `Pure` error or a string stay", neither names it nor
  excludes it. `Box<dyn Error + Send + Sync>` is not `Error`
  (`impl<E: Error> Error for Box<E>` requires `Sized`,
  `alloc/src/boxed.rs:2394`). The input list therefore rejects it, and so
  does reference 2.2. The enforcement sentences admit it. The coercion to
  `dyn Error + Send + Sync` needs no `Pure` (`02:834`). Route B is the
  simplest way to put a `Shared` into an `io::Error`, and it needs no later
  mutation.

The input list and the clause that claims to enforce it disagree, and 03
§3.3's premise (`03:511–514`) holds only under the list. **Fix** (02 and
ADMIT-1, no core tokens):

- In 02, replace the colon clause with: "the conversion must reach, through
  core's blanket `Into`, one of alloc's four `From` impls for
  `Box<dyn Error + Send + Sync>` (from `E: Error + Send + Sync`, `String`,
  `&str`, `Cow<'_, str>`), by identity (ADMIT-1), with `E: Pure`. Any other
  impl is `FX0517`, including a user's `From` or `Into` impl and the
  reflexive `From<T> for T` that passes an already-erased box."
- Change the example's "a user's `Into` impl" to "a user's `From` or `Into`
  impl".
- In ADMIT-1, replace "a user impl selected for" with "any impl other than
  those four selected for". Then the reflexive impl is outside the list by
  name.

## Non-blocking notes

### N1. The cost of excluding the reflexive conversion is not stated

Once B1 is fixed, `io::Error::new(kind, boxed)` is rejected when `boxed` is
a `Box<dyn Error + Send + Sync>`. That rules out re-wrapping the result of
`into_inner()` and wrapping a box built by `?`. It also rules out
`Box<dyn PureErr>` upcast to `dyn Error + Send + Sync`. Rust code does this
often. 02's "The cost:" paragraph (`02:888–893`) lists only the formatting
costs. **Fix** (02, no core tokens): add "an already-erased
`Box<dyn Error + Send + Sync>` cannot be wrapped; pass the concrete error,
or `downcast` it first, or keep the `io::Error`". Optionally add a companion
§15 row under `FX0517`.

## Ready to merge

**No.** B1 of round 39 is closed in substance: the input list, reference
2.2, the generic-definition rule, and 03 §3.3 agree, and every std fill
point for `io::Error` this round found stores only a `Pure` payload under
that list. The clause that enforces the list checks the top-level `Into`
impl against "std's". As worded, it admits a user's `From` impl (route A)
and std's reflexive `From<T> for T` (route B). Both rebuild round 39's
construction in code that rustc accepts. The fix is two sentences in 02 and
ADMIT-1, at no core cost. N1 states the cost the fix makes visible. Round
39's N1 is resolved, and the core-count line is in order.

Validation: this was a read-only review of the working tree at `4a05cc7`,
with `.beads/issues.jsonl` and the five `design/` files modified and
`reviews/round36`–`round39` untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-cze (`br show`);
  - `reviews/round39-codex.md` and `reviews/round33-consistency.md`;
  - 02 §9;
  - in 03: ADMIT-1, §3.1, §3.3, LOCK-4, and SUSP-1;
  - 04 §4.4;
  - reference 2.2;
  - the companion header;
  - 01 D4.
- **Rust 1.96.0's rust-src:**
  - `std/src/io/error.rs`: the constructors, `get_mut`, `into_inner`,
    `downcast`, and every `From` impl for `io::Error`;
  - `alloc/src/boxed/convert.rs` and `alloc/src/boxed.rs`;
  - `std/src/io/buffered/mod.rs` and `std/src/fs.rs`;
  - a grep of `std/src` for internal `Error::new` and `Error::other` calls.
- **Witness:** routes A and B, plus a generic wrapper, compiled with
  `rustc +1.96.0 --edition=2024 --crate-type=lib --emit=metadata`, stdin
  closed, 60 s timeout. Exit 0, no diagnostics. This proves only that rustc
  accepts them. That Tide accepts them is a deduction from the wording.
  No checker exists.

Only this file was written; nothing was staged or committed.

## Resolution

Applied before round 41.

- **B1:** 02 §9 and ADMIT-1 require the conversion to reach one of alloc's
  four `From` impls by identity, and exclude user `From`/`Into` impls and
  the reflexive `From<T> for T` by name.
- **N1:** 02 §9's cost paragraph says an already-boxed error cannot be
  wrapped again.
