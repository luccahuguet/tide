# Tide adversarial/consistency review, round 83: limited spawn and cancellation (fx-sz7)

## Verdict

**approve**: no blocking findings and four non-blocking notes.

The refinement closes round 82's B1: a limited `spawn` observes the
calling task's cancellation, as TASK-3 requires, and when the scope
spawned into is not cancelled the unstarted child is an echo that leaves
that scope alone. One request's deadline can no longer fail a server
scope, and no wait outlasts the caller's cancellation. Round 82's N1 to
N3 and the applied parts of N4 are closed. The two N4 nits left out on
purpose (the companion 14.3 `timeout` row and a §6.5 pointer to §6.1)
are still non-blocking. The decision agrees with §0, §1.1, §1.2, TASK-1
to TASK-6, JOIN-1 and JOIN-2, §9.3, D5, D7, D8, 02 §8.2 and §8.3, core
7.1 and 7.2, and companion 14.3 and 15.2. The notes below are about
wording only.

## What was reviewed

The uncommitted working tree against HEAD `db1bebc`: `git diff --
design/`, SHA-256
`c36c6c56fbec404f342f8e693679231b2a7f5dee04ffdabf948882264903023d`
(03, `reference.md`, `reference-companion.md`). I recomputed the hash.
`git diff --check` passes. The only added lines past 80 columns are the
two table rows (`03:85`, `03:210`), like every row in those tables.
`scripts/count-tokens.sh design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-sz7 with all four comments ("Reversal after round 81"
  and "Refinement after round 82" are the decision under review), and
  `reviews/round82-consistency.md`;
- the whole diff;
- in 03: §0 Terms (`03:31-63`), §1.1's receivers note (`03:179-192`),
  §1.2 and §1.3 (`03:205-263`), §6.1 to §6.6 (`03:1042-1297`), and §9.3
  (`03:1497-1515`);
- `01-design.md` D8 (`01:509-564`) and D7's references;
- `02-memory.md` §8.2 and §8.3 (`02:657-724`);
- core 6.8's time and order bullets (`reference.md:530-541`), 7.1, 7.2,
  and 7.3 (`reference.md:600-699`), and the parallelism bullet at
  `reference.md:792-794`;
- companion 14.3 (`reference-companion.md:84-99`) and 15.2
  (`reference-companion.md:184-246`);
- a grep of `design/` for `limit`, `spawn`, `checkpoint`, `echo`, and
  `deadlock`.

## Earlier findings

- **Round 82 B1, closed.** `03:1080-1082`: "when the calling task is in a
  cancelled scope, a limited `spawn` returns at once and starts nothing".
  `03:1089-1092`: if the scope spawned into is not cancelled, "as when a
  task in a cancelled nested scope spawns into an enclosing scope passed
  down to it, the child counts as an echo and that scope is unaffected."
  So round 82's server example now behaves as wanted: the request scope's
  deadline makes `bg.spawn` start nothing, `bg` does not fail, and no
  wait on `bg`'s 64 slots outlasts the request. The case split is
  complete. The caller always runs inside the scope spawned into: a
  `Scope` cannot leave its origin (`02:690-692`), so a cancelled target
  means a cancelled caller, and only "caller cancelled, target not" was
  left open.
- **Round 82 N1, closed.** `03:1082-1083`: "Like any checkpoint that
  returns a cancellation, it sets the caller's echo bit (TASK-2)." Child
  `A` that spawns into its own scope after sibling `B` failed now has its
  bit set, so `A`'s `?` of the `join` error is an echo and rule 2 reports
  `B`'s error. See N1 below on wording.
- **Round 82 N2, closed.** `03:1082`: "and drops `f` unrun". Which task
  drops it is not said, but only the caller holds `f`. See N3 below.
- **Round 82 N3, closed.** `03:1078`: "A limited `spawn` is a
  checkpoint." `03:1088-1089`: "counts as cancelled by the deadline when
  the scope's own deadline did", which leaves TASK-2 rule 1 in charge of
  panics.
- **Round 82 N4, closed in part.** The TASK-5 rewrap is reverted (the
  diff no longer touches `03:1170-1176`). The companion doc comment reads
  "no request runs past the deadline" (`reference-companion.md:195`).
  The 14.3 `timeout` row (`reference-companion.md:94`) and a §6.5
  pointer from "Programs can still deadlock through channels"
  (`03:1282`) were left out on purpose, as the bead records. Both stay
  non-blocking: 15.2's prose (`reference-companion.md:245-246`) and
  §6.1's deadlock paragraph (`03:1104-1108`) carry the facts.

## Blocking findings

None.

## Non-blocking notes

### N1. "Sets the caller's echo bit" reads as unconditional

`03:1082-1083` says the spawn "sets the caller's echo bit". TASK-2 sets
the bit only for a cancellation "that the body's `Err` or a sibling's
failure caused" (`03:1135-1137`), and "Like any checkpoint" carries that
condition. Read alone, though, "sets" drops it. Take round 82's server:
a handler in `bg` opens a request scope whose own deadline passes, so
its `bg.spawn` starts nothing. Under the literal reading the handler's
bit in `bg` is set, and if the handler then returns the `join` error
through `?`, its `Err` is an "echo" in a `bg` that nobody cancelled, a
state TASK-2 does not define. Under the intended reading the bit stays
clear and the `Err` is an ordinary failure of `bg`, the same as for any
other checkpoint in the request scope that the handler propagates. The
caller also may have no bit at all (the main task's body). Suggest "it
sets the caller's echo bit when any checkpoint would (TASK-2)".

### N2. The reason and the deadlock paragraphs still speak of "this scope"

`03:1094-1097`: "the slots may be held by children of other scopes,
which this scope's cancellation does not reach, so a wait that ignored
it could outlast the scope's deadline". After the refinement the
cancellation observed is the caller's, and the case that motivated the
refinement is the one where the slots are held by the target's own
children. "Which the caller's cancellation does not reach" and "outlast
the caller's deadline" would match `03:1080`. The same goes for
`03:1106-1107`, "Cancelling the scopes of the waiting spawns": both
readings work (cancelling the target cancels the caller inside it,
unless it is shielded), but "the scope of each waiting caller" is the
one that always holds.

### N3. TASK-3's consumption rule and §1.1's receivers note name no exception for `f`

TASK-3 says "Checkpoints that consume a value return it inside the
error" (`03:1150-1151`), and §1.1 relies on it: `wait_with_output` takes
`&mut self` because "TASK-3 requires a consuming operation to return
what it consumed inside its cancellation error" (`03:184-186`). A
limited `spawn` consumes `f` and drops it. `03:1152-1153` says only that
it "reports cancellation by starting nothing", and the drop is stated in
§6.1. The trade is reasonable and already argued (`03:1098-1102`), but
two consequences are unstated: cleanup written inside `f` never runs,
only the `Drop` of its captures (a captured `Child` is dropped unwaited,
the case §1.1 guards against), and code that must run its cleanup
should spawn under `shield` (TASK-6), where the spawn waits and starts
the child. Suggest TASK-3: "A limited `spawn` reports cancellation by
starting nothing and dropping `f` (§6.1)", and §1.1: "a consuming
capability operation".

### N4. Core 7.1's "counts as cancelled" fits the common case only

`reference.md:636-637`: "in a cancelled scope it starts nothing and the
child counts as cancelled". Read with 7.1's failure rule
(`reference.md:608-617`), a child that "counts as cancelled" by neither
the body, a sibling, nor the scope's deadline is a failure. For the rare
nested case 03 makes it an echo instead (`03:1089-1092`). The core is at
24,995 of 25,000 tokens and the common case (the caller is in the
target, or is its body) reads correctly, so no change is needed; a
future trim could say "the child's `join` fails" without new tokens.

## Checked and consistent

- **§0, §1.1, §1.2.** §1.2's `Scope` row, "`spawn` under
  `Tasks::limit`, a checkpoint (§6.1)" (`03:210`), matches §6.1 and the
  channel row's "all checkpoints". §1.1's attenuation cell points to
  §6.1 (`03:85`). §0's checkpoint definition (`03:56-59`) lists only
  capability operations and the exceptions, so it does not contradict a
  checkpointing synchronization operation.
- **TASK-2 and TASK-4, target cancelled.** The unstarted child is an echo
  when the body's `Err` or a sibling's failure cancelled the target,
  counts as cancelled by the target's own deadline under TASK-4, and
  under `cancel()` or an ancestor's cancellation is a failure with
  `E::from(TaskError::Cancelled)`, as an unlimited child that returned
  its first cancellation would be. A sibling's panic falls to rule 1.
  Echo bits after the target is cancelled by an ancestor follow TASK-2
  unchanged (the caller's bit in the ancestor's scope is set by that
  ancestor's cause).
- **Target not cancelled.** The echo is fixed at spawn time; a later
  cancellation of the target, by a sibling or its own deadline, does not
  turn it into a failure, since the child returned before it. It goes
  to TASK-2's `debug!` record with the other echoes. D8's "fails the
  scope only if a child returns the cancellation as its error"
  (`01:519-521`) is not contradicted: the child is defined not to.
- **Caller cases.** The body of the target, a child of it, a grandchild
  holding a captured `Scope` (`02:694`), and a task in a nested scope
  opened by any of these all fit "the calling task is in a cancelled
  scope". After the body has returned, children that still spawn into
  the joining scope behave the same way. Inside `tasks.shield`, TASK-6
  makes the spawn ignore enclosing cancellation, so it waits and starts
  the child, which then sees a cancelled target at its first checkpoint;
  a cancelled scope opened inside the shield is observed as usual.
- **JOIN-1 and JOIN-2.** The unstarted child has finished, so scope exit
  does not wait for it, and its `join` returns
  `Err(TaskError::Cancelled.into())` at once (`03:1189-1194`).
- **Panics.** A panic while the spawn waits cancels the target, so the
  spawn returns and rule 1 re-raises the panic. A panic in a capture's
  `Drop` unwinds in the caller, as in Rust.
- **The deadlock paragraph** (`03:1104-1108`). Cancelling the scope of a
  waiting caller makes its spawn return; the caller, being cancelled,
  runs to completion through failing checkpoints and frees its slot.
  §9.3 advances virtual time to a pending deadline (`03:1506-1509`), so
  the simulator reports a deadlock only when nothing can break it.
- **§9.3 and D8.** `spawn` is already a scheduling point (`03:1510-1512`,
  `reference.md:538-539`). The level-triggered rule keeps the outcome
  of a spawn made after cancellation independent of slot timing, so
  "production cannot deadlock where the simulator makes progress"
  (`01:547-549`, `03:255-256`) holds.
- **02 §8.2 and §8.3.** `f: 's` is dropped in the caller before the
  scope's join, so the loans in `σ` and the failure edge are unchanged.
  The Rust mapping over `std::thread::scope` (`02:704-705`) admits a
  `Task` that wraps no thread.
