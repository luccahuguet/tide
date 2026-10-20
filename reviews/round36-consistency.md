# Tide consistency review, round 36: std types that erase a payload (fx-cze)

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `04302c9`
("Add the Decisions rule: agents make technical design decisions"):
`git diff -- design/`, which changes `02-memory.md` §9,
`03-authority-concurrency.md` ADMIT-1 and §3.3, `reference.md` 2.2 and
6.6, and the companion's core count. It implements bead fx-cze under the
agent decision recorded on the bead on 2026-10-20:

- (a): `io::Error::new` and `io::Error::other` take
  `E: Into<Box<dyn Error + Send + Sync>> + Pure`, and `io::Error` stays
  `Pure`; (b), making `io::Error` not `Pure`, is rejected;
- `core::task` and `alloc::task` (`Waker`, `Context`, `Wake`) go outside
  ADMIT-1 under D8;
- 02 §9 states that std's types take their `Pure` status from the table
  and their type arguments, not from a walk of std's private fields;
- the bead comment records an audit of std's erasing types.

Scope, as asked:

- whether round 35's N1 is closed, and whether a new construction of the
  same shape exists through any std, `core` or `alloc` type;
- whether the new std rule (`02:863–864`) is well defined and consistent
  with the deep rule (`02:796–804`), 03 §3.1 and 01 D7;
- whether every error type that reference 2.2 names is `Pure`;
- whether 02 §9, 03 ADMIT-1 and §3.3, and reference 2.2, 6.6 and 6.9
  agree, and whether 6.9's omission of `core::task` misleads;
- whether the companion's "Core count" line is present and well formed.

The `AGENTS.md` and `.agent-protocols.*` changes were ignored, as
instructed.

## Round-35 N1 status

| Point of round 35 N1 | Status | Evidence |
| --- | --- | --- |
| The construction: `io::Error::other(Tag { id, log: Shared })`, then `join(\|\| e1.to_string(), \|\| e2.to_string())` | **Closed** | `Tag` reaches a `Shared`, so it fails the new `E: Pure` bound (`02:865–866`, `reference.md:94–95`). The other ways into `io::Error` in Rust 1.96's std carry no user payload: `From<ErrorKind>`, `From<NulError>`, `From<TryReserveError>`, `From<fs::TryLockError>` and `From<IntoInnerError<W>>` (which keeps only the inner `io::Error`), plus `from_raw_os_error`. `const_error!` and `from_static_message` are unstable, so `-Zallow-features=` blocks them (`03:409`). A user `impl From<Key> for Box<dyn Error + Send + Sync>` passes the orphan rule (checked with rustc 1.96), but its body has only a `Pure` argument, so it cannot make a handle (`03:504–507`). `downcast`, `get_ref` and `get_mut` give back the bounded payload or a `dyn` that is not `Pure`. The bound's mechanism and diagnostic are unstated (N1). |
| `io::Error`'s status was undecided | **Decided**: `Pure` | `02:863–866`; reference 2.2 (`reference.md:92–95`). |
| Second route: `Waker::from(Arc<W>)` with `W: Wake` holding a `Sender` | **Closed** | ADMIT-1 rejects `core::task` and `alloc::task` (`03:393–394`), which covers the `From<Arc<W>>` impl as "an impl selected for an obligation" (`03:379–381`). `Waker::noop` and `Context` go with them. 02 agrees (`02:871`). |
| "Why the text does not decide it": the deep walk would make `Vec` not `Pure`, so std's status must come from the table | **Answered, but the answer is under-defined** | The new rule drops the walk and puts nothing in its place for what the walk found. Two new constructions get through (B1, B2). |
| 03 §3.3's "`Pure` arguments reach no capability or handle" | **Amended, but still false** | "since std bounds every payload it erases by `Pure`" (`03:505–506`) is contradicted by `fmt::Arguments` and `fmt::Formatter` (B1), and the rule lets `mpsc::IntoIter` and `Cow` through (B2). |
| Fix: add `core::task` and `alloc::task::Wake` to ADMIT-1 | **Done** | `03:393–394`. 6.9 leaves it out on purpose (N3). |
| Fix: one sentence on std's `Pure` status in 02 §9 | **Done**, see B2 | `02:863–864`. |
| Fix: audit std's other erasing types | **Partly done** | `02:868–871` and the bead comment. They cover `Box<dyn Trait>`, `dyn Any`, `Box<dyn Error + Send + Sync>`, `Waker`, `Panicked` and the error enums. The `fmt` types' verdict is wrong (B1), and the audit misses the std types the walk used to catch (B2). |

