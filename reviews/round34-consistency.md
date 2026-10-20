# Tide consistency review, round 34: OnceLock and Sync mutation in the `Pure` table

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ce2bd3`
("File follow-up beads from rounds 32-33: fx-u18, fx-obr, fx-t99"):
`git diff -- design/`, which changes `02-memory.md` §9,
`03-authority-concurrency.md` (ADMIT-1 and §3.1), `reference.md` 6.6 and
6.9, and `reference-companion.md` (14.1 and the core count). It implements
bead fx-u18 and its owner decision of 2026-10-20:

- (b) `OnceLock` and `LazyLock` are rejected by ADMIT-1 (`FX0520`), and
  `OnceCell` and `LazyCell` stay admitted and `Pure`;
- reference 6.6's semantic rule is replaced by a structural one: "a type
  with a user-written `unsafe impl Sync` is a synchronization handle, so not
  `Pure` unless `#[trusted_pure]`";
- no change to D7 or D9.

Scope, as asked:

- whether the four texts name one not-`Pure` set and one admission outcome
  for the four cell types, and whether 01 D7 or D9 contradicts them;
- whether the structural rule is well defined, decidable, closes the
  parallel-region hole, and spares ordinary safe programs;
- interactions with `static` items, `Shared<Option<T>>`, the simulator,
  `std::sync::raw`, and the trust diff;
- stale mentions anywhere in `design/`.

## Scope answers

| Question | Answer | Evidence |
| --- | --- | --- |
| Same not-`Pure` set | **Yes** for the sync-handle part. 02 (`02:823–827`), 03 §3.1 (`03:435–437`), and 6.6 (`reference.md:471–473`) all list `Shared`, the channel ends, `Scope`, `Task`, `Arc`, `sync::Weak`, atomics, `std::sync::raw`, and types with a user-written `unsafe impl Sync`. 6.6 still omits `extern` types, foreign function pointers, and `ThreadBound` (N7). | diff |
| Same outcome for the four cells | **Yes**, apart from the `static` case (B1). `FX0520` for `OnceLock` and `LazyLock` appears in 02 (`02:845–846`), ADMIT-1 (`03:389–390`), 6.9 (`reference.md:556`), and companion 14.1 (`reference-companion.md:35`). `OnceCell` and `LazyCell` are named `Pure` (`02:836–837`) and "stay" (`03:390`). | diff |
| D7 or D9 contradicted | **D7, yes** (B2). D9 (`01:565–576`) is not: its bounds are unchanged, and it names no cell type. D7's admission list (`01:419–429`) is open-ended ("rejects the rest"), so omitting `OnceLock` there is fine. | `01:450–454` |
| Rule well defined | **Mostly.** Two readings remain open: partial impls (N2) and what counts as "user-written" (N2, N6). | `02:826` |
| Decidable structurally | **Yes.** Each reading in N2 is a syntactic test of impls, or a trait selection rustc already performs. | — |
| Closes the hole | **For `Sync`, yes**: no sound counterexample was found. The `Send` route stays open as a trust hole, as before (N1). | below |
| Rejects ordinary safe programs | **No in v0.** `unsafe impl` needs the `unsafe` grant (`reference.md:956`, `FX0901`), so safe code cannot write one. v1 imported crates are open (N6). | — |

**Search for a counterexample.** The target was a `Pure + Sync` value that
one `join` or `par_map` branch can change while another branch reads it.

- **Std's types.** Std's `Sync` types with interior mutability are each
  listed as not `Pure` (`02:823–827`), or rejected (`OnceLock`,
  `LazyLock`), or not exported by Tide's std (`Once`, `Condvar`, `Barrier`,
  `std::sync::Mutex`; `reference.md:1007`). `core::sync::atomic` is
  `FX0520` (`03:388`). `-Zallow-features=` (`03:404`) blocks the unstable
  `SyncUnsafeCell` and `Exclusive`.
- **User types.** A user type that is mutable through `&` reaches
  `UnsafeCell`, which is not `Sync`. Making such a type `Sync` takes an
  `unsafe impl Sync` on it, or on a type that encloses it, and the deep rule
  (`02:794–800`) carries the rule's verdict outward.
