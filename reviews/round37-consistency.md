# Tide consistency review, round 37: the restored std walk (fx-cze)

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `04302c9`
("Add the Decisions rule: agents make technical design decisions"):
`git diff -- design/`, which changes `02-memory.md` §9,
`03-authority-concurrency.md` ADMIT-1, §3.1 and §3.3, `reference.md` 2.2
and 6.6, and the companion's core count and 14.1 `FX0603` row. It
implements bead fx-cze and answers round 36, adopting round 36's proposed
B2 wording for 02 §9. Scope, as asked:

- whether round 36's B1 and B2 are closed, and whether N1–N4 are handled
  or reasonably deferred;
- whether the restored rule (std's types walked like any other, with the
  raw-pointer and erased-payload exceptions) is well defined and consistent
  with 02 §9's deep rule, 03 §3.1 and §3.3, and 01 D7;
- whether the stated cost contradicts reference 6.5's sinks,
  `#[derive(Debug)]`, contracts, memoization or `const fn`;
- one more attempt at a value that meets a `Pure` bound through a std type
  hiding a payload that is not `Pure`, or through a std type the
  raw-pointer exception wrongly makes `Pure`;
- whether companion 14.1's `FX0603` row renders as a valid table row.

The core reference is unchanged in this pass and was not re-audited beyond
the lines cited.

## Round-36 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (`fmt::Arguments`, `fmt::Formatter` were `Pure`) | **Resolved** | Both are named not `Pure` by the `dyn` rule (`02:871–873`). Round 36's three constructions now fail: `log_to`'s closure captures `&Arguments` (`FX0603`, `03:915`); `self.cell.with(\|c\| write!(f, ..))` captures `&mut Formatter` (`FX0603`); `#[pure] fn render(a: fmt::Arguments)` is `FX0511` (`03:486–487`). The derivation also holds without the list: `Arguments`' `rt::Argument` holds an erased `NonNull<()>` (rust-src 1.96), and `Formatter` holds `&mut dyn fmt::Write`. 03 §3.3's premise is rewritten (`03:507–510`). The stated cost has two defects of its own (B1, B2 below). |
| B2 (the std rule dropped the walk) | **Resolved** | 02 §9 now walks std, `core` and `alloc` items with two exceptions (`02:864–873`), in round 36's wording. 03 §3.1 adds "with std's two exceptions" (`03:429–431`). The synchronization-handle bullet adds "any std type that holds one, such as the iterators a `Receiver` returns" (`02:827–829`), so `mpsc::IntoIter` is not `Pure` by the list and by the walk. `Cow::Owned(<B as ToOwned>::Owned)` normalizes to the user's `Owned` type, which is walked, so round 36's `View`/`Owned` construction fails. D7's "structural over every reachable type, including ... generic arguments" (`01:464–465`) and `02:798` now hold as written. Generic `Cow<'_, B>` leaves one gap that predates this change (N2). |
| N1, mechanism and code | **Resolved**; agree | "a bound the checker adds to these admitted methods (`FX0517`)" (`02:870–871`). ADMIT-1 is not amended, but 02 §9 owns `Pure`, and 03 §3.1's `FX0517` is the stated fallback for a `Pure` obligation with no rule of its own (`03:423–426`). One owner is enough. Reference 2.2 agrees (`reference.md:92–95`). |
| N1, v1 crates and RUST-6 callers | **Deferred** to fx-0o3; agree | fx-0o3's comment of 2026-10-20 states the `io::Error` case and two candidate rules. fx-0o3 already owns imported-type classification. |
| N2, `NonNull` | **Resolved** | "raw pointers (`*const T`, `*mut T`, `NonNull<T>`)" (`02:830`). 03 §3.1 (`03:444`) and reference 6.6 (`reference.md:473–474`) say "raw pointers", which 02 now defines. |
| N2, `std::os::fd`, `Backtrace` | **Deferred**; agree, though "per reference 12.3" overstates 12.3 | 12.3's last row is open-ended ("`Ordering`, `HashMap`, `BTreeMap`, `PathBuf`, ... \| as Rust", `reference.md:1015`), so 12.3 does not itself say these modules are not exported. D18 does make Rust's own paths unnameable (`03:374–375`). The restored walk lowers the risk anyway. `Backtrace`'s `Inner::Captured` holds a `LazyLock` (rust-src 1.96), so it is not `Pure` either way. `OwnedFd` (an `i32`) is a resource handle by 02's definition (`02:818–820`). If either module is ever exported, the open question is admission (`Backtrace::capture` reads `RUST_BACKTRACE`), not `Pure`. |
| N3, `std::task::Waker` path | **Resolved** | "the driver reports `std::task::Waker` and the like as `FX0520`, as for `OnceLock`" (`03:395–396`). `Poll` falls under "`core::task`". 12.3's `std::task` row (`reference.md:1006`) agrees. |
| N4, undefined error types; `SendError<T>` | **Deferred**; agree | 2.2 now states the contract that `Cancelled`, `UrlError` and `json::Error` must meet (`Error + Send + Sync + Pure`, `reference.md:92–94`), so a later definition is checked against it. "Std error types" lists named types, so `SendError<T>` (`Pure` only when `T` is) is not covered. Nothing to change now. |

