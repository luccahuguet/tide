# Tide consistency review, round 25: the fx-i5p decisions (G3, G4, G5, G6, G9)

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `417fbd5`
("Decide v0 contract details from the E1 dialect round (fx-eky)"):
`git diff -- design/`, which changes `01-design.md` (D15),
`03-authority-concurrency.md` (§0, §1.1, TASK-2, TASK-3, §6.4, §9.2, §9.3,
§9.5), `reference.md` (6.1, 6.3, 6.7, 6.8, 7.2, 7.3), and
`reference-companion.md` (14.1, 17, the recorded count). It implements bead
fx-i5p, whose comment of 2026-10-20 records the owner's decisions on G3, G4,
G5, G6, and G9 from `trials/e1-dialect/grading.md` §5 and §2.3 (C1–C4, C8).
Scope, as asked:

- whether each owner decision is carried out in 03 (and 01 D15 for G4) and
  stated in the reference or companion, with no "provisional" marker left
  on a decided item;
- whether the changed lines contradict 01, 02, 03, 04, the README, the rest
  of the reference pair, or the owner's decisions, including the Rust facts
  the text relies on;
- whether anything elsewhere is left stale;
- the core's token count against D14's budget.

Rule preservation beyond the changed lines and their neighbours was not
re-audited.

## Decision status

| Gap | 03 / 01 | Reference pair | Status |
| --- | --- | --- | --- |
| G3 channel errors | §6.4 (`03:809–841`) gives the three enums, `into_inner()`, `is_cancelled()`, Rust's `try_*` types, iteration ending at the first error, and "a disconnect is not a cancellation and sets no echo bit"; TASK-2 (`03:757–758`) and TASK-3 (`03:767`) follow | `reference.md:690–695` (7.3), `:661–669` (7.2); companion 14.1 row (`companion:47`) | **Carried out**; the companion row's diagnostic is wrong (B1) |
| G4 crash and rename | §1.1 (`03:95–99`), §9.5 (`03:1152–1162`); 01 D15 gains one clause (`01:846–848`) | atomic replace in 6.3 (`reference.md:417–422`); companion 17 Crashes (`companion:438–444`) | **Carried out**; the surviving set needs a coherence rule (N3) |
| G5 network | `is_transient` (`03:115–120`), round-trip cost, no-peer `Refused`, unknown-host `Dns` (`03:1101–1106`) | 6.7 (`reference.md:503–505`), 6.8 Time (`:532–537`), companion 17 Network time (`companion:401–405`); "provisional" markers gone | **Carried out**; the `reqwest-retry` attribution is inaccurate (N2), and 6.8 dropped a marker on an undecided item (N1) |
| G6 directory listings | §0 (`03:34–35`), §1.1 Directory listings (`03:100–108`) | 6.3's comment (`reference.md:384`) says only that a `DirEntry` carries the `Fs`'s authority | **Not stated in the reference pair**: the admitted `DirEntry` methods and `OsStr`/`OsString` (B2) |
| G9 stdin and setup | §1.1 Standard input (`03:109–114`), §9.2 (`03:1084–1092`) | 6.1 (`reference.md:334`), 6.8 table (`:524–525`), companion 17 Setup types (`companion:406–412`), companion 14.1 row (`companion:48`) | **Carried out**; `&Stdin: io::Read` is in 03 only (N5) |

**Rust facts checked.** Against the rust-src of rustc 1.96.0 on this
machine (`library/std/src/sync/mpsc.rs`, `io/stdio.rs`, `sys/fs/unix.rs`):

- `RecvError` is a unit struct, and `SendError<T>(pub T)` has no reason
  field and no `into_inner`, so §6.4's departure and its stated reason hold.
  `RecvTimeoutError` is `{Timeout, Disconnected}`, and `TryRecvError` and
  `TrySendError` are as 03 keeps them.
- `Stdin::read_line(&self, ..)` is inherent. `lines(self)` takes `self`, so
  Tide's `&self` receiver loosens it. `read_to_string` and `read_to_end`
  come from `io::Read`, and `is_terminal` from `IsTerminal`, which `Stdin`
  implements. `impl Read for &Stdin` is stable since 1.78.
