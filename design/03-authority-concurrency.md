# Authority, Concurrency, and Simulation

Status: draft 1 of the normative chapter for D7, D8, D9, D15, D16, and the
Rust-import part of D13 in `design/01-design.md` (draft 2.1). That document
owns the decisions; this chapter turns them into rules a compiler, runtime,
and simulator can implement. Where a rule needs a decision the design does
not state, this chapter says so and lists it in §13.

Related chapters: `02-memory.md` owns origins, `Send`/`Sync`, the typing of
scopes and `lock`, and the full `Pure`/`Frozen` tables (its §8–§9);
`04-contracts.md` owns verification of `lock` bodies and `par_reduce`;
`reference.md` is the agent-facing summary. This chapter restates the
definitions it relies on and adds the operational rules. API names marked
provisional in `reference.md` are fixed here unless noted.

## 0. Terms

- **Capability**: a value of a standard capability type (§1.1) or of a
  resource handle derived from one (`File`, `TcpStream`, `TcpListener`,
  `Child`). Every operation that reads or changes the outside world, or that
  observes nondeterminism, is a method on a capability.
- **Synchronization handle**: a value through which tasks observe each other:
  `Shared<T>`, channel ends, `Scope<E>`, `Task<U, E>`, `sync::Weak<T>`, and
  atomics. Creating one needs no capability; sharing it needs a spawn.
- **Authority kind**: one element of the closed set `fs.read`, `fs.write`,
  `net`, `clock`, `rand`, `env`, `stdin`, `proc`, `tasks`, `sync`, `accel`,
  `foreign`. Declared and exercised authority (§4) are sets of kinds, each
  with the access path that carries it.
- **Effect**: a sink a function writes to (`out`, `err`, `log`, `trace`,
  `dbg`). Effects are reported beside authority but are not authority.
- **Suspend**: give up the current OS thread or worker until an event
  arrives. Only capability and synchronization operations suspend.
- **Checkpoint**: an operation at which a task observes cancellation. Every
  capability operation that may suspend is a checkpoint. Joins and `lock`
  acquisition are not (§6).
- **Task**: a unit of concurrent execution with its own cancellation state,
  sink buffers, and, under the evented runtime, frame stack. `main` runs as
  the main task.

## 1. The Standard Capability Set

### 1.1 Capability types

The standard library reference holds the full signatures. "Suspends" lists
the operations that may suspend; all are checkpoints except the shielded
scope-exit join. Every capability error type has a cancellation variant whose
`is_cancelled()` is true (`NetError::Cancelled`, `io::ErrorKind::Cancelled`).

| Type | Main operations | Attenuation | Suspends |
| --- | --- | --- | --- |
| `Fs` | `read`, `read_to_string`, `write`, `write_synced`, `open`, `create`, `create_dir_all`, `read_dir`, `metadata`, `exists`, `rename`, `remove_file` | `dir(path, Mode::Read \| Mode::ReadWrite)`, `read_only()` | all |
| `File` | `read`, `read_owned`, `write`, `write_owned`, `seek`, `sync_all`, `close` | none | all |
| `Net` | `connect(addr)`, `bind(addr)`, `bind_udp(addr)`, `resolve(host)`, `get(&Url, deadline)` | `only(["host:port", ..])`, `no_bind()` | all |
| `TcpStream`, `TcpListener`, `UdpSocket` | `read`, `write`, `read_owned`, `write_owned`, `accept`, `send_to`, `recv_from`, `shutdown` | none | all |
| `Clock` | `now() -> Instant`, `system_time()`, `sleep(d)`, `sleep_until(t)` | `monotonic_only()` | `sleep`, `sleep_until` |
| `Rand` | `next_u64`, `gen_range`, `gen_bool`, `fill_bytes`, `shuffle`, `rng() -> Pcg64` | none | none |
| `Env` | `args()`, `var(k)`, `vars()`, `current_dir()` | `only(["HOME", ..])`, `args_only()` | none |
| `Stdin` | `read_line`, `lines`, `read_to_string`, `read_to_end`, `is_terminal` | none | all reads |
| `Proc` | `command(prog) -> Command` (`.output()`, `.status()`, `.spawn()`), `signals(&[Signal])`, `exit(code)` | `only(["git", ..])` | `output`, `status`, `Child::wait`, `Signals::recv` |
| `Tasks` | `scope`, `shield`, `checkpoint`, `yield_now`, `fault_point` | `limit(n)` (live children) | `checkpoint`, `yield_now`, `scope` exit (shielded) |
| `Accel` | generated Bend kernel calls (§10) | `budget(threads)`, `no_gpu()` | kernel calls |

Notes:

- **Paths.** An `Fs` resolves paths beneath its directory with
  `openat2(RESOLVE_BENEATH)` semantics: `..` and symlinks cannot leave it.
  `root.fs` is rooted at `/`. A `Mode::Read` `Fs` returns `PermissionDenied`
  on writes. Read versus write is not a type distinction; exercised authority
  (§4) classifies each method as `fs.read` or `fs.write`.
- **Owned buffers.** The kernel only ever owns runtime buffers.
  `read(&mut buf)` copies out of one; `read_owned(buf)` and `write_owned(buf)`
  move a `Vec<u8>` in and return it, inside the error on failure. Cancellation
  never leaves the kernel writing into unwound Ferox memory.
- **Randomness.** `rand.rng()` returns a seeded `Pcg64`, an ordinary `Pure`
  value. Authority is spent at seeding; drawing from a `Pcg64` is
  authority-free and inlines. `Pcg64::seed_from_u64(n)` needs no capability.
- **Processes.** A child process is not capability-checked, so `Proc` is the
  widest grant in the set and `ferox review` lists every new path to it.
- **`fault_point(name) -> bool`** returns `false` under every real runtime and
  folds to a constant; under the simulator it is a seed-driven fault trigger
  (§9.4).

### 1.2 Synchronization handles

| Type | Operations | Waits |
| --- | --- | --- |
| `Shared<T>`, `T: Send + Pure` | `Shared::new(v)`, `clone()` (same cell), `Shared::same(&a, &b)`, `reset(v)`; the `lock` expression (§6.5) | acquisition, not a checkpoint |
| `Sender<T>`, `SyncSender<T>`, `Receiver<T>`, `T: Send` | `channel()`, `sync_channel(n)`, `send`, `try_send`, `recv`, `try_recv`, `recv_deadline(t)`, iteration | `send` on a full `sync_channel`, `recv`; both checkpoints |
| `Scope<E>` | `spawn`, `set_deadline(t)`, `cancel()` | none |
| `Task<U, E>` | `join` | yes, shielded (§6.3) |
| `sync::Weak<T>` | `upgrade` | none |
| `std::sync::atomic::*` | sequentially consistent `load`, `store`, `fetch_add`, `compare_exchange` | none |

Weaker atomic orderings, `Mutex`, and `RwLock` live in `std::sync::raw`,
which requires the `unsafe` grant. To wait on several sources, send one enum
through one channel; there is no `select`.

### 1.3 `Root` and the four runtimes

