# Tide adversarial/consistency review, round 73: File::close under cancellation (fx-nyy)

## Verdict

**request changes**: one blocking finding, a one-line fix that leaves
TASK-5 incomplete if skipped. There are four non-blocking notes and two
out-of-scope findings.

The decision is sound. `File::close(self)` is shielded: it may suspend,
it is not a checkpoint, and it returns close(2)'s own result even in a
cancelled scope. This removes the TASK-3 conflict the bead names, because
an operation that is not a checkpoint never returns a cancellation error,
so it has nothing to carry back. The decision adds no uninterruptible
behavior that `drop` lacks, since `Drop` already closes descriptors
(TASK-6). It also matches how JOIN-1 and JOIN-2 already use "shielded".
I found no other consuming checkpoint among the handles. The new §1.1
intro now matches every row of its table, and it also fixes an
incidental problem in the old intro (see "Checked and consistent").

The blocking gap is a leftover. TASK-5 is 03's list of what cancellation
cannot interrupt, and it does not name `File::close`, though 03 §1.1 and
core 7.2 now say close is never interrupted. The notes cover the
narrowed TASK-3 wording, which now differs from D8 and 7.2 and from 03's
own reasoning; D8's sentence, which close now contradicts word for word;
two gaps in the stated reasons; and the 7.2 sentence, which contradicts
itself if read literally.

## What was reviewed

The uncommitted working tree against HEAD `579d785`: `git diff --
design/`, SHA-256
`7d664c0f8e2d05a1ec5942a7b8d9a42d27214c4f37e4723a46adc5ea420329c4`
(03, `reference.md`, `reference-companion.md`). I recomputed the hash.
`git diff --check -- design/` passes. I did not run
`scripts/count-tokens.sh`, as instructed.

I read:

- `AGENTS.md`, `GOVERNANCE.md`, fx-nyy (`br --no-db show fx-nyy`), round
  70 (B1), and round 71 as the format model;
- the whole diff;
- 01 D8 (`01:509-559`);
- in 03: §0 Terms (`03:31-62`), §1.1's intro, table, and notes
  (`03:66-199`), §1.2 (`03:201-219`), §6.1 to §6.4
  (`03:1026-1184`), §7 (`03:1247-1309`), §9.3 to §9.5
  (`03:1444-1526`), and §10 (`03:1597-1650`);
- in the core: 6.3's `Fs` and `File` signatures (`reference.md:380-398`),
  6.8's scheduling points (`:530-541`), 7.1 (`:600-640`), and 7.2
  (`:659-684`);
- companion 20 (`reference-companion.md:525-570`) and the count line
  (`:14`);
- a grep of `design/` for `close`, "consume a value", `shield`,
  `checkpoint`, `spawn_isolated`, and `AsRef`.

## Blocking findings

### B1. TASK-5 omits `File::close`

TASK-5 is titled "What cancellation cannot interrupt" and lists
"Authority-free computation, `Shared::with` acquisition and closures,
foreign calls, and joins", followed by "Deadlines are therefore soft"
(`03:1119-1124`). The diff adds a new member of that set. 03 says of
close, "Like `drop`, it cannot be interrupted" (`03:195-196`), and core
7.2 now lists `File::close` among the things that "are never
interrupted" (`reference.md:665-667`). TASK-5 does not list it.

