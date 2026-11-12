# Tide for Rust Programmers

Status: the agent-facing reference for the **v0 core** of Tide, as the
tier table of `design/01-design.md` (draft 3.7, the decision record) defines
it. It summarizes 01 and the normative chapters `02-memory.md`,
`03-authority-concurrency.md`, and `04-contracts.md`. Syntax is provisional
until the E1 gate (D0). `reference-companion.md`, loaded on demand, holds
Rust-habit tables, example programs, readings where the documents
disagree, and the sections cited here as "companion N".
Budget: under 25,000 tokens (D14).

**(provisional)** marks a detail no normative document decides; this
reference picks the most Rust-like option consistent with the decisions.
Diagnostic codes (`FX0201`, ...) are placeholders.

## 0. How to use this document

You know Rust. Write Rust. A Tide program is a valid Rust 2024 crate: Tide's
checker admits it, then a pinned nightly rustc builds it against Tide's std
(D18). Each Rust habit that would mean something different fails with a
stable code and, usually, a machine-applicable fix. What you hit first: no
ambient I/O (capabilities passed down from `main`, 6), no `async` (scopes,
7), three lifetime forms gone (3.2), overflow traps in every build and
lossless-only `as` (4), `Shared<T>` instead of `Arc<Mutex<T>>` (7.4), and
only allowlisted library items (6.9).

## 1. What is unchanged

Everything not listed in sections 2–13 works as in Rust 2024: syntax,
ownership and borrowing, drop order and drop check, and traits and generics
within 5's limits. Std's traits, every `Iterator` adapter included, are
admitted (6.9). `static` is immutable only (6.4); `unsafe` needs a grant
(11).

**Standard types** keep their Rust API: scalars, `str`, `String`, slices,
arrays, tuples, `Option`, `Result`, `Vec`, `Box`, `Rc`, `Arc` (created with
`&Tasks`, minus count observers, 6.4), `Weak`, `Cell`, `RefCell`,
`VecDeque`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `BinaryHeap`,
`Cow`, `Path`, `PathBuf`, `OsStr`, `OsString`, `Duration`, `Instant`,
`SystemTime`, `std::{cmp, ops, mem, fmt, convert}`, and the `std::io` traits
and types (`Read`, `Write`, `Seek`, `BufRead`, `BufReader`, `BufWriter`,
`io::Error`, `io::ErrorKind`, `io::Result`). What changes is how you
*obtain* I/O objects (6) and the default hasher (6.4). Dropping a
`BufWriter` discards unflushed bytes; `Lines` ends after a cancellation
error (03 §1.1). Testing is Rust's
(`#[test]`, `#[should_panic(expected = "..")]`, `#[cfg(test)] mod tests`,
the `assert!` family); a test may take parameters (10.2).

## 2. Surface: `!` forms, derives, attributes

### 2.1 The fixed built-in `!` forms

Std supplies exactly these, as in Rust:
`print!`, `println!`, `eprint!`, `eprintln!` (no capability needed, 6.5;
a lint in libraries); `format!`, `write!`, `writeln!`, `vec!`, `matches!`;
`assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`; `panic!`, `todo!`
(also a typed hole, 13), `unimplemented!`, `unreachable!`; `dbg!` (an
error in release builds, `FX0507`); and the log sink (6.5) `error!`,
`warn!`, `info!`, `debug!`, `trace!`, with no `use` (`log::info!` and
`tracing::info!` are also accepted; provisional). Tide adds `invariant!`
(10.1) and `hole!` (13), not imported. The `unsafe` grant (11) admits
`asm!`, `global_asm!`, and `naked_asm!` (`FX0520` without it).

Any other `!` form is `FX0103`, including `debug_assert_eq!` (write
`debug_assert!(a == b)`), `concat!`, and crate macros; `include_str!`,
`env!`, and `option_env!` are `FX0520` (6.9), `thread_local!` and
`lazy_static!` `FX0502`; `macro_rules!` and procedural macros are
`FX0101`: use a generic function or trait, loop over a `const` array
(test tables), or commit a generator's `.rs` output.

### 2.2 Derives

The derive set is closed: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`,
`Clone`, `Copy`, `Debug`, `Default` (as Rust), `Error`, `Encode`, `Decode`.
They and the `Error` trait are in the prelude (12.3); remove
`use thiserror::Error;`, `use std::error::Error;`, and `use serde::..`.
`Serialize`/`Deserialize`/`thiserror::Error` are `FX0102` with safe fixes.

- **`Error`** has `thiserror` semantics: `#[error("..")]` on a struct, an
  enum, or each variant generates `Display`; `#[from]` on a single-field
  variant generates `From` and marks the field as the `source`; `#[source]`
  (or a field named `source`) sets `Error::source`; `#[error(transparent)]`
  forwards both. If you print the `source()` chain, leave the source out of
  each message, or it prints twice.
- **`Encode`/`Decode`** are compiler-generated serialization, JSON only in
  v0: `std::json::encode(&v) -> Vec<u8>`,
  `std::json::decode::<T>(&bytes) -> Result<T, json::Error>`. Keys:
  `#[codec(rename = "..")]`, `#[codec(rename_all = "camelCase")]`,
  `#[codec(default)]`, `#[codec(skip)]`, with serde's spellings and defaults
  (provisional): a missing `Option` field decodes as `None` without
  `#[codec(default)]`, and enums are externally tagged
  (`{"Set":{"key":"a"}}`, `"Clear"`).

Std error types (`io::Error`, `NetError`, `UrlError`, `TaskError`,
`LockError`, `Panicked`, `Cancelled`, `json::Error`) are
`Error + Send + Sync + Pure`; `io::Error::new` and `other` take only a `Pure`
error or a string.

### 2.3 `const fn`, attributes, `cfg`

- `const fn` as Rust, but deterministic and fuel-limited, with
  `Pure + Sync` parameters (6.6), none `&mut` (`FX0518`).
- Kept Rust attributes: `#[derive]`, `#[test]`, `#[should_panic]`, `#[cfg]`,
  `#[allow]`/`#[warn]`/`#[deny]`, `#[inline]`, `#[must_use]`, `#[repr]`,
  `#[non_exhaustive]`, `#[deprecated]`, `#[doc]`. `#[cfg]` accepts `test`,
  `debug_assertions`, `target_os`, `target_arch`, `unix`, `windows`, and
  `feature = ".."` (provisional beyond `test`).
- Tide attributes: `#[error]`, `#[from]`, `#[source]`, `#[codec]`,
  `#[tide::main]` (6.2), `#[pure]` (6.6), `#[sim_test]` (6.8),
  `#[requires]`, `#[ensures]`, `#[invariants]` (10.1), `#[protected]` and
  `#[trusted_pure]` (11): toolchain proc macros and their derives' helper
  attributes, not user macros.
- `#[tokio::main]`, `#[async_trait]`, `#[path]`, `#[macro_export]`, and
  `#[proc_macro*]` are errors with fixes.

## 3. Lifetimes

Tide has Rust's borrow checker and Rust 2024's lifetimes (syntax, elision,
outlives bounds, `+ use<..>`, implied bounds, object-lifetime defaults,
`impl Trait` capture), minus three forms (3.2).

### 3.1 Elision is Rust's

Rust's elision rules apply unchanged. A borrowing type in a signature elides
as `'_` (`Token<'_>`) or, as in Rust, the bare name (`Token`). In an impl
header write `impl Collector<'_, '_>`; the bare `impl Collector` is an
error, as in Rust. In `const` and `static` types, elided lifetimes are
`'static`: `const CASES: [(&str, &[&str]); 2] = ..;`.

### 3.2 Named lifetimes: where elision is not enough

