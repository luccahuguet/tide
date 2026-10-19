# Tide reference companion

Status: the on-demand part of `design/reference.md`, the agent-facing
reference for the v0 core. Load it when you want a Rust habit's fix, a
complete program, the reading the reference takes where the design
documents disagree, the simulator's fault injection and crashable nodes,
or what v0 leaves out. Its sections continue the core's numbering, and a
bare section number below 14 refers to the core. Sections 14–16 tabulate
or illustrate rules the core states; 17 and Appendix A hold what the core
only points to. It is outside the core's 25,000-token budget (D14).

Core count, 2026-10-19: 24,950 tokens (CI counting is planned). Method:
input tokens that `claude -p --model claude-opus-5-5 --tools ""
--system-prompt "Reply with OK." --output-format json` reports with
`reference.md` as the prompt, minus those for the prompt `.`.

## 14. Rust habits and their fixes

### 14.1 Habits

| Rust habit | Tide | Code |
| --- | --- | --- |
| `trait Foo<'a>`, `for<'a>`, `type Item<'a>` | a lifetime on the method or impl, `Fn` sugar, owned items or indices | `FX0201`, `FX0404`, `FX0403` |
| `impl Lexer { .. }` for `struct Lexer<'a>` | `impl Lexer<'_>` or `impl<'a> Lexer<'a>` | error with fix |
| `fn new(s: &str) -> Self` in `impl<'a> P<'a>` | `fn new(s: &'a str) -> Self` | error with fix |
| `Box<dyn Fn()>` in a signature or field, capturing a borrow | `Box<dyn Fn() + 'a>`, or move owned data in | `FX0203` |
| Self-referential struct | indices, `Arena`, or owner and view apart | `FX0210` |
| `macro_rules!`, proc macros | functions, generics, generators | `FX0101` |
| `#[derive(Serialize, Deserialize)]`, `thiserror::Error` | `Encode`, `Decode`, `Error` | `FX0102` safe |
| `debug_assert_eq!` | `debug_assert!(a == b)` | `FX0103` |
| `include_str!`, `env!`; `x.sin()`, `x.to_bits()`, `a.min(b)`; `core::sync::atomic`; `path.exists()` | a generated `const`; `math::sin(x)`, `float::bits(x)`, `float::min(a, b)`; `AtomicU64::new(0, tasks)`; `fs.exists(path)` | `FX0520` |
| `x as u8` (lossy), `n as f64` (`usize`) | `u8::try_from(x)?`, `x.truncate()`, `f64::round_from(n)` | `FX0301` |
| Relying on release-mode wrapping | `wrapping_add`, `Wrapping<T>` | run-time trap |
| `impl Display for T` in another module | move it to `T`'s module | `FX0401` |
| `std::fs::read`, `OpenOptions::open`, `env::args`, `Instant::now`, `thread_rng` | capability methods (14.2) | `FX0501` |
| `static COUNTER: AtomicUsize`, `OnceLock`, `lazy_static!`, `thread_local!` | a `const fn`-computed `static`, or a value built in `main` | `FX0502` |
| `fn main()` or `fn main(root: Root)` without the attribute | add `#[tide::main]` | `FX0512` safe |
| `fn helper(root: &Root)` | pass attenuated capabilities | `FX0503` |
| `Root::sim` outside tests | real capabilities from `main` | `FX0504` |
| `dbg!` left in code; `println!` in a library | remove, or `debug!`; return data or log | `FX0507`, `FX0508` |
| `async fn`, `.await`, `#[tokio::main]` | plain `fn` taking capabilities | `FX0601` safe |
| `thread::spawn`, `tokio::spawn` | `tasks.scope(\|s\| s.spawn(..))` | `FX0602` |
| I/O or a nested `with` inside a `Shared::with` closure | do it before or after | `FX0603` |
| `Arc<Mutex<T>>` + `.lock().unwrap()` | `Shared<T>` + `.with(\|x\| ..)?` | `FX0604` |
| `Shared::new(v)`, `Arc::new(v)`, `channel()` without a `Tasks` | `Shared::new(v, tasks)`, `Arc::new(v, tasks)`, `channel(tasks)`; `tide fix --thread Tasks` | `FX0505` |
| `tokio::select!` | a channel of an enum, deadlines | `FX0605` |
| `Mutex<Sender<T>>`, a `Shared` inside a `Shared` | payloads are `Send + Pure`; pass handles separately | `FX0606` |
| Capturing a capability, `&RefCell`, or `Arc` in `join`/`par_map` | compute first, borrow `Pure + Sync` data | `FX0610`, `FX0611` |
| `par_map` or `par_for` over `Shared` cells, channel ends, or files | spawn children in a `tasks.limit(n)` scope (7.1) | `FX0612` |
| `catch_unwind` | `tasks.spawn_isolated` | `FX0701` |
| Ignoring a `Result` | `?`, handle it, or `let _ =` | `FX0702` |
| `#[requires(a, b)]` | `#[requires(a)]` `#[requires(b)]` | safe fix |
| `unsafe` without a grant or `// SAFETY:` | add the grant; write the comment | `FX0901`, `FX0902` |
| `mod foo;` for a missing file, `#[path]` | create `foo.rs`; move the file | `FX0910`, `FX0911` |
| Unused `use`; `extern crate foo;` | delete | `FX0913`, `FX0912` safe |
| `a.rs` uses `b`, `b.rs` uses `a` | move shared items to a third module | `FX0914` |
| `build.rs` | a generator you run; commit its output | `FX0915` |

