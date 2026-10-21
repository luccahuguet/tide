# Tide adversarial/consistency review, round 78: sink order wording (fx-kdj)

## Verdict

**approve**: no blocking findings and two non-blocking notes.

Round 77's N1 to N3 are closed. The text-only fixes agree with SINK-1,
SINK-2, PAR-2, §8.4, core 6.5, and the `Stdio` note in §1.1. No added line
runs past 80 columns. The notes are wording: the antecedent of "Any other
record" in SINK-5 (N1), and the fx-kdj decision comment, which still
quotes the round 77 wording (N2).

## What was reviewed

The uncommitted working tree against HEAD `494246d`: `git diff --
design/`, SHA-256
`98ebddf8fbb9c8bce4a5ec0814f94c8a866d841eee21c057cb358840dab939f0`
(03 and `reference-companion.md`; `reference.md` unchanged). I
recomputed the hash. `git diff --check -- design/` passes, and no added
line is longer than 80 columns.

I read:

- `AGENTS.md`, fx-kdj and its decision comment, and round 77;
- the whole diff;
- in 03: the `Stdio` note (`03:168-178`), §2.4's sink table and SINK-1
  to SINK-6 (`03:370-421`), PAR-2 (`03:1364`), and §8.4
  (`03:1414-1425`);
- core 6.5 (`reference.md:444-462`);
- companion 20's `Stdio` and "Output order" bullets
  (`reference-companion.md:555-567`);
- a grep of `design/`, `trials/`, and `README.md` for `lossless`, "never
  dropped", and "cannot be written". The remaining `lossless` hits are
  all about `as` conversions.

## Round 77 items

| Item | Status | Where |
| --- | --- | --- |
| N1. "Lossless" next to the discard sentence; other write failures | Closed | `03:412-413` ("never dropped under load"); `03:417-418` ("Any other record that cannot be written to its destination is discarded") covers stdout, stderr, and `root.log.to_file` destinations, so SINK-4's wait always ends |
| N2. Nested joins | Closed | `03:406-408`: "when the outermost join appends them to its task's output" |
| N3. Companion 20's two bullets | Closed | `reference-companion.md:558-562` scopes the claim to "used as a `Stdio`" and names `root.log`; `reference-companion.md:563-564` states the condition as "the process's own (inherited, or `io::stdout()` or `io::stderr()`)" |

## Blocking findings

None.

## Non-blocking notes

### N1. The antecedent of "Any other record"

SINK-5's new sentence follows "If stdout is a closed pipe, the process
exits quietly with status 141" (`03:415-417`). That sentence speaks of
stdout, not of records, so "other" has no record to contrast with. The
paragraph opens with "`out` and `err`", so a reader could take "any other
record" to mean records other than `out` and `err`, which would leave an
`out` record that fails with `ENOSPC` uncovered again. The intended
reading (every failed write except the closed stdout pipe) is the only
one that fits SINK-4's "or dropped it where SINK-5 allows", so this is a
nit.

**Possible fix** (`03:417-418`): "Apart from that case, a record that
cannot be written to its destination is discarded."

### N2. The fx-kdj decision comment quotes the old wording

The comment says "a record that cannot be written to stderr is
discarded" and "branch records count once their join appends them". The
text now says "its destination" and "the outermost join". The design
documents own the decision, so nothing contradicts them, but a short
appended comment noting the round 77 rewording would keep the bead
accurate for the next session. Comments are append-only, so do not edit
the existing one.

## Checked and consistent

- **SINK-1 and 6.5.** Discarding a record that cannot be written keeps
  "The `impl Write` handles from `stdout()` and `stderr()` always return
  `Ok`" (`03:378-379`) and core 6.5's "always return `Ok`"
  (`reference.md:447-448`) true: the failure never reaches the caller.
- **SINK-2.** "Its destination" covers every destination SINK-2 allows,
  including `root.log.to_file(file)`. The closed-pipe exit remains
  specific to stdout.
- **SINK-5's two cases.** "Never dropped under load" contrasts with
  `log` and `trace`, which "drop records under overload"
  (`03:418-419`). A write failure is a different cause, so the absolute
  word no longer has an exception.
- **PAR-2 and §8.4.** §8.4's "the join appends the buffers"
  (`03:1416-1417`) is true of each join, nested or not. SINK-4's
  "outermost join appends them to its task's output" picks the join
  whose append makes records visible to the writer. Records discarded by
  an outer join under PAR-2 are therefore never counted as emitted.
- **Companion against 03 and 6.5.** "Used as a `Stdio`" limits the
  descriptor claim to the conversion, which matches 03's "whatever
  destination `root.log` gives the sinks" (`03:172`) and leaves 6.5's
  sink description intact. The companion's drop caveat names only
  overload; a failed write also discards a record, but the bullet
  summarizes and points to SINK-4, which carries the full rule.

## Prose and wrapping

Every added line is at most 80 columns, and `git diff --check` passes.
