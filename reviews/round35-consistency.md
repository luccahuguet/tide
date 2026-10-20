# Tide consistency review, round 35: the response to round 34

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ce2bd3`
("File follow-up beads from rounds 32-33: fx-u18, fx-obr, fx-t99"):
`git diff -- design/`, which changes `02-memory.md` §8.1 and §9,
`03-authority-concurrency.md` ADMIT-1 and §3.3, `reference.md` 6.4, 6.6 and
6.9, and `reference-companion.md` 14.1 and its core count. It implements
bead fx-u18 under the owner's revised decision of 2026-10-20:

- (b) stands: `OnceLock` and `LazyLock` are rejected by ADMIT-1
  (`FX0520`), and `OnceCell` and `LazyCell` stay admitted and `Pure`;
- the clause "a user-written `unsafe impl Sync` makes a type a
  synchronization handle" is dropped. 03 §3.3 instead argues structurally
  that a user `Sync` type with interior mutability needs `unsafe impl Sync`,
  a trust hole under the `unsafe` grant;
- no change to D7 or D9.

Scope, as asked:

- whether round 34's B1, B2, N1, N3, N4, N5 and N7's `LazyCell` row are
  closed; whether N2 is moot; whether deferring N6 and N7's v1 point is
  right;
- whether 02 §9 and §8.1, 03 ADMIT-1, §3.1 and §3.3, reference 6.4, 6.6 and
  6.9, and companion 14.1 agree with each other and with 01 D7 and D9;
- whether the rewritten 03 §3.3 argument is sound;
- whether the companion's "Core count" line is present and well formed.

## Round-34 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | 6.4's `FX0502` list now reads "a `static` holding atomics, `Mutex`, or `Cell`" (`reference.md:431`). Companion row 35 reads "a local or `static` `OnceLock` or `LazyLock`", with `FX0520` (`reference-companion.md:35`). Row 36 no longer names `OnceLock` (`reference-companion.md:36`). 03's replacement table gives no code (`03:279`). A grep finds no other text that gives a `static` `OnceLock` `FX0502`. A `static` `OnceLock` now has one derivable code, `FX0520`. |
| B2 | **Superseded** by the owner's revised decision, and closed by it | The user-`unsafe impl Sync` item is gone from 02 (`02:825–828`), 03 §3.1 (`03:437–438`) and 6.6 (`reference.md:471–472`). 02's sentence again reads "Creating a synchronization handle takes `&Tasks`" (`02:850–852`). D7's definition by creation through `&Tasks` (`01:450–454`) is therefore true again. The same holds for 03 §0, §1.2 (`03:166`), §3.2 (`03:477–479`) and reference 7.4. 03 §3.3 no longer rests on the retired rule (`03:502–513`). Round 34's other fix items (a separate not-`Pure` item; adding it to the `Pure + Sync` arguments) are moot. |
| N1 | **Resolved** | 02 §8.1: "`unsafe impl Send` and `unsafe impl Sync` are allowed in packages granted `unsafe`, and each one is a trust hole" (`02:623–624`). This matches reference 11 (`reference.md:955`). 03 §3.3 ends "an unsound one breaks this argument as an unsound `unsafe impl Send` does" (`03:512–513`), which names the `Send` hole. |
| N2 | **Moot**; agreed | No rule names a user-written `unsafe impl Sync` any more, so neither the partial-impl reading nor the macro-output reading arises. |
| N3 | **Resolved**, in 02 rather than 03 | `#[trusted_pure]` "claims what `Pure` guarantees below, including that no operation on the type depends on other tasks" (`02:795–796`). That restates the obligation std carries for `Pure` types (`02:854–855`), and it covers a sync-mutating cell: any read of it would return a result that depends on other tasks. 02 owns the table, so this is the right place. 03 §3.1's "authority-free in fact" (`03:422–423`) does not conflict with it. |
| N4 | **Resolved** | "Build the value before the parallel region; between spawned tasks, use a `Shared<Option<T>>`" (`02:849–850`). Companion row 35 agrees (`reference-companion.md:35`). |
| N5 | **Resolved** | ADMIT-1: "Tide's std does not export them, and the driver reports a path to Rust's as `FX0520`, not as rustc's unresolved path; `OnceCell` and `LazyCell` stay" (`03:389–392`). 6.9 lists them under `FX0520` (`reference.md:555`). |
| N6 | **Deferred**; agreed, but it has no durable home | It concerns v1's crate import (RUST-2, RUST-4), which the bead excludes. `br list --all` shows no bead for it, and the `.beads/issues.jsonl` diff changes only fx-u18. See N3 below. |
| N7, the v1 items in 6.6 | **Deferred**; agreed | `extern` types, foreign function pointers and `ThreadBound` are v1 (FFI §5, RUST-4). Read as a v0 reference, 6.6 names the same set as 02 and 03 §3.1. |
| N7, `LazyCell` | **Resolved** as applied | 02 §8.1 adds "`LazyCell<T, F>` \| if `T, F: Send` \| no" (`02:614`), which is Rust's derivation through `UnsafeCell`. 03 §3.3 names `LazyCell` among the types that are not `Sync` (`03:507`). The `Sync`-excludes lists in 02 (`02:866–867`) and 03 (`03:466–467`) still omit it. Those lists are illustrative, so this is acceptable. |
| N7, `Arc` rationale | Accepted in round 34 | `reference.md:477–478`. |

