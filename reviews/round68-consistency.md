# Tide adversarial/consistency review, round 68: process handle API (fx-obr)

## Verdict

**approve with fixes** (one blocking finding, fixed by one clause in 03;
seven non-blocking notes)

The change settles what rounds 32 (N1, N2) and 33 (N3) left open. `Child`
keeps Rust's public pipe fields, and ROOT-4 is narrowed to allow them. The
pipe ends are resource handles. `Stdio` is not `Pure` but takes no
capability to make. `Command` is a handle, not inert data like
`OpenOptions`. The new ROOT-4 also repairs an older contradiction: the
previous "no public fields" was false of `Root` itself (`03:200-201`,
`reference.md:364-365`). The one blocking point is that 03, which owns the
decision, now says `Command` keeps "Rust's API" without excluding the
ambient `Command::new`, which the reference bans.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `eb096ea`:
`git diff -- design/`, SHA-256
`ce0fe514bb4fdb1bb1f98164e9d20cd529f8a4427418f6a581ab2df4641a5457`
(02, 03, `reference.md`, `reference-companion.md`; 96 insertions, 27
deletions). `git diff --check -- design/` passes. I did not rerun
`scripts/count-tokens.sh`, so the stated 24,994 is unverified.

I read:

- `AGENTS.md`, `GOVERNANCE.md`, and fx-obr (`br show fx-obr`);
- round 32 N1–N4, round 33 N3–N4, and round 66 (for the format);
- 01 D6 (`01:380-394`) and D7 (`01:407-445`);
- 02's `Pure` lists (`02:812-841`);
- in 03: §0 (`03:31-60`), §1.1 including the new notes (`03:65-177`),
  §1.3 (`03:198-201`), ROOT-1 to ROOT-5 (`03:236-269`), SINK-1 to SINK-9
  (`03:340-390`), ADMIT-1 (`03:400-440`), §4.1 (`03:822-862`), §4.4
  (`03:896-910`), TASK-6 (`03:1099-1103`), RUST-2 (`03:1640-1645`), §9.2
  (`03:1397-1416`), and §9.5 (`03:1458-1466`);
- in the core: 6.1–6.3, 6.5, 6.6, 6.8–6.10, 12.3 (`reference.md:320-575`,
  `:990-1015`);
- companion 14.2 (`reference-companion.md:66-82`), 17's node-entry note
  (`:436-442`), and the new 20 (`:525-561`).

I checked the Rust claims against the stable `std::process` API: `Child`
has exactly three public fields, `stdin: Option<ChildStdin>`,
`stdout: Option<ChildStdout>`, and `stderr: Option<ChildStderr>`, plus a
private handle. `Command`'s builder methods, `output`, `status`, and
`spawn` take `&mut self`, and so do `Child::wait`, `try_wait`, and `kill`.
`wait_with_output` takes `self` and closes stdin first. `Command::stdin`,
`stdout`, and `stderr` take `impl Into<Stdio>`. `Stdio` has `From` impls
for `File`, `ChildStdin`, `ChildStdout`, and `ChildStderr` (Rust 1.20) and
for `io::Stdout` and `io::Stderr` (1.74). `std::fs::File` has no `close`
method.

## Bead closure

| Bead point | Status | Evidence |
| --- | --- | --- |
| How `Child`'s pipes are reached under ROOT-4 | **Decided**: Rust's public fields stay, and ROOT-4 is narrowed | `03:158-161`, `03:257-262`; companion `:543-548` |
| Pipe ends classified | **Decided**: resource handles, kind `proc`, suspending | `03:36-38`, `03:161-163`, `03:83`; `02:822-823`; companion `:549-550` |
| `Stdio` classified | **Decided**: not `Pure`; made with no capability | `03:163-169`, `02:827-828`; companion `:551-555` |
| Receivers handles take | **Partly decided** (N2) | `03:170-173`; `reference.md:338-339`; companion `:561-562` |
| `Command` carries `Proc`'s authority, unlike `OpenOptions` | **Stated** in 03, the core, and the companion | `03:153-158`; `reference.md:335`, `:467`; companion `:527-532` |
| Where `Child`, `Signals`, and `ReadDir` live | **Stated** in 03 (process types) and the companion (all) | `03:152-153`; companion `:558-560` |
| Core under budget | Stated as 24,994; not rechecked | companion `:14` |
| No contradiction among 02, 03, and the pair | **One** (B1) | below |

## Blocking findings

### B1. 03 keeps Rust's whole `Command` API, so it does not exclude `Command::new`