| Need | Write |
| --- | --- |
| result borrows from one input only | `fn pick<'a>(a: &'a str, b: &str) -> &'a str` |
| a token that outlives the `&mut self` call | `fn next_word(&mut self) -> Option<&'a str>` in `impl<'a> Lexer<'a>` |
| what a borrowing value borrows, not `&self` | `fn text(&self) -> &'a str` in `impl<'a> Token<'a>` |
| an element of a slice of borrows | `fn longest<'a>(words: &[&'a str]) -> &'a str` |
| an associated type that borrows | `type Item = Result<Token<'a>, LexError>;` in `impl<'a> Iterator for Lexer<'a>` |
| a value with two independent borrows | `fn find<'n, 't>(n: &'n str, t: &'t str) -> Option<Match<'n, 't>>` |

Traps, all as in Rust:

- `&'a mut Vec<&'a str>` ties the mutable borrow to the data, and callers
  then fail to borrow-check; write `&'a mut Vec<&'b str>`.
- In a body, `let src = self.src;` copies the reference out, and
  `&src[a..b]` keeps lifetime `'a`. A plain elided `-> &str` on `&self`
  borrows the reference itself, not `'a`.
- A result can get a lifetime of its own:
  `fn pick<'r, 'a: 'r, 'b: 'r>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str`.
  Relations that well-formed signature types imply need no bound:
  `&'m BTreeMap<char, Vec<&'t str>>` lets a result be `&'m [&'t str]`.

Lifetime parameters on traits, explicit `for<'a>`, and generic associated
types are not in Tide (5.1). An elided output that cannot be resolved, or that
elision ties to the wrong input, is `FX0202`; its fix suggests the named
lifetime (often the impl's `'a`).

### 3.3 Borrowing structs and enums

As in Rust, fields from different sources get different lifetime names
(`struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }`),
and in `impl<'a> Lexer<'a>`, `Self` is `Lexer<'a>`, so `new` takes
`src: &'a str`, not `&str`. `Shared` is invariant, like `Cell`.

```rust
pub struct Lexer<'a> { src: &'a str, pos: usize }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> { Ident(&'a str), Int(&'a str), Punct(&'a str) }

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self { Lexer { src, pos: 0 } }
    pub fn next_token(&mut self) -> Option<Token<'a>> { .. }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> { self.next_token() }
}

fn two(text: &str) -> Option<(Token<'_>, Token<'_>)> {
    let mut lx = Lexer::new(text);
    let a = lx.next()?;
    let b = lx.next()?;                            // fine: `a` borrows text, not lx
    Some((a, b))
}
```

### 3.4 `'static` and `dyn`

- A type is *owned* when it borrows nothing except `'static` data
  (`T: 'static`). Work that outlives the current function requires it:
  `spawn_isolated` (7.1), `Box<dyn Trait>` contents in a signature or
  field, and children of a scope passed down unless they borrow only what
  the signature ties to the scope (7.1).
- `dyn Trait` has Rust's object-lifetime defaults, so in a signature or
  field `Box<dyn Trait>` means `+ 'static`; write `Box<dyn Trait + 'a>` for
  a borrowing object (`Box<dyn Fn(&Row) -> bool + 'a>` in
  `struct Filter<'a>`).

### 3.5 Closures and returned `impl Trait`

- A closure that captures a borrow is a borrowing value. A returned
  `impl Trait` captures every generic parameter in scope, lifetimes
  included, as in Rust 2024:
  `fn over(min: &u32) -> impl Fn(&Person) -> bool` needs no annotation;
  `+ use<'a>` captures less.
- A closure in a `'static` position (`Box<dyn Fn()>` in a signature or
  field, `spawn_isolated`) must capture no borrows: `move` owned values in
  (`clone()`, `to_string()`), or write `+ 'a` (`FX0203`).

### 3.6 What is impossible, and what to write instead

A struct cannot borrow from its own field (`FX0210`):
`struct Doc<'a> { text: String, words: Vec<&'a str> }` cannot be built with
`words` borrowing its own `text`.

| Need | Write |
| --- | --- |
| Parsed view of a buffer | Keep owner and view apart: `let text = fs.read_to_string(p)?; let doc = Doc::parse(&text);` |
| Store both together | Indices: `words: Vec<Range<usize>>` and `&self.text[self.words[i].clone()]` |
| Graphs, parent links, cycles | `Arena<T>` with `Id<T>` handles (below) |

```rust
use std::arena::{Arena, Id};

struct Node { name: String, edges: Vec<Id<Node>> }

let mut g: Arena<Node> = Arena::new();
let a = g.insert(Node { name: "a".into(), edges: vec![] });
let b = g.insert(Node { name: "b".into(), edges: vec![a] });
g[a].edges.push(b);               // a cycle: handles are values, not borrows
g.remove(b);
assert!(g.get(b).is_none());      // stale handle: None, never a dangling read
```

`Id<T>` (a `u32` index and generation) is `Copy`, `Eq`, `Hash`, `Pure`,
`Send`, `Sync` for every `T`, `'static` when `T` is (02 §10.9), and also
`Ord` and `Debug` (provisional).
`Arena` has `new`, `insert(T) -> Id<T>`, `get(id) -> Option<&T>`,
`get_mut`, `remove(id) -> Option<T>`,
`get_disjoint_mut([a, b]) -> Option<[&mut T; 2]>`, `len`, and
`Index`/`IndexMut` (panic on a stale id); `Default`, `contains(id)`, and
`iter()` over `(Id<T>, &T)` (provisional). It is `Pure` when `T` is.

## 4. Integers, floats, conversions

### 4.1 Overflow traps in every build

`+ - * / % << >>`, unary `-`, `abs`, and `pow` trap on overflow, division by
zero, `MIN / -1`, and over-wide shifts, in debug **and release**. A trap is a
panic (9) at the faulting operation. For modular or clamped arithmetic use
Rust's `wrapping_*`, `checked_*`, `saturating_*`, `overflowing_*`,
`Wrapping<T>`, `Saturating<T>`; hashes, checksums, and PRNGs must.

### 4.2 `as` is lossless only

Allowed: same-signed integer widening, unsigned to wider signed,
`bool` to integer, `u8` to `char`, `char` to
`u32` and wider, integer to float when exact (`u16 as f32`, `u32 as f64`),
`f32 as f64`, fieldless enum to integer holding every discriminant.
`usize`/`isize` count as 32 to 64 bits: `u32 as usize` and `usize as u64`,
not `u64 as usize` (provisional). Addresses (11) count as `usize`.
Lossy `as`: `FX0301`:

| Rust | Tide |
| --- | --- |
| `x as u8` (may not fit) | `u8::try_from(x)?`; keep low bits (same signedness): `let b: u8 = x.truncate();` |
| `i32 as u32` (reinterpret) | `x.cast_unsigned()`; back with `cast_signed()` |
| `n as f64` (`usize`, `u64`, `i64`) | `f64::round_from(n)` (nearest, ties to even) |
| `f as i64` | `i64::try_from_float(f.trunc())?` (errors on NaN, infinity, out of range) |
| `x as f32` (`x: f64`) | `f32::round_from(x)` |

`truncate()` takes no arguments; context or `::<T>()` sets its target.
`round_from`/`try_from_float`: provisional names.

### 4.3 Floats

