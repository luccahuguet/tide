# Tide reference companion

Status: the on-demand part of `design/reference.md`, the agent-facing
reference for the v0 core. Load it when you want a Rust habit's fix, a
complete program, the reading the reference takes where the design documents
disagree, the simulator's fault injection and crashable nodes, what a sealed
interface allows, when another agent's branch is safe to test without a VM,
how process handles work, the HTTP API, or what v0 leaves out. Its sections
continue the core's numbering, and a bare section number below 14 refers to
the core.
Sections 14–16 tabulate or illustrate rules the core states; 17–21 and
Appendix A hold what the core only points to. It is outside the core's
25,000-token budget (D14). **(planned)** and **(partial)** mean what they
mean in the core: a v0 surface not built yet, or built in part. The core's
blanket labels apply here too: fixes and `tide` commands are planned.

Core count: 24,999 tokens, recounted on 2026-11-19 with
`scripts/count-tokens.sh design/reference.md` (CI counting is planned).
Rerun it after any change to the core and update this line.

## 14. Rust habits and their fixes

### 14.1 Habits

| Rust habit | Tide | Code |
| --- | --- | --- |
| `trait Foo<'a>`, `for<'a>`, `type Item<'a>` | a lifetime on the method or impl, `Fn` sugar, owned items or indices | `TD0201`, `TD0404`, `TD0403` |
| `impl Lexer { .. }` for `struct Lexer<'a>` | `impl Lexer<'_>` or `impl<'a> Lexer<'a>` | error with fix |
| `fn new(s: &str) -> Self` in `impl<'a> P<'a>` | `fn new(s: &'a str) -> Self` | error with fix |
| `Box<dyn Fn()>` in a signature or field, capturing a borrow | `Box<dyn Fn() + 'a>`, or move owned data in | `TD0203` |
| Self-referential struct | indices, `Arena`, or owner and view apart | `TD0210` |
| `macro_rules!`, proc macros | functions, generics, generators | `TD0101` |
| `#[derive(Serialize, Deserialize)]`, `thiserror::Error` | `Encode`, `Decode`, `Error` | `TD0102` safe |
| `debug_assert_eq!` | `debug_assert!(a == b)` | `TD0103` |
| `include_str!`, `env!`; `x.sin()`, `x.to_bits()`, `a.min(b)`; `core::sync::atomic`; `path.exists()` | a generated `const`; `math::sin(x)`, `float::bits(x)`, `float::min(a, b)`; `AtomicU64::new(0, tasks)` (planned); `fs.exists(path)?` | `TD0520` |
| `x as u8` (lossy), `n as f64` (`usize`) | `u8::try_from(x)?`, `x.truncate()`, `f64::round_from(n)` | `TD0301` |
| Relying on release-mode wrapping | `wrapping_add`, `Wrapping<T>` | run-time trap |
| `impl Display for T` in another module | move it to `T`'s module | `TD0401` |
| `std::fs::read`, `OpenOptions::open`, `env::args`, `Instant::now`, `thread_rng` | capability methods (14.2) | `TD0501` |
| a local or `static` `OnceLock` or `LazyLock` | build the value first; between tasks, `Shared<Option<T>>`; `OnceCell` within one task | `TD0520` |
| `static COUNTER: AtomicUsize`, `lazy_static!`, `thread_local!` | a `const fn`-computed `static`, or a value built in `main` | `TD0502` |
| `const fn bump(x: &mut u32)`, or a `const fn` taking a `&Cell` | a plain `fn`, or `Pure + Sync` parameters | `TD0518` |
| `fn main()` or `fn main(root: Root)` without the attribute | add `#[tide::main]` | `TD0512` safe |
| `fn helper(root: &Root)` | pass attenuated capabilities | `TD0503` |
| `Root::sim` outside tests | real capabilities from `main` | `TD0504` |
| `dbg!` left in code; `println!` in a library | remove, or `debug!`; return data or log | `TD0507`, `TD0508` |
| `async fn`, `.await`, `#[tokio::main]` | plain `fn` taking capabilities | `TD0601` safe |
| `thread::spawn`, `tokio::spawn` | `tasks.scope(\|s\| s.spawn(..))` | `TD0602` |
| `tasks.scope(\|s\| { .. })?;` when no `Err` in the scope names its error type, `TaskError` included | name the scope's error: `tasks.scope::<_, TaskError, _>(\|s\| { .. })?`, or `MyError` | rustc E0283, with tide-admit's help naming the turbofish |
| I/O or a nested `with` inside a `Shared::with` closure; `write!(f, ..)` inside one in `Display::fmt` | do it before or after: `let n = self.cell.with(\|c\| c.hits).map_err(\|_\| fmt::Error)?; write!(f, "{n}")` | `TD0603` |
| `Arc<Mutex<T>>` + `.lock().unwrap()` | `Shared<T>` + `.with(\|x\| ..)?` | `TD0604` |
| `Shared::new(v)`, `Arc::new(v)`, `channel()` without a `Tasks` | `Shared::new(v, tasks)`, `Arc::new(v, tasks)`, `channel(tasks)`; `tide fix --thread Tasks` | `TD0505` |
| `tokio::select!` | a channel of an enum, deadlines | `TD0605` |
| `Err(SendError(v))` | `Err(e)`, then `e.into_inner()`; `e.is_cancelled()` (7.3) | rustc E0532 |
| `Err(RecvError)` | `Err(RecvError::Disconnected)`, or `Err(e)` and `e.is_cancelled()`: the bare name is now a binding that also catches `Cancelled` | rustc warning only (`non_snake_case`) |
| `use std::io::Read;` for `stdin.read_to_string(..)` | drop the import: `Stdin`'s reads are inherent, and `Stdin` and `&Stdin` are still `io::Read` for generic readers (6.1) | `TD0913` safe |
| A pipeline stage returning `Err` when its channel disconnects | return `Ok`: a disconnect alone is not an echo, so the `Err` could outrank the failure that dropped the peer (7.1, 7.3) | — |
| An `Rc<Cell<u32>>` counter captured by a `world.http` peer | `Shared::new(0u32, &sim.tasks)`; peers are `Send` (17) | rustc E0277 |
| `Mutex<Sender<T>>`, a `Shared` inside a `Shared` | payloads are `Send + Pure`; pass handles separately | `TD0606` |
| Capturing a capability, `&RefCell`, or `Arc` in `join`/`par_map` | compute first, borrow `Pure + Sync` data | `TD0610`, `TD0611` |
| `par_map` or `par_for` over `Shared` cells, channel ends, or files | spawn children in a `tasks.limit(n)` scope (7.1) | `TD0612` |
| `catch_unwind` | `tasks.spawn_isolated` | `TD0701` |
| Ignoring a `Result` | `?`, handle it, or `let _ =` | `TD0702` |
| `#[requires(a, b)]` | `#[requires(a)]` `#[requires(b)]` | safe fix |
| `unsafe` without a grant or `// SAFETY:` | add the grant; write the comment | `TD0901`, `TD0902` |
| `mod foo;` for a missing file, `#[path]` | create `foo.rs`; move the file | `TD0910`, `TD0911` |
| Unused `use`; `extern crate foo;` | delete | `TD0913`, `TD0912` safe |
| `a.rs` uses `b`, `b.rs` uses `a` | move shared items to a third module | `TD0914` |
| `build.rs` | a generator you run; commit its output | `TD0915` |