03 now says "`Command` and `Child` otherwise keep Rust's API"
(`03:158-159`). Rust's `Command` API includes `Command::new(prog)`, the
ambient constructor that starts a process with no `Proc`. The reference
bans it: "`Command::new`, `process::exit`, ... are `FX0501`"
(`reference.md:547-550`), and companion 14.2 maps it to
`proc.command("git")` (`reference-companion.md:80`). No sentence in 03
names `Command::new` or `FX0501` (`grep -n 'Command::new\|0501'` finds
none in 03). The owner document therefore literally admits what the
reference rejects.

- **Why "otherwise" does not save it.** "Otherwise" scopes out the
  preceding authority sentence, not the constructor. The same note gives
  the exact contrast case: `OpenOptions` is "Rust's builder without its
  ambient `open`" (`03:96`). That is the form the `Command` sentence
  needs. Round 32's N2 was about precisely this reading by analogy.
- **Acceptance.** fx-obr's third bullet asks for no contradiction between
  03 and the reference pair.

**Fix** (03 only, no core tokens): "`Command` and `Child` otherwise keep
Rust's API, without the ambient `Command::new` (`FX0501`; use
`proc.command`), including `Child`'s public fields ...".

## Non-blocking notes

### N1. 03 does not say the process types are Tide std's own types

ADMIT-1 bans "I/O methods on re-exported std types" (`03:431`). It also
says that std's inherent methods stay callable on the types Tide's std
re-exports (`03:403`). For directory listings, 03 says outright: "These are
Tide std's own types, so ADMIT-1's ban on I/O methods of re-exported std
types does not reach them" (`03:124-125`). The new note says only that
`Command`, `Child`, `Stdio`, and the pipe ends "keep Rust's API" and "live
in `std::process`" (`03:152-159`). A reader who takes "keep Rust's API"
to mean "re-exported" would conclude that ADMIT-1 rejects `spawn`,
`Child::wait`, and pipe reads. That conflicts with the Suspends column
(`03:83`).

They must be Tide's own types in any case:

- a `Command` must carry its `Proc`'s `only([...])` policy to check it at
  `spawn`;
- `Stdio::from(file)` takes Tide's `File`, not Rust's.

The same omission leaves the closed set of `Stdio` conversions implicit.
Rust also has `From<OwnedFd>` (Unix, 1.63) and `From<io::PipeReader>` and
`From<io::PipeWriter>` (1.87). The note's "keeps ... drops ..." list
names neither, and nothing in `design/` mentions `OwnedFd` or `io::pipe`.
ADMIT-1 is an allowlist, so they fail by default. Still, an unnamed
`io::pipe()` would be a cross-task channel made without `&Tasks`.

**Fix** (03, one sentence): "These are Tide std's own types, like
`ReadDir`, so ADMIT-1's ban on I/O methods of re-exported std types does
not reach them, and `Stdio` has no other conversions (no `From<OwnedFd>`,
no `io::pipe`)."

### N2. The receiver rule cites a Tide-only method as Rust's and leaves the others open

- **`File::close`.** 03 says "Resource handles keep Rust's receivers:
  ..., and `File::close` takes `self`" (`03:171-173`). The companion says
  "Handles keep Rust's: `File::close` takes `self`" (`:561-562`). Rust's
  `File` has no `close`. It is Tide's addition (`reference.md:396`,
  `03:1103`), so it is not an example of a kept Rust receiver. The core's
  "handles keep Rust's receivers" (`reference.md:338-339`) has the same
  scope.
- **Handle methods that Rust lacks.** "Keep Rust's receivers" decides
  nothing for `Signals::recv` (no `std` counterpart) or for
  `File`/`TcpStream`'s `read_owned` and `write_owned` (`03:75`, `:78`,
  `:144`). The bead's Scope asks which receivers handles take, and it
  names `Signals`.
- **"Capability types in this table."** 03's Receivers note opens with
  "The methods of the capability types in this table take `&self`"
  (`03:170-171`). The §1.1 table also has rows for `File`, `ReadDir`,
  `DirEntry`, and the sockets (`03:75-78`), and 03 §0 calls those
  capabilities too (`03:32-40`). Read literally, `File::close(self)`
  then contradicts the note's own first sentence. The companion avoids
  this ("the capability types of 6.1", `:561`).

