# 4. Structured tasks and `Shared`

## The idea

A **task** is one strand of work that runs at the same time as others. Under
v0 each task is an operating-system thread. Concurrency bugs come from three
habits, and Tide removes each one.

**Habit 1: starting work that nobody waits for.** In Rust,
`std::thread::spawn` starts a thread that can outlive the function that
started it. Its errors can vanish, and nothing cancels it. Tide uses
**structured concurrency** instead: a task can only be started inside a
**scope**, and the scope doesn't end until every task in it has finished.

```text
  serve(tasks, load)
   └─ tasks.scope(|scope| {          <- the scope opens
        scope.spawn(worker 1)  ──┐
        scope.spawn(worker 2)  ──┤   children may borrow the parent's data,
      })                         │   because they cannot outlive it
   <- the scope closes only when ┘   every child is done
```

Rules that follow from it (`design/01-design.md` D8):

- Starting a task needs the `Tasks` capability (chapter 1), so a function's
  signature shows whether it can run work concurrently.
- The first child that fails cancels its siblings. Then the scope reports the
  failure. A rule decides which failure is reported: a sibling's error that
  is only a reaction to being cancelled (an **echo**) never counts, so the
  real failure isn't masked by the cancellations it caused.
- **Cancellation** is cooperative: a cancelled task notices at its next
  **checkpoint**, an operation such as a file read, a sleep, or a channel
  receive. There the operation returns a `Cancelled` error, which the code
  passes on (usually with `?`) or handles. Not everything is a checkpoint:
  pure computation, reading the clock, starting a task without a limit,
  `Shared::with`, and waiting for children are not interrupted, and a system
  call already running in the kernel may finish first. So **deadlines are
  soft**: a task that keeps computing past its deadline keeps its scope
  waiting until it reaches a checkpoint or ends.
- There is no `async`/`await`, Rust's other concurrency model (`TD0601`). A
  function can only suspend itself (give up its turn and wait) by using a
  capability or a shared handle, so code with neither never suspends
  itself. The operating system can still pause any thread at any moment to
  run another. Where tasks share data, that can change the result: the
  demo's lost update (chapter 5) is exactly such an interleaving. The
  simulator explores those interleavings at the operations where tasks
  interact.

