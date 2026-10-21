# Tide adversarial/consistency review, round 95: TASK-3 scope (fx-ms8)

## Verdict

**revise**: one blocking finding, five non-blocking notes.

The revision closes all of round 94's blocking findings and the notes it
took up. TASK-3 now names the uncancellable class: file-system calls on
regular files and directories (B1). §10 says what an in-process Bend
call returns once its kernel finishes, and what a call queued behind the
serialized runtime returns (B2). "System call" replaces "kernel call"
(N1). The race clause is in (N2). Multi-call operations check "before
each one it starts, except a final close", and the text says that
`fs.write` can leave its target truncated (N3). The simulator is bound to
the same rules (N4). Every claim the new text makes holds under the
blocking, threads, evented, and simulated runtimes.

One new sentence is not settled. "A call with no effect, such as a name
lookup, may instead be abandoned" comes right after the rule that
file-system calls run to their end. Read with that rule, it allows a
runtime to abandon a running `metadata` query or regular-file read,
which contradicts TASK-5. Its "may" also leaves out of TASK-5 a lookup
that a runtime does not abandon (B1). The fix is one sentence in TASK-3.
The core is still 24,996 tokens and contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `e51564d`: `git diff --
design/`, SHA-256
`a3c5c76877ffb90b49772e35afbd957c894f2a91938a3925d45a09e20c2830da`
(01, 03, core, companion). I recomputed the hash. `git diff --check`
passes, and no added line is over 80 columns.
`scripts/count-tokens.sh design/reference.md` gives 24,996, which
matches the companion's count line (`reference-companion.md:14`).

I read:

- `AGENTS.md`; fx-ms8, its decision comment, and the "Revision after
  round 94" comment; `reviews/round94-consistency.md`;
- in 03: §0 (`03:31-62`), §1.1 with all its notes (`03:66-202`), §1.2,
  §1.3's runtime table (`03:224-257`), §5.1 FFI-5 (`03:993-997`), §6.1's
  API, §6.2 TASK-1 to TASK-6 (`03:1125-1231`), §6.3, §6.4
  (`03:1233-1284`), §7.2 and §7.3 (`03:1372-1410`), §9.1 to §9.8
  (`03:1512-1696`), §10 (`03:1698-1758`), and the worked examples 12.2
  and 12.3 (`03:1861-1982`);
- `01-design.md` D8 (`01:509-531`) and D16 (`01:926-944`), and a grep
  for D7's durability text;
- core 6.1 (`reference.md:320-350`), 6.3 (`reference.md:379-427`), 7.2
  (`reference.md:659-684`), and 7.3 (`reference.md:686-699`);
- companion 15.2 (`reference-companion.md:184-246`), the simulation notes
  (`reference-companion.md:395-425`), and 20
  (`reference-companion.md:545-555`);
- a grep of `design/` for "in flight", "consum", "kernel call",
  "abandon", "own result", "uninterrupt", `BendFault`, `is_cancelled`,
  `read_owned`, `write_synced`, TASK-3, and TASK-5.

## Round 94 findings

- **B1, closed.** TASK-3 names the class, "File-system calls on regular
  files and directories, such as reads, writes, `fsync`, opens, renames,
  removals, metadata queries, and directory reads" (`03:1193-1195`), and
  TASK-5 lists "a file-system call already running (TASK-3)"
  (`03:1220`). That covers `create`, `rename`, `sync_dir`, `read_dir`
  iteration, `DirEntry::metadata`, and `File::set_len`.
- **B2, closed.** In-process mode: the kernel "finishes and returns its
  own result, and the task observes the cancellation at its next
  checkpoint; a call still waiting for the serialized runtime returns
  `BendFault::Cancelled`" (`03:1722-1726`).
- **N1, closed.** TASK-3 says "system call" (`03:1190`) and TASK-5
  "file-system call" (`03:1220`). "Kernel" now means a Bend kernel
  everywhere else in 03.
- **N2, closed.** "one that completes before the cancel takes effect
  returns its own result" (`03:1192-1193`).
- **N3, closed.** "checks before each one it starts, except a final
  close" and "a cancelled `fs.write` can leave its target truncated"
  (`03:1199-1202`). The optional pointer to atomic replace was not added,
  which is fine.
- **N4, closed in substance.** "The simulator follows the same rules"
  (`03:1202`) and "cancellation follows the declared mode"
  (`03:1746`). Note N3 below asks for a pointer from §9.