IEEE 754 as Rust. Rust leaves NaN bits and math precision unspecified, so
float methods that read sign or bits (`to_bits`, `is_sign_positive`,
`is_sign_negative`, `copysign`, `signum`, `total_cmp`, `min`, `max`) and math
methods (`sin`, `exp`, `powf`, `ln`, ...) are `FX0520`. Use `std::{float,
math}`, generic over `f32` and `f64` (signatures provisional):
`float::bits(x)` (`u32` for an `f32`; any NaN reads as one canonical NaN),
`float::min`/`max(a, b)` (-0.0 below +0.0), `float::total_cmp(a, b)`
(provisional name), and `math::sin(x)`, ..., one library on every target.
`f64::from_bits`, `sqrt`, `mul_add`, `floor`, `ceil`, `round`, `trunc`, `abs`,
`is_nan`, and `is_finite` stay methods (provisional for rounding and
classification).

## 5. Traits and generics

### 5.1 Power ceiling

Every item signature is annotated; inference is local to a body. Generics
are definition-checked: an instantiation never produces a type error. These
are rejected:

| Not allowed | Rust example | Code | Write instead |
| --- | --- | --- | --- |
| Specialization, overlapping impls | `default fn` | `FX0402` | one impl per type; a newtype |
| Generic associated types | `type Item<'a>` | `FX0403` | owned items or indices |
| Lifetime parameters on traits | `trait Parse<'a>` | `FX0201` | a lifetime on the method or the impl |
| Higher-ranked bounds beyond elision, HKTs | `for<'a> Fn(&'a T) -> &'a U` | `FX0404` | `Fn(&T) -> &U` |
| Negative impls, user auto traits | `impl !Send for T` | `FX0405` | a `PhantomData<Rc<()>>` field |
| Impl where-clause not smaller than its head | `impl<T> Foo for T where Vec<T>: Foo` | `FX0406` | bound `T` directly |
| Polymorphic recursion | `fn f<T>(x: T) { f(vec![x]) }` | `FX0407` | an enum or `Box<dyn ..>` |
| Two inherent methods with one name | two `fn new` | `FX0408` | distinct names |

A missing auto or marker trait never makes impls disjoint:
`impl<T: Pure> K for T` and `impl K for Net` overlap (`FX0402`). A returned
`impl Trait` is
`Send`, `Sync`, or `Pure` only if it names them
(`-> impl Iterator<Item = u8> + Send`); it is `'static` when it captures no
borrow or states `+ 'static`.

### 5.2 Impl placement

An `impl` lives in the module that defines the type or (for trait impls)
the trait (`FX0401`, fix moves the block).

## 6. Authority: capabilities

### 6.1 The rule and the capability set

Code touches the outside world only through capability values it is handed.
Std has no free functions that do I/O, read the time, draw randomness, read
the environment, or spawn. A function's parameter types bound what it can
do, in every dependency.

| Capability (path) | Main methods | Attenuation |
| --- | --- | --- |
| `Fs` (`std::fs`) | 6.3 | `dir(path, Mode) -> io::Result<Fs>`, `read_only() -> Fs` |
| `Net` (`std::net`) | `get(&Url, deadline: Instant) -> Result<Vec<u8>, NetError>`, `connect(addr) -> io::Result<TcpStream>`, `bind(addr) -> io::Result<TcpListener>`, `bind_udp(addr)`, `resolve(host)` | `only(hosts) -> Net`, `no_bind() -> Net` |
| `Clock` (`std::time`) | `now() -> Instant`, `system_time() -> SystemTime`, `sleep(d)`, `sleep_until(t)`, both `-> Result<(), Cancelled>` | `monotonic_only() -> Clock` |
| `Rand` (`std::rand`) | `next_u64()`, `gen_range(a..b)`, `gen_bool(p)`, `fill_bytes(&mut [u8])`, `shuffle(&mut [T])`, `rng() -> Pcg64` | — |
| `Env` (`std::env`) | `args()` (program name first), `var(k)`, `vars()`, `current_dir()`, as `std::env`'s | `only(["HOME", ..])`, `args_only()` |
| `Stdin` (`std::io`) | `read_line`, `lines`, `read_to_string`, `read_to_end`, `is_terminal`: inherent, receivers `&self` (below) | — |
| `Proc` (`std::process`) | `command(prog) -> Command` (a handle: companion 20), `exit(code: i32) -> !` | `only(["git", ..])` |
| `Tasks` (`std::task`) | `scope`, `spawn_isolated`, `shield`, `checkpoint`, `yield_now` (7.1), `fault_point(name) -> bool` (`false` outside simulation, companion 17); creating synchronization handles (7.4) | `limit(n) -> Tasks` |

- Their methods take `&self` (provisional; handles' receivers: companion
  20): `&Rand` or `&Stdin` is enough. For many draws take `let mut
  rng = rand.rng();` once: a `Pcg64` is an ordinary `Pure` value with
  `&mut self` methods (`next_u64`, `gen_range`, `gen_bool`, `shuffle`);
  `Pcg64::seed_from_u64(n)` needs no capability.
- Capabilities are not `Copy` or `Pure` (6.6). They are `Clone` (same
  authority; provisional) and `Send + Sync`, so scoped children may borrow
  them. Pass them by reference: `fs: &Fs`.
- Attenuation returns a narrower capability of the same type, so it chains:
  `root.net.only([..]).no_bind()`. `Net::only` takes `"host:port"` entries
  in any iterable of strings (provisional).
- Every capability error type has a cancellation case whose
  `is_cancelled()` is true (7.2).

### 6.2 `#[tide::main] fn main(root: Root)`

```rust
#[tide::main]
fn main(root: Root) -> Result<(), AppError> {
    let data = root.fs.dir("/var/lib/app", Mode::ReadWrite)?;
    let net = root.net.only(["db.internal:5432"]);
    app::run(&data, &net, &root.clock)?;
    Ok(())
}
```