This matters for the conclusion TASK-5 draws. close(2) can block for a
long time on NFS, FUSE, or some devices, because Linux runs the file
system's flush on the last close. A shielded close therefore holds its
task past a scope deadline, and, because "shielded" here means not a
checkpoint at all (JOIN-1 and JOIN-2's usage, `03:1134-1143`), past the
deadline of a `shield` scope placed around it as well. TASK-6 still
says "bounded cleanup is a shielded scope with its own deadline"
(`03:1125-1127`). A reader who checks TASK-5 to bound cleanup latency
will not see that close is outside every deadline.

**Fix** (03 TASK-5): "Authority-free computation, `Shared::with`
acquisition and closures, foreign calls, joins, and `File::close`
(§1.1)." No core change is needed, since 7.2 already lists it.

## Non-blocking notes

### N1. The narrowed TASK-3 wording differs from D8, 7.2, and 03's own reasoning

TASK-3 now reads "Operations that take ownership of a value to deliver or
keep it (`send`, `read_owned`, `write_owned`) return it inside the error;
`File::close` is shielded instead" (`03:1100-1102`). The other
statements of the same rule still say "operations that consume a value"
(D8, `01:521`; core 7.2, `reference.md:668`). The Receivers note
justifies the decision against the old wording: "returning the `File`
inside one, as TASK-3 asks of consuming checkpoints" (`03:193-194`).
TASK-3 no longer uses "consuming checkpoints", and under its new wording
close, which consumes the `File` to destroy it, not to deliver or keep
it, is not in the rule's class. So "instead" contrasts close with a rule
that, as now worded, would not have covered it.

The narrowing was not needed. TASK-3 speaks only of checkpoints ("every
checkpoint inside it ... returns its cancellation error"), and close is
no longer one, so the old wording already left close out. In practice
the two wordings cover the same checkpoints today (see the handle sweep
below), so no rule is contradicted. But one rule now has two wordings,
and "deliver or keep" is vaguer than "consume": `read_owned` neither
delivers nor keeps its buffer in an obvious sense.

**Fix** (03 TASK-3): "Checkpoints that consume a value (`send`,
`read_owned`, `write_owned`) return it inside the error
(`SendError::Cancelled(v)`); `File::close` consumes its `File` but is
shielded, not a checkpoint (§1.1)."

### N2. D8's cancellation sentence now has one more unstated exception

D8 says "capability operations in a cancelled scope return
`Err(Cancelled)`" (`01:517-518`). By 03 §0, `File` is a capability
(`03:33-37`), so a close in a cancelled scope is a capability operation
that returns its own result, not `Err(Cancelled)`. D8 already
approximated: it names the join and acquisition exceptions but not
`spawn_isolated`, `shield`, or non-suspending operations such as
`Rand::next_u64`. So this is not a new kind of gap, and 03 §0 is the
precise statement. But 01 owns every language decision (AGENTS.md), and
D8's sentence now has a named, user-visible exception it does not
mention.

**Fix** (01 D8, optional): "suspending capability operations in a
cancelled scope, except the few shielded ones (03 §1.1), return
`Err(Cancelled)`". If 01 is left alone, record on fx-nyy that D8's
sentence is read through 03 §0.

### N3. The Receivers note leaves out two costs of the rejected option

The rejected option is "returning the `File` inside one ... would leave
the caller to retry the close" (`03:193-195`). Two points are missing:

- The bead's actual conflict is that an `io::Error` cannot hold a
  `File` (`io::Error` is `Pure` and `File` is not; round 70 B1 made the
  same argument for `Child`). So the option needs a new error type for
  one method. The note gave this reason for `wait_with_output` six lines
  earlier (`03:182-183`) but not for close.
- Because cancellation is level-triggered (TASK-3), the retry would
  fail the same way unless the caller wraps it in `shield`. So every
  close in cleanup would need `shield`, which is the boilerplate that
  shielding close removes.

"Like `drop`, it cannot be interrupted" is accurate: `Drop` closes
descriptors (TASK-6) and is never interrupted. It does not mention the
cost, which is B1's deadline point.

**Fix** (03 Receivers note): "... returning the `File` inside one, as
TASK-3 asks of consuming checkpoints, would need an error type that can
hold a `File` and would make every close in a cancelled scope a retry
under `shield`. Like `drop`, it cannot be interrupted, so a slow close
holds its task past a deadline (TASK-5)."

### N4. Read literally, core 7.2's checkpoint sentence contradicts itself

"Every capability operation that may suspend is a checkpoint;
authority-free code, `Shared::with*`, joins, scope exit, and
`File::close` are not" (`reference.md:665-667`). `File::close` is a
capability operation that may suspend, so the second clause contradicts
the first unless it is read as a list of exceptions. Scope exit already
had this form before the diff, and a reader will most likely take it as
exceptions. The core has one token to spare (24,999 of 25,000,
`reference-companion.md:14`), so a rewrite must not add tokens.

**Fix** (core, only if it costs no tokens): drop the semicolon form for
an exception form, for example "Every capability operation that may
suspend is a checkpoint except scope exit and `File::close`;
authority-free code, `Shared::with*`, and joins are not either". Recount
before taking it. Otherwise leave it as it is.

## Out-of-scope findings

These predate the diff. Each is a candidate follow-up bead, not a
condition for fx-nyy.

- **By-value arguments under the consume rule.** D8, TASK-3, and 7.2
  read literally also cover checkpoints whose generic arguments a caller
  may pass by value: `fs.write(p, data)` and `write_synced(p, data:
  impl AsRef<[u8]>)` (`reference.md:386`), path arguments
  `p: impl AsRef<Path>`, and `Net::connect(addr)`. All return a
  `Pure` error, so a `Vec` or `PathBuf` passed by value is dropped on
  cancellation. The same holds for Bend kernel calls (later tier): a
  call is a checkpoint (`03:1619`) and consumes its arguments
  (`board: BendArray<u32>`, `03:1608`), and `BendFault` (`03:1632-1635`)
  has no cancellation variant, although §1.1 says "Every capability
  error type has a cancellation variant" (`03:70-71`). A fix would say
  that the rule covers values the operation needs to own (channel
  payloads and owned buffers); an argument it only reads, through
  `impl AsRef` or by copying it to a kernel, is dropped. It would also
  add `BendFault::Cancelled`.
- **`spawn` under `Tasks::limit` waits.** The core says "`spawn` waits
  for a free slot (not a checkpoint)" (`reference.md:636`). 03 §1.2's
  `Scope` row gives `spawn` "Waits: none" (`03:207`). It is not a
  checkpoint, so TASK-3 is unaffected, but it is a suspension that 03
  does not list.

## Handle sweep (item 2 of the brief)

I looked for a method that consumes a value, may suspend, and is a
checkpoint, and found none other than `close`:

| Handle or method | Receiver and argument | Checkpoint? | Status |
| --- | --- | --- | --- |
| `File::read_owned`, `write_owned`; the socket versions | `&self`, `buf: Vec<u8>` | yes | returns the buffer inside the error (`03:144-147`) |
| `File::sync`, `set_len`, `seek`, `read`, `write` | `&self` or `&mut self`, borrowed data | yes | nothing consumed |
| `ReadDir` iteration, `DirEntry::metadata`, `file_type` | `&mut self` or `&self` | yes | nothing consumed |
| `TcpStream`, `TcpListener`, `UdpSocket` (`accept`, `send_to`, `recv_from`, `shutdown`) | `&self`, borrowed buffers | yes | nothing consumed (the `addr` of `send_to` is generic: out-of-scope finding above) |
| `Command::output`, `status`, `spawn`; `Child::wait`, `wait_with_output` | `&mut self` | yes | fixed in fx-obr |
| pipe ends, `Signals::recv` | `&mut self`, `&self` | yes | nothing consumed |
| `Stdin` reads | `&self` | yes | nothing consumed (`lines()` takes `&self` in Tide) |
| `SyncSender::send`, `Sender::send` | `&self`, `v: T` | full `sync_channel` only | `SendError::Cancelled(v)` |
| `Shared::reset(v)`, `with*` | `&self` | no (acquisition) | — |
| `Task::join` | `self` | no, shielded (JOIN-2) | — |
| `Scope::spawn(self, f)`, `set_deadline`, `cancel` | `Scope: Copy` | no | — |
| `Tasks::scope(body)`, `spawn_isolated(f)` | `&self`, closure by value | no, shielded | — |
| `BufWriter::into_inner(self)` | `self` | its flush writes through `W`, a checkpoint | Rust's `IntoInnerError` returns the writer, so it already complies |
| `Read::bytes`, `take`, `chain` (adaptors over `File`) | `self` | the adaptor does no I/O; `next` borrows | — |

## Checked and consistent

- **§0 and the §1.1 intro against the table** (item 3). §0: "except the
  shielded ones marked in §1.1, such as `File::close`. Joins and
  `Shared::with` acquisition are not checkpoints either" (`03:56-59`).
  The §1.1 intro: "all are checkpoints except those marked shielded"
  (`03:69-70`). The rows with a shielded mark are `File` ("all; `close`
  shielded", `03:76`) and `Tasks` ("`scope` exit and `spawn_isolated`
  (shielded)", `03:85`). Every other row's Suspends entries are
  checkpoints, including Bend kernel calls (`03:1619`). The new intro
  also fixes the old one ("all are checkpoints except the shielded
  scope-exit join"), which by omission made `spawn_isolated` a
  checkpoint although its `Result<T, Panicked>` has no cancellation
  variant (`03:1036-1037`). `Task::join` is in §1.2, whose "Waits"
  column marks it "yes, shielded" (`03:208`); §0 covers it under
  "Joins".
- **TASK-4 and TASK-2.** A deadline that passes during a close does not
  interrupt it. The next checkpoint observes the cancellation, and close
  sets no echo bit, since it never returns a cancellation error.
- **TASK-6.** "`File::close` is itself shielded (§1.1); other cleanup
  that needs I/O, such as a final write or `sync`, runs under `shield`"
  (`03:1128-1130`) agrees with §1.1 and keeps the old guidance for
  everything except close.
- **JOIN-1 and JOIN-2** use "shielded" to mean not a checkpoint and not
  cancellable (`03:1134-1143`). Close uses it the same way. TASK-6's
  `shield` scope is a different use (it ignores enclosing cancellation
  but has its own deadline), which B1's fix makes visible.
- **Evented runtime (§7).** Close may suspend (for example as an
  io_uring close), so a function that calls it may suspend (SUSP-1), as
  before. Because close takes `self`, no borrow of the `File` can have
  an operation in flight, so TASK-3's "in-flight kernel operation is
  cancelled" clause has nothing to apply to.
- **Simulator (§9).** Close is a handle operation and therefore a
  scheduling point (`03:1458-1460`; core `reference.md:537-538`), like
  the shielded join. Close does not sync. Durability comes only from
  `sync` or `sync_dir` (`03:1499-1500`), and nothing in the diff says
  close makes a write durable or affects torn writes. A crash stops a
  task at its current scheduling point without running destructors
  (`03:1496-1497`), so a crash during a close leaves exactly what a
  crash before or after it leaves.
- **close(2) claims.** "Always closes the descriptor and returns the
  close's own result" is a runtime guarantee. It matches Linux, where
  close(2) releases the descriptor even when it reports an error, so
  the runtime must not retry. "Its only purpose beyond `drop`" is
  accurate: `File` is unbuffered, and close's result is how deferred
  write errors, such as NFS's, reach the program.
- **Core count.** The only change to the core adds ", `File::close`" to
  one list in 7.2. A rise of about six tokens, from 24,993 to 24,999, is
  plausible. Not recounted. The count line's date is 2026-10-21, which
  matches the bead's dates.
- **Companion 20** still says only that `File::close` takes `self`
  (`reference-companion.md:569`). It does not need the shielding rule,
  because 7.2 states it.
- **Core 6.3.** `fn close(self) -> io::Result<()>; // metadata, as std`
  (`reference.md:398`) does not point to 7.2. That is acceptable
  because the core has a single token to spare.

## Prose and wrapping

Every added prose line is at most 76 columns, and the table row is
exempt. The Receivers note ends on "interrupted." alone on a line
(`03:196`), which is the end of the bullet, not an unfilled wrap. The
prose is plain and specific.