**Fix** (03; mirror in companion 20): "The standard capability types
(`Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`, `Proc`, `Tasks`, `Accel`)
take `&self`. Resource handles keep Rust's receivers where Rust has the
method (`Command`'s builder methods, `output`, `spawn`, and `Child::wait`
take `&mut self`). Tide's additions state theirs: `File::close` takes
`self`, and `read_owned`, `write_owned`, and `Signals::recv` take
`&self`" (or `&mut self`, whichever is decided). The core sentence can
stay as it is if "Rust's" is read as "Rust-style", but the companion's
example should change.

### N3. The Suspends column omits `wait_with_output`, which the companion's example uses

The `Proc` row lists `output`, `status`, `Child::wait`, pipe-end reads and
writes, and `Signals::recv` (`03:83`). `Child::wait_with_output` waits for
exit and reads both pipes. The companion's only example calls it
(`reference-companion.md:540`). It suspends under 03 §0's general rule,
but the column is the list the may-suspend analysis (§7) relies on.

**Fix** (03:83): "`Child::wait` and `wait_with_output`". While editing the
row, make `.spawn() -> Child` read `.spawn() -> io::Result<Child>` to
match `signals(..) -> io::Result<Signals>` in the same cell.

### N4. `ROOT-4`'s list of public fields does not cover `SimRoot`'s `world` and `faults`

The new ROOT-4 enumerates what a public field may hold: "only a capability
or handle (`Root`'s fields, `Child`'s pipe ends) or, for `Root::log`, sink
configuration" (`03:259-262`). `SimRoot` has `Root`'s capability fields
plus the public fields `sim.world` and `sim.faults` (`03:1399-1402`;
`reference.md:513-516`). These are neither listed capabilities, nor
resource handles under §0, nor synchronization handles. Round 32 N4 raised
their classification and left it open. Nothing is forgeable here: ROOT-3
confines them to test code (`03:251-256`), and the clause's purpose
("reading one reaches only authority the value already carries") holds for
them. The enumeration is still incomplete, though, if `SimRoot` counts as
a capability type. `NodeRoot` ("an ordinary value",
`reference-companion.md:441-442`) has only `Root`'s capability fields
(`03:1463-1464`), so it fits the rule.

One wording detail: a pipe-end field holds `Option<ChildStdin>`, not a
handle. "holds only a capability or handle, or an `Option` of one" is
exact.

**Fix** (03 ROOT-4, optional): "(`Root`'s and `SimRoot`'s fields, among
them the simulator's `world` and `faults` handles, and `Child`'s pipe
ends)", or restrict the sentence to non-test std types and cite ROOT-3.

### N5. `Stdio::inherit()` versus the buffered `out` and `err` sinks; no replacement for `.stdout(io::stderr())`

- **Ordering.** The note justifies dropping `From<io::Stdout>` and
  `From<io::Stderr>` because "those are sink handles (§2.4), not file
  descriptors" (`03:166-167`). It keeps `inherit()`, which is Rust's
  default for `spawn` and `status`. `inherit()` hands the child the
  process's real descriptors 1 and 2. Meanwhile `out` and `err` go through
  a process-wide buffer that a writer thread drains (SINK-5,
  `03:373-376`). So `println!("start"); proc.command("ls").status()?;`
  can print the child's output before `start`. Rust's synchronous,
  line-buffered `stdout` never does that. Under `root.log.to_file`, the
  child's `err` output also bypasses the configured destination
  (SINK-2). This predates the change, since `Command` already existed, but
  the change is the first text to make inheritance explicit.
- **Porting.** `cmd.stdout(io::stderr())`, which sends a child's stdout
  to the parent's stderr, is a common Rust idiom. It loses its only
  spelling, and nothing gives a replacement. The companion mentions only
  `From<io::Stdout>` (`reference-companion.md:553-555`), not
  `From<io::Stderr>`.

**Fix:** out of this bead's scope (sink semantics, not API surface). File
a follow-up bead, or add one clause: "starting a child with inherited
stdout or stderr first drains `out` and `err`". Also name the port in the
companion: pipe the stream and copy it to `io::stderr()`. Add
`From<io::Stderr>` to the companion's "There is no" sentence.

### N6. The 6.10 trim drops where threading starts

The old core said `tide fix --thread` edits "along the shortest path from a
holder" (HEAD `reference.md:565-566`). The new text says only "makes that
edit" (`reference.md:565-566`). The algorithm is owned by 03 §4.4
(`03:896-906`), so nothing is contradicted. Two things go missing from the
core: the edit starts at the nearest function that already holds the
capability, not at `main`, and it takes one chain, not every chain. Both
matter when an agent predicts the diff. Neither the companion nor 13
(`reference.md:1057-1058`) says it. The loss is small, and the token
saving paid for 6.1's handle note, so this is acceptable.

