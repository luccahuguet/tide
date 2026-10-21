# Tide adversarial/consistency review, round 69: process handle API fixes (fx-obr)

## Verdict

**approve** (no blocking findings; five non-blocking notes, all wording)

Round 68's blocking finding is closed. 03 now excludes the ambient
`Command::new` in the same sentence that keeps Rust's API, so 03 no longer
admits what the reference rejects. N1 to N4 and N7 are closed in 03 and
the companion. The fixes add no contradiction among 01, 02, 03,
`reference.md`, and the companion. The Rust API claims I checked are
correct. What remains is wording: one ambiguous sentence in 03's `Stdio`
note, one Tide-only method missing from the receiver list, two loose
sentences in companion 20, the example's missing `Stdio` import, and a
slightly over-broad citation in ROOT-4.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `eb096ea`:
`git diff -- design/`, SHA-256
`0f3707ae3671d1719b9cd77dc17a843530237c876fd41a854529de33496df06d`
(02, 03, `reference.md`, `reference-companion.md`; 110 insertions, 27
deletions). `git diff --check -- design/` passes. `reference.md` did not
change since round 68, so I did not rerun `scripts/count-tokens.sh`; its
stated 24,994 (companion `:14`) stands as round 68 left it.

This is a full pass over the whole diff, not only the fixes. I read:

- `AGENTS.md`, `GOVERNANCE.md`, fx-obr (`br show fx-obr`), and round 68;
- 01, searched for `Command`, `Child`, `Stdio`, public fields, and
  receivers (no hits that bear on this change);
- 02's `Pure` lists (`02:812-841`);
- in 03: §0 (`03:31-61`), §1.1 with all its notes (`03:65-185`), §1.3
  (`03:206-244`), ROOT-1 to ROOT-5 (`03:248-277`), §2.4's heading,
  ADMIT-1 (`03:408-458`), §3.1 to §3.5 (`03:692-827`), and §9.2
  (`03:1405-1412`);
- in the core: 6.1, 6.3, 6.5, 6.6, 6.8, 6.9, and 12.3
  (`reference.md:320-352`, `:379-400`, `:444-470`, `:510-560`,
  `:989-1016`);
- companion 14.2 (`reference-companion.md:80`), 17's node-entry note
  (`:436-445`), and 20 (`:525-568`).

I checked the Rust claims against the stable `std::process` and `std::io`
API:

- `Command`'s builder methods, `output`, `status`, and `spawn` take
  `&mut self`; so do `Child::wait`, `try_wait`, and `kill`.
  `Child::wait_with_output` takes `self` and drops a still-present stdin
  first.
- `Child` has exactly three public fields, `stdin: Option<ChildStdin>`,
  `stdout: Option<ChildStdout>`, and `stderr: Option<ChildStderr>`.
- `Stdio`'s `From` impls are `File`, `ChildStdin`, `ChildStdout`, and
  `ChildStderr` (1.20); `io::Stdout` and `io::Stderr` (1.74); `OwnedFd`
  (Unix) and `OwnedHandle` (Windows) (1.63); and `io::PipeReader` and
  `io::PipeWriter` (1.87). 03's kept and dropped lists together cover all
  of them except `OwnedHandle`, which "It has no other conversion" covers.
  Rust also has the unsafe `FromRawFd for Stdio`; ROOT-4's unsafe-code
  clause already covers forging through it.
- The example type-checks as Rust: `proc.command("sort")` returns a
  temporary `Command` that lives to the end of the statement, so
  `.stdin(..).stdout(..).spawn()` works as with `Command::new`;
  `.stdin(a.stdout.take().unwrap())` passes a `ChildStdout` through
  `impl Into<Stdio>`; `out.stdout == b"a\nb\n"` uses
  `Vec<u8>: PartialEq<&[u8; 4]>`.

## Round 68 items