- **Core 7.1 and 7.2.** 7.1's bullet agrees with 03 apart from N4. 7.2's
  non-checkpoint list (`reference.md:665-667`) correctly omits the
  spawn. 7.2's partial-results sentence (`reference.md:675-677`) speaks
  of children that ran, and 7.1 already states the limited exception.
- **Companion 15.2.** With no scope deadline, every child that runs
  returns `Ok(Outcome)`: `net.get` returns `Timeout` once the deadline
  passes (`reference.md:670-671`), including for a child that starts
  late, and `fetch_one` maps every other error to `Failed`. The test
  still gives `[Fetched(5), TimedOut]`, and `sim.elapsed()` is 2 s. If
  an ancestor is cancelled, unstarted children are failures and
  `fetch_all` returns `Err(TaskError::Cancelled)`, which its signature
  allows. The prose (`reference-companion.md:242-246`) states the reason
  for dropping the scope deadline.
- **Grep.** No other hit in `design/` for `limit`, `spawn`,
  `checkpoint`, `echo`, or `deadlock` contradicts the decision. Core
  `reference.md:792-794` and companion `reference-companion.md:57`
  recommend a `tasks.limit(n)` scope for concurrent I/O, which the
  decision leaves valid. 03 §12.2 and §12.3 use unlimited scopes.
- **Prose.** Plain, matching the chapter. No new prose line exceeds 80
  columns.

## Out of scope

- **TASK-2's echo cause in nested scopes.** When a checkpoint in a scope
  nested in a child returns a cancellation, TASK-2 does not say how a
  cause in the nested scope (its own sibling's failure) relates to the
  bit the child holds in the outer scope, nor which cause wins when
  several cancel one scope. This predates the diff; N1 only touches it.
- **A child that fails through a grandchild's `join` after an echo**
  (round 82), **`limit(0)`**, **`spawn_isolated` under `limit`**,
  **`limit` on a limited `Tasks`**, and **D8's "suspend only by using a
  capability"** (`01:511-513`) against 03 §0's "capability and
  synchronization operations" (`03:54-55`) are still open from rounds 80
  to 82.
