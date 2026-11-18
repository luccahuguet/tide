# Case study: hyper's pool reaches for an ambient runtime

**No Tide admission code in this case study catches either reported failure.
What Tide gives is the fix's shape, and a test shows it working.** hyper
0.14's client pool runs its idle-connection reaper on whichever tokio runtime
is ambient. Two reports follow from that:

- **The runtime-feature report** is a panic in tokio's timer, under the
  reporter's own executor. In `buggy.rs` that timer, `tokio::time::interval`,
  gets only rustc's `E0433`: Tide admits no external crate (reference §13), so
  any compiler without tokio reports the same. Tide's own rule for that reach,
  an ambient clock or sleep, is `TD0501`. It is shown on Rust's
  `Instant::now()` in `conformance/cases/authority/cap_ambient.rs`, not in
  this case study.
- **The open report, #2808,** is a reaper that never starts. Tide does not
  prevent that. `no_clock.rs`, a pool whose optional clock is left unset, is
  admitted, and the reaping test fails on it.

The Tide codes `buggy.rs` does get cover neither failure:

- `TD0602` rejects the pool's default spawner, `tokio::spawn`, which neither
  reported failure ran through;
- `TD0601` is there because Tide has no async.

The fix's shape: in Tide a pool can start its reaper only in a scope its
caller hands it, and can read time only from a `Clock` it is handed.
`fixed.rs` is written that way, and its `#[sim_test]` shows the pool closing
idle connections on each of its 20 seeds.

**Feature:** capability-passing I/O and structured tasks
(`design/01-design.md` D7 and D8; reference §6.1, §6.9, §7, §7.1).
Survey and ranking: `research/33-case-study-candidates.md` §4.3 and §5,
rank 2.

## The bug

hyper is a Rust HTTP library. Its 0.14 client keeps idle keep-alive
connections in a pool for reuse, and closes each one after it sits idle for
`pool_idle_timeout`, 90 s by default. An interval task does the closing. In
hyper 0.14.32 (`src/client/pool.rs`) that task exists only under
`#[cfg(feature = "runtime")]`, and `spawn_idle_interval` builds it on tokio's
timer and hands it to the client's executor, which defaults to
`tokio::task::spawn`:

```rust
let interval = IdleTask {
    interval: tokio::time::interval(dur),
    pool: WeakOpt::downgrade(pool_ref),
    pool_drop_notifier: rx,
};

self.exec.execute(interval);
```

(hyper v0.14.32, `src/client/pool.rs`, lines 423–429; MIT, © Sean McArthur.)

Two reports follow from that `cfg`:

- **Without `runtime`, idle connections are never closed.**
  [hyperium/hyper#2808](https://github.com/hyperium/hyper/issues/2808), open:
  Fuchsia runs hyper's client on its own executor, and on `async-executor`
  for host tools, so it cannot turn on `runtime`, which pulls in tokio. The
  pool then never starts its idle cleaner, and idle connections stay open.
  The report was filed against 0.13.10; the `cfg` is unchanged in 0.14.32. A
  maintainer answered (2022-04-07) with the 1.0 plan, a `Timer` trait and the
  client's move to hyper-util, and added: "A temporary solution in 0.14.x
  seems feasible though." The issue is still open.
- **With `runtime`, a foreign executor panics.**
  hyper's report ["`runtime` feature is not additive"](https://github.com/hyperium/hyper/issues/2595),
  closed as not planned and tracked in
  [the issue](https://github.com/hyperium/hyper/issues/2857) that the 1.0 fix
  closed. Cargo features are additive, so another crate such as reqwest can
  turn `runtime` on. Under a non-tokio executor, returning a connection to
  the pool then panicked with "there is no reactor running, must be called
  from the context of a Tokio 1.x runtime". The report's backtrace runs
  through `tokio::time::interval` in `PoolInner::spawn_idle_interval`.

**The fix.** [hyper PR #2975](https://github.com/hyperium/hyper/pull/2975)
(commit `23a56005`, 2022-09-15) removed the `runtime` feature for hyper 1.0,
which takes a `Timer` from its caller. The pooled client moved to hyper-util's
`client::legacy`. There the idle interval starts only when the caller sets
`pool_timer`: `spawn_idle_interval` returns early without a timer
([hyper-util `src/client/legacy/pool.rs`](https://github.com/hyperium/hyper-util/blob/1e50e91f9361aae0d030b30667e606c1be185a4a/src/client/legacy/pool.rs#L425-L441)).
So 1.x runs the idle interval only when the caller sets `pool_timer`; without
it, expired connections are dropped only at checkout, and the pool still
reads the ambient clock:

```rust
fn now(&self) -> Instant {
    self.timer
        .as_ref()
        .map_or_else(|| Instant::now(), |t| t.now())
}
```

(hyper-util at `1e50e91`, `src/client/legacy/pool.rs`, lines 342–346; MIT,
© Sean McArthur.)

## The files

| File | What it is | Declared outcome |
| --- | --- | --- |
| `buggy.rs` | hyper 0.14's pool with `runtime`: the pool's task runs in the caller's scope, but its idle interval is spawned with `tokio::spawn` and ticks on `tokio::time::interval`. | admission rejects it, by no code aimed at either report: `TD0602` for the default spawner `tokio::spawn`, which neither reported failure ran through; `TD0601` for the `async` block and its `.await`, because Tide has no async; and only rustc's `E0433` for each `tokio` path, the panicking `tokio::time::interval` included |
| `fixed.rs` | The same pool with the interval passed in: spawned into the caller's scope, ticking on the `Clock` the caller hands it. | its test passes on 20 seeds |
| `no_clock.rs` | `fixed.rs` with the clock optional, as hyper-util's `pool_timer` is, and the test building the pool without one. | `idle_connections_close_after_the_timeout` fails: "an idle connection was still open 181s after its check-in: Timeout" |
| `rust-repro.md` | Plain Rust against hyper 0.14.32: a deterministic reproduction of #2808 on tokio's paused clock. | not run by the workspace; see below |

`fixed.rs` differs from `buggy.rs` in the three places its header lists. Its
test, shared with
`no_clock.rs`, runs a server task per connection over the simulator's TCP
loopback. Each reports when the client closes its connection. The test checks
two connections into a pool whose idle timeout is hyper's 90 s, checking one
out and in again on the way. It then requires the server to see both closed
no earlier than 90 s after the check-in and no later than 181 s after it:
the interval ticks once per timeout, so an idle connection goes at the second
tick at the latest. Virtual time makes the three minutes free.

## What the rewrite models, and what it does not

It models the pool's idle handling: a pool task owns the idle connections
and hands out the most recently returned one, as hyper's does; an interval
ticks once per timeout; on each tick the pool drops every connection idle for
longer than the timeout, which closes it.

It does not model:

- async Rust. Tide has no `async` (reference §7), so the rewrite is the
  blocking translation, a task per concurrent activity. hyper's `Executor`
  and `Timer` traits, futures and wakers are not modelled.
- HTTP. Simulated HTTP is planned, so the pool holds raw TCP connections,
  and there is no h1 dispatch, no request on the connection, and no pool key
  per host.
- the rest of hyper's pool: its per-host limit, its check at checkout that
  drops a connection found expired, and the weak reference and drop notifier
  by which its interval stops when the pool is dropped. In the model the
  owner stops the pool by cancelling the scope it handed over (reference
  §7.2).

So the claim is about where the pool's timer and spawner come from, not about
hyper's code. A hand-written model shows only what it models (fx-dett).

## What Tide would and would not change in hyper

Would:

- **Reject the ambient reach at admission, in its std spellings.** This is
  the rule, not this case study's catch (see the top). Library code has no spawner
  but a scope it is handed (`TD0602` for `tokio::spawn` and
  `std::thread::spawn`), and no clock but a `Clock` it is handed (`TD0501` for
  Rust's `Instant::now()`, as in
  `conformance/cases/authority/cap_ambient.rs`). The same `TD0501` would reject
  the ambient fallbacks above: hyper 0.14 without `runtime` stamps idle
  connections with `std::time::Instant::now()`, and so does hyper-util's pool
  without a timer. A pool's need for a timer and a spawner shows in its
  signature, and its caller supplies both.
- **Remove the runtime axis.** A Tide library never chooses a runtime
  (reference §7.5), so there is no `runtime` feature to turn on or off, and
  the foreign-executor panic has no counterpart. `fixed.rs`'s pool is written
  to run unchanged under the threads runtime and under the simulator; only
  the simulator runs it here.
- **Keep every clock inside the simulator.** The pool's only clock is the one
  it is handed, so under the simulator the 181 s pass in virtual time,
  deterministically. tokio's paused clock does the same for `rust-repro.md`,
  but only because hyper 0.14 with `runtime` stamps with
  `tokio::time::Instant`. Without `runtime`, and in hyper-util without a
  timer, the pool stamps with `std::time::Instant`, which the paused clock
  does not control. In Tide there is no ambient clock to escape the
  simulator.

Would not:

- **make a pool reap.** `no_clock.rs` is admitted. Nothing in Tide requires a
  clock parameter rather than an optional one. The guarantee is narrower: a
  pool without a clock cannot read the time, so it cannot fall back to
  ambient time as hyper-util's does.
- **reject tokio's timer by a Tide rule.** It fails only because Tide
  admits no external crate.

## The plain-Rust reproduction

`rust-repro.md` is a standalone Cargo package, not a member of this
workspace, that reproduces #2808 against hyper 0.14.32; it describes the test
and its two outcomes. Tide's workspace does not run it, since that would mean
adding hyper and tokio as dependencies. It was checked by hand on 2026-11-17,
offline from cached crates, with stable rustc 1.96.0, hyper 0.14.32 and tokio
1.54.1 (its `Cargo.toml` asks for tokio 1; that run resolved 1.54.1). Each
build was run three times, and every run gave its stated outcome.

## What the Rust version would need

Rust can pass the timer in: hyper's own 1.x answer, a `Timer` its caller
passes in, is the same capability idea, written in Rust as an option. What
Rust cannot do is stop a library, or any of its dependencies, from reaching
past that parameter to `Instant::now()` or `tokio::spawn`. Clippy's
`disallowed_methods` and `disallowed_types` lists come closest, but they name
each path to forbid and apply only to the crates Clippy lints with that
configuration, not to dependencies. hyper-util's fallback to `Instant::now()`
is ordinary, reviewable Rust.

## License and source

hyper and hyper-util are MIT-licensed (© Sean McArthur). These files are
written fresh for Tide and copy no hyper code. The two quoted snippets come
from hyper v0.14.32 and hyper-util at `1e50e91`, under the MIT license.
Sources: the hyper issues and pull request linked above, hyper's
`src/client/pool.rs` at tag v0.14.32, and hyper-util's
`src/client/legacy/pool.rs` at `1e50e91f9361aae0d030b30667e606c1be185a4a`, all
read 2026-11-17.

## Run it

`cargo test --workspace` checks every declared outcome, through the examples
runner in `crates/tide-admit/tests/examples.rs`; to run only the examples:

```sh
cargo test -p tide-admit --test examples
```