- `Root` has public fields `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
  `proc`, `tasks`, and `log` (sink configuration, 6.5).
- Every binary needs the attribute (`FX0512`, safe fix); it mints `root`
  via Rust's parameterless entry. `main` takes `root: Root` or nothing,
  returning Rust's `Termination` (taught: `()`, `Result<(), E: Debug>`,
  `ExitCode`). `Err(e)` prints `Error: {e:?}` and exits 1; for `Display`
  and the `source()` chain, log it and return `ExitCode`. Panics exit 101.
- Name `Root` only in `main`'s signature or as `Root::sim`.
  Field patterns may destructure `root`, including `main`'s parameter.
  `root` cannot be passed, borrowed into calls, stored, returned, or captured
  (`FX0503`); move or borrow its fields (`root.fs`, `&root.clock`). Calling
  `main` is `FX0506`; tests call functions taking capabilities.
- `Root::sim(seed)` and everything in `std::sim` exist only in test code
  (`FX0504`, 6.8).

### 6.3 Files: paths, append, durability

```rust
impl Fs {                                         // every method takes &self; p: impl AsRef<Path>
    // read, read_to_string, write (not durable), create_dir_all, read_dir, metadata,
    // exists, rename, remove_file, remove_dir, remove_dir_all: as std::fs (a
    // DirEntry, like a File, carries the Fs's authority)
    fn write_synced(&self, p, data: impl AsRef<[u8]>) -> io::Result<()>;   // durable when it returns
    fn open(&self, p) -> io::Result<File>;                          // read only
    fn create(&self, p) -> io::Result<File>;                        // write; create or truncate
    fn open_with(&self, p, opts: &OpenOptions) -> io::Result<File>; // append, create_new, read+write
    fn sync_dir(&self, p) -> io::Result<()>;                        // make p's entries durable
    fn dir(&self, p, mode: Mode) -> io::Result<Fs>;    // Mode::Read | Mode::ReadWrite; fails if p is not a directory
    fn read_only(&self) -> Fs;
}
impl File {        // io::Read, io::Write, io::Seek, also for &File
    fn sync(&self) -> io::Result<()>;                  // D7; `sync_all`, `sync_data` accepted
    fn set_len(&self, n: u64) -> io::Result<()>;       // truncate or extend; works in append mode
    fn close(self) -> io::Result<()>;                  // metadata, as std
}
```

- **`OpenOptions`** is Rust's builder without its ambient `open` method
  (`FX0501`): `fs.open_with("events",
  OpenOptions::new().read(true).append(true).create(true))?`. As in POSIX,
  such a file reads from offset 0, and every write goes to the end. A `File`
  carries the authority of the `Fs` that opened it.
- **Paths.** `root.fs` is rooted at `/`; a relative path resolves against
  the starting working directory (provisional; `/` under simulation).
  `read_only()` keeps the same base. `fs.dir(p, mode)` resolves `p` like any
  path and returns an `Fs` that resolves relative paths beneath `p`; an
  absolute path, or a `..` or symlink that would leave `p`, fails with
  `ErrorKind::PermissionDenied` (provisional kind), as do writes through a
  `Mode::Read` `Fs`. Missing files give `NotFound`. Call `create_dir_all`
  before attenuating to a directory that may not exist yet.
- **Durability** (D7, D15). Writes are durable only once synced: `file.sync()`
  for contents, `fs.sync_dir(dir)` for the entries changed by `create`,
  `rename`, removals, or a new subdirectory; `write_synced` syncs the
  file and its entry. After a crash, unsynced data may be lost or torn; a
  `rename` is never half done (companion 17). An acknowledged write is:
  write, `sync()`, and, the first time a file or directory is created,
  `sync_dir` of its parent. Atomic replace:
  `fs.write_synced("f.tmp", data)?; fs.rename("f.tmp", "f")?;
  fs.sync_dir(".")?;`.
- **Torn tails.** A crash can leave a partial last record. Cut it off
  (`set_len`, then `sync`) before appending again, or the next acknowledged
  record is glued to the torn bytes and lost on the next read. Companion
  15.4 is a crash-safe log built this way, with its crash test.

### 6.4 No globals; deterministic hashing

Safe code has no mutable globals or thread-locals (`static mut`, a
`static` holding atomics or `Mutex`, `thread_local!`, `lazy_static!`:
`FX0502`). A `static` must be `'static + Sync + Pure`, its value computed
at compile time by `const fn`. Build a value that needs run-time work in
`main` and pass it down.

`HashMap::new()` uses a fixed hash key, so iteration order is deterministic
(but unspecified). For untrusted keys use
`HashMap::with_hasher(RandomState::from_rand(&rand))`. `Arc` has no
`strong_count`, `weak_count`, `get_mut`, `try_unwrap`, or `into_inner`,
because their results depend on other tasks; `make_mut` stays.

### 6.5 Write-only sinks are not authority

`print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`, `error!`..`trace!`,
`std::io::stdout()`/`stderr()` (write-only `impl Write`, always `Ok`), and
`std::trace::span(name)` (guard closing on drop) write task-owned sinks:
no reads, capability, or suspension, so functions stay authority-free; a pure decoder may
`warn!("legacy row {}", row.id)`.

- Arguments always evaluate and records format, even disabled; configuration
  changes no results or panics. Output follows program order, sequential
  order in `join`/`par_map`.
- `main` sets `root.log: std::log::LogConfig` with `level`, `format`,
  `to_file(file)`. `std::log` defines
  `Level::{Error, Warn, Info, Debug, Trace}` and `Format::{Text, Json}`.
  Defaults: `println!` to stdout; `eprintln!` and `info!` upward to stderr;
  `dbg!` to stderr in debug builds (2.1). Library `println!` is lint `FX0508`.
- `Secret<T>` has no `Debug`/`Display`; read with `expose()`.

### 6.6 `Pure` and authority-free functions

`Pure` (prelude) means no authority is reachable from the type. Std declares
it with a blanket impl, so rustc accepts every bound; Tide's checker decides
it structurally wherever a type meets a `Pure` bound or is coerced to a
`Pure` trait object (`FX0517`, unless the rule that needs `Pure` has its
own code). **Not `Pure`**: the capability types and `File`, `Command`, and
other handles derived from them; synchronization handles (`Shared`, channel
ends, `Scope`, `Task`, `Arc`, `sync::Weak`, atomics, `std::sync::raw`); raw
pointers; a `dyn Trait` whose trait lacks a `Pure` supertrait; an unbounded
type parameter. **Everything else is `Pure` exactly when its parts are**,
including references, collections, `Arena`, `Box`, structs, enums,
closures, and `Rc`, `Cell`, and `RefCell`, which are not `Sync`, so their
state never leaves one task. Share by reference inside a scope, not by
`Arc`.

`dyn Trait + Pure` is not Rust (E0225). Declare a companion trait with a
blanket impl and use its object, which upcasts to `&dyn Value`:

```rust
trait PureValue: Value + Pure {}
impl<T: Value + Pure + ?Sized> PureValue for T {}
fn show(v: &dyn PureValue) { .. }
```

A function or closure is **authority-free** when every parameter and capture
type is `Pure`. It cannot do I/O or observe time, randomness, or other
tasks; its result depends only on its arguments' memory; it never suspends.
Generic code is classified per instantiation; require it with a bound,
`F: Fn(&T) -> U + Pure`. `#[pure]` on a function makes losing
authority-freedom a compile error (`FX0511`). Statics (6.4), `const fn`
(2.3), `Shared` (7.4), parallel regions (8), and contract clauses (10.1)
require `Pure`.

### 6.7 Network

`Net::get` performs an HTTP(S) GET and returns the body. `NetError` has
`Timeout` (the operation's deadline passed), `Cancelled`, `Denied` (a host
outside `only`), `Refused`, `Reset`, `Dns`, `Status(u16)` (a non-2xx
response), and `Io(io::Error)`; `is_cancelled()` and `is_transient()`
(`Timeout`, `Reset`, `Refused`, and `Status` 408, 429, or 5xx) classify
them. `NetError` is not `PartialEq`: test it with `matches!`.
`Url::parse(&str) -> Result<Url, UrlError>`; `Url` has the `url` crate's
accessors (`host_str`, `port_or_known_default`, `path`, `as_str`) and
`Display`. `TcpStream` is `io::Read + io::Write`, also for `&TcpStream`.

### 6.8 Simulation

Every capability has a deterministic simulated version. `Root::sim(seed)`
returns a `SimRoot` (`std::sim`) with `Root`'s capability fields, all
simulated (pass `&sim.fs` and the rest to the code under test), plus
`sim.world` (setup), `sim.faults` and `sim.node(name)` (faults and crashable
nodes, companion 17), and inspection methods. It works in any test, plain or
property: the test body runs as the simulation's main task, so it can spawn,
wait, and read virtual time.

| Item | Effect |
| --- | --- |
| `#[sim_test(seeds = 200)] fn t(sim: SimRoot)` | runs the test once per seed with a fresh `SimRoot`; plain `#[sim_test]` runs 100 seeds; more parameters are generated (10.2) |
| `sim.world.file(path: impl AsRef<Path>, contents: impl AsRef<[u8]>)` | create a file (parents included) |
| `sim.world.args(["prog", "a.txt"])`, `env_var(k, v)`, `stdin("text")` | program inputs; `args` as `env.args()` returns them |
| `sim.world.http(addr: &str, f: impl Fn(&Request) -> Reply + Send + 'static)` | a simulated HTTP peer at `"host:port"`; `req.url() -> &Url`; `Reply::ok(body: impl Into<Vec<u8>>)`, `.after(d)`, `Reply::status(503)`, `Reply::hang()` |
| `sim.net.drop_all() -> Net` | a `Net` whose every operation times out |
| `sim.elapsed() -> Duration`, `sim.records() -> Vec<Record>`, `sim.trace()` | virtual time; sink records; the event trace |

