# E1 learnability trial: notes

Source used: `design/reference.md` only. Programs are in `programs/*.fx`.
Section numbers (§) refer to that reference.

## Cross-cutting

1. **One file per program vs. package layout.** §12.1 defines `src/main.fx` /
   `src/lib.fx` plus `ferox.toml` (§12.4). The task wants one file per program,
   so each file starts with a comment naming the path it would have. The
   library-shaped tasks (csv_line, lexer, bsearch, bank, bfs) have tests but no
   `main`.
2. **Import paths the reference never gives.** §12.3 lists
   `std::fs::{Fs, Mode}`, `std::net::{Net, NetError, Url}`,
   `std::time::{Clock, Duration, Instant}`, `std::task::{Tasks, Scope, Task, TaskError}`,
   `std::sync::{Shared, mpsc}`. It never gives paths for `Poisoned` (§7.3),
   `SimRoot` (§6.8), `Faults` (§6.8), `ExitCode` (§6.2, §6.9), `Cow` (§1),
   `OpenOptions` or `File`. I guessed `std::sync::Poisoned`, `std::sim::{SimRoot, Faults}`
   (by analogy with `use std::sim::Reply;` in §15.2), `std::process::ExitCode`,
   `std::borrow::Cow`, and `std::fs::{File, OpenOptions}`. Unused imports are
   errors (§12.2, FX0913), so a wrong guess fails the build.
3. **`Send`/`Sync` of capabilities and `Shared`.** §7.1 says "Values moved or
   borrowed into `spawn` must be `Send`/`Sync` as in Rust". §15.2 borrows `net`
   into `spawn`, which implies `Net: Sync`. The reference never states this for
   `Clock`, `Shared<T>` or `Fs`, and I assumed all of them are `Sync`.
4. **Property-test inputs.** §6.8 says "A `#[test]` function may take
   parameters; it becomes a property test whose inputs are generated (and
   shrunk). A `seed: u64` parameter runs the test under many seeds." It does
   not say:
   - which types can be generated (I used `String`, `usize`, `Vec<String>`,
     `Vec<i32>` and `Vec<(u8, u8, u16)>`);
   - whether `seed` can be combined with other generated parameters (bank and
     word_freq do this);
   - whether `seed` is special-cased by its name or its type;
   - whether a test can filter its inputs. §10.3 says `requires` filters
     generation for functions under test, but nothing covers `#[test]`
     functions, so I used early returns (csv_line).
5. **Capability method receivers.** The reference does not say whether
   `Rand::gen_range`, `Stdin::read_to_end` and similar take `&self` or
   `&mut self`. Rust's versions take `&mut self`, while §6.1 says "Pass them by
   reference: `fs: &Fs`". I avoided `Rand` and `Stdin` entirely: kv_log derives
   keys from loop counters, and wc does not read stdin.

## 1. wc (`wc.fx`)

- **`read_only()` has no signature.** The §6.1 table gives
  "`dir(path, Mode::Read | Mode::ReadWrite) -> io::Result<Fs>`; `read_only()`".
  I guessed `fn read_only(&self) -> Fs` (no `Result`).
- **Attenuated `Fs` with absolute paths is unspecified.** §6.3 says "Paths given
  to an attenuated `Fs` are relative to its directory; `..` cannot escape it."
  An absolute argument such as `wc /etc/hosts` could be reinterpreted under the
  directory, rejected, or allowed. §15.1 attenuates to the cwd with
  `root.fs.dir(&cwd, Mode::ReadWrite)`. Copying that would break absolute paths
  in wc, so I attenuated by mode instead (`read_only()`).
- **`ExitCode`.** §6.2 says "`main` returns `()`, `Result<(), E: Debug>`, or
  `ExitCode`, as in Rust", and §6.9 uses `ExitCode::FAILURE`. The path is not
  given. I also assumed `ExitCode::from(2)` exists, as in Rust.
- **Test messages.** The sim's text for a `NotFound` error is unknown, so the
  test checks only the `wc: missing.txt: ` prefix and `ErrorKind::NotFound`.
