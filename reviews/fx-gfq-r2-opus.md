Model/Effort/Date: claude-opus-5-5, low, 2026-10-31

# fx-gfq r2 Opus confirmation review: frozen checked-toolchain E1 protocol

## Verdict

request_changes, confidence 0.8. There are two blocking findings, and both
have text-only fixes. Every other round-1 blocking finding is resolved.

## Scope

- Candidate: the working tree at
  `/home/lucca/.cache/eon-agents/tide-gfq` against HEAD 1e0cbac. Its diff
  matches `r2.diff` (SHA-256 bd78c008…19ddc9a, confirmed).
- Read in full: `trials/e1-checked/protocol.md`, `trials/e1-checked/tasks.md`,
  and both round-1 reviews.
- Beads: fx-gfq, fx-ic2, and all 14 `e1-prereq` beads, plus the edges and
  status of fx-nkv.1, fx-nkv.5, fx-nkv.6, and fx-kw8.9.
- Also checked: D0 in `design/01-design.md`, the FX codes in
  `design/reference.md`, `rust-toolchain.toml`, and GOVERNANCE.md's Merge
  Rule.
- Recomputed: every checked value in A2, A3, A4, A5, A6, A7, A8, A9, A10,
  B1, B2, B3, B4, B5, B8, B10, C1, C3, C4, C5, C8, and C9, and B7's replay
  oracle.
- Ran `br dep cycles` (none found) and parsed every line of the JSONL
  (128 lines, all valid).
- Nothing in the worktree was edited.

## Round-1 findings

### Codex

1. **B7 preflight rejected its own swap.** Resolved. `tasks.md:555-557`
   exempts a source that is the target of an earlier step.
2. **No owner for FX0301 and the other FX rejections.** Resolved.
   - The new row E1-P14 (`protocol.md:256`) and bead fx-u65 own it, with an
     edge to fx-ic2.
   - Readiness item 3 (`protocol.md:291-293`) requires a negative case per
     code.
   - fx-u65's Acceptance matches item 3.
3. **B7's plan oracle checked names only.** Resolved. `tasks.md:564-568`
   now replays contents. I checked that the oracle is complete: renames
   move contents and never copy them, so when every target holds its
   source's content, no stray temp file can hold any. An empty plan for the
   3-cycle now fails.
4. **A9 had no starting inventory.** Resolved. `tasks.md:321-323` gives
   it. I recomputed case 3:
   - Sell then restock leaves `a`=1, `b`=5.
   - `Insufficient("a")` holds, since 3 < 9 (and 1 < 9 if state carries
     over).
   - `b` goes 0+1=1, then 1+4294967295 overflows `u32`, so `Overflow("b")`
     holds.
   - These results hold whether or not the steps share state.
5. **Concurrency could pass serially.** Partly resolved. A recorded,
   line-cited grader review is an acceptable mechanism for C1, C2, C4, C6,
   and C9, because a mechanical concurrency witness under real threads
   would be flaky. But the claim that C3 and C5 have checked cases a serial
   program fails is false, and the review does not cover C6's early stop.
   See Blocking 1.
6. **The sandbox left host files readable.** Resolved.
   `protocol.md:307-311` sets a read allowlist that excludes repository
   files, other trials, suites, and reference solutions.
7. **Replay was attributed to fx-nkv.6** (non-blocking). Resolved.
   `protocol.md:120-123` names fx-nkv.7 as not a prerequisite and counts
   the limitation against Tide.
8. **The contamination rule covered only writer sessions** (non-blocking).
   Resolved. `protocol.md:36-42` also covers the prerequisite beads, the
   readiness work, and the reference solutions.

### Opus

1. **The iteration measure diluted D0's 1.5x.** Resolved. §7.3 takes the
   mean over the trials that pass final; with no passing trial in an arm,
   threshold 3 is not met. That reading of "iterations to green" is
   faithful.
2. **The held-out rule started too late.** Resolved. It now runs from the
   freeze (`protocol.md:36`).
