# Tide adversarial/consistency review, round 74: File::close fixes (fx-nyy)

## Verdict

**approve**: no blocking findings. There are two non-blocking notes and
one out-of-scope finding.

Round 73's B1, N1, and N3 are closed. TASK-5 now names `File::close`.
TASK-3 is back to the "Checkpoints that consume a value" wording and says
that close consumes its `File` but is not a checkpoint. The Receivers note
now gives the `io::Error` reason and the level-triggered retry reason. The
change as a whole agrees with 03 §0, §1.1, TASK-2 to TASK-6, JOIN-1 and
JOIN-2, §7, §9, core 6.3 and 7.2, and companion 20. TASK-5's new claim is
accurate (see "Checked and consistent"). N2 and N4 were left unapplied on
purpose, and I have no new evidence against either choice.

## What was reviewed

The uncommitted working tree against HEAD `579d785`: `git diff --
design/`, SHA-256
`56f73ed1affaaa1876a59e3f2b9f24e4bf18e378a8dca0b452fcd801fd079386`
(03, `reference.md`, `reference-companion.md`). I recomputed the hash.
`git diff --check -- design/` passes. I did not run
`scripts/count-tokens.sh`, as instructed.

I read:

- `AGENTS.md`, `GOVERNANCE.md`, fx-nyy (`br --no-db show fx-nyy`; it has
  no comments), and round 73;
- the whole diff;
- 01 D8 (`01:509-559`);
- in 03: §0 Terms (`03:31-62`), §1.1's intro, table, and notes
  (`03:66-201`), §1.2's `Task` row (`03:209`), §6.1's signatures
  (`03:1030-1050`), TASK-2 to TASK-6 (`03:1062-1132`), JOIN-1 and JOIN-2
  (`03:1136-1144`), §7 (`03:1249-1311`), §9.3 to §9.5
  (`03:1446-1528`), and §13.2 (`03:1950-1957`);
- in the core: 6.3 (`reference.md:378-399`) and 7.2
  (`reference.md:659-684`);
- companion 20 (`reference-companion.md:540-570`) and the count line
  (`:14`);
- a grep of `design/` and `trials/` for `shield`, `close(self)`,
  "consume a value", and "consuming checkpoint".

## Status of round 73's items

| Item | Status | Where |
| --- | --- | --- |
| B1. TASK-5 omits `File::close` | closed | `03:1120-1123` |
| N1. Narrowed TASK-3 wording | closed: TASK-3 says "Checkpoints that consume a value", and the Receivers note's "as TASK-3 asks of consuming checkpoints" (`03:194`) matches it again | `03:1101-1103` |
| N2. D8's cancellation sentence | not applied, on purpose; no new evidence | `01:517-518` |
| N3. Missing reasons in the Receivers note | closed: the `io::Error` and level-triggered retry reasons are present; the deadline cost moved to TASK-5, which is where B1 put it | `03:193-197` |
| N4. Core 7.2's literal reading | not applied, on purpose (core at 24,999 of 25,000); no new evidence | `reference.md:665-667` |
| Out-of-scope: by-value arguments, `spawn` under `limit` | to be reported to the owner; unchanged by this diff | — |

## Blocking findings

None.

## Non-blocking notes

### N1. "Shielded" means two things within one note