- **Guesses.** `fs.open(path)` returns a `File` that implements `std::io::Read`
  (§1 says the std::io traits behave as in Rust; I assumed `File` implements
  them). I assumed `#[pure]` on a binary's helper is fine (§6.6, provisional).
- **Differences from Rust.**
  - In Rust I would print directly in the loop and call
    `std::process::exit(1)`. Here `main` returns `ExitCode` (§6.9 table).
  - I collect stdout and stderr into a `Report` so the sim test can check the
    failure flag without `main`, which cannot be called from tests because
    `Root` cannot be passed (§6.2). As a side effect, stdout and stderr are no
    longer interleaved.
  - `chunk.len() as u64` stays, because §4.2 allows `usize as u64`.

## 2. csv_line (`csv_line.fx`)

- **Origins on std borrowing types.** §3.3 says "Using a borrowing type never
  needs parameters: write `Lexer`". I applied that to `Cow<'a, str>` by writing
  `Cow<str>`, and assumed Rust elision (§3.1) ties
  `Result<Vec<Cow<str>>, CsvError>` to `line`. §3.1 and §3.2 only show `&str`
  returns and user structs, never std types that carry a lifetime parameter.
- **`'static` in consts.** `const CASES: [(&str, &[&str]); 10]` relies on
  Rust's implicit `'static` in `const` types. §3.4 says "`'static` is kept:
  `&'static str`, string literals, `const` data" but has no example of an
  elided `&str` inside a const type.
- **Test tables.** I followed §2.1 ("test tables → a `const` array of cases
  looped over in one `#[test]`").
- **`#[pure]`.** §6.6 marks it provisional. I put it on the public function
  because §6.6 says it "is useful on public APIs".
- **Differences from Rust.**
  - Rust: `fn split_csv_line<'a>(line: &'a str) -> Result<Vec<Cow<'a, str>>, CsvError>`,
    or `Cow<'_, str>` under the `elided_lifetimes_in_paths` lint. §3.2 accepts
    `'_` (provisional), but §3.3 says the formatter deletes `<'_>`, so I wrote
    `Cow<str>`.
  - Rust tests would use `rstest` or a `macro_rules!` table plus `proptest!`.
    Here they are a const array and a parameterised `#[test]`.

## 3. lexer (`lexer.fx`)

- **Enums with borrowed fields are not covered.** §3.3 covers structs only:
  "Each field whose type contains a borrow gets its own origin, named by the
  field." `enum Token { Ident(&str), Int(&str), Punct(&str) }` has tuple-variant
  fields with no names. I assumed `Token from self.src` binds every origin of
  `Token` to `self.src`, by analogy with §3.3's "`-> Pair from a, b` lets every
  origin of `Pair` borrow from either."
- **`from` in a generic argument that is not last.** I wrote
  `type Item = Result<Token from Self.src, LexError>;`. §3.2 says "inside
  generic arguments, parenthesize: `Vec<&str from (a, b)>`" when there are
  several origins, which implies a comma ends the clause. I could not find an
  example with another generic argument after the `from`, so
  `Result<Token from Self.src, LexError>` might also parse as two origins.
- **A method that returns what `self` borrows.** I wrote
  `fn text(&self) -> &str from *self`. This follows from §3.2: "`from p` means
  'borrows from what the value at place `p` borrows'. If `p` is a reference
  parameter, that is its referent." So `from self` would borrow the `Token`
  itself, and `from *self` borrows the source. I inferred this from the
  `from *words` row; there is no example of it on a method. Rust:
  `fn text(&self) -> &'a str` inside `impl<'a> Token<'a>`.
- **`fn next` signature.** I copied §3.3 and spelled out
  `-> Option<Result<Token from self.src, LexError>>`. Whether
  `-> Option<Self::Item>`, the usual Rust spelling, is accepted is not stated.
- **Differences from Rust.** `struct Lexer<'a>`, `enum Token<'a>` and
  `impl<'a> Iterator for Lexer<'a>` became origin-free declarations. Rust
  elision on `fn lex(src: &str) -> Result<Vec<Token>, _>` would need
  `Token<'_>`. The `tokens_outlive_the_lexer` test is §3.3's `two_words`
  pattern.

## 4. bsearch (`bsearch.fx`)

- **`==` in specs vs. `Ord` (a conflict for generic code).** §10.2 says "`==`
  is model equality: structural, for any type, no `PartialEq` needed." §15.3
  writes `xs[i] == target` for `i64`, where the two notions coincide. For a
  generic `T: Ord`, `cmp == Equal` does not imply structural equality (think
  case-insensitive keys), so a `binary_search` postcondition written with `==`
  would be false. I wrote every spec with `xs[i].cmp(target) == Ordering::...`.
  The reference does not say what `<` and `<=` mean in specs for a generic `T`
  (`PartialOrd` calls, or a model order?).
- **Trait methods in specs.** §10.2 says "Specs call only `spec fn`s ..., the
  built-in spec types, and functions marked `#[spec]` ... Std marks its pure
  accessors (`len`, `is_empty`, indexing, `get`, `contains`, and similar)."
  A user's `Ord::cmp` impl is not `#[spec]`, so it is unclear whether
  `xs[i].cmp(&xs[j])` is legal in a spec. I assumed yes, via the trait's laws.
- **Which laws `Ord` carries.** §5.3 says only "`PartialOrd`/`Ord` (order
  consistent with `Eq`)". It does not list transitivity, antisymmetry or
  totality. My `proof { assert forall ... }` hints in the `Less` and `Greater`
  branches depend on transitivity.
