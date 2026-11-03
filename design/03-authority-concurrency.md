# Authority, Concurrency, and Simulation

Status: draft 1 of the normative chapter for D7, D8, D9, D15, D16, and the
Rust-crate part of D13 in `design/01-design.md`, synchronized with its draft
3.7, which applies D18: every Tide program is a Rust crate that rustc builds
against Tide's std. That document owns the decisions; this chapter turns them into rules a
compiler, runtime, and simulator can implement. Where a rule needs a decision
the design does not state, this chapter says so and lists it in §13.

**Tier** (01's Tiers table). v0 core: §0–§3; declared authority and the trust
diff (§4.1, §4.3–§4.5); admission (§2.5); §6; the may-suspend flag (§7.1); `join`, `par_map`,
and `par_for` (§8.1, §8.2, §8.4); the simulator with crash, restart, and
durability (§9); the worked examples (§12). v1: exercised authority,
effect reporting, and `FX0510` (§4.1, §4.2, SINK-9); FFI, `#[trusted_pure]`
on foreign functions, and `ThreadBound` (§5); the evented runtime and its lowering (§7.2);
`par_reduce` (§8.3); imported Rust crates and Rust consumers (§11, RUST-1
to RUST-6). Later: the Bend boundary and
`Accel` (§10); automatic parallelization. Not tiered by 01 and left out of v0
by `reference-companion.md`: `pct` scheduling and `--minimize` (§9).
Draft 3 removed `Frozen` (now `Pure + Sync`), the `lock` keyword (now
`Shared::with`, §6.5), `par { }` (now `join`, §8), and the moved-out rule (a
child's panic propagates, §6.2).

Related chapters: `02-memory.md` owns origins, `Send`/`Sync`, the typing of
scopes and `Shared::with`, and the full `Pure` table (its §8–§9);
`04-contracts.md` owns verification of `with` closures and `par_reduce`;
`reference.md` is the agent-facing summary. This chapter restates the
definitions it relies on and adds the operational rules. API names marked
provisional in `reference.md` or its companion are fixed here unless noted.

## 0. Terms

- **Capability**: a value of a standard capability type (§1.1) or of a
  resource handle derived from one: a type whose values act on the world
  with no capability argument, because they carry the authority of the
  capability that made them (`File`, `ReadDir`, `DirEntry`, `TcpStream`,
  `TcpListener`, `UdpSocket`, `Command`, `Child`, its pipe ends
  `ChildStdin`, `ChildStdout`, and `ChildStderr`, `Signals`, any other
  such type a capability or handle method returns, and from v1 the opaque
  types that RUST-2 makes resource handles). Every operation that reads
  or changes the outside world, or that observes nondeterminism, is a
  method on a capability.
- **Synchronization handle**: a value through which tasks observe each other's
  timing or cancellation: `Shared<T>`, channel ends, `Scope<'s, 'env, E>`, `Task<'s, U, E>`,
  `Arc<T>`, `sync::Weak<T>`, and atomics. Creating one takes `&Tasks` (D7), so
  authority-free code can neither create nor receive one; sharing it needs a
  spawn.
- **Authority kind**: one element of the closed set `fs.read`, `fs.write`,
  `net`, `clock`, `rand`, `env`, `stdin`, `proc`, `tasks`, `sync`, `accel`,
  `foreign`. Declared and exercised authority (§4) are sets of kinds, each
  with the access path that carries it.
- **Effect**: a sink a function writes to (`out`, `err`, `log`, `trace`,
  `dbg`). Effects are reported beside authority but are not authority.
- **Suspend**: give up the current OS thread or worker until an event
  arrives. Only capability and synchronization operations suspend.
- **Checkpoint**: an operation at which a task observes cancellation. Every
  capability operation that may suspend is a checkpoint, except the shielded
  ones marked in §1.1, such as `File::close`. Joins and `Shared::with`
  acquisition are not checkpoints either (§6).
- **Task**: a unit of concurrent execution with its own cancellation state,
  sink buffers, and, under the evented runtime, frame stack. `main` runs as
  the main task.

- **CAP-1. No ambient operations.** Tide's std has no free function,
  constructor, or method that does what a capability method does. The
  driver reports a path to Rust's ambient one (`std::fs::read`,
  `Instant::now`, `env::args`, `Command::new`), or a call of a timed wait
  such as `Receiver::recv_timeout`, which reads time without a `Clock`, as
  `FX0501`, not as rustc's unresolved name, and so are `Instant::elapsed` and
  `SystemTime::elapsed`, which read the clock too; `recv_deadline(t)` takes an
  `Instant` from a `Clock` (§1.2).

## 1. The Standard Capability Set

### 1.1 Capability types

The standard library reference holds the full signatures. "Suspends" lists
the operations that may suspend; all are checkpoints except those marked
shielded. Every capability error type has a cancellation variant whose
`is_cancelled()` is true (`NetError::Cancelled`, `io::ErrorKind::Cancelled`).

| Type | Main operations | Attenuation | Suspends |
| --- | --- | --- | --- |
| `Fs` | `read`, `read_to_string`, `write`, `write_synced`, `open`, `create`, `open_with(p, &OpenOptions)`, `sync_dir`, `create_dir_all`, `read_dir`, `metadata`, `exists`, `rename`, `remove_file`, `remove_dir`, `remove_dir_all` | `dir(path, Mode::Read \| Mode::ReadWrite)`, `read_only()` | all |
| `File` | `read`, `read_owned`, `write`, `write_owned`, `seek`, `sync`, `set_len`, `close` | none | all; `close` shielded |
| `Net` | `connect(addr)`, `bind(addr)`, `bind_udp(addr)`, `resolve(host)`, `get(&Url, deadline)` | `only(["host:port", ..])`, `no_bind()` | all |
| `ReadDir`, `DirEntry` | iteration, `path`, `file_name`, `metadata`, `file_type` | none | iteration, `metadata`, `file_type` |
| `TcpStream`, `TcpListener`, `UdpSocket` | `read`, `write`, `read_owned`, `write_owned`, `accept`, `send_to`, `recv_from`, `shutdown` | none | all |
| `Clock` | `now() -> Instant`, `system_time()`, `sleep(d)`, `sleep_until(t)` | `monotonic_only()` | `sleep`, `sleep_until` |
| `Rand` | `next_u64`, `gen_range`, `gen_bool`, `fill_bytes`, `shuffle`, `rng() -> Pcg64` | none | none |
| `Env` | `args()`, `var(k)`, `vars()`, `current_dir()` | `only(["HOME", ..])`, `args_only()` | none |
| `Stdin` | `read_line`, `lines`, `read_to_string`, `read_to_end`, `is_terminal` | none | all reads |
| `Proc` | `command(prog) -> Command` (`.output()`, `.status()`, `.spawn() -> io::Result<Child>`), `signals(&[Signal]) -> io::Result<Signals>`, `exit(code)` | `only(["git", ..])` | `output`, `status`, `Child::wait` and `wait_with_output`, pipe-end reads and writes, `Signals::recv` |
| `Tasks` | `scope`, `spawn_isolated`, `shield`, `checkpoint`, `yield_now`, `fault_point`; creating synchronization handles (§1.2) | `limit(n)` (live children, §6.1) | `checkpoint`, `yield_now`, `scope` exit and `spawn_isolated` (shielded) |
| `Accel` (later) | generated Bend kernel calls (§10) | `budget(threads)`, `no_gpu()` | kernel calls |

Notes:

- **Paths.** An `Fs` resolves paths beneath its directory with
  `openat2(RESOLVE_BENEATH)` semantics: `..` and symlinks cannot leave it.
  `root.fs` is rooted at `/`. A `Mode::Read` `Fs` returns `PermissionDenied`
  on writes. Read versus write is not a type distinction; exercised authority
  (§4) classifies each method as `fs.read` or `fs.write`.
- **`exists`.** `fs.exists(p) -> io::Result<bool>` is Rust's
  `std::fs::exists`: `Ok(false)` when the lookup finds no file at `p`
  (`NotFound`, a broken symlink included), and `Err` for any other failure,
  a path that leaves the `Fs` and cancellation included. Like the other
  main `Fs` operations it is a checkpoint, so a `bool` result, as Rust's
  `Path::exists` returns, would have no room for the cancellation error
  (TASK-3), and turning that error into `false` would report a file that is
  there as missing.
- **Append and durability** (D7, D15). `fs.open_with(p,
  OpenOptions::new().append(true).create(true))` opens for append;
  `OpenOptions` is Rust's builder without its ambient `open`. Writes are
  durable only once synced: `file.sync()` (Rust's `sync_all` and `sync_data`
  are accepted spellings) for contents, and `fs.sync_dir(p)` for directory
  entries changed by `create`, `rename`, `remove_file`, `remove_dir`, or a
  new subdirectory. `remove_dir_all(p)` is, as Rust specifies it,
  `remove_file` and `remove_dir` on every path below and including `p`, so
  the whole removal is durable once `sync_dir` of `p`'s parent returns
  (§9.5): nothing below a removed directory is reachable. A cancelled or
  failed `remove_dir_all` can leave part of the tree, since cancellation
  (TASK-3) or an error can stop it between paths. An `Fs` never removes
  its own base: `remove_dir(".")` fails with `InvalidInput`, and
  `remove_dir_all` on a path that names the base empties it and leaves it
  in place (Rust then reports `InvalidInput` for `"."`); `sync_dir(".")`
  makes the emptying durable.
  D7's `Dir::sync_dir` is this method: an `Fs` from `dir()` is the
  directory capability. `write_synced` syncs the file and its entry.
  After a simulated crash, unsynced writes may be lost or torn, but each
  directory-entry change survives whole or not at all, so a `rename` over a
  target leaves it naming the old file or the new one (§9.5). Atomic
  replace is therefore `write_synced` to a temporary, `rename` it over the
  target, and `sync_dir` the parent.
- **Directory listings.** `fs.read_dir(p)` returns `io::Result<ReadDir>`, an
  iterator of `io::Result<DirEntry>`, as in Rust. `DirEntry` keeps Rust's
  `path()`, `file_name() -> OsString`, `metadata()`, and `file_type()`.
  Like a `File`, a `ReadDir` or `DirEntry` is a resource handle that
  carries the authority of the `Fs` that listed it, so it is not `Pure`;
  advancing the iterator, `metadata()`, and `file_type()` are reads through
  that `Fs` and suspend. As Rust's does on Linux, a `ReadDir` ends after
  yielding its first error. These, `Metadata`, and `FileType` are Tide
  std's own types with Rust's methods, so ADMIT-1's ban on I/O methods of
  re-exported std types does not reach them. `OsStr` and `OsString` keep
  Rust's API, which does no I/O.
- **Standard input.** `Stdin`'s reads are inherent methods with `&self`
  receivers: `read_line(&mut String)`, `lines()`, `read_to_string(&mut
  String)`, `read_to_end(&mut Vec<u8>)`, and `is_terminal()`, which Rust
  spreads over `Stdin`, `io::Read`, and `IsTerminal`. `Stdin` and `&Stdin`
  also implement `io::Read`, as in Rust. No call is ambiguous, because
  inherent methods win, but an `io::Read` import used only for these calls
  is unused (`FX0913`, safe fix). `lines()` ends after yielding a
  cancellation error, so `stdin.lines().flatten()` cannot spin in a
  cancelled scope.
- **Network errors.** `NetError` has `Timeout`, `Cancelled`, `Denied` (a
  host outside `only`), `Refused`, `Reset`, `Dns`, `Status(u16)` (a non-2xx
  response), and `Io(io::Error)`. `is_transient()` is true for `Timeout`,
  `Reset`, `Refused`, and `Status` of 408, 429, or 5xx: the failures a
  later identical request can escape. `Cancelled` is not transient,
  because cancellation is level-triggered (TASK-3) and a retry fails the
  same way. `Denied`, `Dns`, and `Io` are not transient either.
- **Owned buffers.** The kernel only ever owns runtime buffers. `read(&mut
  buf)` copies out of one; `read_owned(buf: Vec<u8>)` and `write_owned(buf:
  Vec<u8>)` move the buffer in and return `Result<Vec<u8>, io::BufError>`,
  with the buffer inside the error on failure. Cancellation never leaves the
  kernel writing into unwound Tide memory. `read_owned` appends what one
  read returns, at most `buf`'s spare capacity. It appends nothing at the
  end of input and, as `read` into an empty slice does, when `buf` has no
  spare capacity, so a caller passes a buffer with room, such as
  `Vec::with_capacity(n)`; only then does an empty append mean the end of
  input. `write_owned` writes all of `buf`, as `write_all` does, and
  returns it unchanged. An `io::Error` cannot hold the buffer, so the
  error is `std::io::BufError`, which holds an `io::Error` and the buffer as
  it was passed: a failed read appended nothing, and a failed write returns
  every byte, sent or not. It has the methods of Rust's
  `io::IntoInnerError`, the std error that already returns a value beside an
  `io::Error`: `error()`, `into_error()`, `into_inner() -> Vec<u8>`, and
  `into_parts()`, plus `kind()`, `is_cancelled()`, and `written()`, the
  number of leading bytes of `buf` the kernel accepted before the failure (0
  for a read; TASK-3). It is `Send + Sync + Pure`, implements `Debug`,
  `Display`, and `Error`, and, as `IntoInnerError` does, `From<BufError> for
  io::Error`, so `?` into an `io::Result` drops the buffer.
- **Randomness.** `rand.rng()` returns a seeded `Pcg64`, an ordinary `Pure`
  value. Authority is spent at seeding; drawing from a `Pcg64` is
  authority-free and inlines. `Pcg64::seed_from_u64(n)` needs no capability.
- **Processes.** A child process is not capability-checked, so `Proc` is the
  widest grant in the set and `tide review` lists every new path to it.
- **Process handles.** `Command`, `Child`, its pipe ends, `Stdio`, `Signal`,
  and `Signals` live in `std::process`. They are Tide std's own types, like
  `ReadDir`, so ADMIT-1's ban on I/O methods of re-exported std types does
  not reach them. A `Command` is a resource handle that carries the
  authority of the `Proc` that made it: `output`, `status`, and `spawn`
  start a process with no capability argument, so `fn configure(cmd: &mut
  Command)` is not authority-free. `OpenOptions` differs: it is `Pure` data
  that does nothing until `Fs::open_with` takes it. `Command` and `Child`
  otherwise keep Rust's API, without the ambient `Command::new` (`FX0501`;
  use `proc.command`) and with `wait_with_output`'s receiver below. That API
  includes `Child`'s public fields `stdin: Option<ChildStdin>`, `stdout:
  Option<ChildStdout>`, and `stderr: Option<ChildStderr>`, so
  `child.stdout.take()` ports unchanged; ROOT-4 allows such fields. The pipe
  ends are resource handles with `Proc`'s authority (exercised kind `proc`);
  their reads and writes suspend.
- **`Stdio`.** `Stdio` has `piped()`, `inherit()`, `null()`, and `From` for
  `File` and the three pipe ends; the last three pipe one child into
  another. It also keeps Rust's `From<io::Stdout>` and `From<io::Stderr>`,
  which name the process's own descriptors 1 and 2, as `inherit()` does,
  whatever destination `root.log` gives the sinks: `.stdout(io::stderr())`
  sends a child's output to the parent's standard error, ordered after
  earlier records (SINK-4). It has no other conversion: `From<OwnedFd>`,
  `From<io::PipeReader>`, and `From<io::PipeWriter>` are gone, so every
  other conversion into a `Stdio` starts from a handle. A `Stdio` does
  nothing until a `Command` uses it, so making one takes no capability, but
  it can hold a handle and is therefore not `Pure`.
- **Receivers.** The methods of `Fs`, `Net`, `Clock`, `Rand`, `Env`,
  `Stdin`, `Proc`, `Tasks`, and `Accel` take `&self`. Resource handles keep
  Rust's receivers where Rust has the method: `Command`'s builder methods,
  `output`, `status`, `spawn`, and `Child::wait` take `&mut self`. The one
  exception is `Child::wait_with_output`, which takes `&mut self`, not
  Rust's `self`: it is a checkpoint, and a cancelled call that consumed a
  running `Child` would drop it where the caller could not `kill` and `wait`
  it; an `io::Error` cannot carry it back. A cancelled call leaves the
  `Child` with the caller, to `kill` and `wait` under `shield`; as in Rust,
  the call has already closed `stdin` and taken `stdout` and `stderr`, so
  those fields are `None` and output read so far is lost. A port that binds
  the child without `mut` gets rustc's E0596 and its fix. Of the methods
  Rust lacks, `File::sync`, `read_owned`, `write_owned`, and `Signals::recv`
  take `&self`, and `File::close` takes `self`. `close` is shielded: it
  always closes the descriptor and returns the close's own result, even in a
  cancelled scope, because reporting that result is its only purpose beyond
  `drop`. Returning a cancellation error instead would lose the result.
  Returning the `File` inside one, as `send` returns its payload, would need
  an error type other than `io::Error` and leave the caller a retry that,
  cancellation being level-triggered, fails again unless shielded. Like
  `drop`, it cannot be interrupted (TASK-5).
- **Cancelled `output` and `status`.** Each starts a `Child` the caller
  never sees, so unlike `wait_with_output` it cannot leave one with the
  caller. Once its child has started, a cancelled `output` or `status` tries
  to kill it, as `Child::kill` does, closes its own ends of the child's
  pipes, so a surviving child that writes to them fails (`SIGPIPE` or
  `EPIPE`) rather than blocking on a full pipe, then waits for it to exit,
  shielded, whether or not the kill succeeded, and only then returns its
  `io::ErrorKind::Cancelled` error; output read so far is lost. A call
  cancelled before the spawn starts nothing, and one whose wait completes
  before the cancel takes effect returns its own result (TASK-3). As with
  `Child::kill`, processes the child started are not signalled. Leaving the
  child running, as tokio does without `kill_on_drop`, would let a process
  the call started outlive its scope with no handle left to stop or reap
  it; waiting for it to finish would let a hung child hold the scope past
  any deadline. A caller that wants a child to outlive a cancellation uses
  `spawn` and keeps the `Child`, which, as in Rust, nothing kills on drop.
  The kill can fail: the kernel refuses to signal a setuid child that has
  changed its real and saved user IDs (`EPERM`). The call then logs the
  kill's error in a `warn!` record, waits for the child to exit by itself,
  and returns `Cancelled` as usual; returning the kill's error at once
  would drop the only handle to the child and leave it unreaped. The wait
  cannot be interrupted (TASK-5): a child stuck in an uninterruptible
  system call, as on a network file system, delays it, and a child the
  kill could not stop holds it until it exits.
- **`fault_point(name) -> bool`** returns `false` under every real runtime and
  folds to a constant; under the simulator it is a seed-driven fault trigger
  (§9.4).

### 1.2 Synchronization handles

| Type | Operations | Waits |
| --- | --- | --- |
| `Shared<T>`, `T: Send + Pure` | `Shared::new(v, tasks)`, `clone()` (same cell), `Shared::same(&a, &b)`, `reset(v)`, `with`, `with2`, `with_all` (§6.5) | acquisition, not a checkpoint |
| `Sender<T>`, `SyncSender<T>`, `Receiver<T>`, `T: Send` | `channel(tasks)`, `sync_channel(n, tasks)`, `send`, `try_send`, `recv`, `try_recv`, `recv_deadline(t)`, iteration | `send` on a full `sync_channel`, `recv`, `recv_deadline`; all checkpoints |
| `Scope<'s, 'env, E>` | `spawn`, `set_deadline(t)`, `cancel()` | `spawn` under `Tasks::limit`, a checkpoint (§6.1) |
| `Task<'s, U, E>` | `join` | yes, shielded (§6.3) |
| `Arc<T>`, `sync::Weak<T>` | `Arc::new(v, tasks)`, `clone`, `make_mut`, `Arc::downgrade(&a, tasks)`, `upgrade` | none |
| `std::sync::atomic::*` | `AtomicU64::new(v, tasks)` and the like; sequentially consistent `load`, `store`, `fetch_add`, `compare_exchange` | none |

