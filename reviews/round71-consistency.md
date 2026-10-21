# Tide adversarial/consistency review, round 71: Codex fixes for process handles (fx-obr)

## Verdict

**approve** (no blocking findings; four non-blocking notes, all wording or
precision)

Round 70's B1 is closed. `Child::wait_with_output` now takes `&mut self`,
so it consumes nothing that TASK-3 would have to return in its
cancellation error. A cancelled call leaves the `Child` with the caller,
who can `kill` and `wait` it under `shield`. The decision is correct and
is stated in 03, the companion, and, by pointer, the core. No file in
`design/` still says the method takes `self` or that handles keep all of
Rust's receivers. Round 69's N1 to N5 are closed. The fixes add no
contradiction among 01, 02, 03, `reference.md`, and the companion. What
remains is one sentence in 03 that still lists one exception where there
are two, an unstated pipe-end state after a cancelled call, an
over-broad "so" clause in the new `Stdio` reason, and one unfilled wrap.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `eb096ea`:
`git diff -- design/`, SHA-256
`720553ea4aece8962d2fd8c40e6697cf0b094ec79c15a5a83a3d2cb57324828e`
(02, 03, `reference.md`, `reference-companion.md`; 120 insertions, 27
deletions). I recomputed the hash. `git diff --check -- design/` passes.
I did not run `scripts/count-tokens.sh`, as instructed; the companion's
stated 24,993 (`reference-companion.md:14`) is taken as measured.

This is a narrow confirmation round. I read:

- `AGENTS.md`, `GOVERNANCE.md`'s supervisor section, fx-obr
  (`br --no-db show fx-obr`), round 69, and round 70;
- the whole diff;
- 01 D8 (`01:509-521`);
- in 03: §1.1's preamble and table (`03:65-86`), the Process handles,
  `Stdio`, and Receivers notes (`03:152-188`), ROOT-4 (`03:272-282`),
  TASK-3 to TASK-6 (`03:1090-1119`), and JOIN-1 and JOIN-2
  (`03:1123-1133`);
- in the core: 6.1 (`reference.md:320-345`), 6.3's `File` signatures
  (`:396-398`), 6.6's not-`Pure` list (`:467-472`), and 7.2
  (`:659-669`);
- companion 20 (`reference-companion.md:525-570`);
- a grep of `design/` and `trials/` for `wait_with_output`, "Rust's
  receivers", "keep Rust's", and "Rust's builder".

## Round 70 item

| Item | Status | Evidence |
| --- | --- | --- |
| B1: consuming `wait_with_output(self)` is a checkpoint, and an `io::Error` cannot carry the `Child` | **Closed**: the method takes `&mut self`; the reason cites TASK-3 and `io::Error`; a cancelled call leaves the `Child` for `kill` and `wait` under `shield` | `03:179-185`; companion `:545-548`, `:567-570`; core `reference.md:338` points to companion 20 |

How B1 closes against each rule Codex cited:

- **TASK-3** (`03:1090-1094`) and **7.2** (`reference.md:666-668`): only
  an operation that consumes a value must return it inside the error.
  With `&mut self` the caller keeps the `Child`, so `io::Result<Output>`
  needs no payload.
- **D8** (`01:517-520`): "operations that consume a value return it
  inside the error". This holds for the same reason.
- **Suspends column** (`03:83`): it still lists `wait_with_output`, and
  the preamble (`03:67-69`) still makes it a checkpoint. The method stays
  a checkpoint and is not shielded, so 03 says nothing that contradicts
  the column.
- **Shield and joins**: TASK-6 (`03:1115-1119`) lets `wait` run under
  `shield` despite cancellation. `kill` is not in the Suspends column, so
  it is no checkpoint and needs no shield; "to `kill` and `wait` under
  `shield`" is accurate. JOIN-1 and JOIN-2 concern scope and `Task`
  joins and do not interact with the change.

## The `&mut self` decision

