# E1 round 2: reference gaps and guesses

Source used: `design/reference.md` only. Each entry quotes the section it
concerns. "Guess" marks a choice the reference did not decide.

## Cross-cutting (affects several programs)

1. **Single-file programs: library or binary?** §12.1: "`ferox check path.fx`
   checks one file as a crate root (provisional), for single-file programs
   and exercises." It does not say whether a file without `main` is a
   library root (then `println!` is lint `FX0508`, §2.1) or whether
   `ferox test path.fx` exists. Guess: files with `main` are binaries,
   `csv_line`, `lexer`, `bsearch`, `bank`, `bfs` are libraries and never
   print.
2. **Is the `Error` *trait* in the prelude?** §12.3: "The prelude is Rust's
   plus `Root`, `Pure`, `Owned`, `Secret`, and the derives `Error`,
   `Encode`, `Decode`". That names only the derive, but §15.2 writes
   `Box<dyn Error>` with no import. Guess: the trait is in the prelude
   (followed §15.2), so I call `e.source()` without
   `use std::error::Error;`. That matters because §12.2 makes unused
   imports errors (FX0913). A redundant import might count as one.
3. **Can a plain `#[test]` drive a `SimRoot`'s tasks/clock?** §6.8: "The test
   body runs as the simulation's main task" is stated for `#[sim_test]`.
   §15.1 uses `Root::sim(7)` in a plain `#[test]`, but only for `fs`.
   Guess: `Root::sim(seed)` in a plain or property test works for every
   capability (used in `bank`, `kv_log`, `wc`).
4. **`#[sim_test]` with generated parameters.** §10.2: "A `seed: u64`
   parameter is an ordinary generated input ...; for one fresh simulator per
   seed, use `#[sim_test]` (6.8)". §6.8 shows only `fn t(sim: SimRoot)`. It
   is silent on whether a sim test can also take generated inputs, which a
   concurrent property test needs.
5. **Argument types of simulator setup calls.** §6.8 lists
   `sim.world.file(path, contents)`, `sim.world.http("host:port", |req| Reply::ok(body))`
   but gives no parameter types. It does not say whether `contents` or
   `body` may be `String`/`Vec<u8>` or whether the host may be a `String`,
   and the `req` type and its accessors are unspecified. Guess:
   `impl AsRef<str>` for hosts and `impl Into<Vec<u8>>` for bodies.
6. **`result` in `ensures` when the return type is not `Copy`.** §10.1:
   "In `ensures`, `result` is the return value". The only example matching
   on it (§15.3) has a `Copy` result. Guess: `result` is a place, so I
   wrote `match &result { .. }` for `Result<Vec<..>>` and `Option<Vec<..>>`.
7. **Panic message of a contract violation.** §9 lists "a violated contract"
   among panics but gives no message format, so
   `#[should_panic(expected = "..")]` cannot target it. I used bare
   `#[should_panic]`.

## 1. `wc`

- §6.3: "`root.fs` accepts absolute paths and resolves relative ones against
  the starting working directory". The reference does not say whether
  `root.fs.read_only()` (§6.1 attenuation) keeps that resolution. Guess: yes.
- §6.1 `Env::args() -> impl Iterator<Item = String>`. Silent on non-UTF-8
  arguments (Rust's `args()` panics; `args_os` is not mentioned).
- §6.2: "Tests cannot call `main`, so put the logic in functions that take
  capabilities." The exit code and stdout of a binary cannot be asserted in
  a test. `sim.records()` (§6.8) returns "sink records (`node`, `level`,
  `message`)". It is unclear whether `println!` output is a record. Only
  `count_file`/`Counts` are tested; the exit-code path is untested.
- §6.8: "Errors have the host's `io::ErrorKind`s". Error *messages*
  (`{source}` in `wc: nope.txt: ...`) are unspecified, so the test checks
  `kind()` only.
- Guess (task, not reference): the `total` line is printed always, even for
  one file; usage error exits with `ExitCode::from(2u8)` (§6.2 shows that
  constructor).

