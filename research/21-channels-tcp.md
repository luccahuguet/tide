Source: `/home/lucca/.cache/eon-agents/tide-research-chan/report.md`, 2026-11-10; copied 2026-11-12.

# Channels and simulated TCP for Tide (fx-o4h, fx-hxw)

Research report, 2026-11-10. It prepares the implementers of fx-o4h and fx-hxw. Nothing here adds a language rule; where the design is silent, section 4 says so and recommends a reading.

Read locally: both beads, fx-nkv.3, the 2026-11-10 decisions on fx-byk.1, fx-byk.2, fx-nkv.5 and fx-nkv.6, fx-o8j, fx-zc1, the time report (`tide-research-time/report.md`), `design/03-authority-concurrency.md` (§0, §1.1, §1.2, §2.6, §6.2–6.4, §9, §12.3), `design/reference.md` (§6.1, §6.7, §6.8, §7.2, §7.3), the companion's network notes, `crates/tide-std/src/{lib,task,root}.rs` on `main` at ede7dd5, the conformance stub std, and the scheduler in `.worktrees/fx-nkv.5` (uncommitted, so its names may change). No `trials/e1-checked/` file was opened.

Web sources were fetched on 2026-11-10 by three lookup agents. Claims marked "unverified" rest on memory or a search snippet. The `std::thread::park` token rule in 1.3 was not fetched.

## 1. Design choices

### 1.1 Channel types and errors (fixed by the design)

03 §6.4 and reference §7.3 fix the surface. The implementer copies it; nothing here is a choice.

- Path `std::sync::mpsc`. `channel(tasks) -> (Sender<T>, Receiver<T>)` and `sync_channel(n, tasks) -> (SyncSender<T>, Receiver<T>)`, `T: Send`. The stub already has the argument order `(bound, tasks)`.
- `Sender::send` never blocks and fails only with `SendError::Disconnected(v)`. `SyncSender::send` can also return `Cancelled(v)`. `try_send` and `try_recv` keep Rust's `TrySendError::{Full, Disconnected}` and `TryRecvError::{Empty, Disconnected}`.
- `RecvError { Disconnected, Cancelled }`, `RecvTimeoutError { Timeout, Disconnected, Cancelled }`, `SendError<T> { Disconnected(T), Cancelled(T) }` with `into_inner()`. Each has `is_cancelled()`, `Debug`, `Display`, `Error`, `Clone`, `Copy`, `PartialEq`, `Eq` (`SendError<T>` the last four where `T` has them), and `From<RecvError> for RecvTimeoutError`. Rust's conversions into the `Try*` errors are dropped.
- `Sender` and `SyncSender` are `Clone`. A `Receiver` has one consumer: not `Clone`, and `!Sync` as in Rust.

The conformance stub is behind: `SendError` is a tuple struct, `RecvError` a unit-like struct, and `Sender::send`, `try_send`, `try_recv` and iteration are missing. fx-o4h must replace the stub's `mpsc` module, by the shared-file `include!` that `task.rs` and `sync.rs` use, so production and conformance cannot drift (the bead's equivalence clause).

### 1.2 Which calls are checkpoints, and when cancellation wins

**Options for `SyncSender::send`.** (a) Every call checks cancellation first. (b) Only a call that finds the channel full does.

**Evidence.** The texts say "`send` on a full `sync_channel`", which reads as (b). But 03 §0 defines a checkpoint by operation ("every capability operation that may suspend"), TASK-3 says every checkpoint in a cancelled scope "returns its cancellation error immediately", and §6.1 makes a limited `spawn` start nothing "even when a slot is free". Under (b) a cancelled producer delivers between 0 and `n` more values depending on how fast the consumer drains, so real and simulated runs can differ.