- **N5.** Folded into TASK-3 as the abandonment sentence. See B1.
- **N6, closed.** "copied across (§10)" (`03:1183`). TASK-5 is reflowed.
  The "Unlike `send`" marker was skipped as a nit, which I accept.

## Blocking findings

### B1. The abandonment sentence reaches file-system calls

TASK-3 says:

> File-system calls on regular files and directories, such as reads,
> writes, `fsync`, opens, renames, removals, metadata queries, and
> directory reads, cannot be cancelled once running: such a call runs to
> its end and returns its own result [...]. A call with no effect, such
> as a name lookup, may instead be abandoned. (`03:1193-1199`)

The sentence has two problems.

**Its scope.** "Instead" follows straight after "runs to its end and
returns its own result", so the natural reading is that the permission
is an exception to that rule. The class it names, a call with no effect,
includes members of the file-system list: a `metadata` query, an
`exists`, a directory read, and arguably a regular-file read, which
changes nothing outside the process. "No effect" is not defined. A read
advances the file offset, and an `open` allocates a descriptor. Under
this reading:

- a `threads` runtime may run `File::read` on a helper thread and return
  `Cancelled` while the helper finishes. The offset has advanced past
  bytes the program never sees, so a shielded cleanup that keeps reading
  from the same `File` silently skips data. That is the hidden-effect
  outcome the fx-ms8 decision rejected;
- TASK-5 says that "a file-system call already running" cannot be
  interrupted (`03:1218-1220`), but an abandoned `metadata` call has been
  interrupted from the task's point of view;
- the simulator, which "follows the same rules", has two choices for a
  pending `metadata` call when cancellation arrives: `Ok(meta)` at its
  modeled time, or `Cancelled` at once.

The decision comment's own example, and round 94's N5 that it answers,
concern `getaddrinfo`. That is a library call that is neither a
cancellable system call nor a file-system call. The intended scope is
probably "a call the first two rules do not cover".

**Its "may".** A runtime that does not abandon a lookup leaves a
`resolve`, or the lookup inside `connect("host:port")` or `Net::get`,
blocked through a hung resolver. glibc's defaults (5 s timeout, 2
attempts, per name server) allow tens of seconds. TASK-5, the list a
reader uses to size cleanup deadlines, does not name lookups, so under
"may" it is incomplete. Every runtime already needs the abandonment
mechanism: `net.get(url, deadline)` must return `Timeout` when
`deadline` passes (`reference.md:670-672`), and companion 15.2 promises
that "no request runs past the deadline" (`reference-companion.md:195`),
with no exception for a slow lookup. Requiring abandonment costs nothing
new.

**Fix** (03 TASK-3, one sentence): scope the permission and make it a
rule. For example: "A library call that the kernel cannot cancel and
that has no effect, such as a name lookup, is abandoned instead." This
leaves the file-system rule without exceptions, needs nothing in
TASK-5, and lets the simulator abandon. If "may" is kept on purpose,
TASK-5 must list "a name lookup a runtime does not abandon", and the
sentence must still say that it does not reach file-system calls.

## Non-blocking notes

### N1. "Socket write" now leaves out pipe writes

The old text said "A cancelled write may already have reached the
peer". The diff narrows it to "A cancelled socket write"
(`03:1202-1203`), presumably because a regular-file write now runs to
its end. Pipes are cancellable too: a `ChildStdin` write, or a write to
any pipe end (`03:166-167`, `reference-companion.md:554-555`). A large
pipe write cancelled partway has already delivered part of its bytes to
the child. As worded, the sentence suggests that only sockets can do
this. "A cancelled socket or pipe write" restores the old coverage.

### N2. Uncancellable calls outside the file-system class have no rule

The first sentence covers calls the kernel can cancel: "is cancelled
before the error returns if the kernel can cancel it" (`03:1190-1191`).
The second covers a named class, file-system calls on regular files and
directories. Between them is the case of a system call that the kernel
cannot cancel and that is not on a regular file or directory, such as
`stat` or `rename` of a FIFO or socket path, or an `open` of a device
whose driver sleeps uninterruptibly. Each is rare, and the general
principle is plain. Stating the rule by capability, with the file-system
class as its main example, closes the gap: "A system call the kernel
cannot cancel, such as a file-system call on a regular file or directory
(reads, writes, ...), runs to its end and returns its own result."

### N3. §9 does not point to the cancellation rules