(`Request` and `Record` are provisional.)

- **Time.** Monotonic time starts at zero, wall time at 2000-01-01. Virtual
  time advances only when every task is blocked, jumping to the next timer.
  Computation and `Clock`, `Rand`, `Env`, and `Tasks` calls take none; each
  file operation takes up to 1 ms (provisional), and a fault-free
  `Net::get` 2–20 ms plus the reply's `after(d)` (companion 17). Bound
  `sim.elapsed()` by a deadline your code sets, or with slack. If no task
  can ever run again, the run fails with a deadlock report.
- **Order.** Capability operations, `spawn`, `join`, channel operations,
  `Shared::with*`, and checkpoints are scheduling points; the seed picks the
  interleaving. Parallel regions (8) run sequentially. Each run writes a
  replay manifest that reproduces it bit for bit (companion 17).
- **Records.** Every sink record, `println!` output included, enters
  `sim.records()` with its node (fields: companion 17).

### 6.9 Ambient Rust calls and the admission allowlist

Ambient std calls (`std::fs::read`, `File::open`, `env::args`,
`Instant::now`, `t.elapsed()`, `thread::sleep`, `rx.recv_timeout(d)` (use
`recv_deadline`), `thread_rng`, `io::stdin()`, `Command::new`,
`process::exit`, ...) are `FX0501`; the fix uses a capability in scope or
proposes adding the parameter (companion 14.2).

The checker admits only listed `core`, `alloc`, and std inherent items;
everything else is `FX0520`, notably: hardware intrinsics (`core::arch`);
addresses without the `unsafe` grant (11); `core::sync::atomic`;
`OnceLock`, `LazyLock`; float observers and math methods (4.3); I/O
methods on re-exported std types (`Path::exists`, `metadata`; use `fs`);
and `io::Error::last_os_error`.

### 6.10 Patterns

**Thread a capability.** When a leaf starts needing authority, add the
parameter along the call chain; never reach for a global or `Root`.
`tide fix --thread Fs --to decode_user` makes that edit, test call sites
included.

**App-context struct.** For service code, bundle attenuated capabilities
and clients built in `main` (`pub struct App { db: Db, files: Fs, clock:
Clock }`), and pass narrow pieces down. Keep business logic in
authority-free functions that take data; pass each leaf the single
capability it uses.

## 7. Concurrency

There is no `async`/`await`, `Future`, `Pin`, or executor choice in library
code (`FX0601`; the safe fix deletes `async` and `.await`). A function
suspends only inside a capability or synchronization operation;
authority-free code never suspends. Companion 14.3 maps async Rust idioms.

### 7.1 Scopes and tasks

```rust
impl Tasks {
    fn scope<'env, T, E>(&'env self, body: impl FnOnce(Scope<'_, 'env, E>) -> Result<T, E>) -> Result<T, E>
        where E: From<TaskError> + Send;
    fn spawn_isolated<T: Send>(&self, f: impl FnOnce() -> T + Send + 'static) -> Result<T, Panicked>;
    fn limit(&self, n: usize) -> Tasks;               // attenuation: at most n live children
    fn shield<T>(&self, f: impl FnOnce() -> T) -> T;   // ignore enclosing cancellation
    fn checkpoint(&self) -> Result<(), Cancelled>;
}
pub struct Scope<'s, 'env: 's, E> { .. }            // Copy + Send + Sync: pass by value
impl<'s, 'env, E> Scope<'s, 'env, E> {
    fn spawn<U: Send + 's>(self, f: impl FnOnce() -> Result<U, E> + Send + 's) -> Task<'s, U, E>;
    fn set_deadline(self, at: Instant);
    fn cancel(self);
}
impl<U, E> Task<'_, U, E> { fn join(self) -> Result<U, E>; }
pub enum TaskError { Cancelled, DeadlineExceeded }  // Debug, Clone, Copy, PartialEq, Eq, Error (provisional)
pub struct Panicked { pub message: String }          // (provisional)
```

- **Structure.** Tasks are spawned into scopes and cannot outlive them;
  `thread::spawn` and `tokio::spawn` are `FX0602`. `scope` returns only
  after every child has finished; scope exit and `join` wait, never
  cancelled (7.2). Children may borrow anything that outlives the `scope`
  call, capabilities and `Shared` handles included.
- **Failure.** A child or the body fails when it returns `Err`. The first
  failure cancels the scope (body, children, nested scopes). Once every
  child has finished, `scope` re-raises the panic of the earliest-spawned
  panicking child, if any; otherwise it returns the error of the
  earliest-spawned failing child, if any (`DeadlineExceeded` if the scope's
  own deadline cancelled it, 7.2); otherwise the body's result. An error a
  child returns after seeing a cancellation that the body's `Err` or a
  sibling's failure caused is an echo and never counts, so the root cause
  is reported: the body's error when the children only echo it, and `x`'s
  error when the body fails through `x.join()?`. Other errors go to one
  `debug!` record. To collect per-item errors without cancelling siblings,
  return them as data: `s.spawn(move || Ok(net.get(u, deadline)))` gives
  `Task<'_, Result<Vec<u8>, NetError>, E>`. Since `E: From<TaskError>`, a
  scope's error enum needs a `#[from] TaskError` variant.
- **`join`** returns the child's `Ok` value, or
  `Err(TaskError::Cancelled.into())` if the child failed, panicked, or was
  cancelled (the scope reports the real cause), so `x.join()?` in the body
  is the normal form. Dropping a `Task` without joining is allowed.
- **Panics.** A child's panic cancels the scope and is re-raised (above),
  as in `std::thread::scope`. A panic in the body cancels and joins the
  children, then continues unwinding. Only destructors observe data a
  panicking child was mutating.
- **Isolation.** `tasks.spawn_isolated(move || ..)` runs a `'static` closure
  as a new task, waits for it, and returns `Err(Panicked)` if it panicked,
  the only place a panic becomes a value (servers: one per request). It
  inherits the caller's cancellation.
- **Bounded concurrency.** `let tasks = root.tasks.limit(4);` then spawn one
  child per item: scopes from that `Tasks` run at most 4 children at once,
  and `spawn` waits for a free slot, a checkpoint: in a cancelled scope
  it starts nothing and the child counts as cancelled. A child spawning
  under its own limit can deadlock it.
- **A scope passed down** (`fn serve<'s>(bg: Scope<'s, '_, E>, ..)`)
  outlives the function's locals, so its children must own their data
  (`move` owned values) or borrow what the signature ties to the scope
  (`cfg: &'s Config`).
- The task, not the OS thread, is the isolation unit: values that never
  leave a task need not be `Send` (an `Rc` inside one task is fine).

```rust
fn fetch_both(tasks: &Tasks, net: &Net, clock: &Clock, a: &Url, b: &Url)
    -> Result<(Vec<u8>, Vec<u8>), FetchError>          // FetchError: #[from] TaskError, #[from] NetError
{
    let deadline = clock.now() + Duration::from_secs(10);
    tasks.scope(|s| {
        let x = s.spawn(|| Ok(net.get(a, deadline)?));
        let y = s.spawn(|| Ok(net.get(b, deadline)?));
        Ok((x.join()?, y.join()?))
    })
}
```

### 7.2 Cancellation and deadlines