- **CAP-2.** Every constructor of a synchronization handle takes `&Tasks`
  (D7; the argument position is provisional; `FX0505` without it); cloning
  an existing handle needs nothing.
- **CAP-3.** Weaker atomic orderings, `Mutex`, and `RwLock` live in
  `std::sync::raw`, which requires the `unsafe` grant (`FX0520` without it,
  ADMIT-1).

`Scope` and `Task` come from `tasks.scope`. `Rc` and `Cell` stay
capability-free because they never cross tasks. To wait on several sources,
send one enum through one channel; there is no `select`.

### 1.3 `Root` and the four runtimes

`Root` has public fields `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
`proc`, `tasks`, `accel` (later tier), and `log: LogConfig`. User code cannot
construct it.

The binary's manifest picks the runtime; libraries never choose and compile
the same way for all runtimes.

```toml
[bin]
runtime = "threads"        # threads (default) | blocking | evented (v1)
workers = "cores"          # evented only (v1)
par-threads = "cores"      # the join/par_* pool (§8); 1 makes them sequential
```

The entry point that `#[tide::main]` generates starts the allocator,
runtime, and sinks, builds `Root` with full authority over the real process,
and runs `main(root)` as the main task. Exit statuses follow Rust: 1 after `Err(e)` (printed as
`Error: {e:?}`), 101 after a panic.

| Runtime | Tasks run as | Suspension is | Code generation |
| --- | --- | --- | --- |
| `blocking` | the main thread, plus one OS thread per child, created at spawn and joined at scope exit | a blocking syscall or futex wait | ordinary functions |
| `threads` | one OS thread per task for its whole life (v0); a finished task's thread may be reused | a blocking syscall or futex wait | ordinary functions |
| `evented` (v1) | M:N on per-core workers with work stealing; reactor on io_uring, else epoll, kqueue, or IOCP, chosen at startup | a return to the scheduler | may-suspend functions become state machines (§7) |
| simulator | one task at a time under a seeded scheduler | a scheduling point | baton-passed OS threads; the evented lowering in v1 (§9.1) |

No runtime runs children inline, so a library that spawns is correct under
all four; they differ in cost, not meaning. Because `threads` gives every
live task its own OS thread, blocked tasks never starve a runnable one.
Parallel branches cannot block either (PAR-4), so production cannot
deadlock where the simulator makes progress (D8).
`Tasks::limit(n)` bounds concurrency where a program needs a bound.

Capability types are concrete: production binaries link only the real
implementation, so capability calls are direct and attenuation is a policy
field checked at the system-call boundary (`reviews/round1-performance.md`
N1). Test builds add the simulator behind one predictable branch.

## 2. Rules

### 2.1 `Root`

- **ROOT-1.** `Root` may be named only as the parameter of the binary's
  `main`, written `#[tide::main] fn main(root: Root)`, and in the path
  `Root::sim`, which ROOT-3 confines to test code; any other use is
  `FX0503`. rustc rejects parameters on a plain `main` (E0580); the
  attribute generates Rust's entry point, which mints the `Root`. Every
  binary's `main` carries it, with or without `root`; a `main` without it
  is `FX0512`, whose safe fix adds it.
- **ROOT-2.** `root` cannot be passed, borrowed into a call, stored,
  returned, or captured (`FX0503`). Its fields can be moved or borrowed out
  one by one. Attenuation and wiring therefore happen in `main`, where review
  shows them (§4.3). The fix passes the fields the callee exercises.
- **ROOT-3.** `Root::sim`, `SimRoot`, `NodeRoot`, and everything in
  `std::sim` compile only in test code: `#[cfg(test)]` items, `#[test]` and
  `#[sim_test]` functions, and files under `tests/` (`FX0504`). Under
  `tide sim` the runtime, not user code, builds the simulated root. A
  dependency's test code is never compiled into its dependents, so a
  simulated capability cannot reach production code.