### 14.2 Ambient std calls (6.9)

| Rust | Tide |
| --- | --- |
| `std::fs::read_to_string(p)` | `fs.read_to_string(p)` |
| `File::open(p)` / `File::create(p)` | `fs.open(p)` / `fs.create(p)` |
| `OpenOptions::new().append(true).open(p)` | `fs.open_with(p, OpenOptions::new().append(true))` |
| `file.sync_all()` after `File::create` | `file.sync()`, plus `fs.sync_dir(parent)` |
| `std::env::args()` / `env::var(k)` | `env.args()` / `env.var(k)` |
| `Instant::now()` / `SystemTime::now()` | `clock.now()` / `clock.system_time()` |
| `t.elapsed()` | `clock.now().duration_since(t)`; for `SystemTime`, `clock.system_time().duration_since(t)` |
| `std::thread::sleep(d)` | `clock.sleep(d)?`; in a scope with your own error enum, `clock.sleep(d).map_err(TaskError::from)?` (7.1) |
| `rx.recv_timeout(d)` | `rx.recv_deadline(clock.now() + d)` |
| `rand::thread_rng().gen_range(0..n)` | `rand.gen_range(0..n)`, or `rand.rng()` once (planned) |
| `std::io::stdin()` | a `stdin: &Stdin` parameter (`root.stdin`) |
| `Command::new("git")` | `proc.command("git")` |
| `TcpStream::connect(a)` | `net.connect(a)` |
| `(host, 0).to_socket_addrs()`, `lookup_host(h)` | `net.resolve(h, deadline)` (21); to reach the host, `connect` or `get` by name, since a name grant does not grant its addresses |
| `std::process::exit(1)` | return `ExitCode::FAILURE` from `main`, or `proc.exit(1)` (planned) |

### 14.3 Porting async Rust (7)

| Rust | Tide |
| --- | --- |
| `async fn f(..) -> T` | `fn f(net: &Net, ..) -> T` |
| `x.await` | `x` (`TD0601`, safe fix deletes `.await` and `async`) |
| `#[tokio::main] async fn main()` | `#[tide::main] fn main(root: Root)` |
| `tokio::spawn`, `thread::spawn` | `tasks.scope(\|s\| { s.spawn(..); .. })` (`TD0602`) |
| `std::thread::scope` | `tasks.scope` (children return `Result`) |
| `tokio::join!(a, b)` | two `spawn`s, two `join`s |
| `tokio::time::timeout(d, f)` | an operation deadline, or `s.set_deadline(clock.now() + d)` |
| `buffer_unordered(4)`, `Semaphore` | `tasks.limit(4)` |
| `tokio::select!` | one channel carrying an enum (`TD0605`) |
| `Arc<Mutex<T>>`, `m.lock().unwrap()` | `Shared<T>`, `m.with(\|x\| ..)?` (`TD0604`) |
| `catch_unwind`, `thread::spawn(..).join()` for isolation | `tasks.spawn_isolated(move \|\| ..)` |
| `rayon::join`, `par_iter().map()`, `.reduce(..)` | `join`, `par_map`, then a sequential fold (section 8) |

## 15. Example programs

### 15.1 CLI: arguments and files

`lc [--min N] FILE...` counts lines with at least `N` characters per file.

```rust
// src/main.rs
use std::fs::Fs;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Error)]
enum CliError {
    #[error("usage: lc [--min N] FILE...")]
    Usage,
    #[error("not a number: {0:?}")]
    BadNumber(String),
    #[error("cannot read {path:?}: {source}")]
    Read { path: PathBuf, source: io::Error },
}

#[derive(Debug, Default, PartialEq)]
struct Options { min: usize, files: Vec<PathBuf> }

/// Authority-free.
fn parse_args(args: &[String]) -> Result<Options, CliError> {
    let mut opts = Options::default();
    let mut it = args.iter().skip(1);                  // args[0] is the program name
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--min" => {
                let v = it.next().ok_or(CliError::Usage)?;
                opts.min = v.parse().map_err(|_| CliError::BadNumber(v.clone()))?;
            }
            s if s.starts_with("--") => return Err(CliError::Usage),
            s => opts.files.push(PathBuf::from(s)),
        }
    }
    if opts.files.is_empty() { return Err(CliError::Usage); }
    Ok(opts)
}

/// Authority: reads files through `fs`, nothing else.
fn run(fs: &Fs, opts: &Options) -> Result<String, CliError> {
    let mut report = String::new();
    for path in &opts.files {
        let text = fs.read_to_string(path)
            .map_err(|source| CliError::Read { path: path.clone(), source })?;
        let n = text.lines().filter(|l| l.chars().count() >= opts.min).count();
        report.push_str(&format!("{n:>8} {}\n", path.display()));
    }
    Ok(report)
}

#[tide::main]
fn main(root: Root) -> ExitCode {
    let args: Vec<String> = root.env.args().collect();
    let fs = root.fs.read_only();                      // closures may not capture `root`
    match parse_args(&args).and_then(|opts| run(&fs, &opts)) {
        Ok(report) => { print!("{report}"); ExitCode::SUCCESS }
        Err(e) => { eprintln!("lc: {e}"); ExitCode::FAILURE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::Mode;

    #[test]
    fn counts_in_a_simulated_directory() {
        let sim = Root::sim(7);
        sim.world.file("/w/a.txt", "one\ntwo\n\nthree\n");
        let fs = sim.fs.dir("/w", Mode::Read).unwrap();
        let opts = Options { min: 1, files: vec![PathBuf::from("a.txt")] };
        assert_eq!(run(&fs, &opts).unwrap(), "       3 a.txt\n");
        assert!(matches!(parse_args(&[String::from("lc")]), Err(CliError::Usage)));
    }
}
```