3. **An incomplete run could report "failed".** Resolved. It now says
   "passed or failed" (`protocol.md:299-300`).
4. **`Scope::cancel` and `#[pure]` had no owner.** Resolved.
   - `cancel` is in the fx-zc1 row and Scope ("unless fx-nkv.3 ships it").
     fx-zc1 depends on fx-nkv.3, so this is decided when zc1 starts.
   - `#[pure]` is in the fx-yg5 row and Scope.
5. **Beads cited row IDs that did not exist.** Resolved. The table rows
   carry E1-P1 to E1-P14, and each bead cites the matching number. I
   checked all 14 rows against their bead Scopes; each covers what its row
   lists.
6. **fx-yg5's Acceptance depended on later work.** Resolved. It is now
   phrased against the Tide arm of `tasks.md` and the §10.1 rows.
   `RandomState::from_rand` is gone.
7. **B7 contradicted its own case and had a weak replay check.** Resolved;
   see Codex 1 and 3.

Most of the round-1 Opus non-blocking items were also taken:
- C5 has a Rust lower bound.
- `read_all` is a free function.
- The §2 disclosures are complete.
- The 1.96/1.99 asymmetry is in §11.
- The shuffle and bootstrap RNG are named.
- §7.2 covers Tide's generated tests.
- The first snapshot's holes are recorded.
- Seed counts default to 20.

## Blocking

### 1. §6 calls C3 and C5 serial-proof and leaves them out of the concurrency review

Location: `protocol.md:143-147`, which says "C3, C5, and C10 have checked
cases that a serial program fails".

Only C10 holds up: case 2 deadlocks a serial server. C3 and C5 do not.

**C3** (`tasks.md:721-744`). A program that spawns no producers can pass
every checked case. It creates one `sync_channel(capacity)` per input,
pre-sends each input's first item, and then, in the calling task, receives
the smallest head and sends that input's next item. That keeps capacity,
merges correctly, and detects `Unsorted`. A program that merges the
vectors directly and mentions `sync_channel(` also passes, because a
mention satisfies the structure check (§6).

**C5** (`tasks.md:782-798`). C5 has no structure check in either arm.
Sorting the jobs by duration and sleeping through them one after another
passes:
- Case 1: sleep 10, then 20, then 70 to the limit. The outcome is
  `[Done, TimedOut, Done]` with an elapsed time of 100.
- Case 2: elapsed 6.
- Case 3: elapsed 0.

Separately, C6's "the others stop early, checking at least every 4096
bytes" is checked only by the presence of `checkpoint(`, `.cancel(`, or
`AtomicBool`. A Tide program that calls `.cancel(` while its other
searches run `str::find` with no checkpoint passes. Codex finding 5 named
cancellation as well as concurrency.

**Risk.** Trials that ignore the spec's concurrency pass final in either
arm. P(Tide) >= P(Rust) is decided on 60 trials, so two or three such
trials can flip threshold 2. The Tide writer, who is new to scopes and
channels, is the likelier one to fall back to a serial design. The
protocol's own justification for leaving C3 and C5 out is false.

**Smallest fix**, text only in §6:
- Say only C10 has a serial-failing case.
- Apply the per-trial review to C1–C6 and C9 (C10 optional).
- Have the review also fail:
  - a C3 trial that does not run one producer per input concurrently with
    the merge;
  - a C5 trial whose jobs do not all start at once;
  - a C6 trial whose searches do not check for the stop at least every
    4096 bytes.

### 2. B6's payload type is stated inside B7, which B6 writers never see

Location: `tasks.md:558`, "`PlanError` and B6's `ExportError` carry
`String` names", inside B7.

**Risk.** Each writer receives only its own task (§4.2, `tasks.md:3-7`).
- A B6 writer reads `ExportError::BadName(name)` (`tasks.md:521-522`) with
  no type, and the preface invites "any lifetimes". So `BadName(&'a str)`
  is a reasonable reading.
