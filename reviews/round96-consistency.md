# Tide adversarial/consistency review, round 96: TASK-3 scope (fx-ms8)

## Verdict

**revise**: one blocking finding, four non-blocking notes.

The revision closes round 95's B1 and its notes N1, N2, N3, and N5. The
abandonment sentence is now a rule ("is abandoned instead and returns the
cancellation error") and names a library call, not a system call. A
regular-file read can no longer be abandoned, and TASK-5 needs no lookup
entry. The run-to-end rule is stated by what the kernel can cancel. Pipe
writes are back in. §9.3 points to TASK-3 and classifies simulated
standard input. The Bend bullets are reflowed and settled.

The narrowed sentence still has one problem. Its class, a library call
that the kernel cannot cancel and that has no effect, also takes in an
in-process Bend kernel. D16 makes that kernel pure, and §10 calls it a
foreign call. TASK-3 says such a call is abandoned and returns
`Cancelled`, while §10 says it finishes and returns its own result (B1).
The fix is one clause in TASK-3. The core is unchanged at 24,996 tokens
and contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `e51564d`: `git diff --
design/`, SHA-256
`7bf8a6b07b504949145f2a7d08c7cb99e13ca96cf1f81fd733ea80362d6e064a`
(01, 03, core, companion). I recomputed the hash. `git diff --check`
passes, and no added line is over 80 columns.
`scripts/count-tokens.sh design/reference.md` gives 24,996, which
matches the companion's count line (`reference-companion.md:14`).

I read:

- `AGENTS.md`; fx-ms8, its decision comment, and the "Revision after
  round 94" and "Revision after round 95" comments;
  `reviews/round95-consistency.md`;
- in 03: §0 (`03:31-62`), §1.1 with all its notes (`03:66-202`), §1.3's
  runtime table (`03:224-257`), FFI-5 and §5.2 (`03:993-1006`), §6.2
  TASK-1 to TASK-6 (`03:1125-1232`), §6.3, §6.4 (`03:1248-1285`), §9.2
  and §9.3 (`03:1526-1570`), §9.5 (`03:1589-1620`), §9.8, and §10
  (`03:1701-1761`);
- `01-design.md` D8 (`01:509-546`) and D16 (`01:926-944`), and a grep
  of 01 for "cancel";
- core 6.1's table (`reference.md:330-334`), the simulation table
  (`reference.md:524`), and 7.2 and 7.3 (`reference.md:659-699`);
- companion 15.2 (`reference-companion.md:185-200`), the simulation notes
  (`reference-companion.md:395-430`), and 20
  (`reference-companion.md:540-560`);
- a grep of `design/` for "in flight", "abandon", "own result", "kernel
  can", "cannot cancel", "library call", "name lookup", `BendFault`,
  "consum", `is_terminal`, simulated `stdin`, `resolve`, TASK-3, and
  TASK-5.

## Round 95 findings

- **B1, closed.** "A library call that the kernel cannot cancel and that
  has no effect, such as a name lookup, is abandoned instead and returns
  the cancellation error" (`03:1198-1200`). It is a rule, not a
  permission, so a hung resolver cannot hold a cancelled `resolve`,
  `connect("host:port")`, or `Net::get`. It names a library call, so it
  no longer reaches the file-system class, and TASK-5 needs no lookup
  entry. A new overreach is B1 below.
- **N1, closed.** "A cancelled socket or pipe write may already have
  reached the peer" (`03:1203-1204`), and "socket and pipe waits"
  (`03:1191`).
- **N2, closed.** "A system call the kernel cannot cancel once running,
  such as a file-system call on a regular file or directory (...), runs
  to its end" (`03:1193-1196`). The rule now goes by capability, with
  the file-system class as its main case. See N1 below for TASK-5.
- **N3, closed.** §9.3: "which cancellation cuts short only where TASK-3
  lets it: a file-system call's latency runs out first. Simulated
  standard input behaves as a pipe" (`03:1550-1552`). TASK-3 points back
  with "(§9.3)" (`03:1203`). Nothing else in `design/` says what
  simulated standard input is. §9.2's `world.stdin(impl AsRef<[u8]>)`
  (`03:1536`), core's `stdin("text")` (`reference.md:524`), and the
  companion's setup note (`reference-companion.md:415-416`) give only
  its bytes, so the sentence contradicts nothing.
- **N4, left open as stated.** Core 7.2's Cleanup bullet
  (`reference.md:683-684`) still does not say that deadlines are soft. It
  predates the diff (Out of scope).
- **N5, closed.** The Failures bullet is reflowed (`03:1741-1745`).
  In-process Bend is "a running kernel, a foreign call (TASK-5)"
  (`03:1726-1727`), which places it under TASK-5's "foreign calls".
  Process mode returns `Cancelled` "unless the reply has already arrived
  (TASK-3)" (`03:1725-1726`). Multi-call operations check "before each
  one it starts, except a close" (`03:1200-1201`).

## Blocking findings

### B1. The abandonment rule reaches an in-process Bend kernel

TASK-3:

> A library call that the kernel cannot cancel and that has no effect,
> such as a name lookup, is abandoned instead and returns the
> cancellation error. (`03:1198-1200`)

§10:

> In-process mode cannot stop a running kernel, a foreign call (TASK-5):
> it finishes and returns its own result, and the task observes the
> cancellation at its next checkpoint (`03:1726-1728`)

A stage-1 in-process Bend call is a call into Bend's native library
(`01:934-935`, `03:1705`). The OS kernel cannot cancel it, and it has no
effect: "kernels pure and total" (`01:939-940`), and "Kernel results are
pure" (`03:1746`). It is also a checkpoint (`03:1723`), so TASK-3
governs it. Read literally, TASK-3 says it is abandoned and returns
`BendFault::Cancelled`, and §10 says it returns its own result. Both are
implementable. Abandoning is memory-safe, because arguments are copied
across (`03:1733`, `03:1736-1738`): the task would stop waiting while the
kernel runs on and holds the serialized runtime. So neither text can be
read as an impossibility that defers to the other. An implementer of the
in-process runtime, or of the simulator, which "follows the same rules"
(`03:1203`), gets two answers.

The same class takes in foreign calls generally. A wrapper's foreign
call into a C library that does nothing observable is "a library call
that the kernel cannot cancel and that has no effect". TASK-5 lists
"foreign calls" among what cancellation cannot interrupt
(`03:1219-1220`), and round 95 held that an abandoned call has been
interrupted from the task's point of view. Most such wrappers are not
checkpoints, but one that takes a capability and runs on the blocking
pool (FFI-5, `03:993-997`) blocks the task the way a lookup does. The
"kernel" in the sentence adds to the confusion: in TASK-3 it is the OS
kernel, two sentences before §10's Bend "kernel", and a library call is
not something the OS kernel cancels anyway.

Every case the decision comment and round 95 named is a name lookup
(`getaddrinfo` inside `resolve`, `connect` to a name, and `Net::get`).

**Fix** (03 TASK-3, one sentence, no core change): name the member
instead of the class, and exclude foreign calls. For example: "A name
lookup, a library call that cannot be cancelled and has no effect, is
abandoned instead and returns the cancellation error; a foreign call,
such as an in-process Bend kernel, is not (TASK-5, §10)." If a broader
class is wanted, it must say "a call std makes into the C library" and
still exclude foreign calls.

## Non-blocking notes

### N1. TASK-5's file-system entry and TASK-3's rule differ in scope

TASK-3 now says that any system call the kernel cannot cancel runs to its
end, with file-system calls on regular files and directories as the main
case (`03:1193-1196`). TASK-5 lists "a file-system call already running
(TASK-3)" (`03:1221`). The two disagree at the edges in both directions:

- TASK-5 is wider. A `File` that `fs.open` returns for a FIFO or a
  terminal device is a file to the program, and its `read` is arguably a
  file-system call, but TASK-3 cancels it, because it is a pipe or
  terminal wait the kernel can cancel.
- TASK-5 is narrower. An uncancellable system call that is not a
  file-system call has no TASK-5 entry. No current operation falls
  there, so this is theoretical.

Matching TASK-5 to TASK-3's wording settles both: "a system call already
running that the kernel cannot cancel (TASK-3)". "The last two can block
on a network file system" still reads correctly after it.

### N2. An abandoned lookup must own what it reads

The Owned-buffers note makes the memory-safety promise for kernel I/O:
"Cancellation never leaves the kernel writing into unwound Tide memory"
(`03:144-147`). An abandoned lookup runs on after the task has returned
and unwound, under `threads` and `blocking` on a helper thread and under
`evented` on the blocking pool. So it must hold its own copy of the host
name and write its result into memory it owns, which it then discards.
TASK-3 says only that an argument "it only reads [...] is dropped with
the call" (`03:1181-1184`). For an abandoned call, that is true of the
caller's value only because the call took a copy first. The decision is
sound and implementable, since std owns `resolve`. A clause would put the
obligation next to the rule: "it runs on, on its own copy of the name,
and its result is discarded". Each abandoned lookup also holds its thread
until the resolver gives up, which glibc's defaults allow to take tens of
seconds. That costs threads, not correctness.

### N3. "Behaves as a pipe" says more than cancellation

"Simulated standard input behaves as a pipe" (`03:1551-1552`) settles the
cancellation class, which was the aim. It also implies that
`stdin.is_terminal()` is `false` under the simulator, and that a read can
be short, as pipe reads are. Both are good choices for a deterministic
model, and nothing contradicts them, but `is_terminal` (`03:83`,
`03:130`) has no simulated value stated anywhere else. "Simulated
standard input behaves as a pipe, so `is_terminal()` is `false`" would
make that explicit. This is optional.

### N4. Wording

- `03:1181-1185`: "An argument it only reads, such as `fs.write`'s `impl
  AsRef<[u8]>` data, a path, an address, or a Bend call's arguments,
  which are copied across (§10), is dropped" lists a plural item under a
  singular subject, and the line "Bend call's arguments, which are
  copied across (§10), is dropped" ends early in mid-paragraph. Writing
  "Arguments it only reads [...] are dropped" and reflowing fixes both.
- `03:1190-1204`: TASK-3's second paragraph states five rules in fifteen
  lines (cancellable calls, uncancellable calls, abandoned lookups,
  multi-call operations, and the simulator). A paragraph break before
  "An operation made of several system calls" would help a reader scan
  it, as TASK-2's paragraphs do.
- `03:1726`: "a running kernel, a foreign call (TASK-5)" works as an
  appositive, but "a running kernel, which is a foreign call (TASK-5)"
  reads more plainly, now that "kernel" means the OS kernel three
  bullets up in TASK-3.

## Checked and consistent

- **The return rule** (`03:1179-1188`). It covers `send`'s payload and
  `read_owned` and `write_owned` buffers, and drops read-only arguments.
  D8 (`01:521`) and core 7.2 (`reference.md:668-669`) say "hand a value
  over". The old "consume" phrasing survives only in the Receivers note,
  where it means `wait_with_output` taking `self` (`03:184`), not the
  return rule. Core 6.1's `io::Result` signatures (`reference.md:330`)
  agree that `fs.write` and `connect` have no room for their arguments.
- **§0 and §1.1 intro.** A checkpoint is where a task observes
  cancellation (`03:56-59`). A running file call, or a running in-process
  kernel, observes it at the next checkpoint, so "returns its
  cancellation error immediately" (`03:1177-1179`) holds for checkpoints
  reached after the cancel. `BendFault::Cancelled` and its
  `is_cancelled()` (`03:1743-1744`) meet §1.1's "every capability error
  type has a cancellation variant" (`03:70-71`).
- **Paths and Append notes.** `remove_dir_all` stopping between paths
  (`03:104-106`) matches "checks before each one it starts". Atomic
  replace (`03:115-117`): a running `rename` completes and returns `Ok`,
  and the following `sync_dir` starts in a cancelled scope and returns
  `Cancelled`. That is accurate, because the replacement is visible but
  not yet durable, and §9.5's "a `rename` either happened or did not"
  (`03:1609-1610`) agrees.
- **Owned buffers and the Receivers note.** `read_owned` and
  `write_owned` on a regular file run to their end and return `Ok(buf)`.
  On a socket or pipe they are cancelled, the buffer comes back in the
  error, and a completion that wins the race returns its result
  (`03:1192-1193`). "Except a close" agrees with the Receivers note's
  "the call has already closed `stdin` and taken `stdout` and `stderr`"
  (`03:187-189`): `wait_with_output`'s first system call is the unchecked
  close of `stdin`, and its first checked call is the read that follows.
  `File::close` is shielded and not a checkpoint (`03:192-199`).
- **Under each runtime.**
  - *blocking, threads*: suspension is a blocking system call or a futex
    wait (`03:247-248`). A file call is checked before it starts and then
    blocks on the task's thread, so it runs to its end by construction.
    Socket, pipe, and terminal waits use a poll with a wakeup descriptor,
    and a FIFO or terminal opened through `fs.open` is classified by
    `fstat` at open. A lookup is abandoned by running it on a helper
    thread, which `Net::get`'s own deadline already needs. An in-process
    Bend call queued for the serialized runtime waits on a futex, which
    is cancellable.
  - *evented (io_uring)*: the runtime checks before it submits. It never
    sends a cancel request for a file SQE, so a file call that a worker
    has started runs to its end. This also avoids an io-wq worker's
    interrupted sleep on NFS returning `EINTR` partway. Socket and pipe
    SQEs are poll-armed and cancellable, and a CQE that carries a result
    returns it. Lookups and in-process Bend run on the blocking pool
    (FFI-5).
  - *simulator*: a file call's modeled latency runs out first, socket
    and pipe waits wake on cancellation, and standard input is a pipe
    (`03:1549-1552`). A Bend call follows its declared mode (`03:1749`).
    A host that no peer names gives `Dns` at once (`03:1556-1557`), and a
    named one resolves inside the round trip, which cancellation cuts
    short in either reading.
- **Adversarial cases.**
  - `fs.write`: opened with truncation. A check before the first write
    can leave the target empty, as stated (`03:1201-1202`). The final
    close is unchecked, so a write that finished reports success.
  - `write_synced`: a running `fsync` completes. The unchecked close
    follows, and then the directory open is checked and returns
    `Cancelled`, which is accurate before the entry is durable.
  - `read_dir`, `metadata`, `File::sync`, `set_len`, `seek`: one system
    call each, or getdents per iteration step, so they run to their end.
    TASK-5 covers them, and TASK-6's shielded `sync` (`03:1230-1232`)
    agrees.
  - `Net::get`, `resolve`, `connect` to a name: socket waits are
    cancelled, and the lookup is abandoned. Companion 15.2's "no request
    runs past the deadline" (`reference-companion.md:195`) and the
    simulation note "fails only when its own deadline passes (`Timeout`)
    or its scope is cancelled (`Cancelled`)"
    (`reference-companion.md:424-425`) agree.
  - `Stdin`: a terminal or pipe is cancellable. A redirected regular file
    runs each read to its end and checks before the next. The simulator
    uses a pipe. `lines()` ends after a cancellation error (`03:134-136`).
  - Child pipe ends: cancellable waits. A partly delivered write is the
    "may already have reached the peer" case (`03:1203-1204`).
  - Bend: process mode is a loopback socket wait. It is abandoned, the
    server restarts, and the call returns `Cancelled` unless the reply
    has arrived (`03:1723-1726`). In-process mode is coherent with
    TASK-5 but not with TASK-3's abandonment sentence (B1). A call queued
    for the serialized runtime returns `Cancelled` (`03:1728-1729`).
- **D8, D16.** D8's "capability operations in a cancelled scope return
  `Err(Cancelled)`" (`01:517-518`) speaks of calls made in a cancelled
  scope. D16 does not describe cancellation, and its "kernels pure and
  total" (`01:939-940`) is what puts in-process Bend inside B1's class.
- **Core and companion.** Core 7.2 (`reference.md:661-669`) and
  companion 20 (`reference-companion.md:545-548`) contradict nothing.
  Core needs no change for B1 or N1 to N4.
- **Prose.** Plain and in the chapter's style, apart from N4.

## Out of scope

These predate the diff.

- **A cancelled `output()` or `status()` can orphan its child.** Both
  are checkpoints (`03:84`) that spawn a `Child` internally and take
  `&mut Command` (`03:182`). Cancelled after the spawn, the call drops
  that `Child` while it is still running, and the caller cannot `kill`
  and `wait` it. That is the outcome the Receivers note gives as the
  reason for `wait_with_output`'s `&mut self` (`03:183-186`). Nothing in
  `design/` says whether a cancelled `output` kills and reaps its child.
  The diff's "can stop partway" makes the gap easier to see, but it did
  not create it.
- **Core 7.2 and the companion still describe deadlines as hard**
  (round 95 N4; `reference.md:683-684`).
- **`fs.exists(p) -> bool`** (`reference.md:391`) is a checkpoint with
  no error type (rounds 94 and 95).
- **The error type of `read_owned` and `write_owned`** is not named
  anywhere in `design/` (round 95).
- **A cancelled socket write loses its count** (round 94).
- **D8's "suspend only by using a capability"** (`01:511-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`),
  still open from rounds 80 to 90.
- **D16's "values move or copy across"** (`01:937-938`) against 03 §10,
  where values are always copied (round 95).