## Scope answers

| Question | Answer | Evidence |
| --- | --- | --- |
| Is the restored rule well defined? | **Mostly.** Exception 1 covers only raw pointers to *type-argument* values. Std types that point at a fixed type become foreign handles. Its interaction with the list (Arc) is implicit. | N1 |
| Consistent with 02's deep rule, 03 §3.1 and §3.3, D7? | **Yes.** The walk is restored, the exceptions are cited from 03 §3.1, and §3.3's premise names them. The deep rule's silence on projections predates this change but now matters for `Cow`. | `02:798–804`, `03:429–431`, `03:507–510`, `01:464–465`; N2 |
| Does the cost contradict reference 6.5's sinks? | **No.** Authority-freedom is decided by the function's own parameter and capture types (`03:478–479`), so a caller that formats stays authority-free. SINK-1 already exempts sinks from the may-suspend analysis (`03:325–327`). | `reference.md:450–452` |
| `#[derive(Debug)]`? | **No contradiction.** Derived `fmt` takes `&mut Formatter`, so it is may-suspend like any `Display::fmt`. No rule requires a `Debug` impl to be authority-free. | `03:954–960` |
| Contracts? | **No new cost.** CLAUSE-3 needs `Pure + Sync` argument types (`04:401–403`), and `Arguments` and `Formatter` are `!Sync` in Rust, so `format!` in a clause was already rejected. | rustc 1.96, round 36 |
| Memoization, `const fn`? | **No contradiction.** Both need authority-free functions with `Pure + Sync` parameters (`02:899`, `03:543`), so a memoized function may still call `format!` internally. `const fn` cannot format in Rust anyway. | — |
| LOCK-4 and 03 §3.5? | **Contradicted.** | B1 |
| v1 lowering? | **Gap made universal.** Non-suspending instances now call may-suspend `fmt` instances on every formatting call. | N3 |
| Companion 14.1 `FX0603` row | **Renders**, with 3 cells (pandoc `-f gfm`; `\|` inside code renders as `\|`). Its fix does not compile. | `reference-companion.md:44`; B2 |

**Construction attempts.** None succeeded:

- **`io::Error`.** Every std producer was checked: `new` and `other` (now
  bounded), `From<ErrorKind>`, `From<NulError>`, `From<TryReserveError>`,
  `From<IntoInnerError<W>>` (which keeps the inner `io::Error`), and
  `From<fs::TryLockError>`. The only private constructor, `_new`, is reached
  only through `new` and `other` (rust-src 1.96 `io/error.rs`, `fs.rs`,
  `buffered/mod.rs`).
- **A user conversion.** A user `impl From<K> for Box<dyn Error + Send +
  Sync>` with `K: Pure` builds its payload from `Pure` input alone, so it
  cannot reach a handle (`03:510–511`).
- **Exception 1.** Two kinds of std type could hide shared state behind a
  raw pointer that counts as `T`:
  - an `Arc`-like header with atomic counts. `Arc` and `sync::Weak` are
    listed, so they are not `Pure`;
  - a guard. `RwLockReadGuard`'s `inner_lock: &sys::RwLock` reaches
    atomics through a reference, and `MutexGuard` holds `&Mutex<T>`.

  `JoinHandle` holds an `Arc`. `Once`, `Condvar` and `Barrier` hold atomics.
  None of these is `Pure` under the walk, whether or not Tide's std exports
  it.