- If the frozen suite constructs `BadName(String)`, that writer fails
  §7.1's acceptance-suite compile. First-check would then measure a guess,
  which is exactly what the round-1 note on this ambiguity tried to remove.
- A B7 writer meanwhile reads about a task it was never given.
- Task text freezes when the first gate writer starts (§2), so this is
  cheap to fix only now.

**Smallest fix:**
- In B6, add "`BadName` and `Duplicate` carry `String`."
- In B7, change line 558 to "`PlanError`'s variants carry `String`
  names."

## Non-blocking

- **Readiness may expose slow simulated suites.**
  - C3 case 2 is 8 × 10,000 items at capacity 2, about 160k channel
    handoffs per seed. Over 20 seeds on a thread-based baton scheduler in
    debug mode, that can exceed the 10-second per-test limit.
  - C4 case 4 (1,000 orders × 20 seeds) is similar.
  - Readiness item 2 will catch this. Decide in advance whether a seed or
    size reduction counts as a precision fix (§2) or as a change to the
    expected outcome.
- **Writers are not told about the concurrency review.** The preface says
  acceptance "do[es] exactly what the task's checked cases say"
  (`tasks.md:32-33`), and nothing mentions the review that §7.2 gates on.
  The spec text does say "concurrently", and the effect is the same in
  both arms. One preface sentence would still align the writer's stopping
  rule with §7.2.
- **§7.3 has a selection effect.** Conditioning on passing trials lets an
  arm that passes only easier tasks show fewer iterations. Threshold 2
  bounds most of this. The report could add, for information, the mean
  over the tasks where both arms passed.
- **Iterations past green.** §7.3 counts every run, including runs after
  the trial first went green. "To green" could mean runs up to the first
  passing `./test`. Pick one and say so.
- **The trial order is not reproducible from the seed alone.** The
  spelling of the `(task, arm, replicate)` tuples is unpinned (task ID or
  name, replicate 0/1 or 1/2). Readiness item 6 records the order, so this
  is only a reproducibility nit.
- **Two bead edges are thin.**
  - fx-hxw's edge on fx-o8j still has no stated reason.
  - fx-iob promises "sequential under the simulator" without depending on
    fx-nkv.5 or fx-nkv.6.
- **No decisions are recorded on fx-gfq yet** (no comments). The
  Decisions rule asks for them: the concurrency-review mechanism, the
  iteration measure, and the all-FX-codes scope of fx-u65 should go on the
  bead when it closes.
- **fx-u65 covers every FX code the reference names**, including FX0901,
  FX0910, FX0911, FX0915, and FX0920, which E1 tasks cannot reach. That is
  a defensible technical choice but makes fx-ic2 later. If it proves heavy,
  item 3 could be narrowed to the codes reachable from a single-file,
  no-grant package. That narrowing is a technical correction, but it
  loosens a readiness check, so it needs the owner.

## Reasoning

**D0.** §8 states D0 exactly: F >= 0.8x, P >= Rust, and I <= 1.5x on the
30-task suite. §7.3's passing-trial mean is the literal reading of
"iterations to green". §1, §9, and §10.2 keep a pilot or an incomplete run
from reporting a verdict.

**Beads.**
- The 21 owners in §10.1 are all edges of fx-ic2, as is fx-gfq itself.
- The 14 `e1-prereq` beads cite E1-P1 to E1-P14 correctly, and each Scope
  covers its row.
- There are no cycles, and every JSONL line parses.
- fx-kw8.9 depends on fx-nkv.1, so the beads that depend only on fx-kw8.9
  still wait for the production std.

**Checked cases.** Every value I recomputed is correct, including the
reworked A9 and B7 cases.

**What remains.** Two defects, both introduced or left by the round-1
fixes:
- The concurrency resolution rests on a false claim about C3 and C5 and
  leaves out C6's cancellation.
- The String-payload fix landed in the wrong task.

Both fixes are a few sentences. Once they land, I expect a clean round.