`Root` has public fields `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
`proc`, `tasks`, `accel`, and `log: LogConfig`. User code cannot construct
it.

The binary's manifest picks the runtime; libraries never choose and compile
the same way for all runtimes.

```toml
[bin]
runtime = "threads"        # threads (default) | blocking | evented
workers = "cores"          # threads and evented
par-threads = "cores"      # the par pool (§8); 1 makes par sequential
```

The generated entry point starts the allocator, runtime, and sinks, builds
`Root` with full authority over the real process, and runs `main(root)` as
the main task. Exit statuses follow Rust: 1 after `Err(e)` (printed as
`Error: {e:?}`), 101 after a panic.

| Runtime | Tasks run as | Suspension is | Code generation |
| --- | --- | --- | --- |
| `blocking` | the main thread, plus one OS thread per child, created at spawn and joined at scope exit | a blocking syscall or futex wait | ordinary functions |
| `threads` | OS threads from a pool; a task stays on one thread for life | a blocking syscall or futex wait | ordinary functions |
| `evented` | M:N on per-core workers with work stealing; reactor on io_uring, else epoll, kqueue, or IOCP, chosen at startup | a return to the scheduler | may-suspend functions become state machines (§7) |
| simulator | one task at a time under a seeded scheduler | a scheduling point | baton-passed OS threads, the interpreter, or the evented lowering (§9.1) |

No runtime runs children inline, so a library that spawns is correct under
all four; they differ in cost, not meaning.

Capability types are concrete: production binaries link only the real
implementation, so capability calls are direct and attenuation is a policy
field checked at the system-call boundary (`reviews/round1-performance.md`
N1). Test builds add the simulator behind one predictable branch.

## 2. Rules

### 2.1 `Root`

- **ROOT-1.** `Root` may be named only as the parameter of the binary's
  `main`.
- **ROOT-2.** `root` cannot be passed, borrowed into a call, stored,
  returned, or captured (`FX0503`). Its fields can be moved or borrowed out
  one by one. Attenuation and wiring therefore happen in `main`, where review
  shows them (§4.3). The fix passes the fields the callee exercises.
- **ROOT-3.** `Root::sim`, `SimRoot`, `NodeRoot`, and everything in
  `std::sim` compile only in test code: `#[cfg(test)]` items, `#[test]` and
  `#[sim_test]` functions, and files under `tests/` (`FX0504`). Under
  `ferox sim` the runtime, not user code, builds the simulated root. A
  dependency's test code is never compiled into its dependents, so a
  simulated capability cannot reach production code.
- **ROOT-4.** Capability and handle types cannot be made from data: they have
  no public fields, no `Default` or `Decode` impls, and no `Shape`
  construction (D6). `mem::zeroed`, `transmute`, and raw-pointer reads in
  safe std require `T: Pure`. Unsafe code can still forge a capability by
  writing raw memory, so the lockfile marks every package with an `unsafe` or
  `ffi` grant `authority: declared, unverified`.

### 2.2 No mutable globals, no thread-locals

- **GLOBAL-1.** A `static` must be `Owned`, `Sync`, `Frozen`, and `Pure`
  (`FX0502`). There is no `static mut` in safe code.
- **GLOBAL-2.** `static T: Lazy<Table> = Lazy::new(build);` is allowed when
  `build` is an authority-free `fn` and `Table` meets GLOBAL-1. It runs at
  most once and yields the same value whichever task forces it, so the static
  is observably immutable. A panic in `build` poisons it for every later
  access.
- **GLOBAL-3.** Safe code has no thread-locals and no task-locals. The
  runtime's per-task state (cancellation, deadline, sink buffers, frame
  stack) is reachable only through the operations in this chapter.
- **GLOBAL-4.** Addresses are unobservable in safe code: no pointer-to-integer
  casts, no `{:p}`, and `Hash` on a reference hashes the referent.
  `ptr::eq` and `Shared::same` compare identity without exposing it.
- **GLOBAL-5.** `HashMap::new()` uses a fixed key, so iteration order is
  deterministic though unspecified. Maps keyed by untrusted input take a key
  from `Rand`: `HashMap::with_hasher(RandomState::from_rand(&rand))`.
- **GLOBAL-6.** `Arc<T>` omits `strong_count`, `weak_count`, `get_mut`,
  `try_unwrap`, and `into_inner`, because their results depend on when other
  tasks drop clones. `make_mut` stays: its observable result does not depend
  on timing. `sync::Weak` is a synchronization handle, because `upgrade`
  observes other tasks' drops. `Rc` and `rc::Weak` keep the full Rust API;
  they never leave their task.

What replaces each use of global state:

| Rust habit | Ferox |
| --- | --- |
| `lazy_static!`, `LazyLock`, `OnceLock` tables | `const`, or a lazy static under GLOBAL-2 |
| a global cache, memo table, or interner | a value owned by the caller and passed by `&mut`; in concurrent code, a `Shared<Cache>` |
| `#[global_allocator]` | the binary's manifest; allocator thread caches belong to the runtime |
| per-core counters | `metrics` output (§2.4); a `Shared` or atomic when the program must read the value back |
| `thread_rng()` | `rand.rng()` once, then pass the `Pcg64` |
| a global logger | the sinks (§2.4) |
| global configuration | a `Frozen` config value built in `main` and passed by reference |

### 2.3 Resource exhaustion

Allocation needs no capability; its failure aborts the process. Stack
overflow abandons the task. Every determinism claim here holds modulo
exhaustion of memory, stack, or `const fn` fuel.

### 2.4 Write-only sinks

| Sink | Forms |
| --- | --- |
| `out` | `print!`, `println!`, `std::io::stdout()` |
| `err` | `eprint!`, `eprintln!`, `std::io::stderr()` |
| `dbg` | `dbg!` |
| `log` | `error!`, `warn!`, `info!`, `debug!`, `trace!` |
| `trace` | `std::trace::span(name)` (a guard that closes on drop), `std::metrics::count`, `gauge`, `histogram` |

- **SINK-1. Write-only.** A sink operation returns `()`, or its argument for
  `dbg!`. The `impl Write` handles from `stdout()` and `stderr()` always
  return `Ok`. Nothing a sink returns depends on the world. Sinks never
  suspend, are not checkpoints, and do not affect `Pure`, authority-freedom,
  or the may-suspend analysis.
- **SINK-2. Configured only in `main`.** `root.log` sets levels, text or JSON
  format, and destinations (`root.log.to_file(file)`, where `file` is a
  `File` that `main` opened with its own authority). Defaults: `out` to
  stdout; `err`, and `log` at `info`, to stderr; `dbg` to stderr in debug
  builds; `trace` discarded unless a destination is set.
- **SINK-3. Arguments.** Format arguments are evaluated only when the
  record's level is enabled. A panic in a formatting impl called by a sink is
  caught and printed as `<fmt panicked>`, so sink configuration cannot change
  control flow.
- **SINK-4. Order.** Within a task, records appear in program order. Across
  tasks, real runtimes interleave them in emission order and the simulator in
  scheduler order. Inside `par`, each branch writes to its own buffer, and the
  buffers are appended in branch order at the join, so output matches the
  sequential program (§8.4).
- **SINK-5. Loss and backpressure.** `out` and `err` are lossless: when the
  process-wide buffer passes its high-water mark, the emitting OS thread
  blocks until the writer thread drains it. That is backpressure, not
  suspension. If stdout is a closed pipe, the process exits quietly with
  status 141, matching the default `SIGPIPE` behavior that `cmd | head`
  relies on. `log` and `trace` drop records under overload and report the
  count as a `log` record.
- **SINK-6. Under simulation.** Every record enters the event trace with
  node, task, and virtual time; tests read them with `sim.records()`.
- **SINK-7. Elision.** Output from authority-free code may appear zero or
  one times under memoization and `const fn` evaluation.
- **SINK-8. Release and library rules.** `dbg!` is an error in release and
  CI profiles (`FX0507`) and blocks publishing. `print!` in a library package
  is lint `FX0508`. Printing inside a `lock` body is lint `FX0607`, since
  `out` can block while the cell is held. `log` forms above the manifest's
  `release-max-level` compile to nothing in release builds.
- **SINK-9. Reported, not granted.** Sink use is an exercised effect (§4.1).
  The lockfile records each package's effects (`effects = ["log", "out"]`),
  and a dependency that adds one appears in the trust diff. Effects are not
  approval-gated by default; `[policy] gate-effects = true` gates them.
