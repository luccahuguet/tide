# Tide adversarial/consistency review, round 87: limit edge cases and echo attribution (fx-0ai)

## Verdict

**request changes**: one blocking finding and five non-blocking notes.

Three of the four decisions are sound and fit the chapter: `limit(0)`
panics, a limit on a limited `Tasks` adds a count acquired atomically
with the old one, and `spawn_isolated` takes no slot. The nested-scope
echo rule and the single cause per scope also hold under every case I
tried: ancestor cancellation, deadlines, `cancel()`, panics, and nested
scopes with their own causes. The sibling-join mark closes round 82's
case. It is unconditional, though, and that breaks one case fx-sz7
settled: a child that joins an unstarted sibling which left the scope
unaffected (§6.1) becomes an echo in a scope nobody cancelled, so its
error is lost (B1). One added condition fixes it. The core is unchanged
and contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `fceb12e`: `git diff --
design/`, SHA-256
`c7d5b4a2138e8cdb5a6d114b6e721eec5f07af122cad9908a82e6a919ef871e1`
(03 only). I recomputed the hash. `git diff --check` passes, and no
added line is over 80 columns. `scripts/count-tokens.sh
design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-0ai with its decision comment, and rounds 82 to 84
  (`reviews/round82-consistency.md`, `round83-consistency.md`,
  `round84-codex.md`);
- the whole diff (`03:1112-1119`, `03:1149-1158`);
- in 03: §0 (`03:31-63`), §1.1's `Tasks` row and receivers note
  (`03:85`, `03:179-199`), §1.2 and §1.3 (`03:203-257`), §6.1 to §6.6
  (`03:1042-1316`), §9.3 (`03:1515-1538`), and §12.2's `fetch_all`
  (`03:1830-1847`);
- `01-design.md` D5 and D8 (`01:509-564`);
- `02-memory.md` §8.2 and §8.3 (`02:657-727`);
- core 6.6's capability notes (`reference.md:343-347`), 7.1 to 7.4
  (`reference.md:581-740`);
- companion 14.3 (`reference-companion.md:84-99`) and 15.2
  (`reference-companion.md:184-246`);
- a grep of `design/` for `limit`, `echo`, `spawn_isolated`, `join`,
  `atomically`, and `FIFO`.

## Earlier findings

- **Round 82 Out of scope, "a child that fails through a grandchild's
  `join` after an echo", closed.** The case is a child `A` that spawns
  `G` into its own scope after sibling `B` failed and returns `G`'s
  `join` error. `G` is `A`'s sibling, so `03:1155-1157` marks `A`, and
  rule 2 reports `B`'s error. See N1 for the variant it misses.
- **Round 83 Out of scope, TASK-2's echo cause in nested scopes,
  closed** by `03:1149-1155`.
- **`limit(0)`, `spawn_isolated` under `limit`, `limit` on a limited
  `Tasks`** (rounds 80 to 82), **closed** by `03:1112-1119`.
- **Round 83 N1, reopened by the new join mark** (B1).

## Blocking findings

### B1. The join mark turns an unstarted echo into a lost error

`03:1155-1157`: "A child whose `join` of a sibling returns `Err` is
marked too, since the scope reports that sibling's outcome itself
(JOIN-2)." The reason fails for one kind of sibling: the unstarted
child of `03:1090-1093`, which "counts as an echo and that scope is
unaffected". The scope reports nothing about it.

Take round 82's server. Handler `H`, a child of `bg`, opens request
scope `r` with its own deadline. After the deadline, `H` (as `r`'s body)
calls `bg.spawn(f)`. `H` is in a cancelled scope, so the spawn starts
nothing. `bg` is not cancelled, so the unstarted task `G` is an echo and
`bg` is unaffected. `H` then calls `G.join()?`. The join returns `Err`.
`G` is `H`'s sibling in `bg`, so the new sentence marks `H`. `H` returns
the error, which makes it an echo. Echoes are not failures and do not
cancel a scope (`03:1131-1133`). So `bg` keeps running, rule 2 skips
`H`, and unless something else fails, `bg` returns its body's `Ok`. The
request's error goes only to the `debug!` record.

This conflicts with four texts:

- The diff's own nested rule (`03:1153-1155`): a cause inside the
  nested scope "is that scope's to report, and it reaches this scope as
  the child's own result". The cause here is `r`'s deadline, so `H`'s
  `Err` should count in `bg` as `H`'s own result. If `H` had returned
  `r`'s `DeadlineExceeded` from any other checkpoint, it would fail `bg`.
- fx-sz7's settled reading. Round 83 N1
  (`reviews/round83-consistency.md:88-102`) named this exact path: an
  echo "in a `bg` that nobody cancelled, a state TASK-2 does not
  define", and the intended reading is "an ordinary failure of `bg`".
  fx-sz7 adopted that with "when any checkpoint would" (`03:1083-1084`).
  Round 84 (`reviews/round84-codex.md:41`) adds: "Propagating the
  returned join error can still fail the caller normally." The bead puts
  changes to fx-sz7's decision out of scope.
- D8 (`01:537-539`, `01:545-546`): an echo is an error returned "after
  observing a cancellation caused by the body's `Err` or a sibling's
  failure", and "an independent child failure is never hidden". Nothing
  in `bg` caused this one.
- The outcome also depends on who joins. If a grandchild in `r` joins
  `G`, no sibling join takes place, `r` returns `DeadlineExceeded`, and
  `H` fails `bg` when it propagates that.

**Fix.** Mark only when the scope reports the sibling, for example: "A
child whose `join` of a sibling returns `Err` while this scope is
cancelled is marked too". A sibling whose `join` returns `Err` either
failed or panicked, which cancels the scope, or was cancelled with the
scope, or is the unstarted echo of §6.1. Only the last kind leaves the
scope uncancelled. With this condition, the `cancel()` and ancestor
cases keep their gain: `A` stops outranking `B`'s own error. An
alternative is an explicit exception: "unless that sibling is an
unstarted child that left this scope unaffected (§6.1)".

## Non-blocking notes

### N1. A join from a scope nested in a child is not marked

The mark covers only a `join` by the sibling task itself. Suppose `T` in
`bg` fails, and a task `C` joins `T`. `C` is a child of scope `inner`,
nested in `bg`'s child `H`, and holds `T` because `T` was moved to it or
because `C` spawned `T` through `bg` passed down. `C` is no sibling of
`T`, so neither `C` nor `H` is marked. `C`'s `Err(Cancelled)` is an
ordinary failure of `inner`, since the cause reached `inner` from `bg`.
If `H` hit no checkpoint, it propagates the error with no bit set. If
`H` was spawned before `T`, rule 2 then reports `Cancelled` in place of
`T`'s error. This is round 82's class of bug, one level down. Combined
with B1's fix, one sentence covers both: "A `join` of one of this
scope's tasks that returns `Err` while this scope is cancelled sets the
bit of the child it runs in, directly or in a nested scope, as a
checkpoint there would."

### N2. `limit(0)`'s reason overreaches; the panic hits empty inputs

`03:1113-1114`: "code that may start no child is better given no
`Tasks`". A `Tasks` also gives `checkpoint`, `shield`, `spawn_isolated`
(which, after this diff, takes no slot), and the only way to create
`Shared`, channels, `Arc`, and atomics (§0, `03:41-45`). So code that
needs those but should start no child cannot be "given no `Tasks`".
Panicking in `limit` also turns `tasks.limit(n.min(items.len()))` on
empty input into a crash, though that code would never spawn. Rust's
`Semaphore::new(0)` is legal and `chunks(0)` is not, so either choice
has a precedent. Panicking at the first limited `spawn` under a zero
count would keep the bug report at the point of misuse and make an
unused zero count harmless. If the eager panic stays, a reason like
"every limited `spawn` from it would wait until cancelled, a bug
better reported where the count is made" avoids the overreach.

### N3. The deadlock paragraph's "the same `Tasks`" misses chained limits

`03:1105-1107`: "opens a nested scope from the same `Tasks`". With
chained counts, a child holding a slot of `outer` that opens a scope from
`outer.limit(k)` and spawns has the same hold-and-wait: the new child
needs an `outer` slot (`03:1115-1116`). The same holds for a clone, if
clones share the count (Out of scope). "From a `Tasks` that shares a
count with it" covers every case.

### N4. LOCK-3 does not state the property the analogy borrows

`03:1116-1117`: "as LOCK-3 does for cells, no spawn holds one count while
it waits for another". LOCK-3 says cells are "acquired atomically in a
global order" and that "acquisition waits per cell in FIFO order"
(`03:1277-1279`). It never says no cell is held during a wait, and
ordered acquisition normally holds the lower cells. "Taking every count
at once, as `with2` takes its cells" claims less. Fairness is also
unspecified. A waiter that needs both counts can lose every freed outer
slot to waiters that need only the outer count, because an inner child's
exit frees both slots together. The simulator makes the result
deterministic, and §6.1 never promised FIFO for single counts, so this is
not new.

### N5. Wording

- The cause list (`03:1150-1151`) omits a panic in the body, which also
  cancels the scope (`03:1126-1127`). It does not change any outcome,
  since the panic keeps unwinding, but the list reads as complete.
- "The first event" does not settle a tie: a scope's own deadline set to
  the same instant as an ancestor's. TASK-4 settles only scope against
  operation deadlines (`03:1186-1188`). Scheduler order, as in TASK-2's
  first sentence, would do.
- "is marked too" (`03:1156`) uses a new word for "its bit is set".
- `03:1118-1119` gives the reason in terms of running tasks, but the
  count bounds live children (`03:85`, `03:1077`). The isolated task is
  no scope child either way, and the caller is suspended, so the bound
  on concurrent work holds. Under `threads` each nesting level still
  adds an OS thread.

## Checked and consistent

- **Echo rule, nested scopes with their own causes.** If `inner`, inside
  `bg`'s child `H`, is cancelled first by one of its own children's
  failures and `bg` is cancelled later by a sibling, checkpoints in
  `inner` return `inner`'s cancellation, so `H`'s bit stays clear. `H`
  returns the earlier independent error, and rule 2 orders it against
  the sibling by spawn order. In the reverse order the cause of `inner`
  is "an enclosing scope's cancellation", a checkpoint in `inner` sets
  `H`'s bit, and `H`'s propagated `Err` is an echo. A child of `inner`
  that fails there without a checkpoint is an independent failure in
  both scopes, as a flat child would be. Several levels of nesting work
  transitively.
- **Ancestor cancellation, deadlines, `cancel()`.** With the cause
  pinned to the first event, TASK-4's "after the scope's own deadline
  ... has cancelled the scope" (`03:1179-1180`) now has one reading: a
  failure that comes first keeps later errors echoes, and a deadline that
  comes first makes them count as `DeadlineExceeded`. An ancestor's or a
  `cancel()` cause sets no bit, so children's errors are failures, as
  round 83 recorded for unstarted children. An inherited effective
  deadline reaches a nested scope as an enclosing cancellation, which
  agrees with TASK-4's "set on this scope, not an ancestor".
- **Panics.** A sibling's panic is not a "failure" for the bit, but rule 1
  re-raises it whatever the bits say. A nested scope's panic re-raises in
  `H` and reaches `bg` as a child panic.
- **Sibling joins, apart from B1.** If the sibling failed independently,
  the scope reports it and the joiner is excluded. If it echoed, rule 3
  reports the body's error or rule 2 reports the real failure. If it was
  cancelled by the scope's own deadline, it is reported as
  `DeadlineExceeded` either way. If it was cancelled by `cancel()` or an
  ancestor, its own error is reported. If it panicked, rule 1 applies.
  An unstarted sibling in a cancelled target falls under TASK-2 and
  TASK-4 through `03:1085-1090`. A child that catches the join error and
  later fails for its own reason stays an echo. That is the same sticky
  bit as a child that catches a checkpoint's cancellation, and the scope
  is cancelled by a cause it reports, so D8's "never hidden" holds. The
  body's joins are unchanged (rule 2 over rule 3, `03:1163-1164`), and
  for the uncancelled unstarted case the body's `x.join()?` is reported
  as the body's `Err`, which is correct.
- **JOIN-1 and JOIN-2.** "The scope reports the actual failure"
  (`03:1211-1212`) is the reason the mark relies on, and the core's
  "(the scope reports the real cause)" (`reference.md:624`) states the
  same for a child that fails through a sibling's join.
- **Chained limits.** Each child holds a slot in both counts for its
  whole life, so the inner count's holders are a subset of the outer's,
  and the effective bound is the minimum. A larger inner `n` cannot widen
  it. Acquisition is atomic, so a spawn cancelled during a chained wait
  holds nothing to release and returns at once (`03:1080-1082`). Under
  `shield` it waits for both counts. Other scopes from the outer `Tasks`
  are bound only by the outer count, which matches "all scopes from that
  `Tasks` share one count" (`03:1076`).
- **Escape routes.** `spawn_isolated`'s closure is `'static` (`03:1052`,
  `02:692-693`), so it cannot capture a `Scope` or borrow the limited
  `Tasks`. It can spawn only through an owned clone, whose children then
  take slots (see Out of scope). A limited child that calls
  `spawn_isolated` holds its slot while it waits, so it cannot deadlock
  on its own slot unless the isolated task spawns into the same count.
  That case falls under the deadlock paragraph, and the isolated task
  inherits the caller's cancellation (`03:1129`), so the paragraph's
  remedy still works. Neither `shield` nor a passed-down `Scope` escapes:
  a spawn into a scope counts against the counts of the `Tasks` that made
  the scope.