## Scope answers

| Question | Answer | Evidence |
| --- | --- | --- |
| Is round 35 N1 closed? | **Its two routes are closed.** Two new constructions of the same shape exist. | B1, B2 |
| Is the std rule well defined and consistent with the deep rule, 03 §3.1 and D7? | **No.** It leaves out associated types and held handles, does not say whether "std" includes `core` and `alloc`, and goes against three unamended texts. | B2 |
| Are all std error types `Pure`? | **Yes, for the ones defined; the claim cannot be checked for three.** `TaskError` and `LockError` are fieldless enums (`03:753`, `03:896`). `Panicked` holds a `String` (`03:754`). `NetError` holds data and `io::Error` (`03:135–137`), so it is `Pure` now that `io::Error` is. `Cancelled`, `UrlError` and `json::Error` have no definition in `design/`, so for them 2.2 is a declaration. | N4 |
| Do 02 §9, 03 ADMIT-1 and §3.3, and reference 2.2, 6.6 and 6.9 agree? | **On `io::Error` and `Waker`, yes.** All of them carry over B1 and B2, and 03 §3.3 states the false premise outright. 6.6 says nothing on std's own types, which is acceptable inside the core budget. | `02:863–871`, `03:393–394`, `03:504–506`, `reference.md:92–95`, `reference.md:470–479` |
| Does 6.9's omission of `core::task` mislead? | **No, with one diagnostic gap.** 6.9's list is "notably" (`reference.md:554`). Reference 7 already rules out `Future`, `Pin` and executors (`reference.md:576–578`), so no v0 program reaches for a `Waker`. The gap: Rust's usual path, `std::task::Waker`, lands in Tide's own `std::task` (`reference.md:1006`) and fails as rustc's unresolved import, not as `FX0520`. | N3 |
| Core count line | **Present and well formed.** "Core count: 24,994 tokens on 2026-10-20, measured by `scripts/count-tokens.sh design/reference.md` (CI counting is planned)" (`reference-companion.md:12–14`). That is 6 tokens under D14's 25,000. The rise from 24,969 fits 2.2's added clause less 6.6's cut. It was not rerun, as instructed. Every fix below fits outside the core. | — |

## New blocking problems

### B1. `fmt::Arguments` and `fmt::Formatter` erase unbounded payloads and meet `Pure`-only bounds

- **The claim.** "`fmt::Arguments` is neither `Send` nor `Sync`"
  (`02:870–871`) is offered as the reason it needs no bound. Both facts hold
  in rustc 1.96 for `Arguments<'_>` and `Formatter<'_>` (checked). But
  several rules need `Pure` without `Send` or `Sync`:
  - authority-freedom: "every parameter type ... and every capture type is
    `Pure`" (`03:475–477`; D7 at `01:465–466`);
  - `Shared::with` closures: `Pure` only (`03:542`, LOCK-4 at
    `03:911–915`, `02:747–749`);
  - SUSP-1's classification of non-suspending instances (`03:950–956`).
- **The status the rule gives them.** Under the new rule, both types are
  `Pure`: neither is listed, and their only type argument is a lifetime
  (`02:863–864`, `02:836`). Yet `Arguments` erases a reference to every
  formatted value, of any type, and `Formatter` erases a
  `&mut dyn fmt::Write`.
- **Construction 1: nested acquisition.** This compiles in Rust with
  `Mutex` standing in for `Shared` (checked):

  ```rust
  fn log_to(cell: &Shared<Vec<String>>, args: fmt::Arguments<'_>) {
      cell.with(|v| v.push(args.to_string())).ok();  // captures &Arguments: "Pure"
  }
  // Tag { id, log: Shared<Vec<u32>> }; its Display runs self.log.with(..)
  ```

  Task A runs `log_to(&x, format_args!("{}", tag_y))` and task B runs
  `log_to(&y, format_args!("{}", tag_x))`. Each holds one cell and then
  acquires the other inside its closure, so the two tasks can deadlock.
  That breaks "`with` cannot deadlock against `with`: no closure acquires"
  (`03:927–929`) and LOCK-4's "cannot ... call `with` again".
- **Construction 2: I/O under the lock.** A `Display` for a type holding a
  `Shared` writes from inside the closure:
  `self.cell.with(|c| write!(f, "{}", c.hits))`. That captures
  `f: &mut Formatter`. Under `writeln!(stream, "{stats}")`, the closure
  does network I/O while it holds the cell. Under v1's evented runtime it
  would suspend there, which LOCK-4 forbids.