- **SINK-10. Secrets.** `Secret<T>` implements neither `Debug` nor `Display`,
  so formatting one is a type error; `expose()` reads the value. Exfiltration
  still needs `Net`, `Fs`, or `Proc`.

## 3. `Pure`, `Frozen`, and Authority-Free Functions

### 3.1 Definitions

Both traits are compiler-known auto traits. Outside std they can be
implemented only by `unsafe impl` in a package granted `unsafe`, and each
such impl is a trust hole. Both are deep: they look through every field,
private ones included, and through `Box`, `Rc`, and `Arc`; recursive types
take the greatest fixed point; `dyn` and `impl Trait` types have exactly the
auto traits their bounds name; a type parameter has one only if bounded by
it. `02-memory.md` §9 has the full tables.

**`Pure`** means that no authority is reachable. Not `Pure`:

- the capability types (`Root`, `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`,
  `Proc`, `Tasks`, `Accel`) and the resource handles derived from them;
- the synchronization handles (`Shared`, channel ends, `Scope`, `Task`,
  `sync::Weak`, atomics, everything in `std::sync::raw`);
- foreign handles: raw pointers, `extern` types, foreign function pointers;
- `ThreadBound` types;
- `dyn Trait` without `+ Pure`, and unbounded type parameters.

Everything else is `Pure` exactly when its components are, including `Cell`,
`RefCell`, `Rc`, `Arc`, collections, closures, and Ferox function pointers
(a pointee's authority arrives through its arguments).

**`Frozen`** means that nothing reachable can change through a shared
reference. Not `Frozen`: `Cell`, `RefCell`, `OnceCell`, `UnsafeCell`,
atomics, `Rc` and `rc::Weak` (their counts change through `&`), every
synchronization handle and capability, raw pointers, `dyn Trait` without
`+ Frozen`, and unbounded type parameters. `Arc<T>` is `Frozen` when `T` is,
because GLOBAL-6 makes its count invisible.

### 3.2 Authority-free functions

A function, method, or closure is **authority-free** when every parameter
type, including `self`, and every capture type is `Pure`. Generic functions
are classified per instantiation; a generic definition that needs the
property states it as a bound (`F: Fn(&T) -> U + Pure`). `extern` functions
are never authority-free; their wrappers may be (§5).

`#[pure]` asserts authority-freedom for every instantiation the bounds
allow; an edit that breaks it is `FX0511`. Every public function's interface
records the property anyway, and `ferox review` reports its loss as an API
change, which answers the design's open question 4.

### 3.3 What authority-freedom guarantees

For an authority-free function `f`:

1. **Determinism.** `f`'s result, its panic if any, and the final state of
   memory reachable from its arguments are a function of that memory's state
   at entry. Mutation through `&mut` and through task-private cells is
   allowed; it is a deterministic state transition.
2. **No suspension.** `f` never suspends and never observes cancellation. The
   evented runtime compiles it as an ordinary function (§7).
3. **No authority.** `f` performs no I/O and observes no time, randomness,
   environment, or other task. Its only outputs besides its result and its
   arguments' memory are sink records.
4. **Runtime independence.** `f` behaves identically under every runtime,
   including the simulator, and under every backend (D12).

The argument: safe code has no ambient source (§2.1, §2.2); `Pure` arguments
reach no capability or handle; and the memory they reach cannot be mutated
concurrently, because every type that is `Sync` and mutable through `&` is a
synchronization handle and therefore not `Pure`, while `Cell`, `RefCell`, and
`Rc` are not `Sync` and so cannot be shared with another task.

### 3.4 What it does not guarantee

- termination (divergence is tracked only under `prove`), absence of panics
  or overflow traps, absence of allocation, or bounded time and memory;
- absence of sink output, or of mutation through `&mut` parameters and
  task-private cells;
- memoizability, which also needs `Frozen` parameters and no `&mut`;
- safety for `par`, which also needs `Send` and `Frozen` shared captures;
- constant time or freedom from timing side channels.

### 3.5 Where each property is required

| Use | Requires |
| --- | --- |
| `par`, `join`, `par_*` closures | authority-free, `Send`; shared captures `Frozen + Sync`; mutable captures disjoint |
| compiler memoization, `const fn` | authority-free, `Frozen` parameters, no `&mut` parameters |
| `static`, lazy statics | `Owned + Sync + Frozen + Pure` |
| `Shared<T>` payloads | `T: Send + Pure` |
| `lock` bodies | captures `Pure`; calls only authority-free functions |
| `spec fn`, `#[spec]` functions | authority-free; specs read only owned and `Frozen` state (`04-contracts.md`) |
| foreign callbacks | authority-free (§5.3) |

## 4. Declared and Exercised Authority

### 4.1 How the compiler computes them

**Declared authority** of a function is computed from its signature: for
each parameter (and capture, for closures), the authority kinds reachable
from its type, each labelled with its access path. For
`fn handle(app: &App, req: &Request)` where `App` holds `db: Db` (which holds
a `Net`), `files: Fs`, and `clock: Clock`, declared authority is
`{app.db.net: net, app.files: fs.read+fs.write, app.clock: clock}`. A
`dyn Trait` without `+ Pure` and an unbounded type parameter contribute
`opaque(..)`, resolved below.

**Exercised authority** is computed bottom-up from bodies:

1. A capability method call contributes its kind (`Fs::read` is `fs.read`,
   `Fs::write` is `fs.write`), attributed to the access path of the receiver.
2. A call contributes the callee's exercised summary, with the callee's
   parameter paths replaced by the caller's argument paths.
3. A closure's exercised authority counts at the place it is created, so a
   higher-order callee such as `map` exercises nothing of its own.
4. A coercion of a concrete type to `dyn Trait` counts the exercised
   authority of that type's `Trait` methods at the coercion site; a call
   through the `dyn` value then counts nothing new. Together, 3 and 4 account
   for every closure and trait object exactly once, where its authority is
   chosen.
5. A foreign call contributes its wrapper's declared kinds (FFI-2) plus
   `foreign`.
6. Sink use contributes effects (`log`, `out`, ...).

Recursive groups take a fixed point over a finite lattice; generic
summaries refer to their type parameters' trait methods and resolve at
instantiation. The interface stores, per public item, declared and exercised
authority with paths, effects, and the authority-free and may-suspend flags
(§7). A body edit that changes an exercised summary makes dependents
recompute summaries, not re-check types; that change is exactly what review
exists to show.

The lockfile stores each package's public authority as the union of its
public items' exercised authority, plus effects and grants. An upgrade that
widens it fails until approved (D11, D13).

### 4.2 The least-authority lint

`FX0510` fires on a function other than `main` that holds a capability
parameter or field path it never exercises. It compares capability types, not
read versus write, so an `Fs` whose holder only reads is not flagged; review
suggests `Mode::Read` attenuation in `main` instead. The fix narrows a bundle
to the fields used (`&App` to `&Clock`) or removes the parameter. It is
`safe` for private items and `suggest` for public ones. The lint skips trait
method impls and methods whose receiver is the only authority-bearing
parameter; review still shows their exercised paths.

### 4.3 The trust diff

`ferox review <base>` has an authority section. It lists only items whose
summaries changed, grouped by kind, with the attenuation in `main` that
reaches each changed path:

```text
authority                                                        3 changes
  + app::sync::push          exercised  net       via app.db.net   new
      reaches: main: root.net.only(["db.internal:5432"])
  ~ app::store::compact      exercised  fs.read -> fs.read+fs.write  widened
      reaches: main: root.fs.dir("/var/lib/app", Mode::ReadWrite)
  + dep acme/fast-log 1.4.3  public     net                        needs approval
effects
  + app::decode::user        log
trust holes                                                      unchanged
```

New paths to `proc`, `foreign`, and `accel` are listed first. `--full`
prints every public item's declared and exercised authority.

### 4.4 `ferox fix --thread`

`ferox fix --thread <Cap> --to <fn>` adds a capability parameter along a
call chain as one checked, atomic, multi-file edit:

1. Choose the shortest call path from a function holding a `Cap` to `<fn>`;
   ties go to the path with fewest public functions.
2. Append `name: &Cap` to each function on it and pass it at every call site
   of those functions.
3. In test code, pass the matching field of the test's `Root::sim` value,
   adding `let sim = Root::sim(0);` if needed.
4. At a trait method, stop and report: the capability must live in the
   implementing type, which `--via <Type>.<field>` does.

The edit carries its base snapshot hash (D14) and applies entirely or not at
all.

### 4.5 The app-context pattern

A struct of attenuated capabilities and clients built by `main` is the
documented pattern for service code (`reference.md` §6.10). It stays honest
because exercised authority is tracked per field path (a method that starts
using `self.db.net` shows in review with no signature change), `FX0510`
narrows free functions that use one field of the bundle, `Root` cannot be
stored (ROOT-2) so attenuation stays visible in `main`, and the lockfile's
public authority does not depend on bundling. Experiment E6 (D0) measures how
often agents collapse everything into one bundle; if bundles defeat review,
the fallback is a declared authority list on types that hold capabilities.

## 5. Foreign Calls

### 5.1 Declarations and wrappers

- **FFI-1.** `extern` blocks are allowed only in packages granted `ffi`.
  Calling a foreign function is `unsafe` and needs a `// SAFETY:` comment.
- **FFI-2.** A safe function that calls a foreign function directly is a
  *wrapper*. A wrapper must either take at least one capability, in which
  case its declared authority is its claim about what the foreign code may
  do, or be `#[trusted_pure(reason = "...")]` (`FX0509`). Every foreign call
  is a counted trust hole (D11); the trust surface shows the wrapper's claim
  beside it as `foreign: claims fs.write`.
- **FFI-3.** `#[trusted_pure]` claims all of the following, and a reviewer
  approves all of them at once:
  1. the result and the final state of the arguments' memory are a function
     of the arguments;
  2. no I/O and no reads of time, randomness, the environment, or global
     state that outlives the call;
  3. the function is reentrant and safe to call from several threads at once
     (it may run inside `par`);
  4. any callback runs only during the call;
  5. the same holds for the foreign destructor of any handle it returns.
  Pure foreign code that is not thread-safe is wrapped in a type that is not
  `Sync`, and its functions take that handle.
- **FFI-4. Attenuation is the wrapper's job.** Foreign code ignores Ferox
  attenuation, so a wrapper that gives foreign code a path must check it.
  The preferred form passes an open descriptor instead: `fs.open(p)?` returns
  a `File`, and the wrapper passes `file.as_raw_fd()`.
- **FFI-5. Blocking.** Inside a wrapper that takes capabilities, a foreign
  call runs under the evented runtime on the blocking pool while the task
  waits, unless the `extern` item is marked `#[nonblocking]`, in which case
  it runs inline on the worker. Calls from `#[trusted_pure]` wrappers run
  inline, like any authority-free code.

### 5.2 Simulation of foreign effects

A wrapper that takes capabilities may name a model,
`#[sim_model(path::to::model_fn)]`: a Ferox function with the same signature,
written against simulated capabilities. The sim build lists every reachable
wrapper without one (`FX0514`); `ferox sim` refuses to run while any remain
unless `--allow-unmodeled` is given, which marks the run non-replayable.
`#[trusted_pure]` calls need no model; the determinism self-check (§9.7)
catches some false claims.

### 5.3 Callbacks

- **FFI-6.** A Ferox function or closure passed to foreign code must be
  authority-free (`FX0513`). It therefore never suspends, so no foreign frame
  is ever live across a suspension point, and a task can migrate between
  workers without moving foreign frames (`reviews/round1-codex.md` B3).
- **FFI-7.** Panics never unwind through foreign frames. The std adapter
  `ffi::callback(f)` catches a panic, returns the foreign API's sentinel, and
  re-raises the panic after the foreign call returns; a panic escaping a raw
  `extern "C" fn` aborts. A callback that needs effects records them in memory
  it was given, and the wrapper performs them afterwards.

### 5.4 `ThreadBound`

`ThreadBound` marks values tied to one OS thread: foreign handle types
declared `#[thread_bound]`, and imported Rust types that are not `Send`
(§11). A `ThreadBound` type is neither `Send`, `Sync`, nor `Pure`. Under the
evented runtime the compiler computes, from liveness, where a
`ThreadBound` value is live across a suspension point and pins the task to
its current worker over that range; a pinned task is never stolen, and a
foreign call taking a `ThreadBound` argument runs inline rather than on the
blocking pool. Under the other runtimes tasks never migrate, so pinning is a
no-op.

## 6. Tasks

### 6.1 Scopes

The origin structure is `02-memory.md` §8.2; the names here are normative.

```ferox
impl Tasks {
    pub fn scope<T, E, F>(&self, body: F) -> Result<T, E>
        where E: From<TaskError> + Send, F: FnOnce(Scope<E> from self) -> Result<T, E>;
    pub fn shield<T>(&self, body: impl FnOnce() -> T) -> T;
    pub fn checkpoint(&self) -> Result<(), Cancelled>;
}
impl<E> Scope<E> {                        // Copy; origin σ
    pub fn spawn<U: Send, F>(self, f: F from self) -> Task<U, E>
        where F: FnOnce() -> Result<U, E> + Send;
    pub fn set_deadline(self, at: Instant);
    pub fn cancel(self);
}
impl<U, E> Task<U, E> {
    pub fn join(self) -> Result<U, E>;    // shielded
}
pub enum TaskError { Panicked(String), Cancelled, DeadlineExceeded }
```

Children may borrow anything that outlives the `scope` call. A scope passed
down (`fn serve(bg: Scope<E>, ..)`) takes children that own their data or
borrow what the signature ties to it. On the scope's `Err` path every place
lent mutably into it is treated as moved out (D5, `02-memory.md` §8.3).

### 6.2 Failure, cancellation, and deadlines

- **TASK-1. Failure.** A child fails when it returns `Err` or panics. The
  body fails when it returns `Err`; a panic in the body abandons the parent
  task after its children are cancelled and joined.
- **TASK-2. First failure wins.** The first failure (in real time, or in
  scheduler order under the simulator) cancels the scope, its body, and its
  nested scopes. `scope` returns after every child has finished, with that
  failure converted into `E`; later failures go to one `debug!` record.
- **TASK-3. Level-triggered.** Once a scope is cancelled, every checkpoint
  inside it, in any task, returns its cancellation error immediately, and
  keeps doing so. Operations that consume a value return it inside the error
  (`SendError(v)`). An in-flight kernel operation is cancelled before the
  error returns. A cancelled write may already have reached the peer.
- **TASK-4. Deadlines.** `set_deadline(t)` lowers the scope's deadline to
  `t`; a nested scope's effective deadline is the minimum over its
  ancestors. When it passes, the scope is cancelled with failure
  `TaskError::DeadlineExceeded`, even if the body later returns `Ok`. When a
  scope deadline and an operation's own deadline fall on the same instant,
  the scope's cancellation is observed.
- **TASK-5. What cancellation cannot interrupt.** Authority-free
  computation, `lock` acquisition and bodies, foreign calls, and joins.
  Deadlines are therefore soft. The evented runtime logs a `warn!` record
  for a task that runs more than 100 ms (configurable) without reaching a
  checkpoint; the simulator reports the same condition by step count.
- **TASK-6. Shielding.** Operations inside `tasks.shield(|| ..)` ignore the
  cancellation of enclosing scopes; bounded cleanup is a shielded scope with
  its own deadline. `Drop` in std does no I/O beyond closing descriptors; a
  buffered writer dropped unflushed logs a `warn!`. Cleanup that needs I/O is
  an explicit `close(self)` called under `shield`.

### 6.3 Joins

- **JOIN-1.** Scope exit joins every child, is shielded, and cannot be
  cancelled. Children borrow the parent frame, so this is a memory-safety
  rule.
- **JOIN-2.** `Task::join` is shielded too: it is not a checkpoint and
  returns only when the child has finished. It returns the child's `Ok`
  value, or `Err(TaskError::Cancelled.into())` if the child failed or was
  cancelled; the scope reports the actual failure. A join waits no longer
  than the child, and in a cancelled scope the child is itself cancelled.
- Dropping a `Task` without joining is allowed; its failure still reaches the
  scope.

### 6.4 Channels

Channels follow Rust's `std::sync::mpsc`: `channel()` is unbounded and
`sync_channel(n)` bounded; `Sender` is `Clone`; a `Receiver` iterates until
every sender is dropped. `send` on a full `sync_channel` and `recv` are
checkpoints. `recv_deadline(t)` returns `Timeout` at `t`. Payloads must be
`Send`. An unbounded `channel()` in a library is lint `FX0608`.

### 6.5 `lock`

```ferox
lock (e1, .., en) |x1, .., xn| body        // ei: &Shared<Ti>, xi: &mut Ti
```

- **LOCK-1. Payloads.** `Ti: Send + Pure` (`FX0606`). A cell never holds a
  capability, channel end, or `Shared` handle.
- **LOCK-2. Identity.** Before acquiring anything, the runtime compares the
  cells' identities. If two operands name the same cell, `lock` returns
  `Err(LockError::Aliased)` and the body does not run.
- **LOCK-3. Order.** Cells are acquired in a global order by cell id, a
  process-unique counter assigned by `Shared::new` and deterministic under the
  simulator. Acquisition waits per cell in FIFO order. It is not a checkpoint:
  a task in a cancelled scope still acquires, runs the body, and observes
  cancellation at its next checkpoint.
- **LOCK-4. Body.** The body is checked as a closure: `return` and `?` leave
  the body; captures must be `Pure`; calls must be to authority-free
  functions; it contains no `lock` (`FX0603`). It cannot suspend, touch a
  capability, or wait for a cell another task can hold. Sinks are allowed;
  printing is lint `FX0607`.
- **LOCK-5. Result.** `lock` evaluates to `Result<R, LockError>`, where `R` is
  the body's type and `LockError` is `Aliased` or `Poisoned`.
- **LOCK-6. Poisoning.** A panic in the body poisons every cell it held. A
  later `lock` naming a poisoned cell returns `Err(Poisoned)` without running
  its body. `cell.reset(v)` replaces the payload and clears the poison; no
  safe API reads a poisoned payload.
- **LOCK-7. Invariants.** A cell's invariant is its payload type's
  invariant. The body may assume it at entry and must restore it at exit
  (`04-contracts.md` §4.4).
- **LOCK-8. Dynamic sets.** `sync::lock_all(&cells, |xs: &mut [&mut T]| ..)`
  locks a slice of cells under the same rules. Its body closure must be
  authority-free, and `&[&Shared<T>]` is not `Pure`, so it cannot be named
  inside another lock body.

`lock` cannot deadlock against `lock`: no body acquires, every acquisition
takes its whole set in one global order, and every body finishes without
suspending unless it diverges. Programs can still deadlock through channels;
the simulator reports a global deadlock with the blocked-task graph (§9.3).
Condition waits are channels; reader-writer and hand-over-hand locking live
in `std::sync::raw`.

### 6.6 `Send`, `Sync`, and task isolation

`Send` and `Sync` are derived structurally as in Rust (`02-memory.md` §8.1).
They are checked at task boundaries only: spawn captures, child results and
errors, channel payloads, `Shared` payloads, and `par` branches. A value that
never crosses a boundary need not be `Send`, even when the evented runtime
migrates its task between OS threads. That is sound because safe code reaches
no thread-local state (GLOBAL-3), the scheduler's hand-off orders the task's
accesses, `ThreadBound` values pin (§5.4), and no foreign frame is live
across a suspension point (FFI-6). `par` branches run simultaneously and so
require `Send`.

## 7. Compiling Suspending Functions

### 7.1 Which functions may suspend

- **SUSP-1.** A function instance is classified may-suspend if and only if,
  after monomorphization, some parameter or capture type is not `Pure`. A call
  through a closure or `dyn` value may suspend exactly when that value's
  type is not `Pure`. This is the semantic rule and the interface flag;
  authority-free code never suspends.
- **SUSP-2.** Generic code is classified per instance. `map(xs, |x| f(net, x))`
  gets a may-suspend instance because the closure captures `&Net`;
  `map(xs, |x| x + 1)` gets an ordinary one.
- **SUSP-3.** As a code-generation refinement, the compiler may compile an
  instance as non-suspending when its body, transitively, contains no
  suspending operation. The refined flag of a public item is codegen data in
  the interface: a change regenerates dependents' code but does not re-check
  them.

### 7.2 Lowering under the evented runtime

- Each may-suspend instance becomes a **frame record** (the locals live
  across suspension points, plus a state index) and a **resume function**
  that switches on the state.
- Frames are allocated LIFO in the task's **frame stack**: a list of heap
  chunks that starts at 1 KiB and grows by doubling. Frames never move, so a
  reference into a frame, including a child's borrow of its parent's locals,
  stays valid across suspension without `Pin`.
- Without recursion or may-suspend `dyn` calls below a spawn, the first
  chunk is sized to the deepest frame chain exactly. Recursive calls allocate
  in the chunk list; a task over its cap (default 8 MiB) panics as on stack
  overflow.
- Calls to non-suspending functions are ordinary calls on the worker's native
  stack, so native frames are never live across a suspension point.
- A suspending leaf registers with the reactor and returns `Pending`. The
  scheduler later resumes the innermost frame directly; a finished frame
  writes its result into its caller's frame and resumes it, so suspension
  costs O(1) regardless of depth.
- A may-suspend `dyn` method's vtable entry records its frame size, and the
  caller allocates that frame on the frame stack.
- The runtime walks the frame chain for logical backtraces with spans.

Cost model, to be checked by E5: a suspension is a return to the scheduler
plus one indirect call, a may-suspend call is a bump allocation, and an idle
connection handler holds tens to hundreds of bytes.

### 7.3 The other runtimes and foreign code

Under `blocking` and `threads`, every function is compiled as an ordinary
function and suspension blocks the OS thread. The simulator uses either
lowering (§9.1). Foreign code calls only authority-free Ferox code (FFI-6),
which is never lowered, and wrapper calls that block run on the blocking pool
while the task's frame waits (FFI-5). One package compiled to mid-IR serves
every runtime; the instantiation cache keys may-suspend instances by
lowering.

## 8. Deterministic Parallelism

### 8.1 Forms and admission

`par { e1, .., en }` yields a tuple; `join(f, g)` is its function form; slices
offer `par_map`, `par_for`, `par_chunks_mut`, `par_reduce`, and `par_sort`
(stable). Each branch or closure must be authority-free and `Send`
(`FX0610`), may capture shared data only if it is `Frozen + Sync`
(`FX0611`), and may hold disjoint `&mut` splits. Branches may borrow the
enclosing frame, because `par` joins before continuing. A `par` branch is not
a task: it has no checkpoints and cannot be cancelled.

### 8.2 Semantics

- **PAR-1.** A `par` region's values, its writes through `&mut`, and its
  sink output are identical to sequential evaluation with branches left to
  right and `par_reduce` defined by the canonical grouping G(n) (§8.3), on any
  core count, runtime, or schedule. Two qualifications: resource exhaustion,
  and divergence (PAR-2).
- **PAR-2.** If several branches panic, the panic first in sequential order
  is reported; a later branch's panic waits for every branch to its left. A
  panicking branch sets a stop flag that branches to its right check at their
  forks and block boundaries. If a branch to the right of the first panic
  diverges without reaching one, the region may diverge where the sequential
  program would have panicked.
- **PAR-3.** Floating-point code is never contracted to FMA or reassociated
  unless written so (`mul_add`), and `to_bits` canonicalizes NaN (D3).
  The optional automatic parallelization pass only runs independent
  authority-free calls in parallel; it never regroups a fold.

### 8.3 `par_reduce` and the canonical grouping

`xs.par_reduce()` takes a slice of `M: Combine` (`reference.md` §5.3). It is
**defined** as `tree_fold(G(xs.len()), xs)`, where G is:

1. An empty slice reduces to `M::empty()`.
2. The input is cut into blocks of B = 2048 elements; the last may be short.
3. A block is folded with 8 strided accumulators: accumulator j left-folds
   the elements at offsets ≡ j (mod 8), and the non-empty accumulators are
   combined as `((a0 ⊕ a1) ⊕ (a2 ⊕ a3)) ⊕ ((a4 ⊕ a5) ⊕ (a6 ⊕ a7))`. This
   fixes the lane count on every target, so SIMD leaves are reproducible.
4. Blocks `[a, b)` with more than one block split at `a + 2^k`, the largest
   power of two less than `b - a`, and the halves are combined left to right.

G depends only on the length. B and the lane count are part of the language
version. If `M: Monoid` and its associativity law is proved, the result also
equals `fold`; with a tested law, that equality is a counted assumption
(`04-contracts.md` §5). `Sum<f64>` is only `Combine`, so a float sum is G's
grouping. A checked integer `Sum` traps if any partial sum in G overflows,
which can differ from the sequential fold; `Sum<i128>` or wrapping
arithmetic avoids that.

### 8.4 Sinks and scheduling

Each branch writes sink records to its own buffer; the join appends the
buffers in branch order (SINK-4). `par` runs on one process-wide
work-stealing pool sized by `par-threads`, shared with evented workers and
reduced by any `Accel` budget. An unpromoted fork costs about a branch. v1
promotes by lazy binary splitting: a fork exposes its right branch only when
the local deque runs low. Heartbeat promotion, checked at forks and block
boundaries so loops need no polling, replaces it if E5 justifies it.
Scheduling is unobservable, so the policy can change freely. Under the
simulator, `par` runs on real cores.

## 9. Deterministic Simulation

### 9.1 Engines

A simulation is defined by its scheduler decisions, so three engines can run
it: baton-passed OS threads, one runnable at a time (default for
`ferox test`); the evented lowering on one thread (default for long
`ferox sim` runs); and the interpreter. Decisions happen at the same points
in each, so a seed yields one trace in all three; conformance tests check it.

### 9.2 `Root::sim`

`Root::sim(seed)` returns a `SimRoot` with the same capability fields as
`Root`, all simulated, plus `sim.world` for setup (`file`, `args`,
`env_var`, `stdin`, `http` peers, `proc` models), `sim.faults` (§9.4),
`sim.node` (§9.5), and `sim.elapsed()`, `sim.records()`, and `sim.trace()`
for assertions.
The test body runs as the simulation's main task. `#[sim_test(seeds = N)]`
gives a test a fresh `SimRoot` per seed; `ferox test` runs 100 seeds by
default and `ferox sim --test` runs more.

### 9.3 Time and scheduling

- Monotonic time starts at zero, wall time at 2000-01-01T00:00:00Z.
  Computation takes no virtual time; capability operations take modeled
  latencies drawn from the seed.
- Virtual time advances only when every task on every node is blocked, and
  then jumps to the next timer or delivery. If no task can ever run again,
  the run fails with a global deadlock report: each blocked task, the span of
  its blocking operation, and what it waits for.
- **Scheduling points** are every capability operation, every handle
  operation (`spawn`, `join`, `send`, `recv`, `lock` acquire and release,
  atomics, `Weak::upgrade`), and every checkpoint. Between two points a task
  runs alone. Because safe code is data-race-free, interleavings at
  synchronization operations reach every observable outcome, so this
  granularity loses timings, not behaviors.
- Strategies: `random` (default), `pct(depth)` (probabilistic concurrency
  testing), and `fifo` for debugging.

### 9.4 Fault injection

| `sim.faults` method | Effect |
| --- | --- |
| `net_drop(p)`, `net_duplicate(p)`, `net_delay(min, max)`, `net_reorder(on)` | per-message faults |
| `partition(&[&a], &[&b, &c])`, `heal()` | cut links between node groups; connections time out or reset |
| `disk_error(prefix, p, kind)` | `EIO`, `ENOSPC`, and similar on matching paths |
| `torn_writes(on)` | a crash keeps a seed-chosen prefix of unsynced writes |
| `clock_skew(&node, ppm)` | per-node clock drift |
| `crash(&node)`, `restart(&node)` | §9.5 |
| `at(t, fault)` | schedule any fault at virtual time `t` |
| `fault_point(name, p)` | probability that `tasks.fault_point(name)` returns true |
| `accel_crash(p)` | Bend kernel failures (§10) |

`ferox sim` defaults to swarm testing: each seed enables a random subset of
faults at random rates.

### 9.5 Nodes

`sim.node("b")` creates a node with its own filesystem, network address
(`"b"` resolves to it), clock skew, environment, and task tree.
`node.start(entry)` runs `entry: FnOnce(NodeRoot) -> Result<(), E>`, a
closure that is `Owned + Send` and captures only `Frozen + Pure` data; a
`NodeRoot` has `Root`'s capability fields. Nodes share no memory and
interact only through the simulated network and filesystem.

A crash is not a cancellation: the node's tasks stop at their current
scheduling points without running destructors, its memory is freed, only
synced file data survives (subject to `torn_writes`), and peers'
connections reset after a modeled delay. `restart` reruns the entry over the
surviving disk. `sim.run_for(d)` and `sim.run_until(&handle)` drive the
world; `node.fs()` inspects a node's disk.

### 9.6 `ferox sim`

```text
ferox sim [<bin> -- args | --test <name>] [--seeds N | --seed S] [--time 10m]
          [--strategy random|pct] [--faults swarm|none|<plan.toml>]
          [--minimize] [--allow-unmodeled] [--json]
```

`<bin>` runs `main` as a single node with a simulated root. Output is one
NDJSON record per failing seed and a summary:

```json
{"kind":"sim_failure","target":"kv::tests::acked_writes_survive","seed":3141,
 "commit":"e9d8ed4","toolchain":"ferox 0.4.1","virtual_time":"0.704s","steps":18342,
 "failure":{"type":"panic","message":"acked k17 missing on backup",
            "span":"src/kv.fx:121:13","node":"client","task":"0"},
 "faults":["partition#1 at 0.200s"],
 "replay":"ferox sim --test kv::tests::acked_writes_survive --seed 3141",
 "trace":"target/sim/acked_writes_survive-3141.ndjson","trace_hash":"9f2c…"}
```

Each trace event records virtual time, node, task, kind (`cap`, `sched`,
`fault`, `sink`, `spawn`, `exit`, `cancel`, `lock`, `chan`, `assert`), the
operation with its source span, its arguments, and its result. A seed
replays bit for bit given the same commit and toolchain. `--minimize`
delta-debugs the fault plan and the scheduler's preemptions toward a shorter
failing trace.

### 9.7 Determinism self-check

The first seed of every run, and every failing seed, runs twice and the
trace hashes are compared. On a mismatch `ferox sim` reports the first
diverging pair of events with spans. The usual causes are an unmodeled
foreign effect, a false `#[trusted_pure]` claim, unsafe code observing
addresses, or a toolchain bug.

### 9.8 Coverage and limits

The replay guarantee holds only when every reachable foreign effect has a
model (D15). The sim build lists the ones that do not (`FX0514`, §5.2). For
imported Rust crates, sim builds interpose the libc entry points for time,
randomness, and environment and route them to the simulator; file and socket
calls from an unmodeled crate fail with `EPERM`. Runs that use
`--allow-unmodeled` are marked non-replayable.

Not simulated: CPU time and performance; memory exhaustion; stack depth;
the interleaving inside `par` (its results are deterministic anyway);
hardware faults beyond injected disk errors; foreign code's internals and
threads; kernel behavior beyond the models; signals; timing side channels;
real child processes, which need a `sim.world.proc` model.

## 10. The Bend Boundary

```toml
[bend-dependencies]
life = { bend = "=2.0.37", src = "kernels/", mode = "process" }  # "inproc" needs grants = ["ffi"]
```

```ferox
#[bend(dep = "life", def = "Life.steps", reference = life_steps_ref)]
pub fn life_steps(accel: &Accel, board: BendArray<u32>, n: u32) -> Result<BendArray<u32>, BendFault>
    ensures result.is_ok() ==> result.unwrap().len() == board.len()
        by bend_law("kernels/LAWS.bend", "steps_keep_size");
```

- **Build.** The pin is exact. The build runs the pinned `bend`, caches by
  the hash of sources, laws, proofs, and version, and generates a shim that
  calls the def at exactly the declared types, so Bend's checker rejects a
  mismatch. Defs that are IO-typed or depend on foreign or `@unsafe` defs are
  rejected. An unsupported host gets one diagnostic, or the `reference`.
- **`Accel`.** Calls require `&Accel`, which only `root.accel` provides. In
  process mode the supervisor runs at most `budget` kernel servers; in-process
  mode serializes calls, because Bend's runtime is process-global. The budget
  is subtracted from the `par` pool. A call is a checkpoint; cancelling it
  abandons the request and restarts the server.
- **Process protocol.** The kernel server listens on loopback TCP, since Bend
  has no stdin. A request is `FXB1 <def> <schema-hash> <len>` followed by
  the arguments in a generated S-expression grammar; the reply is
  `OK <value>` or `ERR <code> <text>`. Values are copied; closures,
  references, capabilities, and handles never cross.
- **Type map** (`research/11-bend-interop.md` H2). `u32`, `f32`, `bool`,
  `()`, `Option`, `Result`, and first-order enums and structs of these copy
  across. `BendArray<u32 | f32>` pads to a power of two with the logical
  length passed. `Vec<T>`, `String`, and `[u8]` copy to lists. `BendNat` is
  range-checked below 2^48. 64-bit types are compile errors for now. Bend
  arithmetic wraps; Ferox's checks stop at the boundary.
- **Failures.** `BendFault` is `Crashed { code, stderr_tail }`,
  `FailStop(n)`, `Limit(what)` (from pre-validation of Bend's known limits),
  `Protocol` (version or schema mismatch), or `Unavailable`. A kernel that
  returns Bend's `Result` is declared `-> Result<Result<T, E>, BendFault>`.
- **Simulation.** Kernel results are pure, so under the simulator the call
  runs the `reference` implementation if one is declared, and otherwise the
  real kernel outside virtual time, recording the result's hash. Latency is
  modeled, and `accel_crash` exercises the failure paths.
- **Imported claims.** A `by bend_law` clause is accepted only if Ferox can
  render it as a Bend law (first-order fragment), the pinned `bend` verdict
  passes, and the verdict lists no foreign or `@unsafe` dependency. The claim
  enters the trust surface as `external: bend` with version and hashes in the
  lockfile. It never counts as Ferox-proved; a Ferox proof may use it only as
  a counted assumption. Under `assert` and `test` the clause is still checked
  at run time where executable.
- **Differential tests.** With a `reference`, `ferox test` property-tests the
  kernel against it; deterministic `par` makes exact comparison meaningful.

## 11. Imported Rust Crates

```toml
[rust-dependencies]
regex = { version = "1.11", authority = [] }
rusqlite = { version = "0.32", authority = ["fs.read", "fs.write"], build = true }
```

- **RUST-1. Shims.** Ferox generates explicitly instantiated C-ABI shims
  for the synchronous APIs it uses; async, macro-driven, and lifetime-heavy
  APIs need a hand-written Rust shim.
- **RUST-2. Authority.** With `authority = []`, every wrapper is
  `#[trusted_pure]` and carries FFI-3's claims. Otherwise every free function
  and constructor wrapper takes a capability for each declared kind, and the
  opaque types they return are resource handles: not `Pure`, carrying that
  authority, so their methods need no further parameters. Per-item overrides
  narrow this (`items."Connection::open_in_memory" = []`).
- **RUST-3. Symbol scan.** After the build, the linked objects' symbols are
  mapped to kinds: `open`, `stat`, `unlink` to `fs`; `socket`, `connect`,
  `getaddrinfo` to `net`; `clock_gettime` to `clock`; `getrandom` to `rand`;
  `getenv` to `env`; `fork`, `execve`, `posix_spawn` to `proc`;
  `pthread_create` to `tasks`; `dlopen` and raw `syscall` to `foreign`. A
  kind found but not declared fails the build (`FX0515`) with the symbol and
  object.
- **RUST-4. Threads and panics.** Rust types that are not `Send` import as
  `ThreadBound`; `Send` and `Sync` otherwise carry over. Panics are caught in
  the shim and re-raised in the calling task. Callbacks follow FFI-6.
- **RUST-5. Build scripts** run only with `build = true`, in a sandbox: no
  network, read-only sources, a writable output directory only, a cleared
  environment, and CPU and time limits. Outputs are hashed into the
  lockfile, a rebuild with different output warns, and each script is a
  counted trust hole (D13). The crate's proc macros run in the same sandbox.

What this cannot guarantee: a present symbol may be unreachable, and the
scan misses inline-assembly syscalls and inherited descriptors; Rust code
ignores attenuation (FFI-4); Rust `unsafe` may be unsound; simulation through
libc interposition is best-effort; a build script can emit malicious code
into its outputs; threads a crate spawns escape Ferox scheduling. The trust
surface marks each crate `rust: declared, scan-checked, unverified`.

## 12. Worked Examples

### 12.1 A command-line tool

```ferox
#[derive(Debug, Error)]
enum CliError {
    #[error("usage: lc <file>...")]
    Usage,
    #[error("{0}")]
    Io(#[from] io::Error),
}

/// Exercised authority: fs.read through `fs`.
fn count_files(fs: &Fs, paths: &[String]) -> Result<Vec<usize>, CliError> {
    paths.iter().map(|p| Ok(count_lines(&fs.read(p)?))).collect()
}

/// Authority-free.
fn count_lines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

fn main(root: Root) -> Result<(), CliError> {
    let paths: Vec<String> = root.env.args().skip(1).collect();
    if paths.is_empty() {
        return Err(CliError::Usage);
    }
    let fs = root.fs.read_only();
    for (path, n) in paths.iter().zip(count_files(&fs, &paths)?) {
        println!("{n:>8} {path}");
    }
    Ok(())
}

#[test]
fn counts_lines() {
    let sim = Root::sim(1);
    sim.world.file("a.txt", "x\ny\n");
    assert_eq!(count_files(&sim.fs, &[String::from("a.txt")]).unwrap(), vec![2]);
}
```

`println!` needs no capability; review shows `count_files` exercising only
`fs.read` through a read-only `Fs`.

### 12.2 A concurrent fetcher with a deadline

```ferox
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("task: {0}")]
    Task(#[from] TaskError),
}

/// A per-URL failure is a value; only the deadline or a panic fails the call.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url], budget: Duration)
    -> Result<Vec<Result<Vec<u8>, NetError>>, FetchError>
{
    let deadline = clock.now() + budget;
    tasks.scope(|s| {
        s.set_deadline(deadline);
        let pending: Vec<_> = urls.iter()
            .map(|u| s.spawn(move || Ok(net.get(u, deadline))))
            .collect();
        pending.into_iter().map(|t| t.join()).collect()
    })
}

#[test]
fn slow_host_hits_the_deadline() {
    let sim = Root::sim(7);
    sim.world.http("fast.test:80", |_| Reply::ok("a"));
    sim.world.http("slow.test:80", |_| Reply::ok("b").after(Duration::from_secs(30)));
    let urls = [Url::parse("http://fast.test/").unwrap(), Url::parse("http://slow.test/").unwrap()];
    let r = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls, Duration::from_secs(5));
    assert!(matches!(r, Err(FetchError::Task(TaskError::DeadlineExceeded))));
    assert_eq!(sim.elapsed(), Duration::from_secs(5));   // virtual time; the test takes milliseconds
}
```

Children borrow `urls` and `net` from the caller. At five virtual seconds
the scope deadline fires (TASK-4), the slow child's `get` returns
`NetError::Cancelled` as a value, both joins return, and `scope` reports
`DeadlineExceeded`.

### 12.3 A replicated key-value node under a partition

```ferox
pub struct Node { pub net: Net, pub fs: Fs, pub clock: Clock, pub tasks: Tasks, pub backup: Option<String> }

pub fn serve(node: &Node, addr: &str) -> Result<(), ServeError> {   // ServeError: #[from] TaskError, io::Error, ..
    let listener = node.net.bind(addr)?;
    node.tasks.scope(|s| loop {
        let conn = listener.accept()?;
        s.spawn(move || {
            if let Err(e) = handle(node, &conn) {
                info!("connection closed: {e}");   // one bad connection must not stop the server
            }
            Ok(())
        });
    })
}

fn handle(node: &Node, conn: &TcpStream) -> Result<(), ServeError> {
    while let Some(Msg::Put { key, val }) = frame::read(conn)? {
        let reply = match put(node, &key, &val) {
            Ok(()) => Msg::Ack,
            Err(e) => { warn!("put {key}: {e}"); Msg::Fail }
        };
        frame::write(conn, &reply)?;
    }
    Ok(())
}

/// Acknowledge only after the write is durable here and on the backup.
fn put(node: &Node, key: &str, val: &str) -> Result<(), ServeError> {
    node.fs.write_synced(key, val.as_bytes())?;
    let Some(backup) = &node.backup else { return Ok(()) };
    node.tasks.scope(|s| {
        s.set_deadline(node.clock.now() + Duration::from_millis(500));
        let conn = node.net.connect(backup)?;
        frame::write(&conn, &Msg::Put { key: key.into(), val: val.into() })?;
        match frame::read(&conn)? {
            Some(Msg::Ack) => Ok(()),
            _ => Err(ServeError::NotReplicated),
        }
    })
}

#[cfg(test)]
mod tests {
    fn node(r: NodeRoot, backup: Option<&str>) -> Node {
        Node { net: r.net, fs: r.fs, clock: r.clock, tasks: r.tasks, backup: backup.map(String::from) }
    }

    #[sim_test(seeds = 500)]
    fn acked_writes_survive(sim: SimRoot) {
        let backup = sim.node("backup");
        let primary = sim.node("primary");
        backup.start(|r| serve(&node(r, None), ":7000"));
        primary.start(|r| serve(&node(r, Some("backup:7000")), ":7000"));
        sim.faults.at(Duration::from_millis(200), Fault::partition(&[&primary], &[&backup]));
        sim.faults.at(Duration::from_secs(2), Fault::heal());
        let client = sim.node("client").start(|r| write_keys(&r.net, "primary:7000", 50));
        let acked = sim.run_until(&client).unwrap();      // keys answered with Ack
        for key in &acked {
            assert!(backup.fs().exists(key), "acked {key} missing on backup");
        }
    }
}
```

During the partition the replication deadline fires, `put` returns `Err`,
and the client gets `Fail`. A version that acknowledged before replicating
fails on some seed with the §9.6 record, and its replay command reproduces the
run exactly.

### 12.4 A bank transfer with `lock`

```ferox
#[derive(Debug, Error)]
pub enum BankError {
    #[error("insufficient funds")]
    Insufficient,
    #[error("balance overflow")]
    Overflow,
    #[error("lock: {0}")]
    Lock(#[from] LockError),
}

pub struct Account { pub balance: u64 }

pub fn transfer(from: &Shared<Account>, to: &Shared<Account>, amount: u64) -> Result<(), BankError> {
    if Shared::same(from, to) {
        return Ok(());                                   // otherwise `lock` returns Err(Aliased)
    }
    lock (from, to) |a, b| {
        let debited = a.balance.checked_sub(amount).ok_or(BankError::Insufficient)?;
        let credited = b.balance.checked_add(amount).ok_or(BankError::Overflow)?;
        a.balance = debited;                             // both checks pass before any write,
        b.balance = credited;                            // so an error leaves both cells unchanged
        Ok(())
    }?
}

#[sim_test(seeds = 1000)]
fn money_is_conserved(sim: SimRoot) {
    let accounts: Vec<Shared<Account>> = (0..4).map(|_| Shared::new(Account { balance: 100 })).collect();
    sim.tasks.scope(|s| {
        for _ in 0..8 {
            let mut rng = sim.rand.rng();
            let accounts = &accounts;
            s.spawn(move || {
                for _ in 0..20 {
                    let (i, j) = (rng.gen_range(0..4), rng.gen_range(0..4));
                    let _ = transfer(&accounts[i], &accounts[j], rng.gen_range(0..50));
                }
                Ok::<(), TaskError>(())
            });
        }
        Ok::<(), TaskError>(())
    }).unwrap();
    let total: u64 = accounts.iter().map(|c| (lock (c) |a| a.balance).unwrap()).sum();
    assert_eq!(total, 400);
}
```

Pairs arrive in both orders; LOCK-3's global order prevents deadlock, and
each seed explores another interleaving. Checking both amounts before any
write means an early `?` never leaves a half-applied transfer.

## 13. Open Questions and Required Changes

### 13.1 Open questions

1. Read-mostly data and counters: `lock (read a)`, snapshot cells, sharded
   counters (`reviews/round1-performance.md` N3).
2. Whether one enum through one channel replaces `select` well enough.
3. How a library exported through the C ABI, called from C, Rust, or Bend,
   receives capabilities; probably a generated init entry that builds a
   `Root` for it.
4. The porting cost of GLOBAL-6 (`Arc::get_mut`, `try_unwrap`, counts).
5. FFI-6 excludes callbacks that need effects, such as GUI event loops.
6. Integration tests that need the real filesystem, since ROOT-1 keeps `Root`
   out of tests.
7. Benchmarks for G(n)'s constants, the 100 ms watchdog, and baton-passed
   threads in large simulations.
8. Whether read versus write should be a type distinction on `Fs`.

### 13.2 Changes this chapter needs elsewhere

- `02-memory.md` §8.4 and `reference.md` §7.3 say duplicate `lock` operands
  panic and `lock` returns `Result<R, Poisoned>`. D8 (draft 2.1) says
  `Err(Aliased)`; this chapter follows D8 with `LockError`, and those two
  documents should change.
- `reference.md` §7.1: `TaskError` gains `DeadlineExceeded`, `join` is
  shielded (JOIN-2), and `Scope<E>` is passed by value as in `02-memory.md`
  §8.2.
- D9 says "heartbeat scheduling"; v1 specifies lazy binary splitting, with
  heartbeat promotion at forks gated on benchmarks (§8.4). D9 should also
  carry PAR-2's divergence qualification.
- D7's capability list should include `Accel`, as `02-memory.md` §9 already
  does.