TASK-3 binds the simulator to "the same rules", and §9.3's model is
compatible with them. A task in a capability call waits out the call's
modeled latency (`03:1548-1549`). Cancellation can wake a pending socket
wait at once, and a pending file call completes at its modeled time.
§9 does not say this, though, and a reader of §9.3 alone would expect
cancellation to end every modeled latency. One case is also
unclassified. The rule goes by the kind of object, and `world.stdin`
(`03:1535`) does not say whether simulated standard input is a pipe
(cancellable) or a regular file (runs to its end). A clause in §9.3,
"a pending call ends early on cancellation only where TASK-3 allows,
and simulated standard input is a pipe", anchors both. The §10
Simulation bullet already does this for Bend (`03:1746`).

### N4. Core and companion still describe deadlines as hard

Round 94 placed core 7.2's Cleanup bullet out of scope, because it
does not say that deadlines are soft (`reference.md:683-684`). The diff
widens that gap from `File::close` to every running file-system call.
Core 7.2 contrasts checkpoints with operations that "are not, and are
never interrupted" (`reference.md:665-667`), and an agent that loads only
the reference pair will size a cleanup deadline around `file.sync()` as
if the deadline bounded it. The core is at 24,996 of 25,000 tokens, but
the companion is outside the budget. One sentence there, for example in
the simulation notes or beside section 20, would reach agents at no core
cost: "A running file-system call, like `File::close`, finishes even
after its scope is cancelled (03 TASK-3, TASK-5); on a network file
system, a cleanup deadline is soft."

### N5. Wording

- `03:1741`: "whose `is_cancelled()` is true (§1.1). A kernel that"
  leaves a 55-column line in the middle of the Failures bullet.
  Reflowing the bullet fixes it.
- `03:1723`: "like a foreign call (TASK-5)" implies that an in-process
  Bend call is not one of TASK-5's "foreign calls", so TASK-5's list
  (`03:1218-1220`) appears to omit a running in-process kernel. "as a
  foreign call does" would settle that an in-process call is covered
  there.
- `03:1720-1722`: process mode "returns `BendFault::Cancelled`"
  unconditionally, but a reply already read when the cancel arrives
  falls under TASK-3's race clause and returns its own result. TASK-3
  governs, so this is a reading nit.
- `03:1199-1200`: "except a final close" leaves a check before
  `write_synced`'s close of the file, which is not final because the
  directory sync follows it. A check there is harmless, since
  `Cancelled` is accurate before the entry is durable and `drop` closes
  the descriptor. "except a close" would be simpler.

## Checked and consistent

- **The return rule.** No change since round 94: it covers `send`'s
  payload and `read_owned` and `write_owned` buffers (`03:1179-1181`,
  `03:144-147`, `03:1264`), and drops read-only arguments. D8 (`01:521`)
  and core 7.2 (`reference.md:668-669`) say "hand a value over". Core 6.3's
  signatures (`io::Result`, `reference.md:382-398`) settle that
  `fs.write`'s data is not handed over.
- **Receivers note** (`03:183-199`). "a cancelled call that consumed a
  running `Child` would drop it where the caller could not `kill` and
  `wait` it; an `io::Error` cannot carry it back" no longer relies on
  TASK-3, and "as `send` returns its payload" keeps the argument.
  Companion 20 (`reference-companion.md:545-548`) agrees.
- **§0 and §1.1 intro.** A checkpoint is where a task observes
  cancellation (`03:56-59`). A file call already running observes it at
  the next checkpoint, so nothing contradicts "returns its cancellation
  error immediately". `BendFault::Cancelled` with `is_cancelled()`
  (`03:1740-1741`) meets §1.1's "every capability error type has a
  cancellation variant" (`03:70-71`) and core 6.1 (`reference.md:349-350`).
- **Paths and Append notes.** `remove_dir_all` stopping between paths
  (`03:104-106`) matches "checks before each one it starts". Atomic
  replace (`03:115-117`, `reference.md:421-423`) under cancellation: a
  running `rename` completes and returns `Ok`, and the following
  `sync_dir` starts in a cancelled scope and returns `Cancelled`. That is
  accurate, because the replacement is visible but not yet durable, and
  §9.5's "a `rename` either happened or did not" (`03:1606-1607`) agrees
  with the program's view.