The Receivers note uses "shielded" first for close itself, meaning not a
checkpoint, as in JOIN-1 and JOIN-2 ("`close` is shielded", `03:190`).
Six lines later it uses the word for a call run under `tasks.shield`
("a retry that, cancellation being level-triggered, fails again unless
shielded", `03:196-197`). These are different mechanisms. A retry under
`shield` would work because `shield` ignores the enclosing cancellation,
not because the retry has stopped being a checkpoint. Round 73 noted that
03 uses the word in both senses (TASK-6's "shielded scope" against
JOIN-1's "shielded"). This is the first place where both senses fall in
one paragraph about one operation, so a reader can take "unless shielded"
to mean the decision that the note is in the middle of justifying. TASK-5
also says "shielded scope" (`03:1123`), but TASK-6 owns that phrase and
the meaning is clear there.

**Fix** (03 Receivers note): "... leave the caller a retry that,
cancellation being level-triggered, fails again unless run under
`shield`."

### N2. Record the decision and the unapplied notes on fx-nyy

fx-nyy has no comments. AGENTS.md ("Decisions") asks for the decision
and its reasons on the bead as well as in the owning design document.
Round 73's N2 also asked that, if 01 is left alone, the bead record that
D8's "capability operations in a cancelled scope return
`Err(Cancelled)`" is read through 03 §0. N4 likewise rests on the core
budget.

**Fix** (fx-nyy, before closing): one comment that states the decision
(close is shielded: not a checkpoint, always closes the descriptor,
returns close(2)'s result), its reasons as given in 03 §1.1, the handle
sweep's result (no other consuming checkpoint), and why round 73's N2
(D8 is read through 03 §0) and N4 (no token to spare in the core) were
not applied.

## Out-of-scope findings

This predates the diff, and the new TASK-5 clause makes it more visible.
It is a candidate follow-up bead, not a condition for fx-nyy.

- **Checkpoints whose kernel operation cannot be cancelled.** TASK-3
  says "An in-flight kernel operation is cancelled before the error
  returns" (`03:1103-1104`). On Linux, regular-file reads, writes, and
  `fsync` sleep uninterruptibly. On NFS, for example, a writeback or
  commit can wait on the server for a long time. Neither a signal under
  the threads runtime nor an io_uring cancel request reliably stops a
  regular-file operation that is already running. Such a checkpoint
  therefore returns its cancellation error only when the kernel
  operation ends, and it can overrun a shielded scope's deadline just as
  close can. TASK-5's new clause says close "can block on a network file
  system even inside a shielded scope with a deadline" (`03:1122-1123`).
  That is true, but because no other operation is named, a reader may
  conclude that `sync` or `write` on the same file system is bounded by
  that deadline. A fix would add to TASK-3: "when the kernel cannot
  cancel it, as with regular-file I/O, the error returns once it
  completes"; and to TASK-5: "and the kernel part of a checkpoint that
  cannot be cancelled (TASK-3)".

## Checked and consistent

- **§0 and §1.1.** §0: "except the shielded ones marked in §1.1, such as
  `File::close`" (`03:56-59`). The §1.1 intro: "all are checkpoints except
  those marked shielded" (`03:69-70`). The table marks close (`03:76`) and
  `Tasks`' `scope` exit and `spawn_isolated` (`03:85`). These were
  unchanged since round 73 and are still consistent.
- **TASK-3** (`03:1099-1105`). The rule applies only to checkpoints, so
  close is outside it by definition. The added clause explains this
  rather than creating a second exception. "An in-flight kernel operation
  is cancelled" has nothing to act on for close: close takes `self`, so
  no borrow of the `File` can have an operation in flight.
- **TASK-5's new claim** (`03:1120-1123`), "which can block on a network
  file system even inside a shielded scope with a deadline". The claim is
  accurate. Linux runs the file system's flush on the last close of a
  descriptor. NFS's flush writes back dirty pages for close-to-open
  consistency and waits for the server. FUSE and some devices also block
  in close. Close is not a checkpoint, so the cancellation that a
  `shield` scope's own deadline triggers is never observed during it.
  "Shielded scope with a deadline" is TASK-6's own phrase for bounded
  cleanup (`03:1128`), so the clause tells the reader of TASK-6 exactly
  which bound close ignores. Under the evented runtime, "block" means the
  task stays suspended, not that a worker is blocked. TASK-5 used that
  loose sense for joins already. "Can" makes it a possibility, not a
  guarantee. The 100 ms `warn!` that follows counts running time without
  a checkpoint, so a long close goes unreported, as a long join already
  does.
- **TASK-6** (`03:1127-1132`) matches §1.1 and TASK-5: close is shielded
  by itself, and other cleanup I/O runs under `shield`.
- **TASK-2 and TASK-4.** Close never returns a cancellation error, so it
  never sets the echo bit. A deadline that passes during a close is
  observed at the next checkpoint.
- **JOIN-1 and JOIN-2** (`03:1136-1144`) use "shielded" to mean not a
  checkpoint, which is the sense the table, §0, TASK-3, TASK-6's first
  clause, and the note's first use share.
- **The Receivers note** (`03:182-197`). The `io::Error` reason now
  matches the `wait_with_output` argument six lines earlier. "Returning
  the `File` inside one" refers back to "a cancellation error". "Like
  `drop`, it cannot be interrupted (TASK-5)" points to the rule that now
  lists close.
- **01 D8** (`01:517-521`). Nothing new contradicts it beyond what round
  73's N2 already recorded. "operations that consume a value return it
  inside the error" holds vacuously for close, because close never
  returns a cancellation error.
- **§7.** Close may suspend, so a function that calls it may suspend
  (SUSP-1), as before. Under the blocking and thread runtimes, suspension
  blocks the OS thread (`03:1307-1308`), which is TASK-5's literal case.
- **§9.** Close remains a scheduling point (`03:1460-1462`). Nothing in
  the diff says close makes data durable. Durability still comes only
  from `sync` and `sync_dir` (`03:1500-1501`).
- **Core 6.3 and 7.2, companion 20.** These are unchanged since round 73.
  7.2 lists `File::close` among the operations that are not checkpoints
  and are "never interrupted", which agrees with TASK-5. Companion 20
  (`reference-companion.md:569`) still states only the receiver, and
  that is enough. The count line (24,999 on 2026-10-21) was not
  recounted.
- **Stale guidance.** No text in `design/` or `trials/` still says to call
  close under `shield`. The old TASK-6 sentence was the only one.

## Prose and wrapping

Every added prose line is at most 76 columns, and the table row is
exempt. The wraps are not filled greedily, which matches the rest of 03
and is not a finding. For example, "joins," would fit at the end of
`03:1121`. "Deadlines are therefore soft." starts a new line after the
shortened `03:1123`. It did the same after the old "joins." line, so the
layout is unchanged. The prose is plain and specific.
