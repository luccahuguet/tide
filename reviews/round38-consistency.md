# Tide consistency review, round 38: confirmation of the round-37 fixes (fx-cze)

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `4a05cc7`
("Require a Codex supervisor pass after clean Opus reviews"):
`git diff -- design/`, which changes `02-memory.md` §9,
`03-authority-concurrency.md` ADMIT-1, §3.1, §3.3, §3.5, LOCK-4 and
SUSP-1, `reference.md` 2.2 and 6.6, and the companion's core count and 14.1
`FX0603` row. It implements bead fx-cze and applies round 37's proposed
fixes for B1, B2, N1, N2 and N3. This is a narrow confirmation round.
Scope, as asked:

- whether round 37's B1, B2, N1, N2 and N3 are closed;
- whether the new SUSP-1 sentence holds: whether a non-suspending instance
  (every parameter and capture type `Pure`) can build an argument that is
  not `Pure` and reaches a suspending operation, checked against 03
  §3.2–§3.3 and §7.

Nothing else was re-audited.

## Round-37 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (LOCK-4 and 03 §3.5 claimed every call in a `with` closure is authority-free) | **Resolved** | LOCK-4 now reads "so nothing it reaches holds authority, including the `fmt::Arguments` that `format!` and the sinks build in it, and it cannot suspend, touch a capability, or call `with` again" (`03:915–920`), in round 37's wording. 03 §3.5's row reads "`Pure`, so nothing they reach holds authority (LOCK-4)" (`03:546`). A grep over `design/` for "every call in" finds only `04:569`, which is about contract clauses. Reference 7's `with` bullet ("cannot do I/O, suspend, or nest another `with`", `reference.md:733–736`) never made the stronger claim. "Cannot suspend" is now backed by the new SUSP-1 sentence (`03:961–965`), since `f` is itself a non-suspending instance: its captures are `Pure` and its parameter is `&mut T` with `T: Send + Pure` (LOCK-1). |
| B2 (the `FX0603` fix example did not compile, and 02 described it backwards) | **Resolved** | Both places now use `let n = self.cell.with(\|c\| c.hits).map_err(\|_\| fmt::Error)?; write!(f, "{n}")` (`02:882–883`, `reference-companion.md:44`). 02 says "copy the data out, then format" (`02:881`). **Type check:** inside `impl fmt::Display`, with a `Shared` stand-in whose `with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, LockError>` matches 03 §6.5's signature, rustc 1.96.0 compiles the line with no error or warning, and the program prints `3`. `map_err` turns `Result<u64, LockError>` into `Result<u64, fmt::Error>`, so `?` needs only the identity `From`. **Rendering:** the companion's header (`reference-companion.md:20–21`) plus line 44, through `pandoc -f gfm -t html`, gives one `<tr>` with 3 `<td>` cells. The escaped pipes render as `\|` inside the code span. The 02 code span crosses a line break (`02:882–883`), which CommonMark renders as a space. |
| N1 (exception 1 covered only type-argument pointees; the list's precedence was unstated) | **Resolved**, with a wording point (N1 below) | Exception 1 now covers "plain values, of a type argument (...) or of a fixed type (`string::Drain`'s `String`, `panic::Location`'s `str`)" (`02:867–871`). "A type the list above names is not `Pure` whatever its fields" (`02:866–867`) puts `Arc` and `sync::Weak` beyond exception 1. The sentence speaks of a named type's own fields, so it does not override exception 2 for `io::Error`'s payload: exception 2 says how a payload field counts in its containing type's walk, and `io::Error` itself is not on the list. That reading agrees with reference 2.2 (`reference.md:92–95`). |
| N2 (projections) | **Resolved** | 02's deep rule: "a type parameter, or a projection on one that does not normalize (`<B as ToOwned>::Owned`), is `Pure` only if it is bounded by it" (`02:804–805`). 03 §3.1: "a type parameter, or a projection on one that does not normalize, has one only if bounded by it" (`03:435–436`). Round 37's generic `Cow` construction now needs `B::Owned: Pure` at the definition. The summary lists still name only unbounded type parameters (N4 below). |
| N3 (non-suspending instances calling may-suspend `fmt` instances) | **Resolved** in SUSP-1, with wording points (N2, N3 below) | The new sentence (`03:961–965`) states that such a call never suspends and that the evented runtime runs the callee as an ordinary call. The claim holds (see below). Its reason is narrower than its subject, and its `§7.2` citation points at text that states only the converse. |

## Does the new SUSP-1 sentence hold?

**Yes.** In safe code, no non-suspending instance can build a value that is
not `Pure` and reaches a suspending operation. The sentence also follows
from text that already exists.

- **It is a corollary of 03 §3.3.** An instance with every parameter and
  capture type `Pure` is authority-free by 03 §3.2's definition
  (`03:478–480`). Guarantee 2 says that `f` never suspends (`03:499–500`).
  A callee that suspended would suspend `f`. So the sentence is true
  exactly when §3.3 holds, and its "since" clause is §3.3's argument
  (`03:507–511`) applied to arguments.
- **Every suspending operation needs a handle or a capability.** 03 §1.2's
  table lists the suspending operations: channel `send`, `recv` and
  `recv_deadline`, `Task::join`, `with` acquisition, and capability I/O.
  Each needs a channel end, a `Task`, a `Shared` or a capability. Every
  handle constructor takes `&Tasks` (`03:166–167`, `reference.md:722–727`).
  Capabilities have no ambient source (§2.1, §2.2). Core atomics,
  `OnceLock`, `LazyLock`, `core::task` and `alloc::task` are not admitted
  (`03:388–396`), and `std::sync::raw` needs the `unsafe` grant
  (`03:171–172`).
- **The values it can build.** A non-suspending instance can build values
  that are not `Pure`: `fmt::Arguments`, `Formatter`, `Box<dyn Error + Send
  + Sync>`, `&mut dyn fmt::Write`, or a closure behind `Box<dyn Fn()>`. Each
  wraps only values reachable from its `Pure` inputs. A `dyn` call on one
  dispatches to an impl whose `self` type is that `Pure`-built type, and no
  function body can reach authority except through its arguments
  (`03:449–451`, "a pointee's authority arrives through its arguments").
  A Tide `fn` pointer parameter such as `g: fn(&dyn Display)` is the same
  case: `g` is may-suspend, but its argument is built from `Pure` values.
- **Sinks and parallel regions.** `std::io::stdout()` builds a handle from
  nothing, but SINK-1 says sinks never suspend (`03:323–327`). SINK-5's
  blocking under backpressure is "backpressure, not suspension"
  (`03:346–349`). `join` and the `par_*` methods need no `Tasks`
  (`03:1019`), but their closures must be `Pure` (`03:1021–1027`), so they
  hand nothing that is not `Pure` to another branch.
- **Outside safe code.** Under the `unsafe` grant, an integer cast to a raw
  pointer and dereferenced could forge a `&Shared<T>` from a `Pure` value.
  That is the trust hole §3.3 already names for unsound `unsafe`
  (`03:519–524`), together with `#[trusted_pure]` (`03:426–429`) and
  RUST-6's unchecked callers (`03:466–469`). The sentence inherits §3.3's
  scope; it adds no hole.

## New blocking problems

None.

## Non-blocking notes

### N1. "Plain" in exception 1 is undefined, and `io::Error` itself depends on it

- **The word.** Exception 1 applies to a raw pointer "through which a std
  type owns or borrows plain values" (`02:867–868`). Nothing in `design/`
  defines "plain". If it only means "values of a known type", it adds
  nothing. If it is read as "values holding no erased payload", it changes
  `io::Error`'s verdict.
- **`io::Error`'s representation.** On 64-bit targets, rust-src 1.96 uses
  `Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>)`
  (`std/src/io/error/repr_bitpacked.rs:129`), where `Custom` holds
  `error: Box<dyn Error + Send + Sync>` (`std/src/io/error.rs:204–207`).
  Under the walk, `io::Error`'s only pointer is a `NonNull<()>` to a fixed
  type that contains the erased payload. Exception 1's widening to fixed
  types is what lets the walk reach that payload for exception 2 to
  classify. Before this pass, with type-argument pointees only, the pointer
  was a foreign handle, which would have made `io::Error` not `Pure`,
  against reference 2.2. The widening fixes that, which round 37 did not
  observe. A strict reading of "plain" would undo the fix.
- **Fix** (02 §9, no core tokens): drop "plain", so the clause reads "owns
  or borrows values of a type argument (...) or of a fixed type (...)",
  and optionally add "`io::Error`'s packed representation" to the
  fixed-type examples.

### N2. The SUSP-1 sentence's reason fits only half of "non-suspending instance"

- **The subject.** "Non-suspending instance" (`03:961`) also names SUSP-3's
  refined instances: "the compiler may compile an instance as
  non-suspending when its body, transitively, contains no suspending
  operation" (`03:969–971`). A refined instance may take `&Net` and pass it
  on, so the reason "its arguments that are not `Pure` ... were built from
  `Pure` values" is false for it. The conclusion still holds there, by
  SUSP-3's own precondition. In context, the sentence follows SUSP-1's
  classification and means that one, so this is ambiguity, not a false
  statement.
- **What the reason omits.** The reason covers arguments, but SUSP-1's
  previous sentence also classifies by "the callee value's type"
  (`03:957–958`). A locally built `Box<dyn Fn()>` called with no
  arguments is a call that "may suspend" under that sentence. The new
  sentence covers it only if the callee value counts as an argument.
- **Fix** (03 §7.1, no core tokens): "A call from an instance that SUSP-1
  does not classify may-suspend never suspends either, since every value
  it calls or passes that is not `Pure`, such as the `Formatter` that
  `to_string` passes to `T::fmt`, was built from `Pure` values (§3.3)".

### N3. The `§7.2` citation states the converse

- **The citation.** The sentence ends "under the evented runtime it runs
  the callee as an ordinary call (§7.2)" (`03:964–965`). §7.2 says "Calls
  to non-suspending functions are ordinary calls on the worker's native
  stack" (`03:993–994`), which is the opposite direction. It also says
  "Each may-suspend instance becomes a frame record ... and a resume
  function" (`03:982–983`), and it gives no form in which a native frame
  calls one.
- **No contradiction.** §7.2 calls the lowering open (`03:977–980`), and
  "native frames are never live across a suspension point" (`03:993–994`)
  still holds, since the callee never suspends. The sentence states a
  requirement that §7.2 should record and does not.
- **Fix** (03 §7.2, no core tokens): add a bullet, for example "A
  may-suspend instance called from a non-suspending one runs to completion
  before the call returns (SUSP-1), either as an ordinary-call copy or by
  driving its resume function on the frame stack; which is open." Then
  change SUSP-1's citation to point at that bullet. Alternatively, drop
  the citation.

### N4. The not-`Pure` summaries still name only unbounded type parameters

02's list ends "an unbounded type parameter" (`02:836`). 03 §3.1's list ends
"unbounded type parameters" (`03:447`), and reference 6.6 says "an unbounded
type parameter" (`reference.md:474–475`). The deep rule above each list now
includes non-normalizing projections (`02:804–805`, `03:435–436`), so the
lists are incomplete summaries rather than contradictions. Adding "or
projection" to 02's and 03's lists costs no core tokens. Reference 6.6 can
stay as it is.

### N5. Layout

`03:436` is 98 columns, since the projection clause was joined to "`02-memory.md`
§9 has the full table." on one line. `02:871` ("borrowing that type. An")
and `02:883` (``write!(f, "{n}")`), a helper that``) are short, unreflowed
lines. These are cosmetic only.

## Ready to merge

**Yes.**

- **Closed:** round 37's B1 and B2. LOCK-4 and 03 §3.5 now claim only
  what holds, and the `FX0603` example type-checks under rustc 1.96 and
  renders as a 3-cell row.
- **Resolved:** N1, N2 and N3. The precedence sentence does not disturb
  exception 2, and the projection rule appears in both owners. The SUSP-1
  sentence is true as a corollary of 03 §3.3. No construction was found
  in safe code that passes a value from a non-suspending instance that is
  not `Pure` and reaches a suspending operation.
- **Remaining:** N1 and N2 here are a word each in 02 and 03 and can land
  before merge. N3 is a v1 bullet in §7.2. N4 and N5 are optional.

Validation: this was a read-only review of the working tree at `4a05cc7`,
on 2026-10-20. The tree had `.beads/issues.jsonl`, `design/02-memory.md`,
`design/03-authority-concurrency.md`, `design/reference.md` and
`design/reference-companion.md` modified, and
`reviews/round36-consistency.md` and `reviews/round37-consistency.md`
untracked.

- **Read:**
  - the full `design/` diff;
  - fx-cze (`br show`);
  - `reviews/round37-consistency.md` and `reviews/round33-consistency.md`;
  - in 02: §9's deep rule, not-`Pure` list and std paragraph;
  - in 03: §1.2's handle table and its notes, §2.4 SINK-1 to SINK-5,
    ADMIT-1, §3.1 to §3.5, §6.5 with LOCK-1 to LOCK-7, §7.1, §7.2, and
    §8.1 with PAR-4;
  - reference 2.2, 6.5, 6.6, and 7's `Shared` bullets.
- **rustc 1.96.0 check:** the `FX0603` example inside `Display::fmt`, with
  a `Shared` stand-in using 03 §6.5's `with` signature, compiles cleanly
  and prints `3`. The scratch file was outside the repository.
- **rust-src 1.96 reads:** `std/src/io/error.rs` and
  `io/error/repr_bitpacked.rs`.
- **Rendering:** the companion's header and line 44 were rendered with
  `pandoc -f gfm -t html`, giving one row of 3 cells.
- **Greps** over `design/` for "every call in", "format before", "before
  or after", "projection", "unbounded", "bounded by it", `map_err`,
  `Barrier`, `Condvar`, `std::sync::raw`, atomics, and `stdout`.
- **Line lengths:** the added lines over 80 columns are `03:436` and two
  table rows (`03:546`, `reference-companion.md:44`); table rows are exempt
  by convention.
- **Not done:** `scripts/count-tokens.sh` was not rerun. The companion's
  new count (24,994) is the author's measurement.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before the Codex pass (round 39): N1 ("plain" dropped; `io::Error`'s
tagged representation named), N2 and N3 (SUSP-1 reworded; the native-frame
lowering left open in §7.2), N4 (02 and 03 summary lists name projections;
reference 6.6 is left as is under the core budget), and N5 (reflow).