### 15.2 Bounded concurrent fetcher with a deadline

```rust
// src/main.rs
use std::net::{Net, NetError, Url};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum Outcome { Fetched(usize), Failed(String), TimedOut }

/// At most 4 requests in flight; no request runs past the deadline.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url], budget: Duration)
    -> Result<Vec<Outcome>, TaskError>
{
    let deadline = clock.now() + budget;
    tasks.limit(4).scope(|s| {
        let handles: Vec<_> = urls.iter()
            .map(|url| s.spawn(move || Ok(fetch_one(net, url, deadline))))
            .collect();
        handles.into_iter().map(|h| h.join()).collect()
    })
}

fn fetch_one(net: &Net, url: &Url, deadline: Instant) -> Outcome {
    match net.get(url, deadline) {
        Ok(body) => Outcome::Fetched(body.len()),
        Err(NetError::Timeout) => Outcome::TimedOut,
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

#[tide::main]
fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let urls: Vec<Url> = root.env.args().skip(1).map(|a| Url::parse(&a)).collect::<Result<_, _>>()?;
    for (url, out) in urls.iter().zip(fetch_all(&root.tasks, &root.net, &root.clock, &urls, Duration::from_secs(5))?) {
        println!("{url}: {out:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{Reply, SimRoot};

    #[sim_test]
    fn slow_host_times_out(sim: SimRoot) {
        sim.world.http("fast.test:443", |_req| Reply::ok("hello").after(Duration::from_millis(20)));
        sim.world.http("slow.test:443", |_req| Reply::ok("late").after(Duration::from_secs(30)));
        let urls = [Url::parse("https://fast.test/").unwrap(), Url::parse("https://slow.test/").unwrap()];
        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls, Duration::from_secs(2)).unwrap();
        assert_eq!(got, vec![Outcome::Fetched(5), Outcome::TimedOut]);
        assert!(sim.elapsed() <= Duration::from_secs(2));   // virtual time: runs instantly
    }
}
```

Each child passes the deadline to `net.get`, which returns `Timeout` once
it passes, even for a child that starts late, so every child turns its
error into an `Outcome` and the scope returns `Ok` with one outcome per
URL. A scope deadline would lose outcomes: once it cancels the scope, a
limited `spawn` starts nothing (7.1).

### 15.3 Contracts and a generic search

```rust
// src/search.rs
use std::cmp::Ordering;

/// Like `slice::binary_search`.
#[requires(xs.is_sorted())]
#[ensures(match result {
    Ok(i) => i < xs.len() && xs[i] == *target,
    Err(i) => i <= xs.len()
        && (forall |j: usize| j < i ==> xs[j] < *target)
        && (forall |j: usize| i <= j && j < xs.len() ==> xs[j] > *target),
})]
pub fn binary_search<T: Ord + Pure + Sync>(xs: &[T], target: &T) -> Result<usize, usize> {
    let (mut lo, mut hi) = (0, xs.len());
    while lo < hi {
        invariant!(lo <= hi && hi <= xs.len());
        let mid = lo + (hi - lo) / 2;                   // (lo + hi) / 2 could overflow
        match xs[mid].cmp(target) {
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
            Ordering::Equal => return Ok(mid),
        }
    }
    Err(lo)
}

#[requires(xs.is_sorted())]
#[ensures(xs.is_sorted())]
#[ensures(xs.len() == old(xs.len()) + 1)]
#[ensures(xs.contains(&x))]
pub fn insert_sorted(xs: &mut Vec<i64>, x: i64) {
    let at = xs.partition_point(|&y| y < x);
    xs.insert(at, x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_inserted(mut xs: Vec<i64>, x: i64) {
        xs.sort();                                      // construct, don't filter (10.2)
        insert_sorted(&mut xs, x);
        assert!(binary_search(&xs, &x).is_ok());
    }
}
```

`binary_search`'s clauses use `==` and `<` on `T` exactly as code does, so
`T` is bounded `Pure + Sync` (10.1). Its `requires` and quantified
`ensures` are sampled (10.1). Generated contract tests (10.2) run both
functions with `T = i64`, and `xs.is_sorted()` is a recognized shape, so
sorted inputs are generated rather than filtered.

### 15.4 A crash-safe log and its simulation test

`Log` follows 6.3's durability rules and cuts a torn tail; the test crashes
and restarts its node (17) and checks that every acknowledged line
survived. `Log` uses only built `Fs` and `File` operations; its test is
planned, since it needs nodes and faults.

```rust
pub struct Log { file: File }

impl Log {
    /// Opens or creates `events`, dropping a torn tail left by a crash.
    pub fn open(dir: &Fs) -> io::Result<Log> {
        let mut file = dir.open_with("events", OpenOptions::new().read(true).append(true).create(true))?;
        dir.sync_dir(".")?;                           // the new entry survives a crash
        let mut text = Vec::new();
        file.read_to_end(&mut text)?;
        let keep = text.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        if keep < text.len() {
            file.set_len(keep as u64)?;               // never acknowledged, so safe to drop
            file.sync()?;
        }
        Ok(Log { file })
    }
    /// Durable when it returns Ok; acknowledge only after that.
    pub fn append(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.file, "{line}")?;
        self.file.sync()
    }
    pub fn lines(&mut self) -> io::Result<Vec<String>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut s = String::new();
        self.file.read_to_string(&mut s)?;
        Ok(s.lines().map(String::from).collect())
    }
}
```