- **ROOT-4.** Capability and handle types cannot be made from data: each has
  a private field, so no struct literal or `Shape` construction (D6) builds
  one, and none has a `Default` or `Decode` impl. A public field holds a
  capability or handle (`Root`'s capability fields), an `Option` of one
  (`Child`'s pipe ends), sink configuration (`Root::log`), or test-only
  simulator controls (`SimRoot`'s `world` and `faults`), so reading one
  reaches only authority the value already carries. Tide's std bounds by
  `Pure` (`FX0517`) the type that each std operation listed here produces,
  under any path that names it: `mem::zeroed` and `transmute`, `core`'s
  too; `MaybeUninit::assume_init` and its `Box` and `Rc` forms;
  `slice::align_to`, which reinterprets bytes as `transmute` does;
  `Vec::set_len`, which makes a buffer's raw memory into elements; and
  raw-pointer reads. A raw-pointer read is a deref (`*p`, `&*p`, `&mut *p`,
  `(*p).field`), a read method of a raw pointer or `NonNull`, or a
  conversion of one to a reference or an owner (`as_ref`, `as_mut`,
  `slice::from_raw_parts`, `Box::from_raw`, `Vec::from_raw_parts`), since
  each hands safe code a value no constructor made. A union's field read,
  by expression or pattern, counts too: it reinterprets another field's
  bytes, as `transmute` does. Assigning to such a place, or borrowing it
  raw (`&raw const (*p).field`), reads nothing. So generic unsafe code that
  reads through a `*const T`, or sets a `Vec<T>`'s length, bounds `T` by
  `Pure`. The bound is defense in depth over this list, not a closed
  guarantee: unsafe code can still forge a capability by writing raw
  memory, so the lockfile marks every package with an `unsafe` or `ffi`
  grant `authority: declared, unverified`.
- **ROOT-5.** No code calls `main` or uses it as a value (`FX0506`), so a
  test cannot mint a real `Root` through it.

### 2.2 No mutable globals, no thread-locals

- **GLOBAL-1.** A `static` must be `Sync + Pure`, and, as in Rust,
  `'static`. There is no `static mut` in safe code. A non-`Pure` static
  and a `static mut` are `FX0502`; rustc reports a missing `Sync` (E0277).
- **GLOBAL-2.** v0 has no lazy statics (D7). A static's initializer is
  evaluated at compile time by `const fn` (D6); a value that needs run-time
  initialization is built in `main` and passed down. Nothing runs on first
  access, so no initializer can print, panic, wait, or differ between tasks
  or simulated nodes. `lazy_static!` is `FX0502`, and Rust's `LazyLock` and
  `OnceLock` are `FX0520` (ADMIT-1).
- **GLOBAL-3.** Safe code has no thread-locals and no task-locals
  (`thread_local!` is `FX0502`). The runtime's per-task state
  (cancellation, deadline, sink buffers, frame stack) is reachable only
  through the operations in this chapter.
- **GLOBAL-4.** Raw pointer addresses are observable only in packages
  granted `unsafe` (D15): raw pointer types are admitted only there
  (ADMIT-1), casts to them included, because their `PartialOrd`, `Ord`,
  `Hash`, and `Debug` impls observe addresses, and so are pointer-to-integer
  casts, `addr`, and `{:p}`. A function pointer's code address is one too:
  its casts to integers, ordering, `Hash`, and `Debug` need the grant, and
  `==` does not. `Hash` on a reference hashes the referent. `Shared::same`
  and `std::ptr::eq` compare identity without exposing it; Tide's `ptr::eq`
  takes `&T` where core's takes raw pointers, so it needs no grant.
- **GLOBAL-5.** `HashMap::new()` uses a fixed key, so iteration order is
  deterministic though unspecified. Maps keyed by untrusted input take a key
  from `Rand`: `HashMap::with_hasher(RandomState::from_rand(&rand))`.
- **GLOBAL-6.** `Arc<T>` and `sync::Weak<T>` are synchronization handles
  (D7), so neither is `Pure` and creating either takes `&Tasks`: the last
  release runs `T::drop` in whichever task drops last, `make_mut` calls
  `Clone` only when another clone exists, and `upgrade` observes other
  tasks' drops. `Arc` also omits `strong_count`, `weak_count`, `get_mut`,
  `try_unwrap`, and `into_inner`. `make_mut` stays, since only code that
  already observes other tasks can hold an `Arc`. `Rc` and `rc::Weak` keep
  the full Rust API; they never leave their task.

What replaces each use of global state:

| Rust habit | Tide |
| --- | --- |
| `lazy_static!`, `LazyLock`, `OnceLock` tables | a `const` or `static` computed by `const fn`, or a value built in `main` and passed by reference (GLOBAL-2) |
| a global cache, memo table, or interner | a value owned by the caller and passed by `&mut`; in concurrent code, a `Shared<Cache>` |
| `#[global_allocator]` | the binary's manifest; allocator thread caches belong to the runtime |
| per-core counters | `metrics` output (§2.4); a `Shared` or atomic when the program must read the value back |
| `thread_rng()` | `rand.rng()` once, then pass the `Pcg64` |
| a global logger | the sinks (§2.4) |
| global configuration | a `Pure + Sync` config value built in `main` and passed by reference |

### 2.3 Resource exhaustion and nested panics

Allocation needs no capability; its failure aborts the process. Every
determinism claim here holds modulo exhaustion of memory, stack, or
`const fn` fuel.

- **PANIC-1. Stack overflow** aborts the process, as in Rust (D5): no
  destructors run and `spawn_isolated` cannot contain it. Under the simulator
  it fails the seed. v0's simulator runs in one process (§9.1), so the abort
  is reported from outside it: `tide test` and `tide sim` run the binary as a
  child process, which may run several tests and seeds at once. Before a seed
  runs, the simulator writes its replay manifest with the seed and build
  identity, and appends the rest of REPLAY-1's contents as the run goes.
  Tide's `#[test]` writes a start record for every test, since libtest reports
  none for tests it runs concurrently. When the child aborts, `tide test` or
  `tide sim` reruns each test and seed that was in flight in a child of its
  own, reports each that aborts alone (a simulated one by seed with its replay
  command, any other by test name), and starts a fresh child for the tests and
  seeds not yet run. The run fails even if no rerun aborts, and then reports
  the abort with the tests and seeds that were in flight. An abort from
  PANIC-2 is reported the same way. The depth at which an overflow occurs is
  outside the determinism claim, so a rerun or replay may not reproduce it.
- **PANIC-2. Panic during unwinding.** A panic raised while a task is
  unwinding from another panic, for example in a destructor, aborts the
  process, as in Rust (D5).

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
- **SINK-3. Arguments.** Format arguments are always evaluated, and the
  record formatted, whatever level is enabled and in every profile; a
  disabled record is then discarded (D7). Configuration therefore cannot
  change results, panics, or state that an argument or a formatting impl
  mutates. The compiler skips formatting a disabled record only where that
  is unobservable. A panic in an argument or formatting impl propagates like
  any other panic; sinks are not a catch point (D5).
- **SINK-4. Order.** Within a task, records appear in program order. Across
  tasks, real runtimes interleave them in emission order and the simulator
  in scheduler order. Inside `join`, `par_map`, and `par_for`, each branch
  writes to its own buffer, and the buffers are appended in branch order at
  the join, so output matches the sequential program (§8.4). The buffers of
  branches after the first panicking one are discarded (PAR-2). A child
  process whose stdout or stderr is the process's own (`Stdio::inherit()`,
  the default for `spawn` and `status`, or `io::stdout()` and
  `io::stderr()`, §1.1) writes to the descriptors directly, past the writer
  thread. So when a `Command`'s `spawn`, `status`, or `output` starts such a
  child, it first waits until the writer has written every sink record
  emitted before the call, or dropped it where SINK-5 allows:
  `println!("building"); cmd.status()?` prints in that order, as in Rust. A
  branch's records count as emitted when the outermost join appends them to
  its task's output. The wait is backpressure (SINK-5), not suspension. The
  simulator needs none, since records enter its trace when emitted (SINK-6).
  A parallel branch cannot start a child, since its closure is `Pure`
  (§8.1).
- **SINK-5. Loss and backpressure.** `out` and `err` are never dropped under
  load: when the process-wide buffer passes its high-water mark, the
  emitting OS thread blocks until the writer thread drains it. That is
  backpressure, not suspension. If stdout is a closed pipe, the process
  exits quietly with status 141, matching the default `SIGPIPE` behavior
  that `cmd | head` relies on. Any other record that cannot be written to
  its destination is discarded. `log` and `trace` drop records under
  overload and report the count as a `log` record.
- **SINK-6. Under simulation.** Every record enters the event trace with
  node, task, and virtual time; tests read them with `sim.records()`.
- **SINK-7. Elision.** Output from authority-free code may appear zero or
  one times under memoization and `const fn` evaluation.
- **SINK-8. Release and library rules.** `dbg!` is an error in release and
  CI profiles (`FX0507`) and blocks publishing. `print!` in a library package
  is lint `FX0508`. Printing inside a `Shared::with` closure is lint
  `FX0607`, since `out` can block while the cell is held. `log` forms above
  the manifest's `release-max-level` emit nothing in release builds, but
  their arguments are still evaluated (SINK-3).
- **SINK-9. Reported, not granted (v1).** Sink use is an exercised effect
  (§4.1).
  The lockfile records each package's effects (`effects = ["log", "out"]`),
  and a dependency that adds one appears in the trust diff. Effects are not
  approval-gated by default; `[policy] gate-effects = true` gates them.
- **SINK-10. Secrets.** `Secret<T>` implements neither `Debug` nor `Display`,
  so formatting one is a type error; `expose()` reads the value. Exfiltration
  still needs `Net`, `Fs`, or `Proc`.

### 2.5 Admission

Under D18 rustc compiles a Tide crate against Tide's std, injected as `std`,
so the real std's paths cannot be named. `core` and `alloc` stay reachable,
and std's inherent methods stay callable on the types Tide's std re-exports,
so the rules above also need an admission check (D7).

- **ADMIT-1. Allowlist.** The checker resolves every path, method call,
  operator, deref, impl selected for an obligation, and macro expansion, and
  admits only listed items of `core` and `alloc` and listed inherent methods
  of std's types, keyed on crate identity rather than crate name. Anything
  else is `FX0520`, with a fix where std offers a replacement. Outside the
  list are at least:
  - safe hardware intrinsics such as `core::arch::x86_64::__cpuid`;
  - raw pointer types and `std::sync::raw` (CAP-3) without an `unsafe`
    grant, and every address observer (GLOBAL-4);
  - `core` atomics, which are created without `&Tasks` (§1.2);
  - `OnceLock` and `LazyLock`, which a parallel region could initialize
    from either branch (`02-memory.md` §9). Tide's std does not export
    them, and the driver reports a path to Rust's as `FX0520`, not as
    rustc's unresolved path; `OnceCell` and `LazyCell` stay;
  - any conversion `io::Error::new` and `io::Error::other` perform that
    does not reach, through core's blanket `Into`, one of alloc's four
    `From` impls for `Box<dyn Error + Send + Sync>` from a `Pure` error or a
    string, including a user `From` or `Into` impl and the reflexive
    `From<T> for T` (`FX0517`, `02-memory.md` §9);
  - `core::task` and `alloc::task` (`Waker`, `Context`, `Wake`), which
    exist only for async (D8) and hide the code a wake runs. Tide's
    `std::task` does not export them, and the driver reports
    `std::task::Waker` and the like as `FX0520`, as for `OnceLock`;
  - float observers and math methods (`to_bits`, `copysign`, `total_cmp`,
    `min`, `max`, `sin`, `exp`, and the rest; PAR-3);
  - I/O methods on re-exported std types (`Path::exists`, `metadata`,
    `read_dir`, `canonicalize`);
  - `io::Error::last_os_error`;
  - compile-time reads of files and the build environment (`include_str!`,
    `env!`, `option_env!`), inside attributes too;
  - `global_asm!`, which places machine code without an `unsafe` block,
    and `asm!`, in a package without an `unsafe` grant; under the grant
    they are `unsafe` holes (RUN-4);
  - the internal runtime entry points of Tide's std.

  The allowlist is versioned with the rustc pin and reviewed whenever the
  pin moves. A backstop runs after rustc's analysis, when neither rustc nor
  the checker found an error, over the instances that can run: those that
  the items a build of the crate exports, its statics, and its entry point
  reach through the uses rustc records. An instance of a non-local impl that a
  rule denies, or that needs an `unsafe` grant the package lacks, is
  `FX0520`, and where an item outside `core`, `alloc`, and Rust's std is
  used, the `Pure` part of its where-clauses at the instance's arguments is
  decided as at a use. It
  catches routes to an impl that the checks of resolved uses miss. Since it
  runs after type checking, a check also reports post-monomorphization
  errors, such as a `const` that fails for one instance of a generic.
- **ADMIT-2. The driver owns rustc** (D18). It passes rustc no
  user-controlled `--extern`, `-L`, `--sysroot`, `--cfg`, or semantic `-C`
  flags, and passes `-Zallow-features=` so user code cannot enable unstable
  features, and `-Zpolonius=no`, since the pinned nightly otherwise checks
  borrows with Polonius and v0 has NLL's verdicts (`02-memory.md` ORIGIN-6).
  It also passes `-Zshare-generics=no`, so the crate instantiates upstream
  generics itself, except `#[inline(never)]` ones an upstream crate already
  instantiated, and the backstop sees them (ADMIT-1).
  A build that skips the checker, such as Cargo with Rust's own std, is not
  a Tide build and carries none of this chapter's guarantees.

### 2.6 Running an unreviewed branch

An orchestrator runs `tide check`, `tide test`, and `tide sim` on branches
no one has reviewed (D13; research/13, X3, X8, and E6). These rules state
what that can do. The *candidate* is the branch's tree; its *base* is a
reviewed revision, such as the head of the protected branch it will merge
into, fetched from the protected remote.

- **RUN-1. The guarantee.** `tide check`, `tide test`, `tide sim`, and `tide
  sim --replay` run the candidate's code in two places only. One is
  compile-time evaluation inside rustc (`const fn`, and the toolchain's own
  derives and attributes expanding the candidate's source), which performs
  no I/O. The other is the test and simulation processes of RUN-3, where the
  candidate's code holds no real capability: test code cannot name a real
  `Root` (ROOT-1, ROOT-5) or the runtime's minting entry point (ADMIT-1),
  and builds only simulated ones (ROOT-3), and `tide sim <bin>` runs `main`
  on a simulated root that the runtime builds. A simulated root's clock,
  filesystem, network, randomness, environment, standard input, and
  processes are the simulator's models (§9), and the sinks go to pipes the
  harness reads (RUN-3). A replay manifest holds what it replays inline
  (REPLAY-1), so `--replay` reads no other host path. So the candidate's
  code reads, writes, sends, and starts nothing outside the run except
  through the holes of RUN-4. The toolchain itself reads the candidate's
  tree, writes its artifact cache, the run's output directory, and the
  copies in `target/sim/`, and fetches locked packages from the registries
  the operator's configuration names, never the tree, running none of their
  code (D13).
- **RUN-2. The build reads only the candidate's tree.** The driver (ADMIT-2)
  gives rustc nothing else to read or run:
  - rustc reads source through the driver's file loader, which serves only
    `.rs` files under the package's source directories and the toolchain's
    own inputs. Any other path reads as missing, so `include_str!`,
    `include_bytes!`, `include!`, and `#[path]` never read a file outside
    them, though ADMIT-1 and `FX0911` reject them anyway, and no byte of an
    outside file can reach a diagnostic. An attribute that rustc resolves to
    a file read outside the loader, such as `#[debugger_visualizer]`, is
    rejected before the pass that reads it;
  - rustc, with its LLVM backend, and the linker run in a build sandbox,
    Landlock and seccomp on Linux as in RUN-3: they may read only the
    package's source directories, the toolchain (the host C library and
    start files the linker needs included), and the artifact cache, write
    only the cache, which also holds their temporary files, create no
    socket, and start no program but the toolchain's linker. So a read the
    file loader never sees, such as an assembler `.include` or `.incbin` in
    granted `unsafe` assembly, cannot reach outside them, though it can
    embed a cache entry. One level, reported as RUN-3 says, covers this
    sandbox and RUN-3's; a build that cannot install it fails, and without
    `full` the build's reads rest on the loader alone;
  - every path the toolchain resolves in the candidate's tree or the run's
    output directory, the file loader's and the harness's included, resolves
    beneath that root, as `Fs` paths do (§1.1): a symbolic link or `..` that
    leaves it reads as missing. Each run's output directory is fresh and
    lies in the toolchain's state directory, outside the tree; the harness
    copies the manifests and traces it reports into `target/sim/` (§9.6),
    replacing, never following, whatever is at each copy's path;
  - rustc and the driver run with an environment the toolchain sets, not the
    caller's, so `env!` and `option_env!` find nothing of the operator's
    even before ADMIT-1 rejects them;
  - nothing in the candidate's tree chooses the toolchain, the linker, rustc
    flags, or a program to run: the toolchain ignores `rust-toolchain.toml`,
    `.cargo/config.toml`, `RUSTFLAGS`, and `RUSTC_WRAPPER`, and neither the
    manifest nor the lockfile names a program or a source outside the
    repository and the operator's registries;
  - no build script, install hook, or proc macro of a Tide package runs
    (D13, `FX0915`).
- **RUN-3. Test and simulation processes.** `tide test` and `tide sim` run
  each test or simulation binary as a child process (PANIC-1). The harness
  starts it with an environment the toolchain sets, not the caller's; in a
  new session with no controlling terminal; with both soft and hard
  core-size limits at exactly one byte, which survive exec and stop core
  dumps to a file or a piped collector, or it fails to start, saying why (as
  on a host whose hard limit is below one byte, which an unprivileged
  harness cannot raise); and with no inherited descriptor but an empty
  standard input, pipes for standard output and error, the result channel
  (RUN-6), the run's output directory, the contents of any `--replay`
  manifest or `--faults` plan, which the harness reads and passes in, and,
  in the later tier, a connection to each Bend kernel server it needs
  (RUN-4, hole 4). Between fork and exec it installs an operating-system
  sandbox, which therefore covers everything the binary runs, its static
  constructors included. On Linux (v0) the sandbox is Landlock and seccomp,
  both as allowlists:
  - Landlock lets the child open by path only the binary and the libraries
    it loads, to read and execute, and the run's output directory, where it
    may create only regular files and directories, and no links; its scopes
    confine signals and abstract Unix sockets to the child;
  - seccomp admits only the system calls the runtime needs, among them
    memory, threads, futexes, clocks, randomness, polling, signal handling
    within the process, reads, writes, seeks, and closes on open
    descriptors, the opens that Landlock confines, `stat`-family and
    `access` calls, and exit. Threads are created only by `clone` with
    thread flags, which seccomp can inspect (`clone3` reports `ENOSYS`, so
    the runtime falls back to `clone`), and `execve` succeeds only for what
    Landlock lets the child execute, its own binary with the loader and
    libraries it maps, so the child starts no other program or process.
    Every other call fails, among them process creation, namespaces,
    sockets, io_uring (which seccomp cannot inspect, research/10), keyrings,
    System V IPC (shared memory, semaphores, and message queues), POSIX
    message queues, extended-attribute and link reads, metadata changes
    (permissions, owners, extended attributes, times), resource-limit
    changes (`prlimit64` passes only when it sets no new limit), and
    tracing.

  Neither mechanism confines reading the metadata of a host path (whether it
  exists, and its type, mode, owner, size, and times), so code that reaches
  hole 1 or 5 of RUN-4 can read that much. A foreign file or socket call
  that no model intercepts is refused (§9.8). The toolchain reports the
  sandbox level, of builds and runs alike, in `tide review` (RUN-5) before
  anything runs, in `tide check`'s verdict, and in every run result:
  `sandbox: full`; `partial`, naming what the kernel cannot enforce; or
  `none` (other operating systems in v0). If the harness cannot install the
  level it reports for this host, it fails the run. Without `full`, RUN-1
  rests on the checker and compiler alone. This fixed policy is separate
  from kernel enforcement of a deployed program's attenuated `Root` (D7,
  v1).
- **RUN-4. Holes.** RUN-1 does not hold through the following; any other
  route is a defect in RUN-1 to RUN-3.
  1. **UNSAFE-1. `unsafe` grants.** `unsafe` blocks and functions (with a
     body or not: a required trait method too), `unsafe extern` blocks,
     which link the native libraries their `#[link]` names even when
     empty, and every item in one (a `safe` one too, since callers reach
     foreign code through it with no `unsafe` block), `unsafe impl` of any
     trait (`Send` and `Sync`, `02-memory.md` §8.1, or a user's `unsafe
     trait`), Rust 2024's unsafe attributes (`#[unsafe(no_mangle)]`,
     `#[unsafe(export_name)]`, `#[unsafe(link_section)]`), and
     `#[trusted_pure]` need the package's `unsafe` grant (`FX0901`), and so
     do `asm!` and `global_asm!` (`FX0520`, ADMIT-1); all are trust holes. Declaring an `unsafe trait` needs no
     grant, since only an impl asserts its contract. No grant admits
     `unsafe impl Pure`, which overlaps std's blanket impl (E0119,
     `FX0402`; `#[trusted_pure]` is the spelling), or `#[may_dangle]` (an
     unstable feature: `02-memory.md` §6, ADMIT-2). Unsafe code can forge a
     capability (ROOT-4), read process memory, replace symbols the runtime
     calls, make raw system calls, and write the result channel. The
     candidate's safe code reaches the same through an unsound `unsafe`
     abstraction the base already holds.
  2. **`ffi` grants** (v1). Foreign code ignores capabilities (FFI-4), and a
     foreign effect with no simulator model (`FX0514`) acts on the host
     unless the sandbox refuses it.
  3. **Imported Rust crates** (v1): their code (RUST-1 to RUST-4), and their
     build scripts and proc macros, which run at build time in RUST-5's
     sandbox and can emit code into the build (§11).
  4. **The Bend boundary** (later, §10). The pinned `bend` checks and builds
     the candidate's kernels at build time, and Tide trusts its rejection of
     IO-typed, foreign, and `@unsafe` defs. Since the child may create no
     socket, a test calling a kernel without a `reference` reaches a server
     that the harness starts, restarts, and stops under a sandbox of its
     own, connected to the child by an inherited descriptor; the Bend
     runtime's bugs are reachable through it.
  5. **The toolchain and the kernel**: soundness bugs in rustc, LLVM, the
     driver, Tide's checker, Tide's std, the toolchain's derives and
     contract macros, or the linker; an allowlisted item that reaches the
     world (ADMIT-1); bugs that input can exploit in the compiler, the
     harness (which reads the child's pipes, result channel, and output
     directory outside the sandbox), the toolchain's parsers of the
     manifest, lockfile, `tide.seal`, approval records, replay manifests,
     and fault plans, or the fetcher (which unpacks the registry archives
     the lockfile picks); and bugs in the operating-system kernel that
     enforces RUN-3. rustc has open soundness bugs that safe code can turn
     into memory corruption (rust-lang/rust#25860, which the `cve-rs` crate
     uses), so even safe code can forge a capability.
  6. **Availability and side channels.** A candidate can spin, allocate,
     recurse, spawn tasks, and write sink output until a limit the operator
     sets stops it. RUN-1 bounds what a run reaches, not what it consumes,
     nor side channels such as timing (§9.8).