## Scope answers

| Question | Answer | Evidence |
| --- | --- | --- |
| Same not-`Pure` set | **Yes.** For v0, 02 (`02:816–835`), 03 §3.1 (`03:435–442`) and 6.6 (`reference.md:469–473`) list the same items. Each list names `std::sync::raw` among the synchronization handles. | diff |
| Same outcome for the four cells | **Yes.** `OnceLock` and `LazyLock` are `FX0520` whether local or `static` (`02:846–847`, `03:389–392`, `reference.md:555`, `reference-companion.md:35`). `OnceCell` and `LazyCell` are `Pure` (`02:838`), not `Sync` (`02:613–614`), and admitted (`03:392`). | diff |
| Agreement with D7 | **Yes.** D7's admission list is open-ended (`01:419–430`). Its definition of synchronization handles is true again (B2). Its parallel bullet, "`Pure` excludes atomics and synchronization handles, and `Sync` excludes `Cell` and `RefCell`" (`01:479–482`), holds because `OnceLock` and `LazyLock` are no longer admitted. | — |
| Agreement with D9 | **Yes.** D9's bounds (`01:567–576`) are unchanged and name no cell type. | — |
| 6.4 and GLOBAL-1 | **Yes.** `static` atomics, `Mutex` and `Cell` each fail `Sync + Pure` (`03:245–246`). | — |
| Core count line | **Present and well formed.** "Core count: 24,969 tokens on 2026-10-20, measured by `scripts/count-tokens.sh design/reference.md` (CI counting is planned)" (`reference-companion.md:12–14`). The figure is under the budget and below HEAD's 24,997, which fits the diff's net cut in 6.4 and 6.6. It was not rerun, as instructed. | — |

**Search for a counterexample to 03 §3.3.** The target was a `Pure + Sync`
value, reachable through safe code or a sound `unsafe impl Sync`, whose
state one `join` or `par_map` branch can observe another branch changing.

- **Std's interior-mutable `Sync` types.** Each one is listed as not `Pure`
  (`Shared`, channel ends, atomics, `std::sync::raw`), or rejected
  (`OnceLock`, `LazyLock`, `core::sync::atomic`), or absent from Tide's std
  (`Once`, `Condvar`, `Barrier`, `std::sync::Mutex`). `SyncUnsafeCell` and
  `Exclusive` are unstable and blocked by `-Zallow-features=` (`03:407`).
  No counterexample here.
- **Composite user types.** A struct that wraps a Tide atomic or a `Shared`
  is `Sync` with no `unsafe impl`, and the deep rule makes it not `Pure`.
  No counterexample, but the argument omits this case (N2).
- **Sound `unsafe impl Sync` with run-time synchronization.** The
  synchronizing field must be an atomic, a lock, a raw pointer or a foreign
  handle. The deep rule sees each of these, and a `static` cannot supply
  one (GLOBAL-1, `FX0502`). No counterexample.