- **Panics in `prove` mode.** §10.3 says "every panic site (overflow, indexing,
  `unwrap`, `assert!`) is an obligation". Can a call to a generic `T::cmp`
  panic? §10.3 also says "Function bodies are opaque to callers: a caller sees
  only the contract", but `Ord::cmp` has no stated contract.
- **Authority-freedom of generic code (a possible conflict).** §6.6 says
  "Generic functions are authority-free per instantiation", while §5.1 says
  "Generics are definition-checked: an instantiation never produces a type
  error." If spec functions must be authority-free (§10.2), a definition-checked
  generic spec might need `T: Ord + Pure`. I left the bound as `T: Ord`.
- **Unconfirmed syntax.**
  - Generic `spec fn`s (`spec fn sorted<T: Ord>`) never appear in the
    reference.
  - `assert forall |j| ... ;` inside `proof { }` combines §10.2's `assert e;`
    with a quantifier.
  - Quantifier precedence is not given, so I parenthesised
    `result == (exists |i| ...)`.
- **Differences from Rust.** The API matches `slice::binary_search`
  (`Result<usize, usize>`). Rust has no contracts; I would have written the same
  loop and relied on `proptest` alone.

## 5. fetch_all (`fetch_all.fx`)

- **No way to cap concurrency.** §7.1 offers `spawn`, `join`, `set_deadline`
  and `cancel`: no semaphore, no bounded scope, no `buffer_unordered`. I built a
  pool of 4 workers that claim indices from a `Shared<usize>` with
  `lock (next) |n| { ... }?`. A bounded `sync_channel` work queue would not
  work, because Rust's `mpsc::Receiver` is single-consumer and §7.3 says
  "Channels are Rust's `std::sync::mpsc`".
- **What a deadline does to `scope`'s return value.** §7.1 says "The first
  failure cancels the scope ... and `scope` returns that error". §7.2 says
  cancellation also comes from "its deadline passing". It never says whether a
  deadline cancellation makes `scope` return `Err` when every child returned
  `Ok`. §15.2's test `.unwrap()`s the scope after a timeout, so I assumed it
  returns `Ok`.
- **Locking after cancellation.** §7.3 says "Acquisition is not a cancellation
  point", so I assumed `lock` still works after the deadline. My workers keep
  claiming indices, each `net.get` fails at once with a cancelled error, and
  every URL still gets a result.
- **`Poisoned`.** It is needed for `?` on `lock`, so I added
  `#[from] Poisoned` to `FetchError`. That assumes `Poisoned: Error`, which
  `#[from]` needs under thiserror semantics; the reference does not say.
  §7.1's `fetch_both` comment ("FetchError: #[from] TaskError, #[from] NetError")
  was the model.