- **RUN-5. Surfacing holes before running.** `tide review <base>` runs the
  candidate's code only at compile time (RUN-1) and starts no process from
  the candidate's build except the behavior diff's RUN-3 children below. Its
  first line, and fields under `--json`, give `run: covered` or `run: not
  covered`, the sandbox level this host's runs get (RUN-3), the base commit
  it resolved, and the candidate's snapshot hash (D14) over the bytes the
  build reads; the reasons follow. The base must be a reviewed revision,
  such as a commit fetched from the protected remote, never a commit of the
  candidate itself. `tide test --snapshot <hash>` and `tide sim --snapshot
  <hash>` start no test unless the full hash of the bytes their build read
  matches, so a verdict applies to the tree it judged. A candidate is
  covered when, against the base, it adds or widens no grant; adds or
  changes no imported Rust crate (its declaration or lockfile entry), build
  script, or Bend dependency (its pin, kernel sources, or law files); adds
  no trust hole; and adds, removes, or changes neither a source file nor the
  lockfile entry (version, source, or content hash) of a package with an
  `unsafe` or `ffi` grant or of any package such a package depends on,
  transitively, since the soundness of `unsafe` code can rest on any item
  its crate can see. A covered candidate reaches only holes reviewed in the
  base. v1's behavior diff (D11), which runs the candidate's seeds under
  RUN-3, runs only for a covered candidate on a host with `sandbox: full`,
  unless the operator asks for it.
- **RUN-6. Results.** Results travel on a channel that no sink writes, so
  printing cannot forge a pass. A child that exits, at any status, before
  writing the result of a test it started fails that test and the run, and
  tests it never started run in a fresh child (PANIC-1). The harness builds
  every command it prints, such as a replay command, only from values it
  checks: a test name the build declared, an integer seed, and a path it
  chose, so nothing the child writes reaches a printed command. The
  toolchain escapes control characters (C0 and C1) and bidirectional
  overrides in everything it prints, to a terminal, pipe, or log: the
  child's output, diagnostics, and `tide review`'s lines, file names
  included. Code that reaches hole 1 or 5 of RUN-4 can still write the
  channel and forge results and traces, a `#[protected]` test's results and
  v1's behavior diff included, so a pass from an unreviewed candidate is
  evidence only up to those holes.

**Operating guidance.** For a covered candidate on a host whose sandbox is
`full`, an orchestrator runs `tide test` and `tide sim` inside its agent
session without a VM per run. What remains is confined as follows: an
unsound reviewed abstraction and toolchain soundness bugs at run time meet
RUN-3's sandbox, though they can forge results (RUN-6) and read host
metadata; the Bend runtime serving reviewed kernels meets its server's
sandbox; compiler bugs exploited at build time meet the build sandbox
(RUN-2), which bounds their file and network access but not the code they
emit; and bugs in the harness, the parsers, the fetcher, or the kernel, and
resource use meet only the session's sandbox, which confines a kernel bug
only if it is a VM. So the build, `tide check` and `tide review` included,
also runs inside the session's sandbox, and artifacts built from an
unreviewed candidate are not shared with builds outside it. For any other
candidate, the orchestrator reviews the listed holes first, or runs the
branch inside a VM.

What RUN-1 does not cover: an agent session runs arbitrary commands (`git`,
package managers, `curl`), and `tide run` and a built binary run with a real
`Root`, and `tide eval`'s root and sandbox are not yet set (D14). Only a
microVM or OS confinement (Landlock, seccomp) bounds those, so a fleet keeps
one sandbox per agent session; RUN-1 removes the need for one per test run,
since tests and seeded simulations run in sandboxed child processes with
simulated capabilities and virtual time. Kernel enforcement (D7, v1) gives
the deployed process its own OS sandbox, derived from `main`'s attenuations.
A microVM, for its part, does not protect the credentials inside it, which
RUN-3's environment and sandbox keep from test code; gives a reviewer no
evidence of what changed in what code may do, which the trust diff does; and
is gone in production (research/13, X8 and E6).

Conformance tests (D0), by rule. A *canary* is a file, environment variable,
listener, or program planted outside the candidate that the test watches for
any access.

| Rule | Tests |
| --- | --- |
| RUN-1 | A test and `tide sim <bin>` exercise every capability of a simulated root: each call appears in the simulator's trace, and no host canary sees an access. `tide sim --replay` of a manifest naming a canary path opens no host path. |
| RUN-2 | `include_str!`, `include_bytes!`, `include!`, `#[path]`, and `#[debugger_visualizer]` naming a canary file: each is rejected, an access trace of the build shows no open of the canary, and its bytes appear in no diagnostic, artifact, or cache entry. A committed symbolic link from a source directory to a canary reads as missing, and the canary is never opened; one committed at `target/sim/` is replaced, and the harness writes nothing through it. `env!` and `option_env!` of a canary variable set in the caller's environment: rejected, and the value appears nowhere. A `rust-toolchain.toml`, a `.cargo/config.toml` naming a canary `rustc-wrapper` and `linker`, and `RUSTFLAGS` and `RUSTC_WRAPPER` in the caller's environment: none takes effect. On a host with `sandbox: full`, under an `unsafe` grant, `global_asm!` and `asm!` with `.include` and `.incbin` naming a canary, written directly and built by an assembler macro: the build opens nothing outside its allowed set, and the canary's bytes appear in no diagnostic, artifact, or cache entry. A `build.rs`: `FX0915`, and it never runs. A manifest or lockfile naming a source outside the repository and the operator's registries: rejected. |
| RUN-3 | Under an `unsafe` grant, a test and a static constructor (`#[unsafe(link_section = ".init_array")]`) each try to open a canary file, connect to a canary listener, run a canary program, set up io_uring, create a link, FIFO, or socket file in the output directory, change a canary file's permissions, owner, extended attributes, or times, read the user keyring, open a message queue or System V shared memory, read a canary's extended attributes or a link outside the run, fork a process, trace or signal the harness, read a canary variable from the caller's environment, and crash, on a host that pipes core dumps to a collector, with the harness started under a soft core-size limit of zero, and again after the test tries to lower its own soft limit to zero: each fails or writes nothing outside the run, the result says `sandbox: full`, and no canary sees an access. Run on a pseudo-terminal, a test that tries to inject input into the terminal receives none, since the child has no controlling terminal and its streams are pipes. On a Linux kernel without Landlock, `tide review` and the result say `sandbox: partial`, naming Landlock. |
| RUN-4 | Each unsafe attribute without an `unsafe` grant is `FX0901`, and `asm!` and `global_asm!` are `FX0520`; under the grant, each appears in the trust diff as a hole. |
| RUN-5 | `run: not covered`, naming the reason, for each of: a new or widened grant; a new `unsafe` block; an edit to a safe function in a package with an `unsafe` grant, in a descendant module of the one holding the `unsafe` block too; an edit to an ungranted in-tree package that a granted package depends on; a new version, source, or content hash in the lockfile for a granted dependency or for a package it depends on; a new dependency without grants whose own manifest grants `unsafe` to one of its dependencies; a changed lockfile entry of an imported Rust crate; an edited Bend kernel source; a new imported Rust crate or `build = true`; a new Bend dependency. `run: covered` for a candidate that edits only packages that hold no grant and that no granted package depends on. An access trace of `tide review` shows no process started from the candidate's build except behavior-diff children, and none of those for a candidate not covered or on a host without `sandbox: full`. `tide test --snapshot` with a hash from another tree starts no test. |
| RUN-6 | A test that prints a pass record to stdout does not pass. A child that exits with status 0 inside a started test fails that test and the run, and the tests it never started run in a fresh child. Control characters and bidirectional overrides in a test's output, in a diagnostic's file name, and in a `tide review` line reach a pseudo-terminal and a log escaped. A test name and a replay manifest path written by the child with shell metacharacters never appear in a printed command; the printed replay command names only the declared test, the seed, and the harness's path. |

## 3. `Pure` and Authority-Free Functions

### 3.1 Definitions

`Pure` is a marker trait the checker owns (D7). Tide's std declares it with a
blanket impl, so rustc accepts every `Pure` bound and coherence never treats
`Pure` as making two impls disjoint (`02-memory.md` PURE-2). The checker
decides it structurally wherever a type meets a `Pure`-implying obligation or
is coerced to a `Pure` trait object (`02-memory.md` PURE-1). A failure is
reported under the code of the rule that needs `Pure` (`FX0502`, `FX0518`,
`FX0603`, `FX0606`, `FX0610`, `FX0611`, `FX0612`), and otherwise, as at a
user-written bound, a `Pure` supertrait's impl, or a coercion to a `Pure`
trait object, as `FX0517`. A type the structural rule rejects that is
authority-free in fact, such as a foreign handle, may be marked
`#[trusted_pure(reason = "...")]` in a package granted `unsafe` (UNSAFE-1),
and each such mark is a trust hole. The mark vouches for the type's own
fields only: a generic marked type is `Pure` exactly when its type
arguments are, as a std type is (`02-memory.md` §9), since its author can
vouch for what they wrote but not for every type a caller passes, and
failing closed keeps the trust hole as wide as what the reviewer approved.
The rule is deep, through every field, private ones included;
`dyn` types are `Pure` only through a companion supertrait (below), and
`02-memory.md` §9 has the full table.