- **Construction 3: a nondeterministic authority-free function.**
  `#[pure] fn render(a: fmt::Arguments<'_>) -> String { a.to_string() }`
  passes `FX0511`. Yet its result depends on whatever another task last
  wrote into the shared state that `Tag`'s `Display` reads. That breaks
  §3.3's determinism and "no other task" items (`03:492–499`).
- **Why it is in scope.** Each construction is round 35's shape: a std type
  that hides a payload that is not `Pure` meets a `Pure` bound, and code
  classified as authority-free locks or blocks. fx-cze's acceptance
  requires that no such construction remain.

**Fix** (02 and 03, no core tokens):

- List `fmt::Arguments` and `fmt::Formatter` as not `Pure`, by the `dyn`
  rule, since each erases a value whose type is not bounded by `Pure`.
  B2's fix derives this on its own.
- Replace the `fmt::Arguments` clause at `02:870–871` with that verdict.
- Rewrite 03 §3.3's premise (`03:505–506`) to name the derived rule
  rather than "std bounds every payload it erases".
- Costs, which should be stated:
  - a `with` closure can no longer capture `f` or a passed-in `args`.
    Format before you lock: `let n = self.cell.with(|c| c.hits)?;
    write!(f, "{n}")`, which is better practice anyway;
  - helpers that take `fmt::Arguments` are not authority-free;
  - under SUSP-1, every `Display::fmt` instance is may-suspend. That is
    true of a `Formatter` over a socket. v1's lowering (§7.2) already
    faces the same case for `dyn` values created inside authority-free
    code.

### B2. The std rule drops the walk without replacing what the walk caught

`02:863–864`: "Std's own types take their `Pure` status from this section
and their type arguments, not from a walk of std's private fields." The
walk caught more than raw pointers. Two std types that the walk with
substitution (`02:798–799`) correctly made not `Pure` are now `Pure`:

- **`mpsc::IntoIter<T>`** (and `Iter`, `TryIter`). "A `Receiver` is
  iterable" (`03:850`), following Rust's `mpsc`, so `rx.into_iter()`
  returns a std type that owns a `Receiver<T>`. 02's list names
  `Receiver<T>` exactly and has no "any type that holds one" clause for
  synchronization handles (`02:825–828`), unlike its resource-handle
  bullet (`02:822–824`). So `IntoIter<u32>` is `Pure` by its type
  argument. It is also `Send` when `T` is, so
  `join(move || it.next(), || ..)` is accepted (`02:763`). The branch
  blocks on `recv`, and its result depends on which producer task sent
  first. That breaks determinism and runtime independence (`03:492–502`).
  03 §0's "channel ends" (`03:43`) and 03 §3.1's list (`03:439`) could be
  read to cover it, but the new rule routes std's types to 02's
  enumeration.
- **`Cow<'_, B>`**, admitted as Rust's (`reference.md:40`). Its public
  variant `Owned(<B as ToOwned>::Owned)` holds an associated type of `B`,
  not `B`. Take `struct View { id: u32 }` (no `Clone`) with
  `impl ToOwned for View { type Owned = Owned; .. }`, where
  `Owned { view: View, log: Shared<Vec<u32>> }` implements `Borrow<View>`.
  Then `Cow::Owned(Owned { .. })` is `Pure` by its type argument `View`,
  and `Sync`. Two `join` branches that each take a `&Cow<View>` and run
  `if let Cow::Owned(o) = c { o.log.with(..) }` leave the log in schedule
  order. The Rust analogue compiles and runs (rustc 1.96, checked).
- **Three consistency defects in the rule itself:**
  - "Std's own types" does not say whether it covers the `core` and
    `alloc` items that ADMIT-1 admits. `fmt::Arguments` and `Cow` live
    there. If it does not cover them, `Vec` (`alloc`) is walked again,
    which was the problem the rule set out to solve.
  - "Their type arguments" leaves out associated types (`Cow`) and the
    handles a std type holds (`IntoIter`).
  - It contradicts three texts that still state the walk without
    exception: 02's deep rule, "looks through every field, private ones
    included" (`02:798`); 03 §3.1, the same words (`03:427–428`); and
    D7, "structural over every reachable type, including ... generic
    arguments" (`01:464–465`). A reader of 03 or D7 alone walks
    `Vec`'s raw pointer.

**Fix** (02 §9, 03 §3.1 and §3.3; no change to D7's meaning; no core
tokens). Keep the walk for std too, with two declared exceptions, as Rust
does for `Send` with `unsafe impl Send for Vec<T> where T: Send`. Replace
`02:863–871` with something like:

