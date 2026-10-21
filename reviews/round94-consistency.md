# Tide adversarial/consistency review, round 94: TASK-3 scope (fx-ms8)

## Verdict

**revise**: two blocking findings, six non-blocking notes.

The diff settles the three points fx-ms8 names, and most of it holds.
The return rule now covers what a checkpoint hands over, a channel
payload or an owned buffer, and says that read-only arguments are
dropped. That matches every suspending method in §1.1 and needs no new
error types. `BendFault::Cancelled` meets §1.1's "every capability error
type has a cancellation variant". The rule that an uncancellable call
returns its own result can be built under all four runtimes. Two gaps
remain. TASK-3 names only regular-file reads, writes, and `fsync` as
uncancellable, but path and metadata calls on the same file system
(`open`, `rename`, removals, `metadata`, directory reads, `set_len`)
also sleep uninterruptibly. Read as a complete list, it makes the
runtimes and the simulator cancel a `rename` that the kernel finishes
anyway (B1). An in-process Bend call that cannot be stopped "finishes
first", but nothing says what the call then returns (B2). Both fixes
are a few words in 03. The core is still 24,996 tokens and contradicts
nothing.

## What was reviewed

The uncommitted working tree against HEAD `e51564d`: `git diff --
design/`, SHA-256
`1ff34b906beebc27a9e25dd39f4dcc73c1cb90db1f233ce1b830ec8767c7a337`
(01, 03, core, companion). I recomputed the hash. `git diff --check`
passes, and no added line is over 80 columns.
`scripts/count-tokens.sh design/reference.md` gives 24,996, which
matches the companion's count line (`reference-companion.md:14`).

A word diff of the §1.1 Receivers note (`03:179-199`) confirms that the
rewrap changed only two passages: the `wait_with_output` reason
(`03:184-186`) and "as `send` returns its payload" (`03:196`), which
replaces "as TASK-3 asks of consuming checkpoints".

I read:

- `AGENTS.md`; fx-ms8 and its decision comment; the out-of-scope
  findings of `reviews/round73-consistency.md` (by-value arguments,
  `BendFault`) and `reviews/round74-consistency.md` (uncancellable
  kernel operations), which the bead answers;
- in 03: §0 (`03:31-62`), §1.1 with all its notes (`03:66-202`), §1.2
  (`03:204-213`), §1.3's runtime table (`03:245-257`), §5.1 (`03:965-997`),
  §6.2 TASK-1 to TASK-6 (`03:1125-1226`), §6.3, §6.4 (`03:1228-1279`),
  §6.5's LOCK-3, §7 (`03:1343-1405`), §9.1 to §9.5 (`03:1507-1622`), and
  §10 (`03:1693-1749`);
- `01-design.md` D7's durability text, D8 (`01:509-558`), and D16
  (`01:926-944`);
- core 6.1 (`reference.md:320-350`), 6.3 (`reference.md:379-427`), 7.2
  (`reference.md:659-684`), and 7.3 (`reference.md:686-697`);
- companion 15.2 (`reference-companion.md:184-246`) and 20
  (`reference-companion.md:545-555`);
- `research/11-bend-interop.md` H6 (`research/11:169-173`);
- a grep of `design/` for "consume", "consuming", "in flight",
  "in-flight", "interrupted", "is cancelled before", `BendFault`, and
  `exists`.

## Blocking findings

### B1. TASK-3 names too few uncancellable calls

TASK-3 says "Regular-file reads and writes and `fsync` cannot be
cancelled once running" (`03:1192-1193`), and TASK-5 lists "a
regular-file kernel call already running" (`03:1215`). Every other call
into a file system behaves the same way. On Linux, `openat`, `renameat`,
`unlinkat`, `mkdirat`, `statx`, `getdents64`, and `ftruncate` take
inode locks and wait for disk or server I/O in uninterruptible or
killable sleeps. On NFS they wait on the server, as `fsync` does.
Under io_uring these operations run in io-wq workers. A cancel request
stops one only before a worker picks it up, as with a regular-file read.
That covers most of `Fs`: `open`, `create`, `open_with`, `rename`,
`remove_file`, `remove_dir`, `create_dir_all`, `read_dir` and its
iteration, `metadata`, `exists`, `sync_dir`, and `File::set_len`
(`03:75-76`).

Read as a complete list, which is how TASK-5's title "What cancellation
cannot interrupt" presents it, the text gets three things wrong:

