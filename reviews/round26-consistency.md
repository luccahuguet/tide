# Tide consistency review, round 26: confirmation of the round-25 answers

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `417fbd5`
("Decide v0 contract details from the E1 dialect round (fx-eky)"):
`git diff -- design/`, which changes `01-design.md` (D15),
`03-authority-concurrency.md` (§0, §1.1, §1.2, TASK-2, TASK-3, §6.4, §9.2,
§9.3, §9.5), `reference.md` (1, 6.1, 6.3, 6.7, 6.8, 7.2, 7.3, 13), and
`reference-companion.md` (14.1, 17, the recorded count). It implements bead
fx-i5p, whose comment of 2026-10-20 records the owner's decisions on G3, G4,
G5, G6, and G9. Scope, as asked:

- whether each round-25 item (B1, B2, N1–N8) is closed as its Resolution
  claims, including the parts it declines and whether the reasons hold;
- whether the changed lines contradict 01, 02, 03, 04, the README, the rest
  of the reference pair, or the owner's decisions, including the Rust facts
  the new text relies on;
- the core's token count against the companion's figure and D14's budget.

Rule preservation beyond the changed lines and their neighbours was not
re-audited; round 25 did that for the first pass of this change.

## Round-25 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | Companion 14.1 splits the row (`companion:47–48`): `Err(SendError(v))` keeps E0532, and `Err(RecvError)` is listed as a catch-all binding with only `non_snake_case`. 03 §6.4 adds "A bare `Err(RecvError)` still compiles, as a binding that catches every error" (`03:844–846`). Rechecked with rustc 1.96.0, edition 2024: a local `enum RecvError { Disconnected, Cancelled }` and `Err(RecvError) => break` builds with one `non_snake_case` warning and takes the arm for `Cancelled`. |
| B2 | **Resolved**, with a residual wording gap (N2) | `OsStr` and `OsString` join reference 1's list (`reference.md:40`). 6.3's comment adds "(a DirEntry, like a File, carries the Fs's authority)" (`:384–385`), and "as std::fs" brings in `DirEntry`'s four portable methods. 03 §1.1 gains a `ReadDir`, `DirEntry` row (`03:72`). 6.9 is unchanged (`reference.md:557`), so the Resolution's "no longer reads against it" rests only on the parenthetical. |
| N1 | **Resolved** | `reference.md:534–536`: "each file operation takes up to 1 ms (provisional)". The fix added a 93-column line (N5). |
| N2 | **Resolved** | 03 drops the `reqwest-retry` claim (`03:120–126`). It gives its own reason ("the failures a later identical request can escape") and lists `Denied`, `Dns`, and `Io` as not transient. This matches the owner's G5 set and reference 6.7 (`reference.md:504–506`). |
| N3 | **Partly resolved**; the reason for not asking the owner **holds** | 03 §9.5 (`03:1168–1171`) and companion 17 (`companion:445–447`) add "a change to an entry survives only if the change that created the entry does". That closes the rename-after-create case. It does not close the other case round 25 named, a file created inside a new, unsynced subdirectory (N3 below). Narrowing to reachable states does not change the owner's "otherwise unspecified", so no owner question was needed. |
| N4 | **Resolved** | `03:108–109`: "As Rust's does on Unix, a `ReadDir` ends after yielding its first error". This matches `library/std/src/sys/fs/unix.rs:865–881` (`end_of_stream` set on either error or end). `03:117–119`: `lines()` ends after a cancellation error. Rust's `Lines` has no such stop, so this is a departure, which the text shows by giving its reason. |
| N5 | **Resolved**; both declined parts **decline with reasons that hold only in part** (N1) | `recv_deadline` is a checkpoint in 03 §1.2 (`03:145`) and reference 7.3 (`reference.md:692–693`). Companion 17's intro lists its new parts (`companion:385–386`). Reference 13 points to companion 17 for replay (`reference.md:1062`). The pipeline-stage advice is a companion 14.1 row (`companion:49`). §0's `ReadDir` and `DirEntry` now have a §1.1 row. The 6.1 example stays, and that reason holds: the "(provisional)" marker covers the general rule that every capability method takes `&self`, which no normative text decides. The reason does not cover `&Stdin: io::Read`, which stays in 03 only (N1). |
| N6 | **Resolved**, with an imprecision in the new wording (N4) | `03:835–839` drops the `From` conversions into `TryRecvError` and `TrySendError`, keeps `RecvError` into `RecvTimeoutError`, and conditions `SendError<T>`'s derives on `T`. |
| N7 | **Out of scope**, as reported | No change; it predates fx-i5p. |
| N8 | **Resolved** for the six named lines | None of `03:35`, `:767`, `:813`, `reference.md:505`, `:514`, `:695` is still over 80 columns after rewrapping. The answers added `reference.md:536` (93 columns), plus a ragged break at `03:839` (N5). |

