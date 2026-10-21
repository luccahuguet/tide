# Tide adversarial/consistency review, round 97: TASK-3 scope (fx-ms8)

## Verdict

**accept**: no blocking findings, two non-blocking notes.

The revision closes round 96's B1, N1, N2, and N4. The abandonment rule
now names its one member, the name lookup, and a new sentence says that a
foreign call, an in-process Bend kernel among them, is not abandoned. That
removes the clash between TASK-3 and §10. TASK-5's entry uses TASK-3's own
words. The abandoned lookup's copy of the name is stated. TASK-3 is split
into three paragraphs. Word diffs show that the TASK-3 split and the
rewraps of TASK-5 and the §10 `Accel` bullet change no words beyond the
stated fixes. N3 was skipped as optional, and nothing in `design/`
contradicts that. The core is unchanged at 24,996 tokens and contradicts
nothing.

## What was reviewed

The uncommitted working tree against HEAD `e51564d`: `git diff --
design/`, SHA-256
`147ba5fd2613695293e32e171a1a6dd52a21118b84b6e1ae30a172f3f786a6ac`
(01, 03, core, companion). I recomputed the hash. `git diff --check`
passes, and no added line is over 80 columns.
`scripts/count-tokens.sh design/reference.md` gives 24,996, which
matches the companion's count line (`reference-companion.md:14`).

Word diffs:

- TASK-3 against round 96's text (taken from that round's transcript):
  "An argument [...] is dropped" became "Arguments [...] are dropped",
  "needs it again" became "needs one again" (N4), the lookup sentence was
  rewritten and extended (B1, N2), and a paragraph break was added before
  "An operation made of several system calls" (N4). Nothing else changed.
- TASK-5 against HEAD: the only changes are the added "a running system
  call the kernel cannot cancel (TASK-3)," and "(§1.1), which" becoming
  "(§1.1). The last two". The rest is reflow.
- The §10 `Accel` bullet against round 96's text: only "which is" was
  added before "a foreign call (TASK-5)" (N4). The rest is reflow.

I read:

- `AGENTS.md`; fx-ms8, its decision comment, and the revision comments
  after rounds 94, 95, and 96; `reviews/round96-consistency.md`;
- in 03: §0 (`03:31-62`), §1.1 with all its notes (`03:66-202`), §1.3's
  runtime table (`03:224-262`), §5.1 and §5.2 (`03:965-1007`), §6.1's
  limited `spawn`, §6.2 TASK-1 to TASK-6 (`03:1125-1237`), §6.3, §6.4
  (`03:1252-1290`), §7.2 and §7.3 (`03:1377-1415`), §9.3
  (`03:1550-1575`), §9.5 (`03:1593-1635`), §10 (`03:1705-1767`), and
  RUST-2 and RUST-3 (`03:1783-1795`);
- `01-design.md` D7 (`01:407-470`), D8 (`01:509-546`), and D16
  (`01:926-944`);
- core 6.1 (`reference.md:327-350`), 6.3's `Fs` and `File` block
  (`reference.md:380-396`), and 7.2 and 7.3 (`reference.md:659-699`);
- companion 15.2 (`reference-companion.md:184-246`), the simulation notes
  (`reference-companion.md:405-430`), and 20
  (`reference-companion.md:538-560`);
- a grep of `design/` for "in flight", "in-flight", "kernel operation",
  "consum", "abandon", "own result", "cannot be cancelled", "never
  interrupted", "runs to its end", `BendFault`, "name lookup",
  `getaddrinfo`, `resolve(`, "network file system", and "soft".

## Round 96 findings

- **B1, closed.** "A name lookup, a library call that cannot be cancelled
  and has no effect, is abandoned instead and returns the cancellation
  error; it runs on, on its own copy of the name, and its result is
  discarded. A foreign call, such as an in-process Bend kernel, is not
  abandoned (TASK-5, §10)." (`03:1198-1202`). The rule now has one member.
  A file-system call is a system call, not a library call, and the
  sentence before lists metadata queries and directory reads under
  run-to-end (`03:1193-1198`), so the lookup rule cannot be read to cover
  them. The next sentence excludes foreign calls by name, and §10 agrees
  (`03:1730-1734`). N1 below suggests a wording change.
- **N1, closed.** TASK-5 lists "a running system call the kernel cannot
  cancel (TASK-3)" (`03:1225`), which matches TASK-3's "A system call the
  kernel cannot cancel once running" (`03:1193-1194`). A FIFO or terminal
  read through a `File` is now outside TASK-5, as TASK-3 cancels it.
  "The last two can block on a network file system" (`03:1226`) still
  reads correctly.