- **A sound implementation.** A sound `unsafe impl Sync` that mutates
  through `&self` needs a synchronizing field: an atomic, a lock, a raw
  pointer, or a foreign call. Each of these is already not `Pure` or is
  `FX0520`.

So the new rule changes no verdict for sound code. It removes the
assumption that every unsafe author is sound, and it makes 6.6's old
semantic test decidable. Two routes remain open: an unsound `unsafe impl
Send` (N1), and `#[trusted_pure]` on a type, whose claim is unstated (N3).

**Token count.** `scripts/count-tokens.sh design/reference.md` measured
24,995 twice on the tree and 24,997 on HEAD. Both agree with the companion's
"Core count" lines (`reference-companion.md:12`, HEAD).

## New blocking problems

### B1. A `static` `OnceLock` gets `FX0502` only from a rule that no longer applies

- **Two rows, two codes.** Companion 14.1 gives two codes for the same
  type: a local `OnceLock` is `FX0520` (`reference-companion.md:35`), and
  `static ... OnceLock` is `FX0502` (`reference-companion.md:36`).
  Reference 6.4 also lists "a `static` holding atomics, `Mutex`,
  `OnceLock`, or `Cell`" under `FX0502` (`reference.md:431–433`).
- **What `FX0502` needs.** GLOBAL-1 raises `FX0502` when a `static` is not
  `Sync + Pure` (`03:245`). `OnceLock<T>` is `Sync`.
  - Before this diff, 6.6 listed `OnceLock` as not `Pure`.
  - The diff removes it there (`reference.md:472`) and does not add it to
    02 or 03 §3.1.
  - Under 02's table, `OnceLock<u32>` therefore falls under "everything
    else is `Pure` exactly when its components are" (`02:835`).
- **Depth does not rescue the row.** Std's private fields are not walked:
  `Vec`'s private fields reach `NonNull`, a raw pointer, yet 02 names
  collections `Pure`. So std types take their status from the table.
- **Result.** By the stated rules, `static X: OnceLock<T>` gets `FX0520`
  from admission and never `FX0502`. Row 36 and 6.4 predict a diagnostic
  that the rules do not derive. Nothing states a precedence between the two
  codes.

**Fix** (saves core tokens):

- drop `OnceLock` from 6.4's `FX0502` parenthetical (`reference.md:432`);
- drop it from companion row 36;
- make row 35 read "a `OnceLock` or `LazyLock`, local or `static`".

03's replacement table (`03:279`) needs no change, since it gives no code.

### B2. User `unsafe impl Sync` types are called synchronization handles, but every text defines those by creation through `&Tasks`

- **The definitions.** D7 defines synchronization handles by creation:
  "creating one takes `&Tasks`. Authority-free code can neither create nor
  receive them" (`01:450–454`). The same definition appears in five more
  places:
  - 03 §0's glossary (`03:42–46`), whose list also lacks `std::sync::raw`
    and the new types;
  - 03 §1.2's "Every constructor of a synchronization handle takes
    `&Tasks`" (`03:166`);
  - 03 §3.2's "An authority-free body also cannot create a synchronization
    handle" (`03:476–478`);
  - 03 §3.3's argument, "the body cannot create a handle without `&Tasks`"
    (`03:501–502`);
  - reference 7.4 (`reference.md:722–727`).
- **The new members.** The diff adds "every type with a user-written
  `unsafe impl Sync`" to the synchronization-handle list (`02:826`,
  `03:436–437`, `reference.md:472–473`). A user's constructor takes no
  `&Tasks`, so an authority-free body can create one.
- **The narrowing is partial.** 02 noticed the conflict and narrowed its
  own sentence to "Creating a **std** synchronization handle takes
  `&Tasks`" (`02:849`). D7 and the five places above still make the
  universal claim. The owner's comment says the rule "fits D7's existing
  'synchronization handle' term; no D7/D9 change". D7's text does not
  support that.
- **No behavioral harm.** A locally created instance is not `Pure`, so no
  `join` or `par_*` closure can capture it (`02:775`). The contradiction is
  definitional, but it is the kind the bead's acceptance rules out.
- **The stale premise.** 03 §3.3 still rests its argument on the rule the
  owner retired: "every type that is `Sync` and mutable through `&` is a
  synchronization handle and therefore not `Pure`" (`03:503–505`).