**`Pure`** means that no authority is reachable. Not `Pure`:

- the capability types (`Root`, `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`,
  `Proc`, `Tasks`, `Accel`) and the resource handles derived from them;
- the synchronization handles (`Shared`, channel ends, `Scope`, `Task`,
  `Arc`, `sync::Weak`, atomics, everything in `std::sync::raw`);
- foreign handles: raw pointers, `extern` types, foreign function pointers;
- `ThreadBound` types;
- `dyn Trait` whose trait does not have `Pure` as a supertrait, and
  unbounded type parameters and projections on them.

Everything else is `Pure` exactly when its components are, including `Cell`,
`RefCell`, `Rc`, collections, closures, and Tide function pointers (a
pointee's authority arrives through its arguments).

**`Pure` trait objects.** `dyn Trait + Pure` is not Rust (E0225): only auto
traits can be added to a trait object. A trait object that must be `Pure`
uses a companion supertrait with a blanket impl (`02-memory.md` DYN-2):

```rust
pub trait PureValue: Value + Pure {}
impl<T: Value + Pure + ?Sized> PureValue for T {}
```

`&dyn PureValue` upcasts to `&dyn Value`. To rustc the blanket impl makes
every `Value` a `PureValue`, so the checker's structural check runs at each
coercion to `dyn PureValue`, not at the impl.

**Unchecked callers.** Rust code that calls a Tide crate is not checked
(RUST-6, v1), so a closure or trait impl it passes in may meet a `Pure` bound
without being authority-free. Such values are trust holes, as foreign
callbacks are.

**`Pure + Sync`** replaces draft 2's `Frozen` (D7): `Sync` excludes `Cell`,
`RefCell`, `OnceCell`, `UnsafeCell`, and `Rc`, and `Pure` excludes atomics
and every other synchronization handle, so nothing reachable from a
`Pure + Sync` value changes while another task or branch reads it.

### 3.2 Authority-free functions

A function, method, or closure is **authority-free** when every parameter
type, including `self`, and every capture type is `Pure`. Generic functions
are classified per instantiation; a generic definition that needs the
property states it as a bound (`F: Fn(&T) -> U + Pure`). Foreign functions,
declared in `extern` blocks, are never authority-free (`#[pure]` on one is
`FX0511`); their wrappers may be (§5), and so may a Tide function with a
foreign ABI (`extern "C" fn f() {..}`), as a callback must be (FFI-6): its
body is checked Tide code. An authority-free body also cannot create a
synchronization handle, since that takes `&Tasks` (§1.2).

- **AFREE-1.** `#[pure]` asserts authority-freedom of the function it marks,
  with a body or not (a required trait method), but not of the items nested
  in its body, for every instantiation the bounds allow; an edit that breaks
  it is `FX0511`. Every public function's interface records the property
  anyway, and `tide review` reports its loss as an API change, which answers
  the design's open question 4.

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
   including the simulator, and in every build profile (D12).

The argument: safe code has no ambient source (§2.1, §2.2); `Pure` arguments
reach no capability or handle, since std's types are walked like any other
and an erased payload is `Pure` only when std admits only `Pure` concrete
payloads into it (`02-memory.md` §9), and the body cannot create a handle without
`&Tasks`; and the memory they reach cannot be mutated concurrently, because
every std type that is `Sync` and mutable through `&` is a synchronization
handle and therefore not `Pure`, or is not admitted (`OnceLock`,
`LazyLock`), while `Cell`, `RefCell`, `OnceCell`, `LazyCell`, and `Rc` are
not `Sync` and so cannot be shared with another task. Outside std, a `Sync`
type that is mutable through `&` either has such a std type as a field, and
so is not `Pure` (the rule is deep), or reaches an `UnsafeCell` through
`unsafe impl Sync`, a trust hole under the `unsafe` grant (`02-memory.md`
§8.1). A sound one either keeps readers and writers apart statically, as a
token-branded cell does through `&mut`, or synchronizes at run time through
atomics, `std::sync::raw`, raw pointers, or foreign calls, none of which is
`Pure`; an unsound one breaks this argument as an unsound `unsafe impl Send`
does. `Arc` is a handle too, because its last release and `make_mut` run
user code at timing-dependent points (GLOBAL-6).

### 3.4 What it does not guarantee

- termination (divergence is tracked only under `prove`), absence of panics
  or overflow traps, absence of allocation, or bounded time and memory;
- absence of sink output, or of mutation through `&mut` parameters and
  task-private cells;
- memoizability, which also needs `Pure + Sync` parameters and no `&mut`;
- safety for parallel branches, which also needs `Send` (and `Sync` for a
  closure called from several branches), `Pure + Sync` shared captures,
  and `Pure` elements (PAR-4);
- constant time or freedom from timing side channels.

### 3.5 Where each property is required

| Use | Requires |
| --- | --- |
| `join` closures; `par_*` closures | `FnOnce + Pure + Send`; `Fn + Pure + Send + Sync`; for both, shared captures `Pure + Sync` and mutable captures disjoint; `par_*` elements `Pure + Sync` when shared, `Pure + Send` when split or moved (PAR-4) |
| compiler memoization, `const fn` | authority-free, `Pure + Sync` parameters, no `&mut` parameters (AFREE-2); a `const fn` body evaluated at compile time is part of its consumers' cache keys (D12) |
| `static` (computed at compile time, GLOBAL-2) | `Sync + Pure`, and `'static` as in Rust |
| `Shared<T>` payloads | `T: Send + Pure` |
| `Shared::with` closures | `Pure`, so nothing they reach holds authority (LOCK-4) |
| contract clauses (v0) | as `04-contracts.md` CLAUSE-3: calls only to authority-free functions with `Pure + Sync` argument types that take `&mut` only to temporaries the clause creates |
| `spec fn`, `#[spec]` functions (v1) | as `04-contracts.md` §1.5–§1.7: concrete parameter types `Pure`; type parameters of spec and proof fns need no bound |
| foreign callbacks (v1) | authority-free (§5.3) |

- **AFREE-2. `const fn`.** A `const fn`'s parameter types are `Pure + Sync`,
  and none is `&mut` (`FX0518`; a type that names `&mut`, as
  `Option<&mut T>` does, or holds one in a field, as
  `struct Counter<'a>(&'a mut u32)` does, counts), although rustc accepts
  both. A type parameter, including one in `Self`'s type, needs the bound at
  the definition (§3.2, D4). Memoization rejects nothing: the compiler
  memoizes only calls that meet the same conditions.

## 4. Declared and Exercised Authority

### 4.1 How the compiler computes them

**Declared authority** of a function is computed from its signature: for
each parameter (and capture, for closures), the authority kinds reachable
from its type, each labelled with its access path. For
`fn handle(app: &App, req: &Request)` where `App` holds `db: Db` (which holds
a `Net`), `files: Fs`, and `clock: Clock`, declared authority is
`{app.db.net: net, app.files: fs.read+fs.write, app.clock: clock}`. A
`dyn Trait` whose trait lacks a `Pure` supertrait and an unbounded type
parameter contribute
`opaque(..)`, resolved below.

**Exercised authority** (v1; v0 reports declared authority only) is computed
bottom-up from bodies:

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

### 4.2 The least-authority lint (v1)

`FX0510` fires on a function other than `main` that holds a capability
parameter or field path it never exercises. It compares capability types, not
read versus write, so an `Fs` whose holder only reads is not flagged; review
suggests `Mode::Read` attenuation in `main` instead. The fix narrows a bundle
to the fields used (`&App` to `&Clock`) or removes the parameter. It is
`safe` for private items and `suggest` for public ones. The lint skips trait
method impls and methods whose receiver is the only authority-bearing
parameter; review still shows their exercised paths.

### 4.3 The trust diff

`tide review <base>` has an authority section. It lists only items whose
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

### 4.4 `tide fix --thread`

`tide fix --thread <Cap> --to <fn>` adds a capability parameter along a
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

## 5. Foreign Calls (v1)

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
     (it may run inside `join` or `par_map`);
  4. any callback runs only during the call;
  5. the same holds for the foreign destructor of any handle it returns.
  Pure foreign code that is not thread-safe is wrapped in a type that is not
  `Sync`, and its functions take that handle.
- **FFI-4. Attenuation is the wrapper's job.** Foreign code ignores Tide
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
`#[sim_model(path::to::model_fn)]`: a Tide function with the same signature,
written against simulated capabilities. The sim build lists every reachable
wrapper without one (`FX0514`); `tide sim` refuses to run while any remain
unless `--allow-unmodeled` is given, which marks the run non-replayable.
`#[trusted_pure]` calls need no model; the determinism self-check (§9.7)
catches some false claims.

### 5.3 Callbacks

- **FFI-6.** A Tide function or closure passed to foreign code must be
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
(§11). It is a checker-owned structural property: a type is `ThreadBound`
if any component is, through fields, type arguments, and closure captures. A
`ThreadBound` type is neither `Send`, `Sync`, nor `Pure`. A `ThreadBound`
type may be coerced to `dyn Trait`, or be the hidden type of a returned
`impl Trait`, only if the opaque type names `+ ThreadBound`
(`-> impl Debug + ThreadBound`) or the object's trait has it as a companion
supertrait, as for `Pure` (§3.1): `dyn Debug + ThreadBound` is not Rust
(E0225), so the object is `Box<dyn DebugThreadBound>` with
`trait DebugThreadBound: Debug + ThreadBound {}` and a blanket impl. Such a
value pins like the concrete type, so erasure cannot hide the value from the
liveness analysis (round-2 consistency review B8). Under the evented runtime the
compiler computes, from liveness, where a `ThreadBound` value is live across
a suspension point and pins the task to its current worker over that range;
a pinned task is never stolen, and a foreign call taking a `ThreadBound`
argument runs inline rather than on the blocking pool. Under the other
runtimes tasks never migrate, so pinning is a no-op.

## 6. Tasks

### 6.1 Scopes

The origin structure is `02-memory.md` §8.2; the names here are normative.

```rust
impl Tasks {
    pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
        where E: From<TaskError> + Send, F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>;
    pub fn spawn_isolated<T: Send>(&self, f: impl FnOnce() -> T + Send + 'static)
        -> Result<T, Panicked>;
    pub fn shield<T>(&self, body: impl FnOnce() -> T) -> T;
    pub fn checkpoint(&self) -> Result<(), Cancelled>;
}
impl<'s, 'env, E> Scope<'s, 'env, E> {    // Copy; origin 's
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
        where F: FnOnce() -> Result<U, E> + Send + 's;
    pub fn set_deadline(self, at: Instant);
    pub fn cancel(self);
}
impl<U, E> Task<'_, U, E> {
    pub fn join(self) -> Result<U, E>;    // shielded
}
pub enum TaskError { Cancelled, DeadlineExceeded }
pub struct Panicked { pub message: String }   // provisional
```

Children may borrow anything that outlives the `scope` call. A scope passed
down (`fn serve<'s>(bg: Scope<'s, '_, E>, cfg: &'s Config)`) takes children
that own their data or borrow what the signature ties to it. A child's panic propagates (TASK-1), so
no code after `scope` sees a place a panicking child was mutating; when
`scope` returns, its loans have ended (D5, `02-memory.md` §8.3).

Under `tasks.limit(n)`, all scopes from that `Tasks` share one count: at
most `n` of their children are live at once, and `spawn` waits for a free
slot. A limited `spawn` is a checkpoint. `spawn` keeps
`std::thread::scope`'s shape and returns a `Task`, not a `Result`, so it
reports cancellation through the child: when the calling task is in a
cancelled scope, a limited `spawn` returns at once and starts nothing, even
if a slot is free, and drops `f` unrun: cleanup written in `f` never runs,
so a cleanup spawn belongs under `shield`. It sets the caller's echo bit
when any checkpoint would (TASK-2). The child's `join` returns
`Err(TaskError::Cancelled.into())`. If the scope spawned into is
itself cancelled, the child counts as one whose first checkpoint returned
the cancellation and which returned `E::from(TaskError::Cancelled)`, so
TASK-2 and TASK-4 decide the outcome unchanged: it is an echo when the
body's `Err` or a sibling's failure caused the cancellation, and counts as
cancelled by the deadline when the scope's own deadline did. If it is not,
as when a task in a cancelled nested scope spawns into an enclosing scope
passed down to it, the child counts as an echo and that scope is
unaffected.

The wait observes cancellation because the count is shared: the slots may
be held by children of other scopes, which the caller's cancellation does
not reach, so a wait that ignored it could outlast the scope's deadline
without bound. Starting the child over the limit instead would break the
attenuation. The cost is that a child that never starts cannot turn the
cancellation into a value; to give every item an outcome by a time, pass
the deadline to the operations rather than set it on the scope (companion
15.2). Returning a `Result` from `spawn` would put a `?` on every spawn for
a wait that only limited scopes have.

A task that holds a slot and waits for one, such as a child that spawns
into its own scope or opens a nested scope from a `Tasks` that shares its
count, can deadlock the count once every slot is held that way. Cancelling
the scopes of the tasks waiting in `spawn`, by a deadline or `cancel()`,
breaks it unless they wait under `shield` (TASK-6). Otherwise the
simulator reports a global deadlock once no task can run (§9.3).

A clone of a limited `Tasks` shares its counts, as it shares its
authority. `limit` on a limited `Tasks` adds a count and keeps the old
one: a child spawned through the result takes a slot in each count at
once, so the outer bound still holds and no spawn holds one count while it
waits for another. Everything but `spawn` works on `limit(0)`, so
`limit(n.min(len))` on empty input is fine, but a `spawn` under a zero
count panics, as `chunks(0)` does, since it could only wait until
cancelled. It panics in a cancelled scope and under `shield` too, so the
misuse shows on every run.

`spawn_isolated` takes no slot: its caller, which may hold one, waits for
the isolated task, so no more children run at once.

### 6.2 Failure, cancellation, and deadlines

- **TASK-1. Failure and panics.** A child or the body fails when it returns
  `Err`. A child's panic propagates as in Rust's `std::thread::scope`: it
  cancels the scope, and once every child has finished, `scope` re-raises a
  child's panic in the calling task (TASK-2 picks which). A panic in the body
  cancels and joins the children, then continues unwinding. The only
  recovery point is `tasks.spawn_isolated(f)`: it runs the `Owned` closure
  `f` as a new task, waits for it, inherits the caller's cancellation, and
  returns `Err(Panicked)` if `f` panicked (D5).
- **TASK-2. Scope results.** The first failure or child panic (in real
  time, or in scheduler order under the simulator) cancels the scope, its
  body, and its nested scopes. Unless the body panicked (TASK-1), `scope`
  then waits for every child and decides its outcome by one precedence rule
  (D8):
  1. if any child panicked, it re-raises the panic of the earliest-spawned
     panicking child;
  2. otherwise, if any child failed, echoes excluded, it returns the error
     of the earliest-spawned failing child, except that a child cancelled
     by the scope's own deadline is reported as
     `E::from(TaskError::DeadlineExceeded)` (TASK-4);
  3. otherwise it returns the body's result, `Ok` or `Err`.

  An `Err` that a child returns after observing a cancellation caused by the
  body's `Err` or a sibling's failure is a **cancellation echo**, not a
  failure (D8). The runtime keeps one bit per child, set when a checkpoint
  in the child, in a task it waits for with `spawn_isolated` (directly or
  through further isolated calls), or in a scope nested in either returns a
  cancellation error that the body's `Err` or a sibling's failure caused; it
  never inspects `E`. A scope's cancellation has one cause, the first event
  that cancelled it: its body's `Err` or panic, a child's failure or panic,
  its own deadline, `cancel()`, or an enclosing scope's cancellation. An
  enclosing deadline at the same instant as the scope's own comes first. A
  checkpoint in a nested scope sets the bit only for a cancellation that
  reached the nested scope from this one; a cause inside the nested scope,
  such as a failure among its own children, is that scope's to report, and
  it reaches this scope as the child's own result. A `join` that returns
  `Err` sets the bit exactly where a checkpoint at the same point and moment
  would if no `shield` were in force there. Ignoring `shield` only lets that
  checkpoint see the enclosing scopes' cancellations; every scope keeps its
  actual first cause, so a nested scope that its own cause cancelled first
  still marks nothing. A child that calls `x.join()?` directly after a
  sibling's failure cancelled this scope is an echo too. A child that fails
  with no bit set is an independent failure and counts under rule 2, ahead
  of the body's `Err`. A channel disconnect alone sets no bit, so an `Err`
  returned after one by a child whose bit is clear is an independent failure
  (§6.4).
  So when the body's `Err` cancels the scope and the children only echo
  it, rule 3 reports the body's error; when the body fails through
  `x.join()?` because `x` failed, rule 2 reports `x`'s error (JOIN-2).
  Spawn order, not timing, chooses among failed children. Failures,
  echoes, and panics the rule does not report go to one `debug!` record.
- **TASK-3. Level-triggered.** Once a scope is cancelled, every checkpoint
  inside it, in any task, returns its cancellation error immediately, and
  keeps doing so. A checkpoint that hands a value over, a channel payload or
  an owned buffer (`send`, `read_owned`, `write_owned`), returns it inside
  the error (`SendError::Cancelled(v)`, `io::BufError`, §1.1). Arguments it
  only reads, such as `fs.write`'s `impl AsRef<[u8]>` data, a path, an
  address, or a Bend call's arguments, which are copied across (§10), are
  dropped with the call; a caller that needs one again passes a reference or
  a clone. Returning every such argument would need a generic error type for
  each operation. `File::close` consumes its `File` but is shielded, not a
  checkpoint (§1.1). A limited `spawn` reports cancellation by starting
  nothing and drops its closure (§6.1).

  A system call in flight when the scope is cancelled is cancelled before
  the error returns if the kernel can cancel it, as with socket and pipe
  waits and sleeps; one that completes before the cancel takes effect
  returns its own result. A system call the kernel cannot cancel once
  running, such as a file-system call on a regular file or directory (reads,
  writes, `fsync`, opens, renames, removals, metadata queries, and directory
  reads), runs to its end and returns its own result, and the task observes
  the cancellation at its next checkpoint, so a rename or write that
  finished reports success. A host-name lookup, a library call that cannot
  be cancelled and has no effect, is abandoned instead and returns the
  cancellation error; it keeps running on its own copy of the name, and its
  result is discarded. A foreign call, such as an in-process Bend kernel, is
  not abandoned (TASK-5, §10).

  An operation made of several system calls checks before each one it
  starts, except a close, and can stop partway: a cancelled `fs.write` can
  leave its target truncated, and `remove_dir_all` part of its tree (§1.1).

  A single `write` to a socket or pipe returns a cancellation error only if
  the kernel accepted no byte of it; one that accepted any bytes returns
  `Ok(n)`, as in Rust. A backend can meet this, for example, by waiting for
  the descriptor to accept data and then making a non-blocking write,
  waiting again if it would block, so a cancel either ends a wait or finds
  the write done. Writes made of several system calls, such as `write_all`
  and `write_owned`, can stop after the kernel has accepted part of the
  data, which may already have reached the peer. `write_owned` reports how
  much in `BufError::written()` (§1.1); `write_all` keeps Rust's
  `io::Result<()>` and, as in Rust for every error, loses the count. When
  code must finish or frame a stream under `shield` after a cancellation,
  the writes that the cancellation may cut short use `write` or
  `write_owned`, so the cleanup knows where the stream stands; after a
  failed `write_all` the stream's position is unknown, and the usual
  course, which level-triggered cancellation favours anyway, is to drop it.

  The simulator follows every rule of TASK-3 (§9.3).
- **TASK-4. Deadlines.** `set_deadline(t)` lowers the scope's deadline to
  `t`; a nested scope's effective deadline is the minimum over its
  ancestors. When it passes, the scope, its body, and its children are
  cancelled (TASK-3), but expiry alone is not a failure (D8). A child that
  returns `Err` after the scope's own deadline (set on this scope, not an
  ancestor) has cancelled the scope counts as cancelled by that deadline; if
  it is the earliest-spawned failing child, `scope` returns
  `E::from(TaskError::DeadlineExceeded)` (TASK-2). The rule never rewrites
  the body's error. If every child and the body return `Ok`, for example
  because children turned cancellation into values, `scope` returns the
  partial results. `cancel()` cancels the scope in the same way, and a
  failing child's own error is reported. When a scope deadline and an
  operation's own deadline fall on the same instant, the scope's
  cancellation is observed.