| Item | Status | Evidence |
| --- | --- | --- |
| B1: 03 keeps Rust's whole `Command` API | **Closed**: "without the ambient `Command::new` (`FX0501`; use `proc.command`)" | `03:160-161`; companion `:544-545` |
| N1: process types not said to be Tide's own; `Stdio` conversions implicit | **Closed**: "Tide std's own types, like `ReadDir`", and the dropped `From<OwnedFd>`, `From<io::PipeReader>`, `From<io::PipeWriter>` are named | `03:153-155`, `03:169-172`; companion `:556-557` |
| N2: receiver rule | **Closed**: capability types listed by name; Rust's receivers where Rust has the method; Tide's additions stated. One method missing (N2 below) | `03:175-181`; companion `:564-567`; core `reference.md:338-339` unchanged, as round 68 allowed |
| N3: Suspends column, `spawn` result | **Closed** | `03:83` |
| N4: ROOT-4 field list | **Closed**: `Option` of a handle and `SimRoot`'s `world` and `faults` named. `sim.node` is a method (`reference.md:515`), so the list is complete | `03:268-271` |
| N5: inherited descriptors versus buffered sinks | **Out of scope** by decision. I found no follow-up bead in `br list`; one should be filed before fx-obr closes, since the finding predates the change and has no other home | — |
| N6: 6.10 trim | **Out of scope** by decision; acceptable as round 68 said | — |
| N7: antecedent, awkward phrase, missing import, wrapping | **Closed**, except that the example imports `io::Write` but not `Stdio` (N4 below) | `03:167-168`; companion `:535`, `:561-563` |

## Bead closure

| Bead point | Status | Evidence |
| --- | --- | --- |
| How `Child`'s pipes are reached under ROOT-4 | **Decided**: Rust's public fields stay; ROOT-4 allows an `Option` of a handle | `03:161-165`, `03:268-269`; companion `:547-550` |
| Pipe ends classified | **Decided**: resource handles, kind `proc`, suspending | `03:36-38`, `03:165-166`, `03:83`; `02:822-823`; companion `:551-552` |
| `Stdio` classified | **Decided**: not `Pure`; made with no capability; closed conversion set | `03:167-174`, `02:827-828`; companion `:553-558` |
| Receivers handles take | **Decided** | `03:175-181`; companion `:564-567` |
| `Command` carries `Proc`'s authority, unlike `OpenOptions` | **Stated** in 03, the core, and the companion | `03:155-160`; `reference.md:335`, `:470`; companion `:527-532` |
| Where `Child`, `Signals`, and `ReadDir` live | **Stated** | `03:152-153`; companion `:561-563` |
| Core under budget | 24,994, unchanged since round 68 | companion `:14` |
| No contradiction among 02, 03, and the pair | **None found** | below |

## Blocking findings

None.

## Non-blocking notes

### N1. 03's `Stdio` sentence lets "and so are" attach to the wrong clause

"Rust's `From<io::Stdout>` and `From<io::Stderr>` are gone, since those
are sink handles (§2.4), not file descriptors, and so are `From<OwnedFd>`,
`From<io::PipeReader>`, and `From<io::PipeWriter>`" (`03:169-172`). The
intended reading is "those three are gone too". The nearest antecedent,
though, is "those are sink handles", and that reading is false of
`OwnedFd` and the `io::pipe` ends. The sentence also gives a reason for the
first pair and none for the second.

**Fix** (03): split it. "Rust's `From<io::Stdout>` and `From<io::Stderr>`
are gone, since those are sink handles (§2.4), not file descriptors. So
are `From<OwnedFd>`, `From<io::PipeReader>`, and `From<io::PipeWriter>`:
a raw descriptor or an `io::pipe` end would reach a child without a
capability."

### N2. The receiver list of Tide-only methods omits `File::sync`

03's Receivers note states the receivers of "the methods Rust lacks":
`read_owned`, `write_owned`, `Signals::recv`, and `File::close`
(`03:179-181`). `File::sync` is also a Tide method; Rust has only
`sync_all` and `sync_data`, the accepted spellings (`03:97-98`). The core
already gives `fn sync(&self)` (`reference.md:396`), so nothing is
contradicted. The list just reads as complete and is not.

**Fix** (03; mirror in companion `:566`): "`read_owned`, `write_owned`,
`File::sync`, and `Signals::recv` take `&self`, and `File::close` takes
`self`."

### N3. Companion 20's two receiver sentences are loose

- **Colon** (`reference-companion.md:544-546`). "keep Rust's methods and
  receivers, except the ambient `Command::new` (`FX0501`): the builder
  methods, ... take `&mut self`". The colon makes the receiver list read
  as an elaboration of the exception. The bullet also omits
  `wait_with_output`, which takes `self` and is the one `Child` method the
  example above it calls.
- **"Only ... promise"** (`:564-567`). "Only the capability types of 6.1
  promise `&self` receivers" is followed at once by handle methods that
  take `&self`. The intended contrast is that capability types take
  `&self` for every method.

**Fix** (companion): end the first sentence at "(`FX0501`)." and continue
"The builder methods, `output`, `status`, `spawn`, `wait`, and `kill`
take `&mut self`, and `wait_with_output` takes `self`." In the Receivers
bullet write "Only the capability types of 6.1 take `&self` for every
method."