**Fix** (recommended, no D7 change):

- List the new types as their own not-`Pure` item rather than as
  synchronization handles, in 02 §9, 03 §3.1, and 6.6. For example: "every
  type with a user-written `unsafe impl Sync`, which can share mutation
  through `&self`".
- Restore 02:849's sentence to "Creating a synchronization handle".
- Rewrite 03 §3.3's premise structurally: "std's `Sync` types that mutate
  through `&` are listed synchronization handles, and any other such type
  reaches an `UnsafeCell` through a user `unsafe impl Sync`, which is not
  `Pure` (`02-memory.md` §9)".
- Add the new item to the `Pure + Sync` argument in 02 (`02:865–868`) and 03
  (`03:465–468`), since "every other synchronization handle" no longer
  covers it.

The alternative is to amend D7 and the five places to say "std's
synchronization handles". That changes D7, so it needs the owner's approval
(the bead's Notes). The owner should confirm the wording either way,
because the decision text says "is a synchronization handle".

## Non-blocking notes

### N1. `unsafe impl Send` still lets two branches share a `Cell`

- **The case.** `struct S<'a>(&'a Cell<u32>)` with `unsafe impl Send`, or
  a `Send` wrapper around `Rc<Cell<u32>>`, is `Pure + Send`, because
  references, `Cell`, and `Rc` are `Pure` (`02:835–837`).
- **The race.** Two `join` closures (`FnOnce + Pure + Send`, `02:767–775`)
  can each move in a copy that points at the same cell, and both write it.
- **Status.** Such an impl is unsound, and 02 already calls each
  `unsafe impl Send` a trust hole (`02:622–623`). But the new rule decides
  `Sync` structurally and leaves `Send` to trust. 03 §3.3's "`Cell`,
  `RefCell`, and `Rc` are not `Sync` and so cannot be shared with another
  task" (`03:505–506`) is true only modulo that hole.

**Fix**: in 03 §3.3, add "modulo unsound `unsafe impl Send` (a trust hole,
`02-memory.md` §8.1)". 02 §8.1 should also say that `unsafe impl Sync`
needs the grant: reference 11 does (`reference.md:956`), 02:622 names only
`Send`.

### N2. "A type with a user-written `unsafe impl Sync`" leaves two readings open

- **Partial impls.** Take `struct Buf<T>(UnsafeCell<T>)` with
  `unsafe impl Sync for Buf<u8>`.
  - Under the type-constructor reading, any impl naming `Buf` taints
    every `Buf<_>`, so `Buf<u16>` is not `Pure`.
  - Under the selected-impl reading, `Buf<u16>` has no `Sync` impl and
    stays `Pure`.
  - The two readings differ for `Vec<Buf<u16>>` passed to `par_for`,
    which needs only `Pure + Send`.
- **"User-written".** Read literally, the word excludes `macro_rules!` and
  derive output. It is also unclear whether it includes other Tide
  packages and imported Rust crates (N6).

**Fix**: in 02 §9, define "outside std, after macro expansion", and pick
the type-constructor reading. It is the simpler test and errs toward not
`Pure`.

### N3. `#[trusted_pure]` on a type has no stated claim

- **The escape.** The new rule says such a type is not `Pure` "unless
  `#[trusted_pure]`".
- **The only claim list.** FFI-3 (`03:652–663`) lists claims for
  functions. For types, 02 and 03 say only "authority-free in fact"
  (`02:792`, `03:421–422`).
- **The gap.** A reviewer could approve a sync-mutating cell as
  "authority-free", since it does no I/O. That would reopen exactly the
  hole the rule closes.

**Fix**: state in 03 §3.1, or in FFI-3, that `#[trusted_pure]` on a type
also claims that nothing reachable through `&self` changes while the value
is shared. That is the `Pure + Sync` guarantee of `02:865–868`.

### N4. `Shared<Option<T>>` is offered where it cannot be used

- **The offer.** 02 says `OnceLock` "shared by `&` in a parallel region"
  is rejected, then offers "Build the value before the region, or use a
  `Shared<Option<T>>`" (`02:846–849`).
