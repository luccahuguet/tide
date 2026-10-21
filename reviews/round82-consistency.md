# Tide adversarial/consistency review, round 82: limited spawn and cancellation (fx-sz7)

## Verdict

**revise**: one blocking finding and four non-blocking notes.

The reversal closes round 81's B1: the slot wait is now a checkpoint, so
no limited `spawn` waits past its caller's cancellation for slots held by
other scopes. Round 81's N1 to N4 and round 80's N1, N3, and N4 are
closed or moot. The decision agrees with §0, §1.1, §1.2, TASK-2 to
TASK-6, JOIN-1 and JOIN-2, §9.3, D8, 02 §8.2 and §8.3, core 7.1 and 7.2,
and companion 14.3 and 15.2. Companion 15.2's edited example and its
test still hold. One case is left open: a limited `spawn` made from
inside a cancelled nested scope into an enclosing scope that is not
cancelled. Read with TASK-3 as written, it fails the enclosing scope.
Read the other way, it brings back round 81's unbounded wait (B1).

## What was reviewed

The uncommitted working tree against HEAD `db1bebc`: `git diff --
design/`, SHA-256
`7203f6c5b8a09330a567763bb8e78be3dee883a3dea862fab0ba5a0c70219dff`
(03, `reference.md`, `reference-companion.md`). I recomputed the hash.
`git diff --check` passes. The only added lines past 80 columns are the
two table rows (`03:85`, `03:210`), like every row in those tables.
`scripts/count-tokens.sh design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-sz7 with all three comments (the last, "Reversal after
  round 81", is the decision under review), and
  `reviews/round80-consistency.md` and `reviews/round81-consistency.md`;
- the whole diff;
- in 03: §0 Terms (`03:31-63`), §1.1's table and notes (`03:67-203`),
  §1.2 (`03:205-223`), §1.3 (`03:225-263`), §6.1 to §6.6
  (`03:1043-1292`), §7.1, §9.3 (`03:1490-1512`), and the §12.2 fetcher
  (`03:1798-1838`) and §12.3 server (`03:1840-1880`);
- `01-design.md` D5, D7's `Tasks` lines, and D8 (`01:509-564`);
- `02-memory.md` §8.2 and §8.3 (`02:657-734`);
- core 6.8's time and order bullets (`reference.md:530-541`), 7.1, 7.2,
  and 7.3 (`reference.md:574-699`);
- companion 14.3 (`reference-companion.md:86-99`) and 15.2
  (`reference-companion.md:184-247`);
- a grep of `design/` for `limit`, `spawn`, `checkpoint`, `shielded`,
  `deadlock`, "never interrupted", and "cannot be cancelled".

## Earlier findings

- **Round 81 B1, closed.** The wait is a checkpoint (`03:1078`), and the
  stated reason is the shared count (`03:1088-1091`). A spawn whose
  caller is cancelled no longer waits on slots held by other scopes'
  children (but see B1 below for which cancellation it observes).
- **Round 81 N1, closed.** `03:1101-1102`: "the simulator reports a
  global deadlock once no task can run (§9.3)", which matches §9.3's "If
  no task can ever run again" (`03:1501-1503`). A pending deadline is a
  timer, so virtual time advances to it and the cancellation breaks the
  cycle, as `03:1100-1101` says.
- **Round 81 N2, moot.** The advice to give nested work "its own
  `Tasks`" is gone. How `limit` chains on a limited `Tasks` is still
  unstated; it predates the diff (Out of scope).
- **Round 81 N3, closed.** Core 7.1 adds "A child spawning under its own
  limit can deadlock it." (`reference.md:637-638`) and drops the
  shielded-wait sentence, at 24,995 tokens.
- **Round 81 N4, moot.** The sentences it named are gone.
- **Round 80 N1, closed.** `03:1078-1080` and `03:1095-1096` give both
  sides: `spawn` keeps `std::thread::scope`'s shape, and a `Result` would
  "put a `?` on every spawn for a wait that only limited scopes have".
- **Round 80 N3, moot.** The body no longer needs `tasks.checkpoint()`
  to stop spawning: each limited `spawn` in a cancelled scope starts
  nothing.
- **Round 80 N4, moot.** The sentences it named are gone.

## Blocking findings

### B1. A limited `spawn` from a cancelled nested scope into an enclosing scope fails that enclosing scope

`03:1080-1081`: "in a cancelled scope a limited `spawn` returns at once
and starts nothing". This does not say whose cancellation counts: that
of the scope spawned into, or that of the calling task. The two differ
when the caller sits in a scope nested inside the target, which 02 §8.2
endorses twice: "Grandchildren" (`02:694`) and "A scope passed down"
(`02:696-703`, `reference.md:639-642`). The caller is always inside the
target, because a `Scope` cannot leave its origin, so a cancelled target
implies a cancelled caller (TASK-2, `03:1115-1116`). The reverse does
not hold.

Example: a server opens `lim.scope(|bg| ..)` with `lim =
root.tasks.limit(64)`, and a connection handler running in `bg` opens a
per-request `tasks.scope(|s| { s.set_deadline(t); bg.spawn(move ||
log_it(..)); .. })`. The request's deadline passes while the
`bg.spawn` waits, because long connections hold all 64 slots.

- **Caller's cancellation (what TASK-3 requires).** TASK-3: "every
  checkpoint inside it, in any task, returns its cancellation error
  immediately" (`03:1142-1144`). The spawn is a checkpoint inside the
  cancelled request scope, so it starts nothing. Under `03:1082-1086` the
  unstarted child in `bg` then "counts as one whose first checkpoint
  returned the cancellation and which returned
  `E::from(TaskError::Cancelled)`". In `bg` that child has no echo bit:
  neither `bg`'s body nor a sibling caused the cancellation
  (`03:1129-1133`). It was not cancelled by `bg`'s own deadline either
  (TASK-4). So it is an independent failure: it cancels `bg` (TASK-2,
  "The first failure ... cancels the scope") and `bg` returns
  `Cancelled`. One request's deadline stops the server. Under an
  unlimited `bg`, the same child starts and runs normally, since `bg` is
  not cancelled. So "TASK-2 and TASK-4 decide the outcome unchanged"
  (`03:1083-1084`) gives a result that only the limit causes. The
  fiction is also false here: a child of `bg` that ran would not have
  seen the request scope's cancellation at its first checkpoint.
- **Target's cancellation only.** The spawn waits as long as the 64
  connections run, which is round 81's B1 again, for the outward case.
  It contradicts TASK-3 for this checkpoint and the reason `03:1088-1091`
  gives for the reversal. It also defeats TASK-6's recipe, "bounded
  cleanup is a shielded scope with its own deadline" (`03:1171-1173`),
  for cleanup that spawns into an outer limited scope.

The diff introduces this. Before it, a limited `spawn` was not a
checkpoint, and no rule turned a caller's cancellation into a failure of
another scope.

**Possible fix** (03 §6.1, one or two sentences; the core can stay as
it is because its wording fits the common case):

- Keep TASK-3's reading: the spawn observes its caller's cancellation.
  Add: "If the scope spawned into is not itself cancelled, the unstarted
  child counts as an echo: its `join` returns
  `Err(TaskError::Cancelled.into())`, and that scope is unaffected."
  The caller is cancelled and unwinds through its own checkpoints, so
  nothing is lost that a cancelled caller could still use. TASK-2's
  echo-bit sentence (`03:1129-1131`) would then name this case, or
  §6.1 can say it sets the bit.
- Alternatively, state that the spawn observes only the target's
  cancellation, exempt it in TASK-3, and accept the unbounded outward
  wait with a remedy. That reopens the case the reversal was made to
  close, so I would not choose it.

## Non-blocking notes

### N1. Whether the limited `spawn` sets the caller's echo bit is unstated

TASK-2 sets a child's bit "when a checkpoint in the child, or in a scope
nested in it, returns a cancellation error" caused by the body's `Err`
or a sibling's failure (`03:1129-1131`). A limited `spawn` is a
checkpoint, but it returns a `Task`, not an error. Take child `A`,
spawned first, that spawns into its own scope after sibling `B` has
failed. The unstarted grandchild is an echo. But `A` then returns its
`join`'s `Err(Cancelled)` through `?`, and `A`'s bit is not set unless
another checkpoint set it. So `A` is an independent failure and, as the
earliest-spawned one, rule 2 reports `Cancelled` in place of `B`'s
error. Unlimited scopes have the same path through `join` (Out of
scope), but here the caller did observe the cancellation. One clause
would close it: "it also sets the caller's bit, as a checkpoint that
returned the cancellation would."

### N2. TASK-3's rule for consumed values does not say what happens to `f`

TASK-3: "Checkpoints that consume a value return it inside the error"
(`03:1144-1145`). A limited `spawn` consumes `f` and returns neither `f`
nor an error. The new sentence (`03:1146-1147`) sits where an exception
belongs but does not say that `f` and its captures are dropped unrun,
in the spawning task. That matters for captured `Sender`s (dropping one
can disconnect a channel at once) and for `Drop` that logs a `warn!`
(TASK-6). Suggest "starting nothing; `f` is dropped unrun in the
spawning task".

### N3. Precision in the first §6.1 paragraph

- `03:1078`: "The wait is a checkpoint." But `03:1081` makes the whole
  limited `spawn` a checkpoint, "even if a slot is free", as the §1.2 row
  says (`03:210`). Suggest "A limited `spawn` is a checkpoint."
- `03:1085-1086`: "the scope reports `DeadlineExceeded` when its own
  deadline did" holds only when no child panicked. TASK-2 rule 1 comes
  first. "It counts as cancelled by that deadline (TASK-4)" is exact and
  no longer.

### N4. Small leftovers and wording

- The TASK-5 hunk (`03:1165-1167`) only rewraps "file system" and
  changes no word. It is left over from the shielded-wait version, where
  TASK-5 listed the wait. Revert it to keep the diff to the decision.
- `03:1276-1277`: "Programs can still deadlock through channels" is
  still true but now incomplete. "or a shared limit (§6.1)" would point
  to the new case.
- Companion 15.2's doc comment, "none runs past the deadline"
  (`reference-companion.md:195`): children queued past the deadline start
  after it and then return `Timeout`. "No request runs past the
  deadline" would be exact.
- Companion 14.3 maps `tokio::time::timeout` to "an operation deadline,
  or `s.set_deadline(..)`" and `Semaphore` to `tasks.limit(4)`
  (`reference-companion.md:94-95`). A port of `buffer_unordered` plus
  `timeout` that picks the scope deadline loses outcomes. 15.2's prose
  (`reference-companion.md:242-246`) explains why. A "(not under
  `limit`, 15.2)" after `s.set_deadline` would warn at the point of use.

## Checked and consistent

- **§0, §1.1, §1.2.** §0 makes every suspending capability operation a
  checkpoint and does not list the synchronization ones (`03:56-59`). The
  channel operations are already in that class, so a checkpointing
  `spawn` fits. §1.2's `Scope` row, "`spawn` under `Tasks::limit`, a
  checkpoint (§6.1)" (`03:210`), matches the channel row's "all
  checkpoints". §1.1's attenuation cell points to §6.1 (`03:85`).
  "Suspends" correctly leaves `spawn` to §1.2. The HEAD text of §0 and
  TASK-5 is restored, so no "non-checkpoint wait" claim is left over.
- **TASK-2, TASK-4, JOIN-2, when the target is cancelled.** An unstarted
  child counted as returning `E::from(TaskError::Cancelled)` after the
  cancellation is an echo when the body's `Err` or a sibling caused it,
  counts as `DeadlineExceeded` under the scope's own deadline, and under
  `cancel()` or an ancestor's cancellation is a failure with its own
  error, as TASK-4 says for `cancel()` (`03:1160-1161`). Its `join`
  returns at once with `Err(TaskError::Cancelled.into())`, as JOIN-2
  gives for a cancelled child. `E: From<TaskError>` makes the error
  constructible.
- **The reasons.** The shared-count argument (`03:1088-1091`) holds: a
  scope's cancellation reaches its body, children, and nested scopes
  (TASK-2), not children of sibling scopes from the same `Tasks`.
  Starting over the limit breaks the attenuation. The `Result`
  alternative's cost is stated.
- **The deadlock paragraph** (`03:1098-1102`). Both routes are real: a
  child may capture its `Scope` (`02:694`), and it may open a scope from
  the same `Tasks`. Cancelling the target of each waiting spawn makes it
  return at once (and its caller, being inside it, is cancelled too), so
  the holders run to their next checkpoint and free slots, as softly as
  TASK-5 allows.
- **TASK-6.** A limited `spawn` inside `tasks.shield` ignores the
  enclosing scopes' cancellation like any checkpoint, so it waits and
  starts the child, which then sees its scope's cancellation at its
  first checkpoint. Nothing in §6.1 contradicts that.
- **§9.3 and D8.** `spawn` is already a scheduling point (`03:1504-1506`;
  core `reference.md:538-539`). Production and the simulator use the
  same semaphore, and the level-triggered rule ("even if a slot is free")
  makes the outcome for a spawn made after cancellation independent of
  slot timing, so "production cannot deadlock where the simulator makes
  progress" (`01:547-549`, `03:255-256`) holds. D8's "fails the scope
  only if a child returns the cancellation as its error"
  (`01:518-521`) covers the counted child.
- **02 §8.2 and §8.3.** An unstarted child borrows nothing longer than
  the spawn call, and the wait happens before the scope's join, so the
  loans in `σ` and the failure edge are unchanged (`02:683-689`,
  `02:717-719`).
- **Core 7.1 and 7.2.** 7.1's bullet (`reference.md:634-638`) agrees with
  03. 7.2's list of non-checkpoints (`reference.md:665-667`) correctly
  omits the spawn. 7.2's partial-results sentence (`reference.md:675-677`)
  describes children that ran. 7.1 states the limited exception, so
  nothing contradicts it, and the budget leaves no room to repeat it.
- **Companion 15.2.** Without the scope deadline, `fetch_one` maps every
  error to an `Outcome`, so each child returns `Ok` and the scope returns
  one outcome per URL. Core 7.2 says `net.get` "returns
  `NetError::Timeout` when `deadline` passes" (`reference.md:670-671`),
  so a late-starting child gets `Timeout`. 03 does not say whether a
  past deadline outranks `Dns` or `Denied`, but any of them becomes an
  `Outcome`, so the argument holds. The test holds: `fast.test` replies
  in 20 ms plus 2–20 ms of packets (`03:1494-1497`), giving
  `Fetched(5)`; `slow.test`'s `get` returns `Timeout` at 2 s, giving
  `TimedOut`; and `sim.elapsed()` is exactly 2 s, which satisfies
  `<= 2 s`. With only two URLs no spawn waits. With an ancestor
  cancelled, unstarted children are failures, so `fetch_all` returns
  `Err(TaskError::Cancelled)`, which its signature allows. Dropping the
  `is_cancelled()` arm is correct: with no scope deadline, `Cancelled`
  comes only from an ancestor and becomes `Failed`.
- **03 §12.2 and §12.3.** Both use unlimited scopes (`03:1814`,
  `03:1849`, `03:1875`), so neither is affected, as the bead says.
- **Grep.** No other hit in `design/` for `limit`, `spawn`, `checkpoint`,
  `shielded`, `deadlock`, "never interrupted", or "cannot be cancelled"
  contradicts the decision.
- **Prose.** Plain, matching the chapter. No new prose line exceeds 80
  columns.

## Out of scope

- **A child that fails through a grandchild's `join` after an echo.** In
  an unlimited scope, a child that spawns into its own scope after a
  sibling failed and returns the grandchild's `join` error through `?`
  has no echo bit if it hit no checkpoint itself, so it can outrank the
  root cause (TASK-2, JOIN-2). This predates the diff; N1 covers only
  the limited case.
- **`limit(0)`.** Every limited `spawn` from it waits until cancelled.
  Neither 03 nor the core says whether `limit(0)` is rejected. The
  reversal makes the wait cancellable, but the question remains.
- **`spawn_isolated` under `limit`** and **`limit` on a limited
  `Tasks`.** Both are still unstated (rounds 80 and 81).
- **D8's "suspend only by using a capability"** (`01:511-513`) is looser
  than 03 §0's "capability and synchronization operations"
  (`03:54-55`), as rounds 80 and 81 noted.
