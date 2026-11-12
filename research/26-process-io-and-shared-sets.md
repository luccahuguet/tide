# Process I/O, sinks, exit codes (fx-5vj) and multi-cell Shared (fx-ugm)

Research report, 2026-11-10. Read: AGENTS.md; beads fx-5vj, fx-ugm, fx-ecb, fx-nkv.4/.5/.6,
fx-o8j, fx-zc1; `design/03` §1.1–1.3, §2.1, §2.4, §6.5, §9; `design/reference.md` §6.1, 6.2, 6.5,
6.8, 7.4; companion 16–17; `crates/tide-std/src/{root.rs,lib.rs,sync.rs,io/mod.rs}`; the
conformance stub std; `.worktrees/fx-nkv.5` (cell identity only). Nothing under
`trials/e1-checked/` was opened, `protocol.md` included, so I could not check E1-P7/E1-P3's exact
surface lists; the coordinator should compare §10.1 with the recommendations marked **[surface]**.

Citations marked *(recalled)* are stable official URLs I cite from memory without re-fetching
them in this run; all others were confirmed by search in this run.

## Summary of recommendations

| # | Choice | Recommendation |
| --- | --- | --- |
| A1 | Sink transport | Keep today's synchronous write-whole-record-under-lock-and-flush; no writer thread in v0 |
| A2 | `Root::log` type | `LogConfig`, private field, `&self` setters `level`, `format`, `to_file`; not `Clone`; text line `level: message` |
| A3 | `stdout()`/`stderr()` | Tide's own `Stdout`/`Stderr` implementing Tide `io::Write`, always `Ok`; one `write` call = one locked write |
| A4 | `Env` | Tide's own `Args`/`Vars` iterators; Rust's panics on non-Unicode; denied variable reads as `NotPresent`; `only` intersects |
| A5 | `Stdin` | One process-wide buffered reader; `lines()` returns `io::Lines<StdinReader>` pulling one line per fill; cancellation checked at entry (soft) |
| A6 | `main` and exits | Keep the `Termination` bound; 0 / 1 / given / 101 / 141 fall out of A1 |
| A7 | Simulation | Inputs in the simulation state; sink events raw; `records()` assembles lines per node and stream |
| B1 | Acquisition | Ordered blocking by cell id, ticket reserved lazily per cell; no try-and-back-off, no global lock |
| B2 | Cell id | `u64` process counter (real), fx-nkv.5's simulation-local id (sim); cross-domain operands panic |
| B3 | Aliasing | Pointer identity, checked before any ticket; `Aliased` beats `Poisoned` |
| B4 | Poison, `reset` | First poisoned cell ends acquisition; panic poisons all held; `reset` swaps under the lock, drops the old payload after release |
| B5 | Simulation | One point before the whole set, one after; per-cell `cell` events in id order |

---

## Part A — fx-5vj

### A1. Sink transport: writer thread or synchronous writes

03 SINK-4/5 describe a "process-wide buffer" drained by a "writer thread". `__sink` today formats
the whole record, writes it under the stream's lock and flushes (`root.rs`).