- An implementer of the threads or blocking runtime finds no way to
  cancel a running `stat` or `rename`, yet TASK-3 says the kernel can
  cancel it.
- The simulator gives every capability operation a modeled latency
  (`03:1543-1544`). Following TASK-3 literally, it would cancel a
  simulated `rename` during that latency and return `Cancelled`. On
  Linux the same `rename` completes and returns `Ok`. The simulator
  would then test exactly what the decision comment rejected: reporting
  `Cancelled` after the effect happened. Here that also leaves §9.5's
  crash model, where "a `rename` either happened or did not", out of
  step with the program's view.
- TASK-5 tells a reader bounding cleanup that only a running read,
  write, `fsync`, or `close` can block past a shielded scope's deadline.
  A `rename` or `sync_dir` on a hung NFS mount blocks just as long, and
  atomic replace (`03:115-117`, `reference.md:421-423`) is made of those
  calls.

**Fix** (03 TASK-3 and TASK-5): name the class, not three calls. For
example, TASK-3: "Calls on regular files and directories, such as
reads, writes, `fsync`, opens, renames, removals, and metadata queries,
cannot be cancelled once running"; TASK-5: "a file-system call already
running (TASK-3)". The core needs nothing, since 7.2 does not list
uncancellable calls.

### B2. An in-process Bend call's result after cancellation is undecided

§10 says that in process mode cancelling a call "returns
`BendFault::Cancelled`", and that "in-process mode cannot stop a running
kernel, which, like a foreign call, finishes first (TASK-5)"
(`03:1715-1718`). It does not say what the in-process call returns once
the kernel finishes. Both readings are plausible:

- "finishes first", then the checkpoint returns `BendFault::Cancelled`,
  as process mode does; or
- the call returns the kernel's result, as TASK-3 decides for a call the
  kernel cannot cancel ("runs to its end and returns its own result",
  `03:1193`) and as a foreign call does under TASK-5.

The caller matches on this value. The two readings also differ in
whether `mode = "inproc"` versus `"process"` (`03:1697`), a manifest
setting, changes what a cancelled program sees. fx-ms8 asks for "a
cancellation story for Bend calls", and this is the half that is
missing.

**Fix** (03 §10 `Accel`): pick one and say it. The second fits the
decision comment's "like a foreign call" and TASK-3's rule. Discarding
a pure result costs nothing, but TASK-3 already rejected inventing a
`Cancelled` after the work is done. For example: "in-process mode cannot
stop a running kernel, which, like a foreign call, finishes and returns
its result; the task observes the cancellation at its next checkpoint
(TASK-3, TASK-5)". A call still queued behind the serialization lock is
not running, so it returns `Cancelled`. The text already implies this,
and saying it costs four words.

## Non-blocking notes

### N1. "Kernel call" now means two things

TASK-3's new paragraph says "A kernel call in flight ... if the kernel
can cancel it" (`03:1190-1191`), and TASK-5 says "a regular-file kernel
call" (`03:1215`). §1.1's `Accel` row (`03:86`) and §10 use "kernel
calls" for Bend kernels. A reader coming from §10 can take TASK-3's
paragraph to govern Bend calls, which is one source of B2's ambiguity.
"System call" in TASK-3 and TASK-5 removes the overlap.

### N2. A cancellable call that completes before its cancel

"is cancelled before the error returns if the kernel can cancel it"
(`03:1190-1191`) does not cover the race in which a cancellable call
completes first. Under io_uring, a socket `recv` can post its
completion with data while the cancel request is in flight. If the
runtime then returns `Cancelled`, the bytes are gone from the stream;
`read_owned` returns its buffer, but nothing says whether the buffer
holds them. The old text had the same gap. The diff's own principle,
that a call which finished returns its own result, extends to it in one
clause: "a call that completes before the cancellation takes effect
returns its own result".

### N3. Multi-call operations: what "stop partway" leaves

"An operation made of several kernel calls, such as `fs.write` or
`remove_dir_all`, checks between them and can stop partway (§1.1)"
(`03:1195-1197`). The §1.1 pointer covers only `remove_dir_all`
(`03:104-106`). Two consequences are left for the reader to find:

- `fs.write` opens with `O_TRUNC`, so cancellation between the open and
  the first write leaves the target empty. Rust's `fs::write` can do the
  same on an I/O error, so this is not new behavior, but cancellation
  now causes it too. Atomic replace (`03:115-117`) is the answer, and a
  short pointer to it would help.