### 14.2 Ambient std calls (6.9)

| Rust | Tide |
| --- | --- |
| `std::fs::read_to_string(p)` | `fs.read_to_string(p)` |
| `File::open(p)` / `File::create(p)` | `fs.open(p)` / `fs.create(p)` |
| `OpenOptions::new().append(true).open(p)` | `fs.open_with(p, OpenOptions::new().append(true))` |
| `file.sync_all()` after `File::create` | `file.sync()`, plus `fs.sync_dir(parent)` |
| `std::env::args()` / `env::var(k)` | `env.args()` / `env.var(k)` |
| `Instant::now()` / `SystemTime::now()` | `clock.now()` / `clock.system_time()` |
| `std::thread::sleep(d)` | `clock.sleep(d)?` |
| `rand::thread_rng().gen_range(0..n)` | `rand.gen_range(0..n)`, or `rand.rng()` once |
| `std::io::stdin()` | a `stdin: &Stdin` parameter (`root.stdin`) |
| `Command::new("git")` | `proc.command("git")` |
| `TcpStream::connect(a)` | `net.connect(a)` |
| `std::process::exit(1)` | return `ExitCode::FAILURE` from `main`, or `proc.exit(1)` |

### 14.3 Porting async Rust (7)

| Rust | Tide |
| --- | --- |
| `async fn f(..) -> T` | `fn f(net: &Net, ..) -> T` |
| `x.await` | `x` (`FX0601`, safe fix deletes `.await` and `async`) |
| `#[tokio::main] async fn main()` | `#[tide::main] fn main(root: Root)` |
| `tokio::spawn`, `thread::spawn` | `tasks.scope(\|s\| { s.spawn(..); .. })` (`FX0602`) |
| `std::thread::scope` | `tasks.scope` (children return `Result`) |
| `tokio::join!(a, b)` | two `spawn`s, two `join`s |
| `tokio::time::timeout(d, f)` | an operation deadline, or `s.set_deadline(clock.now() + d)` |
| `buffer_unordered(4)`, `Semaphore` | `tasks.limit(4)` |
| `tokio::select!` | one channel carrying an enum (`FX0605`) |
| `Arc<Mutex<T>>`, `m.lock().unwrap()` | `Shared<T>`, `m.with(\|x\| ..)?` (`FX0604`) |
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

