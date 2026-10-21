# Tide adversarial/consistency review, round 81: limited spawn and cancellation (fx-sz7)

## Verdict

**revise**: one blocking finding and four non-blocking notes.

Round 80's B1, N1, N3, N4, and N5 are closed, and N2 stays unapplied for
the stated budget reason (core at 24,993 of 25,000 tokens). The decision
still holds together with §0, §1.1, §1.2, TASK-2 to TASK-5, JOIN-1,
§9.3, D8, 02 §8.2 and §8.3, and core 7.1 and 7.2. But the fix for round
80's B1 made the count one per limited `Tasks`, shared by every scope
from it. That makes the paragraph's other liveness claim, that a
cancelled scope's wait ends as soon as TASK-5 allows, false whenever the
slots are held by children that the cancellation does not reach. The
rejection of option (a) does not cover that case (B1).

## What was reviewed

The uncommitted working tree against HEAD `db1bebc`: `git diff --
design/`, SHA-256
`fd23ab3402e13453755d7ce0af9db8360e750e0ea86a1aa5bc0e9900bdc577c4`
(03 and `reference.md`). I recomputed the hash. `git diff --check`
passes. The only added lines past 80 columns are the two table rows
(`03:86`, `03:211`), as for every row in those tables.
`scripts/count-tokens.sh design/reference.md` gives 24,993.

I read:

- `AGENTS.md`, fx-sz7 with both decision comments, and
  `reviews/round80-consistency.md`;
- the whole diff;
- in 03: §0 Terms (`03:31-63`), §1.1's table and `Receivers` note
  (`03:67-203`), §1.2 (`03:205-223`), §1.3 with its deadlock sentence
  (`03:225-263`), §6.1 to §6.6 (`03:1043-1292`), §7.1 (`03:1296-1316`),
  PAR-4 (`03:1370-1382`), and §9.3 (`03:1491-1513`);
- `01-design.md` D8 (`01:509-564`) and its `Tasks` lines;
- `02-memory.md` §8.2 and §8.3 (`02:657-734`);
- core 6.8's scheduling bullets (`reference.md:531-541`), 7.1, 7.2, and
  7.3's `Tasks::limit` pointer (`reference.md:574-699`), and section 8's
  pointer (`reference.md:792-794`);
- companion 14.3 (`reference-companion.md:57`, `88-99`) and 15.2
  (`reference-companion.md:184-245`);
- a grep of `design/` for `limit`, `spawn`, `checkpoint`, `shielded`,
  `deadlock`, "never interrupted", and "cannot be cancelled".

## Round 80 findings