- **Sound `unsafe impl Sync` with static synchronization.** A
  token-branded cell (GhostCell) is a sound `Pure + Sync` value mutated
  through `&self` while other branches hold `&` to it. It is not a
  counterexample: a reader needs `&GhostToken`, which conflicts with the
  writer's `&mut GhostToken`. It does falsify the argument's wording (N2).
- **Std types that erase a `Send + Sync` payload.** `io::Error::new` and
  `core::task::Waker` give conditional counterexamples. Their `Pure` status
  and admission are undecided, and the conditions they need are not
  excluded by the text (N1). This gap predates fx-u18.

## New blocking problems

None.

## Non-blocking notes

### N1. Std types that erase a `Send + Sync` payload have no `Pure` status

- **The construction.** It needs `io::Error::new` (or `io::Error::other`)
  admitted with Rust's signature, `E: Into<Box<dyn Error + Send + Sync>>`.
  It also needs `io::Error` to be `Pure`, the default reading for a std
  error type, since 02 §9 does not list it.
  - The parent, which holds `&Tasks`, builds a log `Shared<Vec<u32>>` and
    two errors, `io::Error::other(Tag { id: 1, log: log.clone() })` and
    `io::Error::other(Tag { id: 2, log })`.
  - `Tag`'s `Display::fmt` runs `self.log.with(|v| v.push(self.id))`.
  - `join(|| e1.to_string(), || e2.to_string())` captures only `&io::Error`.
    The closures are `Pure + Send`, and the shared captures are
    `Pure + Sync` (std error types are `Send + Sync`, `reference.md:92–94`).
  - `io::Error`'s `Display` delegates to the erased payload. After the
    region, the log reads `[1, 2]` or `[2, 1]`, depending on the schedule.
    If `fmt` reads the log, a branch's own result depends on the other
    branch. Each branch also runs a lock operation, which can block, in
    code classified as authority-free.
- **A second route.** `core::task::Waker` is `Send + Sync`. Safe code builds
  one with `Waker::from(Arc<W>)` for any `W: alloc::task::Wake`, and `W`
  may hold a `Sender`. `wake_by_ref(&self)` runs `W`'s code. Nothing in
  ADMIT-1, D8 or reference 7 names `core::task` or `alloc::task`; D8 and
  reference 7 exclude only `async`, `Future`, `Pin` and executors
  (`reference.md:576–578`).
- **Why the text does not decide it.** The deep rule "looks through every
  field, private ones included" (`02:798`). Applied to std's own fields, it
  would make `Vec` not `Pure` through its raw pointer, so std's types must
  take their status from the table and from std's own declarations. The
  table names neither type. The obligation "No operation on a `Pure` type
  may return a result that depends on other tasks" (`02:854–855`) implies
  that std must not make `io::Error` `Pure` while its payload is
  unbounded. No text states the resulting decision.