**Recommendation: (a).** `recv`, `recv_deadline` and `SyncSender::send` check cancellation on entry, whatever the channel holds. A cancelled `recv` returns `Cancelled` even when a message is buffered and leaves it there. Trio states the same property: "either receive will successfully return an object, or it will raise Cancelled while leaving the channel unchanged" (https://trio.readthedocs.io/en/stable/reference-core.html). `Sender::send`, `try_send`, `try_recv`, clone and drop are not checkpoints (§6.4 says they never block).

**After a wait.** Kotlin shows what to avoid: with its "prompt cancellation guarantee", a cancelled `receive` may already hold the element, which then needs an `onUndeliveredElement` hook (https://kotlinlang.org/api/kotlinx.coroutines/kotlinx-coroutines-core/kotlinx.coroutines.channels/-receive-channel/receive.html). tokio's `send` drops the message when cancelled (https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html#method.send). Tide must return the value. Order on wake:

- Receiver: cancellation, then a message, then `Disconnected`, then `Timeout`. A message stays queued when cancellation wins, so nothing is lost. Cancellation before `Timeout` is TASK-4's tie rule.
- Blocked sender: if its value was already taken or moved into the buffer, return `Ok(())` (TASK-3: an operation "that completes before the cancel takes effect returns its own result"). Otherwise remove its entry and return `Cancelled(v)` or `Disconnected(v)`.

### 1.3 Blocking, FIFO wake-ups and the data model

**Evidence.**

- Shuttle keeps its own queue and FIFO waiter lists (`waiting_senders`, `waiting_receivers`), wakes only the head ("unblock the first waiting receiver if any"), makes a new arrival queue behind existing waiters, and wakes all senders only when the receiver drops (https://raw.githubusercontent.com/awslabs/shuttle/main/shuttle-std/src/sync/mpsc.rs).
- loom models a counter and wakes every blocked thread; it has no `sync_channel` and no `recv_timeout` (https://raw.githubusercontent.com/tokio-rs/loom/master/src/sync/mpsc.rs). madsim simulates no user channel (https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/task/mod.rs).
- Kotlin: "Send and receive operations to channels are fair with respect to the order of their invocation ... served in first-in first-out order" (https://kotlinlang.org/docs/channels.html). tokio: "calls to `send` and `reserve` complete in the order they were requested" (URL above). Go: "Channels act as first-in-first-out queues" (https://go.dev/ref/spec#Channel_types).
- `Shared::with` and the recommended `limit` already wait FIFO with direct hand-off (time report 1.7).

**Recommendation.** One `Arc<Chan<T>>` with one mutex over:

```
buffer: VecDeque<T>            // at most n when bounded
pending: VecDeque<(Ticket, T)> // blocked senders with their values, arrival order
receiver: Option<Ticket>       // the one blocked receiver
senders: usize, receiver_alive: bool, id: usize
```

- `SyncSender::send`: if `buffer.len() < n` and `pending` is empty, push and wake the receiver. Otherwise push `(ticket, v)` on `pending` and wait. No barging: a sender never passes an earlier one.
- `recv`: pop `buffer`; then move the head of `pending` into `buffer` and wake that sender. With `n == 0` take straight from the head of `pending`. So a rendezvous `send` returns only after a `recv` took the value, which is Rust's guarantee: "each `send` will not return until a `recv` is paired with it" (https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html).
- A sender that leaves early (cancelled, disconnected) removes its own entry. Because the value sits in channel state, not in a hand-off slot, there is no half-delivered state and no wake to pass on.
- `try_send` on `n == 0` succeeds only when a receiver is blocked, as Rust says: "there is no `Receiver` available to acquire the data" gives `Full` (https://doc.rust-lang.org/std/sync/mpsc/enum.TrySendError.html).
- The wait itself is the runtime's one primitive from the time report (1.6): `wait(kind, span, until, cancellable: true)`. fx-o4h adds the wait kinds `recv(chan id)` and `send(chan id)` and no cancellation logic of its own.
- Real runtime: tickets hold a `Thread`; the wait is a `park_timeout` loop that rechecks the channel under its lock. An `unpark` that arrives before the `park` is not lost, because the token stays set (https://doc.rust-lang.org/std/thread/fn.park.html, unverified). Lock order is channel, then runtime; `cancel` unparks without taking a channel lock.
- Simulator: one scheduling point on entry (before looking at the channel, as fx-nkv.5 decision 3 does for `Shared::with`), then the cancellation check, then act or `block`. A wake only makes the waiter runnable; the seed picks who runs. Waking adds no point. The channel mutex is never held across a point. `try_send` and `try_recv` are points too (§9.3 lists `send` and `recv`). Creation and drop are not points.
- The channel copies the simulation identity from `Tasks` (fx-nkv.5 decision 2) and takes a simulation-local id (decision 6).

### 1.4 Disconnect and drop

**Evidence.** Rust: after the senders disconnect, "messages sent before the disconnect will still be properly received" (https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html), and `Ok` from `send` "does not mean that the data will be received" (https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html). crossbeam: "No more messages can be sent, but any remaining messages can still be received. Send and receive operations on a disconnected channel never block" (https://docs.rs/crossbeam-channel/latest/crossbeam_channel/). Go instead panics on a send to a closed channel (https://go.dev/ref/spec#Send_statements); Tide keeps Rust's error.

**Recommendation.** Rust's rules, unchanged:

- Last sender dropped: wake the receiver. `recv` drains `buffer`, then returns `Disconnected`.
- Receiver dropped: wake every blocked sender (each gets `Disconnected(v)`), and drop the buffered values on the dropping thread after releasing the channel lock, since a destructor may use another channel.
- Drop never blocks, is not a scheduling point (§9.3 does not list it; like task finish, it only makes waiters runnable), and records a `close` trace event. It must be safe during unwinding.
- A disconnect sets no echo bit (03 §6.4): the channel code never calls `observe` for it.

### 1.5 `recv_deadline`

Rust's `recv_deadline` is unstable (`deadline_api`, #46316; same Receiver page); crossbeam's is stable. Shuttle's `recv_timeout` "never times out" and loom's is `unimplemented!`, so neither is a model. Tide has what they lack: fx-o8j's timers.

**Recommendation.** `recv_deadline(t)` is `recv` with `until = Some(t)`: a class-1 timer under the simulator (time report 1.3), a `park_timeout` bound on the real runtime through the process anchor (time report 1.1). A message that is already there is returned even when `t` has passed; `Timeout` is returned only when the wait would block at or after `t`. A past deadline with an empty channel takes the entry point and returns `Timeout` without blocking.

### 1.6 Iteration

Rust: `iter()` "will return `None` when the channel has hung up" (Receiver page). 03 §6.4: iteration "ends at the first error of either kind".

**Recommendation.** `iter()`, `IntoIterator for Receiver<T>` and `for &Receiver<T>`, each `next()` being one `recv().ok()`; `try_iter()` over `try_recv`. Each `next()` is a checkpoint and a point. No fused state is needed: after `Cancelled`, the next `recv` is cancelled again.

### 1.7 Production implementation (AP-DEPENDENCY-001)

Capability needed: a bounded and unbounded MPSC queue whose blocked ends can be woken by a scope's cancellation at once, give the payload back, block through the baton scheduler, and name their wait in the deadlock report.

| Candidate | Verdict |
| --- | --- |
| Rust's `std::sync::mpsc` (crossbeam-based since 1.67, https://blog.rust-lang.org/2023/01/26/Rust-1.67.0/) | Rejected. A blocked `send` or `recv` cannot be woken by a third party; a blocked `send` cannot return its value; a task blocked in it holds the baton forever (time report pitfall 5). Polling with `recv_timeout` breaks "immediately" and determinism. |
| `crossbeam-channel` (MIT OR Apache-2.0, one dependency, `crossbeam-utils`; https://crates.io/api/v1/crates/crossbeam-channel/0.5.17/dependencies) | Rejected. Same blocking problem under the simulator, so an owned implementation is needed anyway; two implementations could not be equivalent by construction. |
| Owned queue (1.3) | **Chosen.** About 300 lines over `Mutex` and `VecDeque`, no `unsafe`, one implementation for both runtimes that differs only in the wait primitive. Shuttle made the same choice. |

No dependency is added. Replacement cost: one module.

### 1.8 Deadlock report and channel trace events

**Evidence.** Shuttle: no runnable task while an attached task is unfinished is `StepError::Deadlock`, reported as "deadlock! blocked tasks: [..]" (https://raw.githubusercontent.com/awslabs/shuttle/main/shuttle-engine/src/runtime/execution.rs). loom: "There is no active thread. Unless all threads have terminated, the test has deadlocked" (https://raw.githubusercontent.com/tokio-rs/loom/master/src/rt/execution.rs). Go's `checkdead` does not fire while a timer is pending (https://raw.githubusercontent.com/golang/go/master/src/runtime/proc.go). Go's synctest counts only channels made inside the bubble as durably blocking (https://pkg.go.dev/testing/synctest).

**Recommendation.** No detector of its own. A channel wait is a `block` with kind `recv(chan id)` or `send(chan id)` and the call's span. When `pick` finds no runnable task and no timer, the shared report (time report 1.9) lists them and the process exits nonzero. Whichever of fx-zc1 and fx-o4h lands first builds the report; the other adds a wait kind. A pending `recv_deadline` timer is not a deadlock: time jumps to it. The real runtime detects nothing; the design promises the report only under the simulator.

Trace events, kind `chan` (03 §9.6 names it), names provisional until fx-nkv.7:

| Operation | Fields beyond step, time, task, span |
| --- | --- |
| `new` | `chan`, `bound` (null when unbounded) |
| `send`, `try_send` | `chan`, wait `send(chan)` when blocked, result `ok`, `full`, `disconnected`, `cancelled` |
| `recv`, `try_recv`, `recv_deadline` | `chan`, `until`, wait `recv(chan)`, result `ok`, `empty`, `disconnected`, `cancelled`, `timeout` |
| `close` | `chan`, `end` (`sender` with the count left, or `receiver`) |

Never record the payload: `T` has no `Debug` bound.

### 1.9 `Net`: shape, addresses and attenuation

**Shape.** Follow fx-byk.1 decision 3: one `Net`, `TcpListener` and `TcpStream` type each, with a private `Real`/`Sim` enum, and shared code in front of both backends in this order: cancellation check, policy check, argument check, backend. `Net` carries `only: Option<Arc<[(host, port)]>>` and `may_bind: bool`. `only` and `no_bind` return a narrower clone; `only` of `only` is the intersection.

**Addresses.** Rust's `connect` takes `impl ToSocketAddrs`, and `to_socket_addrs()` on a string does a DNS lookup with no capability. Options: (A) `impl AsRef<str>`; (B) re-export Rust's trait; (C) a sealed Tide trait with no callable method, implemented for `SocketAddr`, `&SocketAddr`, `(IpAddr, u16)`, `(&str, u16)`, `&str` and `String`, turned inside `Net` into `Addr(SocketAddr)` or `Host(String, u16)`. **Recommend C**: Rust's call sites compile unchanged and resolution stays inside `Net`. B leaks an ambient lookup. `SocketAddr`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr` and `Shutdown` are re-exported from Rust: they are plain data.

In this bead resolve only IP literals and `localhost` (to `127.0.0.1`). On the real side any other host name goes through Rust's resolver in line; it is not abandoned on cancellation as TASK-3 describes, which is recorded as a limit until `Net::resolve` has an owner. In simulation another name fails (nodes and `Dns` belong to companion 17).

**Attenuation.**

- `no_bind()`: `bind` returns `io::ErrorKind::PermissionDenied`, the kind `Fs` uses for a refused authority.
- `only(hosts)`: `connect` to an address not listed returns `PermissionDenied` (`NetError::Denied` is `Net::get`'s error type). Match the host as written, case-insensitively, with IP literals compared as parsed addresses, and the port exactly; never match through a lookup. `only` does not restrict `bind`: reference §6.1's own example chains `root.net.only([..]).no_bind()`, which would be redundant otherwise.
- Accepted connections are not filtered by `only`.

### 1.10 `TcpListener` and `TcpStream` surface

Rust's API, as the bead's Scope lists it.

- `Net::bind(addr) -> io::Result<TcpListener>`; `accept() -> io::Result<(TcpStream, SocketAddr)>`; `incoming()`; `local_addr()`.
- `Net::connect(addr) -> io::Result<TcpStream>`; `io::Read` and `io::Write` for `TcpStream` and `&TcpStream` (Rust has both, https://doc.rust-lang.org/std/net/struct.TcpStream.html); `peer_addr()`, `local_addr()`, `shutdown(Shutdown)`; `flush` does nothing.
- Checkpoints (03 §1.1 table): `bind`, `connect`, `accept`, `read`, `write`, `shutdown`. `local_addr` and `peer_addr` read state fixed at creation: not checkpoints, no latency, no scheduling point.
- Drop closes: "The connection will be closed when the value is dropped" (same page). Drop is not a checkpoint.
- Out of this bead: `read_owned`, `write_owned` (need `io::BufError`), `try_clone`, socket options, and read or write timeouts, which would read time without a `Clock`.

**Real backend.** Wrap Rust's `TcpListener` and `TcpStream`, set non-blocking. Each operation loops: check cancellation, make the call, and on `WouldBlock` wait in `poll(2)` on the socket and a wake descriptor, bounded by the earliest enclosing scope deadline (time report 1.5, lazy deadlines). 03 TASK-3 sketches exactly this for writes, and it gives "a cancellation error only if the kernel accepted no byte" for free, since a non-blocking `write` returns `Ok(n)` or `WouldBlock`.

- Wake descriptor: an `eventfd` cached per OS thread inside std, registered as the thread's waker while it waits; `cancel` writes to it. Parking cannot wake a thread inside `poll`.
- Dependency: `poll` and `eventfd` are not in Rust's std. Use `rustix` with feature `event`, the crate fx-byk.1 decision 2 already pins. fx-hxw does not depend on fx-byk.1, so whichever lands first adds the pin. Rejected: blocking sockets (a blocked `accept` in a cancelled scope never returns, so scope exit hangs) and `poll` with short timeouts and no wake descriptor (an idle server wakes many times a second; acceptable only as a fallback).
- `connect`: check cancellation, then Rust's `connect_timeout` bounded by the earliest enclosing scope deadline, or plain `connect` without one. A connect in flight is not cut short by a sibling's failure. Loopback connects finish or refuse at once; record the limit against TASK-3.
- A real backend call made by a simulation task must panic (pitfall 2.6).

### 1.11 Simulated loopback: data model

**Evidence.**

- turmoil: "a simplification of real TCP. We implement just enough to ensure fidelity", with `Segment { Syn, Data(u64, Bytes), Fin(u64), Rst }`, a per-bind accept `VecDeque`, `ephemeral_ports: 49152..=65535` assigned sequentially, `AddrInUse` for a bound port, `ConnectionRefused` for a SYN to no bind, `Ok(0)` after `Fin`, `BrokenPipe` on write after shutdown, and `Rst` on close with unread data (https://raw.githubusercontent.com/tokio-rs/turmoil/HEAD/crates/turmoil/src/envelope.rs, `config.rs`, `host.rs`, `net/tcp/stream.rs` beside it).
- madsim: a stream is two "reliable, ordered" one-way channels; latency is drawn per payload, `Duration::from_millis(1)..Duration::from_millis(10)` by default; port 0 takes the lowest free port; a dropped stream gives the peer a 0-byte read; shutdown and partial writes are TODOs (https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/net/network.rs, `tcp/stream.rs`).
- FoundationDB `Sim2Conn`: a pair with a `peer` pointer, a `recvBuf` and byte counters; the receiver is handed a random prefix of what was sent, and a short write under `buggify` (https://raw.githubusercontent.com/apple/foundationdb/main/fdbrpc/sim2.cpp).
- TigerBeetle's packet simulator is packet level (`one_way_delay_min`, `path_maximum_capacity`), not a byte stream (https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/testing/packet_simulator.zig). smoltcp's `Loopback` is a FIFO packet device under a real TCP stack (https://docs.rs/smoltcp/latest/smoltcp/phy/struct.Loopback.html): far more than this bead needs. Go's `net.Pipe` has "no internal buffering" (https://pkg.go.dev/net#Pipe), which would hide every bug that needs a send buffer.

**Recommendation.** A `SimNet` in the simulation state, under one mutex never held across a scheduling point, in `BTreeMap`s with simulation-local ids:

```
listeners: BTreeMap<u16, Listener { addr, backlog: VecDeque<ConnId>, waiter }>
conns:     BTreeMap<ConnId, Conn { ends: [End; 2] }>
End  { addr, open, read_shut, write_shut, reset_at: Option<Instant>, inbox: Pipe }
Pipe { segments: VecDeque<(arrive: Instant, Bytes | Fin)>, bytes: usize, waiters }
```

- One host: `127.0.0.1`. A bind to `0.0.0.0:p` listens there too and `local_addr` reports what was bound, as on Linux. No real socket exists on the `Sim` path.
- Port 0: draw a port from 49152..=65535 from the net sub-stream, redrawing while taken. A drawn port is deterministic per seed and stops tests hard-coding the first ephemeral port, which a sequential allocator invites. A bound port gives `AddrInUse` (https://man7.org/linux/man-pages/man2/bind.2.html). A dropped listener frees its port at once: Rust sets `SO_REUSEADDR` on non-Windows targets "to quickly rebind a socket" (https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/sys/net/connection/socket/mod.rs).
- Backlog: at most 128 established connections; a connector that finds it full waits. Linux queues "completely established sockets waiting to be accepted" (https://man7.org/linux/man-pages/man2/listen.2.html), so `connect` returns before `accept` is called.
- Each direction holds at most 65,536 bytes, in flight or unread (provisional). Segments keep send order: `arrive = max(now + latency, previous arrive)`.
- Delivery is lazy. Nothing moves bytes at `arrive`; a reader treats segments with `arrive <= now` as readable and otherwise waits with `until` set to the first segment's `arrive`. That needs no timer callback in the scheduler: the reader's own timer is the "delivery" of §9.3.

### 1.12 Latency, scheduling points and results per operation

Draws come from a net sub-stream with its own domain constant (fx-nkv.5 decision 5), uniform in 1,000,000..=10,000,000 ns per packet (03 §9.3: "each network packet takes 1–10 ms"). Every operation first checks cancellation: `Cancelled`, no draw, no time, no effect (as fx-byk.2 decision 2). Unlike file latencies, network waits are cancellable (TASK-3: socket waits; §9.3).

| Operation | Model |
| --- | --- |
| `bind` | One point, no packet, no time. `AddrInUse`, or `PermissionDenied` under `no_bind`. |
| `connect` | Two draws (one round trip), one cancellable timer for their sum. On wake: cancelled gives `Cancelled` with no effect; no listener gives `ConnectionRefused` ("found no one listening", https://man7.org/linux/man-pages/man2/connect.2.html; the companion's "`Refused` after one round trip"); else create the `Conn`, push it on the backlog, wake the acceptor, return. The effect is atomic at the wake, so there is no half-open state to undo. |
| `accept` | One point. Pop the backlog, or wait cancellably with kind `accept(port)`. No latency of its own. |
| `write(buf)` | One point. `BrokenPipe` after local `shutdown(Write)` (send(2): "EPIPE The local end has been shut down", https://man7.org/linux/man-pages/man2/send.2.html). If the peer's inbox is full, wait cancellably with kind `write(conn)`. Else accept `n = min(len, space)`, one draw, queue the segment, wake the reader, return `Ok(n)` with no virtual time: the packet's delay is the reader's. |
| `read(buf)` | One point. Empty `buf`: `Ok(0)`. Copy arrived bytes, up to `buf.len()`, across segment boundaries; wake a blocked writer. An arrived `Fin` with nothing before it: `Ok(0)`. Else wait cancellably, kind `read(conn)`, `until` the next `arrive`. |
| `shutdown(Write)` | One point, one draw, queue `Fin`. A second call is `Ok(())`, as on Linux (TcpStream page). |
| `shutdown(Read)` | Local reads return `Ok(0)` ("All currently blocked and future reads will return Ok(0)", https://doc.rust-lang.org/std/net/enum.Shutdown.html); the peer's later writes are accepted and discarded (provisional; Linux's behaviour is unverified). |
| drop of a stream | No point, not a checkpoint. One draw; queue `Fin`. Mark the end closed. |
| write to a closed peer | Accept and discard the bytes, and set `reset_at = now +` one round trip; from then on writes give `BrokenPipe` and reads `ConnectionReset`. This mirrors a real first write that succeeds before the reset arrives (which error a real later write gives is unverified). |
| drop of a listener | Frees the port; connections still in the backlog are reset. |

EOF and half-close follow: after `shutdown(Write)` or a drop, the peer reads what was sent and then `Ok(0)` ("returning zero indicates the connection was shut down correctly", https://doc.rust-lang.org/std/io/trait.Read.html), while the other direction keeps working.

**Partial I/O.** A read returns what has arrived, which can be less than one `write` (capacity) or more (two segments), so message boundaries are never preserved. Do not add seed-drawn short reads in this bead: FoundationDB does it, but the bead keeps faults out and the design lists no such fault-free behaviour. Record it as a candidate for the fault plan.

**Trace events**, kind `cap` with a `net.` operation prefix (as the time report uses `cap` for `clock.`), names provisional:

| Operation | Fields |
| --- | --- |
| `net.bind`, `net.close_listener` | `addr`, result |
| `net.connect` | `addr`, `conn`, `latency`, wait `connect(addr)`, result |
| `net.accept` | `port`, `conn`, `peer`, wait `accept(port)`, result |
| `net.read`, `net.write` | `conn`, `n`, `latency` (write), wait `read(conn)` or `write(conn)`, result |
| `net.shutdown`, `net.close` | `conn`, `how`, `latency` |

Record byte counts, not bytes.

### 1.13 Keeping real and simulated TCP equivalent

**Evidence.** turmoil sells a "drop-in replacement for `tokio::net`" (https://raw.githubusercontent.com/tokio-rs/turmoil/HEAD/README.md); madsim: "All I/O-related interfaces must be mocked during the simulation" (https://raw.githubusercontent.com/madsim-rs/madsim/main/README.md); FoundationDB: "Flow supports both production execution and deterministic simulated execution" (https://apple.github.io/foundationdb/testing.html). Go's synctest warns that real network waits are invisible to a simulator: "goroutines blocked on network I/O prevent a synctest bubble from becoming idle" (https://pkg.go.dev/testing/synctest).

**Recommendation.** fx-byk.2 decision 4's method: one suite of `fn case(net: &Net, tasks: &Tasks) -> Vec<Observation>` run on a real `Net` at `127.0.0.1:0` and on `sim.net`, asserting equal observations (values or `ErrorKind`s). Observations exclude port numbers and timing. The shared front code (1.9) owns cancellation, attenuation and argument errors, so those cannot differ. Claim equivalence for Linux only, and only for what the suite covers; where the kernel's answer depends on timing (the kind of a write after the peer closed), the suite asserts the set both sides allow.

## 2. Pitfalls

1. **Lost wake-ups.** Queue the ticket and re-check the condition under the channel lock before every park; under the simulator, register and `block` with no point between. A sender that leaves `pending` early must not strand the next: after any removal, re-run "move the head into the buffer if there is room".
2. **Deadlock by a live sender.** A `Sender` kept alive outside the scope whose children iterate the `Receiver` never disconnects, so scope exit waits forever. The simulator reports it (`recv(chan)` plus `scope_exit`); the real runtime hangs. Worth a line in the companion's porting table.
3. **Polling hides cancellation and time.** A `try_recv` loop is not a checkpoint and keeps its task runnable, so virtual time never advances (time report pitfall 4). The step guard recommended there covers it.
4. **Unbounded buffers.** `channel(tasks)` is unbounded by design (lint `FX0608`). The simulated backlog and socket inbox must be bounded (1.11), or a writer with no reader never blocks and a missing-reader bug shows as memory growth, not a deadlock report.
5. **Partial I/O.** Code that assumes one `read` per `write` passes on real loopback and in a simulator that delivers whole segments. The capacity bound makes `write` short for large buffers; tests in section 3 must cover `write_all` and `read_exact` paths.
6. **Real network leaking into simulation.** A `Sim` `Net` has no path to a real socket; an address outside the simulated host is refused, never dialled. A real `Net` used by a simulation task blocks the baton holder in a system call the scheduler cannot see: panic at the backend boundary. A `SimNet` handle used outside its simulation panics (fx-nkv.5 decision 2).
7. **Port reuse.** Ephemeral allocation skips bound ports and frees a port only when its listener drops. On the real side the refused-connect test has a small race with other processes taking the freed port; keep it loopback-only.
8. **Draws in destructors.** Dropping a stream draws a latency. Drop order is deterministic under the baton, but a drop after the simulation ended, or during an abort, must draw nothing and emit nothing.
9. **Locks across points.** Neither the channel mutex nor the `SimNet` mutex may be held while the task takes a point or blocks; payload destructors run outside both.
10. **Mixed instants.** `recv_deadline` with an `Instant` from another clock compares plain offsets (time report pitfall 12).
11. **Sandbox.** 03 §2.6's RUN-3 seccomp list refuses sockets, so a real-loopback test cannot run inside that sandbox once it exists (section 4, item 8).

## 3. Cheapest falsifying tests

Unit tests in `tide-std` or driver-built toolchain tests; simulated ones use `Root::sim(seed)`.

| Recommendation | Test |
| --- | --- |
| 1.2 entry check | In a cancelled scope: `recv` on a channel holding a message returns `Cancelled`, and a later `recv` under `shield` returns the message. `SyncSender::send` with free capacity returns `Cancelled(v)` with `v` intact. `Sender::send` and `try_recv` still succeed. |
| 1.2 wake order | A sender blocked on a full channel in a scope a sibling's failure cancels returns `Cancelled(v)`; the receiver then sees exactly the earlier values. Both runtimes. |
| 1.3 capacity (Acceptance) | `sync_channel(2)`: two sends return, the third is `blocked at send(chan)` in the trace until one `recv`. |
| 1.3 FIFO | Three senders blocked in a known order on `sync_channel(1)`: values arrive in that order on every seed. |
| 1.3 rendezvous | `sync_channel(0)`: `try_send` with no receiver is `Full`; a `send` returns only after the `recv` event in the trace. |
| 1.4 disconnect (Acceptance) | Send three, drop every sender: `for x in rx` yields three and ends. Drop the receiver: a blocked sender gets `Disconnected(v)`; a child that returns `Ok` on it does not fail the scope. |
| 1.5 | Under the simulator `recv_deadline(now + 5 s)` on a silent channel returns `Timeout` with `sim.elapsed() == 5 s`; with a scope deadline at the same instant, `Cancelled`; with a message already queued and a past deadline, the message. |
| 1.7 one implementation | The same channel program run on the real runtime and on 20 seeds gives the same multiset of received values. |
| 1.8 deadlock (Acceptance) | Two tasks each `recv` from a channel only the other sends on: the run exits nonzero and lists both at `recv(chan id)` with spans. The same with a 1 s `recv_deadline` on one side is not a deadlock. |
| 1.8 determinism | One seed run twice gives byte-identical traces; two producers show both arrival orders across seeds. |
| 1.9 attenuation (Acceptance) | `net.no_bind().bind("127.0.0.1:0")` is `PermissionDenied` on real and simulated `Net`. `only(["127.0.0.1:1"])` refuses a connect to another port and does not refuse `bind`. |
| 1.10 cancellation | `accept` blocked in a scope that a sibling's failure cancels returns `ErrorKind::Cancelled`, on both runtimes; so does every checkpoint called in an already cancelled scope. |
| 1.11 ports (Acceptance) | `bind("127.0.0.1:0")` then `local_addr()` has a non-zero port, and a connect to it succeeds; a second bind to it is `AddrInUse`; after the listener drops, a bind succeeds and a connect is `ConnectionRefused`. Real and simulated. |
| 1.12 exchange (Acceptance) | Two simulated tasks echo a message over loopback: the bytes match, `sim.elapsed()` is within 4 packets' bounds, and the trace is identical on a rerun of the seed. |
| 1.12 EOF, half-close | Client writes, `shutdown(Write)`, then reads the reply: the server sees `Ok(0)` after the data and its reply still arrives. A dropped stream gives the peer `Ok(0)`. |
| 1.12 partial I/O | A 1 MiB `write_all` against a slow reader completes and `read_to_end` returns it intact; a single `write` of 1 MiB returns less than its length. |
| 1.12 no reader | A writer filling a connection nobody reads ends in a deadlock report naming `write(conn)`, not in memory growth. |
| 1.13 | The equivalence suite itself, plus a simulated connect to `93.184.216.34:80` that returns `ConnectionRefused` without a real socket (run it with networking unavailable). |

## 4. Where the design is silent or ambiguous

1. **`SyncSender::send` with free capacity in a cancelled scope** (1.2). The texts say "on a full `sync_channel`"; §0 and TASK-3 define checkpoints by operation. Recommend: every call checks. 03 §6.4 should say so.
2. **Whether a ready message beats cancellation in `recv`** is unstated beyond TASK-3's "immediately". Recommend cancellation first, message left in place.
3. **`try_send`, `try_recv`, clone and drop under the simulator.** §9.3 lists `send` and `recv` only. Recommend: the `try_` forms are scheduling points, not checkpoints; clone and drop are neither.
4. **`recv_deadline` with a past deadline and a queued message.** Recommend returning the message.
5. **Address argument type and name resolution** (1.9). The design writes `connect(addr)` and uses strings. Recommend the sealed trait; resolution of other host names waits for `Net::resolve`'s owner. 03 §12.3 binds `":7000"`, which Rust's parser rejects; it belongs to the node tier and this bead should not accept it.
6. **Error kind for attenuation on `io::Result` operations.** `Denied` exists only on `NetError`. Recommend `PermissionDenied`.
7. **Whether `only` limits `bind` or accepted peers.** Recommend no (the chained example), and literal matching without lookups.
8. **"Real loopback works in the harness sandbox"** (fx-hxw Acceptance) against 03 §2.6, whose seccomp list refuses sockets. Today's tests run in rustc's ordinary harness with no isolation, so the Acceptance can pass now. Whether RUN-3 later admits loopback sockets for tests is the owner of RUN-3's question; report it, as fx-byk.1 did for `openat2`.
9. **`incoming()` in a cancelled scope.** Rust's iterator "will never return None" (https://doc.rust-lang.org/std/net/struct.TcpListener.html), so `incoming().flatten()` would spin on `Cancelled`. Recommend the precedent 03 §1.1 sets for `stdin.lines()`: yield the cancellation error once, then `None`.
10. **`shutdown` as a checkpoint.** The §1.1 table lists it under "all", so cleanup that half-closes in a cancelled scope needs `shield`. Recommend following the table and saying so beside `File::close`.
11. **`local_addr` and `peer_addr`** are not in the table. Recommend: no checkpoint, no latency, no point.
12. **Latency of `bind`, `accept` and a refused `connect`.** Only packets and `Net::get` have stated latencies. Recommend 1.12's table; the numbers stay provisional with the file bound.
13. **Simulated socket capacity, backlog size and the ephemeral range** are unstated. Recommend 65,536 bytes, 128 and 49152..=65535, all provisional.
14. **Writes after the peer closed, and reads after `shutdown(Read)`,** differ by platform and timing. Recommend the model in 1.12 and keep them out of the equivalence claim beyond the allowed set.

DONE
