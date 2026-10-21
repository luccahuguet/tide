# Tide adversarial/consistency review, round 88: limit edge cases and echo attribution (fx-0ai)

## Verdict

**request changes**: one blocking finding and three non-blocking notes.

The revision closes round 87's B1 for the case it named and closes N1 to
N5. The `limit` rules now hold under every case I tried: zero counts in
chains, clones, cancellation during a chained wait, `spawn_isolated` in a
limited child, and every escape route I could find. The cause list and the
nested-scope rule for checkpoints also hold. The join mark does not follow
them, though. It sets the bit under any cause and in any nested scope,
where a checkpoint sets it only for the body's `Err` or a sibling's
failure that reached the scope it runs in. Two cases lose or misreport an
error. One of them is round 87's B1 again, by a later cancellation (B1).
Tying the mark to the checkpoint rule fixes both. The core is unchanged
and contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `fceb12e`: `git diff --
design/`, SHA-256
`022f9d44edc5b9c02f0e6acc1a30c5f3cbb178684d4e6cf36a6ed4befb83903a`
(03 only). I recomputed the hash. `git diff --check` passes, and no
added line is over 80 columns. `scripts/count-tokens.sh
design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-0ai with both decision comments, and
  `reviews/round87-consistency.md`;
- the whole diff (`03:1105-1121`, `03:1151-1162`);
- in 03: §0 (`03:31-63`), §1.1's `Tasks` row and receivers note
  (`03:85`, `03:179-199`), §1.2 and §1.3 (`03:203-257`), §6.1 to §6.6
  (`03:1042-1321`), §9.3 (`03:1520-1538`), §12.2's `fetch_all`
  (`03:1830-1872`), and §13 (`03:2007-`);
- `01-design.md` D5's recovery sentence (`01:365-375`), D7's tail and D8
  (`01:505-565`);
- `02-memory.md` §8.2 and §8.3 (`02:657-730`);
- core 6.6's capability notes (`reference.md:335-350`) and 7.1
  (`reference.md:581-645`);
- companion 14.3 (`reference-companion.md:84-99`) and 15.2
  (`reference-companion.md:184-246`);
- a grep of `design/` for `limit`, `echo`, `spawn_isolated`, `clone`,
  `chunks(0`, and `same authority`.

## Earlier findings

- **Round 87 B1, closed for the case it named.** "while this scope is
  cancelled" (`03:1159-1160`) leaves an unstarted echo in a `bg` nobody
  cancelled unmarked, so `H` fails `bg` normally, as fx-sz7 settled. The
  same mechanism returns when `bg` is cancelled between the spawn and the
  join (B1 below).
- **Round 87 N1, closed.** "directly or in a nested scope"
  (`03:1160-1161`) marks `H` when `C`, a child of a scope nested in `H`,
  joins a failed child of `bg`.
- **Round 87 N2, closed.** `limit(0)` returns a usable `Tasks`, and the
  panic moves to the `spawn` (`03:1116-1119`), so
  `limit(n.min(len))` on empty input is harmless.
- **Round 87 N3, closed** by "from a `Tasks` that shares its count"
  (`03:1106-1107`), which covers chains and clones.
- **Round 87 N4, closed.** The LOCK-3 analogy is gone, "at once"
  (`03:1115`) claims only what it needs, and fairness between chained
  waiters is left unspecified on purpose.
- **Round 87 N5, closed.** The cause list has the body's panic
  (`03:1152`), the same-instant deadline tie is settled (`03:1153-1155`),
  "sets the bit" replaces "is marked", and the `spawn_isolated` reason
  speaks of children (`03:1120-1121`).
- **Round 87 Out of scope, clones of a limited `Tasks`, closed** by
  `03:1112-1113`, which agrees with core's "same authority"
  (`reference.md:343-344`).

## Blocking findings

### B1. The join mark ignores the cause that a checkpoint checks

A checkpoint sets the bit only for a cancellation "that the body's `Err`
or a sibling's failure caused" (`03:1149-1150`), and in a nested scope
only for one "that reached the nested scope from this one"
(`03:1155-1156`). The join mark (`03:1158-1162`) asks only that this scope
be cancelled. Its reason, "this scope reports the joined child's outcome
itself", fails in two cases.

**(a) Round 87's B1 by a later cancellation.** Take round 87's server:
`bg` comes from a limited `Tasks` and has its own shutdown deadline.
Handler `H` opens request scope `r`. After `r`'s deadline, `H`, as `r`'s
body, calls `bg.spawn(f)`. The spawn starts nothing, and the unstarted
`G` is an echo that leaves `bg` unaffected (`03:1090-1093`), so `bg`
never reports it. Then `bg`'s deadline passes, the other handlers turn
the cancellation into values, and `H` calls `G.join()?`. The join returns
`Err` while `bg` is cancelled, so `H`'s bit is set, `H`'s `Err` is an
echo, and rule 2 skips it. `bg` returns its body's `Ok`, and the
request's error goes only to the `debug!` record. Without the mark, `H`'s
`Err` after `bg`'s own deadline counts as `DeadlineExceeded`
(`03:1184-1187`) and `bg` fails, as D8 requires: a deadline "fails the
scope only if a child returns the cancellation as its error"
(`01:519-520`). With `cancel()` or an ancestor's cancellation in place of
the deadline, the outcome is the same, against TASK-4's "a failing
child's own error is reported" (`03:1190-1191`). A checkpoint in `r` at
the same point would set nothing: `r`'s own deadline came first.

**(b) A nested scope's own cause.** In `bg`'s child `H`, scope `inner`
is cancelled first by its child `K`'s independent failure. Later `bg` is
cancelled by `cancel()`, and `bg`'s child `G` returns its own `Err`.
Then `C`, another child of `inner`, joins `G` through a `Task` passed
down. The join returns `Err` while `bg` is cancelled, so `H`'s bit is
set. `inner` returns `K`'s error, `H` propagates it, and rule 2 skips it
as an echo. `bg` reports `G`'s cancellation error, and `K`'s real failure
is lost. This contradicts the diff's own sentence that a cause inside the
nested scope "is that scope's to report, and it reaches this scope as
the child's own result" (`03:1156-1158`), and D8's "an independent child
failure is never hidden" (`01:545`). A checkpoint in `C` would return
`inner`'s cancellation, whose cause is `K`, and set nothing. When `bg`'s
cause is a sibling's failure instead, that failure is reported, and the
mark only reorders two real failures against spawn order.

Under `cancel()`, an ancestor's cancellation, or a deadline, the mark
also makes a joiner an echo that D8's definition (`01:537-539`) does not
cover.

**Fix.** Make the join follow the checkpoint rule, for example: "A
`join` of one of this scope's children that returns `Err` sets the bit
where a checkpoint at the same point, outside `shield`, would: in the
child it runs in, directly or in a nested scope, since this scope reports
the joined child's outcome itself (JOIN-2)." "Outside `shield`" keeps a
shielded join marked, since a join is shielded anyway. Round 82's case
and round 87's N1 case are both still marked, since their scopes were
cancelled by a sibling's failure. The cost: under `cancel()` or an
ancestor's cancellation, a joiner's `Cancelled` again counts as its own
error and may outrank the joined child's error by spawn order. TASK-4
already accepts that for checkpoints, so the two agree. D8 then needs no
change.

## Non-blocking notes

### N1. Checkpoints in a `spawn_isolated` task are not attributed

The bit is set by "a checkpoint in the child, or in a scope nested in
it" (`03:1148-1149`). A task that `H` runs with `spawn_isolated` is
neither: it is a new task (`03:1131`) and opens no scope of `H`'s. It
inherits `H`'s cancellation, though, and `H` waits for it. In core's
server pattern, "servers: one per request" (`reference.md:632`), when a
sibling of `H` fails, the isolated request work sees `Cancelled` and
returns it in its `T`. `H` propagates it with no bit set, so `H` counts as
an independent failure and, if spawned first, outranks the real one:
round 82's class through isolation. The wording predates this diff, but
item 4 asks how a cause relates to an outer child's bit across nesting.
"A checkpoint in the child, in a task it runs with `spawn_isolated`, or
in a scope nested in either" would settle it. If it stays out, it belongs
on a bead.

### N2. A zero count in a cancelled scope has two rules

"a `spawn` under a zero count panics" (`03:1118-1119`) and "when the
calling task is in a cancelled scope, a limited `spawn` returns at once
and starts nothing" (`03:1080-1082`) both apply to a zero-count spawn in
a cancelled scope. If the cancellation rule wins, the misuse shows only
on paths where no deadline or failure raced it, which the simulator
finds only on some seeds. "panics, even in a cancelled scope or under
`shield`" makes it deterministic.

### N3. Wording

- "whose `checkpoint`, `shield`, and handle constructors work as usual"
  (`03:1116-1117`) reads as a closed list, but `scope`, `spawn_isolated`,
  `yield_now`, `fault_point`, and `limit` work too, and the next sentence
  relies on `spawn_isolated` working. "on which everything but `spawn`
  works as usual" is shorter and complete.
- "an enclosing scope's cancellation, which comes first when ..."
  (`03:1153-1155`): the relative clause reads at first as part of the
  list. A separate sentence would read better: "When an enclosing
  deadline falls at the same instant as its own, the enclosing
  cancellation comes first."

## Checked and consistent

- **Cause list and nested checkpoints.** One cause per scope, first event
  wins. A nested scope cancelled first by its own child keeps `H`'s bit
  clear, and `H`'s error counts by spawn order; in the reverse order the
  cause is the enclosing cancellation and the bit is set. This works
  transitively through several levels: each intermediate scope's cause is
  "an enclosing scope's cancellation", which sets no bit there, so its
  child's `Err` is an ordinary failure there and an echo only at the level
  whose own cause was a failure. A body panic in the list changes no
  outcome, since the panic keeps unwinding.
- **Deadline ties.** An enclosing deadline at the same instant as a
  scope's own is the enclosing cause, so TASK-4's "set on this scope, not
  an ancestor" (`03:1184-1185`) does not apply, and the nested scope
  reports its children's own errors. That matches "a nested scope's
  effective deadline is the minimum over its ancestors" (`03:1181-1182`).
  Other same-instant events fall under TASK-2's real-time or scheduler
  order (`03:1133-1134`).
- **Joins, apart from B1.** The body's joins are unchanged (rule 2 over
  rule 3, `03:1167-1169`). A join by a child of a scope nested in the
  body marks no child of this scope, and the body's `Err` loses to the
  joined child's error under rule 2. A join of a child that failed
  independently, echoed, panicked, or never started in a
  cancelled target (`03:1085-1090`) is reported by this scope, so the
  joiner's mark is correct when the cause is a failure. A child that
  catches the join error and fails later stays an echo, like one that
  catches a checkpoint's cancellation.
- **Chained limits and zero counts.** A spawn takes a slot in each count
  at once (`03:1114-1115`), so the inner holders are a subset of the
  outer's and a larger inner `n` cannot widen the bound. A zero count
  anywhere in a chain, or in a clone of one, makes every `spawn` through
  it panic. A spawn cancelled during a chained wait holds nothing and
  returns at once; under `shield` it waits for every count.
- **Clones.** Shared counts (`03:1112-1113`) agree with core's `Clone`
  note (`reference.md:343-344`) and with "all scopes from that `Tasks`
  share one count" (`03:1076-1077`).
- **`spawn_isolated` under `limit`.** It takes no slot, and the caller
  keeps its own slot while it waits, so live children do not grow. A
  limited child whose isolated task spawns through a clone sharing its
  count falls under the deadlock paragraph (`03:1105-1110`), and the
  inherited cancellation (`03:1131`) still breaks it.
- **Escape routes.** The isolated closure is `'static` (`03:1052`,
  `02:692-693`), so it can spawn only through an owned clone, which shares
  the counts. `limit` never drops a count, `shield` changes no count, and a
  spawn into a passed-down `Scope` counts against the `Tasks` that made
  that scope. I found no way to get a looser `Tasks` from a limited one.