## 2. `csv_line`

- §3.1 gives the exact signature
  `fn split_csv(line: &str) -> Result<Vec<Cow<str>>, CsvError>   // Cow borrows line`
  and I used it. The private helper returning `(Cow<str>, usize)` relies on
  the same elision.
- §6.6: "`#[pure]` on a function makes losing authority-freedom a compile
  error". It does not say how `#[pure]` orders with doc comments and
  contract clauses. Guess: an ordinary attribute before `pub fn`.
- §10.2: "`ferox test` also calls each authority-free function that has
  `ensures` on generated inputs". The reference does not say how a `&str`
  parameter is generated. Guess: generate a `String` and borrow it.
- §2.1 shows a `const` table with `for (input, want) in CASES`. §3.1 says
  "In `const` and static types, elided origins are `'static`:
  `const CASES: [(&str, &[&str]); 2]`". I relied on inner slices of
  different lengths coercing in a `const` array (Rust does this).
- Guess (design): a `"` inside an unquoted field and text after a closing
  quote are errors (strict RFC 4180), not literal characters.

## 3. `lexer`

- §3.3 shows `next_token(&mut self) -> Option<Token from self.src> { .. }`
  with the body elided. I copy the field out (`let src = self.src;`) and
  slice `&src[a..b]`, assuming the slice keeps origin `self.src` and not the
  `&mut self` borrow. Rust allows this for a `&'a str` reached through
  `&mut self`, but the reference never shows a body.
- §3.2/§3.3 use `self.src` in an inherent signature and `Self.src` in the
  associated type (`type Item = Token from Self.src;`). I assumed they name
  the same origin so `next()` can return `self.next_token()`.
- `Option<Result<Token from self.src, LexError>>`: §3.2 says "Elsewhere
  `from` takes one origin, and a comma ends it". It is unclear whether
  `from` is allowed nested two type arguments deep. I assumed yes.
- §3.3's example `Token` is an enum with "three origins, all bound together
  below". I used a struct with one `text: &str` field (plus `kind`,
  `offset`). The reference does not say whether that is more idiomatic.
- §2.2: "`#[error("..")]` on the type or each variant". I put it on a
  *struct* (`LexError`), assuming "the type" includes structs.
- Guess (design): an unexpected character yields `Err` once and is skipped,
  so iteration continues.

## 4. `bsearch`

- §10.1: "repeated clauses are conjoined in order" appears under the bullet
  "One expression per clause". I assumed it covers repeated `invariant`
  clauses on one `while` loop too (the only loop example has one invariant).
