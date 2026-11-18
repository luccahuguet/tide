# Case study: discovery races a host's exit (Eon)

**Tide's admission would not have rejected this bug, and Tide does not find it
unaided.** The buggy discovery is ordinary Tide: two file operations through
the `Fs` it is handed. What catches the bug is a `#[sim_test]`, written to
state the property, that runs discovery while the run's host exits. On 13 of
100 seeds the seed puts the host's whole exit between discovery's read and
its check, and discovery fails; the runner reruns each failing seed and
checks that its trace is identical. The fixed discovery passes on all 100.
Eon's fix already landed with its own deterministic test, so Tide does not do
what Eon could not. Its gain is narrower: the seed finds the interleaving,
where Eon's test needed a hook written into discovery to place the exit by
hand.

**Feature:** the simulator's seeded scheduling of tasks and file operations,
and the rerun and replay of a failing seed (`design/01-design.md` D15 and
D17; reference §6.8), over the one file system a program has, the `Fs` it is
handed (D7). Survey and ranking: `research/33-case-study-candidates.md` §3
and §5, rank 3.

## The bug

Eon is a terminal workspace (github.com/Yazelix/eon). Each terminal runs in an
Orbit host, which publishes a record for its run (Live while it serves, a
tombstone once it has exited) and serves on a Unix socket. Eon's supervisor
finds runs it did not start itself by discovery: it reads each record and, for
a Live one, checks the host and then stats the socket.

