# Tide adversarial/consistency review, round 76: sink order and inherited child output (fx-kdj)

## Verdict

**request_changes**: one blocking finding and four non-blocking notes.
The direction is sound and I would approve it once B1 is fixed.

The change makes two decisions. SINK-4's drain-before-start wait is the
right mechanism. It matches Rust for `println!` and `eprintln!`, keeps
`spawn` non-suspending, and needs nothing new from the evented runtime
or the may-suspend analysis. Restoring `From<io::Stdout>` and
`From<io::Stderr>` for `Stdio` is justified. fx-obr removed them because
of an ordering hazard, and SINK-4 now closes that hazard. The restored
conversions grant nothing that `inherit()` does not already grant, and
no stale text about the old choice is left in `design/` or `trials/`.
B1 is that the wait leaves out `dbg!`, which SINK-2 sends to standard
error by default, so the ordering guarantee is incomplete for a sink
whose output lands on descriptor 2.

## What was reviewed

The uncommitted working tree against HEAD `494246d`: `git diff --
design/`, SHA-256
`6c9847fdabb0305cbd3c420b38b0af341e675bc14964af3fb7177a0dd4716dae`
(03 and `reference-companion.md`; `reference.md` unchanged). I
recomputed the hash. `git diff --check -- design/` passes. I did not run
`scripts/count-tokens.sh`, as instructed.

I read:

- `AGENTS.md`, `GOVERNANCE.md`, fx-kdj (`br --no-db show fx-kdj`; no
  comments), fx-obr and its closing comment, round 68 N5, and round 74 as
  a format model;
- the whole diff;
- 01 D7's sink bullet (`01:441-450`);
- 02's not-`Pure` list (`02:810-840`);
- in 03: §0's Task term (`03:60-62`), §1.1's intro, table (the `Proc`
  row, `03:84`), and notes (`03:66-201`), ROOT-4 (`03:283-289`), GLOBAL-3
  (`03:306-309`), §2.4 SINK-1 to SINK-10 (`03:367-435`), RUN-1
  (`03:495-512`), §6.5's `with` signatures (`03:1199-1210`), §7
  (`03:1261-1323`), §8.1 and PAR-4 (`03:1327-1349`), §8.4
  (`03:1411-1423`), §9.1 to §9.3 (`03:1427-1480`), and §9.8
  (`03:1596-1609`);
- core 6.1's `Proc` row (`reference.md:335`), 6.5
  (`reference.md:444-462`), and 6.8 (`reference.md:510-543`);
- companion 20 (`reference-companion.md:526-575`);
- a grep of `design/`, `trials/`, and `README.md` for `io::Stdout`,
  `io::Stderr`, `io::stderr()`, `Stdio`, "file descriptor", "sink
  handle", "writer thread", and "sink buffer".

## Blocking findings

### B1. The wait omits `dbg!`, which writes to standard error by default

SINK-4's wait covers "every `out` and `err` record, and ... every `log`
record" (`03:403-405`). SINK-2 sends `dbg` "to stderr in debug builds"
(`03:385-386`), the same descriptor that an inherited child's stderr,
or `.stdout(io::stderr())`, writes to directly. The reason SINK-4 gives
for the wait ("writes to the descriptors directly, past the writer
thread", `03:402-403`) applies to `dbg` records just as it does to
`err` records. Even so, `dbg!(x); cmd.status()?` gets no ordering
guarantee. In Rust, `dbg!` writes to an unbuffered `stderr`, so its
output always comes before the child's, and the "as in Rust" parity the
bead asks for (D18) fails for the one sink that exists only to debug.
`trace` has the same gap whenever its destination is set to a
descriptor (`03:386`). The companion's bullet copies the omission
("every earlier `println!`, `eprintln!`, and log record",
`reference-companion.md:562-564`). It also says those records are
"written out", although SINK-5 lets `log` records be dropped
(`03:415-416`).

**Fix** (03 SINK-4, `03:403-405`): "So a call that starts such a child
first waits until the writer has written every record emitted before
the call, or dropped it where SINK-5 allows: `println!(...)` ...". This
covers `out`, `err`, `dbg`, `log`, and `trace` without listing them, and
it stays correct if a sink is added later. Companion
(`reference-companion.md:562-564`): "Starting a child first writes out
every earlier `println!`, `eprintln!`, `dbg!`, and log record, so ...".

## Non-blocking notes

### N1. Say whether the wait is conditional, and which `spawn` is meant

SINK-4 describes the child "whose stdout or stderr is the process's
own" and then says "So `spawn`, `status`, and `output` first wait"
(`03:399-403`). Read literally, the wait is unconditional. That means
every `output()` call waits for the writer to drain, including one with
Rust's default piped stdout and stderr (`output`'s defaults are piped
stdout and stderr and a null stdin, which is why `03:401` rightly names
only `spawn` and `status` as inheriting). It also means `spawn` with
`Stdio::piped()` on both streams waits. When the process's stdout is a
pipe that a slow reader drains, such as `prog | less`, that wait blocks
a worker OS thread under the evented runtime where Rust would not block.
The rule's own reason holds only when a child inherits a descriptor. In
addition, "`spawn`" sits in a rule that has just mentioned `join` and
the parallel forms, so a reader can take it for `Scope::spawn`.