- **N2, closed.** "it runs on, on its own copy of the name, and its
  result is discarded" (`03:1200-1201`) puts the memory obligation next to
  the rule. That agrees with the Owned-buffers note's "Cancellation never
  leaves the kernel writing into unwound Tide memory" (`03:144-147`).
- **N3, skipped as optional.** `is_terminal()` still has no stated
  simulated value. "Simulated standard input behaves as a pipe"
  (`03:1555-1556`) implies `false`, and nothing says otherwise.
- **N4, closed.** All three wording points are fixed: the plural subject
  (`03:1181-1184`), the paragraph break (`03:1203`), and "kernel, which
  is a foreign call (TASK-5)" (`03:1730-1731`).

## Blocking findings

None.

## Non-blocking notes

### N1. "Name lookup" and "foreign call" could each be pinned down

- "A name lookup" (`03:1198`) is safe in context, because its appositive
  says "a library call". But Linux calls path resolution "pathname
  lookup", and `fs.metadata(p)` and `fs.exists(p)` are lookups of a name
  with no effect. "A host-name lookup" would rule that reading out
  without the appositive. "On its own copy of the name" would then read
  "of the host name".
- "A foreign call" (`03:1201`) and TASK-5's "foreign calls" (`03:1224`)
  mean §5's calls and in-process Bend. `getaddrinfo`, which std calls
  inside `resolve`, is also a call into a C library, and RUST-3 lists it
  among linked symbols (`03:1791`). TASK-3 names the lookup explicitly,
  so the specific rule wins, but "A foreign call (§5)" would tie the term
  to its section. Both changes are optional.

### N2. Wording

- `03:1200`: "it runs on, on its own copy of the name" doubles "on".
  "It keeps running on its own copy of the name" says the same thing more
  plainly.

## Checked and consistent

- **The return rule** (`03:1179-1185`). It covers the payload of `send`
  on a full `sync_channel` and the buffers of `read_owned` and
  `write_owned`, and drops read-only arguments. An unbounded
  `Sender::send` never blocks and is not a checkpoint (`03:1276-1277`),
  so the "checkpoint" qualifier keeps it out. D8 (`01:521`) and core 7.2
  (`reference.md:668-669`) say "hand a value over". The Receivers note
  (`03:182-185`) no longer cites TASK-3 for `wait_with_output` and gives
  its own reason instead. Core 6.1 and 6.3 signatures (`reference.md:330`,
  `reference.md:384-388`) agree that `fs.write`, `write_synced`, and
  `connect` have no room for their arguments. `life_steps` takes `board`
  by value (`03:1716`), and a clone is the stated way to keep it.
- **§0 and §1.1 intro.** A checkpoint is where a task observes
  cancellation (`03:56-59`). A running uncancellable call observes it at
  the next checkpoint, so "returns its cancellation error immediately"
  (`03:1177-1178`) holds for checkpoints reached after the cancel.
  `BendFault::Cancelled` and its `is_cancelled()` (`03:1748-1749`) meet
  "every capability error type has a cancellation variant" (`03:70-71`).
- **Paths, Append, and atomic replace.** `remove_dir_all` stopping between
  paths (`03:104-106`) matches "checks before each one it starts"
  (`03:1204-1206`). In atomic replace (`03:115-117`), a running `rename`
  completes and returns `Ok`, and the following `sync_dir` returns
  `Cancelled`. That is accurate, since the replacement is visible but not
  yet durable, and §9.5 agrees (`03:1613-1614`).
- **Owned buffers and Receivers.** On a regular file, `read_owned` and
  `write_owned` run to their end and return `Ok(buf)`. On a socket or
  pipe they are cancelled, and the buffer comes back in the error once
  the kernel has let go of it. `wait_with_output`'s first system call is
  the unchecked close of `stdin`, which matches "except a close" and "the
  call has already closed `stdin`" (`03:187-188`).
- **Stdin note.** A terminal or pipe read is cancellable. A redirected
  regular file runs each read to its end and is checked before the next.
  `lines()` ends after a cancellation error (`03:134-136`). The simulator
  uses a pipe (`03:1555-1556`).
- **§5.** FFI-5 runs a capability wrapper's foreign call on the blocking
  pool while the task waits (`03:993-997`). TASK-3 says the call is not
  abandoned and TASK-5 says it is not interrupted, so the waiting task
  resumes only when the call returns. `#[trusted_pure]` calls are
  authority-free code, which TASK-5 already lists. RUST-2 wrappers carry
  FFI-3's claims (`03:1783-1788`) and fall under the same entry.