The one near miss is generic `Cow` (N2).

## New blocking problems

### B1. "A helper that takes `fmt::Arguments` is not authority-free" contradicts LOCK-4 and 03 §3.5

- **The new text.** 02 §9 says "a helper that takes `fmt::Arguments` is
  not authority-free" and "`format!` and the sinks build their `Arguments`
  and `Formatter` locally" (`02:878–882`). In Rust 1.96, `format!` expands
  to `$crate::fmt::format(format_args!(..))` (`alloc/src/macros.rs:114`).
  `alloc::fmt::format(args: Arguments<'_>)` (`alloc/src/fmt.rs:649`) is
  exactly such a helper. `write!(s, ..)` on a local `String` calls
  `String::write_fmt(&mut self, Arguments)`, which is another.
- **The contradicted text.**
  - LOCK-4: "`f` must be `Pure` ...: it captures no capability, channel
    end, or `Shared` handle, so every call in it is authority-free"
    (`03:915–917`).
  - 03 §3.5: "`Shared::with` closures | `Pure`, so every call in them is
    authority-free" (`03:546`).

  `cell.with(|v| v.push(format!("{n}")))` is a `Pure` closure, and LOCK-4
  accepts it. Its call to `fmt::format` is not authority-free, and under
  SUSP-1 it is a may-suspend instance (`03:954–955`). LOCK-4's own "Sinks
  are allowed" (`03:918`) is the same case, since a sink receives the
  record's `Arguments`.
- **Why it matters.** Read literally, "every call in it is authority-free"
  is a check that rejects `format!` and `to_string`-style helpers inside
  `with`. Read as a derivation, it is false. The rule's real guarantee is
  different: every value `f` can reach was built from `Pure` values, so
  nothing it reaches holds authority, even through a callee classified
  may-suspend. That is the argument 03 §3.3 already makes for
  authority-free bodies (`03:507–511`). The fx-cze acceptance requires no
  contradiction. This one was latent at HEAD, whose literal walk also made
  `Arguments` not `Pure` through its `NonNull<()>`, but 02 now states it.

**Fix** (03 only, no core tokens):

- LOCK-4: replace "so every call in it is authority-free and it cannot
  suspend, touch a capability, or call `with` again" with "so nothing it
  reaches holds authority, including the `fmt::Arguments` that `format!`
  and the sinks build in it, and it cannot suspend, touch a capability, or
  call `with` again".
- 03 §3.5's row: "`Pure`, so nothing they reach holds authority (LOCK-4)".

### B2. The `FX0603` fix example does not compile, and 02 describes it backwards

- **The example.** `let n = self.cell.with(|c| c.hits)?; write!(f, "{n}")`
  appears at `02:878` and `reference-companion.md:44`. Inside
  `Display::fmt`, which returns `fmt::Result`, the `?` needs
  `From<LockError> for fmt::Error`. No text defines that impl (grep for
  `From<LockError>` and `fmt::Error` over `design/` finds nothing).
- **Checked.** With rustc 1.96, a `Shared` stand-in returning
  `Result<R, LockError>`, and `LockError { Aliased, Poisoned }`, the line
  is E0277: "`?` couldn't convert the error to `std::fmt::Error`". An agent
  that follows the companion's fix for `FX0603` gets a rustc error.
- **The wording.** 02 says "format before taking the cell" (`02:877`), but
  the example reads under the cell and formats after releasing it. The
  companion's "do it before or after" is right.

**Fix** (02 and companion, no core tokens):

- Use `let n = self.cell.with(|c| c.hits).map_err(|_| fmt::Error)?;
  write!(f, "{n}")` in both places, and change 02's "format before taking
  the cell" to "copy the data out, then format".
- Alternatively, if Tide's std is meant to provide
  `impl From<LockError> for fmt::Error`, state it in 03 §6.5 beside
  `LockError`. Tide's std owns `LockError`, so the orphan rule allows the
  impl. It would turn a poisoned cell into the panic that `to_string`
  raises on a formatting error. `map_err` is the smaller change and
  states nothing new.

