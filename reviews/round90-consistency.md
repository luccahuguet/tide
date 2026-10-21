# Tide adversarial/consistency review, round 90: limit edge cases and echo attribution (fx-0ai)

## Verdict

**approve**: no blocking findings, two non-blocking notes.

The revision closes round 89's B1 and N1 to N3. The join rule now has
one example, and in the case it is meant for, a child that calls
`x.join()?` itself, the example agrees with the rule. Under the rule
every adversarial case from rounds 82, 87, 88, and 89 comes out as those
rounds settled. The `limit` rules are unchanged from round 89 and still
hold. One narrow nested reading of the example still disagrees with the
rule (N1). The rule is stated first and is exact, so this is wording,
not a gap an implementer has to guess at. The core is unchanged and
contradicts nothing.

## What was reviewed

The uncommitted working tree against HEAD `fceb12e`: `git diff --
design/`, SHA-256
`07ddff4f55dd6ac9906bea4fce2d7f583665fe913e7d07ff3168fa22fb99faa7`
(03 only). I recomputed the hash. `git diff --check` passes, and no
added line is over 80 columns. `scripts/count-tokens.sh
design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-0ai with all four comments (the decision and the
  revisions after rounds 87, 88, and 89), and the blocking findings of
  `reviews/round87-consistency.md`, `round88-consistency.md`, and
  `round89-consistency.md` in full;
- the whole diff (`03:1105-1123`, `03:1148-1168`, `03:1257-1259`);
- in 03: §0 (`03:31-63`), §1.1's `Tasks` row (`03:85`), §1.2
  (`03:203-213`), §6.1 to §6.4 (`03:1042-1261`), including fx-sz7's
  limited-spawn text (`03:1076-1103`), and §9.3 (`03:1522-1543`);
- `01-design.md` D8 (`01:509-555`);
- `02-memory.md` §8.2 and §8.3 (`02:657-735`);
- core 6.6's capability notes (`reference.md:336-350`) and 7.1
  (`reference.md:580-655`);
- companion 14.3's `limit` and `spawn_isolated` rows
  (`reference-companion.md:53-98`) and 15.2 (`reference-companion.md:200`);
- a grep of `design/` for `limit(0`, `chunks(0`, `echo`, `echo bit`,
  `whose bit`, `observed`, `first event`, and `spawn_isolated`.

## Earlier findings

- **Round 89 B1, closed.** Both general examples are gone. The one left
  (`03:1163-1164`), "a child that fails through `x.join()?` after a
  sibling's failure cancelled this scope is an echo too", is the one
  round 89 proposed. When the child calls the join itself, a checkpoint
  at that point returns this scope's cancellation, whose cause is the
  sibling's failure, so the rule sets the bit as the example says. Round
  87's server case no longer matches the example, since `bg` is not
  cancelled by a sibling's failure there. N1 below covers the one nested
  reading that still differs.
- **Round 89 N1, closed.** "in the child, in a task it waits for with
  `spawn_isolated`, or in a scope nested in either" (`03:1150-1153`)
  names the fan-out scope that an isolated request task opens. The reach
  sentence (`03:1157-1161`) applies to it unchanged.
- **Round 89 N2, closed.** "as if no `shield` were in force"
  (`03:1163`) picks the reading that keeps round 82's case marked when
  the join sits under a user's `shield`.
- **Round 89 N3, closed.** "whose bit is clear" replaces "observed" in
  TASK-2 (`03:1168`) and §6.4 (`03:1259`). The `spawn_isolated` sentence
  is its own paragraph (`03:1122-1123`). The zero-count override of
  `03:1080-1082`, TASK-3 (`03:1178-1179`), and core
  (`reference.md:635-637`) stays explicit at `03:1119-1120`, which round
  89 accepted given the core budget.

## Blocking findings

None.

## Non-blocking notes

### N1. The example still reads on a join in a nested scope

The example (`03:1163-1164`) does not say where the join runs. If `H`,
a child of `bg`, calls `x.join()?` in the body of its nested scope `r`,
`H` still "fails through `x.join()?`". Take round 87's server and add a
sibling failure. `r`'s deadline passes. `H`, as `r`'s body, calls
`bg.spawn(f)`, and the spawn starts nothing (`03:1080-1082`). Then
sibling `S` fails and cancels `bg`. `H` calls `G.join()?`. The example
says `H` is an echo. The rule says it is not: a checkpoint in `r`'s body
returns `r`'s cancellation, whose one cause is `r`'s own deadline
(`03:1153-1157`), and that cancellation did not reach `r` from `bg`
(`03:1157-1161`). So `H`'s error counts under rule 2 against `S`'s by
spawn order, which is right: the request's deadline is a real failure
of `H`. The same split appears when `r` was first cancelled by a failure
among its own children.

The rule comes first, says "exactly", and is the only thing an
implementer can build, so this does not block. The natural reading of
the example is the direct join anyway. One word would close it, for
example "a child that calls `x.join()?` directly after a sibling's
failure cancelled this scope is an echo too".

### N2. Wording

- `03:1164-1166`: "A child / that fails with no bit set is an /
  independent failure" leaves a 34-column line in the middle of the
  bullet. Reflowing the paragraph fixes it.
- `03:1257-1259`: "(TASK-2)" now appears twice in one sentence. The
  second, after "whose bit is clear", can go.

## Checked and consistent

- **Echo rule against earlier rounds.** With the rule as written:
  - Round 82: `A` joins `B` after `B`'s failure cancelled `bg`. A
    checkpoint in `A` returns `bg`'s cancellation, whose cause is `B`'s
    failure, so `A` is an echo and `B` is reported. This needs `B`'s
    failure to cancel `bg` before `B`'s join returns, which TASK-2's
    "the first failure ... cancels the scope" (`03:1135-1137`) implies.
  - Round 87 B1: the unstarted echo `G` in an uncancelled `bg`. A
    checkpoint in `r` returns `r`'s cancellation, caused by `r`'s own
    deadline, so `H`'s bit stays clear and its error is an ordinary
    failure of `bg`. This matches fx-sz7's "when any checkpoint would"
    (`03:1083-1084`).
  - Round 87 N1: `C`, a child of `H`'s nested scope `inner`, joins a
    failed child of `bg` after a sibling's failure cancelled `bg`.
    `inner`'s cause is the enclosing cancellation, which reached it from
    `bg`, so `H`'s bit is set.
  - Round 88 B1(a): `bg` later cancelled by its own deadline,
    `cancel()`, or an ancestor. `r`'s cause came first, so no bit, and
    `H`'s error counts under TASK-4 (`03:1185-1193`).
  - Round 88 B1(b): `inner` first cancelled by `K`'s failure. A
    checkpoint in `C` returns `inner`'s cancellation, caused by `K`, so
    no bit, and `K`'s error reaches `bg` as `H`'s result, as
    `03:1159-1161` says.
  - Round 88 N1 and round 89 N1: a sibling's failure cancels `bg`; the
    isolated request task, or its fan-out scope `f`, sees it. A
    checkpoint in either sets `H`'s bit, and a cause inside `f` stays
    `f`'s. An isolated task run from deeper down, by a child of a scope
    nested in `H`, is "inside" that scope in TASK-3's sense ("every
    checkpoint inside it, in any task", `03:1174-1175`), so it is
    covered as a scope nested in `H`.
  - Round 89 N2: `H`'s shielded cleanup scope `s` with its own deadline.
    As if no `shield` were in force, `s` would have been cancelled
    through `bg` first, so the join sets the bit. A runtime can decide
    this by comparing `s`'s own events with the enclosing chain's first
    cancellation, both real events.
  - The body joins: no bit exists, and rule 2 still outranks rule 3.
    A caught join error leaves the bit set, as a caught checkpoint
    error does.
- **One cause.** The cause list (`03:1153-1156`) covers every event that
  can cancel a scope; an echo is not a failure and cancels nothing, and
  fx-sz7's unstarted echo leaves its scope unaffected. One cause also
  sharpens TASK-4: a deadline that passes after a sibling's failure
  cancelled the scope did not cancel it, so no child counts as cancelled
  by that deadline. The enclosing-first tie (`03:1156-1157`) agrees with
  TASK-4's minimum over ancestors (`03:1183-1184`) and "set on this
  scope, not an ancestor" (`03:1186-1187`).
- **D8 and §0.** D8's echo, an error returned "after observing a
  cancellation caused by the body's `Err` or a sibling's failure"
  (`01:536-538`), reads on a joiner that sees `Err(Cancelled)` at a
  point where a checkpoint would see that cancellation. §0 still says
  joins are not checkpoints (`03:56-59`); the rule borrows the
  checkpoint's answer without making the join one.
- **`limit`.** Clones share counts (`03:1112-1113`), in line with core's
  "same authority" (`reference.md:343-344`) and "all scopes from that
  `Tasks` share one count" (`03:1076-1077`). Chained counts are taken at
  once (`03:1113-1116`), so no spawn holds one while it waits for
  another, and a spawn cancelled in a chained wait holds nothing. A zero
  count anywhere in a chain or clone makes every `spawn` through it
  panic, cancelled scope and `shield` included (`03:1117-1120`);
  `scope`, `spawn_isolated`, `checkpoint`, `shield`, and handle
  constructors still work, so `limit(n.min(len))` on empty input is
  safe. §9.3's deadlock report (`03:1532-1535`) no longer has a
  zero-count case to cover. The deadlock paragraph's "a `Tasks` that
  shares its count" (`03:1106-1107`) covers clones and chains.
- **`spawn_isolated` under `limit`.** It takes no slot, and its caller
  is blocked while it runs (`03:1122-1123`), so concurrent work does not
  grow. An isolated closure is `'static` (`03:1052`, `02:692-693`): it
  can spawn only through an owned clone, which shares the counts, and it
  cannot hold a `Task<'s>` of `bg`, so it never joins one. A limited
  child whose isolated task spawns through a shared count is the
  deadlock paragraph's case, and the inherited cancellation breaks it.
- **§1.1, §1.2, §6.4.** The `Tasks` row (`03:85`), §1.2's limited
  `spawn` checkpoint (`03:210`), and §6.4's disconnect paragraph
  (`03:1257-1261`) agree with the diff.
- **D5, D7, 02 §8.2 and §8.3.** Origins, loans, the `Owned` closure of
  `spawn_isolated`, and handle constructors taking `&Tasks` are
  untouched.
- **Core and companion.** Core 7.1 (`reference.md:588`,
  `reference.md:603-637`) is silent on zero counts, chains, clones, and
  join marks, and contradicts none of them. Companion 15.2 uses one
  unchained `limit(4)`.
- **Grep.** No other hit in `design/` contradicts the diff.
- **Prose.** Plain and in the chapter's style; `git diff --check` is
  clean and no added line is over 80 columns.

## Out of scope

- **D8's "suspend only by using a capability"** (`01:511-512`) against
  03 §0's "capability and synchronization operations" (`03:54-55`), still
  open from rounds 80 to 89.