```rust
#[sim_test(seeds = 200)]
fn acked_lines_survive_a_crash(sim: SimRoot) {
    let node = sim.node("db");
    node.start(|r: NodeRoot| {
        r.fs.create_dir_all("/data")?;
        r.fs.sync_dir("/")?;                              // /data itself survives
        let mut log = Log::open(&r.fs.dir("/data", Mode::ReadWrite)?)?;
        for i in 0..100 {
            log.append(&format!("entry {i}"))?;
            info!("acked {i}");                           // read back below
        }
        Ok::<(), io::Error>(())
    });
    sim.faults.at(Duration::from_millis(20), Fault::crash(&node));
    sim.faults.at(Duration::from_millis(40), Fault::restart(&node));   // reruns the entry
    sim.run_for(Duration::from_secs(1));
    let disk = node.fs();
    disk.create_dir_all("/data").unwrap();
    let lines = Log::open(&disk.dir("/data", Mode::ReadWrite).unwrap()).unwrap().lines().unwrap();
    let records = sim.records();
    for i in records.iter().filter(|r| r.node == "db").filter_map(|r| r.message.strip_prefix("acked ")) {
        assert!(lines.contains(&format!("entry {i}")), "acked entry {i} lost");
    }
}
```

`tide test` reports a failing seed with a replay command that loads its
manifest:
`tide sim --replay target/sim/acked_lines_survive_a_crash-1729.manifest.json`.
`tide test acked_lines_survive_a_crash --seed 1729 --trace` instead reruns
that seed against the current source, for example after a fix.

## 16. Where the documents disagree

The chapters are synchronized with 01 draft 3.7. These points remain open;
this reference takes the reading shown.

1. **`root.fs` and relative paths.** Settled by `fx-byk.1`: both 03 §1.1
   and reference 6.3 resolve relative paths against the starting working
   directory (`/` under simulation).
2. **`with_all`.** 01 D8 says cells are accessed only through `with` and
   `with2`; 02 §8.4 and 03 §6.5 add `with_all` under a provisional name.
   Kept (7.4).
3. **`#[pure]`.** 03 §3.2 settles it (`TD0511`); 01 still lists it as open
   question 4. Kept (6.6).

### File backend behaviour (6.3; fx-byk.1 and fx-byk.2)

Production provides real and simulated `Fs`, `File`, `OpenOptions`, listing
and metadata items; conformance includes those same definitions.
The recorded std choices (research/19-files.md S1–S12; 03 §1.1 and §9) are:

- Listings snapshot names sorted by bytes; metadata exposes type and length.
- Directory length is 0 on both backends because the host's directory
  representation is not modeled.
- Linux's 4096-byte pathname limit (including NUL) and 255-byte name limit
  apply to each syscall-shaped argument.
- Real `write_synced` follows at most 40 final symlinks, separately from each
  parent-directory lookup's Linux budget. Ordinary `write` has Linux's total
  40-link pathname budget; mixed directory/final chains can therefore succeed
  only with `write_synced` (03 §1.1).
- Entry-changing `rename`, `remove_file`, and `remove_dir` reject final `.`
  or `..` with `InvalidInput`. File access mismatches give `PermissionDenied`,
  and a read-only `Fs` cannot produce a `ReadWrite` child.
- Confined real operations need Linux `openat2` and fail closed with
  `Unsupported` if unavailable. Real files are Linux-only in v0; tide-std does
  not build on other targets. Root relative
  paths use the starting working directory. Kinds, rather than error text or
  raw OS codes, define real/simulated equivalence.
- Simulated file calls take one scheduling point and one uniform
  `0..=1_000_000` ns draw from a separate fs seed stream. Pre-cancelled calls
  have no draw, time or effect; shielded close still takes its point and draw.
  Multi-call operations apply whole, a limit until fault modelling.
- `world.file` returns `()`, creates parents and replaces file contents
  from simulated `/`, immediately without scheduling, time, draws or mode
  checks. Setup errors panic; setup records a `world` trace event.

## 17. Simulation: faults, nodes, crashes

The rest of 6.8's simulator: fault injection, network time, setup types,
crashable nodes, records, and replay. Records, the setup types of `args`,
`env_var`, `stdin`, and `file`, and the per-packet TCP latency are built,
and so, in part, is the replay manifest (README Status); everything else
here, packet faults included, is planned; `sim.world.http` peers and
`Net::get` timing are built (research/42 §9).