- **Retry policy.** Only `NetError::Timeout`, `NetError::Cancelled` and
  `is_cancelled()` appear (§7.2, §15.2). With no way to classify retryable
  errors, I retry everything else up to 3 times. `clock.sleep` returns
  `Result<(), Cancelled>` (§6.1), and I mapped that to `NetError::Cancelled`,
  assuming it is a unit variant.
- **Simulation semantics.**
  - What `Faults { net_drop: 0.5, .. }` does to an HTTP `get` (retransmit and
    get slower, or error) is not stated. The half-drop test can therefore
    assert only shape (one result per URL, in order), the deadline, and that
    bodies arrive intact. It cannot assert how many requests succeed.
  - No sim hook reports how many requests are in flight. I checked the limit
    through virtual time instead: 8 one-second replies through 4 slots take at
    least 2 s. This assumes `Reply::ok(..).after(d)` serves requests
    concurrently and that sim network latency is negligible.
- **No attenuation in `main`.** `only(["host:port", ..])` (§6.1) takes a
  literal array, and both the `Url` API (host, port) and whether `only` accepts
  a `Vec<String>` are unspecified. So `main` passes `root.net` unattenuated.
- **Differences from Rust.** Rust would use tokio:
  `stream::iter(urls).map(fetch).buffered(4)` inside
  `tokio::time::timeout(10s, ..)`, or a `Semaphore`. Here it is blocking code in
  a scope with `set_deadline`, as §7.5 suggests.

## 6. bank (`bank.fx`)

- **Same-account detection.** §7.3 says "Passing two handles to the same cell
  panics (aliasing is checked at run time)" but offers no `Shared::ptr_eq`. To
  refuse rather than panic, I keep the cells in a `Bank` and compare indices.
- **No consistent multi-cell snapshot.** `lock (c1, c2, ..)` has a fixed
  arity, so locking a whole `Vec<Shared<_>>` at once is impossible. `total()`
  locks one cell at a time and is exact only when no transfer is running; the
  test calls it after the scope joins.
- **`lock` forms.**
  - The single-cell form `lock (cell) |a| { a.balance }?` appears only in the
    §7.5 table.
  - Whether the cell list may contain arbitrary expressions (`lock (self.cell(id)?) ...`)
    is not stated, so I bind locals first.
  - §7.3 says "`return` and `?` leave the body, not the function", so
    `return Err(BankError::Insufficient{..})` inside the body is correct.
- **Interleaving.** §7.3 says "Under simulation the interleaving of senders is
  chosen by the seed". That covers channels; I assumed the seed also orders
  `lock` acquisition and task scheduling. I spawn tasks in a loop and drop the
  `Task` handles, which §7.1 permits.
- **Derives.** I did not derive `PartialEq` on `BankError` because it would
  need `Poisoned: PartialEq`, which is unknown. Tests use `matches!`.
- **Differences from Rust.**
  - Rust uses `Arc<Mutex<Account>>` and must order the two locks by id to avoid
    deadlock. §7.3's `lock (a, b)` does that itself.
  - A Rust property test would be `proptest!` plus `std::thread::scope`, with
    nondeterministic, unreplayable interleavings. Here the seed makes each
    interleaving replayable.
  - I used `u64` balances with an overdraft check before the subtraction, since
    a subtraction underflow traps (§4.1). `Bank::new` checks the total once, so
    a credit can never overflow.

## 7. bfs (`bfs.fx`)

- **`Handle<T>` traits are not stated.** §3.7 says "handles are `Copy` values,
  not borrows". BFS needs a visited set and a parent map, so I assumed
  `Handle<T>: Eq + Hash + Debug`. Tests also compare `Option<Vec<Handle<Node>>>`
  with `assert_eq!`.