It is correct and the smaller of the two fixes Codex offered. A non-`Pure`
error type carrying the `Child` would add a type to learn and a
`match` at every call. The cost is a porting edit: a Rust program that
binds `let child = ...spawn()?` without `mut` and then calls
`wait_with_output` fails with E0596, and rustc's own help suggests `mut
child`. 03 states that cost (`03:185-186`). E0596 is the right code
("cannot borrow as mutable, as it is not declared as mutable").
Chained calls on a temporary,
`proc.command(..).spawn()?.wait_with_output()?`, still compile, because
autoref takes `&mut` of a temporary.

"The one exception" (`03:180`) holds for `Command` and `Child`. Their
only consuming Rust method is `wait_with_output`; `wait`, `try_wait`,
`kill`, and the builder methods take `&mut self`, and `id` takes
`&self`.

Consistency across `design/`: the grep finds no remaining claim that
`wait_with_output` takes `self`. The core no longer says "handles keep
Rust's receivers" (`reference.md:338-339` now points to companion 20).
Companion 20 now says "with two exceptions" (`:544-545`), and its
Receivers bullet says "except `wait_with_output`" (`:568`). The only
place that still reads as one exception is 03's Process handles note
(N1 below). The `trials/` hits ("Rust's builder") are frozen trial-arm
references and are out of scope.

## Companion example

`let mut child = proc.command("sort")...spawn()?;` (`:536-537`). The
binding is `mut`, which both `child.stdin.take()` and the `&mut self`
`wait_with_output` need. `let mut input` is needed for `write_all`. The
import line is `use std::{io::Write, process::Stdio};` (`:535`). The
fragment still type-checks as Rust, with the method's receiver changed.

## Round 69 items

| Item | Status | Evidence |
| --- | --- | --- |
| N1: "and so are" attaches to the wrong clause | **Closed**: split into two sentences, and the second pair has its own reason. The new reason over-reaches (N3 below) | `03:169-173` |
| N2: `File::sync` missing from the receiver list | **Closed** in 03 and the companion | `03:186-188`; companion `:569-570` |
| N3: companion 20 colon; "Only ... promise" | **Closed**: "with two exceptions" plus a separate sentence for the `&mut self` list; "Only the capability types of 6.1 take `&self` for every method" | companion `:544-549`, `:567-568` |
| N4: example lacks the `Stdio` import | **Closed** | companion `:535` |
| N5: ROOT-4 cites all of `Root`'s fields | **Closed**: "(`Root`'s capability fields)" | `03:275` |
| Round 68 N5 follow-up bead (inherited descriptors versus buffered sinks) | **Still not filed**: `br --no-db list` shows no such bead. Not blocking for this diff; file it before fx-obr closes, as round 69 said | — |

## Blocking findings

None.

## Non-blocking notes

### N1. 03's Process handles note still names one exception to Rust's API

"`Command` and `Child` otherwise keep Rust's API, without the ambient
`Command::new` (`FX0501`; use `proc.command`)" (`03:160-161`). Nineteen
lines later the Receivers note adds a second departure,
`wait_with_output(&mut self)`. The companion already says "with two
exceptions" (`:544-545`). The two notes are in the same list, so nothing
is contradicted, but the first reads as complete.

**Fix** (03): "`Command` and `Child` otherwise keep Rust's API, without
the ambient `Command::new` (`FX0501`; use `proc.command`) and with
`wait_with_output`'s receiver below."

### N2. A cancelled `wait_with_output` leaves the pipe ends unspecified

Rust's `wait_with_output` closes `stdin` and takes `stdout` and `stderr`
out of the `Child` before reading them. With `&mut self`, those fields
are moved out of a borrowed `Child`. 03 does not say what a cancelled call
leaves in them. Either answer is sound, and TASK-3 does not require one,
because the pipe ends are not arguments to the call. But the answer
decides what the caller can do next: if the call keeps the pipe ends in
place, a `wait` without `kill` can block forever on a child stalled on a
full pipe. If it drops them, the output read so far is lost.

**Fix** (03 Receivers note, after "under `shield`"): "As in Rust, the
call first closes `stdin` and takes `stdout` and `stderr`, so after a
cancelled call those fields are `None` and the output read so far is
lost." This assumes the Rust behavior is the one intended. If not, state
the other choice instead.

### N3. The new `Stdio` reason is broader than the conversion set

"`From<OwnedFd>`, `From<io::PipeReader>`, and `From<io::PipeWriter>` are
gone too, so a `Stdio` redirects only to a handle and never to a raw
descriptor" (`03:171-173`). `Stdio::inherit()`, kept two lines earlier,
redirects the child to the parent's own descriptors 0 to 2, which are
not handles. `null()` and `piped()` are not handles either. The sentence
also calls `OwnedFd` a "raw" descriptor, which in Rust's terms is
`RawFd`, and an `io::pipe` end is neither. The intended claim is about
the `From` conversions only. Round 68's out-of-scope N5 covers the
`inherit()` interaction itself, which is not reopened here.

**Fix** (03): "... are gone too: a `Stdio` wraps an existing descriptor
only when a handle carries it, never through an `OwnedFd` or an
`io::pipe` end."

### N4. One unfilled wrap in companion 20

The Rust's API bullet ends a sentence on a 40-column line,
"`wait`, and `kill` take `&mut self`." (`:549`), and continues the same
bullet on the next line. It renders correctly as a soft break, but the
paragraph is not filled like its neighbors.

**Fix** (companion): reflow `:548-553` so the bullet fills to about 76
columns.

### Process note

AGENTS.md's Decisions rule asks that a technical decision be recorded on
the bead as well as in its owning design document. 03 records the
`&mut self` decision and its reasons. fx-obr has no comment that
records it. Add one when the bead's state is next updated.

## Prose and wrapping

Every added prose line is at most 77 columns. Example lines 538 and 540
are 80 and 77 columns, within the limit. Table rows are exempt. Apart
from N4, I found no new short trailing lines.

## Checked and consistent

- **Core count line.** The companion says 24,993 (`:14`). The core's
  only change since round 69 is the 6.1 bullet, which is shorter than
  the "handles keep Rust's receivers" wording. A two-token drop from
  24,995 is plausible. Not recounted.
- **Core 6.1 bullet.** "Their methods take `&self` (provisional;
  handles' receivers: companion 20)" (`reference.md:338-339`). "Their"
  refers to the capability types in the table above. The bullet no longer
  claims anything about handles that 03 contradicts.
- **02.** The resource-handle and `Stdio` bullets (`02:819-828`) are
  unchanged since round 69 and agree with 03.
- **ROOT-4.** `Child`'s private field is Rust's `handle`; companion 20's
  "Each handle still has a private field" is true of `Child` and the
  pipe ends.
- **6.10 trim.** The `--thread` sentence (`reference.md:564-566`) no
  longer says "shortest path from a holder". That change predates this
  round and was settled as round 68 N6; 03 §4.4 (`03:912-920`) still
  owns the rule.

## Ready to merge

**Yes**, after the governance loop's narrow Codex re-check of the
round-70 fixes. No blocking findings remain. N1 to N4 are wording fixes.
If any lands, it is a text-only change for the governance loop. Before
closing fx-obr, file the round 68 N5 follow-up bead and record the
`&mut self` decision on fx-obr.
