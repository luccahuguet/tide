# Tide adversarial/consistency review, round 92: Codex round 91 fixes (fx-0ai)

## Verdict

**approve**: no blocking findings, one non-blocking note.

This is a narrow confirmation round. It covers only the fixes for
`reviews/round91-codex.md`. Both are closed. B1: the failed-join rule
now borrows a checkpoint's view of the enclosing cancellations and does
not replay history without shields, so Codex's case reports `K`. N1:
isolation is transitive. The rewrap changed no words beyond the two
fixes. The fixes add nothing new except one intended consequence:
round 89 N2's shielded cleanup case is decided the other way (N1 below).

## What was reviewed

The uncommitted working tree against HEAD `fceb12e`: `git diff --
design/`, SHA-256
`78c5498a2eb2c5a0ae61386ff5005c90e101c5e508f1c20ed7a0b32b8dc3554d`
(03 only). I recomputed the hash. `git diff --check` passes, and no
added line is over 80 columns (the longest is 76). The core is
unchanged.

I read:

- `AGENTS.md`, fx-0ai with all five comments, including the revision
  after round 91;
- `reviews/round91-codex.md`, `reviews/round90-consistency.md`, and
  round 89's N2 (`reviews/round89-consistency.md:142-158`);
- the whole diff, and in 03: §0's checkpoint and suspend entries
  (`03:54-59`), fx-sz7's limited-spawn text (`03:1076-1093`), TASK-1 to
  TASK-6 (`03:1126-1211`), and §6.3 (`03:1213-1225`);
- `01-design.md` D8's scope-results bullet (`01:534-545`).

The round-91 text came from Codex's session log, where Codex printed
`03:1145-1185` as it stood. A word diff of that paragraph against the
current one (`03:1148-1171`) shows only these changes:

- `` `spawn_isolated`, `` became `` `spawn_isolated` (directly or through
  further isolated calls), ``;
- "at the same point would, as if no `shield` were in force:" became
  "at the same point and moment would if no `shield` were in force
  there.", followed by the new sentence "Ignoring `shield` only lets
  that checkpoint see the enclosing scopes' cancellations; every scope
  keeps its actual first cause, so a nested scope that its own cause
  cancelled first still marks nothing." and a new "A" that begins the
  example sentence.

So the rewrap changed no other words. The §6.1 hunk (`03:1105-1123`) is
the same as the text Codex reviewed, and so is the §6.4 hunk.

## Earlier findings

- **Round 91 B1, closed.** `bg` spawns `H`, then `S`. `H`, its bit
  clear, enters `shield` and opens nested scope `s`. `S` fails and
  cancels `bg`. The shield keeps that cancellation from reaching `s`
  (TASK-6). Then `s`'s child `K` fails independently, so `s`'s one
  actual cause is `K`'s failure. `H`, as `s`'s body, calls `K.join()?`.
  A checkpoint there, ignoring the shield, could see `bg`'s
  cancellation, but `s` keeps its actual first cause, `K`, which arose
  inside `s` and did not reach it from `bg`. "a nested scope that its
  own cause cancelled first still marks nothing" (`03:1164-1166`) says
  so in so many words. `H`'s bit stays clear, `s` reports `K`, `H`
  returns `K`'s error as an independent failure, and `bg` reports `K`
  by spawn order. "and moment" (`03:1162`) rules out the historical
  reading: the checkpoint is placed at the join's real time, with the
  real causes.
- **Round 91 N1, closed.** "(directly or through further isolated
  calls)" (`03:1151-1152`) settles `H → I1 → I2`. `I2` inherits `bg`'s
  cancellation through `I1` (TASK-1, `03:1131-1133`). A checkpoint in
  `I2` returns that cancellation, which `S`'s failure caused, so `H`'s
  bit is set. A deeper isolated task run by a child of a scope that `I1`
  opens is still covered as "inside" that scope (TASK-3), as round 90
  found.

## Blocking findings

None.

## Non-blocking notes

### N1. Round 89 N2's shielded cleanup case now marks nothing

Round 89 N2 (`round89-consistency.md:147-158`) chose the replay reading
for TASK-6's bounded cleanup. `H` runs `tasks.shield(|| tasks.scope(|s|
{ s.set_deadline(t); .. c.join()? .. }))`. `S`'s failure cancels `bg`
during the cleanup, and `s`'s own deadline later cancels `s`. Round 90
recorded that the join sets the bit "as if no `shield` were in force",
since `s` "would have been cancelled through `bg` first"
(`round90-consistency.md:137-141`). Under the revised text, `s`'s actual
first cause is its own deadline, so the join marks nothing. `s` returns
`DeadlineExceeded` (TASK-4), and if `H`'s bit is still clear that error
counts against `S` by spawn order.

This reversal is the right call, and the text decides it. The case has
the same shape as Codex's B1 (a shielded nested scope, cancelled later
by its own cause), so the two cases have to agree. A cleanup that times
out is a real failure of `H`, as round 90 N1 accepted for a request
deadline. As round 89 noted, `H` usually already has its bit set from
the checkpoint that sent it into cleanup, and the bit stays set, so few
programs see the difference. The parts of the shield wording that
matter still hold:

- **A direct join under a user's `shield`.** A child that calls
  `x.join()?` inside its own shield, after a sibling's failure cancelled
  `bg`, on a sibling that echoed: a checkpoint at that point, ignoring
  the shield, returns `bg`'s cancellation, which the sibling caused, so
  the bit is set and `bg` reports the real failure (round 82, under
  `shield`).
- **A shielded nested scope that is not itself cancelled.** A
  checkpoint in it sees the enclosing cancellation once the shield is
  ignored, as round 87 N1's case does without a shield, so the bit is
  set.

The one leftover is a narrow variant. Inside a shielded `s` whose own
deadline has passed, `H` joins a sibling from `bg` that echoed. The join
marks nothing, so `H`'s `Cancelled` error can outrank `S`'s by spawn
order. The unshielded case where the deadline comes first behaves the
same way, and round 90 N1 accepted that. Fixing the variant would take a
per-join-target rule, and rounds 88 to 91 kept the rule as "what a
checkpoint would see". Nothing needs to change. The revision comment on
fx-0ai already records that replay was rejected.

## Checked and consistent

- **Shielded scopes are not cancelled by enclosing scopes.** B1's
  outcome depends on `s` not having been cancelled through `bg` while
  the cancellation was merely hidden. TASK-6 says so only for
  operations, but TASK-3 forces the same model. If `s` were cancelled,
  every checkpoint inside it, in `K` and its siblings too, would return
  the error, and a shielded scope with its own deadline could not do
  bounded cleanup. "Ignoring `shield` only lets that checkpoint see the
  enclosing scopes' cancellations" (`03:1163-1164`) reads the same way.
- **Unchanged cases.** Round 82's case, round 87 B1 (`r`'s own
  deadline, uncancelled `bg`), round 87 N1, round 88 B1(a) and (b), and
  round 90 N1's nested join all involve no shield or no replay, so
  they come out as round 90 traced them. fx-sz7's "when any checkpoint
  would" (`03:1083-1084`) and D8's echo definition (`01:536-538`) still
  agree.
- **Prose.** The added sentence is plain and in the chapter's style.
  `git diff --check` is clean and no added line is over 80 columns.

## Out of scope

- Round 91 N2 (D8's "suspend only by using a capability" against 03 §0)
  is still open, as in rounds 80 to 91.
