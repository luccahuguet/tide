# Case study: one slow close blocks every request (Eon)

**Tide's admission would not have rejected this bug.** The buggy loop is
ordinary Tide: one task, a channel, and waits on the `Clock` it is handed.
What catches the bug is a test. In a `#[sim_test]`, Venus's Inspect must be
answered under its 6 s read deadline while a tab closes. The test fails on
the buggy revision and passes on the fixed one. Rust can write the same test
(see the Rust section below). What Tide adds is that the test runs in virtual
time, is deterministic, and needs no fake clock written for it. Async Rust
gets the same from tokio's paused clock; Eon's thread-based supervisor would
need a clock it is handed.

**Feature:** structured tasks, scope deadlines and cancellation
(`design/01-design.md` D8; reference §7.1–§7.3), and the simulator
(reference §6.8). Survey and ranking: `research/33-case-study-candidates.md`
§3.2 and §5, rank 1.

## The bug

Eon is a terminal workspace (github.com/Yazelix/eon). Its supervisor answers
every control request on one loop thread. Venus, the window, reads the
workspace with an Inspect every 250 ms and gives up after 6 s.

- **eon-8wh7h** (closed). Closing a tab stopped its terminals one after
  another on that thread. Each program slow to exit cost Orbit's 500 ms stop
  grace. Eon's record 01 reproduced it: a 14-terminal close took 6992 ms
  under IO and CPU stress, and Inspect timed out at 6011 ms, Venus's exact
  failure. Fixed in Eon commit `d431fcab` ("Stop a close's terminals together
  under one shared bound (eon-8wh7h)", 2026-11-04). The commit message says:

  > A close now sends Stop to all of its live terminals through the existing
  > stop_managed_sessions batch, and reconciles the ones that ended first,
  > all under one shared five-second deadline [...]. Confirmed ends are
  > pruned in close order; only unconfirmed terminals stay.

  The bead's second slice, Eon commit `1766afd1`, makes Venus say plainly
  when Eon is slow to answer. This case study does not model it.
- **eon-pimct** (open). Each terminal start still waits on the same thread
  for the new terminal's Ready, up to the 15 s `TERMINAL_START_TIMEOUT` on
  Eon's development branch `edge` (commit `283bf76a`; eon-pimct cites it at
  `dcaef1c`). The v0.1.16 release on `main` waits up to 5 s.
  Eon has observed one Inspect waiting 3.8 s behind a start. The bead's own
  falsifier is: "hold one new Orbit's Ready publication pending: an
  independent read-only Inspect must complete before that start is released
  or times out."

Both commits are on Eon's public `main`. GitHub issue links follow when the
issues are filed in Eon's repository, after Tide's launch.

## The files

| File | What it is | Declared outcome |
| --- | --- | --- |
| `buggy.rs` | The supervisor before `d431fcab`: a close waits for each terminal's end in turn, inside the request loop. | `inspect_is_answered_while_a_tab_closes` fails: the Inspect sent 250 ms into a 14 × 500 ms close gets no answer within 6 s, on every seed |
| `fixed.rs` | The supervisor after `d431fcab`: a close stops every terminal at once in a scope whose deadline is 5 s, and keeps the terminals whose end it could not confirm. | its tests pass: the same Inspect test, and `a_close_keeps_only_the_unended` |
| `open_start.rs` | `fixed.rs` plus terminal starts as Eon runs them today: a start waits in the loop for Ready. | `inspect_is_answered_while_a_terminal_starts` fails: with the new terminal's Ready held, Inspect gets no answer within 6 s |