**Owner decisions.** Every point in the fx-i5p comment is carried out in
03, and none is contradicted:

- G3: §6.4 (`03:816–851`), TASK-2 (`03:763–764`), TASK-3 (`03:773`);
- G4: §1.1 (`03:97–101`), §9.5 (`03:1162–1176`), 01 D15 (`01:846–848`),
  reference 6.3's atomic replace (`reference.md:418–423`);
- G5: §1.1 Network errors (`03:120–126`), §9.3 (`03:1111–1116`);
- G6: §0 (`03:34`), §1.1 row and Directory listings (`03:72`, `:102–111`);
- G9: §1.1 Standard input (`03:112–119`), §9.2 (`03:1094–1102`).

The reference pair states all of them except `&Stdin: io::Read` (N1).

**Rust facts rechecked.** Against rust-src of rustc 1.96.0 on this machine:

- `mpsc.rs:406–418`, `:444`: `SendError<T>` derives `PartialEq, Eq, Clone,
  Copy`, and `RecvError` and `RecvTimeoutError` also derive `Debug`.
  `SendError<T>`'s `Debug`, `Display`, and `Error` are `impl<T>`, with no
  bound on `T` (`:1143`, `:1150`, `:1157`). This bears on N4.
- `mpsc.rs:1185`, `:1222`, `:1249`: the three `From` impls are as §6.4
  describes.
- `unix.rs:865–881`: `ReadDir::next` sets `end_of_stream` on error.
- A probe with a stand-in `Stdin` that has an inherent `read_to_string(&self,
  ..)` and `Read` for `Stdin` and `&Stdin`, under `use std::io::Read;`,
  built and resolved both calls to the inherent method. rustc reported
  **`unused import: std::io::Read`**. This is B1 below.

**Leftovers.** I grepped `design/`, `README.md`, and `GOVERNANCE.md` for
`RecvError`, `SendError`, `RecvTimeoutError`, `DirEntry`, `ReadDir`,
`OsStr`, `is_transient`, "iterates until", `recv_deadline`, "torn",
"journaling", "rename", "6.8", and "companion 17". No stale claim remains
outside the item below and the notes. The README and 02 and 04 say nothing
about these items. 01 mentions `Stdin` only in D7's capability list
(`01:411`).

**Budget.** `scripts/count-tokens.sh design/reference.md </dev/null` printed
**24,989** and exited 0. That equals the companion's recorded figure
(`companion:12`) and is 11 tokens under D14's 25,000.

## New blocking problems

### B1. "Code that imports `io::Read` keeps working" is false: the import becomes an `FX0913` error

03 §1.1's new Standard input bullet (`03:112–119`) says: "`Stdin` and
`&Stdin` also implement `io::Read`, as in Rust; inherent methods win, so
code that imports `io::Read` keeps working."

- **Inherent methods do win.** The call is not ambiguous.
- **The import is then unused.** rustc counts a trait import as used only
  when method resolution picks one of the trait's methods. Rust code reads
  stdin by writing `use std::io::Read;` and then `stdin.read_to_string(&mut
  s)`. In Tide that call resolves to the inherent method, so the import is
  unused. rustc 1.96.0 warns `unused import: std::io::Read` on the probe
  above.
- **Tide makes that an error.** Reference 12 says "Unused imports are
  errors (`FX0913`, safe fix)" (`reference.md:984`), and the companion
  repeats it (`companion:59`).