- **Under each runtime.**
  - *blocking, threads*: suspension is a blocking system call or a futex
    wait (`03:247-248`). A file call is checked before it starts and runs
    on the task's thread to its end. Socket, pipe, terminal, and sleep
    waits can be woken. A lookup runs on a helper thread and is
    abandoned. An in-process Bend call queued for the serialized runtime
    waits on a futex, which is cancellable. A running kernel runs on the
    task's thread to its end.
  - *evented (io_uring)*: the runtime checks before it submits. A running
    file SQE runs to its end. Socket and pipe SQEs are cancellable, and a
    CQE that carries a result returns it. Lookups and in-process Bend run
    on the blocking pool (FFI-5). One edge for implementers: a runtime
    cannot tell before an `open` whether the path names a FIFO or a
    terminal device, whose open waits cancellably. TASK-3 already gives
    the answer, since such an open is not "on a regular file or
    directory". So a runtime that never sends a cancel for file SQEs must
    still send one for opens. On NFS, where the wait is killable, the
    cancel has no effect and the open runs to its end, as the rule says.
  - *simulator*: a file call's modeled latency runs out first, socket and
    pipe waits wake on cancellation, standard input is a pipe
    (`03:1553-1556`), a lookup sits inside the round trip that
    cancellation cuts short (`03:1558-1559`), and a Bend call follows its
    declared mode (`03:1754`).
- **Adversarial cases.**
  - `fs.write`: the open truncates. A check before the first write can
    leave the target empty, as stated (`03:1205-1206`). The final close is
    unchecked, so a write that finished reports success.
  - `write_synced`: a running `fsync` completes. The close is unchecked,
    and then the directory open is checked and returns `Cancelled`, which
    is accurate before the entry is durable.
  - `read_dir`, `metadata`, `File::sync`: one system call each, or one
    getdents per iteration step, so each runs to its end. TASK-5 covers
    them, and TASK-6's shielded `sync` (`03:1234-1236`) agrees.
  - `Net::get`, `resolve`, and `connect` to a host name: socket waits are
    cancelled and the lookup is abandoned. Companion 15.2's "no request
    runs past the deadline" (`reference-companion.md:195`) and the
    simulation note's "fails only when its own deadline passes
    (`Timeout`) or its scope is cancelled (`Cancelled`)"
    (`reference-companion.md:424-425`) agree.
  - Child pipe ends, `Child::wait`, and `Signals::recv`: cancellable
    waits. A partly delivered write is the "may already have reached the
    peer" case (`03:1207-1208`).
  - Bend: process mode is a loopback socket wait. It is abandoned, the
    server restarts, and the call returns `Cancelled` unless the reply
    has arrived (`03:1728-1730`). In-process mode is a foreign call that
    finishes and returns its own result (`03:1730-1732`), now in
    agreement with TASK-3 (`03:1201-1202`). A queued call returns
    `Cancelled` (`03:1732-1734`). Under the simulator, cancellation
    follows the declared mode, and a `reference` implementation is Tide
    code, which TASK-5 lists as authority-free computation.
- **D7, D8, D16.** D7 says only that files support explicit durability
  (`01:432-433`). D8's "capability operations in a cancelled scope return
  `Err(Cancelled)`" (`01:517-518`) speaks of operations started in a
  cancelled scope. D16 does not describe cancellation.
- **Core and companion.** Core 7.2 (`reference.md:661-669`) and companion
  20 (`reference-companion.md:545-548`) contradict nothing. Core needs no
  change for N1 or N2.
- **Elsewhere in `design/`.** The grep turned up no other text on
  in-flight operations, abandonment, or Bend cancellation. In 02 and 04,
  "abandon" means a task or value abandoned by a panic (`02:729`,
  `04:866-874`), which is a different sense.
- **Prose.** Plain and in the chapter's style, apart from N2.

## Out of scope

These predate the diff.

- **A cancelled `output()` or `status()` can orphan its child** (round
  96). The revision comment says it is to be filed. `br list` shows no
  such bead yet.
- **Core 7.2 and the companion still describe deadlines as hard**
  (rounds 95 and 96; `reference.md:683-684`).
- **`fs.exists(p) -> bool`** (`reference.md:391`) is a checkpoint with
  no error type (rounds 94 to 96).
- **The error type of `read_owned` and `write_owned`** is not named
  anywhere in `design/` (rounds 95 and 96).
- **A cancelled socket write loses its count** (round 94).
- **D8's "suspend only by using a capability"** (`01:511-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`),
  still open from rounds 80 to 90.
- **D16's "values move or copy across"** (`01:937-938`) against 03 §10,
  where values are always copied (rounds 95 and 96).