- §10.1: "A quantifier body extends as far right as possible". The
  reference does not say where it stops inside a `match` arm in `ensures`
  (at the arm's `,`?). I avoided the question: the `None` arm uses
  `!xs.contains(target)` and `lower_bound` uses separate `ensures`.
- §10.2: "`ferox test` also calls each authority-free function that has
  `ensures` on generated inputs that satisfy its `requires`". For a generic
  `fn lower_bound<T: Ord>`, the reference does not say which `T` is
  instantiated. With `requires is_sorted(xs)`, random vectors seldom
  qualify. Would this automatic run fail as `vacuous` ("A run with fewer
  than 100 accepted cases fails as vacuous")? If so, every sorted-input
  contract breaks `ferox test` unless an `Arbitrary` exists. This is a
  likely real gap.
- Quantified loop invariants make each iteration O(n) in debug builds. §15.3
  mentions only that "each call checks the quantified `ensures` in linear
  time". Silent on invariant cost.

## 5. `fetch_all`

- "Network drops half the packets": §6.8 offers only
  `sim.faults.net_drop(p)`, where "an operation is delayed and fails with
  `NetError::Timeout` at its deadline (provisional)". That is per
  operation, not per packet, and the type of `p` is unstated. Guess:
  `net_drop(0.5)`.
- §7.1: with `limit(n)`, "`spawn` waits for a free slot (provisional; the
  wait is not a checkpoint)". The reference does not say what happens to a
  spawn still waiting when the scope deadline passes. Guess: it eventually
  spawns, and the child's `net.get` returns a cancellation error at once.
  My second test depends on this.
- §7.2: "When a scope deadline and an operation deadline coincide, the
  operation sees the cancellation". So a deadline cut-off shows up as
  `NetError::Cancelled`, not `Timeout`. I normalized
  `e.is_cancelled()` to `NetError::Timeout`, as §15.2 does with
  `Outcome::TimedOut`.
- §6.1 `only(hosts)` and §6.2 `only(["db.internal:5432"])`: it is unclear
  whether entries must be `host:port`, or whether a bare host matches any
  port. Guess: `host:port` built from `host_str()` and
  `port_or_known_default()`.
- §6.8: "capability operations take small seed-chosen latencies". So
  `clock.now()` itself may advance time, and `sim.elapsed()` can exceed the
  10 s budget slightly. I asserted `<= BUDGET + 1s`; the size of these
  latencies is unspecified.
- §6.7 does not say whether `NetError` implements `PartialEq` (it holds
  `io::Error`), so tests use `matches!`.
- Guess: `Duration::from_secs` is usable in a `const` (Rust `const fn`; §2.3
  only constrains user `const fn`).

## 6. `bank`

- §10.1: clauses "may call any function or trait method whose arguments are
  `Pure` ..., never a capability or `Shared`". So
  `requires plan.iter().all(|&(f, t, _)| f < accounts.len() && t < accounts.len())`
  on `run_transfers(accounts: &[Shared<Account>], ..)` appears to be
  forbidden, even though `len()` never touches a cell. Guess: forbidden; I
  documented the precondition in prose instead.
- Concurrent property test: see cross-cutting 3 and 4. I wrote
  `#[test] fn total_is_conserved(seed: u64, balances: Vec<u32>, plan: ..)`
  with `Root::sim(seed)` and `sim.tasks.scope`, which assumes a plain test
  can drive simulated tasks.
- §7.1 `scope` requires `E: From<TaskError> + Send`. So the domain error
  needs a `Task(#[from] TaskError)` variant just to use `?` on `join` (the
  reference says so implicitly via `FetchError` in §7.1).
- §7.4 lists `LockError` derives "(provisional)" but gives none for
  `TaskError`, so `BankError` derives only `Debug, Error` and tests use
  `matches!`.
- §7.4 example traps on `b.balance += amount` ("traps on overflow: keep
  totals bounded"), and a trap inside `f` poisons both cells. I used
  `checked_add` plus an `Overflow` variant so a transfer never poisons.
- `with_all` is a "(provisional name)". The reference does not say whether
  it accepts an empty slice.

## 7. `bfs`

- §3.6 lists `Arena` methods but no `Default` impl, so `Graph::new()` is
  written by hand. `contains` is provisional, so I used
  `get(id).is_some()`.
- §6.6 lists what is `Pure` ("collections, `Box`, `Rc`, ...") but does not
  name `Arena<T>`. The `ensures` on `shortest_path` calls `self.has_edge`
  and needs `&Graph` to be `Pure`. Guess: `Arena<T>` is `Pure` when `T` is.
- §10.1 says Rust's `(0..n).all(|i| ..)` works in clauses. I assumed a
  closure capturing `self` (`path.windows(2).all(|w| self.has_edge(..))`)
  is also fine.
- §3.6: `Id<T>` "is also `Ord` and `Debug` (provisional)". The tests'
  `assert_eq!` on `Option<Vec<Id<Node>>>` needs `Debug`.

## 8. `kv_log`

- **Contradiction.** §6.8: `node.start(entry)` runs
  "`entry: FnOnce(NodeRoot) -> Result<T, E>`", yet `restart` "reruns the
  entry over the surviving disk". A `FnOnce` cannot be rerun. Guess: pass
  a `fn` item (`fn writer(r: NodeRoot) -> io::Result<()>`), which is
  `Fn + Copy`.
- §6.2: "`Root` can be named only in `main`'s signature". The reference
  does not say whether `NodeRoot` has the same restriction. Guess: a named
  `fn` may take `NodeRoot`.
- §6.8 does not give the return value of `sim.faults.restart(&node)`, or
  whether the original `NodeTask` (or `run_until`) follows the restarted
  run. I used `run_for`. Crashing a node whose entry already returned is
  also unspecified. Guess: it is a harmless power cut.
- §6.8 does not say whether dropping the `NodeTask` returned by
  `node.start` stops the node. The reference's own example discards it, and
  so do I.
- **Torn tail + append.** The §6.3 `Log` example ignores a torn last line
  when reading but keeps appending after it. The next acknowledged record
  is then glued onto the torn bytes, and a reader that skips bad lines would
  lose it. The reference does not discuss this. I truncate with
  `file.set_len` (§6.3: "(provisional)") and `sync()` on open. It is also
  silent on whether `set_len` works on a file opened with `append(true)`.
- §6.3: `read_to_end` on a file opened `read(true).append(true)`. Guess: the
  read position starts at 0 (Rust/POSIX behaviour).
- §2.2: the JSON shape of an `enum` under `Encode` (externally tagged?) is
  unspecified. Tests build log bytes with `json::encode` rather than by
  hand.
- §6.3 `write_synced(&self, p, data)` and `write(&self, p, data: impl AsRef<[u8]>)`.
  The first omits its parameter type. Guess: the same as `write`.
- Guess: the crash test logs `info!("acked {i}")` after each synced `set`
  and reads them back with `sim.records()` (pattern from §6.8). It also
  assumes records from a crashed run survive in `sim.records()`.

## 9. `word_freq`

- §8: "`par_map`, `par_for`, and `par_sort` (stable) are inherent slice
  methods" but has no signatures. Guess: `fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>`.
- §6.4: `HashMap` iteration is "deterministic (but unspecified)". The merge
  goes into a `BTreeMap`, so order does not matter. Ties in `top` are broken
  by word explicitly.
- §8: "`par_reduce` is v1 (Appendix A)". The merge is a sequential fold, as
  §8 advises.
- File reading stays sequential, because `par_map` closures must be `Pure`
  (§8, FX0610/FX0611). The reference gives no pattern for parallel *I/O*
  over many files (a `tasks.limit(n)` scope would be the guess).

## 10. `app_errors`

- "Two library errors": the reference has no multi-package example in a
  single file. I modelled the libraries as inline `mod config { }` and
  `mod client { }` (§1: "inline `mod name { }`").
- §2.2: "`#[from]` on a single-field variant generates `From` and, as in
  thiserror, also marks the field as the `source` (provisional)". Because
  of that, display messages omit `{0}` so the printed source chain does not
  repeat itself. The reference shows both styles (`"bad config: {0}"` with
  `#[from]` in §2.2) without saying which is idiomatic.
- §6.2: after `Err(e)` from `main`, "the program prints `Error: {e:?}`"; "to
  print `Display` and the `source()` chain, return `ExitCode` and log the
  error yourself". I followed this. Printing the chain relies on the
  `Error` trait being in scope (cross-cutting 2).
- §6.7 lists `NetError::Denied` "(a host outside `only`)" without its
  shape. Guess: a unit variant, like `Timeout`.
- §2.2 `#[codec(default)]`: the reference does not say whether an
  `Option<T>` field is optional without it. I added it.
- §6.2: "Attenuate in `main` and hand each subsystem only what it needs".
  The `DataDir` error has to be built in `main` (`fs.dir` returns
  `io::Result`), so `main` mixes attenuation and error mapping. I added no
  `#[from] io::Error`, because two variants wrap `io::Error`.
- Guess: chaining attenuations (`root.net.only([..]).no_bind()`,
  `root.clock.monotonic_only()`) is allowed; §6.1 says attenuation "returns
  a narrower capability of the same type".
