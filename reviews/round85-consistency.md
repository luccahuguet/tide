# Tide adversarial/consistency review, round 85: Codex round 84 fixes (fx-sz7)

## Verdict

**approve**: no blocking findings and two non-blocking notes.

This is the narrow Opus confirmation round over the fixes made in
response to `reviews/round84-codex.md`. B1, N1, and N3 are closed, and
each fix is accurate. N2 was left out on purpose: the core is at 24,995
of 25,000 tokens, and 03 §6.1 and TASK-3 state the exact rule. That
choice stands, as it did for round 83's N4. The fixes add no new
contradiction. One line added before round 84, not by its fixes, is
over 80 columns (N1 below).

## What was reviewed

The uncommitted working tree against HEAD `db1bebc`: `git diff --
design/`, SHA-256
`79d033f24f788667f017d9a4dc305cf6f55cb619ea2a42166602f5ba635f34a4`
(03, `reference.md`, `reference-companion.md`). I recomputed the hash
and it matches. `git diff --check` passes.
`scripts/count-tokens.sh design/reference.md` gives 24,995.

I read:

- `AGENTS.md`, fx-sz7 with its comments, `reviews/round83-consistency.md`,
  and `reviews/round84-codex.md`;
- the whole diff;
- in 03: §1.2's closing paragraph (`03:252-257`), §6.1's three new
  paragraphs (`03:1076-1110`), TASK-1 to TASK-6 (`03:1114-1185`),
  JOIN-1 and JOIN-2 (`03:1187-1199`), and §12.2 (`03:1807-1852`);
- the companion's header (`reference-companion.md:10-16`);
- a grep of `design/` for `unlimited`, `no limit`, and `limit(`.

I did not re-review the decision itself.

## Round 84 findings

- **B1, closed and accurate.** The doc comment for `fetch_all`
  (`03:1816-1819`) now says that "`tasks` must be unlimited: under
  `limit`, a child not started by the deadline never starts and fails the
  scope (§6.1)". The prose after it (`03:1846-1847`) says "`sim.tasks` is
  unlimited". I traced Codex's case through the rules, with
  `tasks.limit(1)` and two URLs that both outlast the budget. Child 0
  holds the slot, and the body waits in `spawn` for child 1. The scope's
  deadline cancels the scope, and the body is inside that scope, so the
  limited `spawn` returns and starts nothing (`03:1080-1083`). The target
  scope is cancelled by its own deadline, so child 1 "counts as cancelled
  by the deadline" (`03:1086-1091`). It is not an echo, because neither
  the body's `Err` nor a sibling's failure caused the cancellation
  (TASK-2). Child 0 turns its cancellation into `Ok(Err(Cancelled))`.
  `t.join()` on child 1 returns `Err(TaskError::Cancelled.into())`
  (JOIN-2), so `collect` makes the body return `Err`. TASK-2 rule 2
  comes first: child 1 is the earliest-spawned failing child, so `scope`
  returns `E::from(TaskError::DeadlineExceeded)` (TASK-4). The function
  returns `Err(FetchError::Task(DeadlineExceeded))` and loses the partial
  results, as the comment says. "Not started by the deadline" also
  covers a `spawn` that the body reaches only after expiry, which starts
  nothing even if a slot is free (level-triggered). The precondition is
  "unlimited", not "a limit of at least `urls.len()`", and that is
  right: the count is shared by every scope from that `Tasks`
  (`03:1076-1078`), so a large enough limit is not a guarantee either.
  Nothing in `design/` says `sim.tasks` is limited. `limit` is an
  attenuation (`reference.md:588`) that a program applies "where a
  program needs a bound" (`03:257`), so the unattenuated root `Tasks` is
  unlimited. The test is unchanged and still holds.
- **N1, closed.** `03:1107-1109`: cancelling the scopes of the tasks
  waiting in `spawn` "breaks it unless they wait under `shield`
  (TASK-6)". This matches TASK-6, under which operations inside `shield`
  ignore the cancellation of enclosing scopes. See N2 below on one edge.
- **N3, closed.** `reference-companion.md:14` records 24,995, which
  matches the recount.
- **N2, not applied on purpose.** 03 §6.1 (`03:1080-1093`) and TASK-3
  (`03:1154-1155`) state the exact rule. Codex rated N2 non-blocking and
  required any clarification to keep the budget. With 5 tokens of
  headroom, leaving it is reasonable.

## Blocking findings

None.

## Non-blocking notes

### N1. The TASK-3 line is 97 columns

`03:1155` reads "cancellation by starting nothing and drops its closure
(§6.1). An in-flight kernel operation is". It is the only added prose
line over 80 columns; the other two are table rows, like every row in
those tables. The round 84 fixes did not add it. It came from the
response to round 83's N3, after round 83 had checked widths. Rewrapping
TASK-3 is a text-only fix that changes no meaning.

### N2. "Unless they wait under `shield`" is broader than TASK-6

TASK-6 shields a task only from scopes that enclose the shield. Take a
task that waits in `spawn` into a scope opened *inside* a `shield`.
Cancelling that scope still makes its `spawn` return, even though the
task is "under `shield`". The sentence holds for the case that matters,
because a deadline or `cancel()` outside the shield does not reach the
task. A more exact wording would be "unless a `shield` lies between them
and the cancelled scope". This is a nit; the pointer to TASK-6 already
gives the rule.

## Checked and consistent

- **The fixes add no new contradiction.** The doc comment's "fails the
  scope" agrees with TASK-2 rule 2 and TASK-4. "Never starts" agrees with
  "starts nothing" (`03:1082`), and "(§6.1)" points to the rule. The
  prose's "`sim.tasks` is unlimited" sits before the deadline narrative
  and leaves it unchanged. The deadlock paragraph's last sentence,
  "Otherwise the simulator reports a global deadlock once no task can
  run (§9.3)", now covers both the shielded case and the case where no
  one cancels.
- **The fixes agree with companion 15.2.** The new §6.1 cost sentence
  (`03:1098-1101`) and companion 15.2's prose
  (`reference-companion.md:242-246`) say why a limited scope needs
  operation deadlines. §12.2's precondition is the same point from the
  other side, for an unlimited scope with a scope deadline.
- **Prose.** Plain, matching the chapter. The lines that the round 84
  fixes changed are all within 80 columns.

## Out of scope

- Round 83's N2: "outlast the scope's deadline" (`03:1097`) still names
  the scope, where "the caller's" would match `03:1080`. It is
  unchanged and still non-blocking.
- Round 84's N4 (`limit(0)`, chained limits, isolated tasks and the
  count, and TASK-2's echo attribution in nested scopes) is still open.
  It was already there before this change.