- **§0, §1.1, §1.2.** The `Tasks` row's "`limit(n)` (live children,
  §6.1)" (`03:85`) and §1.2's "`spawn` under `Tasks::limit`, a
  checkpoint" (`03:210`) agree. `spawn_isolated` is already marked as a
  shielded wait.
- **§6.4 and LOCK-3.** The disconnect rule (`03:1250-1254`) depends only
  on the bit and stays true. Apart from N4, LOCK-3 is unaffected.
- **§9.3 and D8.** Deadlock detection and the claim that "production
  cannot deadlock where the simulator makes progress" (`01:547-549`,
  `03:255-256`) still hold: atomic multi-count acquisition adds no new
  wait-for edge between counts.
- **D5, 02 §8.2 and §8.3.** `limit(0)`'s panic is a bug report, as D5
  intends. Nothing changes origins, loans, or the failure edge.
- **Core and companion.** Core 7.1's signature comment "at most n live
  children" (`reference.md:588`), its bounded-concurrency bullet
  (`reference.md:634-638`), and its isolation bullet
  (`reference.md:630-633`) are silent on the new cases but not
  contradicted. Companion 15.2 uses one unchained `limit(4)`, and §12.2's
  `fetch_all` note (`03:1834-1836`) is unaffected.
- **Grep.** No other hit in `design/` for `limit`, `echo`,
  `spawn_isolated`, or `join` contradicts the diff.
- **Prose.** Plain, matching the chapter. No new line exceeds 80 columns.

## Out of scope

- **Whether a clone of a limited `Tasks` shares its counts.** Core says
  capabilities are `Clone` with "same authority; provisional"
  (`reference.md:343-344`), and §6.1 speaks of scopes "from that
  `Tasks`" (`03:1076`). If a clone got a fresh count, cloning would
  escape any limit, and `spawn_isolated` with a cloned `Tasks` would
  escape it too. "Same authority" implies shared counts, but neither
  text says it. This predates the diff.
- **D8's "suspend only by using a capability"** (`01:511-513`) against
  03 §0's "capability and synchronization operations" (`03:54-55`), still
  open from rounds 80 to 83.