- **TASK-5. What cancellation cannot interrupt.** Authority-free
  computation, `Shared::with` acquisition and closures, foreign calls,
  joins, a running system call the kernel cannot cancel (TASK-3), the wait
  for the child of a cancelled `output` or `status`, and `File::close`
  (§1.1). The last three can block even inside a shielded scope with a
  deadline: on a network file system, or, for a child the kill could not
  stop, until the child exits (§1.1). Deadlines are therefore soft, a
  cleanup scope's (TASK-6) included: a deadline cancels at the next
  checkpoint and cannot cut short any of these, so code must not rely on
  one to bound them. The evented runtime logs a `warn!` record for a task
  that runs more than 100 ms (configurable) without reaching a checkpoint;
  the simulator reports the same condition by step count.
- **TASK-6. Shielding.** Operations inside `tasks.shield(|| ..)` ignore the
  cancellation of enclosing scopes; bounded cleanup is a shielded scope with
  its own deadline, which bounds only work that reaches a checkpoint
  (TASK-5). `Drop` in std does no I/O beyond closing descriptors; a buffered
  writer dropped unflushed logs a `warn!`. `File::close` is itself shielded
  (§1.1); other cleanup that needs I/O, such as a final write or `sync`,
  runs under `shield`.

### 6.3 Joins

- **JOIN-1.** Scope exit joins every child, is shielded, and cannot be
  cancelled. Children borrow the parent frame, so this is a memory-safety
  rule.
- **JOIN-2.** `Task::join` is shielded too: it is not a checkpoint and
  returns only when the child has finished. It returns the child's `Ok`
  value, or `Err(TaskError::Cancelled.into())` if the child failed,
  panicked, or was cancelled; the scope reports the actual failure, or
  panics (TASK-2). A join waits no longer
  than the child, and in a cancelled scope the child is itself cancelled.
- Dropping a `Task` without joining is allowed; its failure still reaches the
  scope.

### 6.4 Channels

Channels follow Rust's `std::sync::mpsc`, except that creating one takes
`&Tasks` (§1.2): `channel(tasks)` is unbounded and `sync_channel(n, tasks)`
bounded; `Sender` is `Clone`; a `Receiver` is iterable. `send` on a full
`sync_channel`, `recv`, and `recv_deadline(t)` are checkpoints;
`recv_deadline` returns `Timeout` at `t`. Payloads must be `Send`
(`02-memory.md` SEND-3). An unbounded `channel` in a library is lint
`FX0608`.

Rust's `RecvError` is a unit struct and `SendError(T)` gives no reason, so
neither can say that a checkpoint failed because of cancellation (TASK-3).
The errors of the checkpointing operations are therefore enums with a
cancellation case:

```rust
pub enum RecvError { Disconnected, Cancelled }
pub enum RecvTimeoutError { Timeout, Disconnected, Cancelled }
pub enum SendError<T> { Disconnected(T), Cancelled(T) }  // into_inner(self) -> T
```

Each has `is_cancelled()` and Rust's `Debug`, `Display`, `Error`, `Clone`,
`Copy`, `PartialEq`, and `Eq`; as in Rust, `SendError<T>` has the last four
only where `T` does. `RecvError` converts into `RecvTimeoutError`; Rust's
conversions into `TryRecvError` and `TrySendError` are dropped, since those
have no `Cancelled` case. `Sender::send` never blocks, so it fails only
with `Disconnected`. `try_send` and `try_recv` never block and keep
Rust's `TrySendError` and `TryRecvError`. Iterating a `Receiver` ends at
the first error of either kind; code that must tell a cancelled stream
from a finished one calls `recv`. Rust patterns that name the old shapes,
`Err(RecvError)` and `SendError(v)`, become `Err(RecvError::Disconnected)`
and `e.into_inner()`. A bare `Err(RecvError)` still compiles, as a binding
that catches every error.

A disconnect is not a cancellation and alone sets no echo bit (TASK-2): a
child that returns `Err` because its peer dropped the other end, and
whose bit is clear, fails independently.
A pipeline stage therefore returns `Ok` on a disconnect, so it cannot
outrank the failure that dropped its peer.

### 6.5 `Shared::with`

```rust
impl<T: Send + Pure> Shared<T> {
    pub fn new(v: T, tasks: &Tasks) -> Shared<T>;
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError>;
    pub fn with2<U: Send + Pure, R>(a: &Shared<T>, b: &Shared<U>,
        f: impl FnOnce(&mut T, &mut U) -> R + Pure) -> Result<R, LockError>;
    pub fn with_all<R>(cells: &[Shared<T>], f: impl FnOnce(&mut [&mut T]) -> R + Pure)
        -> Result<R, LockError>;            // provisional name for draft 2's lock_all
}
pub enum LockError { Aliased, Poisoned }
```

These methods replace draft 2's `lock` keyword (D8).

- **LOCK-1. Payloads.** `T: Send + Pure` (`FX0606`). A cell never holds a
  capability, channel end, or `Shared` handle.
- **LOCK-2. Identity.** Before acquiring anything, `with2` and `with_all`
  compare the cells' identities. If two operands name the same cell, the call
  returns `Err(LockError::Aliased)` and `f` does not run.
- **LOCK-3. Order.** Cells are acquired atomically in a global order by cell
  id, a process-unique counter assigned by `Shared::new` and deterministic
  under the simulator. Acquisition waits per cell in FIFO order. It is not a
  checkpoint: a task in a cancelled scope still acquires, runs `f`, and
  observes cancellation at its next checkpoint.
- **LOCK-4. Closure.** `f` must be `Pure` (`FX0603`): it captures no
  capability, channel end, or `Shared` handle, so nothing it reaches holds
  authority, including the `fmt::Arguments` that `format!` and the sinks
  build in it, and it cannot suspend, touch a capability, or call `with`
  again. `return` and `?` leave `f`. Sinks are allowed; printing is lint
  `FX0607`.
- **LOCK-5. Result.** Each method returns `Result<R, LockError>`, where `R` is
  `f`'s result type.
- **LOCK-6. Poisoning.** A panic in `f` poisons every cell it held. A later
  call naming a poisoned cell returns `Err(Poisoned)` without running `f`.
  `cell.reset(v)` replaces the payload and clears the poison; it acquires
  the cell like `with` and is a scheduling point. No safe API reads a
  poisoned payload.
- **LOCK-7. Invariants (v1).** A cell's invariant is its payload type's
  invariant. `f` may assume it at entry and must restore it at exit
  (`04-contracts.md` §4.4).

`with` cannot deadlock against `with`: no closure acquires, every call takes
its whole set atomically in one global order, and every closure finishes
without suspending unless it diverges. Programs can still deadlock through channels;
the simulator reports a global deadlock with the blocked-task graph (§9.3).
Condition waits are channels; reader-writer and hand-over-hand locking live
in `std::sync::raw`.

### 6.6 `Send`, `Sync`, and task isolation

`Send` and `Sync` are derived structurally as in Rust (`02-memory.md` §8.1).
They are checked at task boundaries only (SEND-3): spawn captures, child
results and errors, channel payloads, `Shared` payloads, and parallel
branches. A value that never crosses a boundary need not be `Send`, even
when the evented runtime (v1) migrates its task between OS threads. That is
sound because safe code reaches no thread-local state (GLOBAL-3), the
scheduler's hand-off orders the task's accesses, `ThreadBound` values pin
(§5.4), and no foreign frame is live across a suspension point (FFI-6).
Parallel branches run simultaneously and so require `Send`.