- **§0, §1.1, §1.2, §1.3.** The `Tasks` row's "live children"
  (`03:85`), §1.2's limited `spawn` checkpoint (`03:210`), and §1.3's
  bound (`03:257`) agree with the diff.
- **§6.3, §6.4, §9.3.** JOIN-2's "the scope reports the actual failure"
  (`03:1216-1217`) is the mark's reason. The disconnect rule
  (`03:1255-1259`) depends only on the bit. A zero count no longer
  produces a waiting `spawn`, so §9.3's deadlock report covers only the
  deadlock paragraph's cases.
- **D5, D7, D8, 02 §8.2 and §8.3.** Nothing changes origins, loans, the
  failure edge, or `spawn_isolated`'s `Owned` closure. D8's echo
  definition holds once B1 is fixed.
- **Core and companion.** Core 7.1's "at most n live children"
  (`reference.md:588`) and bounded-concurrency bullet
  (`reference.md:634-638`) are silent on zero counts, chains, and clones
  but not contradicted. Companion 15.2 uses one unchained `limit(4)`;
  §12.2's note (`03:1838-1840`) is unaffected.
- **Grep.** No other hit in `design/` contradicts the diff, and §13 lists
  no open item it settles.
- **Prose.** Plain and matching the chapter; no added line is over 80
  columns.

## Out of scope

- **D8's "suspend only by using a capability"** (`01:510-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`), still
  open from rounds 80 to 87.
