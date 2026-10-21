# Tide adversarial/consistency review, round 77: sink order fixes (fx-kdj)

## Verdict

**approve**: no blocking findings and three non-blocking notes.

B1 and N1 to N4 of round 76 are closed. The extended SINK-4, the new
SINK-5 sentence, the restored `Stdio` conversions, and companion 20 agree
with SINK-1 to SINK-10, 01 D7, core 6.5, §1.1, §8, and §9. The notes are
wording: SINK-5's "lossless" next to the new discard sentence, plus
writer failures the sentence still leaves out (N1); nested joins in the
N2 sentence (N2); and the companion's `Stdio` and "Output order" bullets
(N3).

## What was reviewed

The uncommitted working tree against HEAD `494246d`: `git diff --
design/`, SHA-256
`d8bfec42d6ec3f4d6faf4d1893fbff2fcbde4564ebbc17dd50f0c1106e974df9`
(03 and `reference-companion.md`; `reference.md` unchanged). I
recomputed the hash. `git diff --check -- design/` passes. I did not run
`scripts/count-tokens.sh`, as instructed.

I read:

- `AGENTS.md`, `GOVERNANCE.md`, fx-kdj and its decision comment, and
  round 76;
- the whole diff;
- in 03: §0's Task term (`03:60-62`), the `Proc` row (`03:84`), the
  `Stdio` note (`03:168-177`), §2.4 SINK-1 to SINK-10 (`03:367-436`), §8.1
  and PAR-4 (`03:1329-1351`), PAR-1 and PAR-2 (`03:1355-1370`), and §8.4
  (`03:1413-1425`);
- 01 D7's sink bullet (`01:441-450`);
- core 6.5 (`reference.md:444-462`);
- companion 20 (`reference-companion.md:525-574`);
- a grep of `design/`, `trials/`, and `README.md` for `lossless`, "closed
  pipe", "writer thread", `io::stdout()`, `io::stderr()`, and
  `From<io::St`.

## Round 76 items

| Item | Status | Where |
| --- | --- | --- |
| B1. The wait omits `dbg!` | Closed | `03:404-405` ("every sink record ... or dropped it where SINK-5 allows"); companion lists `dbg!` and the drop caveat (`reference-companion.md:562-565`) |
| N1. Conditional wait; which `spawn` | Closed | `03:399-404`: "when a `Command`'s `spawn`, `status`, or `output` starts such a child"; the condition names `inherit()`, the defaults, and both conversions |
| N2. Records in another task's branch buffers | Closed, with N2 below | `03:406-407` |
| N3. Wait's end when stderr cannot be written | Closed for stderr; see N1 below | `03:416` |
| N4. Decisions on fx-kdj | Closed | fx-kdj comment records both decisions, the reversal of fx-obr, the `to_file` cost, and two rejections with reasons |
| Simulation clause (round 76 aside) | Declined, with reason | fx-kdj comment: `sim.world.proc` output is out of the bead's scope. I accept that. |

## Blocking findings

None.

## Non-blocking notes

### N1. "Lossless" next to the discard sentence, and other write failures

SINK-5 opens "`out` and `err` are lossless" (`03:411`) and now says "A
record that cannot be written to standard error is discarded"
(`03:416`). `err` goes to standard error by default (SINK-2,
`03:385`), so an `err` record can now be lost. The colon defines
"lossless" as backpressure instead of overload dropping, which is the
contrast with `log` and `trace` (`03:417-418`). So the meaning can be
recovered, but an absolute word in a rule now has an exception two
sentences later.

The new sentence also covers only standard error. SINK-4's wait ends
when every earlier record is "written ... or dropped ... where SINK-5
allows" (`03:404-405`). Some writer failures fall outside both of
SINK-5's cases: stdout that fails with something other than a closed
pipe (`ENOSPC` or `EIO` on a redirected file, `EBADF` on a closed
descriptor 1), or a `root.log.to_file` destination that fails. SINK-5
says nothing about these, so whether the wait ends depends on unstated
writer behavior. fx-kdj's comment says "so the wait always ends", which
claims more than the text guarantees. The gap predates fx-kdj except for
the wait that now depends on it. Round 76 N3 asked only about stderr, so
this is the rest of that point, not a reopening.

**Fix** (03 SINK-5, `03:411-416`): "`out` and `err` are never dropped
under load: when the process-wide buffer ... drains it. ... If stdout is
a closed pipe, the process exits quietly with status 141, ... relies on.
Any other record that cannot be written to its destination is
discarded." This keeps SINK-1's always-`Ok` handles and covers stdout,
stderr, and files with one rule. The fx-kdj comment then becomes true as
written.