**Fix** (optional, companion only): one line in 14 or 20, such as
"`--thread` starts from the nearest function that holds the capability
and takes the shortest chain (03 §4.4)".

### N7. Prose and example

- **Antecedent** (`03:163-165`). "`From` for `File` and the three pipe
  ends, which pipe one child into another" reads as if `From<File>` also
  pipes children together. Write: "`From` for `File` and for the three
  pipe ends; the pipe ends pipe one child into another".
- **Awkward phrase** (`reference-companion.md:558-560`). "as Rust has them
  where Rust has them" is awkward. `Signal` and `Signals` are not in
  Rust's std, and the rest are. Write: "`ReadDir` and `DirEntry` are in
  `std::fs`. All but `Signal` and `Signals` are where Rust puts them."
- **Missing import** (`reference-companion.md:538`). The example's
  `input.write_all(..)` needs `io::Write` in scope. Rust's prelude does not
  provide it, and Tide's prelude is Rust's plus listed items
  (`reference.md:991-994`). The example's other claims are correct.
  `wait_with_output` closes stdin itself, so the explicit `drop(input)` is
  what lets `sort` see EOF; `out.stdout == b"a\nb\n"` type-checks
  (`Vec<u8>: PartialEq<&[u8; 4]>`). Optional fix: a leading
  `use std::{io::Write, process::Stdio};` line.
- **Wrapping.** Every changed prose line is at most 76 columns. The
  example's line 537 is 80 columns. The 03 notes end on a nine-column
  line (`` `Pure`. ``, `03:169`), which is cosmetic.

## Checked and consistent

- **01 D6.** "allowed only for types that opt in and whose fields are all
  visible" (`01:387-389`). ROOT-4's "each has a private field, so no
  struct literal or `Shape` construction (D6) builds one" relies on the
  second condition and agrees with it.
- **`Root` and `Root::log`.** `Root`'s public fields (`03:200-201`,
  `reference.md:364-365`) now satisfy ROOT-4; the old "no public fields"
  did not. `LogConfig` can hold a `File` after `to_file`, but only one
  that `main` opened with its own authority (SINK-2, `03:355-357`). So
  "reaches only authority the value already carries" holds. ROOT-4 now
  requires `Root` to have a private field, which 03 §1.3 does not mention
  but does not contradict ("User code cannot construct it").
- **Synchronization handles.** None has a public field. Tide's own
  `Arc`, atomics, and channel ends take `&Tasks` in their constructors
  (`03:189-190`), so the unchanged "no `Default`" clause holds for them.
- **Exercised authority (§4.1).** Rule 1 attributes a capability method
  call's kind to its receiver path (`03:837-838`). Pipe-end calls
  therefore contribute `proc`, as the note says, and `cmd.arg(..)` on a
  `&mut Command` makes `configure` exercise `proc`. This is consistent
  with "not authority-free".
- **02 and 03 §0.** They list the same twelve std handles in the same
  order (`02:820-823`, `03:35-38`). `Stdio` stands in its own 02 bullet,
  outside the resource handles, matching 03's "acts on nothing itself".
  The structural rule would make `Stdio` not `Pure` anyway, through its
  private `File` variant. The explicit bullet also fixes its status for
  readers who cannot see private fields.
- **Reference 6.6.** "the capability types and `File`, `Command`, and
  other handles derived from them" (`reference.md:467-468`) agrees with
  02. `Stdio` falls under "Everything else ... when its parts are", which
  gives the same answer, and companion 20 states it.
- **Simulation (§9).** The change adds nothing to the simulator. `proc`
  models stay in `sim.world` (`03:1400-1401`, `03:1568`), as the bead's
  out-of-scope line asks.
- **TASK-6.** Dropping a pipe end only closes its descriptor
  (`03:1101-1102`), which is the I/O the example's `drop(input)` relies
  on.
- **Companion intro.** "17–20 and Appendix A hold what the core only
  points to" (`reference-companion.md:10-11`) is right, since 6.1 points
  to 20 (`reference.md:335`).

## Ready to merge

**After B1.** B1 is one clause in 03 and costs no core tokens. N1 to N3
are short 03 edits that complete the bead's own decisions (type
ownership, receivers, suspension), and they should land in the same pass.
N4 and N7 are optional wording. N5 belongs in a follow-up bead. N6 is
acceptable as is.