- "Between them" could include the gap before the final close. A check
  there would report `Cancelled` after the last write finished, which
  is the outcome the decision comment rejected. `close` is not a
  checkpoint (§1.1), so the intended reading is probably "before each
  call it starts". Saying so closes the question. `write_synced` is
  unaffected: `Cancelled` before its `fsync` is accurate, because the
  file is not yet durable, which is what `write_synced` promises.

### N4. The simulator's side of the rules is implicit

§1.3 says the runtimes "differ in cost, not meaning" (`03:252-253`), so
the simulator must follow TASK-3's split. §9 does not say so, and two
cases need it. A simulated file call in flight when cancellation
arrives completes at its modeled time and returns its result (with B1,
for every file-system call). A simulated Bend call (`03:1735-1738`)
during its modeled latency should behave as the dependency's declared
mode does under B2. One sentence in §9.3 or §10's Simulation bullet
would pin both.

### N5. `resolve` and `get` under the threads runtime

The split is between calls the kernel can cancel and calls it cannot.
`Net::resolve`, and the lookup inside `Net::get`, is likely to be
`getaddrinfo` on a blocking thread, a library call that retries on
`EINTR`. It can be abandoned but not cancelled, as Bend process mode
abandons its request (`03:1716`). Abandoning a read-only lookup is safe.
TASK-3 could allow it in a clause: "or abandoned if it has no effect".
Otherwise an implementer has to infer that it is permitted.

### N6. Wording

- `03:1183`: "a Bend call's arguments, which are copied to the server"
  describes process mode only. In-process mode has no server, and D16
  says values "move or copy across" (`01:937-938`). "copied across
  (§10)" covers both modes.
- `03:1218`: "soft. The evented runtime logs a `warn!` record" leaves a
  49-column line in the middle of the bullet. Reflowing the paragraph
  fixes it.
- `03:1179-1188`: a limited `spawn` hands its closure over but drops it.
  The appositive list ("a channel payload or an owned buffer") excludes
  it, and the exception follows at once. It reads correctly, but
  "Unlike `send`," before "a limited `spawn`" would show it is an
  exception on purpose.

## Checked and consistent

