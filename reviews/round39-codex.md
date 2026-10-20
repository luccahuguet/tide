# Tide AI supervisor review, round 39: std payload erasure and `Pure` (fx-cze)

Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**request_changes — high confidence.**

Keeping `io::Error` `Pure` and excluding the async task types is an acceptable
direction. The proposed constructor obligation does not establish the
invariant it relies on: a `Pure` conversion input can produce an erased value
whose concrete type is not `Pure`. Safe mutation can subsequently put a
`Shared` handle into that value. This recreates the round-35 determinism hole.

Reviewed `git diff -- design/` against HEAD
`4a05cc7aa3ecd16f0520b9b4194eed45b1a6d2e9`. The diff's SHA-256 is
`9f2ef39c6dba3edfb8eb2304f8335c763e79ba2bb645f5f4f6f74f1d6f3f6520`.
The required model pass and the primary session's independent corroboration
agree on this verdict. No override was requested or recorded.

## Blocking findings

### B1. P1 — `E: Pure` constrains the conversion input, not the boxed concrete type

**Locations:** `design/02-memory.md:871–875`; the resulting guarantee at
`design/03-authority-concurrency.md:508–512` and
`design/reference.md:92–95`.

Rust's constructors call `error.into()`. `E` need not implement `Error`, and
the conversion need not box an `E`. A user can therefore write:

```rust
struct Key(u32); // Pure

struct Payload {
    id: u32,
    log: Option<Shared<Vec<u32>>>,
}

impl Into<Box<dyn Error + Send + Sync>> for Key {
    fn into(self) -> Box<dyn Error + Send + Sync> {
        Box::new(Payload { id: self.0, log: None })
    }
}
```

Give `Payload` an `Error` impl, a `Debug` impl that writes a fixed string, and
a `Display::fmt` that copies `self.id` into a local, then calls
`log.with(|xs| xs.push(id))` if `log` is `Some`. The `with` closure captures
only that `u32`; it need not capture the formatter or the payload.

`Payload` is structurally not `Pure` even when `log` is `None`, but building
that empty value requires no authority. Both constructors accept `Key` under
the proposed obligation. A caller already holding a shared log can then do:

```rust
let mut a = io::Error::other(Key(1));
let mut b = io::Error::new(io::ErrorKind::Other, Key(2));
a.get_mut().unwrap().downcast_mut::<Payload>().unwrap().log = Some(log.clone());
b.get_mut().unwrap().downcast_mut::<Payload>().unwrap().log = Some(log);
join(|| a.to_string(), || b.to_string());
```