- **Status.** This predates the diff and is outside the bead ("other
  `Pure` questions"). 03 §3.3's premise covers std's interior-mutable types
  but not std's erasing types. Its first clause, "`Pure` arguments reach no
  capability or handle" (`03:502–503`), fails here if `io::Error` is `Pure`.

**Fix**: none in this bead. File a bead, owner decision, to:

- bound the payload of `io::Error::new` and `io::Error::other` by `Pure`,
  or classify `io::Error` as not `Pure`. The bound is cheaper, since
  `io::Error` appears in most `Result`s;
- add `core::task` and `alloc::task::Wake` to ADMIT-1's "at least" list,
  under D8;
- state in 02 §9 that std's types take their `Pure` status from the table
  and their type arguments, not from a walk of std's private fields;
- audit std's other erasing types against the obligation at `02:854`.

### N2. 03 §3.3's "outside std" premises are stronger than its conclusion needs

- **The first premise.** "Outside std, a type is `Sync` over interior
  mutability only by `unsafe impl Sync`" (`03:508–509`). A user
  `struct Counter { n: AtomicU64 }` is `Sync` through its field, with no
  `unsafe impl`. Depth makes it not `Pure` (`02:798–799`), but the argument
  does not say so.
- **The second premise.** "a sound one synchronizes through atomics,
  `std::sync::raw`, raw pointers, or foreign calls" (`03:510–511`).
  - A GhostCell-style cell is a sound `unsafe impl Sync` over `UnsafeCell`
    with no run-time synchronization: writes need `&mut GhostToken`, and
    reads need `&GhostToken`.
  - Its fields, `UnsafeCell<T>` and a lifetime brand, are `Pure`. Its
    package needs the `unsafe` grant only for `UnsafeCell::get`.
  - Determinism survives, because the borrow checker keeps every reader
    out while a writer exists. But the memory is "mutated concurrently"
    with another branch's `&` to it (`03:504`). The enumeration is
    therefore incomplete.
- **Two wording points.**
  - "is a synchronization handle and therefore not `Pure` or is not
    admitted" (`03:505–506`) parses as "not (`Pure` or admitted)". A comma
    before "or is not admitted" would fix it.
  - Line 513 is a short line left by the edit.

**Fix** (03 only, no core tokens): rewrite the outside-std sentence as:

> Outside std, a `Sync` type that is mutable through `&` either has such a
> std type as a field, and so is not `Pure` (the rule is deep), or reaches
> an `UnsafeCell` through `unsafe impl Sync`, a trust hole under the
> `unsafe` grant (`02-memory.md` §8.1). A sound one either keeps readers
> and writers apart statically, as a token-branded cell does through
> `&mut`, or synchronizes at run time through atomics, `std::sync::raw`,
> raw pointers, or foreign calls, none of which is `Pure`.

The rest of the paragraph can stay as it is, then be reflowed.

### N3. Two deferred findings have no durable home

- **Round 34's N6** (v1 imported crates under RUST-2 and RUST-4). Round 34
  recommended a v1 bead. None exists: `br list --all` shows only fx-u18,
  fx-zm4, fx-qzp and fx-t99 near this topic.
- **N1 above** needs an owner decision.

AP-PLAN-001 allows reporting instead of filing. Without a bead, the next
round on `Pure` will find both again.

**Fix**: report both to the owner. If approved, file one bead for each.
They touch different chapters and releases, so they should stay separate.

## Ready to merge

**Yes.** Round 34's two blocking items are closed:

- **B1:** a `static` `OnceLock` now has one derivable code, `FX0520`.
- **B2:** the revised decision removes the clause that broke D7's
  definition, and D7 and D9 now agree with every edited text.

N1, N3, N4, N5 and N7's `LazyCell` row are applied as asked. N2 is moot.
The deferral of N6 and N7's v1 point is right.

The four texts name one not-`Pure` set and one outcome for the four cell
types. The rewritten §3.3 argument holds for interior mutability: no
`Pure + Sync` counterexample was found through safe code or a sound
`unsafe impl Sync`. Its outside-std wording should be tightened before
merge (N2, 03 only). The erasing-type gap (N1) predates the bead, and it
and round 34's N6 need beads (N3).

Validation: this was a read-only review of the working tree at `5ce2bd3`,
on 2026-10-20. The tree had `.beads/issues.jsonl` and four `design/` files
modified, and `reviews/round34-consistency.md` untracked.

- **Read:**
  - the full `design/` diff and the `.beads/issues.jsonl` diff;
  - fx-u18 with both owner comments;
  - `reviews/round33-consistency.md` and `reviews/round34-consistency.md`;
  - in 02: §8.1, §8.5, §9, and R4–R5;
  - in 03: §1.2, §2.2, §2.5, §3.1 to §3.4;
  - 01 D7 and D9;
  - reference 2.2, 6.4, 6.6, 6.9, 7, 11, and the module table;
  - companion lines 1–40.
- **Greps** over `design/` for `OnceLock`, `LazyLock`, `OnceCell`,
  `LazyCell`, `sync::raw`, `unsafe impl Sync`, `trusted_pure`, `io::Error`,
  `Waker`, `core::task`, `async` and `Future`.
- **Not done:** `scripts/count-tokens.sh` was not run, as instructed.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before commit.

- **N2:** 03 §3.3's outside-std sentences take the wording proposed above,
  with the comma fix and the paragraph reflowed.
- **N1, N3:** reported to the owner (erased `Send + Sync` payloads in std
  types such as `io::Error` and `Waker`; round 34's N6).

The verdict is approve with notes, so no further round is run.
