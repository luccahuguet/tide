# Tide adversarial/consistency review, round 80: limited spawn and cancellation (fx-sz7)

## Verdict

**revise**: one blocking finding and five non-blocking notes.

The decision itself (a limited `spawn`'s wait suspends, is shielded like a
join, and in a cancelled scope still starts the child) is coherent with
§0, §1.1, §1.2, TASK-2 to TASK-5, JOIN-2, §9.3, D8, and core 7.1, and it
keeps companion 15.2's "every URL gets an outcome" true. But the new
§6.1 paragraph claims the wait always ends once the scope is cancelled,
and that is false when the live children hold slots while waiting for
another slot of the same limit. A pattern the design itself endorses
(a child spawning into its scope, 02 §8.2 "Grandchildren"), and very
likely the core's own two examples composed, then deadlock in a way
that neither cancellation nor a deadline can break (B1).

## What was reviewed

The uncommitted working tree against HEAD `db1bebc`: `git diff --
design/`, SHA-256
`99b1137210a195f497b12abb1c927c5e35c729b88be32bab4386bd0d5937333a`
(03 and `reference.md`). I recomputed the hash. `git diff --check`
passes. The only added lines past 80 columns are the two table rows
(`03:86`, `03:211`), as for every row in those tables.
`scripts/count-tokens.sh design/reference.md` gives 24,993.

I read:

- `AGENTS.md`, fx-sz7 and its decision comment, and round 73's finding
  (`reviews/round73-consistency.md:190-195`);
- the whole diff;
- in 03: §0 Terms (`03:31-63`), §1.1's table and the `Receivers` and
  `File::close` notes (`03:67-200`), §1.2 (`03:205-223`), §1.3's
  deadlock sentence (`03:253-258`), §6.1 to §6.3 (`03:1043-1180`), §6.5
  (`03:1221-1269`), §7.1 SUSP-1 to SUSP-3 (`03:1285-1305`), and §9.3
  (`03:1480-1502`);
- `02-memory.md` §8.2 and §8.3 (`02:657-734`);
- `01-design.md` D5 and D8 (`01:356-379`, `01:509-565`) and D7's
  `Tasks` line;
- core 6.8's scheduling bullets (`reference.md:530-541`), 7.1, 7.2, and
  7.3's `Tasks::limit` pointer (`reference.md:574-700`);
- companion 14.3's `Semaphore` row and 15.2
  (`reference-companion.md:95`, `184-246`);
- a grep of `design/` for `limit`, `spawn`, `checkpoint`, `shielded`,
  "never interrupted", and "cannot be cancelled".

## Blocking findings

### B1. "The wait ends when a live child finishes" fails when a child waits for a slot while holding one

`03:1089-1091`: "The wait ends when a live child finishes; in a cancelled
scope each live child stops at its next checkpoint, so the wait is as
soft as a deadline (TASK-5)." Core 7.1 (`reference.md:636`) now also says
the wait is "never interrupted", with "(provisional)" removed.

A live child never reaches its next checkpoint if it is itself in a
limited `spawn`'s wait, which the diff makes uninterruptible. Two routes
lead there:

1. **Same scope.** `Scope` is `Copy + Send + Sync` (`reference.md:592`),
   and "A child may capture the `Scope`" (`02:694`). Under `limit(n)`, if
   all `n` live children call `s.spawn(..)` (a crawler that spawns a task
   per discovered link), each holds a slot and waits for one. No child
   finishes, so no slot frees.
2. **Nested scope from the same `Tasks`.** The new text says "scopes from
   that `Tasks` run at most `n` children at once" (`03:1077-1078`, same
   wording in `reference.md:635`). Read as one count shared across
   scopes, which the "attenuation" framing (`reference.md:588`) and
   companion 14.3's `Semaphore` mapping (`reference-companion.md:95`)
   suggest, the core's own idiom deadlocks once its two examples are
   composed. Take `let tasks = root.tasks.limit(4);`
   (`reference.md:634`), spawn four children, and have each call
   `fetch_both(&tasks, ..)` (`reference.md:647`). Each child opens a
   scope from the limited `Tasks` and waits in `spawn` for a slot that
   only the four waiting children can free.