**Fix** (03 SINK-4): "So `Command`'s `spawn`, `status`, and `output`,
when the child's stdout or stderr is the process's own, first wait
...". If the unconditional wait is intended because it is simpler, say
so: "first wait, whatever the child's stdio, ...".

### N2. Records still in another task's parallel branch buffers

The wait covers records "emitted before the call" (`03:405`). Across
tasks, SINK-4 orders records by emission (`03:394-396`). Records that a
parallel branch writes stay in that branch's buffer until its join
appends them (`03:396-398`, `03:1413-1415`). Suppose task A is inside a
long `par_map` whose branches print, and task B calls `status()`. B
cannot wait for A's branch records without waiting for A's join, and
nothing tells the reader that it does not. The last sentence, "A
parallel branch cannot start a child" (`03:408-409`), covers the case
where the branch itself starts the child. It does not cover this one.

**Fix** (03 SINK-4): end the wait sentence with "; a branch's records
count as emitted when its join appends them". That makes the cross-task
case follow from the existing emission-order rule.

### N3. The wait's end when a descriptor cannot be written

The wait ends when the writer "has written every `out` and `err`
record" (`03:403-404`). SINK-5 defines only the closed-stdout case,
which exits with status 141 (`03:413-415`), and that also ends the
wait. If standard error is a closed pipe, or descriptor 2 is closed,
`err` records can never be "written". SINK-1's always-`Ok` handles
(`03:378-379`) imply that the writer discards them, but nothing says
so, and the wait now depends on it. The gap in SINK-5 predates this
change, but before this change no caller waited on it.

**Fix**: B1's "written ... or dropped where SINK-5 allows" wording
leaves this to SINK-5. Then add to SINK-5: "A write to standard error
that fails is discarded." Alternatively, have the wait sentence say
"written, or failed to write".

### N4. Record the decision on fx-kdj, including the reversal of fx-obr

fx-kdj has no comments. fx-obr's closing comment still records
"From<io::Stdout>/<io::Stderr> dropped (sink handles, not fds)". The
design text is correct, but under AGENTS.md "Decisions" the decision and
its reasons belong on the bead too. Here that means the reason for the
reversal: SINK-4's wait removes the ordering hazard that was the
substantive objection in fx-obr, the conversions grant nothing beyond
`inherit()`, and the idiom ports unchanged. The cost belongs there as
well: under `root.log.to_file`, a write through `io::stderr()` reaches
the file, while the same value used as a `Stdio` reaches descriptor 2
(`03:171-172`). Round 68 N5's other remedy, piping the stream and
copying it to `io::stderr()`, should be recorded as rejected and the
reason given: it is more code and loses porting parity.

**Fix** (fx-kdj, before closing): one comment that covers the above.

## Simulation

The bead's scope includes ordering "under simulation". The diff says
the simulator needs no wait because records enter its trace when emitted
(`03:407-408`, SINK-6 `03:417-418`). That is right for the wait itself.
It does not say where a `sim.world.proc` model's inherited output goes
(`03:1442`, `03:1609`). The bead puts other process semantics under
simulation out of scope, and proc models are only named so far, so I do
not block on it. One clause would make the ordering claim complete
without defining models: "...since records enter its trace when emitted
(SINK-6), before anything a `sim.world.proc` model does after the call."

## Checked and consistent