/// At most 4 requests in flight; every URL gets an outcome by the deadline.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url], budget: Duration)
    -> Result<Vec<Outcome>, TaskError>
{
    let deadline = clock.now() + budget;
    tasks.limit(4).scope(|s| {
        s.set_deadline(deadline);                       // cancels stragglers; not a failure
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
        Err(e) if e.is_cancelled() => Outcome::TimedOut,
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

The children turn every error, including the deadline's cancellation, into
an `Outcome`, so the scope returns `Ok` with one outcome per URL (7.2).

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
survived.

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

1. **`root.fs` and relative paths.** 03 §1.1 says `root.fs` "is rooted at
   `/`"; this reference resolves relative paths against the starting
   working directory (6.3).
2. **`with_all`.** 01 D8 says cells are accessed only through `with` and
   `with2`; 02 §8.4 and 03 §6.5 add `with_all` under a provisional name.
   Kept (7.4).
3. **`#[pure]`.** 03 §3.2 settles it (`FX0511`); 01 still lists it as open
   question 4. Kept (6.6).

## 17. Simulation: faults, nodes, crashes

The rest of 6.8's simulator: fault injection, crashable nodes, and the
replay manifest.

| Item | Effect |
| --- | --- |
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

(`SimNode` and `NodeTask` are provisional.)

- **Packet loss.** `net_drop(p)` drops each packet independently with
  probability `p`. A UDP datagram is simply lost. TCP streams and
  `Net::get` retransmit, so loss shows up as delay: each loss of the same
  packet waits a retransmission timeout, 200 ms and doubling (provisional).
  An operation fails only when its own deadline passes (`Timeout`) or its
  scope is cancelled (`Cancelled`).
- **Replay manifest**: seed, content hashes of the source and every
  dependency, compiler version, target, semantics-affecting build options,
  simulated setup, and injected inputs.
- **Records** of a node stay in `sim.records()` if the node later crashes.
- **Node entries.** `restart` calls the entry again, so it is a `fn` item or
  an `Fn` closure, never `FnOnce`. It is `Send`, and its captures are
  `'static`, `Pure`, and `Sync`, so a run cannot change them; clone inside it
  what each run consumes. Unlike `Root`, a `NodeRoot` is an ordinary value:
  pass it or its fields to helpers.
- **Crashes.** A crash stops a node's tasks at their current scheduling
  points without running destructors, frees its memory, and resets its
  peers' connections. Only synced data survives (6.3); unsynced writes and
  directory entries may be lost or torn, as the seed decides. Crashing a
  node whose entry has returned is the same power cut (provisional). The
  test body itself cannot crash: put crash-tested code in a node.
- **Restarts.** `restart(&node)` crashes a running node, then calls the
  entry again with a fresh `NodeRoot` over the surviving disk. A `NodeTask`
  follows its node: `run_until(&t)` returns when some run of the entry
  returns, and panics if the node is down with no restart scheduled.
  Dropping it does not stop the node. (All provisional.)

15.4 is a complete crash test.

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
  `FX0510`, and effect reporting for sinks; FFI (`extern`, the `ffi` grant,
  `#[sim_model]`, `ThreadBound`); imported Rust crates, linked directly
  behind wrappers that take capabilities (`[rust-dependencies]`, linked-symbol
  scan); Rust packages depending on a Tide crate through its generated
  `Cargo.toml` (RUST-6).
- **Runtime**: the evented runtime, whose lowering under D18 is open;
  production record and replay (D17).
- **Parallelism**: `par_reduce` with its canonical grouping, `Combine`,
  `Monoid`, `Sum`/`Product`/`Min`/`Max`.
- **Memory**: field-borrow declarations, spelling open under D18.

**Later:** the Bend tandem and `Accel` (D16); deferred overflow traps;
automatic parallelization; `Shape` reflection; Polonius-precision borrows;
interactive-proof export; snapshot-and-branch simulation.

**Removed** (01 draft 3): `Frozen`, the `lock` keyword, `par { }`, and the
moved-out rule. Not yet tiered by 01, and left out of v0 here: codec formats
other than JSON, the codec plug-in API, and the `flatten`,
`deny_unknown_fields`, and `try_from` keys; `pct` scheduling and
`--minimize` in the simulator.