### N4. The example imports `io::Write` but not `Stdio`

Round 68 N7 suggested `use std::{io::Write, process::Stdio};`. Only
`io::Write` was added (`reference-companion.md:535`). `Stdio` is not in
Rust's prelude, and Tide's prelude adds only listed items
(`reference.md:991-994`); 12.3's `std::process` row does not list it
either (`:1004`). The companion's other examples import what they use
(`:109-112`, `:188-190`). So the fragment imports one item and leaves out
the other.

**Fix** (companion `:535`): `use std::{io::Write, process::Stdio};`
(40 columns).

### N5. ROOT-4 cites all of `Root`'s fields as capabilities

"A public field holds a capability or handle (`Root`'s fields), ...,
sink configuration (`Root::log`)" (`03:267-269`). `Root::log` is one of
`Root`'s fields (`03:208-210`), so the first parenthesis includes the
field that the third case classifies differently.

**Fix** (03, optional): "(`Root`'s capability fields)".

## Prose and wrapping

Every added prose line is at most 76 columns. Table rows are exempt. The
example's line 538 is exactly 80 columns, which is within the limit. 03
and the companion break a few inline code spans across lines (`` `fn ``
/ `` configure(..)` ``, `` `stdout: `` / `` Option<ChildStdout>` ``,
`` `&mut `` / `` self` ``). These render correctly; 03 already does the
same elsewhere (`03:94-95`). No new short trailing lines.

## Checked and consistent

- **03 §0 and 02.** Both list the same std resource handles in the same
  order, now with the three pipe ends (`03:34-40`, `02:820-826`). 02's
  separate `Stdio` bullet (`02:827-828`) agrees with 03's `Stdio` note.
  03 §3.1 does not name `Stdio` (`03:717-726`), but "Everything else is
  `Pure` exactly when its components are" gives the same answer through
  its private `File` or pipe-end variant.
- **Authority-freedom (§3.2, §3.3).** `Stdio::piped()` can run inside an
  authority-free body and yields a non-`Pure` value. That breaks nothing:
  §3.3's argument says the body cannot create a *handle* without
  `&Tasks`, and a `Stdio` is not a handle; it acts only once a `Command`,
  which carries `Proc`, takes it.
- **ROOT-4 and the types it covers.** No capability or handle type has a
  `Default` impl in Rust (`Command`, `Child`, the pipe ends, `Signals`), so
  the unchanged clause holds. `SimRoot`'s other members, `node`,
  `elapsed`, `records`, and `trace`, are methods (`03:1408-1412`,
  `reference.md:515`), so `world` and `faults` are its only non-capability
  public fields. `NodeRoot` has only `Root`'s capability fields
  (`03:1471-1472`).
- **Receivers across the pair.** 03 names the capability types
  (`03:175-176`), which matches 03 §3.1's list minus `Root`. The core's
  "Their methods take `&self` (provisional; handles keep Rust's
  receivers)" (`reference.md:338-339`) agrees with it, and 6.3 states
  `File::close(self)` (`:398`) as 03 does. `Stdin`'s `&self` `lines()`
  differs from Rust's `lines(self)`, but that is an older decision for a
  capability type (`03:127-130`) and is not affected by this change.
- **ADMIT-1.** "Tide std's own types" (`03:153-155`) keeps `spawn`,
  `Child::wait`, and pipe I/O out of the ban on I/O methods of re-exported
  std types (`03:440`), as the Suspends column needs.
- **`FX0501`.** 03 (`03:160-161`), the core (`reference.md:547-550`), and
  companion 14.2 (`:80`) and 20 (`:544-545`) all exclude `Command::new` and
  point to `proc.command`.
- **Checkpoints.** Companion 20 says pipe-end reads and writes "suspend
  and are checkpoints" (`:551-552`); 03 §0 makes every capability operation
  that may suspend a checkpoint (`03:56-58`), and §1.1's table preamble
  says the same (`03:67-68`).
- **Sinks.** 03's `Stdio` note cites §2.4, which is the write-only sinks
  section (`03:349`); the companion cites the core's 6.5, which is too
  (`reference.md:444`).
- **Companion intro.** "how process handles work" and "17–20" match the
  new section (`reference-companion.md:8-11`).

## Ready to merge

**Yes.** No blocking findings remain. N1 to N5 are wording fixes the
author can take in the same pass; if any lands, it is a text-only change
for the governance loop. Before closing fx-obr, file the follow-up bead
for round 68's N5 (inherited descriptors versus the buffered `out` and
`err` sinks), since nothing tracks it yet.