### N2. Nested joins in "when its join appends them"

`03:406-407` says "A branch's records count as emitted when its join
appends them." In a nested region, an inner branch's join appends its
buffer to the enclosing branch's buffer (§8.4, `03:1415-1416`), not to
the task's output. Read literally, the inner records count as emitted
while they are still held, so another task's `status()` cannot have
waited for them. They can also still be discarded afterwards if an
outer branch to their left panics (PAR-2, `03:1365-1366`). The intended
reading, where records count once they leave the whole parallel region,
is clear from context.

**Fix** (03 SINK-4): "A branch's records count as emitted when the
outermost join appends them to its task's output."

### N3. Companion 20's two bullets

- The `Stdio` bullet says "`io::stdout()` and `io::stderr()` name the
  process's descriptors, as `inherit()` does"
  (`reference-companion.md:559-560`). That is true of the values used
  as a `Stdio`. As sinks they write through `root.log`'s destination
  (core 6.5, `reference.md:446-449`; 03 `03:170-171` already says
  "whatever destination `root.log` gives the sinks"). An agent that
  loads only the reference pair could conclude that `eprintln!` also
  bypasses `root.log.to_file`.
- The "Output order" bullet's condition, "whose stdout or stderr is
  inherited" (`reference-companion.md:562`), is narrower than SINK-4's
  "the process's own". The `.stdout(io::stderr())` case is covered only
  if the reader carries over "as `inherit()` does" from the bullet
  above.

**Fix** (`reference-companion.md:558-565`): "... sends a child's output
to your standard error: as a `Stdio`, `io::stdout()` and `io::stderr()`
name descriptors 1 and 2, as `inherit()` does, even when `root.log`
sends the sinks elsewhere. ..." and "**Output order.** Starting a child
whose stdout or stderr is the process's own (inherited, or
`io::stdout()` or `io::stderr()`) first writes out ...".

## Checked and consistent

- **SINK-1 and 6.5.** The wait belongs to `Command`'s methods, not to a
  sink, so "Sinks never suspend" (`03:380`) and core 6.5's "never
  suspend" (`reference.md:450`) still hold. `spawn` is not in the `Proc`
  row's "Suspends" column (`03:84`). The wait blocks an OS thread as
  SINK-5's high-water blocking does, so it is not a checkpoint.
- **SINK-2 and the drop clause.** "Every sink record" covers `out`,
  `err`, `dbg`, `log`, and `trace`, whatever destination they have.
  Waiting on records bound for a file is unnecessary but harmless. "Or
  dropped it where SINK-5 allows" covers both overload drops and the new
  stderr discard.
- **D7, 6.5, §8.** "Inside `par` their output is buffered and emitted
  in sequential order" (`01:444-445`) and "Order is program order"
  (`reference.md:455-456`) agree with SINK-4 and with the N2 sentence.
  "A parallel branch cannot start a child" follows from §8.1's `Pure`
  closures and PAR-4 (`03:1335`, `03:1349-1351`).
- **§9 and SINK-6.** "The simulator needs none, since records enter its
  trace when emitted" is consistent with SINK-6 (`03:419-420`) and with
  §8.4's sequential branches under simulation (`03:1424-1425`).
- **§1.1 and `Stdio`.** The note's "whatever destination `root.log`
  gives the sinks" (`03:170-171`) states the cost recorded on fx-kdj. The
  claim that "every other conversion into a `Stdio` starts from a handle"
  is still exact. The SINK-4 condition names the same three routes to
  descriptors 1 and 2 that §1.1 allows. A `File` opened on `/dev/stdout`
  would also reach descriptor 1 without the wait. SINK-4 claims nothing
  for that case, so this is not an overclaim.
- **Rust parity.** `println!` (line-buffered with a trailing newline),
  `eprintln!`, and `dbg!` (unbuffered stderr) all come before a child's
  output in Rust, so the companion's "as in Rust" holds for the forms
  that bullet lists.
- **Stale text.** Nothing in `design/`, `trials/`, or `README.md` still
  says the sink conversions are gone or that the wait is unconditional.
- **Core budget.** `reference.md` is unchanged.

## Prose and wrapping

Every added line is at most 76 columns, and `git diff --check` passes.
The `03:395-396` re-wrap noted in round 76 is still there. It is harmless.