These are safe Rust APIs. `get_mut` exposes the boxed error mutably, and
`downcast_mut` requires `Error + 'static`, with no `Pure` restriction.
The signatures are documented in Rust's
[`io::Error` API](https://doc.rust-lang.org/std/io/struct.Error.html#method.get_mut)
and [`Error` API](https://doc.rust-lang.org/std/error/trait.Error.html#method.downcast_mut-2),
and were checked against the installed Rust 1.96.0 source.

The two errors still have the supposedly `Pure + Send + Sync` type
`io::Error`. Consequently both `join` closures pass the stated bounds, but
their formatting acquires a shared cell and leaves `[1, 2]` or `[2, 1]`
according to scheduling. This violates D7's reachable-authority invariant
(`01-design.md:455–468`), D9's sequential equivalence
(`01-design.md:567–579`), and the no-suspension claim in the amended SUSP-1.

The earlier reviews' argument that a conversion with `Pure` input cannot
*create a handle* is insufficient. It can create an empty container whose
type permits a handle, and the mutable accessor permits insertion later.

**Smallest fix:** retain `E: Pure`, but admit these constructor conversions
only when the boxed concrete type is established to be structurally `Pure`.
For v0, accept the audited standard conversion from
`E: Error + Send + Sync + Pure + 'static`, which boxes `E` unchanged, and
the standard `String`, `&str`, and `Cow<'_, str>` conversions, which box
`StringError(String)`. Reject arbitrary custom `Into`/`From` conversions at
these constructors with `FX0517`; callers can pass their concrete `Pure`
error instead. Identify the accepted impls by identity, as ADMIT-1 already
requires. Rust's
[boxed error conversions](https://doc.rust-lang.org/src/alloc/boxed/convert.rs.html)
provide these standard cases.

Enforce this during generic definition checking: an
`E: Into<Box<dyn Error + Send + Sync>> + Pure` bound alone is insufficient.
A generic wrapper can state `Error + Send + Sync + Pure + 'static` instead.
Deferring rejection until instantiation would introduce a separate D4
violation.

## Non-blocking improvements

### N1. Chapter 04 retains the stronger formatting claim removed from chapter 03

`design/04-contracts.md:887–889` still says that `Pure` closures "call only
authority-free functions". Under the explicit `fmt::Arguments` verdict,
`cell.with(|v| v.push(format!("{n}")))` has `Pure` captures but calls
`alloc::fmt::format(Arguments)`, whose parameter type is not `Pure`.
Chapter 03 correctly replaces that inference in its table and LOCK-4
(`03:547`, `03:916–920`).

Apply the same wording to chapter 04: the closure and payload reach no
authority, so they cannot acquire another cell or suspend. This is a
pre-existing explanatory overstatement in the v1 verification discussion,
not a second independent safety blocker introduced by this diff.

## Alternative designs

- Require an erased output with a companion supertrait,
  `PureError: Error + Pure`. This expresses the desired invariant directly,
  but integrating it with Rust's inherent constructors requires more
  conversion/API work than restricting their admitted conversions. It must
  preserve ordinary string errors explicitly.
- Making `io::Error` not `Pure` closes the hole, but propagates the restriction
  into `io::Result` and containing error enums. The bead reasonably rejects
  that cost. B1 does not require reversing that decision.

The standard-conversion restriction is the recommended minimal repair.
An analysis that merely proves a conversion creates no authority on this
execution does not establish the invariant under subsequent safe mutation.

## Minimal patch for approval

1. In 02 §9 and ADMIT-1, require the converted concrete payload type to be
   `Pure`, with the standard-conversion admission rule and generic checking
   behavior above. Keep the existing async exclusions and std field walk.
2. Align reference 2.2 with the admitted input forms. The core has only six
   recorded tokens of headroom, so pay for any addition with a cut and
   recount if it changes.
3. Check that B1 is rejected at construction, including a generic wrapper
   bounded only by `Into<Erased> + Pure`. Preserve acceptance of concrete
   `Pure` errors, string inputs, and ordinary `io::Result` use. The eventual
   checker regression must cover the mutation step, not just direct
   construction from a type already holding a handle.

N1 is a small accompanying clarification. After changes, follow governance's
narrow Opus confirmation and Codex re-check; this report does not approve a
future patched revision.

## Reasoning

The consumer is the checker and code relying on its `Pure` bounds. At an
admitted std constructor or a `Pure` obligation, the observable contract is
that the resulting value cannot later expose authority through safe
operations. The cheapest falsifier here is a conversion followed by safe
mutation, then formatting in `join`.

I read AGENTS.md, the supervisor merge rule, fx-cze and its decision comment,
the design diff, round 35 N1, and rounds 36–38. The independent supervisor
pass used `gpt-6.1-sol` at `xhigh`, as governance requires for this
spec-shaping change. The primary session separately inspected the rules and
verified the reported construction.

The other changes withstand the cases examined:

| Surface | Assessment |
| --- | --- |
| Round-35 direct `io::Error::other(Tag { log: Shared, .. })` | Rejected by `E: Pure`; B1 bypasses that check through conversion and later mutation. |
| `Waker`, `Context`, `Wake` | The `core::task` and `alloc::task` exclusion closes the wake callback route. The usual `std::task` paths receive the specified diagnostic. |
| `Box`/`Rc` trait objects and `dyn Any` | The existing companion-supertrait rule rejects unbounded erased payloads. `Arc` remains excluded regardless of payload. |
| `fmt::Arguments`, `Formatter`, and debug builders holding a formatter | Correctly not `Pure`; their inability to cross some Rust thread boundaries alone would not protect `Pure`-only lock closures. |
| `Cow` and receiver iterators | Restoring the field walk follows `B::Owned` and the held `Receiver`; the projection rule requires a separate bound when normalization is unavailable. |
| Owning/borrowing std pointers | The semantic pointee exception permits ordinary containers, slice iterators, `String::drain`, and `panic::Location`; it does not override the explicit synchronization-handle exclusions. |
| Panic payloads and listed Tide errors | Unbounded erased panic data follows the `dyn` rule; `Panicked` carries a `String`. The reference's `Pure` promise constrains the eventual definitions of the listed error types. |
| Formatting and SUSP-1 | Locally erasing actually `Pure` values does not introduce authority. Leaving the v1 native-to-frame lowering open is consistent with D18's existing open question. B1 invalidates the claimed premise for `io::Error`. |

Rust evidence was checked against local `rustc 1.96.0
(ac68faa20 2026-05-25)` and its installed rust-src: `std/src/io/error.rs`
(constructors and mutable accessor), `core/src/error.rs` (downcasting),
`alloc/src/boxed/convert.rs` (standard boxed conversions),
`std/src/io/error/repr_bitpacked.rs`, and the relevant formatting and
collection layouts. Current official documentation corroborates the API
signatures; the installed source fixes the version used for the witness.

The primary session compiled a Rust witness with `Arc<Mutex<Vec<u32>>>`
standing in for Tide's `Shared<Vec<u32>>`. It included the custom conversion,
both constructor forms, mutable downcasting and insertion, and scoped
threads formatting the two errors. Command: `rustc --edition=2024
--crate-type=lib --emit=mir -o - -`, supplied through stdin by a Python
subprocess with a 30-second timeout and MIR discarded. Result: exit 0,
no diagnostics. This proves Rust permits the conversion and mutation; Tide
acceptance and the scheduling consequence are deductions from the design,
not an executable Tide test.

Not checked: executable Tide behavior (no checker exists), a new reference
token count, or v1 imported-crate enforcement. Only this review file was
written; the design, beads, and earlier reviews were left unchanged.

## Resolution

Applied, then confirmed by Opus rounds 40 and 41 and the Codex re-check
round 42 (approve).

- **B1:** 02 §9 admits only alloc's four `From` impls for
  `Box<dyn Error + Send + Sync>` (from a `Pure` error, `String`, `&str`,
  `Cow<str>`), by identity; generic wrappers bound by
  `Error + Send + Sync + Pure + 'static`, checked at definition. ADMIT-1 and
  03 §3.3 agree; reference 2.2 says "a `Pure` error or a string".
- **N1:** 04 now says the closures and payloads reach no authority.