In both cases the scope's deadline passes, the scope is cancelled, and
nothing moves: the waits are not checkpoints (`03:1079-1080`), and JOIN-1
keeps scope exit waiting. This is a permanent hang, not the bounded delay
"soft" means in TASK-5 (`03:1153-1160`). The simulator would report it as
a global deadlock (`03:1491-1493`), so the design does not lose the bug,
but the paragraph's liveness claim is wrong and the reasoning leaves out
the one alternative this matters for. Rejected option (a) (in a cancelled
scope, return a `Task` whose child never runs) would let a deadline break
both deadlocks. The text weighs (a) only against the lost per-item values
(`03:1086-1088`).

The diff did not create the hazard: the provisional core rule already
said "not a checkpoint". But fx-sz7 is the decision on how the wait meets
cancellation, and the new normative text asserts termination. The
counting domain ("scopes from that `Tasks`": one count or one per scope)
decides whether route 2 exists, and only 03 can own that answer.

**Possible fix** (03 §6.1 only; the core can stay as it is):

- State the counting domain. One count per limited `Tasks` value, shared
  by every scope from it, matches "attenuation" and the `Semaphore`
  mapping. A per-scope count removes route 2 but not route 1.
- Replace the "wait ends" sentence with a true one, for example: "The
  wait ends when a live child of that `Tasks` finishes. A task that holds
  a slot and waits for another, such as a child that spawns into its own
  scope or opens a scope from the same limited `Tasks`, can deadlock, and
  cancellation does not break it; the simulator reports it as a global
  deadlock (§9.3). Give such children an unlimited `Tasks` or a separate
  limit."
- Add one clause to the rejection of (a) saying why its liveness benefit
  does not outweigh the lost values. Or, if it does, choose a variant
  that both keeps the bound and lets cancellation end the wait (for
  example, in a cancelled scope `spawn` returns at once with a child
  that is recorded as cancelled before it started), and rework TASK-2's
  accounting for such children.

## Non-blocking notes

### N1. The "no error to carry a cancellation" reason is circular in 03

`03:1080-1081` justifies the shielded wait "because `spawn` returns a
`Task`, not a `Result`". The real alternative was to change that, and the
bead's decision comment rejects it as option (c): it "changes the
`std::thread::scope` shape at every call site for a wait only limited
scopes have." 03 owns the decision (AGENTS.md, Decisions), but only the
bead records that reason. One clause in §6.1 would close it, for example
"and making `spawn` a fallible checkpoint would change Rust's
`std::thread::scope` signature at every call site for a wait only
limited scopes have."

### N2. Core 7.2's list of non-checkpoints omits the limited `spawn`

`reference.md:665-667`: "authority-free code, `Shared::with*`, joins,
scope exit, and `File::close` are not, and are never interrupted." That
reads as the complete list. 7.1 (`reference.md:636`) states the `spawn`
case, so nothing contradicts it, and the core has 7 tokens of headroom.
If the author wants the list complete, move "(not a checkpoint, never
interrupted)" out of 7.1 and add "a limited `spawn`," to 7.2's list,
which costs about the same. Leaving it as it is would also be fine.

### N3. Checking `tasks.checkpoint()` in the body has a cost the sentence hides

`03:1091-1092`: "A body that should stop spawning once its scope is
cancelled checks `tasks.checkpoint()` in its loop." This is true:
`checkpoint` is a `&self` method on `Tasks` (`03:1056`), the body can
reach the outer `Tasks` it called `scope` on, and TASK-2 and TASK-4
cancel the body with the scope. But `checkpoint` returns
`Result<(), Cancelled>`, so `?` needs `E: From<Cancelled>` (scopes
require only `From<TaskError>`). And if the body returns that error,
TASK-4 reports the body's error ("never rewrites the body's error"),
which drops the partial results that companion 15.2 depends on. A body
that `break`s on the error keeps them, but only for the items it
spawned. Suggest "checks `tasks.checkpoint()` in its loop and stops, at
the cost of an outcome for each unspawned item".

### N4. Wording in the new §6.1 paragraph

- `03:1087-1088`: "and drop those values". "Those" has to reach back
  across a clause to "a child that turns cancellation into a value".
  Try "and lose the value such a child would report".
- `03:1088-1089`: "would exceed the limit, which is an attenuation". The
  point is that the limit cannot be exceeded because it is an
  attenuation. Try "would break the limit, an attenuation".