So the code does not keep working. It fails to check until `tide fix`
deletes the import. The import survives only if the module calls some other
`Read` method, for example on a `File`. This is the question the E1 writer
raised (`trials/e1-dialect/notes.md`, item 7: "that import is an
unused-import error (`FX0913`)"), and the new normative text answers it
wrongly. This is the same kind of false-diagnostic claim as round 25's B1,
and an agent will act on it.

**Fix** (03 only, no core tokens): "…inherent methods win, so no call is
ambiguous; an `io::Read` import used only for these calls is unused
(`FX0913`, safe fix)." Optionally, add a companion 14.1 row: "`use
std::io::Read;` for `stdin.read_to_string` | delete it: `Stdin`'s reads are
inherent (6.1) | `FX0913` safe". That row would also fix N1.

## Non-blocking notes

### N1. `&Stdin: io::Read` is now absent from the reference pair, and the change removed the text that implied it

Round 25's N5 asked for `&Stdin: io::Read` in the reference pair, and the
Resolution declines it without a reason. The only reason it gives ("since
`&Stdin` is enough") answers the 6.1 example, not this point.

Before this change, reference 6.1's row read "as `io::Stdin`'s", which
carried Rust's `Read` impls with it. It now reads "inherent, receivers
`&self`" (`reference.md:334`). Nothing in the pair now says that `Stdin`
is `io::Read` at all. A reader therefore cannot learn that
`BufReader::new(stdin)` or a `fn parse(r: impl Read)` taking `stdin`
works. `File` and `TcpStream` both state theirs ("also for `&File`",
`reference.md:395`; `:509`). The owner decided this point (fx-i5p
comment, G9), and the bead's acceptance asks that "the reference or
companion states each decision".

**Fix** (no core tokens): a sentence in the companion, or the 14.1 row
proposed under B1, extended: "`Stdin` and `&Stdin` are `io::Read`, as in
Rust."

### N2. 6.9 still bans "`metadata`" on "std types", without D7's "re-exported"

The B2 answer relies on 6.3's parenthetical alone. 6.9 still reads "I/O
methods on std types (`Path::exists`, `metadata`; use `fs`)"
(`reference.md:557`). That is the sentence that made the E1 writer avoid
`DirEntry::metadata` and `file_type` (`trials/e1-dialect/notes.md`,
item 10). `DirEntry` is reached through `read_dir` "as std::fs"
(`reference.md:383–385`), so a reader can still take it for a std type.
01 D7 words the ban "I/O methods on re-exported std types" (`01:427`).
03 now relies on that word: "These are Tide std's own types, so ADMIT-1's
ban on I/O methods of re-exported std types does not reach them"
(`03:109–110`). The core lacks it.

**Fix** (about 2–3 core tokens, within the 11 of headroom): in 6.9, write
"I/O methods on re-exported std types", as D7 does, or round 25's
`Path::metadata`. Recount after the edit.

### N3. The dependency rule does not cover an entry inside a lost new directory

The adopted rule is "a change to an entry survives only if the change that
created the entry does" (`03:1169–1171`, `companion:446–447`). It covers
`rename("a", "b")` after an unsynced `create("a")`. It does not cover the
second case round 25 named. Take a `create("d/x")`, or a `write_synced`
that syncs `x`'s entry in `d`, inside a directory `d` whose own creation is
unsynced. The change to `d/x` is a change to entry `x`, created by itself,
so the rule lets it survive while `d`'s entry is lost. 03 introduces the
rule with a colon, as the definition of "one a file system could reach"
(`03:1169–1170`), so it reads as complete. In practice the state is
unreachable: `x` simply has no path. The simulator still has to pick that
reading itself.

**Fix** (03 §9.5 and companion 17, no core tokens): "…only if the changes
that created the entry and the directory holding it do", or round 25's
"only with the unsynced changes that created what it names".

### N4. "Which for `SendError<T>` hold where `T`'s do" also conditions `Debug`, `Display`, and `Error`

`03:835–837` reads: "Each has `is_cancelled()` and Rust's `Debug`,
`Display`, `Error`, `Clone`, `Copy`, `PartialEq`, and `Eq`, which for
`SendError<T>` hold where `T`'s do." The relative clause takes in the whole
list. In Rust, only the derived `Clone`, `Copy`, `PartialEq`, and `Eq`
depend on `T`. `Debug` (which prints `SendError { .. }`), `Display`, and
`Error` are `impl<T>` with no bound (`mpsc.rs:1143–1157`). Read literally,
03 makes `SendError<T>` `Error` only where `T: Error`, which would break
`?` into a `Box<dyn Error>` for most payloads.

**Fix:** "…`Debug`, `Display`, and `Error`, and `Clone`, `Copy`,
`PartialEq`, and `Eq` where `T`'s are".

### N5. Line breaks

- `reference.md:536` runs to 93 columns ("`Net::get` 2–20 ms plus the
  reply's `after(d)` (companion 17). Bound `sim.elapsed()` by a"). N1's
  answer added it.
- `03:839` ends after "`Cancelled` case. `Sender::send` never". The N6
  insertion left the paragraph unreflowed.
- `03:832`, a line in a code block, runs to 82 columns. This is acceptable
  in code.

### N6. Pre-existing, out of scope: `remove_dir_all`

Reference 6.3 lists `remove_dir_all` among `Fs`'s std-like methods
(`reference.md:384`). 03 §1.1's `Fs` row omits it (`03:69`), and §9.5's list
of entry changes (`create`, `rename`, `remove_file`, new subdirectory,
`03:1164–1165`) does not say how its several removals behave in a crash.
This predates fx-i5p. It is reported here, not fixed.

## Ready to merge

**No.** B1 is a false claim in the normative chapter. It contradicts
reference 12's `FX0913`, on the exact question E1 raised. The fix is one
clause in 03 and costs no core tokens. With B1 fixed, the change is ready:

- every round-25 item is closed or declined with a sound reason, except
  N1's `&Stdin: io::Read`, which is free to add to the companion;
- the core measures 24,989 tokens, 11 under D14's budget;
- no other changed line contradicts 01, 02, 04, the README, the reference
  pair, or the owner's decisions.

N2 costs about 3 core tokens and closes B2's residue. N3 to N5 cost none.

Validation: this was a read-only review of the working tree at `417fbd5`,
with `.beads/issues.jsonl` and four `design/` files modified and
`reviews/round25-consistency.md` untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-i5p and its comment, via `br show` and `br comments`;
  - round 25 with its Resolution, and round 24 for format;
  - in 03: §0, §1.1, §1.2, TASK-2, TASK-3, §6.4, §9.2, §9.3, and §9.5;
  - 01 D7 (`01:411–433`) and D15;
  - reference 1, 6.1, 6.3, 6.7, 6.8, 6.9, 7.2, 7.3, 12 (`FX0913`), and 13;
  - companion 14.1 and 17;
  - `trials/e1-dialect/notes.md`, items 7, 10, and 11, and
    `trials/e1-dialect/grading.md`'s C3 row.
- **Rust:** read rust-src 1.96.0 (`sync/mpsc.rs`, `sys/fs/unix.rs`,
  `io/mod.rs`). Compiled two probes with `rustc --edition 2024` in the
  scratchpad:
  - inherent `Stdin` reads beside `Read` for `Stdin` and `&Stdin`, under
    `use std::io::Read`: an unused-import warning;
  - `Err(RecvError)` against an enum `RecvError`: it builds with a
    `non_snake_case` warning and catches `Cancelled`.
- **Line lengths:** checked every added non-table line of the diff for more
  than 80 columns.
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null` was
  run once, with a 400 s timeout. It printed `24989` and exited 0.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the working tree before round 27.

- **B1:** 03 §1.1 now says no call is ambiguous because inherent methods
  win, but an `io::Read` import used only for these calls is unused
  (`FX0913`, safe fix). Companion 14.1 has the matching row.
- **N1:** that companion row also says `Stdin` and `&Stdin` stay
  `io::Read` for generic readers.
- **N2:** reference 6.9 reads "I/O methods on re-exported std types", as
  D7 does.
- **N3:** 03 §9.5 and companion 17: a change survives only if the changes
  that created its entry and every directory above it do.
- **N4:** 03 §6.4: as in Rust, `SendError<T>` has `Clone`, `Copy`,
  `PartialEq`, and `Eq` only where `T` does. `Debug`, `Display`, and
  `Error` are unconditional.
- **N5:** the lines are rewrapped.
- **N6:** predates this change and is out of scope; reported to the owner.

The core measures 24,993 tokens.