- **Option 1, synchronous (current).** Each record is one `write_all` under Rust's stream lock.
  Rust locks stdout per `print!` call, so records from different threads never mix, though their
  relative order is arbitrary
  (https://users.rust-lang.org/t/synchronization-of-writing-to-stdout-in-threads/18803;
  source: https://doc.rust-lang.org/src/std/io/stdio.rs.html). Nothing is buffered at exit,
  which matters because `process::exit` runs no destructors
  (https://doc.rust-lang.org/std/process/fn.exit.html *(recalled)*;
  https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/raii/drop_skipped.html).
- **Option 2, writer thread with a bounded queue.** This is what async log appenders do, and
  their documented cost is loss at exit: `tracing-appender`'s `non_blocking` is lossy by default
  and needs a `WorkerGuard` held until exit to flush
  (https://tracing.rs/tracing_appender/non_blocking/). Every exit path (return, `Err`, panic,
  141, later `Proc::exit`) would need a drain, and Rust's own `Termination` print of
  `Error: {e:?}` goes straight to fd 2, past the queue, so it could overtake queued records.

**Recommend option 1.** The observable contract in SINK-4/5 is: program order within a task,
emission order across tasks, `out`/`err` never dropped, 141 on a closed stdout, other failed
writes discarded. Synchronous writes meet all of it; "blocks until drained" becomes "blocks in
`write`", and SINK-4's wait before starting a child is trivially satisfied. `log` "drops under
overload" permits loss but does not require it. The writer thread is mechanism, not contract
(AP-CONTRACT-001); it belongs with the evented runtime (v1). Record this reading in 03 §2.4.

Line atomicity across *processes* is not promised either way: POSIX guarantees non-interleaving
only for pipe writes of at most `PIPE_BUF` bytes (https://man7.org/linux/man-pages/man3/write.3p.html;
https://nullprogram.com/blog/2016/08/03/), and `write_all` may split a long record into several
`write` calls. Within the process the stream lock makes each record contiguous. Do not claim more.

### A2. `Root::log`: type, methods, defaults, text format

03 §1.3 names the type (`log: LogConfig`); `root.rs` says no document fixes it. What is missing
is the method set, receivers, value types and the text/JSON line shapes. SINK-2 gives the
capabilities: levels, text or JSON, `to_file(file)`; and the defaults.

Options for shape:
1. **`&self` setters over process sink state**: `root.log.level(Level::Debug)`,
   `root.log.format(Format::Json)`, `root.log.to_file(file)`. Matches `to_file`'s spelling in
   SINK-2 and the capabilities' `&self` rule (reference 6.1).
2. Consuming builder (`root.log.level(..).apply()`): a forgotten `apply` silently does nothing.
3. Plain public fields: cannot hold a `File` destination coherently and invites reads, which
   SINK-1's "nothing a sink returns depends on the world" discourages.

**Recommend 1**, with: `pub struct LogConfig` (private field, no `Clone`, no `Default`, no
getters); `enum Level { Error, Warn, Info, Debug, Trace }` and `enum Format { Text, Json }` in
`std::log` *(module path is my choice; the design names none)*; `level` sets the threshold for the
`log` sink only. The `log` crate (`set_max_level`, https://docs.rs/log/latest/log/fn.set_max_level.html
*(recalled)*) and `tracing` use the same one-threshold model.

"Configured only in `main`" (SINK-2): ROOT-2 lets fields be moved out one by one, so a
`LogConfig` could in principle be handed to a callee. The design text does not say whether that
is forbidden. Do not add a rule; make `LogConfig` not `Pure` in tide-admit's table (it configures
process state, so it should not hide inside "authority-free" data) and flag the question on
`fx-djp`. `SimRoot` has no `log` field in 03 §9.2 or the stub; keep it that way: under simulation
every record is captured whatever its level (SINK-6 says "every record").

Text shape (design silent). Evidence: the twelve-factor guidance is that an app writes its event
stream unbuffered to stdout/stderr and leaves timestamps and routing to the environment
(https://12factor.net/logs *(recalled)*); a timestamp also makes stderr non-reproducible.
**Recommend** text `level: message\n` (`warn: legacy row 7`), JSON
`{"level":"warn","message":"..."}\n`, hand-written with fixed key order as fx-nkv.6 decided for
traces (no serializer dependency); no timestamp, no target. Mark both provisional in 03.

`to_file` needs `File`, which fx-byk.1 (in flight) owns. If `File` has not landed when fx-5vj
starts, land `level` and `format` only and leave `to_file` labelled "not provided yet" in
`lib.rs`; the Acceptance asks for "root.log defaults", not file destinations.

Log macros (`error!` … `trace!`) come from `tide-macros`, as `print!` does (`root.rs` explains
why `#[macro_export]` cannot be used). Each calls one `__sink::log(level, args)` that formats
first, then discards if disabled (SINK-3). `dbg!`, the `trace` sink and `release-max-level` are
not in the bead's Scope sentence; leave them. **[surface]**

`BufWriter`'s Drop warning (fx-ecb hand-off): emit `warn!` with the byte count only, never the
bytes (they may be secrets, SINK-10's spirit). `shared.rs` is included by the stub too; `__sink`
lives in `root.rs`, also shared, so the path resolves in both.

### A3. `io::stdout()` / `io::stderr()` handles

SINK table: `std::io::stdout()` is a form of `out`. SINK-1: the `impl Write` handles always
return `Ok`.

- Types: Tide's own `Stdout` and `Stderr` (Rust's implement Rust's `Write`, not fx-ecb's).
  `write(buf)` returns `Ok(buf.len())`, `flush` `Ok(())`. Also implement `Write` for `&Stdout` /
  `&Stderr`, as Rust does. 03 §1.1 requires `From<io::Stdout>`/`From<io::Stderr>` for `Stdio`
  later; keep the names.
- Each `write` call goes through the same `__sink` path as `print!` (one locked write, flush,
  141 on `BrokenPipe` for stdout, discard otherwise). Rust's stdout is line-buffered and its
  stderr unbuffered (https://doc.rust-lang.org/std/io/struct.Stderr.html *(recalled)*); flushing
  each write removes the difference and any exit-time flush.
- **Capture pitfall with prior art:** libtest captures `println!` but not
  `io::stdout().write_all(..)`, a long-standing wart (https://github.com/rust-lang/rust/issues/90785).
  Tide must route both through one function so `sim.records()` sees both.
- `lock()`: Rust code often writes `io::stdout().lock()`. The design does not mention it.
  A lock held across scheduling points contradicts "sinks never suspend". Recommend not providing
  it; `BufWriter::new(io::stdout())` is the batching idiom. **[surface]**
- Broken pipe: Rust ignores `SIGPIPE` at startup, so writes fail with `EPIPE` and `println!`
  panics (https://github.com/rust-lang/rust/issues/46016). The current `exit(141)` matches the
  code a shell reports for a `SIGPIPE` death; a parent calling `waitpid` sees a normal exit, not
  a signal. That is what SINK-5 says ("exits quietly with status 141"); keep it. The unstable
  `-Zon-broken-pipe` flag exists (test:
  https://rust.googlesource.com/rust/+/HEAD/tests/ui/process/println-with-broken-pipe.rs) but
  would tie Tide to a nightly flag for no contract gain.

### A4. `Env`

Reference 6.1: "`args()` (program name first), `var(k)`, `vars()`, `current_dir()`, as
`std::env`'s"; attenuations `only([..])`, `args_only()`.

**Return types.** Rust's `env::Args` and `env::Vars` have no public constructor, so the stub's
`pub use std::env::Args` (marked GUESS) cannot serve simulated arguments. **Recommend Tide's own
`env::Args` and `env::Vars`**: `Iterator<Item = String>` / `<Item = (String, String)>`,
`ExactSizeIterator`, `DoubleEndedIterator`, `Debug`, over an owned `Vec`. Not `Pure` is
unnecessary: they hold plain strings. `VarError` can stay Rust's (public variants
`NotPresent`, `NotUnicode(OsString)`).

**Non-UTF-8.** Rust: `args()` and `vars()` panic *during iteration* on non-Unicode data;
`var` returns `VarError::NotUnicode` (https://doc.rust-lang.org/std/env/fn.args.html;
https://doc.rust-lang.org/std/env/enum.VarError.html *(recalled)*). Options: mirror Rust; decode
lossily; add `args_os`. Lossy decoding silently changes file names. The design lists no `_os`
methods. **Recommend mirroring Rust exactly**, panic timing included (at the offending item, not
at `args()`), and adding nothing. Say in 03 that `_os` variants are absent in v0.

**Snapshot.** Tide has no `set_var` (no ambient mutation; Rust 2024 made `set_var` `unsafe`
because the environment is unsynchronized process state:
https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html *(recalled)*), so
the real environment is constant for the process and may be read per call or once.

**Attenuation semantics (design silent on all four points).**
- *A denied variable.* Deno throws `NotCapable` for a name outside `--allow-env=..`
  (https://docs.deno.com/api/deno/~/Deno.Env); WASI gives the guest only what the host passed, so
  an unlisted variable simply does not exist (wasmtime's default context has an empty environment:
  https://docs.wasmtime.dev/api/src/wasmtime_wasi/cli.rs.html). Rust's `VarError` has no room
  for `Denied` without a Tide-owned enum, which the reference's "as `std::env`'s" argues against.
  **Recommend the WASI reading:** `var` returns `NotPresent`, `vars()` omits it.
- *Chaining.* `only(a).only(b)` is the intersection; attenuation never widens (same rule as
  `Net::only`).
- *`only` and arguments.* `only` narrows variables only; `args()` and `current_dir()` stay.
- *`args_only()`.* No variables. `current_dir()`: the name says arguments only, so return
  `Err(PermissionDenied)` (the provisional kind `Fs` uses). This one is a guess; record it as
  provisional.

`only` takes any iterable of strings (as `Net::only` does, reference 6.1). `Env(())` gains a
policy field; `capability!` in `root.rs` is shared with the stub, so change the stub's `Env` in
the same commit.

`current_dir() -> io::Result<PathBuf>` with Tide's `io::Error`; `/` under simulation
(reference 6.3).

### A5. `Stdin`

**Shared buffer.** Reads take `&self` and `Stdin` is `Clone + Send + Sync`, so the buffer is
process-wide behind a mutex, as Rust's is (a global `Mutex<BufReader<StdinRaw>>`,
https://doc.rust-lang.org/src/std/io/stdio.rs.html). Simplest correct real backend: delegate to
Rust's `std::io::stdin()` and map errors into Tide's `io::Error`. Rust's raw stdin already maps
`EBADF` (fd 0 closed) to end-of-file (same source, `handle_ebadf`), which Tide inherits.

**Semantics to keep from Rust.** `read_line` appends, returns `Ok(0)` at EOF, keeps the
newline; on invalid UTF-8 it returns `InvalidData`, leaves the `String` unchanged and has
consumed the line, so the next call continues (https://doc.rust-lang.org/std/io/trait.BufRead.html#method.read_line
*(recalled)*). `lines()` strips `\n` or `\r\n`. A bad line is one `Err` item, then later lines
arrive. The endless-`Err` hazard that clippy's `lines_filter_map_ok` warns about is a *persistent*
error (https://rust-lang.github.io/rust-clippy/master/index.html#lines_filter_map_ok); in Tide
the persistent one is cancellation, which fx-ecb's `Lines` already ends after.

**`lines()` return type.** Rust returns `Lines<StdinLock<'static>>`, holding the lock for the
iterator's life. In Tide that would hold a lock across scheduling points and starve other
readers. `Stdin` itself cannot implement `BufRead` (`fill_buf` must return a borrow into the
locked global buffer). **Recommend `io::Lines<StdinReader>`**, where `StdinReader` (public, private
field, holds a `Stdin` clone and a local `Vec<u8>`) implements Tide's `BufRead`: `fill_buf`, when
its local buffer is empty, takes the global lock, moves **exactly one line** (through `\n` or EOF)
into the local buffer, and unlocks. Because `Lines` consumes whole lines, dropping the iterator
midway loses no input, and two tasks reading lines interleave at line boundaries. Add
`StdinReader` to `NOT_PURE` with kind `stdin` (fx-ecb comment 3).

**Cancellation.** 03 §1.1 lists all reads as waits; TASK-3 says a pipe wait in flight is
cancelled "if the kernel can cancel it". Options:
1. Check cancellation at entry and return `ErrorKind::Cancelled`; a read already blocked in the
   kernel finishes when input or EOF arrives. std-only.
2. `poll(2)` on fd 0 plus a wake descriptor: truly cancellable, needs `libc`/`rustix`
   (AP-DEPENDENCY-001 gate) and non-blocking handling of a possibly shared terminal.
3. A reader thread feeding a queue, callers waiting on fx-zc1's wait primitive: cancellable and
   std-only, but one more thread and read-ahead beyond what the program asked for.

**Recommend 1 for this bead**, stated in 03 as a v0 limit ("a blocked standard-input read
observes cancellation when the read returns"), the same soft reading fx-zc1 recorded for
deadlines. Batch programs with piped input, the Acceptance's case, never notice. Option 3 is the
upgrade once fx-zc1's `wait(kind, ..)` exists; do not build it speculatively. This narrows TASK-3
for one operation, so it needs the design-review round the bead's first comment already demands
for `LogConfig`.

`is_terminal()`: Rust's `IsTerminal` on the real side; `false` under simulation ("simulated
standard input behaves as a pipe", §9.3).

### A6. `main`'s return types and exit codes

`__rt::enter<R: Termination>` already delegates to Rust. Rust's impls: `()`→0;
`Result<T: Termination, E: Debug>` prints `Error: {err:?}` to stderr and returns
`ExitCode::FAILURE`, which is 1 on Unix (https://doc.rust-lang.org/std/process/trait.Termination.html;
RFC 1937: https://github.com/rust-lang/rfcs/blob/master/text/1937-ques-in-main.md); a panic that
escapes `main` makes the runtime return 101
(https://rust.googlesource.com/rust/+/refs/heads/main/library/std/src/rt.rs).

- The bound admits more than the reference's three (`Result<ExitCode, E>`, `!`, user impls of
  the re-exported `Termination`). The reference says "as in Rust". **Recommend keeping the
  bound**: restricting needs a sealed trait and a new diagnostic for no stated benefit
  (AP-CONTRACT-001, fewest unsupported restrictions). Note it in 03 §1.3 as "Rust's `Termination`
  set; the three listed are the taught ones".
- Rust's `Error:` line bypasses `__sink::err`. With A1's synchronous sinks the order on fd 2 is
  still program order. This is the concrete coupling between A1 and A6: a queued sink would break it.
- `Proc::exit(code)` is in the capability table but not in fx-5vj's Scope; leave it to the
  process bead. When built, it must not rely on destructors (see A1) and `i32` codes are
  truncated to 8 bits by Unix `wait` (https://man7.org/linux/man-pages/man3/exit.3.html *(recalled)*).

### A7. Simulation: `world.args/env_var/stdin`, `sim.records`

Prior art. FoundationDB runs the whole cluster in one deterministic process and checks
determinism by comparing a final random draw ("unseed") and, on mismatch, trace logs
(https://pierrez.github.io/fdb-book/meet_fdb/correctess.html). madsim intercepts time and
randomness at the libc level and swaps tokio's APIs
(https://risingwave.com/blog/deterministic-simulation-a-new-era-of-distributed-system-testing/;
https://docs.rs/madsim/); I found no evidence that either, or turmoil, fakes process arguments,
environment or stdin: they simulate services, not CLI programs. WASI is the closer model: the
embedder builds a context with arguments, environment and stdio streams and the guest sees only
that (https://docs.wasmtime.dev/api/src/wasmtime_wasi/cli.rs.html). Tide's `sim.world` is that
context.

Recommendations (design text fixes signatures in §9.2; the rest is silent):
- **Storage.** Arguments, environment (`BTreeMap<OsString, OsString>`, ordered so `vars()` is
  deterministic) and stdin bytes live in the simulation state fx-nkv.5 put behind
  `Option<Arc<..>>`; `Env` and `Stdin` carry the same private simulation reference `Tasks` does.
  Keep them inspectable so fx-nkv.7's manifest (REPLAY-1) can dump them inline.
- **Signatures.** Use 03's: `args(impl IntoIterator<Item: AsRef<OsStr>>)`,
  `env_var(impl AsRef<OsStr>, impl AsRef<OsStr>)`, `stdin(impl AsRef<[u8]>)`. The stub's
  `args<S: Into<String>>` is a GUESS that contradicts it; fix the stub.
- **Repeat calls.** `args` appends (as `Command::args`:
  https://doc.rust-lang.org/std/process/struct.Command.html#method.args *(recalled)*); `env_var`
  overwrites its key (as `Command::env`); `stdin` appends bytes. Unset: no arguments (Rust
  documents that `args` may even lack the program name), empty environment, empty stdin.
- **Stdin model.** A pipe whose writer has closed: reads drain the bytes, then return `Ok(0)`;
  a read never blocks. Each read is one scheduling point (capability operation, §9.3), takes no
  virtual time and draws no randomness, which keeps scheduler streams stable (fx-nkv.5 decision 5).
- **Env calls** are scheduling points too (§9.3: "every capability operation") and take no time
  (reference 6.8).
- **Sinks.** Per fx-nkv.6 decision 6, a sink call under simulation appends a `sink` event
  (stream, bytes as emitted) and writes nothing to the real descriptors. Routing uses the current
  task's context (`CURRENT`), since sinks take no capability.
- **`records()`.** Companion 17: `Record { node, time, sink, message }`, "one line without its
  newline". Derive records from events: concatenate per (node, sink stream) in event order, split
  on `\n`; a record takes the time of the event that completed its line; an unterminated tail
  is a final record. This matches what a terminal would show for `print!("a"); print!("b\n")`.
  Bytes from `stdout().write` that are not UTF-8 convert lossily (the field is a `String`).
  Derive `Debug, Clone, PartialEq, Eq` on `Record`, `Sink`, `Level` so tests can compare them
  (the stub derives nothing).
- **`node` outside any node.** Silent. Recommend `"main"`, provisional. `time` is fx-o8j's
  virtual now, or zero until it lands.

---

## Part B — fx-ugm

### B1. How to take several cells

LOCK-3: "Cells are acquired atomically in a global order by cell id … Acquisition waits per cell
in FIFO order." Three mechanisms exist:

1. **Ordered blocking acquisition** (resource hierarchy). Dijkstra's dining-philosophers fix:
   number the resources and always take the lower first; the wait-for graph cannot close a cycle
   (https://howardhinnant.github.io/dining_philosophers.html, "Ordered";
   https://rosettacode.org/wiki/Dining_philosophers). The Linux kernel does this for same-class
   locks, ordering by address (`pipe_double_lock`:
   https://kunit.googlesource.com/linux/+/023d43c7b5a23a81fe8afa9f37296f8ed4be11fb) or by a stable
   key when addresses are the wrong order (run-queue locks ordered by core then CPU:
   https://lkml.org/lkml/2021/5/4/86). *Java Concurrency in Practice* orders account locks by
   `System.identityHashCode` and needs a third tie-breaking lock because hashes collide
   (https://next.sysgears.com/articles/escaping-deadlocks/); a unique counter needs no tie-break.
2. **Try-and-back-off.** C++ `std::lock`/`scoped_lock` require only "a deadlock-avoidance
   algorithm"; Hinnant's measurements favour try-lock, release, yield, retry over ordered locking
   under heavy contention, because ordered holders sit on one lock while waiting for the next
   (same Hinnant page; its conclusion as I recall it is "smart & polite" wins, the search summary
   confirms only that one algorithm dominates).
3. **All-or-nothing under one global lock** (wait until every cell is free, then take all), the
   shape fx-zc1 chose for `Tasks::limit` counts.

Assessment against Tide's text:
- (2) cannot keep "waits per cell in FIFO order": a waiter that backs off must give up its
  ticket or leave a hole in the queue, and retries are timing-dependent, which the simulator
  cannot make meaningful. It can also livelock.
- (3) is the most literal "atomically", but it needs a process-wide lock on every `with` (cells
  are otherwise independent) and a fairness scheme of its own: a `with2(A, B)` can starve while
  single-cell callers keep A and B alternately busy, unless it queues on both, which is (1) again.
- (1) is what LOCK-3 describes, and std's `Mutex` gives no help here: it offers no multi-lock
  and documents that re-locking on the same thread "might panic or deadlock"
  (https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock *(recalled)*).

**Recommend (1) with the lazy-ticket rule already recorded on the bead:** sort operands by id;
for each in ascending order, reserve its ticket, wait until served, keep its guard; run `f`;
advance `serving` on every cell, drop all guards, then `notify_all` each. "Atomically" then
means what an observer can see: `f` runs only with all cells held, no other closure sees a
partial acquisition, and under the simulator there is no scheduling point inside the set (B5).

*Why lazy tickets are sufficient (sketch for the reviewer).* A task waits only (i) on a cell's
queue behind earlier tickets, or (ii) not at all. Anyone holding a ticket on cell `j` has already
acquired all its operands below `j`, so it is either queued on `j`, holding `j` and queued on a
higher cell, or running a closure that cannot suspend (LOCK-4). Every wait therefore points to an
earlier ticket on the same cell or to a higher cell: well-founded, so no deadlock and no
starvation. Eager reservation breaks the first clause (the bead's X/Y example).

*Fairness.* FIFO is per cell, by arrival *at that cell*. A `with2(A, B)` that arrived before a
`with(B)` may run after it. That is consistent with "waits per cell in FIFO order"; parking_lot
documents the same trade between strict FIFO hand-off and throughput for single mutexes
(https://docs.rs/parking_lot/latest/parking_lot/type.FairMutex.html). Say so in the rustdoc; do
not promise global arrival order.

*Cost.* Hinnant's convoy effect applies in principle; Tide's closures are short, `Pure` and
non-suspending, and printing inside them is already a lint (`FX0607`). Accept it.

### B2. Cell identity for ordering

LOCK-3: "a process-unique counter assigned by `Shared::new` and deterministic under the
simulator". fx-nkv.5 decision 6: ids are simulation-local counters and "fx-ugm must use them
too"; its worktree already gives a simulated cell `SimCell { runtime, id: usize }` from
`scheduler.allocate_cell()`.

- Address ordering (`Arc::as_ptr`) is the kernel's idiom but varies per run and exposes
  addresses to behaviour, which §2.2 treats as authority. Rejected by the text.
- **Recommend**: real cells take an id from one `static AtomicU64` (`fetch_add`, `Relaxed`; a
  `u64` cannot wrap in practice); simulated cells use fx-nkv.5's id. One private
  `fn order_key(&self)` on the cell hides which.
- **Mixed domains.** A real cell and a simulated one, or cells of two simulations, can carry the
  same number. fx-nkv.5 decision 2 already panics on a handle used across simulations. `with2`
  and `with_all` must make that check on *all* operands before sorting; otherwise equal ids give
  an arbitrary order and a real deadlock.

### B3. Detecting aliases

- **Identity is pointer identity**, not id equality: `Arc::ptr_eq` semantics
  (https://doc.rust-lang.org/std/sync/struct.Arc.html#method.ptr_eq *(recalled)*). For `with2`
  the handles have different static types, so compare the allocations' addresses as thin
  pointers; two live `Arc` allocations never share an address (each holds its counts). Safe
  code can only alias when `T == U`.
- `Shared::same(a, b)` is that comparison. It acquires nothing and is not a scheduling point
  (§9.3's list has `with*` acquire/release and `reset`, not `same`); §2.2 already says it
  "compares identity without exposing it".
- `with_all`: after the domain check, sorting indices by id puts aliases adjacent (within one
  domain, equal id means same cell); detect them in that pass, O(n log n), before the first
  ticket. Java's hash-order idiom shows what goes wrong when the key is not unique; here it is.
- **Precedence.** LOCK-2 says identity is compared "before acquiring anything", so an aliased
  call returns `Aliased` even if the cell is poisoned, and takes no ticket.
- `with_all(&[])` runs `f` on an empty slice (reference 7.4): no point? It should still take the
  two scheduling points so a trace does not depend on the slice length being zero only by
  accident; this is silent, recommend "points as for any call".

### B4. Poisoning and `reset`

- A poisoned operand is discovered only once its cell is held. **Recommend stopping at the first
  poisoned cell**: consume that ticket, release everything held, return `Err(Poisoned)`; `f`
  does not run and no cell changes state. (Acquiring the rest first only lengthens the hold.)
- A panic in `f` sets `poisoned` on every held cell before any is released (LOCK-6), then
  releases, notifies, and `resume_unwind`s, as `with` does. Keep `with`'s single unconditional
  advance/drop/notify step (fx-nkv.4's review rebuttal depends on it); the cleanest way is to make
  `with` the one-cell case of a private `acquire_set`/`release_set` pair, so there is one owner of
  the ticket protocol (AP-OWNERSHIP-001).
- **Operand order.** `f` receives `&mut [&mut T]` in the *caller's* order, not id order. Collect
  `(original index, &mut value)` from the guards, sort by index, then build the slice; this is
  expressible in safe Rust.
- **`reset(&self, v: T)`** (returns `()`, reference 7.4): acquires like `with` (ticket, FIFO,
  scheduling points; not a checkpoint), then `mem::replace`s the payload and clears the flag
  whether or not the cell was poisoned. The design is silent on where the old payload is dropped.
  Its `Drop` is user code and may panic. **Recommend dropping it after release**, on the caller:
  a panic then propagates to the caller with the cell already clean and unlocked, instead of
  re-poisoning the cell being repaired. LOCK-6's "no safe API reads a poisoned payload" still
  holds in the sense Rust's `Mutex` accepts: the destructor runs on it.
- Waiters queued behind a `reset` see a clean cell; those ahead of it still get `Poisoned`.
  That follows from FIFO and needs no rule.

### B5. Under the simulator

fx-nkv.5 decision 3 puts one point before ticket reservation and one after release, and decision
4 makes a contended cell an internal panic, valid because nothing between acquire and release is
a point. Multi-cell operations keep that property only if **the whole set is taken between two
points**: one point before the first reservation, none between cells, one after the last
release. Emit one `cell` acquire event per operand in ascending id and one release event each
(descending), all at that step, so traces are deterministic (fx-nkv.6 decision 1). The task's
wait state at the first point is "ready at cells(ids ascending)".

### B6. Production/conformance equivalence

`sync.rs` is `include!`d by the stub, which also has its own `impl` block with `with2`, `same`
and `with_all` as `unimplemented!()` fixtures. Once `sync.rs` defines them the stub block is a
duplicate-definition error: delete it in the same change, and move `conformance/rules.md`'s
LOCK-2/3/6 rows from deferred to covered. `reset` has no stub at all today.

---

## 2. Pitfalls

1. **Two paths to fd 1.** Macros and `io::stdout()` must share one sink function, or simulation
   capture repeats libtest's gap (rust#90785).
2. **Exit without flush.** Any buffering in the sink path is lost on `exit(141)`, on a later
   `Proc::exit`, and can reorder against Rust's `Error:` line.
3. **`stdin` read-ahead.** A 8 KiB `BufReader` on fd 0 swallows input a child with inherited
   stdin expected. Rust has the same behaviour; note it, do not fix it here.
4. **Non-Unicode panics are lazy.** A test that only calls `env.args()` without iterating proves
   nothing.
5. **`capability!` shape change.** `Env` and `Stdin` gain fields; the macro is shared with the
   stub and tide-admit keys the type paths.
6. **Windows.** Environment names are case-insensitive there; `only` on v0's Linux compares
   bytes. Say "v0: exact match".
7. **Ticket holes.** Any early return between reserving a ticket and advancing `serving`
   (a panic in the domain check, an alias found late) strands every later waiter. Do all checks
   before the first reservation.
8. **Guard order versus operand order** in `with_all` (B4).
9. **Id collisions across domains** (B2) turn into real deadlocks only under rare interleavings;
   the check must be unconditional, not a debug assertion.
10. **`AtomicUsize` tickets** wrap with `wrapping_add` and stay correct; the *cell id* must not
    wrap, hence `u64`.
11. **Log level under simulation.** If the sim filtered by level, `sim.records()` would depend
    on configuration; SINK-6 says every record.
12. **Held-out rule.** Tests for both beads must be written from the design text; none of the
    recommendations above derives from E1 files.

## 3. Cheapest falsifying tests

fx-5vj (toolchain tests that build a binary through tide-admit and run it):
- **Exits**: four tiny mains → statuses 0, 1 with stderr exactly `Error: <Debug>\n`,
  `ExitCode::from(7)` → 7, panic → 101. Falsifies A6.
- **141 and order**: binary printing in a loop with stdout closed by the parent after one line →
  status 141, empty stderr; binary doing `println!`, `stdout().write_all`, `eprintln!`, then
  returning `Err` with stdout and stderr on one pipe → bytes in program order. Falsifies A1/A3.
- **Threads**: eight tasks each printing 1,000 fixed 200-byte lines → every output line is
  intact. Falsifies record atomicity.
- **Env**: child spawned with args `["a", "b c"]`, `K=V` and a non-UTF-8 variable →
  `args()` lists them after the program name; `var("K")`; `var` of the bad one is `NotUnicode`;
  iterating `vars()` panics (101); `only(["K"]).var("HOME")` is `NotPresent`;
  `only(["K"]).only(["HOME"]).vars()` is empty; `args_only()` keeps arguments.
- **Stdin**: piped `"a\nb\r\n\xff\nc"` → `lines()` yields `Ok("a")`, `Ok("b")`, `Err(InvalidData)`,
  `Ok("c")`, then `None`; `read_to_end` after a partial `lines()` returns exactly the rest
  (falsifies the one-line-per-fill claim); a read in a cancelled scope returns `Cancelled`
  without consuming input.
- **Log defaults**: `info!`, `debug!` with a side-effecting argument → stderr has only
  `info: ..`, and both arguments' side effects ran, the disabled record's included (SINK-3); after `root.log.level(Level::Debug)` both appear.
- **Sim**: `world.args`, `env_var`, `stdin` set, code under test reads them back; `print!("a");
  println!("b"); eprintln!("c"); stdout().write_all(b"d\n")` → `records()` equals
  `[Out "ab", Err "c", Out "d"]`; the real stdout of the test process is empty; same seed twice
  gives equal records.

fx-ugm:
- **No deadlock, serialized**: real threads, two cells, half the tasks call `with2(&a, &b)` and
  half `with2(&b, &a)`, each moving one unit; with a wall-clock watchdog outside the scope
  (as `sync.rs`'s tests do) the run ends and the sum is conserved. A version with three cells and
  `with_all` over rotating subsets. Falsifies B1; a variant constructed to hit the bead's X/Y
  interleaving (X holds A, Y queued on A, then X reaches B) checks lazy tickets specifically:
  with eager reservation it hangs.
- **Aliased**: `with2(&a, &a.clone(), ..)` and `with_all(&[a.clone(), b, a])` return `Aliased`,
  a flag set by `f` stays false, the ticket counter of `a` is unchanged, and a following `with`
  succeeds; the same on a poisoned `a` still returns `Aliased` (B3 precedence).
- **Operand order**: `with_all(&[c_high_id, c_low_id], |xs| ..)` sees `xs[0]` as the first
  operand's payload.
- **Poison**: a panic in `with2`'s `f` poisons both; `with_all` naming one poisoned cell returns
  `Poisoned` and leaves the others usable; `reset(v)` then `with` returns the new value; `reset`
  with a payload whose `Drop` panics leaves the cell usable (B4).
- **`same`**: true for clones, false for two cells with equal payloads.
- **Sim**: under several seeds, two tasks doing opposite-order `with2` finish; the trace shows
  acquire events in ascending id within one step; a real cell passed with a simulated one panics
  with the cross-simulation message (B2).

## 4. Silent or ambiguous design text

| Where | Gap | Recommended reading |
| --- | --- | --- |
| 03 §1.3, SINK-2 | `LogConfig`'s methods, value types, module; whether it may leave `main` | A2; no new rule, flag on `fx-djp` |
| SINK-2 | text and JSON line shape, timestamps | `level: message`; fixed-key JSON; no timestamp; provisional |
| SINK-4/5 | whether the writer thread is contract | No: observable order and loss rules only (A1) |
| SINK table vs fx-5vj Scope | `dbg!`, `trace` sink | out of this bead |
| §2.4, reference 6.5 | `stdout().lock()` | not provided |
| reference 6.1 | `args()`/`vars()` types; non-Unicode; `_os` forms | own iterators; Rust's panics; none |
| 03 §1.1 | `only`: denied read, chaining, effect on args and cwd; `args_only` and `current_dir` | `NotPresent`; intersection; variables only; `PermissionDenied` (provisional) |
| 03 §1.1 vs TASK-3 | cancelling a blocked stdin read | checked at entry in v0; record the limit |
| 03 §9.2 | repeat `world.args/env_var/stdin`; unset defaults; stdin latency | append / overwrite / append; empty; zero |
| companion 17 | `Record.node` outside a node; partial lines; non-UTF-8 output; derives | `"main"`; per-stream line assembly; lossy; `Debug, Clone, PartialEq, Eq` |
| stub | `World::args<S: Into<String>>` contradicts 03 §9.2 | 03 wins; fix the stub |
| 03 §1.3 / reference 6.2 | `main` return set vs `Termination` | Rust's set |
| LOCK-3 | "atomically" | all held before `f`; no scheduling point inside the set |
| LOCK-3 | FIFO across cells | per cell, by arrival at that cell |
| LOCK-2 vs LOCK-6 | aliased and poisoned together | `Aliased` |
| LOCK-6 | where `reset` drops the old payload; `reset` on a healthy cell | after release; allowed |
| LOCK-3, fx-nkv.5 | operands from different simulations or real and simulated | panic before ordering |
| reference 7.4 | `with_all(&[])` and scheduling points | points as for any call |

Sources: every URL cited inline above. DONE
