# 5. Seeds and the simulator

## The idea

A race condition (chapter 4) shows up only when tasks happen to run in an
unlucky order. On a real machine that order depends on the operating system,
the load, and luck. So the bug appears once in a thousand runs, and when you
try to watch it, it hides.

Tide's answer is **deterministic simulation**. In a test, instead of the real
`Root`, you get a **simulated root**: the same capabilities, backed by models.
Its clock is a fake clock, its files live in memory, its network is a model
of local TCP. And one **scheduler** decides which task runs next:

- the simulator runs **one task at a time**, and switches only at a
  **scheduling point**, a capability or `Shared` operation (chapter 4);
- at each scheduling point, it picks the next task using a **seed**, a number
  that drives a pseudo-random generator. On the same code and inputs, the same
  seed gives the same choices;
- time is **virtual**: it starts at zero and jumps ahead only when every task
  is waiting, so a test that "sleeps an hour" takes no real time.

So **a seed is a schedule.** Run the same test under seeds 0 to 99 and you've
tried 100 different interleavings. Rerun a failing seed on the same code and
it fails the same way, so the race becomes an ordinary failing test.

Two seeds, sketched (the demo's seed 0 fails and seed 1 passes; the steps
below illustrate the idea and aren't copied from their traces):

```text
  seed 0:  worker1 read(0) ─ worker2 read(0) ─ worker1 write(1) ─ worker2 write(1)  → 1 hit  ✗
  seed 1:  worker1 read(0) ─ worker1 write(1) ─ worker2 read(1) ─ worker2 write(2)  → 2 hits ✓
```

Why this works in Tide and is hard elsewhere: every source of
nondeterminism (time, files, network, randomness, task switches) goes through
a capability or a task handle, and a test's root is simulated: test code
can't name the real `Root`. So admitted code without the `unsafe` grant has
no way to read the real clock or start a real thread behind the simulator's
back, barring compiler soundness bugs (chapter 3). (A real file handle used inside a simulation panics rather than
slipping through.) In ordinary Rust, any library can, which is why simulation as a
library breaks down once dependencies call the real clock (`research/10`).

You write one with `#[sim_test]`:

```rust
#[sim_test(seeds = 16)]
fn every_view_is_counted(sim: SimRoot) {
    let load = Load { clients: 2, views: 1 };
    let hits = serve(&sim.tasks, &load).unwrap();
    assert_eq!(hits, 2, "lost hits");
}
```

The simulator also checks itself. It runs the first seed and every failing
seed twice and compares the two **traces** (the logs of every scheduling
decision and event). If they differ, the run wasn't deterministic, and the
test fails with that report.

## Run it

The demo's step 1 does exactly this. Run it (or reuse the output from Setup):

```sh
cargo run -p tide-demo
```

Then open the search log, which has every seed's verdict:

```sh
cat target/tide-demo/logs/search.log
```

## What to look at

In the demo output, step 1:

```text
== 1. Find a seed that exposes the bug (buggy revision, demo/patches/lost-update.patch)
simulation every_view_is_counted: 8 passed; 8 failed (16 seeds)
failing seeds: 0 2 4 8 9 10 12 14 (each cause: lost hits)
same code and input on every seed; only the schedule differs
first failing seed: 0
```

- **About half the seeds fail, all with the same cause** (`lost hits`). Same
  code, same input; only the schedule differs. That proves the bug is a race.
- In `search.log`, each failing seed prints `fresh rerun:` and `replay:`
  commands, ready to paste. Chapter 6 uses them.
- The first line of the log reads `sandbox: full (...)`: the test ran inside
  the kernel sandbox (chapter 8).
- Step 3 runs the **fixed** service over the same 16 seeds: all pass.

## Read the code

**`demo/tests/src/hits_sim.rs`** (14 lines, 5 minutes). The simulation test
that finds the bug.

```sh
cat demo/tests/src/hits_sim.rs
```

- Lines 1–2: the demo pairs this test with one version of the service at a
  time, which the test sees as the crate `tide_candidate`.
- Lines 3–4: imports: `SimRoot`, the simulated root, and the service's `Load`
  and `serve`.
- Line 7, `#[sim_test(seeds = 16)]`: "run this test under the simulator 16
  times, with seeds 0 to 15".
- Line 8: the test receives `sim`, a simulated root. Only tests can have one
  (`TD0504`).
- Line 9: two clients, one view each.
- Line 10: call the real service, handing it the **simulated** `Tasks`
  (`&sim.tasks`). The service code is unchanged; it can't tell. That's
  capabilities paying off: whoever hands out `Tasks` decides how tasks run.
- Line 11: expected hits: clients × views = 2. `u64::from` converts the
  number type without any loss.
- Line 12: print a line. In a simulated run it goes into the trace, not your
  terminal.
- Line 13: the check. If hits isn't 2, the test fails with "lost hits".

## The hardest question

> "Antithesis already runs any program deterministically, and Rust has
> madsim, turmoil, shuttle and loom. Why a whole language for this?"

**What they do well:** Antithesis runs any containerized program in a
deterministic hypervisor, with no code change. It's powerful, and it's a
commercial service. madsim and turmoil simulate Tokio programs. loom and
shuttle explore thread interleavings for small pieces of code.

**What Tide adds:** the library tools work only as long as every line of the
program, its dependencies included, goes through their simulated APIs. One
call to the real clock or a real thread, from any dependency, silently breaks
determinism. In Tide the checker rejects that call. So that nothing bypasses the simulator is a property the checker enforces,
not a convention. (Which interleavings a run explores still depends on the
seeds you try.) And it runs in an ordinary test, on a laptop, as part of the edit
loop.

**The honest limits:**

- **It reproduces what it models.** CPU time is not modeled: a bug that
  depends on one task being *slow* won't show up. Nor is real kernel behavior
  (how a real disk or TCP stack fails), memory exhaustion, or signals. The
  simulator finds **scheduling bugs in Tide-written logic**.
- **Models built today:** tasks, channels, `Shared`, files, TCP on the local
  address, HTTP peers declared in `world.http` (with `Net::resolve` giving
  their names synthetic addresses), child processes declared as
  `sim.world.proc` models, the clock, the environment, and stdin.
  **Planned:** simulated randomness, fault injection (dropped packets, disk
  errors, crashes and restarts), and `tide sim` for whole programs. Today the
  simulator runs tests, not whole programs.
- **Default strategy is random.** Other strategies that hunt for rare
  interleavings more cleverly (such as PCT) are planned.

## Going deeper

- `design/01-design.md` D15 (simulation).
- `design/03-authority-concurrency.md` §9.2 (`Root::sim`), §9.3 (time and
  scheduling), §9.7 (the self-check), §9.8 (coverage and limits).
- `demo/README.md`; `demo/tests/src/hits_sim.rs`.
- `reviews/2026-11-15-achilles-opus.md` W7 (simulator fidelity).