- **B1, closed.** 03 now states the counting domain, "all scopes from
  that `Tasks` share one count" (`03:1077`), which matches companion
  14.3's `Semaphore` mapping and core 7.1's "scopes from that `Tasks`"
  (`reference.md:635`). It states the hold-and-wait deadlock with both
  routes (`03:1094-1096`), says cancellation and deadlines do not break
  it (`03:1097`), weighs it against option (a) (`03:1098-1100`), and
  gives the remedy (`03:1100-1101`). Each claim holds: the slot holders
  all wait in a non-checkpoint `spawn`, so nothing frees a slot; option
  (a) would free one only once a holder's scope, or an ancestor
  (TASK-2's nested-scope cancellation), is cancelled.
- **N1, closed.** `03:1080-1083` gives the reason against a fallible
  `spawn`: "a `?` on every spawn for a wait that only limited scopes
  have".
- **N3, closed.** `03:1101-1103`: the body "breaks out", which sidesteps
  the `E: From<Cancelled>` cost of `?`, and "the children it has spawned
  keep their results" makes plain that unspawned items get none.
  `checkpoint` is a `&self` method on `Tasks` (`03:1056`), and the body
  is cancelled with its scope (TASK-2, `03:1116-1117`), so the check
  works from the body.
- **N4, closed.** `03:1088-1090`: "would exceed the limit" and "lose the
  values such children return".
- **N5, closed.** TASK-5 is reflowed (`03:1164-1168`); "which" still
  attaches to `File::close`.
- **N2, not applied, accepted.** Core 7.1 states the case
  (`reference.md:636`); 7.2's list (`reference.md:665-667`) does not
  contradict it.

## Blocking findings

### B1. A shared count lets a cancelled scope's wait outlast its deadline indefinitely, and the text says otherwise

`03:1092-1093`: "The wait ends when a live child finishes, and in a
cancelled scope each live child stops at its next checkpoint, as late as
TASK-5 allows." With one count per limited `Tasks` (`03:1077`), the live
children holding the slots need not be in the cancelled scope or any
scope nested in it. A scope's cancellation reaches only its body, its
children, and its nested scopes (TASK-2, `03:1116-1117`; TASK-3,
`03:1143-1145`), so children of a sibling scope from the same `Tasks`
keep running.

The pattern is ordinary, not a program bug: a server builds
`let lim = root.tasks.limit(64);` as a global cap and passes `&lim` to
every request handler, each of which opens `lim.scope(..)` with its own
deadline. Once long-running requests hold all 64 slots, a request whose
deadline has passed sits in `spawn` until one of them finishes, which may
be never. Nothing reports it:

- it is not a deadlock, since the other requests make progress, so §9.3's
  report ("If no task can ever run again", `03:1501-1504`) never fires;
- TASK-5's watchdog flags a task that "runs" 100 ms without a checkpoint
  (`03:1169-1171`), and a task waiting for a slot is not running.

So the analogy "shielded, like a join (JOIN-2)" (`03:1079`) breaks at the
point that made shielding joins safe: "A join waits no longer than the
child, and in a cancelled scope the child is itself cancelled"
(`03:1188-1189`). Every other wait that cancellation cannot interrupt is
bounded by work it reaches or by finite work: `Shared::with` closures
cannot suspend (`03:1275-1277`), and scope exit waits for the scope's own
children (JOIN-1). The limited `spawn` would be the first non-checkpoint
wait bounded by tasks outside the cancellation. It also weakens the
mapped Rust idiom. Companion 14.3 maps `Semaphore` to `tasks.limit`
(`reference-companion.md:95`) and `tokio::time::timeout(d, f)` to a scope
deadline (`reference-companion.md:94`), but in tokio a timeout around
`sem.acquire()` does bound the wait.

The decision may still be right: per-item values matter, and option (a)
is the only way to bound this wait. But fx-sz7's second comment and
`03:1098-1100` reject (a) only on the hold-and-wait deadlock ("would
break it only when something cancels the scope"), which this case does
not involve, and the "as late as TASK-5 allows" sentence asserts a bound
that does not hold.

**Possible fix** (03 §6.1 only):

- Limit the sentence to the true case, for example: "In a cancelled
  scope, a slot held by one of its own children frees at that child's
  next checkpoint, as late as TASK-5 allows. Children of other scopes
  from the same `Tasks` are not cancelled, so the wait lasts as long as
  they run."
- Give the reason for keeping (a) rejected in this case too, or reverse
  it. If kept, add the remedy: a scope that must meet its deadline takes
  a limit that no work outside its cancellation shares, as companion
  15.2 does with its own `tasks.limit(4)`.

## Non-blocking notes

### N1. "The simulator reports it" holds only when no other task can run

`03:1097-1098` says the simulator reports the hold-and-wait deadlock.
§9.3 reports only a global deadlock (`03:1501-1504`). If any task outside
the stuck pool keeps running, such as a heartbeat loop or an accept loop
with a timer, no report comes until that task also blocks. §6.5 says
"global deadlock" for the channel case (`03:1277-1278`). Suggest
"reports it as a global deadlock once nothing else can run (§9.3)".

### N2. "Give nested work its own `Tasks`" depends on how `limit` chains, which 03 does not say

`03:1100-1101`. No text says whether `lim.limit(m)` on an already limited
`Tasks` also counts against `lim`'s count. Attenuation implies it must;
otherwise `lim.limit(usize::MAX)` escapes the bound (`reference.md:588`,
"attenuation"). If it does, nested work can get "its own `Tasks` or no
limit" only from a `Tasks` above the limit, such as `root.tasks`. A child
that received only the limited `&Tasks`, like the core's `fetch_both(tasks:
&Tasks, ..)` (`reference.md:647`), cannot follow the advice. One clause
fixes it, for example: "`limit` on a limited `Tasks` also counts against
the outer limit, so nested work takes its `Tasks` from above the limit."

### N3. The core states the cancelled-scope detail but not the deadlock

The agent-loadable core (`reference.md:634-638`) tells agents to use
`limit` for N workers over a list (`reference.md:697`) and lets a child
capture the `Scope`, yet does not warn that a child spawning under the
same limit can hang. That trap is more likely in agent code than a
cancelled-scope subtlety whose outcome already matches an unlimited
scope. Under the budget, a swap at about equal cost would do it: replace
"Even in a cancelled scope, the child then starts and sees the
cancellation at its first checkpoint." with "A child that spawns under
the same limit can deadlock it." Leaving it as it is is also defensible.

### N4. Wording

- `03:1089-1090`: "such children" still reaches back over "Starting the
  child at once would exceed the limit." to the child "that turns
  cancellation into a value" (`03:1086-1087`). Swapping the order of the
  two rejection sentences would put it next to its referent.
- `03:1092`: "a live child" means any child counted by that `Tasks`, as
  `03:1077-1078` defines it; with B1's fix the scope of each phrase
  should be explicit.

## Checked and consistent

- **§0, §1.1, §1.2, and core 7.1.** §0 lists the slot wait among
  non-checkpoints (`03:58-60`); §1.2's `Scope` row says "`spawn` under
  `Tasks::limit`, shielded (§6.1)" (`03:211`); §1.1's attenuation cell
  points to §6.1 (`03:86`); core 7.1 says "not a checkpoint, never
  interrupted" (`reference.md:636`). All agree.
- **TASK-2 to TASK-5.** A child started in a cancelled scope fails or
  turns cancellation into a value at its first checkpoint, exactly as a
  child spawned after cancellation in an unlimited scope, so "TASK-2 and
  TASK-4 decide the outcome as for an unlimited scope" (`03:1085-1086`)
  holds, including echo bits and `DeadlineExceeded`. TASK-5 now lists
  the slot wait (`03:1166`).
- **Rejected option (a).** Under JOIN-2 a skipped child's `join` returns
  `Err(TaskError::Cancelled.into())`, so the body's `x.join()?` fails
  with no child error for rule 2 to report, and 15.2's per-URL
  `Outcome` is lost. "No child error behind it" (`03:1089`) is accurate.
- **Hold-and-wait deadlock.** Both routes are real: a child may capture
  its `Scope` (`02:694`; `reference.md:592`), and a child may open a
  scope from the same `Tasks`. "Cancellation and deadlines do not break
  it" holds because every holder waits in a non-checkpoint `spawn`.
- **D8 and §1.3.** "Production cannot deadlock where the simulator makes
  progress" (`01:547-549`, `03:256-257`) survives: the count is the same
  semaphore under every runtime, `spawn` is a scheduling point
  (`03:1505-1507`), and PAR-4 keeps a `Scope` out of parallel branches
  (`03:1376-1382`), so no pool thread waits for a slot.
- **SUSP-1.** Code that holds a `Scope` already has a non-`Pure` capture
  or parameter, so the new suspension needs no new classification
  (`03:1298-1304`).
- **02 §8.2 and §8.3.** The wait happens in the body or a child before
  the scope's join, so the loans in `σ` and the failure edge are
  unaffected (`02:683-689`, `02:717-719`).
- **Companion 15.2.** `fetch_one` maps `is_cancelled()` to `TimedOut`
  (`reference-companion.md:213`), so each URL past the first four still
  gets an outcome; 15.2 builds its own `tasks.limit(4)` per call, so B1's
  cross-scope case does not reach it when `tasks` is unlimited.
- **Grep.** No other hit in `design/` for `limit`, `spawn`,
  `checkpoint`, `shielded`, or `deadlock` contradicts the decision.

## Out of scope

- **`limit(0)`.** Under this decision every `spawn` from `limit(0)` waits
  forever and cannot be cancelled. Neither 03 nor the core says whether
  `limit(0)` panics, is rejected, or means something else. This predates
  the diff, but the decision turns it into a guaranteed hang.
- **`spawn_isolated` under `limit`.** Still unstated whether it takes a
  slot (round 80, out of scope). The shared count (`03:1077`) speaks only
  of scopes.
- **Companion 15.2's doc comment.** "Every URL gets an outcome by the
  deadline" (`reference-companion.md:195`) is slightly strong, as round
  80 noted.
- **D8's "suspend only by using a capability"** (`01:511-513`) is looser
  than 03 §0's "capability and synchronization operations"
  (`03:54-55`), as round 80 noted. The diff adds one more synchronization
  case of the same kind.
