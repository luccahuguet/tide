# Tide adversarial/consistency review, round 89: limit edge cases and echo attribution (fx-0ai)

## Verdict

**request changes**: one blocking finding and three non-blocking notes.

The revision closes round 88's B1 and N2 to N3 and most of N1. The join
rule is now the checkpoint rule ("exactly where a checkpoint at the same
point, outside `shield`, would", `03:1160-1162`), and under that rule
every case I tried comes out right: round 87's server case, round 88's
nested case, round 82's case, joins of failed, echoed, panicked, and
unstarted children, joins from nested scopes and from the body, and
children that catch a join error. The `limit` rules hold as in round 88,
and the zero-count panic is now deterministic. The blocker is in the two
examples that follow the colon (`03:1162-1164`). Read as written, each
one contradicts the rule it illustrates in a case an earlier round
settled. The first brings back round 87's B1. The fix is text only. The
core is unchanged and contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `fceb12e`: `git diff --
design/`, SHA-256
`7aa2a226869557d9a8f6360712bd9d1aa96e4161f5646222fbad671ae9e6683d`
(03 only). I recomputed the hash. `git diff --check` passes, and no
added line is over 80 columns. `scripts/count-tokens.sh
design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-0ai with all three comments (the decision and the
  revisions after rounds 87 and 88), and `reviews/round88-consistency.md`;
- the whole diff (`03:1105-1122`, `03:1147-1168`);
- in 03: §0 (`03:31-63`), §1.1's `Tasks` row (`03:85`), §1.2 and §1.3
  (`03:203-257`), §6.1 to §6.6 (`03:1042-1323`), including fx-sz7's
  limited-spawn text (`03:1076-1103`), and §9.3 (`03:1522-1543`);
- `01-design.md` D8 (`01:509-555`);
- `02-memory.md` §8.2 and §8.3 (`02:657-735`);
- core 6.6's capability notes (`reference.md:335-350`) and 7.1
  (`reference.md:578-656`);
- companion 15.2 (`reference-companion.md:184-246`) and the 14.3 rows
  that name `limit` and `spawn_isolated` (`reference-companion.md:53-98`);
- a grep of `design/` for `limit(`, `echo`, `spawn_isolated`,
  `same authority`, `chunks(0`, and `first event`.

## Earlier findings

- **Round 88 B1, closed in the rule.** The join now marks exactly where a
  checkpoint would (`03:1160-1162`). In round 88's case (a), a
  checkpoint in `r`'s body returns `r`'s cancellation, whose cause is
  `r`'s own deadline, so `H`'s bit stays clear. `H`'s `Err` after `bg`'s
  deadline then counts as `DeadlineExceeded` (`03:1186-1189`), and `bg`
  fails as D8 requires. In case (b), a checkpoint in `C` returns
  `inner`'s cancellation, caused by `K`, so nothing is marked and `K`'s
  error reaches `bg` as `H`'s result. The examples after the colon still
  state the old outcome for both cases (B1 below).
- **Round 88 N1, mostly closed.** "or in a task it waits for with
  `spawn_isolated`" (`03:1150-1151`) covers checkpoints the isolated task
  runs itself. Checkpoints in scopes that the isolated task opens are
  still not named (N1 below).
- **Round 88 N2, closed.** "It panics in a cancelled scope and under
  `shield` too" (`03:1119-1120`) makes the zero-count panic win over the
  cancelled-scope rule (`03:1080-1082`) on every run.
- **Round 88 N3, closed.** "Everything but `spawn` works on `limit(0)`"
  (`03:1116`) replaces the closed list, and the deadline tie is its own
  sentence (`03:1155-1156`).

## Blocking findings

### B1. The join examples contradict the join rule

The rule: a `join` that returns `Err` "sets the bit exactly where a
checkpoint at the same point, outside `shield`, would" (`03:1160-1162`).
The examples after the colon: "a child that fails through a join of a
sibling that echoed is an echo too, while a join in a scope cancelled by
another cause marks nothing" (`03:1162-1164`). Each example misstates
the rule in one case.

**(a) The first example marks round 87's case again.** fx-sz7 says that
a child spawned into an uncancelled scope from a cancelled nested scope
"counts as an echo and that scope is unaffected" (`03:1090-1093`). Take
round 87's server: handler `H`, a child of `bg`, opens request scope
`r`. After `r`'s deadline, `H`, as `r`'s body, calls `bg.spawn(f)`. `G`
starts nothing and counts as an echo in `bg`. `H` then calls
`G.join()?`. `G` is "a sibling that echoed", so by the first example
`H` is an echo, and rule 2 skips it. By the rule, a checkpoint in `r`'s
body returns `r`'s cancellation, whose cause is `r`'s own deadline, so
the bit stays clear. If `bg` is not cancelled, `H`'s error is an
ordinary failure. If `bg` is later cancelled by its deadline,
`cancel()`, or an ancestor, `H`'s error counts under TASK-4. The round
87 revision on fx-sz7 chose the rule's outcome ("a joiner that
propagates it fails normally"). An implementer who follows the example
loses the request's error, which is round 87's B1.

The first example is also wrong when the join runs in a nested scope
that its own cause cancelled first, as in round 88's case (b): `G`
echoed `bg`'s failure-caused cancellation, yet the rule marks nothing.
The second example happens to cover that case, so in case (b) the two
examples contradict each other.

**(b) The second example unmarks round 87's N1 case.** In `H`, nested
scope `inner` is cancelled by `bg`'s cancellation, which a sibling's
failure caused. `C`, a child of `inner`, joins an echoed child of `bg`
and fails, and `H` propagates it. Under the cause list
(`03:1152-1155`), `inner`'s cause is "an enclosing scope's
cancellation", which is literally "another cause", so the second
example marks nothing. The rule marks `H`: a checkpoint in `C` returns a
cancellation "that reached the nested scope from this one"
(`03:1157-1158`) with a sibling's failure as its cause. If nothing marks
`H` and it was spawned before the real failure, its `Cancelled`
outranks that failure, which is round 82's class of bug.

**Fix.** Drop the examples. The rule, together with the checkpoint
sentences before it, already settles every case. If one example stays,
make it one the rule decides without nesting, for example: "A `join`
that returns `Err` sets the bit exactly where a checkpoint at the same
point, outside `shield`, would, so a child that fails through
`x.join()?` after a sibling's failure cancelled this scope is an echo
too." The joined child may be the failing sibling itself, which is the
common fan-out case. Do not state a rule for joins of echoed siblings in
general.

## Non-blocking notes

### N1. Scopes opened by an isolated task are still not named

The bit is set by a checkpoint "in the child, in a scope nested in it,
or in a task it waits for with `spawn_isolated`" (`03:1149-1151`). In
core's server pattern (`reference.md:630-633`), the isolated request
work often opens its own fan-out scope `f`. When a sibling of `H`
fails, `f` is cancelled through the inherited cancellation
(`03:1132`), `f`'s children see it at their checkpoints, and `f`
returns `Cancelled` to the isolated task. The isolated task itself may
reach no checkpoint: an unlimited `spawn` and `join` are not
checkpoints. `H` then propagates the error with no bit set, which is
round 82's class again. Whether a scope that the isolated task opens is
"nested in" `H` is open to argument. Round 88's wording settles it:
"in the child, in a task it waits for with `spawn_isolated`, or in a
scope nested in either". The reach sentence (`03:1156-1160`) then
applies unchanged, so a cause inside `f` stays `f`'s to report.

### N2. "outside `shield`" has two readings for a shielded nested scope

"outside `shield`" (`03:1161-1162`) is there because a join is shielded
(JOIN-2), so a checkpoint inside the join would never fire. It can also
be read as "as if every enclosing `tasks.shield` were absent". The two
readings differ in TASK-6's bounded-cleanup shape (`03:1203-1205`):
`H`, in `bg`, runs `tasks.shield(|| tasks.scope(|s| { s.set_deadline(t);
.. c.join()? .. }))`. A sibling's failure cancels `bg` during the
cleanup, and later `s`'s own deadline cancels `s`, so `c.join()`
returns `Err`. Under the first reading, a checkpoint there returns
`s`'s cancellation, whose cause is `s`'s own deadline, so the bit stays
clear. Under the second, `s` would have been cancelled earlier through
`bg`, the cause would be the sibling's failure, and the bit is set. The
second reading is the one that keeps round 82's case marked when the
join sits under a user's `shield`. "as if no `shield` were in force"
would say so. Usually the bit is already set by the checkpoint that led
`H` into the cleanup, so few programs differ.

### N3. Wording

- Two older sentences say "observed" where the bit decides: "a child
  that has observed no such cancellation" (`03:1166-1168`) and "has not
  already observed such a cancellation" (`03:1258-1259`). A child marked
  by a join has observed a join error, not a cancellation. "whose bit is
  clear" is exact in both places.
- The zero-count panic overrides "returns at once and starts nothing"
  in a cancelled scope (`03:1080-1082`), TASK-3's "reports cancellation
  by starting nothing" (`03:1178-1179`), and core's "in a cancelled scope
  it starts nothing" (`reference.md:635-637`). The override at
  `03:1119-1120` is explicit, so nothing is ambiguous, and core has no
  room for the edge case.
- `03:1121` begins the `spawn_isolated` sentence on a new line of the
  same paragraph after a short line. It reads as a new topic; a blank
  line before it, or "And `spawn_isolated` takes no slot", would show
  the change of subject.

## Checked and consistent

- **The rule itself.** With the examples removed, "exactly where a
  checkpoint at the same point would" gives one answer for every join:
  - The child joins the sibling whose failure first cancelled the scope:
    the bit is set and the sibling is reported. This depends on the
    failure cancelling the scope before its join returns, which TASK-2's
    "the first failure ... cancels the scope" (`03:1134-1136`) implies.
  - It joins an echoed sibling in a scope cancelled by a sibling's
    failure: the bit is set (round 82's case).
  - It joins an unstarted echo in an uncancelled target: no bit (fx-sz7).
  - It joins any child in a scope cancelled by its deadline, `cancel()`,
    or an ancestor: no bit, and the joiner's error counts under TASK-4,
    as a checkpoint's would.
  - It joins a panicked sibling: no bit, but rule 1 re-raises the panic
    anyway.
  - It joins from a nested scope: the reach condition decides, as for
    checkpoints.
  - The body joins: there is no bit, and rule 2 still outranks rule 3.
  - A child catches a join error and fails later: the bit stays set, as
    after a caught checkpoint error.
- **Cause list and ties.** One cause per scope, first event wins
  (`03:1152-1155`). The list covers every event that can cancel a
  scope: an echo never cancels, and fx-sz7's unstarted echo leaves its
  scope unaffected. Ancestor causes pass down transitively: each
  intermediate scope's cause is "an enclosing scope's cancellation". The
  enclosing-first deadline tie (`03:1155-1156`) agrees with "a nested
  scope's effective deadline is the minimum over its ancestors"
  (`03:1183-1184`). A nested scope in a body whose enclosing deadline
  fires reports `Cancelled`, not `DeadlineExceeded`, since TASK-4 never
  rewrites the body's error. The tie rule extends that to equal
  deadlines, which matches the behavior for a later or missing nested
  deadline. Other same-instant events follow real-time or scheduler
  order (`03:1134-1135`). `cancel()` keeps TASK-4's "a failing child's
  own error is reported" (`03:1192-1193`).
- **Isolated tasks.** The isolated task inherits the caller's
  cancellation, so a checkpoint in it returns the caller's scope's
  cancellation, and its cause decides the bit. An isolated closure is
  `'static` (`03:1052`, `02:692-693`), so it can hold no `Task<'s>` of
  `bg` and cannot join one.
- **Chained limits and zero counts.** A spawn takes a slot in each count
  at once (`03:1113-1116`). A zero count anywhere in a chain, or in a
  clone of one, makes every `spawn` through it panic, cancelled scope
  and `shield` included. A spawn cancelled during a chained wait holds
  nothing and returns at once. Fairness between chained waiters stays
  unspecified on purpose.
- **Clones and escapes.** Shared counts (`03:1112-1113`) agree with
  core's "same authority" (`reference.md:343-344`) and with "all scopes
  from that `Tasks` share one count" (`03:1076-1077`). `limit` never
  drops a count, and `shield` changes none. An isolated closure spawns
  only through an owned clone, which shares the counts. I found no way
  to get a looser `Tasks` from a limited one.
- **`spawn_isolated` under `limit`.** It takes no slot, and its caller,
  child or body, is blocked while the isolated task runs
  (`03:1121-1122`). A limited child whose isolated task spawns through a
  shared count holds a slot while it waits for one. That is the deadlock
  paragraph's case (`03:1105-1110`), and the inherited cancellation
  breaks it.
- **§0, §1.1, §1.2, §1.3, §9.3.** The `Tasks` row (`03:85`), §1.2's
  limited `spawn` checkpoint (`03:210`), and §1.3's bound (`03:257`)
  agree with the diff. A zero count no longer yields a waiting `spawn`,
  so §9.3's deadlock report (`03:1532-1535`) covers only the deadlock
  paragraph's cases.
- **D8, 02 §8.2 and §8.3.** D8's echo definition (`01:537-539`) reads
  on a joiner that sees `Err(Cancelled)` from a join after such a
  cancellation. Origins, loans, the failure edge, and
  `spawn_isolated`'s `Owned` closure are untouched.
- **Core and companion.** Core 7.1 (`reference.md:588`,
  `reference.md:612-637`) is silent on zero counts, chains, clones, and
  join marks, and contradicts none of them. Companion 15.2 uses one
  unchained `limit(4)`. §12.2's unlimited `fetch_all` (`03:1842`) is
  unaffected.
- **Grep.** No other hit in `design/` contradicts the diff, and §13 lists
  no open item it settles.
- **Prose.** Plain and matching the chapter; no added line is over 80
  columns.

## Out of scope

- **D8's "suspend only by using a capability"** (`01:510-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`), still
  open from rounds 80 to 88.