| Item | Effect |
| --- | --- |
| `sim.world.http(addr: &str, f: impl Fn(&Request) -> Reply + Send + 'static)` | a simulated HTTP peer at `"host:port"` (an `only` entry's form; invalid or duplicate panics), serving `http` and `https` alike; `req.url() -> &Url` removes the fragment; `Reply::ok(body: impl Into<Vec<u8>>)`, `.after(d)`, `Reply::status(503)` (a final status, 200–599; others panic), `Reply::hang()`, `Reply::reset()` (`Reset`), `Reply::tls_error()` (`Io`, `InvalidData`); every reply obeys 21's body limit |
| `sim.net.drop_all() -> Net` | a `Net` whose every operation times out |
| `sim.faults.net_drop(p: f64)`, `net_duplicate(p)`, `net_delay(min, max)`, `net_reorder(on)` | per-packet network faults (below) |
| `sim.faults.partition(&[&a], &[&b])`, `heal()` | cut links between node groups |
| `sim.faults.disk_error("/data", p, io::ErrorKind::Other)` | I/O errors on paths with that prefix |
| `sim.faults.torn_writes(on)` | whether a crash may keep a torn prefix of unsynced writes (default on, D15) or only lose them |
| `sim.faults.crash(&node)`, `restart(&node)`, `at(t, Fault::crash(&node))` | crash or restart now, or schedule any `Fault` at virtual time `t` |
| `sim.faults.fault_point(name, p)` | probability that `tasks.fault_point(name)` returns `true` |
| `sim.node("kv") -> SimNode` | a node with its own disk, address (`"kv"` resolves to it), clock, and tasks |
| `node.start(entry) -> NodeTask<T, E>` | runs `entry(NodeRoot) -> Result<T, E>` as the node's main task; `NodeRoot` has `Root`'s fields |
| `node.fs() -> Fs` | the node's disk, for inspection after a crash |
| `sim.run_for(d)`, `sim.run_until(&node_task) -> Result<T, E>` | drive the world |

(`Request`, `SimNode`, and `NodeTask` are provisional.)

- **Network time.** Without faults, each packet takes 1–10 ms, drawn from
  the seed, and a `Net::get` is one round trip (2–20 ms, connection setup
  and name resolution included) plus the reply's `after(d)`. An address
  with no HTTP peer, IP hosts included, gives `Refused` after one round
  trip; a domain that no peer or node names gives `Dns` at once. A
  simulated `get` reaches only `world.http` peers, never TCP listeners.
  Registering a peer on a simulated listener's address panics at setup;
  a later bind on a peer's address gives `AddrInUse`, wildcard/loopback
  aliases included.
- **Name resolution** (research/48 §6). A simulated
  `Net::resolve` takes no time and no draw. Apart from `localhost`
  (which returns `127.0.0.1` with or without a peer), a domain that some
  `world.http` peer names, on any port, resolves to one address,
  `198.18.0.n`. Ordinals start at 1 and count distinct domain peer host
  keys after NET-HOST canonicalization in setup order. `localhost` and
  IP-address peers take no ordinal. Resolving a granted name reveals
  its ordinal, and so the number of earlier distinct peer names,
  including names outside that Net's grants (`localhost` and IP peers
  excepted). Other domains return `Dns`. Simulated `get` and `resolve`
  success for a name proves neither DNS encodability nor real resolver
  acceptance. Real resolver failures are `Dns`. To `get` by synthetic
  address, register an HTTP peer at that address. Without an
  address registration, `get` by address returns `Refused`.
  Simulated TCP stays loopback-only and cannot exercise resolution
  followed by `connect` to a synthetic address.
- **HTTP peer panics.** After the callback, `get` checks scope cancellation
  first (`Cancelled`), then the request deadline (`Timeout`). If neither
  applies, a peer panic propagates to the requesting task.
- **Setup types.** `world.args` takes any `IntoIterator` of `AsRef<OsStr>`
  items and `env_var(k, v)` two `AsRef<OsStr>` values, as `Command::args`
  and `Command::env` do; `stdin` and `file` contents take `impl
  AsRef<[u8]>`, so `"text"` and `b"bytes"` both work. A `world.http` peer
  is `Fn(&Request) -> Reply + Send + 'static`: the simulator calls it from
  the requesting task's thread, so state shared with the test body is a
  synchronization handle from `sim.tasks`.
- **Packet loss.** `net_drop(p)` drops each packet independently with
  probability `p`. A UDP datagram is simply lost. TCP streams and
  `Net::get` retransmit, so loss shows up as delay: each loss of the same
  packet waits a retransmission timeout, 200 ms and doubling (provisional).
  An operation fails only when its own deadline passes (`Timeout`) or its
  scope is cancelled (`Cancelled`).
- **Replay manifest**: seed, content hashes of the source and every
  dependency, compiler version, target, semantics-affecting build options,
  simulated setup, and injected inputs. Replay refuses to run if the source
  or build differs, and otherwise reproduces the run bit for bit.
- **Replay.** `tide test` reports a failing seed with a replay command,
  `tide sim --replay target/sim/<test>-<seed>.manifest.json`;
  `tide test <test> --seed <seed> --trace` instead reruns that seed against
  the current source. `tide sim <bin> -- args` runs `main` under many seeds
  as one node.
- **Records.** A `Record` has `node: String`, `time: Duration`, `sink:
  Sink` (`Out`, `Err`, `Dbg`, or `Log(Level)`), and `message: String`, one
  line without its newline (provisional). A node's records stay in
  `sim.records()` if the node later crashes.
- **Node entries.** `restart` calls the entry again, so it is a `fn` item or
  an `Fn` closure, never `FnOnce`. It is `Send`, and its captures are
  `'static`, `Pure`, and `Sync`, so a run cannot change them; clone inside it
  what each run consumes. Unlike `Root`, a `NodeRoot` is an ordinary value:
  pass it or its fields to helpers. The simulator's handles (`SimRoot`,
  `NodeRoot`, `sim.world`, `Faults`, `SimNode`, `NodeTask`) are not `Pure`
  (02 §9), so an entry cannot capture a node or the fault controls.
- **Crashes.** A crash stops a node's tasks at their current scheduling
  points without running destructors, frees its memory, and resets its
  peers' connections. Synced data survives (6.3) if its entry and the
  directories above it are synced too; unsynced writes may be lost or torn,
  as the seed decides. Each unsynced directory-entry change (`create`,
  `rename`, `remove_file`, `remove_dir`, a new directory) survives whole or
  is lost. A `rename` leaves the target naming the renamed file with the
  source gone, or both as they were. `remove_dir_all` is one change per path
  it removes, so a crash can leave part of the tree; `sync_dir` of the
  removed directory's parent makes the whole removal durable. Which unsynced
  changes survive is otherwise unspecified, except that a change survives
  only if the changes that created its entries and every directory above its
  paths do, a change that takes a name only if the change that freed it
  does, and a directory's removal (by `remove_dir` or a `rename` over it)
  only if every removal in it and every `rename` out of it does. Crashing a
  node whose entry has returned is the same power cut (provisional). The
  test body itself cannot crash: put crash-tested code in a node.
- **Restarts.** `restart(&node)` crashes a running node, then calls the
  entry again with a fresh `NodeRoot` over the surviving disk. A `NodeTask`
  follows its node: `run_until(&t)` returns when some run of the entry
  returns, and panics if the node is down with no restart scheduled.
  Dropping it does not stop the node. (All provisional.)

15.4 is a complete crash test.

## 18. Sealed interfaces (planned)