- "Inherent methods win" was checked with a stand-in `Stdin` that has
  inherent `&self` methods plus `Read` for `Stdin` and `&Stdin`, under
  `use std::io::Read`. Called on `Stdin` and on `&Stdin`, every call
  resolved to the inherent method, with no ambiguity.
- `Command::args<I: IntoIterator<Item = S>, S: AsRef<OsStr>>` and
  `Command::env<K: AsRef<OsStr>, V: AsRef<OsStr>>` match §9.2. The
  `Item: AsRef<OsStr>` bound syntax is stable since 1.79.
- `DirEntry`'s `path() -> PathBuf`, `file_name() -> OsString`,
  `metadata()`, and `file_type()` match §1.1, and they are its only
  portable inherent methods.
- The companion's E0277 for an `Rc<Cell<u32>>` captured by a `Send`
  closure is right.
- The companion's E0532 is right for `Err(SendError(v))`, but wrong for
  `Err(RecvError)` (B1).

**Leftovers.** I grepped `design/`, `README.md`, and the E1 trial files
for the channel error names, "iterates until", "torn", "directory entr",
`read_dir`, `DirEntry`, `OsStr`, `Stdin`, `world.http`, `world.args`,
"transient", "1 ms", and "6.8". The README's only mention (`README.md:147`)
is generic. No example matches on `RecvError` or `SendError`. The two
`world.http` peers in examples (`03:1420–1421`, `companion:228–229`) capture
nothing, so they are `Send`. The stale pointers are in N5.

**Budget.** `scripts/count-tokens.sh design/reference.md </dev/null` printed
**24,976** and exited 0. That equals the companion's figure
(`companion:12`) and is 24 tokens under D14's 25,000.

## New blocking problems

### B1. `Err(RecvError)` is not a compile error: it silently binds a variable

The companion's 14.1 row (`companion:47`) says the patterns `Err(RecvError)`
and `Err(SendError(v))` fail with rustc E0532. 03 §6.4 (`03:834–836`) also
says these patterns "become" the new forms, which implies the old ones stop
compiling.

- **`Err(SendError(v))` does fail with E0532**, as the row says: "expected
  tuple struct or tuple variant, found enum `SendError`".
- **`Err(RecvError)` compiles.** `RecvError` is now an enum, so it lives
  only in the type namespace. A bare identifier in a pattern that names no
  unit struct, unit variant, or constant is a fresh binding. The pattern
  becomes a catch-all named `RecvError`. rustc gives only the
  `non_snake_case` warning, plus `unused_variables` if the arm ignores it.