## 7. Compiling Suspending Functions

### 7.1 Which functions may suspend

- **SUSP-1.** A function instance is classified may-suspend if and only if,
  after monomorphization, some parameter or capture type is not `Pure`. A call
  through a closure, function pointer, or `dyn` value may suspend exactly
  when the callee value's type or any argument's type is not `Pure` (D7):
  a captureless closure or a `fn` pointer is `Pure`, but `f(net, url)` still
  passes `&Net`. This is the semantic rule and the interface flag;
  authority-free code never suspends. A call from an instance SUSP-1 does
  not classify may-suspend never suspends either: every value it calls or
  passes that is not `Pure`, such as the `Formatter` that `to_string`
  passes to `T::fmt`, was built from `Pure` values (§3.3). How the evented
  runtime lowers such a call from a native frame is open (§7.2).
- **SUSP-2.** Generic code is classified per instance. `map(xs, |x| f(net, x))`
  gets a may-suspend instance because the closure captures `&Net`;
  `map(xs, |x| x + 1)` gets an ordinary one.
- **SUSP-3.** As a code-generation refinement, the compiler may compile an
  instance as non-suspending when its body, transitively, contains no
  suspending operation. The refined flag of a public item is codegen data in
  the interface: a change regenerates dependents' code but does not re-check
  them.

### 7.2 Lowering under the evented runtime (v1)

This section states the lowering the evented runtime needs. Under D18 rustc
lowers only `async fn` to state machines, so how Tide builds it without
exposing coloring to users is open; failing that is a reversal condition
of D18.

- Each may-suspend instance becomes a **frame record** (the locals live
  across suspension points, plus a state index) and a **resume function**
  that switches on the state.
- Frames are allocated LIFO in the task's **frame stack**: a list of heap
  chunks that starts at 1 KiB and grows by doubling. Frames never move, so a
  reference into a frame, including a child's borrow of its parent's locals,
  stays valid across suspension without `Pin`.
- Without recursion or may-suspend `dyn` calls below a spawn, the first
  chunk is sized to the deepest frame chain exactly. Recursive calls allocate
  in the chunk list; a task over its cap (default 8 MiB) aborts the
  process, as stack overflow does (PANIC-1).
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
function and suspension blocks the OS thread. The simulator runs on
baton-passed threads (§9.1). Foreign code calls only authority-free Tide code
(FFI-6), which is never lowered, and wrapper calls that block run on the
blocking pool while the task's frame waits (FFI-5).

## 8. Deterministic Parallelism

### 8.1 Forms and admission

`join(f, g)` (`std::par`) runs two closures and returns both results; slices
offer `par_map`, `par_for`, `par_chunks_mut`, `par_sort` (a stable merge sort
whose splits and merges depend only on the input length, D9), and, in
v1, `par_reduce` (§8.3). Draft 2's `par { }` block is removed (D9).
Branches may borrow the enclosing frame, because `join` returns only after
both finish. A branch is not a task: it has no checkpoints and cannot be
cancelled.

- **PAR-5. `join` closures.** `join`'s closures are
  `FnOnce + Pure + Send`, each called once. A closure that is not `Pure`
  or not `Send` is `FX0610`.
- **PAR-6. Shared closures.** The slice methods call one closure from
  several branches at once, so it must be `Fn + Pure + Send + Sync` (D9),
  and so must `par_sort`'s comparator: `Sync` rules out a moved-in `Cell`
  or `RefCell` that two branches could mutate. A closure that misses any
  of the four is `FX0610`.
- **PAR-7. Captures.** Every closure may capture shared data, a `&T`
  borrowed or moved in, only if `T` is `Pure + Sync` (`FX0611`, reported
  in place of PAR-5's and PAR-6's `FX0610`), and may hold `&mut` captures
  only as disjoint splits, which the borrow checker verifies.

- **PAR-4. Element types.** The slice methods require elements that are
  `Pure`, plus `Sync` when branches share them and `Send` when branches
  split or move them (D9): `par_map` requires `T: Pure + Sync`, and
  `par_for`, `par_chunks_mut`, and `par_sort` require `T: Pure + Send`
  (`FX0612`). With the closure and capture rules, every branch is
  authority-free: none can reach a `Shared` cell, a channel end, or a
  capability, so none can synchronize, block, or do I/O.

### 8.2 Semantics

- **PAR-1.** When no branch panics, a parallel call's values, its writes
  through `&mut`, and its sink output are identical to sequential evaluation
  with branches left to right (and, in v1, `par_reduce` defined by the
  canonical grouping G(n), §8.3), on any core count, runtime, or schedule,
  up to resource exhaustion. When a branch panics, the panic propagates (D5)
  as PAR-2 says, and every place the region could write has unspecified,
  memory-safe contents that only destructors observe (D9): a branch to the
  right of the panic may or may not have written before it stopped.
- **PAR-2.** If several branches panic, the panic first in sequential order
  is reported and propagates in the enclosing task; a later branch's panic
  waits for every branch to its left, and buffered sink output from branches
  after the first panicking one is discarded (D9). A panicking branch sets a
  stop flag that branches to its right check at their forks and block
  boundaries. If a branch to the right of the first panic diverges without
  reaching one, the region may diverge where the sequential program would
  have panicked.
- **PAR-3.** Floating-point code is never contracted to FMA or reassociated
  unless written so (`mul_add`). Every operation that reads a float's sign
  or bits treats any NaN as one canonical NaN, `const fn` evaluation
  included, while arithmetic stays native (D3). Rust's inherent observers
  (`to_bits`, `is_sign_positive`, `is_sign_negative`, `copysign`, `signum`,
  `total_cmp`, `min`, `max`) and math methods are outside the allowlist
  (ADMIT-1); std's `float::bits`, `float::min`, and `float::max` take their
  place, and its `math::sin`, `math::exp`, `math::ln`, `math::powf`, and the
  other functions beyond IEEE's basic operations use one math library for
  every target (D15), never the platform's libm. Each is generic over
  `f32` and `f64`. The basic operations `sqrt` and `mul_add` are correctly
  rounded, so they stay methods; on a CPU without hardware FMA, `mul_add`
  runs the toolchain's correctly rounded software `fma` (rustc's
  `compiler_builtins` today), never the platform's libm.
  `from_bits` observes nothing and stays admitted (D3). Formatting, byte
  conversions, and foreign calls are audited against the same rule.
  The automatic parallelization pass (later tier) only runs independent
  authority-free calls in parallel; it never regroups a fold.

### 8.3 `par_reduce` and the canonical grouping (v1)

In v0, code `par_map`s the expensive part and folds sequentially. In v1,
`xs.par_reduce()` takes a slice of `M: Combine` (`reference-companion.md`
Appendix A) and is **defined** as `tree_fold(G(xs.len()), xs)`, where G is:

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
buffers in branch order, discarding those after the first panicking branch
(SINK-4). Parallel branches run on one process-wide work-stealing pool sized
by `par-threads`, shared with evented workers (v1) and reduced by any
`Accel` budget (later). An unpromoted fork costs about a branch. v0 promotes
by lazy binary splitting: a fork exposes its right branch only when the
local deque runs low. Heartbeat promotion, checked at forks and block
boundaries so loops need no polling, replaces it if E5 justifies it.
When no branch panics, scheduling is unobservable, so the policy can change
freely. Under the simulator, parallel regions run sequentially, branches in
sequential order (D9), so a seed replays exactly even when a branch panics.

## 9. Deterministic Simulation

### 9.1 Engines

A simulation is defined by its scheduler decisions, so any engine that makes
them at the same points can run it. v0 has one: baton-passed OS threads, one
runnable at a time, in a rustc debug build (D18). The evented lowering on one
thread may become a second, for long `tide sim` runs, once the evented
runtime exists (v1), and an interpreter a third if one is built (D12). v0
overflow traps are precise in every engine (D3), and every engine runs
parallel regions sequentially (§8.4), so a seed yields one trace in all of
them; conformance tests check it.

### 9.2 `Root::sim`

`Root::sim(seed)` returns a `SimRoot` with the same capability fields as
`Root`, all simulated, plus `sim.world` for setup (`file`, `args`,
`env_var`, `stdin`, `http` peers, `proc` models), `sim.faults` (§9.4),
`sim.node` (§9.5), and `sim.elapsed()`, `sim.records()`, and `sim.trace()`
for assertions.
Setup follows Rust's builders: `world.args(impl IntoIterator<Item:
AsRef<OsStr>>)` and `world.env_var(k: impl AsRef<OsStr>, v: impl
AsRef<OsStr>)` as `Command::args` and `Command::env` take them, and
`world.stdin(impl AsRef<[u8]>)` and `world.file(path, contents: impl
AsRef<[u8]>)` take bytes or text. `world.http(addr, peer)` takes `peer:
impl Fn(&Request) -> Reply + Send + 'static`: the simulator calls it from
whichever task's thread sends the request, so state it shares with the test
body crosses tasks and must be a synchronization handle made from
`sim.tasks`, such as a `Shared<u32>` counter, not an `Rc<Cell<u32>>`.
The test body runs as the simulation's main task. `#[sim_test(seeds = N)]`
gives a test a fresh `SimRoot` per seed; `tide test` runs 100 seeds by
default and `tide sim --test` runs more.

### 9.3 Time and scheduling

- Monotonic time starts at zero, wall time at 2000-01-01T00:00:00Z.
  Computation takes no virtual time; capability operations take modeled
  latencies drawn from the seed, which cancellation cuts short only where
  TASK-3 lets it: a file-system call's latency runs out first. Simulated
  standard input behaves as a pipe. Without faults, each network packet takes
  1–10 ms, and a `Net::get` to a simulated peer is one round trip, two
  packet delays (2–20 ms), plus the reply's `after(d)`; connection setup and
  name resolution are inside that round trip. An address with no peer gives
  `Refused` after one round trip, and a host that no peer or node names
  gives `Dns` at once.
- Virtual time advances only when every task on every node is blocked, and
  then jumps to the next timer or delivery. If no task can ever run again,
  the run fails with a global deadlock report: each blocked task, the span of
  its blocking operation, and what it waits for.
- **Scheduling points** are every capability operation, every handle
  operation (`spawn`, `join`, `send`, `recv`, `Shared::with*` acquire and
  release, `reset`, atomics, `Weak::upgrade`), and every checkpoint. Between two points a task
  runs alone. Because safe code is data-race-free, interleavings at
  synchronization operations reach every outcome the simulator's model can
  observe (§9.8 lists what it leaves out), so this granularity loses
  timings, not modeled behaviors.
- Strategies: `random` (default), `pct(depth)` (probabilistic concurrency
  testing; not in v0 per `reference-companion.md`), and `fifo` for debugging.

### 9.4 Fault injection

| `sim.faults` method | Effect |
| --- | --- |
| `net_drop(p)`, `net_duplicate(p)`, `net_delay(min, max)`, `net_reorder(on)` | per-message faults |
| `partition(&[&a], &[&b, &c])`, `heal()` | cut links between node groups; connections time out or reset |
| `disk_error(prefix, p, kind)` | `EIO`, `ENOSPC`, and similar on matching paths |
| `torn_writes(on)` | whether a crash may also tear unsynced writes, keeping a seed-chosen prefix; on by default (D15); when off, unsynced writes are only lost |
| `clock_skew(&node, ppm)` | per-node clock drift |
| `crash(&node)`, `restart(&node)` | §9.5 |
| `at(t, fault)` | schedule any fault at virtual time `t` |
| `fault_point(name, p)` | probability that `tasks.fault_point(name)` returns true |
| `accel_crash(p)` | Bend kernel failures (§10) |

`tide sim` defaults to swarm testing: each seed enables a random subset of
faults at random rates.

### 9.5 Nodes

`sim.node("b")` creates a node with its own filesystem, network address
(`"b"` resolves to it), clock skew, environment, and task tree.
`node.start(entry)` runs `entry(NodeRoot) -> Result<T, E>` as the node's
main task and returns a handle to it; a `NodeRoot` has `Root`'s capability
fields. The entry is a `fn` item or an `Fn` closure with `Owned` captures
(D15); the closure is also `Send`, and its captures are `Pure + Sync`, so a
run can change nothing it captured. Nodes share no memory and interact only
through the simulated network and filesystem.

A crash is not a cancellation: the node's tasks stop at their current
scheduling points without running destructors, its memory is freed, and
peers' connections reset after a modeled delay. Data made durable by `sync`
or `sync_dir` (§1.1) survives, provided its entry and the entries of the
directories above it are durable too, since a directory whose entry is lost
takes its contents with it; other writes may be lost or torn, as the seed
decides (D15, §9.4). Each unsynced directory-entry change, a `create`,
`rename`, `remove_file`, `remove_dir`, or new subdirectory, survives whole
or is lost, never torn; `remove_dir_all` is one such change per path it
removes, so a crash can leave part of the tree. After a crash, a `rename`
either happened or did not. Either the target names the renamed file and the
source name is gone, or both are as they were, so a replaced target names
the old file or the new one, never neither. Which unsynced changes survive
is otherwise unspecified, except that the surviving set is one a file system
could reach: a change survives only if the changes that created its entries
and every directory above its paths do; a change that takes a name survives
only if the change that freed the name does; and a directory's removal, by
`remove_dir` or by a `rename` over it, survives only if every change that
emptied it does, each removal in it and each `rename` out of it. A
`sync_dir` that makes a directory's removal durable makes those changes
durable too, because a file system cannot remove a directory before emptying
it. Likewise every other change a durable change depends on is made durable
with it, except one: durability never flows from synced data or a synced
entry to the unsynced creation of its own entry or of a directory above it.
So a program may rely only on what `sync_dir` made durable and on each
change being whole. This is POSIX's rename atomicity carried across a crash,
which journaling file systems provide in practice and every atomic-replace
protocol needs. `restart` reruns the entry with a fresh `NodeRoot` over the
surviving durable storage; no volatile state survives. `sim.run_for(d)` and
`sim.run_until(&handle)`, which returns the entry's `Result<T, E>`, drive
the world; `node.fs()` inspects a node's disk.