- **Why it fails there.** `Shared` is not `Pure` (`02:823`), and every
  `join` and `par_*` closure and capture must be `Pure` (`02:767–775`).
  Inside a region, only the first option works. `Shared<Option<T>>`
  replaces `OnceLock` only between spawned tasks.
- **Limits of the replacement**, from 7.4 (`reference.md:704–735`):
  - `with`'s closure must be `Pure`, so an initializer that does I/O runs
    outside the lock, and two tasks may both compute the value;
  - `R` cannot borrow from the payload, so readers clone or work inside
    `with`;
  - `T: Send + Pure` rules out caching a handle.

**Fix**: in 02, change the sentence to "Build the value before the region;
between spawned tasks, use a `Shared<Option<T>>`". Companion row 35 is
fine as written.

### N5. ADMIT-1 cannot see `OnceLock` or `LazyLock`

- **What ADMIT-1 governs.** It governs listed `core` and `alloc` items and
  listed inherent methods of re-exported std types (`03:378–384`,
  `01:419–422`).
- **Where the types live.** `OnceLock` and `LazyLock` live only in Rust's
  std. Tide's `std::sync` exports neither (`reference.md:1007`), and "the
  real std's paths cannot be named" (`03:375`).
- **The result.** `use std::sync::OnceLock` is rustc's unresolved-path
  error, not an admission decision. `OnceCell` and `LazyCell`, in
  `core::cell`, are genuinely admission items.
- **A precedent.** `FX0501` already diagnoses std paths that Tide's std
  lacks, such as `std::fs::read` (`reference.md:548–550`).

**Fix**: in ADMIT-1, say "Tide's std omits `OnceLock` and `LazyLock`, and
the driver reports either path as `FX0520` with the fix". Alternatively,
give them `FX0501`'s mechanism.

### N6. v1 imported crates: the rule's reach is unstated

- **The rules.** RUST-4 carries `Send` and `Sync` over (`03:1363–1365`),
  and RUST-2 makes wrappers `#[trusted_pure]` when `authority = []`
  (`03:1350–1351`).
- **The common case.** `regex` is the chapter's own example
  (`03:1341`), and a `par_map` closure capturing a `&Regex` is common Rust.
  Its internal cache pool is `Sync` through an `unsafe impl` and holds
  atomics and a mutex from real std.
- **No answer in the text.** Nothing says whether:
  - the deep rule walks a crate type's private fields;
  - a crate's `unsafe impl Sync` counts as "user-written";
  - real std's `OnceLock` or `Mutex` inside a crate is `Pure`. The diff
    removed `OnceLock` from the not-`Pure` text.
- **The choices.**
  - **Yes to walking the fields:** `&Regex` is rejected in `par_map`.
  - **No:** a `dashmap` or `once_cell::sync::Lazy` value is `Pure + Sync`,
    and branches can race on it.
- **Scope.** This predates fx-u18, but fx-u18's rule makes the question
  sharper.

**Fix**: none in this bead. File a v1 bead under RUST-2 and RUST-4 to
decide how imported types are classified for `Pure`.

### N7. Small alignment points

- **6.6's not-`Pure` list.** It omits `extern` types, foreign function
  pointers, and `ThreadBound`, which 02 and 03 §3.1 list
  (`reference.md:470–475`, `02:828–830`, `03:438–439`). This predates the
  diff, and all three are v1 (FFI §5, RUST-4). Strictly, the acceptance
  bullet "name the same not-`Pure` set" is met only if the core reference
  is read as v0-only. Labelling the 02 items "(v1)" would make that
  explicit.
- **`LazyCell` in the tables.** `LazyCell` is newly named `Pure`
  (`02:837`) and "not `Sync`" (the owner decision). It is absent from
  02 §8.1's `Send`/`Sync` table (`02:613`), and from the `Sync` excludes
  lists at `02:865–866` and `03:465–466`. These lists are illustrative.
  Adding `LazyCell` beside `OnceCell` at `02:613` would make the "task-local
  like `Cell`" claim checkable.
- **6.6's `Arc` rationale.** 6.6 dropped "because drop timing can reveal
  scheduling" (`reference.md:478–479`). The citation to D7 still carries
  the reason, so this is acceptable as a token saving.

## Stale mentions