- **Rust API.** `impl From<io::Stdout> for Stdio` and `impl
  From<io::Stderr> for Stdio` are stable since Rust 1.74 and name the
  process's descriptors 1 and 2 directly. Rust does not flush `Stdout`'s
  buffer when converting, so "as `inherit()` does" (`03:171`) is
  accurate. `Command::stdout<T: Into<Stdio>>(&mut self, cfg: T)` accepts
  them, so `.stdout(io::stderr())` ports unchanged. Rust's `spawn` and
  `status` inherit all three streams by default; `output` pipes stdout and
  stderr and nulls stdin. `03:401` therefore names the inheriting defaults
  correctly. Rust's `stdout` is line-buffered whether or not it is a
  terminal, so `println!` followed by `status()` prints in order in Rust,
  and the claim at `03:405-406` holds. With a newline-free `print!`, Tide's
  wait does better than Rust: it writes the partial line, where Rust's
  `LineWriter` keeps it. The diff makes no claim about that case.
- **The remaining `Stdio` conversions.** On Unix, Rust's `From` impls for
  `Stdio` are the three pipe ends, `File`, `OwnedFd`, `io::Stdout`,
  `io::Stderr`, `io::PipeReader`, and `io::PipeWriter`. The note keeps
  the first four kinds plus the two sinks and drops the other three, so
  "every other conversion into a `Stdio` starts from a handle"
  (`03:174-176`) is exact.
- **Authority, ROOT-4, `Pure`.** `io::stdout()` needs no capability, and
  `Stdio::inherit()` already hands a child descriptors 1 and 2 with none.
  The restored conversions add only the cross mapping (child stdout to
  parent stderr). A `Stdio` acts only through a `Command`, which carries
  `Proc`'s authority (`03:153-160`). Nothing leaks. `Stdio` stays outside
  ROOT-4's handle set. 02's entry ("acts on nothing itself but can hold a
  `File` or a pipe end", `02:827-828`) is still true. A `Pure` sink
  value converting into a non-`Pure` `Stdio` is no worse than
  `inherit()`, and SUSP-1's "built from `Pure` values" argument
  (`03:1270-1274`) is unaffected.
- **Suspension, checkpoints, §7.** `spawn` is not in the `Proc` row's
  "Suspends" column (`03:84`), so it is not a checkpoint. A wait that
  blocks the OS thread, as SINK-5's backpressure does (`03:410-413`),
  keeps it that way. `status` and `output` already suspend. The wait comes
  before the suspension and is not a checkpoint, so a cancellation during
  the wait is observed at the call's own suspension. Under the evented
  runtime, the wait blocks a worker, as SINK-5's high-water blocking
  already may. The writer is its own OS thread, so draining needs no
  worker and cannot deadlock against the waiting task. A closed stdout
  pipe during the wait exits with status 141 under SINK-5. Applying the
  word "backpressure" to a drain barrier stretches SINK-5's term a
  little, but it names the right mechanism and the right non-property.
- **Parallel branches.** §8.1 requires `join`'s closures to be `Pure`
  (`03:1333`), and PAR-4 makes every branch authority-free
  (`03:1347-1349`). `Command` is not `Pure`, so "A parallel branch cannot
  start a child" (`03:408-409`) holds. A `Shared::with` closure is `Pure`
  too (`03:1204-1208`), so no child starts while a cell is held.
- **D7 and core 6.5.** D7's "inside `par` their output is buffered and
  emitted in sequential order" (`01:444-445`) and core 6.5's "Order is
  program order" (`reference.md:455-456`) agree with the extended
  SINK-4. The core's "write-only `impl Write` handles"
  (`reference.md:447`) describes the values, and the note names their
  types `io::Stdout` and `io::Stderr`, as Rust's `stdout()` returns
  them. The core needs no change, which fits its 24,999-token count.
- **Stale text.** No text in `design/`, `trials/`, or `README.md` still
  says the sink conversions are gone. Rounds 68 and 69 quote the old
  choice, which is history.
- **Bead scope and acceptance.** Both points are decided in 03 with
  reasons. Companion 20 states both: the `Stdio` bullet and the new
  "Output order" bullet. Branch buffers and simulation are addressed,
  subject to N2 and the note under "Simulation". The core is unchanged.

## Prose and wrapping

Every added line is at most 76 columns. The prose is plain. One wrap is
gratuitous: `03:395-396` re-wraps "and the simulator in scheduler order"
without changing any words, which adds a line of diff noise. It is
harmless and not worth a fix. The companion's "your standard error"
(`reference-companion.md:558`) matches that chapter's second-person
voice.