An orchestrator that hands you an interface to implement may seal it in
`tide.seal` (04 §11.1). You may change the bodies of functions sealed by
interface, the default, and add items other than impls that apply to sealed
types (`impl Bar for &Feed` and blanket impls such as `impl<T> Bar for T`
included); write helpers as free functions. You may not change, move, or
delete a sealed item, the definitions of the types its signature or contract
names, a function its contract calls, a sealed test or anything it reaches
short of an interface-sealed body, or, if the manifest is sealed, its
grants, policies, and `[contracts]` table. `tide check` rejects such a
change before anything builds:

```text
error[TD0920]: sealed item `feed::fetch_feed` changed
  signature  + parameter `cache: &Fs`
  authority  net -> net+fs  widened
  contract   - #[ensures(match &result { .. })]  weakened
```

Editing `tide.seal` yourself does not help: the merge checks your branch
against the base's seal and reports a removed or changed entry as
`TD0921`. When a sealed item blocks the task, for example because the work
needs a capability the signature does not take, do not route around it in
unsealed code: restore the item and report what the interface needs to
whoever sealed it.

## 19. Testing an unreviewed branch (planned)

Nothing below exists yet: the sandbox is RUN-3 (bead `fx-ip4x`), and no
`tide` command is built. Until then nothing prints `run: covered`, so
review another agent's branch before testing it, or test it in a VM.

`tide check`, `tide test`, and `tide sim` on a branch no one has reviewed
reach nothing outside the run except through the holes 03 §2.6 lists. Test
code holds only simulated capabilities, the build reads only the branch's
tree, and each test process runs with a toolchain-set environment, no
terminal, and an OS sandbox that admits file opens only of its binary,
libraries, and output directory and refuses new processes and sockets.
Before testing another agent's branch without a VM of its own, run `tide
review <base>`, where `<base>` is a reviewed revision, such as a commit
fetched from the protected remote, never a commit of the branch itself:

```text
run: not covered    sandbox: full    base 87f9cd3    snapshot 9c41e2…
  + grant unsafe          acme/fastcrc
  ~ src/ring.rs           package has an unsafe grant
```

Run the branch's tests in your session without a VM of their own only when
its first line says `run: covered` and `sandbox: full`, and then only as
`tide test --snapshot <hash>` and `tide sim --snapshot <hash>` with the full
hash from `tide review --json`, which run no test on any other tree.
Otherwise review the listed holes first, or run the tests inside a VM. For
a covered branch too, treat a pass as evidence, not proof: a toolchain bug or an unsound
reviewed `unsafe` abstraction can forge results. A covered branch can still
exhaust CPU, memory, or output, so set time and memory limits. This covers
`tide check`, `tide test`, `tide sim`, and `tide review` only: `tide run`,
`tide eval`, a built binary, and anything else the branch asks you to
execute still need your session's sandbox.

## 20. Process handles

A `Command`, like a `File`, is a resource handle: it carries the authority
of the `Proc` that made it, because `output()`, `status()`, and `spawn()`
start a process with no capability argument. So `fn configure(cmd: &mut
Command)` is not authority-free, and `Command` is not `Pure`. `OpenOptions`
is the opposite: `Pure` data that does nothing until `fs.open_with` takes
it.

```rust
use std::{io::Write, process::Stdio};
let mut child = proc.command("sort")
    .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