- **The return rule against every suspending method.** By-value
  arguments that a checkpoint hands over: `SyncSender::send`'s payload
  (`SendError::Cancelled(v)`, `03:1259`), and `read_owned` and
  `write_owned` buffers (Owned buffers note, `03:144-147`, "inside the
  error on failure"). Both are covered. `Sender::send` never blocks
  (`03:1266`), so "`send`" in TASK-3 means the bounded one, as core 7.3
  says (`reference.md:692`). Read-only by-value arguments:
  `fs.write`'s and `write_synced`'s `impl AsRef<[u8]>`
  (`reference.md:386`), every `p: impl AsRef<Path>`, `connect`, `bind`,
  and `send_to` addresses, and Bend arguments. All are now dropped, as
  stated. Methods that take `&self` or `&mut self` (`File`, sockets,
  pipe ends, `Stdin`, `Signals::recv`, `Child::wait`) hand nothing
  over. `wait_with_output` takes `&mut self` and `File::close` is
  shielded, so neither falls under the rule. A limited `spawn` is the
  one stated exception.
- **Receivers note.** The new reason for `wait_with_output`
  (`03:184-186`) does not lean on TASK-3, which is right now that a
  `Child` is not a payload or buffer. It is sound: dropping a Rust
  `Child` neither kills nor waits for the process. The close sentence's
  "as `send` returns its payload" (`03:196`) keeps the argument without
  the old "consuming checkpoints" term. Companion 20
  (`reference-companion.md:545-548`) agrees.
- **D8 and core 7.2.** "operations that hand a value over return it
  inside the error" (`01:521`, `reference.md:668-669`) matches TASK-3.
  D8's "capability operations in a cancelled scope return
  `Err(Cancelled)`" (`01:517-518`) and core 7.2's "every checkpoint
  inside it returns a cancellation error" (`reference.md:661-662`) speak
  of calls made once the scope is cancelled, so they agree with a
  running call that returns its own result. Core 7.2's "are not, and are
  never interrupted" covers only non-checkpoints. The grep finds no
  remaining "consume a value" wording, and no claim that every in-flight
  operation is cancelled.
- **§0 and TASK-3's first paragraph.** A checkpoint is where a task
  observes cancellation (`03:56-59`). A running file call observes it at
  the next checkpoint, and one started in a cancelled scope returns at
  once (`03:1177-1179`), so neither contradicts "returns its
  cancellation error immediately".
- **§1.1 intro.** `BendFault::Cancelled` with `is_cancelled()`
  (`03:1730-1733`) meets "every capability error type has a cancellation
  variant" (`03:70-71`). `research/11:172`'s older `BendFault { code,
  message }` sketch is superseded by §10, as before.
- **Bend process mode.** A call that waits on a loopback TCP reply is a
  socket wait, so abandoning it and restarting the server is a real
  cancellation that returns `Cancelled`. Arguments are copied before
  the call, so dropping them on cancellation loses nothing the caller
  lent.
- **Factual claims.** Linux regular-file reads, writes, and `fsync` do
  sleep uninterruptibly (NFS waits are killable only), and an io_uring
  cancel cannot stop one that an io-wq worker is already running. Socket
  waits, pipe and terminal reads, `waitpid`, and timed sleeps are
  interruptible and cancellable through io_uring's async cancel or, under
  `threads` and `blocking`, a poll on the socket plus a wakeup
  descriptor. "Returns its own result" can be built under every runtime:
  under `blocking` and `threads` the system call simply returns; under
  `evented` the task waits for the completion entry, or for the blocking
  pool under the epoll fallback; the simulator completes the call at its
  modeled time. FUSE can interrupt a running read, a narrow exception
  that the rule's conservative stance tolerates.
- **Adversarial cases.**
  - `read_owned` or `write_owned` on a regular file: it runs to its end
    and returns `Ok(buf)`. Started in a cancelled scope, it returns
    `Err` with the buffer. On a socket it is cancelled and returns the
    buffer inside the error (except for N2's race). The kernel still
    writes only into runtime buffers (`03:144-147`).
  - `fs.write` cancelled between its calls: it stops (N3). `write_synced`
    cancelled before its `fsync` reports `Cancelled`, which is accurate
    because the data is not durable.
  - `Stdin`: a terminal or pipe read is cancellable, and a read from
    redirected regular-file input falls under the regular-file rule by
    kind. `lines()` still ends after a cancellation error (`03:134-136`).
  - Child pipe ends are pipes, so they are cancellable; reads and writes
    are checkpoints (`03:166-167`, `reference-companion.md:554-555`).
  - `Net::get` is all socket waits apart from N5's lookup, and its own
    deadline (`reference.md:670-672`) is unaffected.
- **TASK-5 and TASK-6.** "The last two can block on a network file
  system even inside a shielded scope with a deadline. Deadlines are
  therefore soft" (`03:1216-1218`) agrees with TASK-6's shielded cleanup
  that may `sync` (`03:1221-1226`). In-process Bend calls are covered
  by "foreign calls" in substance. The `warn!` heuristic concerns tasks
  that run without reaching a checkpoint, so a task suspended in a file
  call does not trigger it, which is correct.
- **§6.4, §7, FFI-5.** Channel error shapes are unchanged. The evented
  lowering and FFI-5's blocking pool already carry what "runs to its
  end" needs.
- **Core budget.** 24,996 of 25,000. The core changed by one phrase and
  needs no addition for B1 or B2.
- **Prose.** Plain and in the chapter's style apart from N6.
  `git diff --check` is clean, and no added line is over 80 columns.

## Out of scope

These predate the diff.

- **`fs.exists(p) -> bool`** (`reference.md:391`) is an `Fs` method,
  and §1.1 lists every `Fs` method as suspending and a checkpoint
  (`03:75`). It has no error type, so it cannot return the cancellation
  §1.1's intro promises (`03:70-71`). Rust's `try_exists` returns
  `io::Result<bool>` and would fix it.
- **A cancelled socket write loses its count.** "A cancelled socket
  write may already have reached the peer" (`03:1197-1198`). A write
  that sent `n` bytes and then returns `Cancelled` gives the caller no
  `n`, so a stream cannot be resumed or framed after it. Level-triggered
  cancellation usually means the stream is abandoned anyway.
- **Core 7.2's Cleanup bullet** ("give bounded cleanup its own inner
  scope with a deadline", `reference.md:683-684`) does not say that the
  deadline is soft (TASK-5). This has been true since `close` joined
  TASK-5. Adding it needs a core trim.
- **D8's "suspend only by using a capability"** (`01:511-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`),
  still open from rounds 80 to 90.