- **eon-8ehhn** (closed). A host can exit between discovery's record read and
  its socket check. Orbit's exit first publishes the tombstone and then
  removes the socket (Orbit's contract ORB-C13). Discovery, holding a Live
  record, then finds no socket and fails with "cannot inspect terminal
  endpoint". The error fails whatever needed discovery: a Stop, a final or
  Stack close, or a replacement supervisor's start. The bead was found while
  diagnosing a flaky test (eon-j7zfg).
- **The fix**, Eon commit `27b39095` ("Settle a terminal whose own host
  removed its socket as ended in discovery (eon-8ehhn)", 2026-11-10, on Eon's
  development branch `edge`). After a failed socket check, discovery rereads
  the record once. If it is now the same run's tombstone, the run has ended
  and discovery reconciles it; if it is the same Live record and the host is
  dead, discovery retires it. Anything else keeps the first error. The commit
  message says:

  > No retry, sleep or new timeout; ORB-C13's order makes one read enough.

  The bead's acceptance asked for "a deterministic test that removes the
  socket between the record read and the stat (no sleeps)". The fix adds a
  hook to discovery, called after the host check and before the socket
  check, and its tests remove the socket from inside that hook.

The commit is on Eon's public `edge` branch. A GitHub issue link follows when
the issue is filed in Eon's repository, after Tide's launch.

## The files

| File | What it is | Declared outcome |
| --- | --- | --- |
| `buggy.rs` | Discovery before `27b39095`: a Live record whose endpoint is missing is an error. | `discovery_settles_a_run_whose_host_exits` fails on 13 of 100 seeds, and its output reports discovery's "cannot inspect terminal endpoint" error |
| `fixed.rs` | Discovery after `27b39095`: on a missing endpoint it rereads the record once, and a tombstone means the run ended. | its tests pass: the same race test on 100 seeds, and `a_missing_endpoint_without_an_exit_still_fails` |
| `out_of_order.rs` | `fixed.rs`'s discovery against a host that breaks ORB-C13, removing its endpoint before it publishes its tombstone. | the race test fails on 11 of 100 seeds: the single reread is not enough |

The race test is the same in all three. The simulated file system starts with
the run serving: its endpoint exists and its record says `live`. Two tasks
then run in one scope: the host's exit, and discovery. Every file operation
is a scheduling point, so the seed chooses how the host's two steps
interleave with discovery's two or three. After both tasks join, the test
checks that the record is a tombstone and that discovery did not fail. A run
whose host exited cleanly must come out Live (discovery finished first) or
Ended, never an error.

The property is a test assertion, not a contract on `discover`. An
`#[ensures]` on `discover` is checked when the call returns, and cannot see
what the host does after it (reference §10.1).

`fixed.rs` adds the fix's other promise: a Live record with no endpoint and
no exit still fails. The fix does not settle every missing socket as an end.

`out_of_order.rs` shows what the fix rests on. With the host's steps swapped,
the seed finds an interleaving where the reread still sees `live`, and
discovery fails as before. It shows that ORB-C13's order is load-bearing; it
does not show that Orbit keeps it.

## What the rewrite models, and what it does not

It models the order of four file operations: the host writes the record and
removes the endpoint, and discovery reads the record and checks the endpoint,
rereading the record once in the fix. The record and the endpoint are files
on the simulated `Fs`; the endpoint stands in for the Unix socket.

It does not model:

- The socket itself. Discovery's check is a `metadata` call on a file, not a
  stat of a Unix socket, and no connection is made.
- Two endpoints. Eon's host has two, a presentation socket and a management
  socket, and `reconcile_tombstone` waits for both to go. The model has one.
- A missing record on the reread. Eon's reread treats no record as "not
  ended" and keeps the first endpoint error; the model's reread would return
  its read error instead. The model never removes the record, so this path
  is never reached.
- Run identity. Eon's reread accepts only a tombstone of the same run, or the
  same Live record at the same inode. The model has one run, so any
  tombstone is that run's.
- The host liveness check, and the fix's second branch: a Live record whose
  host is dead (`retire_dead_run`). That needs a host that can crash, and
  the simulator's crash faults are planned.
- Reconciliation itself: reaping the host and removing the record.
- The races the commit leaves open. Its message says: "Record-read,
  listing-to-lease and failed-host races remain". In the model a record read
  is atomic, so the first of these, a record replaced while it is being
  read, cannot happen.
- That Orbit keeps ORB-C13's order. The model assumes it, and
  `out_of_order.rs` shows the fix needs it.

So the claim is about the order of the steps, not about Eon's code. A
hand-written model shows only what it models (fx-dett): the test fails
because the model's discovery checks after a read that may be stale, and
Eon's did too.

## What Tide would and would not change in Eon

Would not:

- reject the buggy discovery. Reading a record and then checking a file is
  ordinary Tide;
- state the property. The test asserts "a run whose host exits cleanly is
  never an error", and someone has to think of it. Without that test, or
  with a test that never runs discovery against an exit, nothing fails;
- choose the fix, or show that one reread is enough in Orbit's real code.
  The simulator explores the interleavings of the model it is given;
- make Eon's own test unnecessary. Eon already had a deterministic test of
  this race.

Would:

- **Find the interleaving by seed.** Discovery needs no hook. The test
  starts the exit and discovery together, and the seed schedules every file
  operation; 13 of 100 seeds hit the window. Eon's test had to add
  `before_endpoint_check` to discovery and remove the socket inside it,
  which means knowing the window in advance. Eon found this one through a
  flaky test.
- **Rerun the failing seed exactly.** The runner reruns each failing seed
  and fails it if the trace differs, and prints the rerun command for that
  seed. Through the driver (`tide-admit --tide-run-test`), a run records a
  manifest that `--replay` checks against.
- **Test the fix's assumption.** Swapping the host's two steps is a
  two-line change, and the same test then fails on some seed.

## What the Rust version would need

Rust can test this race, and Eon's does, with a hook in discovery. To let a
scheduler find it instead, a Rust program needs its file operations behind a
trait with a simulated implementation, and a scheduler that controls the
threads, such as loom or shuttle. Those two schedule threads but simulate no
file system, so the project writes and keeps the simulated one, or adopts a
simulation framework that provides it. Tide's std gives every program that
file system, because `Fs` is its only file access, and every operation on it
is a scheduling point.

## License and source

Eon is the Tide owner's project, licensed under Apache-2.0, as Tide is. These
files are written fresh for Tide and copy no Eon code. The quoted lines come
from Eon's commit `27b39095` and from bead eon-8ehhn, under Eon's license.
Every fact about the bug comes from bead eon-8ehhn, read with `br --no-db`,
and from Eon's commit `27b39095` (its message, its `CHANGELOG.md` entry and
its change to `runtime/crates/eon-runtime/src/sessions.rs`), read with
`git show`.

## Run it

`cargo test --workspace` checks every declared outcome, through the examples
runner in `crates/tide-admit/tests/examples.rs`; to run only the examples:

```sh
cargo test -p tide-admit --test examples
```