let mut input = child.stdin.take().unwrap();   // ChildStdin: write to the child
input.write_all(b"b\na\n")?;
drop(input);                                   // close it: sort sees the end
let out = child.wait_with_output()?;           // out.stdout == b"a\nb\n"
let ok = proc.command("make").stdin(Stdio::null()).status()?.success();
```

As in Rust, `status` and `spawn` inherit stdin unless you set it, and the
simulator does not model inherited stdin, so set `Stdio::null()` or
`piped()` in code you test under simulation (below).

- **Rust's API.** `Command` and `Child` keep Rust's methods and receivers,
  with two exceptions. There is no ambient `Command::new` (`TD0501`), and
  `wait_with_output` takes `&mut self`, not `self`, so a cancelled call
  leaves the `Child` with you to `kill` and `wait` under `shield` (03 §1.1).
  As in Rust, the builder methods, `output`, `status`, `spawn`, `wait`, and
  `kill` take `&mut self`. `Child` keeps its public fields `stdin`,
  `stdout`, and `stderr`, each an `Option` of a pipe end (`ChildStdin`,
  `ChildStdout`, `ChildStderr`), so `child.stdout.take()` works. Each handle
  still has a private field, so no struct literal can make one (03 ROOT-4).
- **Pipe ends** carry `Proc`'s authority and are not `Pure`; their reads
  and writes suspend and are checkpoints.
- **Cancellation.** A cancelled `output()` or `status()` whose child has
  started tries to kill the child (as `Child::kill` does, so not the
  processes it started), closes its ends of the child's pipes, waits for
  it to exit even if the kill failed, and returns
  `io::ErrorKind::Cancelled`; output read so far is lost. A failed
  kill, such as `EPERM` for a setuid child, is logged as a `warn!` record,
  and the call waits for the child to exit by itself. No deadline cuts that
  wait short (03 TASK-5). To let a child outlive a cancellation, `spawn` it
  and keep the `Child`: as in Rust, dropping a `Child` does not kill it
  (03 §1.1).
- **`Stdio`** has `piped()`, `inherit()`, and `null()`, which take no
  capability, and `From` a `File` or a pipe end, so
  `.stdin(a.stdout.take().unwrap())` pipes child `a` into the next. As in
  Rust, `.stdout(io::stderr())` sends a child's output to your standard
  error; used as a `Stdio`, `io::stdout()` and `io::stderr()` name the
  process's descriptors, as `inherit()` does, even when `root.log` sends the
  sinks to a file. There is no `From<OwnedFd>`. A `Stdio` can hold a handle,
  so it is not `Pure`.
- **Output order.** Starting a child whose stdout or stderr is the process's
  own (inherited, or `io::stdout()` or `io::stderr()`) first writes out
  every earlier `println!`, `eprintln!`, `dbg!`, and log record (a log
  record may instead be dropped under overload), so the child's output
  follows them, as in Rust (03 SINK-4).
- **Authority.** A child process is not capability-checked: it runs with
  everything your process can reach (more, for a setuid program), so
  `Proc` is ambient authority for the child, and `tide review` lists new
  paths to `proc` first. Process authority has no end. A child, and
  anything it starts, keeps running with the process's authority after
  the call that started it returns, and after the Tide process exits.
  Lending `&Proc` or a `Command` for one call is not a temporary grant.
- **`only`** (03 PROC-ONLY). `proc.only(["git", "/usr/bin/tar"])` takes bare
  names and absolute paths, matched byte for byte against `command`'s
  program, with no normalization or symlink resolution; `"./tool"` grants
  nothing. A refused program returns `PermissionDenied` from `spawn`,
  `output`, or `status` and starts nothing. Std looks a bare name up itself
  in the `PATH` your process started with (absolute elements only;
  `NotFound` if it has none), so `.env("PATH", ..)` changes only the
  child's own `PATH`, unlike Rust. Whoever can write a directory on that
  `PATH`, or replace the file an entry names, chooses the program.
  Under `only`, setting an `LD_` or `DYLD_` variable, or a C-library one
  such as `GCONV_PATH` (the list is in 03 PROC-ONLY), or a name holding
  `=`, on the command returns `PermissionDenied`. `only` picks the program, not what it does:
  an allowed shell, interpreter, `env`, `xargs`, or a program that runs
  what its arguments name (`git -c core.sshCommand=..`, `find -exec`) is
  full authority. A script, a program steered by the files it reads (git's
  config and hooks in `current_dir`, a Makefile), or one whose output you
  read back gives full authority as well (03 PROC-ONLY).
- **Environment and descriptors.** The child inherits your whole
  environment, as in Rust, whatever your `Env` allows; use `env_clear` or
  `env_remove`. `current_dir` needs no `Fs`. The child gets only the three
  descriptors its `Stdio` sets. The real backend needs Linux 5.11 for
  that; on an older kernel `spawn`, `output`, and `status` fail and start
  nothing.
- **Simulation.** `sim.world.proc("git", model)` declares a model for an
  exact program spelling (no `PATH` search); `ProcessModel::missing()`
  makes spawning it return `NotFound`. An unregistered program or an
  unsupported mode (inherited stdin, `Stdio` from a file or another child's
  pipe) stops the seed with a coverage report, never starting a host
  process. Exit codes, per-`Child` waits, `kill`, and piped I/O follow 03
  §9.9's lifecycle subset; model spellings are provisional. A status with
  no code (after `kill`, or a model's write to a closed pipe) has
  `code() == None` and prints as on Linux (`signal: 9 (SIGKILL)`).
  A model is data built with `ProcessModel::new()`: `args`, `env`, and
  `current_dir` restrict the command it accepts, and the steps `read`,
  `read_to_end`, `copy`, `write`, `write_err`, `close_stdin`/`stdout`/
  `stderr`, `delay(min, max)`, `choose([..])`, and `exit(code)` run in
  order (exit 0 after the last step); `ignore_sigpipe()` makes a write to a
  closed pipe fail and continue.

```rust
#[sim_test]
fn sorts(sim: SimRoot) {
    let sort = ProcessModel::new().read("b\na\n").read_to_end().write("a\nb\n");
    sim.world.proc("sort", sort);
    let mut child = sim.proc.command("sort")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"b\na\n").unwrap();  // dropped: closed
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.stdout, b"a\nb\n");
}
```
- **Paths.** `Command`, `Child`, the pipe ends, and `Stdio` are in
  `std::process`, and `ReadDir` and `DirEntry` in `std::fs`: Rust's paths,
  where Rust has the type.
- **Receivers.** Only the capability types of 6.1 take `&self` for every
  method. Handles keep Rust's receivers, except `wait_with_output`; of
  Tide's own handle methods, `File::sync`, `read_owned`, and `write_owned`
  take `&self`, and `File::close` takes `self`.

## 21. Network: HTTP

`Net::get(&Url, deadline: Instant) -> Result<Vec<u8>, NetError>` performs
one HTTP/1.1 GET, over TLS for `https`, and returns a 2xx response's body.
`NetError` (`#[non_exhaustive]`) has
`Timeout` (the operation's deadline passed), `Cancelled`, `Denied` (a host
outside `only`), `Refused`, `Reset`, `Dns`, `Status(u16)` (a non-2xx
response), and `Io(io::Error)`; `is_cancelled()` and `is_transient()`
(`Timeout`, `Reset`, `Refused`, and `Status` 408, 429, or 5xx) classify
them. `NetError` is not `PartialEq`: test it with `matches!`.
`Url::parse(&str) -> Result<Url, UrlError>`; `Url` has the `url` crate's
accessors (`scheme`, `host_str`, `port`, `port_or_known_default`, `path`,
`query`, `fragment`, `as_str`), no setters, `Display`, `Eq`, `Hash`, `Ord`
and `Encode`/`Decode` as a JSON string. Simulated HTTP peers are in 17.

- **Wire.** The request is fixed: `GET`, `Host`, `User-Agent: tide`,
  `Accept: */*`, `Connection: close`; no body, no custom header, no
  cookies. `Host` uses the key's host (IPv6 in brackets), with the port
  unless it is the scheme's default; SNI uses the key's DNS host, never
  an IP host. The fragment is not sent. One connection per call, closed on
  return; no pool, TLS session reuse or Tide-owned cache. The system
  resolver may share caches across calls and Nets, substitute configured
  aliases and follow DNS aliases. Cancellation abandons its result, not
  its remaining traffic; a name grant authorizes this operator-controlled
  mechanism.
- **Refused before the network.** A scheme other than `http` or `https`,
  or a URL with userinfo (`user:pw@`), is `Io` with `InvalidInput`; a host
  outside `only` is `Denied`. `Url::parse` refuses non-ASCII input, and
  in an `http` or `https` host a `%`, an empty label (`a..b`) or an `xn--`
  label (no internationalized hosts in v0): percent-encode non-ASCII paths
  and queries.