- Checked with rustc 1.96.0, edition 2024: `match r() { Ok(_) => {},
  Err(RecvError) => {..} }` built, ran, and took the arm for
  `Err(RecvError::Cancelled)`. Only the qualified form,
  `Err(mpsc::RecvError)`, gives E0532 ("expected unit struct, unit variant
  or constant, found enum").

The common Rust idiom `Err(RecvError) => break` therefore keeps compiling.
It now also swallows `Cancelled`, and the table promises an error that never
comes. Here the behavior is benign: it ends the loop, as iteration does. The
documented diagnostic is still false, and it is the kind of claim an agent
acts on.

**Fix** (companion only, no core tokens). Split the row:

- `Err(SendError(v))` → `Err(e)` then `e.into_inner()` | rustc E0532.
- `Err(RecvError)` → `Err(RecvError::Disconnected)`, or `Err(e)` with
  `e.is_cancelled()` | "compiles as a catch-all binding; rustc warns
  `non_snake_case`".

If the owner wants a hard error, a checker lint on a pattern binding that
shadows a std type name would be new scope. In 03 §6.4, "become" can stay,
with "(a bare `Err(RecvError)` still compiles, as a binding)" added.

### B2. G6's decisions are not in the reference pair, and 6.9 still reads against them

The bead's acceptance says "the reference or companion states each
decision". For G6, the owner decided four things: `path`, `file_name`,
`metadata`, and `file_type` are admitted; a `DirEntry` carries the `Fs`'s
authority; iteration suspends; and `OsStr`/`OsString` keep Rust's API. The
reference pair states only the authority part (`reference.md:384`).

- **No admitted method list.** 6.9 still lists "I/O methods on std types
  (`Path::exists`, `metadata`; use `fs`)" (`reference.md:557`). That is
  the sentence that led the E1 writer to avoid `DirEntry::metadata` and
  `file_type` (`trials/e1-dialect/notes.md:30`, grading C3). Nothing in
  the core says that `DirEntry` is Tide's own type, outside that ban.
- **No `OsStr`/`OsString`.** Reference 1's "Standard types keep their Rust
  API" list (`reference.md:36–43`) still omits them, which grading C3 names
  as the cause. The companion does not mention them either.

**Fix** (about 15 core tokens, within the 24 of headroom):

- in reference 1's list, add "`OsStr`, `OsString`";
- in 6.3's comment, write "`DirEntry`'s `path`, `file_name`, `metadata`,
  and `file_type`, like a `File`, carry the `Fs`'s authority";
- in 6.9, write `Path::metadata` for `metadata`.

If tokens are short, the method list can go to the companion, with the
one-word 6.9 change kept in the core.

## Non-blocking notes

### N1. 6.8 dropped "(provisional)" from the file-operation latency, which no normative text decides

Before this change, `reference.md` 6.8 read "each file operation takes up to
1 ms and each network packet 1–10 ms, drawn from the seed (provisional)".
G5 decided the network figure, and 03 §9.3 now states it (`03:1101–1106`).
03 says nothing about file-operation latency beyond "modeled latencies drawn
from the seed" (`03:1100–1101`), and neither does the companion. Yet
`reference.md:533–534` now gives "up to 1 ms" with no marker, against the
reference's own rule (`reference.md:13`).

**Fix:** "each file operation takes up to 1 ms (provisional)" in the core,
or decide the figure in 03 §9.3.

### N2. `is_transient` is not "the classes that the `reqwest-retry` crate retries"

`03:117–119` justifies the set by `reqwest-retry`. That crate's
`default_on_request_failure` (TrueLayer/reqwest-middleware, `main`,
`reqwest-retry/src/retryable_strategy.rs`) treats every
`is_timeout() || is_connect()` error as transient. It also treats hyper's
`is_incomplete_message() || is_canceled()` as transient, and
`ConnectionReset`/`ConnectionAborted` I/O errors. Its success path makes
5xx, 408, and 429 transient, so the `Status` part matches. One part is an
inference not checked against reqwest's source: reqwest reports a failed
DNS lookup through the connector, so `is_connect()` is true and the crate
retries it, while Tide's `Dns` is not transient. The decided set is the
owner's and is reasonable. Only the attribution overstates.

**Fix:** "close to the classes `reqwest-retry` retries, minus DNS
failures", or drop the citation.

### N3. "Otherwise unspecified" allows crash states that no file system produces

§9.5 (`03:1152–1162`) and companion 17 (`companion:440–444`) make each
unsynced entry change whole, and leave "which unsynced changes survive"
otherwise unspecified, as the owner decided. Read literally, the seed may
keep a later change and drop an earlier one it depends on:

- a `rename("a", "b")` that survives while the unsynced `create("a")` it
  moved is lost;
- a file's entry that survives inside a new subdirectory whose own entry
  is lost.

The rename rule ("the target names the renamed file") is then undefined.
Journaling file systems commit metadata in order, so they never produce
these states, and the D15 clause invokes them (`01:848`). The simulator
needs some rule to be implementable.

**Fix** (03 §9.5 and companion 17, no core tokens), for the owner to
confirm, since it narrows "otherwise unspecified": "the surviving changes
are closed under dependency: a change survives only with the unsynced
changes that created what it names."

### N4. A cancelled `read_dir` or `lines()` iterator can spin forever under `.flatten()`

§1.1 makes advancing a `ReadDir` a suspending read (`03:105–106`), so it is
a checkpoint. Under TASK-3, every later checkpoint keeps returning the
cancellation. The common idioms `fs.read_dir(p)?.flatten()` and
`.filter_map(Result::ok)` would then skip `Some(Err(Cancelled))` forever,
in a scope that is already cancelled. Rust's own Unix `ReadDir` stops after
its first error (`end_of_stream`, `library/std/src/sys/fs/unix.rs:881`),
and §6.4 makes channel iteration end at the first error for the same
reason. `Stdin::lines()` has the same hazard: clippy's
`lines_filter_map_ok` exists for it.

**Fix** (03 §1.1, no core tokens): "a `ReadDir` or `lines()` iterator
ends after yielding its first error, as Rust's Unix `ReadDir` does."

### N5. Stale or incomplete text around the change

- **`recv_deadline`'s checkpoint.** 03 §6.4 now names `recv_deadline` a
  checkpoint (`03:812–814`), but 03 §1.2's table still says "`send` on a
  full `sync_channel`, `recv`; both checkpoints" (`03:139`). Reference 7.3
  says the same (`reference.md:692`). `RecvTimeoutError::Cancelled`
  implies the answer.
- **§0 and the §1.1 table.** §0 now counts `ReadDir` and `DirEntry` as
  capabilities (`03:34`), but §1.1's table has a `File` row and none for
  them.
- **Companion 17's intro.** It still reads "fault injection, crashable
  nodes, and the replay manifest" (`companion:383–384`), though 17 now also
  holds network time, setup types, replay commands, and `Record`'s fields.
- **The replay pointer.** Reference 13 still points to 6.8 for `tide sim
  --replay` (`reference.md:1061`), whose details moved to companion 17.
- **`&Stdin: io::Read`.** The owner decided it, and it is stated only in
  03 (`03:112–114`).
- **A provisional example.** Reference 6.1's "Every capability method takes
  `&self` (provisional): `&Rand` or `&Stdin` is enough"
  (`reference.md:338`) uses `Stdin` as its example, and G9 has now decided
  `Stdin`'s receivers.
- **The pipeline-stage advice.** 03's "a pipeline stage therefore returns
  `Ok` on a disconnect" (`03:840–841`) is absent from the reference. The
  core's "A disconnect is not an echo (7.1)" (`reference.md:695`) leaves
  the consequence for the reader to derive. The companion is the place for
  it.

### N6. Trait details of the new channel errors

- **Conditional derives.** "`Copy` (where `T` allows)" (`03:829–830`)
  attaches the condition to `Copy` only. In Rust, `SendError<T>`'s derived
  `Clone`, `PartialEq`, and `Eq` are just as conditional on `T`.
- **`From` impls.** Rust also has `From<RecvError> for TryRecvError` and
  `From<SendError<T>> for TrySendError<T>` (`mpsc.rs:1185`, `:1222`). Their
  targets have no cancellation case, so Tide must drop them, and 03 does
  not say so. `From<RecvError> for RecvTimeoutError` (`:1249`) can stay.

**Fix:** "`Clone`, `Copy`, `PartialEq`, and `Eq` where `T` allows; the
`From` conversions into `TryRecvError` and `TrySendError` are dropped."

### N7. Pre-existing, out of scope: 02's not-`Pure` list omits resource handles

03 says resource handles derived from capabilities are not `Pure`
(`03:411`), and the change relies on that for `ReadDir` and `DirEntry`
("like a `File`", `03:103–104`). 02, which owns the full `Pure` table, lists
only "the capability types … and their attenuated forms" (`02:814–815`).
`File`, `TcpStream`, and now `ReadDir` and `DirEntry` fall under its
"Everything else is `Pure` exactly when its components are". This predates
the change. It is reported here, not fixed.

### N8. Line length

These added prose lines run over 80 columns:

- `03:35` (104), `03:767` (84), `03:813` (100);
- `reference.md:505` (94), `:514` (106), `:695` (97).

## Ready to merge

**No.** B1 is a false rustc claim in the companion. B2 leaves half of G6's
decision, the very gap E1 hit, outside the reference pair, against the
bead's acceptance. Both fixes are small: B1 costs no core tokens, and B2
about 15 of the 24 left. N1–N6 can land with them. N3 needs the owner only
because it narrows "otherwise unspecified". G3, G4, G5, and G9 are carried
out, D15's new clause matches §9.5, and nothing in the changed lines
contradicts 01, 02, 04, or the README.

Validation: this was a read-only review of the working tree at `417fbd5`,
with `.beads/issues.jsonl` and four `design/` files modified, on
2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-i5p and its comment, via `br show` and `br comments`;
  - in 03: §0, §1.1, §1.2, §2.5 (ADMIT-1), §6.2–§6.4, §9.1–§9.5, and §13;
  - 01 D7, D8, and D15;
  - 02's `Pure` list;
  - reference 1, 6.1, 6.3, 6.7, 6.8, 6.9, 7.1–7.3, and 12.3;
  - companion 14.1 and 17;
  - `trials/e1-dialect/grading.md` §5 (G3–G9) and C2, C3, C8;
  - `trials/e1-dialect/notes.md` items 8 and 10.
- **Rust:** read rust-src 1.96.0 (`mpsc.rs`, `io/stdio.rs`,
  `sys/fs/unix.rs`). Compiled three probes with `rustc --edition 2024` in
  the scratchpad: the old channel patterns against enum errors, the
  qualified `mpsc::RecvError` pattern, and method resolution for inherent
  `Stdin` methods beside `Read` for `Stdin` and `&Stdin`.
- **`reqwest-retry`:** read `retryable_strategy.rs` on GitHub `main`.
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null` was
  run once, with a 400 s timeout. It printed `24976` and exited 0.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the working tree before round 26.

- **B1:** companion 14.1 splits the row. `Err(SendError(v))` stays E0532;
  `Err(RecvError)` is listed as compiling to a catch-all binding (only a
  `non_snake_case` warning), with `RecvError::Disconnected` or
  `is_cancelled()` as the fix. 03 §6.4 says the same.
- **B2:** reference 1 lists `OsStr` and `OsString` among the standard
  types; 6.3's `Fs` block says a `DirEntry` is as std and carries the
  `Fs`'s authority like a `File`, so 6.9's ban on I/O methods of std types
  no longer reads against it. 03 §1.1 gains a `ReadDir`, `DirEntry` row.
- **N1:** "(provisional)" restored on the 1 ms file-operation figure.
- **N2:** 03 drops the `reqwest-retry` claim. It now states the
  classification on its own terms: `Denied`, `Dns`, and `Io` are not
  transient.
- **N3:** 03 §9.5 and companion 17 add that a change to an entry survives
  only if the change that created the entry does. This narrows "otherwise
  unspecified" only to states a file system can reach, so it does not
  change the owner's decision.
- **N4:** 03 §1.1: a `ReadDir` ends after its first error, as Rust's does
  on Unix, and `Stdin::lines()` ends after yielding a cancellation error
  (a stated departure from Rust).
- **N5:** `recv_deadline` is named as a checkpoint in 03 §1.2 and
  reference 7.3; companion 17's intro lists its new parts; reference 13
  points to companion 17 for replay; the pipeline-stage advice is a
  companion 14.1 row. Declined: `&Stdin: io::Read` stays in 03 only, and
  6.1's provisional `&self` example still holds, since `&Stdin` is enough.
- **N6:** 03 §6.4: `SendError<T>`'s `Clone`, `Copy`, `PartialEq`, and `Eq`
  hold where `T`'s do; `RecvError` converts into `RecvTimeoutError`, and
  Rust's conversions into `TryRecvError` and `TrySendError` are dropped.
- **N7:** out of scope; reported to the owner.
- **N8:** the six lines are rewrapped.

The core measures 24,989 tokens.