### 9.6 `tide sim`

```text
tide sim [<bin> -- args | --test <name>] [--seeds N | --seed S] [--time 10m]
          [--strategy random|pct] [--faults swarm|none|<plan.toml>]
          [--minimize] [--allow-unmodeled] [--json]
tide sim --replay <manifest>
```

`<bin>` runs `main` as a single node with a simulated root. Output is one
NDJSON record per failing seed and a summary:

```json
{"kind":"sim_failure","target":"kv::tests::acked_writes_survive","seed":3141,
 "commit":"e9d8ed4","toolchain":"tide 0.4.1","virtual_time":"0.704s","steps":18342,
 "failure":{"type":"panic","message":"acked k17 missing on backup",
            "span":"src/kv.rs:121:13","node":"client","task":"0"},
 "faults":["partition#1 at 0.200s"],
 "replay":"tide sim --replay target/sim/acked_writes_survive-3141.manifest.json",
 "trace":"target/sim/acked_writes_survive-3141.ndjson","trace_hash":"9f2c…"}
```

Each trace event records virtual time, node, task, kind (`cap`, `sched`,
`fault`, `sink`, `spawn`, `exit`, `cancel`, `cell`, `chan`, `assert`), the
operation with its source span, its arguments, and its result.

- **REPLAY-1. Manifest.** Every simulated run, under `tide sim` or
  `tide test`, writes a replay manifest (D15): the seed; the content hashes
  of the source and of every dependency (D14's snapshot hashes, so an
  uncommitted edit changes them); a semantic build identity, which is the
  compiler version, the target, and every option that affects semantics,
  such as the profile and `cfg` settings; the simulated root's
  configuration (program arguments, environment, files with their
  contents, scheduler strategy, and fault plan, all inline, never as host
  paths); and every injected input, appended as the run goes (PANIC-1).
- **REPLAY-2. Identity check.** `tide sim --replay <manifest>` refuses to
  run, before executing anything, unless the current source hashes and
  build identity equal the manifest's; it then reruns exactly what the
  manifest records, so a failing seed replays bit for bit. The replay
  command printed for a failing `tide test` seed is this command. `--seed`
  on `tide sim` or `tide test` starts a fresh run of one seed against the
  current source; it is not a replay.

`--minimize` (not in v0 per `reference-companion.md`) delta-debugs the fault
plan and the scheduler's preemptions toward a shorter failing trace.

### 9.7 Determinism self-check

The first seed of every run, and every failing seed, runs twice and the
trace hashes are compared. On a mismatch `tide sim` reports the first
diverging pair of events with spans. The usual causes are an unmodeled
foreign effect, a false `#[trusted_pure]` claim, unsafe code observing
addresses, or a toolchain bug.

### 9.8 Coverage and limits

The replay guarantee holds only when every reachable foreign effect has a
model (D15). The sim build lists the ones that do not (`FX0514`, §5.2). For
imported Rust crates, sim builds interpose the libc entry points for time,
randomness, and environment and route them to the simulator; file and socket
calls from an unmodeled crate fail with `EPERM` (v1). Runs that use
`--allow-unmodeled` are marked non-replayable.

Not simulated: CPU time and performance; memory exhaustion; stack depth;
parallel speed-up (parallel regions run sequentially, §8.4);
hardware faults beyond injected disk errors; foreign code's internals and
threads; kernel behavior beyond the models; signals; timing side channels;
real child processes, which need a `sim.world.proc` model.

## 10. The Bend Boundary (later tier, D16)

```toml
[bend-dependencies]
life = { bend = "=2.0.37", src = "kernels/", mode = "process" }  # "inproc" needs grants = ["ffi"]
```

```rust
#[bend(dep = "life", def = "Life.steps", reference = life_steps_ref)]
#[ensures(result.is_ok() ==> result.unwrap().len() == board.len())]
#[bend_law("kernels/LAWS.bend", "steps_keep_size")]
pub fn life_steps(accel: &Accel, board: BendArray<u32>, n: u32) -> Result<BendArray<u32>, BendFault>;
```

- **Build.** The pin is exact. The build runs the pinned `bend`, caches by
  the hash of sources, laws, proofs, and version, and generates a shim that
  calls the def at exactly the declared types, so Bend's checker rejects a
  mismatch. Defs that are IO-typed or depend on foreign or `@unsafe` defs are
  rejected. An unsupported host gets one diagnostic, or the `reference`.
- **`Accel`.** Calls require `&Accel`, which only `root.accel` provides. In
  process mode the supervisor runs at most `budget` kernel servers;
  in-process mode serializes calls, because Bend's runtime is
  process-global. The budget is subtracted from the `join`/`par_*` pool. A
  call is a checkpoint. In process mode, cancelling it abandons the request,
  restarts the server, and returns `BendFault::Cancelled`, unless the reply
  has already arrived (TASK-3). In-process mode cannot stop a running
  kernel, which is a foreign call (TASK-5): it finishes and returns its own
  result, and the task observes the cancellation at its next checkpoint; a
  call still waiting for the serialized runtime returns
  `BendFault::Cancelled`.
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
  arithmetic wraps; Tide's checks stop at the boundary.
- **Failures.** `BendFault` is `Crashed { code, stderr_tail }`,
  `FailStop(n)`, `Limit(what)` (from pre-validation of Bend's known limits),
  `Protocol` (version or schema mismatch), `Unavailable`, or `Cancelled`,
  whose `is_cancelled()` is true (§1.1). A kernel that returns Bend's
  `Result` is declared `-> Result<Result<T, E>, BendFault>`.
- **Simulation.** Kernel results are pure, so under the simulator the call
  runs the `reference` implementation if one is declared, and otherwise the
  real kernel outside virtual time, recording the result's hash. Latency is
  modeled, cancellation follows the declared mode, and `accel_crash`
  exercises the failure paths.
- **Imported claims.** `#[bend_law(file, law)]` attaches to the `ensures`
  directly above it, which keeps that clause one expression (04 §1.2 rule 1).
  The claim is accepted only if Tide can render the clause as a Bend law
  (first-order fragment), the pinned `bend` verdict passes, and the verdict
  lists no foreign or `@unsafe` dependency. The claim enters the trust surface
  as `external: bend` with version and hashes in the lockfile. It never counts
  as Tide-proved; a Tide proof may use it only as a counted assumption. Under
  `assert` and `test` the clause is still checked at run time where
  executable.
- **Differential tests.** With a `reference`, `tide test` property-tests the
  kernel against it; deterministic `par_map` makes exact comparison meaningful.

## 11. Rust Crates

Imports (RUST-1 to RUST-5) are v1; RUST-6 is the other direction, also v1.

```toml
[rust-dependencies]
regex = { version = "1.11", authority = [] }
rusqlite = { version = "0.32", authority = ["fs.read", "fs.write"], build = true }
```

- **RUST-1. Direct linking.** A Tide crate links synchronous Rust crates
  directly (D13, D18), with no C ABI between them. Async, macro-driven, and
  lifetime-heavy APIs (tokio, serde's `'de`) are out of scope, unless a
  hand-written Rust adapter crate, imported the same way, wraps them in a
  synchronous API.
- **RUST-2. Authority.** With `authority = []`, every wrapper is
  `#[trusted_pure]` and carries FFI-3's claims. Otherwise every free function
  and constructor wrapper takes a capability for each declared kind, and the
  opaque types they return are resource handles: not `Pure`, carrying that
  authority, so their methods need no further parameters. Per-item overrides
  narrow this (`items."Connection::open_in_memory" = []`).
- **RUST-3. Linked-symbol scan.** After the build, the linked objects' symbols are
  mapped to kinds: `open`, `stat`, `unlink` to `fs`; `socket`, `connect`,
  `getaddrinfo` to `net`; `clock_gettime` to `clock`; `getrandom` to `rand`;
  `getenv` to `env`; `fork`, `execve`, `posix_spawn` to `proc`;
  `pthread_create` to `tasks`; `dlopen` and raw `syscall` to `foreign`. A
  kind found but not declared fails the build (`FX0515`) with the symbol and
  object.
- **RUST-4. Threads and panics.** Rust types that are not `Send` import as
  `ThreadBound`; `Send` and `Sync` otherwise carry over. A crate's panic
  unwinds into the calling task like any panic. Callbacks follow FFI-6.
- **RUST-5. Build scripts** run only with `build = true`, in a sandbox: no
  network, read-only sources, a writable output directory only, a cleared
  environment, and CPU and time limits. Outputs are hashed into the
  lockfile, a rebuild with different output warns, and each script is a
  counted trust hole (D13). The crate's proc macros run in the same sandbox.

What this cannot guarantee: a present symbol may be unreachable, and the
scan misses inline-assembly syscalls and inherited descriptors; Rust code
ignores attenuation (FFI-4); Rust `unsafe` may be unsound; simulation through
libc interposition is best-effort; a build script can emit malicious code
into its outputs; threads a crate spawns escape Tide scheduling. The trust
surface marks each crate `rust: declared, scan-checked, unverified`.

- **RUST-6. Rust consumers.** A Tide crate is a Rust crate, so Rust code
  depends on it as on any crate; the consumer's manifest and its own `std`
  are unchanged. The Tide crate's exported manifest injects Tide's std with
  `std = { package = "tide_std" }`, for that crate alone. Types Tide's std re-exports from Rust's
  (`Vec`, `String`, `Box`, `Option`, `Result`, slices, `&str`) are the same
  types on both sides; Tide-owned types (`HashMap`, `HashSet`, `Arc`,
  channel ends, the capability types) are distinct and need adapters at the boundary. A Rust
  caller gets capabilities only from a Tide-written entry point, and the
  closures and trait impls it passes in are unchecked trust holes (§3.1).

## 12. Worked Examples

### 12.1 A command-line tool

```rust
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

#[tide::main]
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

```rust
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("task: {0}")]
    Task(#[from] TaskError),
}

/// Every per-URL failure, the deadline's cancellation included, is a value,
/// so the call returns partial results; a panic still propagates. `tasks`
/// must be unlimited: under `limit`, a child not started by the deadline
/// never starts and fails the scope (§6.1).
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
    let r = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls, Duration::from_secs(5)).unwrap();
    assert_eq!(r[0].as_deref().ok(), Some(&b"a"[..]));
    assert!(matches!(r[1], Err(NetError::Cancelled)));
    assert_eq!(sim.elapsed(), Duration::from_secs(5));   // virtual time; the test takes milliseconds
}
```

Children borrow `urls` and `net` from the caller, and `sim.tasks` is
unlimited. At five virtual seconds the scope deadline fires (TASK-4) and
the slow child's `get` returns `NetError::Cancelled`, which the child
returns inside `Ok`. No child returns an error, so the deadline does not
fail the scope: both joins return and `scope` returns the partial results,
one per URL (D8).

### 12.3 A replicated key-value node under a partition

```rust
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
            assert!(backup.fs().exists(key).unwrap(), "acked {key} missing on backup");
        }
    }
}
```

During the partition the replication deadline fires, and the `connect` or
`frame::read` in `put`'s scope returns its cancellation as an error. The body
returns that error through `?`; the scope has no children, so `scope`
returns the body's result (TASK-2), `put` returns `Err`, and the client gets
`Fail`. A panicking `handle` would make the server's scope panic
(TASK-1); a server that must survive one runs each request through
`tasks.spawn_isolated` with owned data. A version that acknowledged before replicating
fails on some seed with the §9.6 record, and its replay command reproduces the
run exactly.

### 12.4 A bank transfer with `Shared::with2`

```rust
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
        return Ok(());                                   // otherwise `with2` returns Err(Aliased)
    }
    Shared::with2(from, to, |a, b| {
        let debited = a.balance.checked_sub(amount).ok_or(BankError::Insufficient)?;
        let credited = b.balance.checked_add(amount).ok_or(BankError::Overflow)?;
        a.balance = debited;                             // both checks pass before any write,
        b.balance = credited;                            // so an error leaves both cells unchanged
        Ok(())
    })?
}

#[sim_test(seeds = 1000)]
fn money_is_conserved(sim: SimRoot) {
    let accounts: Vec<Shared<Account>> =
        (0..4).map(|_| Shared::new(Account { balance: 100 }, &sim.tasks)).collect();
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
    let total: u64 = accounts.iter().map(|c| c.with(|a| a.balance).unwrap()).sum();
    assert_eq!(total, 400);
}
```

Pairs arrive in both orders; LOCK-3's global order prevents deadlock, and
each seed explores another interleaving. Checking both amounts before any
write means an early `?` never leaves a half-applied transfer.

## 13. Open Questions and Required Changes

### 13.1 Open questions

1. Read-mostly data and counters: a read-only `with`, snapshot cells,
   sharded counters (`reviews/round1-performance.md` N3).
2. Whether one enum through one channel replaces `select` well enough.
3. How a library exported through the C ABI, called from C or Bend,
   receives capabilities; probably a generated init entry that builds a
   `Root` for it. Rust callers get them from a Tide-written entry point
   (RUST-6).
4. The porting cost of GLOBAL-6 (`Arc` creation taking `&Tasks`,
   `Arc::get_mut`, `try_unwrap`, counts).
5. FFI-6 excludes callbacks that need effects, such as GUI event loops.
6. Integration tests that need the real filesystem, since ROOT-1 and ROOT-5
   keep a real `Root` out of tests.
7. Benchmarks for G(n)'s constants, the 100 ms watchdog, and baton-passed
   threads in large simulations.
8. Whether read versus write should be a type distinction on `Fs`.

### 13.2 Changes this chapter needs elsewhere

Draft 3.1 and the current reference apply this chapter's earlier requests:
`LockError` with `Aliased`, `TaskError::DeadlineExceeded`, shielded joins,
lazy splitting in D9 with PAR-2's divergence qualification, `Accel` in D7,
and `Shared::new` taking `&Tasks`. Outstanding:

- 01's open question 4 is answered by `#[pure]` and `FX0511` (§3.2).