> Std's types, including the `core` and `alloc` items ADMIT-1 admits, are
> walked like any other, with two exceptions. A raw pointer through which a
> std type owns or borrows values of a type argument (`Box`, `Vec`,
> `String`, `Rc`, the collections, slice iterators) counts as that `T` or
> `&T`. An erased payload counts as `Pure` when every std constructor that
> fills it bounds it by `Pure`: `io::Error::new` and `io::Error::other`
> take `E: Into<Box<dyn Error + Send + Sync>> + Pure`. Any other erased
> payload follows the `dyn` rule, so `Box<dyn Error + Send + Sync>`,
> `dyn Any`, `fmt::Arguments` and `fmt::Formatter` are not `Pure`.
> `Waker` is not admitted (ADMIT-1).

This settles B1. It makes `Cow` follow `B::Owned` and makes the
`Receiver` iterators follow the `Receiver` they hold. It agrees with
`02:798`, `03:427` and D7 as written, needing at most "with std's two
exceptions (02 §9)" added to 03 §3.1. Also add to 02's synchronization
handles "and any std type that holds one, such as the iterators a
`Receiver` returns", to match the resource-handle bullet.

## Non-blocking notes

### N1. The `io::Error` bound has no stated mechanism, code, or v1 scope

- **Mechanism.** `io::Error` is Rust's type, re-exported "as Rust"
  (`reference.md:41–43`), and shared with Rust code (`01:803–805`). rustc
  therefore checks Rust's signature, and the `+ Pure` can only be a
  checker obligation on an admitted inherent method. That makes it
  ADMIT-1's business (`03:378–381`). Neither ADMIT-1 nor 02 says so, and
  no text names the diagnostic. 03 §3.1's fallback gives `FX0517`
  (`03:421–424`), but only by inference.
- **v1.** Because the type is shared, a v1 imported crate or a RUST-6
  caller builds `io::Error` with Rust's unbounded signature. Such a value
  is `Pure` in Tide code. fx-0o3 (v1 crate classification) does not
  mention `io::Error`.

**Fix**: add to ADMIT-1 that "`io::Error::new` and `io::Error::other` are
admitted with an added `E: Pure` obligation (`FX0517`)". Report to the
owner that fx-0o3 should cover `io::Error` values made by crates.

### N2. `NonNull` and fd wrappers: the table is now the only line of defence

Under the new rule, a std type that 02 does not list is `Pure` exactly
when its type arguments are. Three cases follow:

- **`NonNull<T>`.** It is `Pure` when `T` is, and it is admitted in
  packages granted `unsafe`. The walk used to see its `*const T`. A user
  `unsafe impl Sync` type over a `NonNull<u8>` field is then `Pure`. That
  undercuts 03 §3.3's "raw pointers ... none of which is `Pure`"
  (`03:516–518`). It stays inside the `unsafe` trust hole.