## Non-blocking notes

### N1. Exception 1 covers only pointers to type-argument values, and the list carries `Arc`

- **Std types with no matching type argument.** "A raw pointer through
  which a std type owns or borrows values of a type argument" (`02:865–867`)
  leaves out std types whose raw pointer reaches a fixed type. Under the
  walk, each becomes a foreign handle and so is not `Pure` (`02:830`). From
  rust-src 1.96:
  - `string::Drain<'a>` holds `string: *mut String` (`alloc/src/string.rs:3493`);
  - `panic::Location<'a>` holds `filename: NonNull<str>`
    (`core/src/panic/location.rs:43`). `&'static Location<'static>` is
    common in `#[track_caller]` error types;
  - `io::IoSlice<'a>` holds an `iovec` with `*mut c_void`
    (`std/src/sys/io/io_slice/iovec.rs:14–16`).

  Each is an over-rejection, not a hole, but it is unintended. Under it, a
  struct holding a `Location`, or a closure capturing a `String::drain`
  iterator, fails `FX0603`, `FX0517` or authority-freedom.
- **Precedence over the list.** Exception 1, read alone, also covers
  `Arc<T>`. Its `NonNull<ArcInner<T>>` owns `T`, so the pointer "counts as
  `T`" and the walk never sees `ArcInner`'s atomic counts. `Arc` and
  `sync::Weak` are not `Pure` only because the list names them
  (`02:826`). "Walked like any other" implies that the list comes first,
  since it applies to user types too, but that ordering is load-bearing
  and unstated.

**Fix** (02 §9, no core tokens): "A raw pointer through which a std type
owns or borrows plain values (of a type argument, or of a fixed type, as
`string::Drain`'s `String` and `panic::Location`'s `str`) counts as owning
or borrowing that type", and add "A type the list names is not `Pure`
whatever its fields."

### N2. The deep rule does not classify associated-type projections, which `Cow` now relies on

- **The gap.** 02's deep rule says "a type parameter is `Pure` only if it
  is bounded by it" (`02:804`), and 03 §3.1 says the same (`03:435–436`).
  Neither text mentions a projection that does not normalize. The restored
  walk handles concrete `Cow<'_, View>` by normalizing
  `<View as ToOwned>::Owned`. In generic code, though, the variant's type
  is `<B as ToOwned>::Owned`, which neither text covers. Generic code is
  checked once at its definition (D4, `04:410–413`).
- **The near-miss construction:**

  ```rust
  fn note<B: ?Sized + ToOwned + Pure>(log: &Shared<Vec<String>>, c: &Cow<'_, B>)
  where B::Owned: Display {
      log.with(|v| if let Cow::Owned(o) = c { v.push(o.to_string()) }).ok();
  }
  ```

  The construction works if a checker treats `B::Owned` as `Pure` because
  `B: Pure`. Then the closure passes `FX0603` at the definition. With `B =
  View`, whose `Owned` holds a `Shared` and whose `Display` calls `with`,
  it nests an acquisition, which round 36's construction 1 did. rustc's
  analogue for auto traits requires `B::Owned: Send` separately, and that
  is the natural reading here. It predates fx-cze (`struct S<I: Iterator>
  { x: I::Item }` has the same gap), so it does not block.

**Fix** (02 §9 and 03 §3.1, no core tokens): "a type parameter, or a
projection on one that does not normalize (`<B as ToOwned>::Owned`), is
`Pure` only if bounded by it".

### N3. Every formatting call is now a non-suspending instance calling a may-suspend one (v1)

- **The mismatch.** SUSP-1 classifies by parameter types (`03:954–955`).
  `ToString::to_string::<T>` (parameter `&T`) and `fmt::format`'s callers
  are therefore non-suspending, while the `T::fmt` they call is
  may-suspend, as 02 now states (`02:879–880`).
- **What §7.2 lacks.** 03 §3.3 says the evented runtime compiles
  authority-free code "as an ordinary function" (`03:499–500`). §7.2
  states only the other direction: "Calls to non-suspending functions are
  ordinary calls on the worker's native stack" (`03:988–989`). Nothing
  says how a native frame calls a frame-record instance.