- **Level-triggered.** Once a scope is cancelled, every checkpoint inside it
  returns a cancellation error, and keeps doing so: `NetError::Cancelled`,
  `io::ErrorKind::Cancelled`, `Cancelled` (`std::task`) from `clock.sleep`
  and `tasks.checkpoint()`, and channel errors whose `is_cancelled()` is
  true. Every capability operation that may suspend is a checkpoint;
  authority-free code, `Shared::with*`, joins, scope exit, and `File::close`
  are not, and, like an uncancellable system call, are never interrupted,
  so deadlines are soft. Work unwinds through ordinary `Err`
  returns; operations that hand a value over return it inside the error
  (`SendError::Cancelled(v)`).
- **Operation deadlines.** `net.get(url, deadline)` returns
  `NetError::Timeout` when `deadline` passes; `clock.sleep_until(t)` and
  `rx.recv_deadline(t)` likewise (`Instant`s from a `Clock`).
- **Scope deadlines** (D8). `s.set_deadline(t)` lowers the scope's deadline
  (a nested scope uses the earliest of its ancestors'). When it passes, the
  scope is cancelled, but that alone is not a failure: if the body and every
  child still return `Ok`, for example because children turned cancellation
  errors into values, `scope` returns `Ok` with the partial results. A child
  that returns `Err` after the deadline is reported as
  `Err(TaskError::DeadlineExceeded.into())` (7.1); the body's own `Err` is
  returned unchanged. `s.cancel()` cancels the same way but reports the
  child's own error. When a scope deadline and an operation deadline
  coincide, the operation sees the cancellation.
- **Cleanup.** `tasks.shield(|| ..)` ignores enclosing cancellation; give
  bounded cleanup its own inner scope with a deadline.

### 7.3 Channels

Channels are Rust's `std::sync::mpsc`, except that creation takes `&Tasks`
(7.4): `channel(tasks)` (unbounded; a lint in libraries, `FX0608`),
`sync_channel(n, tasks)`, `Sender: Clone`, `send`, `try_send`, `recv`,
`try_recv`, `recv_deadline(t)`, and iteration, which ends at the first
`recv` error. `recv`, `recv_deadline`, and every `SyncSender::send`
are checkpoints, so their errors are enums with `is_cancelled()`:
`RecvError::{Disconnected, Cancelled}`, `RecvTimeoutError` (adds `Timeout`),
and `SendError::{Disconnected(v), Cancelled(v)}` (`into_inner()`). A
disconnect alone is not an echo (7.1). Payloads must be `Send`. A `Receiver` has
one consumer; for N workers over a list, use `Tasks::limit` (7.1). To wait
on several sources (Rust's `select!`, `FX0605`), send one enum through one
channel.

### 7.4 `Shared<T>`

```rust
impl<T: Send + Pure> Shared<T> {                   // std::sync
    fn new(v: T, tasks: &Tasks) -> Shared<T>;      // creation takes &Tasks (argument position provisional)
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError>;
    fn with2<U: Send + Pure, R>(a: &Shared<T>, b: &Shared<U>,
        f: impl FnOnce(&mut T, &mut U) -> R + Pure) -> Result<R, LockError>;
    fn with_all<R>(cells: &[Shared<T>], f: impl FnOnce(&mut [&mut T]) -> R + Pure)
        -> Result<R, LockError>;                   // (provisional name)
    fn same(a: &Shared<T>, b: &Shared<T>) -> bool;
    fn reset(&self, v: T);                         // replace the payload, clear poison
}
pub enum LockError { Aliased, Poisoned }           // Debug, Clone, Copy, PartialEq, Eq, Error (provisional)
```

- `Shared<T>` replaces `Arc<Mutex<T>>` and `.lock().unwrap()` (`FX0604`).
  It is a handle (`clone()` names the same cell), `Send + Sync`, not
  `Pure`. Payloads are `Send + Pure` (`FX0606`), so a cell never holds a
  capability, channel end, or another `Shared`.
- **Creating a handle takes `&Tasks`.** Cells, channel ends, `Arc`,
  `sync::Weak`, and atomics let tasks observe each other's timing (D7), so
  every constructor takes `&Tasks`, last: `Shared::new(v, tasks)`,
  `channel(tasks)`, `Arc::new(v, tasks)`, `Arc::downgrade(&a, tasks)`,
  `AtomicU64::new(0, tasks)` (`FX0505` without it; `tide fix --thread
  Tasks`). Cloning needs nothing. Authority-free code can neither create
  nor receive a handle. A test takes `&sim.tasks` from `Root::sim(seed)`.
- `with2` and `with_all` acquire all their cells atomically in a global
  order, so they cannot deadlock. If two operands are the same cell, the
  call returns `Err(LockError::Aliased)` without running `f`; test identity
  first with `Shared::same(&a, &b)`. `with_all` on an empty slice runs `f`
  on an empty slice.
- `f` must be `Pure`, so it captures no capability, channel end, or
  `Shared` handle and cannot do I/O, suspend, or nest another `with`
  (`FX0603`). Logging is allowed; printing is a lint (`FX0607`).
- Acquisition waits FIFO per cell and is not a checkpoint: it still works in
  a cancelled scope.
- A panic in `f`, an overflow trap included, poisons every cell it held;
  later calls return `Err(LockError::Poisoned)` without running `f`.
  `reset(v)` clears it. Check everything that can fail before the first
  write.
- Atomics (`std::sync::atomic`) are sequentially consistent only and meant
  for library authors; `Mutex` and `RwLock` are in `std::sync::raw` (11).

```rust
pub fn transfer(from: &Shared<Account>, to: &Shared<Account>, amount: u64) -> Result<(), BankError> {
    if Shared::same(from, to) {
        return Ok(());                                  // otherwise Err(Aliased)
    }
    Shared::with2(from, to, |a, b| {
        let debited = a.balance.checked_sub(amount).ok_or(BankError::Insufficient)?;
        let credited = b.balance.checked_add(amount).ok_or(BankError::Overflow)?;
        a.balance = debited;                            // no write until both checks pass,
        b.balance = credited;                           // and nothing here can trap
        Ok(())
    })?                                                 // BankError has #[from] LockError
}
```

### 7.5 Runtimes

The binary's manifest picks the runtime (`[bin] runtime = "threads"`, the
default, or `"blocking"`); tests and `tide sim` use the simulator.
Libraries never choose and behave the same under all of them. `threads`
runs one OS thread per task, so blocked tasks never starve runnable ones.

## 8. Parallelism

Deterministic fork-join for authority-free code:

```rust
use std::par::join;

let (a, b) = join(|| parse(x), || parse(y));
let lens: Vec<usize> = words.par_map(|w| w.len());     // == words.iter().map(..).collect()
pixels.par_for(|p| *p = p.saturating_add(10));         // over &mut [u8], disjoint
```

- `par_map`, `par_for`, and `par_sort` (stable) are inherent slice methods (no
  import): `fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send +
  Sync) -> Vec<U> where T: Pure + Sync` and `fn par_for(&mut self, f: impl
  Fn(&mut T) + Pure + Send + Sync) where T: Pure + Send`; `par_sort` needs `T:
  Pure + Send` too. The element bounds (`FX0612`) keep every `Shared` cell,
  channel, and capability out of reach of a branch.
- `join` closures must be `Pure + Send`; the slice methods' closures are
  shared by branches, so also `Fn` and `Sync` (no owned `Cell`) (`FX0610`).
  Shared captures must be `Pure + Sync` (`FX0611`: not `&RefCell`, `&Rc`,
  `&Arc`, or a capability). Disjoint `&mut` splits are fine.
- When no branch panics, `join`, `par_map`, and `par_for` equal their
  sequential versions, including sink output order. Under simulation they
  run sequentially.
- To reduce, `par_map` the expensive part and fold sequentially;
  `par_reduce` is v1. Branches cannot do I/O: for concurrent I/O, spawn
  children in a `tasks.limit(n)` scope (7.1).
- If a branch panics, the panic propagates in the enclosing task, and data
  the region could write is left unspecified (but memory-safe); only
  destructors see it. If several panic, the first in sequential order is
  reported, and buffered sink output from later branches is discarded.

## 9. Errors and panics

- Error handling is Rust's, but an ignored `Result` is an error
  (`FX0702`); `let _ = r;` discards on purpose (provisional: Rust warns).
- **Panics** (`panic!`, overflow, out-of-bounds, failed `assert!`, `unwrap`
  on `None`, a violated contract) unwind the current task and run
  destructors. A panic during that unwinding, for example in a destructor,
  aborts the process, as does stack overflow, as in Rust. `tide test` and
  `tide sim` then rerun each test and seed in flight alone, report each that
  aborts (a seed with its replay command), and go on in a fresh process; the
  run fails even if no rerun aborts. There is no `catch_unwind` (`FX0701`);
  isolate risky work with `tasks.spawn_isolated` (7.1). A panic in the main
  task exits with code 101.

## 10. Contracts

v0 contracts are executable: one spec, checked at run time in debug and test
builds, erased in release, and used to drive property tests.

### 10.1 Clauses

| Clause | Where | Meaning |
| --- | --- | --- |
| `#[requires(e)]` | attribute on a fn or inherent method, not a trait or trait-impl method (`FX0802`) | precondition, checked at entry; a failure blames the caller |
| `#[ensures(e)]` | as `requires` | postcondition, checked at each return; a failure blames the callee |
| `invariant!(e);` | first statements of a loop body | checked at each loop head: entry (even for zero iterations), every iteration, exit |

- **Loops.** `invariant!` needs `#[invariants]` or any contract attribute on
  its fn, which rewrites its loops. In `for x in a..b`, an invariant sees
  `x` as the next iteration's index: `x == a` before the first iteration,
  `x == max(a, b)` at exit. In `while` and `while let`, the head precedes
  each condition evaluation.
- **One expression per clause**: `#[requires(a, b)]` is an error (the safe
  fix splits it). Repeated clauses of one kind, `invariant!` included, are
  conjoined in order.
- **`result`** (a contextual keyword) in `ensures` is the return value, a
  place the clause cannot move out of: match on `&result` unless it is
  `Copy` (provisional).
- **`&mut` parameters.** In `ensures`, `v` (or `*v`) means the caller's
  referent at return, even if the body reassigns `v` (`v = &mut n.next`);
  `old(*v)` is its entry value. If `v`'s outer lifetime appears in another
  parameter's type, the result type (a returned `impl Trait` captures it),
  or a bound other than on the right of an outlives bound (`T: 'v`), `v`
  may be named only inside `old(..)` (`FX0803`, 04 §4.1); the usual fix
  gives the other parameter its own lifetime.
- **`old(e)`** evaluates `e` at entry and snapshots it: `Copy` values are
  copied, others cloned (a type that is neither is an error), and a reference
  snapshots its referent, so `old(v.len())` costs one word and `old(*v)`
  clones the `Vec`. `e` cannot mention `result` or a quantifier's variable:
  write `old(*v)[i]`. A panic while snapshotting violates the clause.
- **Other parameters** in `ensures` mean their entry values: a `Copy` one
  is copied and a `Clone + Pure + Sync` one cloned at entry (in generic
  code, if so bounded); any other cannot be named if the body assigns,
  moves, or mutably borrows it, through a `&mut` it holds included (narrow
  it, `old(x.len())`, or add the bound). This does not change the clause's
  class.
- **Clause expressions** have code's meaning, overflow traps included
  (widen: `a as u128 + b as u128`), and are read-only: a clause calls only
  authority-free functions whose arguments are `Pure + Sync` and that take
  `&mut` only to temporaries the clause creates (`xs.len()`, `x.cmp(y)`,
  `xs.iter().all(|x| *x > 0)`, your own `is_valid(x)`, closures capturing
  such data). So no clause reaches a capability, `Shared`, `Cell`, or
  `RefCell`, not even `cells.len()` on a slice of handles. A clause that
  uses a value of a type parameter `T` needs `T: Pure + Sync` (the fix adds
  the bound); no instantiation skips a clause. A clause names what the
  body can, private fields included.
- **Added syntax**, clauses only: `a ==> b` (implication, lowest precedence,
  right-associative); `forall |i: T| guard ==> p` and
  `exists |i: T| guard && p`, where `guard` bounds the binder as
  `lo <= i && i < hi` (an unsigned binder implies `0 <= i`) or
  `c.contains(&i)`. An unbounded quantifier is an error. A quantifier body
  extends as far right as possible: to the end of the clause, or of the
  enclosing parentheses or `match` arm. So write
  `result == (exists |i: usize| i < xs.len() && xs[i] == x)`. A quantified
  invariant costs a linear scan each time it is checked.
- **When clauses run.** A clause built only from operators on std types,
  literals, `result`, fields, `old` of `Copy` values, and std's accessors
  (`len`, `is_empty`, indexing, `get`, `contains`, `is_some`) is *checked*;
  any other, such as one with a quantifier, a closure, or a call to your own
  function, is *sampled*. Debug and test builds evaluate checked clauses at
  the points in the table (debug builds: `for` over `a..b` only at entry
  and exit); `tide test` also checks sampled clauses on every
  call, each under a step budget, so the outcome depends only on the
  inputs. One that runs over budget is reported as unchecked, not failed.
  Release builds erase clauses.
- **Violations.** A clause that is false, or whose evaluation panics, is
  violated, and panics with `requires violated: <clause text>` (or
  `ensures`, `invariant`), the function's name, and the blamed span.

```rust
#[requires(xs.len() > 0)]
#[ensures(result < xs.len())]
#[ensures(forall |i: usize| i < xs.len() ==> xs[result] >= xs[i])]
pub fn max_index(xs: &[i64]) -> usize {
    let mut best = 0;
    for i in 1..xs.len() {
        invariant!(best < i);
        if xs[i] > xs[best] { best = i; }
    }
    best
}

#[ensures(dst.len() == old(dst.len()) + src.len())]
pub fn push_all(dst: &mut Vec<i64>, src: &[i64]) { for x in src { dst.push(*x); } }
```

### 10.2 Property tests

- A `#[test]` function with parameters is a property test: `tide test`
  generates and shrinks inputs. Generated types: integers, floats, `bool`,
  `char`, `String`, `Vec`, arrays, tuples, `Option`, `Result`, maps, sets,
  and structs and enums whose fields can all be generated, or that
  implement `Arbitrary`; a `&T` or `&mut T` parameter borrows a generated
  `T` (`&str` from a `String`).
- **With a simulator.** A generated `seed: u64` parameter and
  `Root::sim(seed)` in the body give each case its own simulator (6.8);
  `#[sim_test] fn t(sim: SimRoot, sizes: Vec<u16>)` is shorthand, with
  `seeds = N` cases, and shrinking keeps the seed (provisional).
  `requires` and the vacuity floor apply to both forms.
- **`#[requires]`**, on a test or a tested function, filters and guides
  generation. Conjuncts in a recognized shape are generated directly; the
  rest filter. Shapes: scalar ranges (`n <= 64`, `lo <= hi`), length bounds
  (`xs.len() > 0`), an index below a length (`i < xs.len()`),
  `xs.is_sorted()`, `xs.contains(&x)`, `o.is_some()`, and `x == c`
  (provisional spellings). Your own predicate, such as `is_valid(x)`, only
  filters. An early `return` counts as a pass, so prefer `requires`.
- **Vacuity.** `tide test` runs a property test until 100 cases (or `N`,
  for `seeds = N`) satisfy its `requires`. If under 1% of generated cases
  do, it fails the test as `vacuous` and names the conjunct that rejected
  most. For a rare condition, construct such inputs (`xs.sort()`), use a
  recognized shape, or implement `Arbitrary`.
- **Generated contract tests** (04 §2.4). `tide test` also tests every
  function with an `ensures` on generated inputs satisfying its `requires`,
  under the same vacuity floor: capabilities from a fresh `Root::sim(seed)`,
  private-field types by random public constructor and method calls
  that meet their `requires`, scalars over their full range, type
  parameters as `i64` if it meets the bounds, else `String`. A function
  that cannot be generated (closure, `dyn`, or handle parameters; unmet
  bounds) is skipped as unchecked.
  `#[allow(vacuous, reason = "..")]` reports a vacuous generated test as
  unchecked, visibly in review. (Instantiation, skipping, and `allow` are
  provisional.)

### 10.3 Contracts in review

A contract's hash covers the bodies of every function its clauses call,
transitively. v0 has no solver, so any change to a public item's contract
needs human approval (11), except adding one where there was none.

## 11. Unsafe, trust holes, and review

| Hole | Spelling | Rule |
| --- | --- | --- |
| Unsafe code | `unsafe { .. }`, `unsafe fn`, `unsafe extern` blocks and items, `unsafe impl`, `#[unsafe(..)]` | `unsafe` grant (`FX0901`); each `unsafe { .. }` block needs a `// SAFETY:` comment (`FX0902`, provisional: error) |
| A type the checker finds not `Pure` but that is authority-free | `#[trusted_pure]` with a reason, not `unsafe impl Pure` | `unsafe` grant (`FX0901`) |
| `std::sync::raw` locks, weaker atomic orderings | — | `unsafe` grant (`FX0520` without it) |
| Addresses | `*const T`, `*mut T`, `{:p}`; a function pointer's integer casts, ordering, hashing, `Debug` | `unsafe` grant (`FX0520` without it) |

`tide review <base>` prints the trust diff: public signatures, declared
authority (12.4), contracts, trust holes, and grants. Changes that need a human
approval, recorded outside the agent's write scope and verified by CI:
widening authority, weakening or removing a contract (10.3), adding a trust
hole, and changing a `#[protected]` test. Sealed items (`tide.seal`) are
fixed, except bodies it leaves free (`FX0920`, companion 18). Never make
one of these to get a green check: if a contract, sealed item, or
`#[protected]` test looks wrong, stop and report it. Test another agent's
branch without its own VM only if `tide review` against a reviewed base
prints `run: covered` (companion 19).

## 12. Modules and packages

### 12.1 Files are modules

- As in Rust: `src/main.rs` and `src/lib.rs` are crate roots, `mod net;`
  declares `src/net.rs` (or `src/net/mod.rs`). `mod foo;` with no file is
  `FX0910`; `#[path]` is `FX0911`.
- `tide check path.rs` and `tide test path.rs` treat one file as a crate
  root, for single-file programs and exercises: a binary root if it
  defines `main`, otherwise a library root, where `println!` is lint
  `FX0508` (provisional).

### 12.2 Imports

- `use` as Rust; `extern crate` is `FX0912`.
- **Unused imports are errors** (`FX0913`, safe fix). **Import cycles
  between modules are errors** (`FX0914`): put shared types in leaf modules;
  a child's `use super::*` is fine while the parent does not import the
  child.

### 12.3 Prelude and std paths

The prelude is Rust's plus `Root`, the `Error` trait (`std::error::Error`,
so `Box<dyn Error>` and `e.source()` need no import), the derives `Error`,
`Encode`, `Decode`, and the contract attributes; `Pure` and `Secret` are in
it too (provisional). Import everything else:

| Items | Path |
| --- | --- |
| `Fs`, `Mode`, `File`, `OpenOptions`, `Metadata` | `std::fs` |
| `Net`, `NetError`, `Url`, `UrlError`, `TcpStream`, `TcpListener`, `UdpSocket` | `std::net` |
| `Clock`, `Duration`, `Instant`, `SystemTime` | `std::time` |
| `Rand`, `Pcg64`; `RandomState` | `std::rand`; `std::collections::hash_map` |
| `Env`, `VarError` | `std::env` |
| `Stdin`, and `io::{self, Read, Write, BufRead, ..}` | `std::io` |
| `Proc`, `Command`, `ExitCode` | `std::process` |
| `Tasks`, `Scope`, `Task`, `TaskError`, `Cancelled`, `Panicked` | `std::task` |
| `Shared`, `LockError`, `mpsc`, `Arc`; atomics | `std::sync`; `std::sync::atomic` |
| `join` | `std::par` |
| `float::{bits, min, max, total_cmp}`, `math::{sin, exp, ..}` | `std::float`, `std::math` |
| `Arena`, `Id` | `std::arena` |
| `Cow` | `std::borrow` |
| `json` (`encode`, `decode`, `json::Error`) | `std::json` |
| `span` | `std::trace` |
| `SimRoot`, `NodeRoot`, `SimNode`, `NodeTask`, `Faults`, `Fault`, `Request`, `Reply`, `Record`, `Sink` | `std::sim` (test code only) |
| `Ordering`, `HashMap`, `BTreeMap`, `PathBuf`, ... | as Rust |