- **Arena API.** The names are provisional (§3.7: "`new`, `insert(T) -> Handle<T>`,
  `get(h) -> Option<&T>`, `get_mut`, `remove(h) -> Option<T>`, `Index` (panics
  on a stale handle), `iter`, `len`"). The item type of `iter` is not given,
  there is no `contains`, and handles expose no dense index. I used
  `get(h).is_some()`.
- **Guess.** I chose to have BFS skip edges to removed nodes, relying on
  `get(next)` returning `None` for stale handles (§3.7: "stale handle: None,
  never a dangling read").
- **Differences from Rust.** In Rust I would store nodes in a `Vec<Node>` with
  `usize` ids (or petgraph's `NodeIndex`) and use a `Vec<Option<usize>>` parent
  array, which is cheaper. Arena handles have no index, so I used `HashMap` and
  `HashSet`. Their iteration order is deterministic per §6.9, though BFS does
  not depend on it.

## 8. kv_log (`kv_log.fx`): the hardest

- **No append mode on `Fs` (the main gap).** §6.1 lists `read`,
  `read_to_string`, `write`, `open`, `create`, `create_dir_all`, `read_dir`,
  `metadata`, `exists`, `rename` and `remove_file`, "as `std::fs::*`".
  - Rust's `OpenOptions::new().append(true).open(p)` would be ambient
    (FX0501), and no capability version is given.
  - `create` truncates, and `write` replaces the whole file, which is not an
    append-only log and is not crash-safe.
  - I invented `dir.open_with(LOG_FILE, OpenOptions::new().create(true).append(true))`.
- **Durability calls (possibly contradictory).** I assumed `File::sync_data`,
  `sync_all` and `set_len` work as in Rust, since `File` is an "ordinary type"
  (§6.1). But §6.7's example
  `pub fn sync(_fs: &Fs, file: &File) -> io::Result<()>  // takes the authority it uses`
  suggests fsync needs `Fs` authority. Whether `file.sync_all()` counts as
  exercising `Fs` is unclear. A durable directory entry after creating a file
  (directory fsync) cannot be expressed at all.
- **No crash or restart in the simulator.** The `sim.world` table (§6.8)
  offers `file`, `args`, `env_var`, `stdin`, `http` and `faults`. `Faults`
  shows `net_drop` and `disk_error` only. There is:
  - no crash, power-loss or restart operation;
  - no torn-write knob;
  - no description of whether the in-memory filesystem distinguishes synced
    from unsynced bytes.

  I invented `sim.world.crash()` ("unsynced data is gone, open files are dead")
  and assumed `Fs` capabilities stay valid across it. The test also uses
  `disk_error` faults, so some writes are left "in doubt"; the checker accepts
  either outcome for that single write.
- **Other guesses.**
  - `SimRoot` as the parameter type of a test helper. §6.8 names the type but
    not its path.
  - `Op::Put { key: &str, value: &str }` hits the same enum-origin gap as the
    lexer.
  - `dir.read` returns `ErrorKind::NotFound` for a missing log.
  - `root.fs.dir(..)` requires the directory to exist, so I call
    `create_dir_all` first.
- **Differences from Rust.**
  - The record format and recovery logic would be identical in Rust.
  - Crash testing in Rust needs a trait-abstracted filesystem with a
    hand-written fault injector, or a process-kill harness. In Ferox it is a
    plain `#[test]` with `seed`, if a crash API existed.
  - The checksum uses `wrapping_mul` because of §4.1 ("Hash functions,
    checksums, and PRNGs must use these"). In Rust, release builds would wrap
    silently.

## 9. word_freq (`word_freq.fx`)

- **Paths and method kinds.**
  - `Combine` and `Monoid` paths come from the §5.3 comment
    ("`// std::par: what par_reduce needs`"), so I wrote
    `use std::par::{Combine, Monoid};`.
  - §8 says `par_map` and `par_reduce` are "std functions and slice methods". I
    assumed inherent methods, because §8's example imports only `join` and
    `Sum`.
- **`par_reduce` on an empty slice.** Not specified. My test assumes it returns
  `M::empty()`.
- **Monoid impl and law testing.**
  - `impl Monoid for Counts {}` is an empty impl, because the laws are bodiless
    (§5.3). §5.3 says "A hand-written impl is property-tested under `test`
    mode", but not how `Counts` values are generated for that.
  - Laws compare with model `==` (§10.2). That matches `PartialEq` here, but
    for a type with a custom `PartialEq` the laws would test the wrong
    equality.
- **Design friction.** `Combine::combine(&self, other: &Self) -> Self` (§5.3)
  forces a full map clone at every merge step. A by-value `combine(self, other)`
  would let the larger map absorb the smaller.
- **Differences from Rust.**
  - Rust: rayon `docs.par_iter().map(count).reduce(HashMap::new, merge)`.
    Rayon's grouping is nondeterministic, though a commutative merge hides it.
  - Here the grouping is canonical (§8). I still used `BTreeMap` plus
    alphabetical tie-breaking so the printed top-N is stable.
  - File reading stays sequential before `par_map`, because §8 says closures
    "must be authority-free (`FX0610`)".

## 10. app_errors (`app_errors.fx`)

- **Does `#[from]` imply `#[source]`?** §2.2 says "`thiserror` semantics, built
  in ... `#[from]` on a single-field variant generates `From`; `#[source]` (or a
  field named `source`) sets `Error::source`". It does not say outright that
  `#[from]` also sets `source`, as thiserror does. My error-chain logging in
  `report` and the test `err.source().is_some()` depend on it.
- **Attenuated `Net` errors.** The error an attenuated `Net` returns for a
  disallowed host is unnamed (`NetError::Denied`?), and so is what
  `Reply::status(503)` becomes. The tests assert only `is_err()` and
  `AppError::Net(_)`.
- **Inconsistent-looking return types.** `root.net.only([..])` returns `Net`
  directly (§6.2 example), while `root.fs.dir(..)` returns `io::Result<Fs>`.
  This is documented in §6.2 and §16 item 9, but easy to get wrong.
- **What `..` does in an attenuated `Fs`.** §6.3 says "`..` cannot escape it"
  without saying whether that is an error or clamping. The test asserts only
  that reading `../../../etc/passwd` fails.
- **`main`'s shape.** I chose `fn main(root: Root) -> ExitCode` so that `main`
  can log the `Display` chain through `error!` (§6.5: the default sink is
  `info` and above to stderr). `Result<(), AppError>` would print only
  `Debug`. Attenuation happens in `main` (§6.3: "Do it in `main`"), and only
  fields are borrowed (`&root.clock`), never `Root` (§6.2, FX0503).
- **Unnamed `Url::parse` error.** Its type is not named, so I mapped it to
  `AppError::BadUrl(&'static str)`.
- **Differences from Rust.**
  - Rust would use thiserror for `AppError` and maybe anyhow with `.context()`.
    There, `fs::write` and `reqwest::get` can touch any path or host. Closest
    Rust analogue for attenuation: cap-std.
  - The test idiom `Err(io::Error::from(..))?` shows `?` driving the
    `#[from]` conversion. It is legal Rust but clippy flags it (`try_err`).

## Top gaps, ranked

1. **Persistence.** No append or open-options capability on `Fs`, and no
   simulator crash, power-loss or durability model (§6.1, §6.8). kv_log cannot
   be written without inventing APIs.
2. **Contracts on generic code.** It is unclear what `==`, `<` and trait calls
   mean in specs for a generic `T: Ord`, which `Ord` laws exist, and whether
   generic trait calls can panic under `prove` (§5.3, §10.2, §10.3).
3. **Origins beyond structs.** Not covered: enum variants (especially tuple
   variants), `from` in a generic argument followed by another argument, and
   methods returning what `*self` borrows (§3.2, §3.3).
4. **Missing paths and signatures.**
   - Paths: `Poisoned`, `SimRoot`, `Faults`, `ExitCode`, `Cow`, `OpenOptions`.
   - Signatures: `read_only()`.
   - Traits: those of `Handle<T>`.
   - Receivers: `&self` vs. `&mut self` on `Rand` and `Stdin`.
5. **Concurrency building blocks.**
   - No bounded-concurrency primitive.
   - No sim observability of in-flight work.
   - `lock` has a fixed arity and there is no handle-identity check.
   - `Sync` for capabilities and `Shared` is never stated.
   - Whether a scope deadline makes `scope` return `Err` is unstated.