A grep over `design/` for `OnceLock`, `LazyLock`, `OnceCell`, `LazyCell`,
`sync::raw`, `unsafe impl Sync`, "synchronization handle", and "mutable
through" found:

- **Stale:**
  - `reference.md:432` and `reference-companion.md:36` (B1);
  - `03:42–46`, `03:166`, `03:476–478`, `03:501–505`, `01:450–454`, and
    `reference.md:722–727` (B2).
- **Consistent:**
  - `03:279` (GLOBAL replacement table);
  - `03:171–172`, `03:919–920`, and `reference.md:743–744`, 958
    (`std::sync::raw` under the `unsafe` grant, now listed as not `Pure` in
    all three texts);
  - `02:400` and `02:613` (variance, `Send`/`Sync`);
  - `02:866` and `03:466` (`OnceCell` in the `Sync` excludes lists).
- **None** in `00-first-principles.md`, `04-contracts.md`, or the
  simulator sections. §9.2's `world.http` peer state needs "a
  synchronization handle made from `sim.tasks`" (`03:1120–1122`). That is
  consistent with B2's recommended fix, under which user `Sync` types are
  not synchronization handles.

## Ready to merge

**No, not yet.** Both blocking items are wording fixes that cost no core
tokens; B1 frees a few.

- **B1:** drop `OnceLock` from the `FX0502` texts, so that a `static`
  `OnceLock` has one derivable code, `FX0520`.
- **B2:** list user `unsafe impl Sync` types as their own not-`Pure` item,
  so D7's creation-through-`&Tasks` definition stays true. Otherwise, get
  the owner's approval to narrow D7.

The decision itself is consistent across the four texts and with D9. It
closes the parallel-region hole for `Sync`, and it rejects no safe v0
program. N1–N4 are short 02 and 03 edits. N5 is a mechanism sentence in
ADMIT-1. N6 needs a v1 bead.

Validation: this was a read-only review of the working tree at `5ce2bd3`,
on 2026-10-20. The tree had `.beads/issues.jsonl` and the four `design/`
files modified. The companion's "Core count" line changed from 24,994 to
24,995 during the review; the other three files' diffs did not change.

- **Read:**
  - the full `design/` diff, and fx-u18 with its owner comment;
  - `reviews/round33-consistency.md`;
  - in 02: §8.1, §8.5, and §9;
  - in 03: §0, §1.2, §2.2 GLOBAL-1 to GLOBAL-6, §2.5 ADMIT-1 and ADMIT-2,
    §3.1 to §3.4, §5.1, §6.5, §9.2, §9.7, and §11 RUST-1 to RUST-5;
  - 01 D7 and D9;
  - reference 6.4, 6.6, 6.9, 7.4, 11, and the module table;
  - companion 14.1.
- **Greps:** as listed under Stale mentions, and for `FX0501`, `FX0502`,
  `FX0520`, `std::sync`, and re-exports.
- **Token count:** `scripts/count-tokens.sh` on the tree, run twice
  (24,995 both times), and on HEAD's `reference.md` (24,997).

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before commit; round 35 confirms each item.

- **B1:** `OnceLock` is dropped from both `FX0502` texts (reference 6.4,
  companion 14.1), and the `FX0520` row reads "a local or `static`
  `OnceLock` or `LazyLock`".
- **B2:** the owner revised the decision (fx-u18, second comment): the
  user-`unsafe impl Sync` clause is dropped from 02 §9, 03 §3.1, and
  reference 6.6, so D7 needs no change, and 03 §3.3 states the structural
  argument instead.
- **N1:** 02 §8.1 names `unsafe impl Sync` beside `unsafe impl Send` as a
  trust hole under the `unsafe` grant; 03 §3.3 names the unsound case.
- **N2:** moot under the revised decision.
- **N3:** 02 §9 states what `#[trusted_pure]` claims.
- **N4:** 02 §9 and companion 14.1 offer `Shared<Option<T>>` only between
  spawned tasks.
- **N5:** ADMIT-1 says the driver reports a path to Rust's `OnceLock` or
  `LazyLock` as `FX0520`.
- **N6:** reported to the owner (v1 imported crates).
- **N7:** `LazyCell` is added to 02 §8.1's table; the v1 items in 6.6 are
  deferred; the `Arc` cut stays.