`Faults` is the type of `sim.faults`; there is no `Faults { .. }` literal.

### 12.4 Manifest and grants

```toml
# tide.toml (provisional file name and keys)
[package]
name = "acme/linecount"          # namespaced
version = "0.1.0"
grants = ["unsafe"]              # this package's escape hatches

[bin]
runtime = "threads"              # threads | blocking

[dependencies]
"acme/json-ext" = "1.2"                                   # no grants: no unsafe
"acme/fastcrc" = { version = "0.3", grants = ["unsafe"] }

[[policy]]                       # authority policies (D17; provisional syntax)
modules = "crate::parser::*"
authority = []                   # authority-free
[[policy]]
capability = "Net"
only = ["crate::net::*"]         # no other module may declare Net
```

- **Authority policies** are checked against declared authority (the
  capability types a function's parameters and captures can reach):
  violations are `FX0516`; policy changes appear in the trust diff.
- Escape hatches are granted per dependency. `tide.lock` is on by default,
  with content hashes, and records each package's public authority and trust
  holes; an upgrade that widens either fails until a human approves. A
  dependency name that does not exist is rejected, never guessed.
- No `build.rs`, install hooks, or code that runs at fetch or build time
  (`FX0915`; generators: 2.1). Tide's driver runs rustc itself, not Cargo;
  `RUSTFLAGS` and unstable features do not apply.

## 13. Toolchain loop

- `tide check`: one verdict plus NDJSON diagnostics with fixes marked
  `safe` or `suggest`; `tide fix` applies safe fixes (`--thread <Cap>
  --to <fn>` threads a capability).
- `tide test [filter] [--seed N] [--trace]`: tests, property tests, and sim
  tests, reporting each failure's seed and replay command.
  `tide sim <bin> -- args` runs `main` under many seeds;
  `tide sim --replay <manifest>` replays a run (companion 17).
- `tide eval '<expr>'` evaluates an expression against the package;
  `tide explain FX0201` explains a code with a before/after example;
  `tide review <base>` prints the trust diff (11); `tide fmt` is the one
  canonical format.

**Typed holes**: for `todo!()` or `hole!(name)` in expression position,
`tide check` checks the rest and reports the hole's expected type,
bindings in scope, and candidate functions.

Not in v0: proofs, FFI, imported Rust crates, the evented runtime,
`par_reduce`, and more (companion Appendix A).