The test in `buggy.rs` and `fixed.rs` is the same, apart from handing
`serve` the `Tasks` the fix needs. A 14-terminal tab, each
terminal taking 500 ms to stop (Eon's reproduction), is closed while a Venus
task sends an Inspect every 250 ms for 8 s, each with
`recv_deadline(sent + 6 s)`. Serially the close holds the loop for 7 s, so the
first Inspect times out at 6.25 s. Together, under the bound, it takes 500 ms.

`fixed.rs` adds the fix's own promise, after Eon's test
`a_close_stops_its_terminals_together_and_keeps_only_the_unended`. One
terminal never confirms its end. When the scope's deadline cancels that
stop's wait, the close keeps that terminal and prunes the other 13. Inspect
is still answered, after 4.75 s.

`open_start.rs` is eon-pimct's falsifier, with `edge`'s 15 s start bound. Its
failing input is a start whose Ready never comes, which would hold the loop up
to 15 s. That input is chosen, not an observed incident: Eon's records show one
start holding Inspect for 3.8 s, and none past 6 s. Under `main`'s 5 s bound,
one held start stays under Venus's 6 s, which is why eon-pimct names several
queued starts stacking up.

## What the rewrite models, and what it does not

It models the structure of the request loop. One task serves `Inspect`,
`Close` and `Start` requests from one channel, in order. Venus is a task with
Venus's period and timeout.

It does not model:

- Orbit, its signals, or its grace. A terminal's stop is a wait of the time
  Orbit takes to confirm the end (`clock.sleep(ends_after)`).
- The Unix socket transport, the EONW protocol, or how Eon's waits are
  implemented. A start's Ready is a channel message.
- OS scheduling under load.
- The shape of Eon's fix. Eon writes Stop to every terminal first, then
  reads the replies on the same thread under one shared deadline. The model
  gives each stop its own task in one scope with one deadline, Tide's idiom
  for the same bound.

So the claim is about the loop's structure, not about Eon's code. A
hand-written model shows only what it models (fx-dett): the test fails
because the model's loop waits on a close, and Eon's did too.

## What Tide would and would not change in Eon

Would not:

- reject the serial close or the synchronous start. Nothing in either is
  outside Tide;
- choose the fix. The test states the property ("Inspect is answered within
  6 s while a tab closes"), and someone has to think of it;
- bound Eon's latency. A deadline bounds a wait, not the time the OS takes.

Would:

- **Make the test cheap and exact.** The supervisor's only clock is the
  `Clock` it is handed, so under the simulator its waits run in virtual time.
  The 7 s close and 8 s of refreshes take milliseconds, deterministically.
  Eon's own red/green test runs in real time and fails at
  3.78 s on the serial loop. Eon found the bug with a stress reproduction.
- **State the bound as a deadline.** In `fixed.rs` the five seconds is
  `s.set_deadline(..)`. Every wait inside the scope shares it, a nested
  scope's included (reference §7.2), so a second wait cannot start a fresh
  five seconds while it sits inside that scope. A second scope opened after
  the first would get its own five seconds. Eon's first fix candidate had a
  defect of this kind: a terminal reconciled twice, each time with a fresh
  five-second wait. Its review caught it (eon-8wh7h record 04).
- **Deliver cancellation as an ordinary error.** The deadline cancels each
  unfinished wait, and the cancelled wait returns `Err`, so the fix's filter
  keeps a terminal exactly when its wait did not return `Ok`.

## What the Rust version would need

Rust can write this test. Eon's own test does, in real time. To run it in
virtual time with a repeatable result, a Rust supervisor needs its waits
behind a clock it is handed (a trait with a fake implementation), or an async
runtime with a paused clock, such as tokio's. Tide's std gives every program
that clock, because `Clock` is its only clock. Rust also has no scope
deadline: the shared bound is a deadline value passed to every wait, and
each wait has to honour it.

## License and source

Eon is the Tide owner's project, licensed under Apache-2.0, as Tide is. These
files are written fresh for Tide and copy no Eon code. The quoted lines come
from Eon's commit `d431fcab` and from bead eon-pimct, under Eon's license.
Every figure comes from beads eon-8wh7h and eon-pimct, read with
`br --no-db`, and from Eon's git history: commits `d431fcab` and `1766afd1`
on `main`, and `283bf76a` and `dcaef1c` on `edge` for the 15 s start bound.

## Run it

`cargo test --workspace` checks every declared outcome, through the examples
runner in `crates/tide-admit/tests/examples.rs`; to run only the examples:

```sh
cargo test -p tide-admit --test examples
```