- **Not followed or read.** A 3xx is `Status(code)`: fetch the final URL.
  Proxy variables, `SSL_CERT_FILE` and system trust stores are ignored;
  TLS trusts Mozilla's roots compiled in (`webpki-roots`), so a private CA
  fails with `Io`, `InvalidData`, as any TLS handshake/verification or
  malformed-response failure does. So are a `101` (no upgrade is asked for) and a body over 64 MiB
  (provisional). Interim 1xx heads are skipped; a `204` or `304` ends at
  its head (`204` is `Ok` and empty). A close-delimited HTTPS body ends
  only at TLS `close_notify`; a bare close is `Reset`. A complete framed
  2xx body remains `Ok` on a bare close. If all connection attempts fail,
  all-refused gives `Refused`, otherwise `Io` of the first non-refusal error.
- **TLS guard.** `NoKeyLog`, resumption disabled; Tide never constructs
  `KeyLogFile` and ignores `SSLKEYLOGFILE`. Disabling rustls `logging`
  alone does not establish this.
- **Simulation limit.** A peer models an application response, not TLS
  certificate validation or wire parsing. Simulated HTTPS success asserts
  a successful authenticated transport; it does not prove one.
  `Request::url()` removes the fragment. Every `Reply`, `Reply::ok`
  included, obeys the 64 MiB body limit: larger is `Io(InvalidData)`.
- **Name resolution** (research/48).
  `Net::resolve(host: &str, deadline: Instant) -> Result<Vec<IpAddr>,
  NetError>` returns a nonempty list on `Ok`. A real non-literal lookup
  preserves the resolver's order and duplicates. Given the same resolver
  answer, it returns the list `connect`'s lookup uses with ports removed.
  An unbracketed `::ffff:a.b.c.d` literal resolves to its canonical IPv4
  address. TCP `connect((host, port))` preserves the IPv6 address for that
  spelling. In simulation, names other than `localhost` get one
  deterministic synthetic address if a `world.http` peer names them, or
  `Dns` otherwise (§17). `host` is keyed as `connect((host, port))` keys
  it: a port, URL, empty label, `%`, `xn--` or `*` is `Io(InvalidInput)`.
  Under `only` it is allowed when an entry names the host on any port,
  else `Denied` before any lookup. A passed deadline is `Timeout`,
  literals included. After that check, an IP host or `localhost` returns
  its canonical address without lookup. A lookup failure is `Dns`.
  A name grant does not grant the addresses it returns. Connect by
  name. The answer binds no later `connect` or
  `get`, which look the name up again. No reverse lookups or other record
  types.
- **What `only` bounds.** Outbound peer operations (`get`, `connect`,
  `resolve`, planned peer operations) under 03's NET-HOST rule: exact
  `"host:port"` keys (`resolve` matches the host on any port), every IPv4 spelling as its address, names never
  matching their addresses, checked before their lookup. A URL without a
  port keys as 80 for `http` and 443 for `https`. It does not
  limit `bind` or accepted connections: a binding-capable Net can resolve
  arbitrary bind names even if binding fails, and communicate with
  unlisted inbound peers.
  `only([])` alone is not network silence; delegate `only([..]).no_bind()`
  for an outbound-only destination grant. Not addresses: a name reaches
  whatever its DNS gives, loopback included, and `https` authenticates the
  name only once the handshake completes. Not data: a URL's path and query reach the
  allowed host, and an allowed relay host is full network authority
  (research/42 §6).

## Appendix A. Later tiers (not in v0)

From the tier table of 01 (draft 3.7). The chapters specify most of these.

**v1:**

- **Proofs** (04): the `prove` mode and the `assert`/`test`/`prove` uses
  (chosen in the manifest and by `#[contracts(..)]`) and clause classes;
  `#[decreases(..)]`, `decreases!(..)`, `#[diverges]`; type invariants
  (`#[invariant(..)]`); closure contracts (`requires!`/`ensures!` atop a
  closure body; `f.requires`); trait-method contracts and their inheritance
  by impls; `final`; `#[spec]`, `#[opaque]`, `#[trusted]`;
  proof status; `tide verify`. Spec and proof items (`spec fn`, `proof fn`,
  `law fn`, `ghost let`, `proof { }`, `assume`, views, `int`, `Seq`,
  triggers) keep draft 3.5's grammar until respelled as Rust (01 D10).
- **Authority**: exercised-authority tracking, the least-authority lint
  `TD0510`, and effect reporting for sinks; FFI (`extern`, the `ffi` grant,
  `#[sim_model]`, `ThreadBound`); imported Rust crates, linked directly
  behind wrappers that take capabilities (`[rust-dependencies]`, linked-symbol
  scan); Rust packages depending on a Tide crate through its generated
  `Cargo.toml` (RUST-6).
- **Runtime**: the evented runtime, whose lowering under D18 is open;
  production record and replay (D17).
- **Parallelism**: `par_reduce` with its canonical grouping, `Combine`,
  `Monoid`, `Sum`/`Product`/`Min`/`Max`.
- **Memory**: field-borrow declarations, spelling open under D18.
- **Codecs** (01 D6): strict JSON objects (`#[codec(deny_unknown_fields)]`,
  CODEC-1) and internally tagged unit and named-field enum variants
  (`#[codec(tag = "..")]`, CODEC-2).

**Later:** the Bend tandem and `Accel` (D16); deferred overflow traps;
automatic parallelization; `Shape` reflection; Polonius-precision borrows;
interactive-proof export; snapshot-and-branch simulation; the rest of 03
§9.9 beyond its v0 lifecycle subset: modeled signals, raw PTYs,
parent-only process death and process faults through Proc (D15), including
`Proc::signals` and `Signals`, the
API additions (`Child::signal`, `ExitStatus::signal`, `spawn_pty`/`Pty`) and
subscription rules (one live subscription with `AlreadyExists`, default
disposition on drop, five-signal set) on both real and simulated backends.

**Removed** (01 draft 3): `Frozen`, the `lock` keyword, `par { }`, and the
moved-out rule. Not yet tiered by 01, and left out of v0 here: codec formats
other than JSON, the codec plug-in API, the `flatten` and `try_from`
keys, and the adjacent, untagged, catch-all and tagged newtype or map enum
representations (01 D6); `pct` scheduling and `--minimize` in the
simulator.