- **Under each runtime.**
  - *blocking, threads*: a file call is a blocking system call on the
    task's thread, checked before it starts, so it runs to its end by
    construction. Socket and pipe waits use a poll with a wakeup
    descriptor. If both are ready, the runtime either performs the call,
    which then completes and returns its result, or returns `Cancelled`
    without starting it. Either matches the race clause. Abandoning a
    lookup needs a helper thread, which `Net::get`'s deadline already
    requires (B1).
  - *evented*: a file SQE runs in an io-wq worker, or on the blocking
    pool under the epoll fallback (FFI-5, `03:993-997`). A cancel request
    stops one only before a worker starts it. A cancelled `recv`,
    `accept`, or `waitpid` whose completion entry carries a result
    returns that result, and the runtime waits for the original entry
    before releasing an owned buffer (`03:144-147`). FUSE can interrupt a
    running read, and round 94 already accepted that narrow exception.
  - *simulator*: file calls complete at their modeled time, and socket
    waits wake on cancellation (see N3). A simulated Bend call follows
    its declared mode, including the queue behind the in-process lock.
- **Adversarial cases.**
  - `fs.write`, `create`: opened with truncation. A check before the
    first write can leave the target empty, as stated. A running `create`
    returns its `File`, which the caller drops while unwinding, so no
    descriptor leaks.
  - `write_synced`: `Cancelled` before its `fsync` or directory sync is
    accurate, because its promise is durability.
  - `rename`, `metadata`, `read_dir` iteration, `File::sync`: all run to
    their end (except under B1's reading).
  - `exists` returns `bool`, so it cannot report cancellation at entry.
    This predates the diff (Out of scope). Abandoning it, as B1's reading
    allows, would leave it with no value to return.
  - `read_owned` and `write_owned`: on a file they run to their end and
    return `Ok(buf)`. On a socket they are cancelled, and a completion
    that wins the race returns its result (`03:1192-1193`).
  - `Net::get`, `resolve`, `connect` to a name: socket waits are
    cancellable, and the lookup is B1.
  - `Stdin`: a terminal or pipe is cancellable, and a redirected regular
    file runs to its end, by kind.
  - Child pipe ends: cancellable (N1 for writes).
  - Bend: process mode is a loopback socket wait, abandoned and
    restarted, and returns `Cancelled`. In-process mode lets a running
    kernel finish and return its result. A call queued behind the
    serialized runtime returns `Cancelled`, and under `threads` the wait
    for the runtime's lock needs a cancellable wait, which is
    implementable.
- **TASK-5 and TASK-6.** "The last two can block on a network file
  system even inside a shielded scope with a deadline. Deadlines are
  therefore soft" (`03:1220-1222`) agrees with TASK-6's shielded cleanup
  that may `sync` (`03:1226-1231`). A task suspended in a file call is
  not "running without reaching a checkpoint", so it does not trigger the
  `warn!` heuristic.
- **Worked examples.** 12.2's `get` sees the scope deadline's
  cancellation (`03:1895-1903`). 12.3's `put` calls `write_synced`
  outside the deadline scope (`03:1938-1948`). Neither depends on a
  running file call being cancelled.
- **D8, D16.** D8's "capability operations in a cancelled scope return
  `Err(Cancelled)`" (`01:517-518`) speaks of calls made in a cancelled
  scope. D16 does not describe cancellation.
- **Core budget.** 24,996 of 25,000. B1 needs no core change. N4 is a
  companion addition.
- **Prose.** Plain and in the chapter's style apart from N5. The TASK-3
  bullet's second paragraph follows TASK-2's two-paragraph form.

## Out of scope

These predate the diff.

- **`fs.exists(p) -> bool`** (`reference.md:391`) is still a checkpoint
  with no error type (round 94). Rust's `try_exists` returns
  `io::Result<bool>`.
- **The error type of `read_owned` and `write_owned`** is not named
  anywhere in `design/`. §1.1 says they "return it, inside the error on
  failure" (`03:146`), and TASK-3 now cites them. An `io::Error` cannot
  hold the `Vec<u8>`, so the type is still open.
- **A cancelled socket write loses its count** (round 94).
- **D8's "suspend only by using a capability"** (`01:511-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`),
  still open from rounds 80 to 90.
- **D16's "values move or copy across"** (`01:937-938`) against 03 §10,
  where values are always copied (`03:1730-1737`). TASK-3's "copied
  across (§10)" follows 03. If a stage-1 in-process call ever moves a
  `BendArray`, the argument would be handed over and TASK-3 would need to
  say so.