- **`std::os::fd`, if Tide's std re-exports it.** `BorrowedFd<'_>` and
  `OwnedFd` carry a `File`'s authority: `try_clone_to_owned`, then
  `File::from(OwnedFd)`. Reference 12.3's "as Rust" row ends in "..."
  (`reference.md:1015`), so this is open. The walk would not have caught
  it either, since an fd is an `i32`. The resource-handle definition
  (`02:818–820`) covers it in principle.
- **`std::backtrace::Backtrace`, if re-exported.** It hides a lazy,
  synchronized symbol cache and file reads. `capture()` reads
  `RUST_BACKTRACE` with no `Env`.

**Fix**: list `NonNull` with the raw pointers (`02:829`, `03:441`). B2's
fix makes this automatic only if the owning-pointer exception is limited
to pointers inside std types. State in 03 §1.1 or 12.3 that Tide's std
does not re-export `std::os::fd` or `std::backtrace`, or classify them.

### N3. `std::task::Waker` fails as rustc's error, not `FX0520`

ADMIT-1 handles `OnceLock` by having the driver report a path to Rust's
type as `FX0520`, not as an unresolved path (`03:390–392`). For `Waker`,
`Context` and `Poll`, Rust's usual path is `std::task`. In Tide that
resolves to Tide's own module (`reference.md:1006`), so
`use std::task::Waker` is E0432, and only `core::task::Waker` gets
`FX0520`. 6.9's omission is otherwise harmless (see Scope answers).

**Fix** (03 only): end the new ADMIT-1 item with "`std::task::{Waker,
Context, Poll, Wake}` are reported as `FX0520` too".

### N4. Three error types in reference 2.2 have no definition

`Cancelled`, `UrlError` and `json::Error` appear only by name. 2.2's
"`Pure`" is therefore a promise about future definitions:

- `Cancelled` needs no field;
- `UrlError` can follow the `url` crate's fieldless `ParseError`;
- `json::Error` should hold only data or an `io::Error`, as
  `serde_json::Error` does.

2.2's list also leaves out `RecvError`, `RecvTimeoutError` and
`SendError<T>` (`03:861–863`). The last is `Pure` only when `T` is, so
"Std error types" should not be read as all of them.

**Fix**: none needed now. When 03 or the companion defines these three,
check them against 2.2.

## Ready to merge

**No.**

- **What is closed.** Round 35's two routes are closed. The
  `io::Error::new`/`other` bound stops the `Tag` construction, and
  ADMIT-1's `core::task`/`alloc::task` item stops the `Waker` one. 02, 03
  and reference 2.2 agree on both types.
- **Why it cannot merge.** fx-cze's acceptance also requires that no new
  construction of the same shape remain.
  - B1 is one: `fmt::Arguments` and `Formatter` are `Pure` under the new
    rule, and `Shared::with` closures and authority-freedom need only
    `Pure`.
  - B2 is another: the new rule drops the walk that made
    `mpsc::IntoIter` and `Cow`'s `Owned` variant not `Pure`.
- **The fix.** B2's fix keeps the walk for std with two declared
  exceptions. That settles B1, removes the contradiction with `02:798`,
  `03:427` and D7, and costs no core tokens.
- **Before or alongside merge.** N1 (ADMIT-1 owns the `io::Error`
  obligation) and N3 are small 03 edits that can land with it. N2 and N4
  can wait.

Validation: this was a read-only review of the working tree at `04302c9`,
on 2026-10-20. The tree had `.beads/issues.jsonl` and four `design/` files
modified.

- **Read:**
  - the full `design/` diff;
  - fx-cze with its decision comment, and fx-0o3;
  - `reviews/round35-consistency.md` and `reviews/round33-consistency.md`;
  - in 02: §8.1, §8.4, §8.5 and §9;
  - in 03: §0, §1.1's error notes, §2.5, §3.1 to §3.5, §6.1's types, §6.4,
    §6.5, §7.1 to §7.3, and RUST-6;
  - 01 D7 and the D13 Rust-consumer bullet;
  - reference 1, 2.2, 6.6, 6.7, 6.9, 7, 7.1, 7.3 and 12.3;
  - the companion's core count line.
- **Rust 1.96 checks** (scratch files outside the repository):
  - `Arguments<'_>` and `Formatter<'_>` are neither `Send` nor `Sync`;
  - `fn log_to(.., args: fmt::Arguments)` capturing `args` in a lock
    closure compiles, as does a `Display` writing to `f` inside a lock
    closure;
  - a user `impl From<Key> for Box<dyn Error + Send + Sync>` passes the
    orphan rule;
  - the `Cow<'_, View>` construction compiles and shares across scoped
    threads;
  - `io::Error`'s constructors and `From` impls were read from rust-src
    (`std/src/io/error.rs`, `fs.rs`, `io/buffered/mod.rs`).
- **Greps** over `design/` for `Backtrace`, `std::os`, `OwnedFd`,
  `Stdio`, `NonNull`, `fmt::Arguments`, `Formatter`, `core::task`,
  `std::task`, `Waker`, `IntoIter`, `ToOwned`, `Display`, and the error
  type names.
- **Not done:** `scripts/count-tokens.sh` was not run, as instructed.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before round 37.

- **B1, B2:** 02 §9 takes this review's proposed wording: std's types are
  walked like any other, with the raw-pointer and erased-payload
  exceptions, so `fmt::Arguments` and `fmt::Formatter` are not `Pure`, and
  the `Receiver` iterators and `Cow` follow what they hold. 03 §3.1 cites
  the exceptions; 03 §3.3's premise is rewritten.
- **N1:** 02 §9 names `FX0517`; the v1 case is noted on fx-0o3.
- **N2:** `NonNull<T>` joins the foreign handles. `std::os::fd` and
  `Backtrace` are not exported by Tide's std.
- **N3:** ADMIT-1 reports `std::task::Waker` and the like as `FX0520`.
- **N4:** deferred to the definitions of `Cancelled`, `UrlError`, and
  `json::Error`, which reference 2.2 now constrains.