### N5. TASK-5's ragged wrap

`03:1154-1157`: after the insertion, "(§1.1), which can block on a network
file system" is a short line followed by "even inside a shielded scope
with a deadline." Reflow the bullet. The "which" clause still attaches
to `File::close`, as intended.

## Checked and consistent

- **§0, §1.1, §1.2, and core 7.1 agree.** §0 lists "a limited `spawn`'s
  wait for a slot" among non-checkpoints (`03:58-60`). §1.2's `Scope`
  row says "`spawn` under `Tasks::limit`, shielded (§6.1)" (`03:211`),
  in the same form as the `Task` row's "yes, shielded". §1.1's `Tasks`
  row correctly leaves `spawn` out of "Suspends", since `spawn` is a
  `Scope` method covered by §1.2, and its attenuation cell now points to
  §6.1 (`03:86`). Core 7.1 says the same (`reference.md:634-638`).
  Round 73's finding (`03:207` "Waits: none") is closed.
- **§0 "Suspend".** "Only capability and synchronization operations
  suspend" (`03:54-55`) covers `spawn`, a synchronization-handle
  operation. D8's "A function can suspend only by using a capability"
  (`01:511-513`) is looser, since joins and channel operations already
  suspend without being capability operations. That wording predates the
  diff, which adds one more synchronization case of the same kind.
- **SUSP-1.** A body holding a `Scope` has a non-`Pure` capture or
  parameter, so it is already classified may-suspend (`03:1287-1293`). No
  new classification is needed.
- **JOIN-1 and JOIN-2.** "Shielded, like a join (JOIN-2)" means the same
  as in JOIN-2: not a checkpoint, returns only when it can
  (`03:1173-1175`). `03:1079` uses "shielded" as §1.1 and §1.2 do.
- **TASK-2 to TASK-4.** A child started after cancellation fails its
  first checkpoint. If a body `Err` or a sibling failure caused the
  cancellation, its `Err` is an echo (one bit per child, `03:1119-1123`).
  If a deadline did, it is reported as `DeadlineExceeded` if it is the
  earliest-spawned failing child. In both cases it behaves as in an
  unlimited scope where the child was spawned after the cancellation, so
  "decide the outcome as for an unlimited scope" holds.
- **Rejected option (a).** JOIN-2 returns `Err(TaskError::Cancelled)` for
  a cancelled child. In a deadline-cancelled scope whose other children
  return `Ok`, a skipped child's join would make the body's `x.join()?`
  fail, and rule 3 would return `Cancelled` in place of the partial
  results. "No child error behind it" is accurate.
- **Companion 15.2.** `fetch_one` maps `e.is_cancelled()` to
  `Outcome::TimedOut` (`reference-companion.md:209-215`), so under the
  decision every URL past the first four still gets an outcome. The
  reference to "`fetch_one` ... in companion 15.2" is accurate.
- **§9.3.** `spawn` is already a scheduling point
  (`03:1494-1496`), and a permanent wait falls under the global deadlock
  report (`03:1491-1493`).
- **D8 and §1.3.** "Production cannot deadlock where the simulator makes
  progress" (`01:547-549`, `03:255-257`) survives: B1's deadlock occurs
  under both.
- **02 §8.2 and §8.3.** The decision changes no origin or failure rule.
  The wait happens inside the body or a child, before the scope's
  join, so the loans in `σ` are unaffected.
- **Grep.** No other hit in `design/` for `limit`, `spawn`,
  `checkpoint`, or `shielded` contradicts the decision.

## Out of scope

- **`spawn_isolated` under `limit`.** `tasks.limit(n).spawn_isolated(f)`
  creates a task, but "scopes from that `Tasks` run at most `n`
  children" (`03:1077`) speaks only of scopes. Whether such a task takes
  a slot, and so waits, is unstated in 03 and the core. Its wait would be
  shielded either way (`03:86`). This predates the diff.
- **Companion 15.2's doc comment.** "Every URL gets an outcome by the
  deadline" (`reference-companion.md:195`). With more than four URLs, the
  queued children start after the deadline and report `TimedOut` just
  after it in production (instantly in virtual time). "By" is slightly
  strong. This predates the diff.