**Habit 2: locks that deadlock.** A **lock** (Rust's `Mutex`) lets one task
at a time touch some data. A **deadlock** is two tasks each holding one lock
and waiting forever for the other's. Tide has no `Mutex` (`TD0604`). It has
**`Shared<T>`**, a cell holding a value of type `T`, which you can only use
like this:

```rust
stats.with(|s| s.hits += 1)
```

`with` takes the cell, runs the small function you give it (a **closure**),
and lets go. The closure must be **`Pure`**: it may hold no capability and no
other `Shared`. So while holding the cell it can't read a file, take another cell, or
wait through a capability (`TD0603`). That makes `Shared` cells
deadlock-free by construction (D8): a task holding a cell never waits for
another cell or for capability I/O. It can still print, and printing can
briefly block when the output is backed up (design rules LOCK-4 and SINK-5,
in `design/03-authority-concurrency.md`). To change two
cells together, `Shared::with2(&a, &b, ..)` takes both at once, in a fixed
global order. If the closure panics (crashes), the cell is marked
**poisoned** and later users get an error instead of half-changed data.

**Habit 3: sharing without permission.** Making a `Shared` cell, a channel,
or any other thing tasks share needs `&Tasks` too (`TD0505`). So code with no
capabilities can't even see other tasks' timing. It stays deterministic:
same inputs, same result.

What this does **not** prevent: logic errors. Each `with` is safe on its own,
but two tasks can still interleave a read in one `with` and a write in a later
one, and lose an update. That is a **race condition**: correct-looking code
whose result depends on the order tasks happen to run in. The demo's bug is
exactly that, and chapter 5 shows how Tide finds it.

## Run it

Three Rust habits, each rejected:
[examples/habits/src/lib.rs](examples/habits/src/lib.rs).

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/habits docs/owner-guide/examples/habits/src/lib.rs
```

Output (paths shortened):

```text
…/habits/src/lib.rs:8:18: error[E0433]: cannot find `thread` in `std`: could not find `thread` in `std`
…/habits/src/lib.rs:13:24: error[E0603]: struct `Mutex` is private: private struct
…/habits/src/lib.rs:8:13: error: TD0602: `std::thread::functions::spawn` is not admitted (Tide's std does not export it)
…/habits/src/lib.rs:13:13: error: TD0604: `std::sync::poison::mutex::Mutex::<T>::new` is not admitted
…/habits/src/lib.rs:19:5: error: TD0603: a `Shared::with` closure must be `Pure`: `{closure@lib.rs:19:15}` is not `Pure`: it reaches `Fs`, which is never `Pure`
error: aborting due to 5 previous errors
```

Then read the demo service, which is the right way to write it. It is 58
lines:

```sh
cat demo/service/src/hits.rs
```

## What to look at

- One code per habit: `TD0602` for the bare thread, `TD0604` for `Mutex`,
  `TD0603` for a `with` closure that holds a capability ("it reaches `Fs`,
  which is never `Pure`").
- The two `E` lines are rustc's own view: Tide's std has no `thread` module,
  and its `Mutex` is private (std uses one internally).
- In `hits.rs`: `serve(tasks: &Tasks, load: &Load)`. Its only capability is
  `&Tasks`, so it can start tasks but can't touch files or the network. The
  workers are spawned inside one `tasks.scope(..)`, and `record_hit` does its
  whole update inside one `with`.

## Read the code

**`demo/service/src/hits.rs`**, the whole file (58 lines, 15 minutes). The
service the demo tests, in its fixed form.

- Lines 1–5: `//!` comments describe the file. Read them as prose.
- Lines 6–7, `use ...`: imports. The file uses `Shared` and `LockError` (the
  cell and its error) and `Tasks` and `TaskError` (the capability to start
  tasks and its error).
- Lines 9–13, `pub struct Load`: the input, a record with two numbers: how
  many clients, and how many page views each. `pub` means visible to other
  crates; `u32` is a 32-bit unsigned whole number.
- Lines 15–20, `pub enum ServeError`: "one of these": either a task error or
  a lock error. `#[derive(Debug)]` lets it be printed.
- Lines 22–32, two `impl From<..>` blocks: boilerplate that lets the `?`
  operator (below) turn a `TaskError` or `LockError` into a `ServeError`
  automatically.
- Lines 34–36, `struct Stats`: what lives inside the shared cell, one counter.
- Line 39, the signature: `serve(tasks: &Tasks, load: &Load) ->
  Result<u64, ServeError>`. It takes the `Tasks` capability and the load, and
  returns either a count (`u64`) or an error. **This line is chapter 1 in
  action: the only authority `serve` has is `&Tasks`.** No files, no network.
- Line 40: create the cell, starting at zero hits. Creating a `Shared` needs
  `tasks`, because tasks will share it.
- Line 41: open a **scope**. Everything spawned inside it finishes before
  line 51 is reached. The `::<_, ServeError, _>` part only tells the compiler
  the error type.
- Lines 42–49: for each client, `scope.spawn(|| { .. })` starts a worker task.
  The `|| { .. }` is a closure, the code the task runs: for each view, call
  `record_hit`. The `?` after it means "if this failed, stop and return the
  error".
- Line 50, `Ok(())`: the scope body succeeded. `Ok(..)` wraps a success
  value; `()` is Rust's "nothing", so `Ok(())` means "succeeded, with nothing
  to return". Line 51, `})?`: the scope ends
  here, after waiting for every worker; `?` passes on any failure.
- Line 52: read the final count inside one `with`, and return it.
- Lines 56–58, `record_hit`: the whole update, read and write together,
  inside **one** `with`. No other task can access this cell between the
  read and the write.

Then look at the bug, the patch chapter 5 uses:

```sh
cat demo/patches/lost-update.patch
```

The lines starting with `-` are removed and `+` added. The buggy version
splits `record_hit` into two `with` calls: line one reads the count into
`seen`, line two writes `seen + 1`. Each is safe; together they leave a gap
where another worker can read the same `seen`.

## The hardest question

> "Rust already has scoped threads and mutexes. What's actually new? And
> isn't one thread per task slow?"

**What's new is that the safe way is the only way.** Rust has
`std::thread::scope`, but nothing stops a program from also using
`thread::spawn`, detached tasks, `async` runtimes, or a `Mutex` held across a
network call. Tide admits only the structured form, so every program has the
same shape, and agents and reviewers can rely on it. Three things come with
it that Rust doesn't give you: cancellation and deadlines checked at every
checkpoint (capability operations that wait, such as I/O, sleeps and channel
receives); cells that can't deadlock with each other, because the `Pure` rule
forbids waiting while holding one; and every place where one task can affect
another's timing is a capability or shared-handle operation. In production,
tasks are OS threads that the operating system can switch at any
instruction; but because tasks only interact through those operations, the
simulator can explore the interleavings that matter by switching tasks only
there (chapter 5).

**On speed, be honest:** v0's runtime gives each task its own OS thread. That
is simple, and it can't starve a thread pool, but it costs memory per task and
won't scale to hundreds of thousands of connections. An **evented runtime**
(many tasks on few threads, compiled to state machines, as `async` Rust does
internally) is planned for v1. The parallel helpers (`par_map`, `join`)
start a thread per fork until a planned work-stealing pool exists. Channels
can still deadlock (two tasks each waiting to receive). The simulator reports
that with the blocked tasks listed, but the type system doesn't prevent it.

## Going deeper

- `design/01-design.md` D8 (concurrency), D9 (parallelism).
- `design/03-authority-concurrency.md` §6.1 (scopes), §6.2 (failure,
  cancellation, deadlines), §6.5 (`Shared::with`).
- `design/reference.md` §7.
- `demo/README.md`, "The service" and "The bug".