- **Why SUSP-3 does not help.** `Formatter::write_str` makes a `dyn
  fmt::Write` call, so `fmt` contains a suspending operation transitively.
- **Scope.** This predates fx-cze (any `dyn` created inside authority-free
  code) and is v1. §7.2 calls the lowering open (`03:972–975`). The new
  text makes the case universal.

**Fix** (03 §7.1, no core tokens): add to SUSP-1: "A call from a
non-suspending instance never suspends, since its arguments that are not
`Pure` were built from `Pure` values; under the evented runtime it runs the
callee as an ordinary call (§7.2)." Alternatively, record the case among
§7.2's open lowering questions.

## Ready to merge

**No.**

- **What is closed.** Round 36's B1 and B2 are closed: the walk is back
  for std, `core` and `alloc`, with two declared exceptions;
  `fmt::Arguments`, `fmt::Formatter`, the `Receiver` iterators and `Cow`'s
  `Owned` variant are classified correctly; 02, 03 §3.1 and §3.3, D7 and
  reference 2.2 agree.
- **The dispositions.** N1, N3 and `NonNull` are resolved. The v1 part of
  N1, the fd and `Backtrace` points and N4 are reasonably deferred. No
  construction of round 35's shape was found.
- **Why it cannot merge.** Two text defects remain, each a few words:
  - B1: LOCK-4 and 03 §3.5 claim every call in a `with` closure is
    authority-free, which 02's new sentence about `fmt::Arguments` helpers
    contradicts;
  - B2: the `FX0603` fix example fails rustc with E0277, and 02 describes
    it backwards.
- **Before or alongside merge.** N1 and N2 are one-sentence 02 and 03
  edits that can land with the fixes. N3 is v1 and can go to §7.2's open
  questions.

Validation: this was a read-only review of the working tree at `04302c9`,
on 2026-10-20. The tree had `.beads/issues.jsonl`, `design/02-memory.md`,
`design/03-authority-concurrency.md`, `design/reference.md` and
`design/reference-companion.md` modified, and
`reviews/round36-consistency.md` untracked.

- **Read:**
  - the full `design/` diff;
  - fx-cze with its decision comment, and fx-0o3 with its comment;
  - `reviews/round36-consistency.md` and `reviews/round33-consistency.md`;
  - in 02: §8.4, §8.5 and §9;
  - in 03: SINK-1 to SINK-8, §2.5, §3.1 to §3.5, §6.5 with LOCK-1 to
    LOCK-7, §7.1 and §7.2;
  - 01 D7's `Pure` bullets;
  - 04 CLAUSE-3;
  - reference 1, 2.2, 6.5, 6.6, the `with` bullets of 7, and 12.3;
  - the companion's 14.1 table.
- **rust-src 1.96 reads:**
  - `io/error.rs`, `fs.rs` and `io/buffered/mod.rs` (`io::Error`'s
    producers);
  - `alloc/src/macros.rs` and `alloc/src/fmt.rs` (`format!`);
  - `string.rs` (`Drain`), `panic/location.rs`, the unix `IoSlice`,
    `backtrace.rs` and `os/fd/owned.rs`.
- **rustc 1.96 check:** the `FX0603` fix example inside `Display::fmt`,
  with a `Shared` stand-in, is E0277 (scratch file outside the repository).
- **Rendering:** the companion's header and line 44 were rendered with
  `pandoc -f gfm -t html`; the row has 3 cells.
- **Greps** over `design/` for `format_args`, `fmt::Arguments`,
  `Formatter`, `fmt::format`, `fmt::Error`, `From<LockError>`, "every call
  in", `Display::fmt`, `SUSP-1`, `LOCK-4`, "may-suspend" and `FX0603`.
- **Not done:** `scripts/count-tokens.sh` was not run, since the core is
  unchanged in this pass.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before round 38: the proposed fixes for B1 (LOCK-4 and 03 §3.5),
B2 (`map_err(|_| fmt::Error)?`, "copy the data out, then format"), N1
(fixed-type pointees; listed types stay not `Pure`), N2 (projections), and
N3 (SUSP-1).
