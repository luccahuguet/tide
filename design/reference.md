# Ferox for Rust Programmers

Status: the agent-facing reference for the **v0 core** of Ferox, as the
tier table of `design/01-design.md` (draft 3, the decision record) defines
it. It summarizes 01 and the normative chapters `02-memory.md`,
`03-authority-concurrency.md`, and `04-contracts.md`, which still describe
draft 2.1 in places. Appendix A lists what is not in v0; section 16 lists
where v0 departs from the chapters and where the documents contradict each
other. Syntax is provisional until the E1 gate (D0). Budget: under 25,000
tokens, counted in CI (D14).

**(provisional)** marks a detail no normative document decides; this
reference picks the most Rust-like option consistent with the decisions.
Diagnostic codes (`FX0201`, ...) are placeholders.

## 0. How to use this document

You know Rust. Write Rust. Ferox is Rust's surface with a few changes in
meaning, and every Rust habit that would mean something different fails
loudly with a stable code and, usually, a machine-applicable fix. Section 1
lists what is unchanged, sections 2–11 the differences, 12.3 every std
import path, 14 the habit table, and 15 complete programs.

The differences you will hit first:

1. **No ambient I/O.** Files, network, clock, randomness, environment,
   stdin, processes, and task spawning are methods on capability values that
   `main` receives in `root: Root` and passes down as parameters.
   `println!` and `info!` still work anywhere.
2. **No `async`.** Write blocking-style code; concurrency is structured
   scopes from the `Tasks` capability.
3. **No lifetime parameters** (provisional). Elision works as in Rust. Where
   Rust needs `'a`, write `from`: `fn pick(a: &str, b: &str) -> &str from a`.
4. **Integers trap on overflow in every build**, and `as` only does lossless
   conversions.
5. **Shared mutable state is `Shared<T>`** with closure methods
   (`cell.with(|x| ..)`), not `Arc<Mutex<T>>`.

## 1. What is unchanged

Everything not listed in sections 2–13 works exactly as in Rust (2024
edition):

- **Items and syntax**: `fn`, `struct`, `enum`, `impl`, `trait`, `type`,
  `const`, immutable `static` (6.4), `use`, inline `mod name { }`,
  attributes, doc comments, `where` clauses, `pub`/`pub(crate)`/`pub(super)`.
- **Expressions**: blocks as expressions, `let`/`let else`, `if let` and let
  chains, `match` with guards, loops, labeled `break`/`continue`, `?` on
  `Result` and `Option`, ranges, closures (`move`), `unsafe` blocks (with a
  grant, 11).
- **Patterns**: all of Rust's, with exhaustiveness and binding modes.
- **Ownership and borrowing**: moves, `Copy`, `Clone`, `Drop`, shared xor
  mutable references, NLL, two-phase borrows, reborrowing, deref coercion,
  auto-ref, drop order, Rust's drop check. Only lifetime *syntax* differs
  (section 3).
- **Traits and generics**: default methods, supertraits, associated types
  and consts, blanket impls, `impl Trait` in argument and return position,
  `dyn Trait` with Rust's object safety, operator traits, `Deref`,
  `Iterator`/`IntoIterator`, `From`/`Into`/`TryFrom`/`TryInto`,
  `Display`/`Debug`, `Default`, `Fn`/`FnMut`/`FnOnce`, `Send`/`Sync`
  (structural), turbofish, const generics over integers, `bool`, and `char`.
  Limits in section 5.
- **Closures and iterators**: capture inference, every `Iterator` adapter,
  `collect` into any `FromIterator`.
- **Standard types** with their Rust API: scalars, `str`, `String`, slices,
  arrays, tuples, `Option`, `Result`, `Vec`, `Box`, `Rc`, `Arc` (minus count
  observers, 6.4), `Weak`, `Cell`, `RefCell`, `VecDeque`, `HashMap`,
  `HashSet`, `BTreeMap`, `BTreeSet`, `BinaryHeap`, `Cow`, `Path`, `PathBuf`,
  `Duration`, `Instant`, `SystemTime`, `std::cmp`, `std::ops`, `std::mem`,
  `std::fmt`, `std::convert`, and the `std::io` traits and types (`Read`,
  `Write`, `Seek`, `BufRead`, `BufReader`, `BufWriter`, `io::Error`,
  `io::ErrorKind`, `io::Result`). What changes is how you *obtain* I/O
  objects (from capabilities, section 6) and the default hasher (6.4).
- **Testing**: `#[test]`, `#[should_panic(expected = "..")]`,
  `#[cfg(test)] mod tests { use super::*; .. }`, the `assert!` family.

Source files end in `.fx`.

## 2. Surface: `!` forms, derives, attributes

### 2.1 The fixed built-in `!` forms

Users cannot define macros. The compiler implements exactly these, with
Rust's spelling and meaning:

| Form | Ferox notes |
| --- | --- |
| `print!`, `println!`, `eprint!`, `eprintln!` | Write to the task's stdout/stderr sink; need no capability (6.5). Lint in library packages. |
| `format!`, `write!`, `writeln!`, `vec!`, `matches!` | As Rust, including inline `{name}` arguments. |
| `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` | As Rust. |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | As Rust. `todo!()` is also a typed hole (13). |
| `dbg!` | As Rust. Error in release builds (`FX0507`). |
| `error!`, `warn!`, `info!`, `debug!`, `trace!` | Write-only log sink (6.5). No `use` needed; `log::info!` and `tracing::info!` are accepted spellings (provisional). |

Anything else ending in `!` is `FX0103`, including `debug_assert_eq!`
(write `debug_assert!(a == b)`), `include_str!`, `env!`, `concat!`,
`thread_local!`, `lazy_static!`, and every crate macro. `macro_rules!` and
procedural macros are `FX0101`. Instead of a macro, write a generic function
or trait; for test tables, loop over a `const` array; for generated code or
embedded data, run a generator program yourself and commit its `.fx` output.

```ferox
#[test]
fn parse_cases() {
    const CASES: [(&str, Option<u32>); 3] = [("1", Some(1)), ("x", None), ("", None)];
    for (input, want) in CASES {
        assert_eq!(parse(input), want, "input {input:?}");
    }
}
```

### 2.2 Derives

The derive set is closed: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`,
`Clone`, `Copy`, `Debug`, `Default` (as Rust), `Error`, `Encode`, `Decode`.
All are in the prelude; remove `use thiserror::Error;` and `use serde::..`.

- **`Error`** has `thiserror` semantics, built in: `#[error("..")]` on the
  type or each variant generates `Display`; `#[from]` on a single-field
  variant generates `From` and, as in thiserror, also marks the field as the
  `source` (provisional); `#[source]` (or a field named `source`) sets
  `Error::source`; `#[error(transparent)]` forwards both.
- **`Encode`/`Decode`** are compiler-generated serialization. v0 supports
  JSON only: `std::json::encode(&v) -> Vec<u8>` and
  `std::json::decode::<T>(&bytes) -> Result<T, json::Error>`. Keys:
  `#[codec(rename = "..")]`, `#[codec(rename_all = "camelCase")]`,
  `#[codec(default)]`, `#[codec(skip)]` (provisional: serde's spelling).
- `Serialize`/`Deserialize`/`thiserror::Error` are `FX0102` with safe fixes.

```ferox
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {path:?}")]
    Read { path: PathBuf, source: io::Error },
    #[error("bad config: {0}")]
    Parse(#[from] json::Error),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
pub struct Config {
    pub listen_addr: String,
    #[codec(default)]
    pub workers: u32,
}

let cfg: Config = json::decode(&bytes)?;
```

Std error types (`io::Error`, `NetError`, `UrlError`, `TaskError`,
`LockError`, `Panicked`, `Cancelled`, `json::Error`) implement
`Error + Send + Sync`.

### 2.3 `const fn`, attributes, `cfg`

- `const fn` as Rust, but authority-free (6.6), deterministic, and
  fuel-limited, with no `&mut` parameters.
- Kept Rust attributes: `#[derive]`, `#[test]`, `#[should_panic]`, `#[cfg]`,
  `#[allow]`/`#[warn]`/`#[deny]`, `#[inline]`, `#[must_use]`, `#[repr]`,
  `#[non_exhaustive]`, `#[deprecated]`, `#[doc]`. `#[cfg]` accepts `test`,
  `debug_assertions`, `target_os`, `target_arch`, `unix`, `windows`, and
  `feature = ".."` (provisional beyond `test`).
- Ferox attributes: `#[error]`, `#[from]`, `#[source]`, `#[codec]`,
  `#[pure]` (6.6), `#[sim_test]` (6.8), `#[protected]` (11).
- `#[tokio::main]`, `#[async_trait]`, `#[path]`, `#[macro_export]`, and
  `#[proc_macro*]` are errors with fixes.

## 3. Origins instead of lifetimes (provisional)

This whole section is provisional: E1 compares `from` against Rust's own
lifetime syntax. Ferox has Rust's borrow checker without lifetime
parameters. An **origin** names where a borrow comes from: a parameter, a
place path, or `'static`. A Rust signature with named lifetimes that map
exactly onto origins is accepted, and `ferox fmt` rewrites it to `from`.

### 3.1 Elision is Rust's

Rust's elision rules apply unchanged. Elided positions are `&T`/`&mut T`
without `from`, and every origin of a borrowing type written without `from`
(as Rust's `S<'_>`): `Lexer`, `Token`, `Cow<str>`, `Chars`,
`std::slice::Iter<T>`.

```ferox
fn first_word(s: &str) -> &str                          // from s
fn get(&self, key: &str) -> Option<&V>                  // from self
fn split_csv(line: &str) -> Result<Vec<Cow<str>>, CsvError>   // Cow borrows line
fn lex(src: &str) -> Vec<Token>                         // Rust: Vec<Token<'_>>
```

In `const` and `static` types, elided origins are `'static`:
`const CASES: [(&str, &[&str]); 2] = ..;`. Write `Lexer`, never `Lexer<'a>`
or `Lexer<'_>`.

### 3.2 `from` clauses: where Rust needs `'a`

`from p` after a type means "borrows from what the value at place `p`
borrows". If `p` is a reference parameter, that is its referent.

| Ferox | Rust |
| --- | --- |
| `fn longest(a: &str, b: &str) -> &str from a, b` | `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str` |
| `fn pick(a: &str, b: &str) -> &str from a` | `fn pick<'a>(a: &'a str, b: &str) -> &'a str` |
| `fn next_word(&mut self) -> Option<&str from self.src>` | `-> Option<&'a str>` in `impl<'a> Lexer<'a>` |
| `fn text(&self) -> &str from *self` (on `Token`) | `fn text(&self) -> &'a str` in `impl<'a> Token<'a>` |
| `fn longest(words: &[&str]) -> &str from *words` | `fn longest<'a>(words: &[&'a str]) -> &'a str` |
| `fn add(m: &mut HashMap<&str from w, u32>, w: &str)` | `fn add<'a>(m: &mut HashMap<&'a str, u32>, w: &'a str)` |
| `fn merge(a: &str, b: &str) -> Vec<&str from (a, b)>` | `-> Vec<&'a str>` with `a`, `b: &'a str` |
| `type Item = Result<Token from Self.src, LexError>;` | `type Item = Result<Token<'a>, LexError>;` |
| `fn find(n: &str, t: &str) -> Option<Match from { needle: n, line: t }>` | `-> Option<Match<'n, 't>>` |

Rules:

- `from` binds looser than `&` and `dyn` (`&str from a` is `(&str) from a`)
  and applies to **every elided origin position** in the type before it. To
  target an inner one, parenthesize: `&mut (&str from v)`.
- A bare list `from a, b` is allowed only at the end of a return type.
  Elsewhere `from` takes one origin, and a comma ends it, so
  `Result<Token from Self.src, LexError>` has two type arguments; write
  several origins as `from (a, b)`.
- Places are a parameter or `self`, then fields and `*` (no calls or
  indexing). `from self.src` is the field's origin, not the `&mut self`
  borrow; that is how a token from `&mut self` survives the next call.
  `from *self` is what the value behind `&self` borrows; a plain elided
  `-> &str` on `&self` borrows the reference itself.
- `from p` where `p` holds no borrow (`from self.pos`, `pos: usize`) is an
  error; the fix is `from self`.
- `T from { name: place, .. }` binds a multi-origin type's origins one by
  one; `-> Match from a, b` lets every origin borrow from either.
- `from` is a contextual keyword only after a type, so `String::from` and
  `impl From<X>` are unaffected.
- Rust lifetimes that do not map onto origins are `FX0201` with a suggested
  `from`: outlives bounds (`'a: 'b`, `T: 'a`), lifetime parameters on traits,
  `for<'a>`. A borrow that elision ties to the wrong origin is `FX0202`; its
  fix suggests `from self.field`.

### 3.3 Borrowing structs and enums

A struct or enum declares no lifetime parameters. Each borrowed position in
a field's type is an origin named by its path: `src` for `src: &str`, `*out`
for the elements of `out: &mut Vec<&str>`, `lx.src` inside a field
`lx: Lexer`, `Ident.0` for a tuple variant `Ident(&str)`. Variance is
Rust's: `&T` covariant; `&mut T`, `Cell`, `RefCell`, `Shared` invariant.

```ferox
pub struct Lexer { src: &str, pos: usize }        // Rust: Lexer<'a> { src: &'a str, .. }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token { Ident(&str), Int(&str), Punct(&str) }   // three origins, all bound together below

impl Lexer {                                       // Rust: impl<'a> Lexer<'a>
    pub fn new(src: &str) -> Lexer { Lexer { src, pos: 0 } }     // elision: from src
    pub fn next_token(&mut self) -> Option<Token from self.src> { .. }
}

impl Iterator for Lexer {
    type Item = Token from Self.src;               // associated types name origins
    fn next(&mut self) -> Option<Self::Item> { self.next_token() }
}

fn two(text: &str) -> Option<(Token, Token)> {
    let mut lx = Lexer::new(text);
    let a = lx.next()?;
    let b = lx.next()?;                            // fine: `a` borrows text, not lx
    Some((a, b))
}
```

- **Sharing an origin.** Fields get distinct origins by default. When one
  field must hold data borrowed from another's origin, write `from` on the
  field: `struct Collector { src: &str, out: &mut Vec<&str from src> }`
  (Rust: `<'s, 'o>` with `&'o mut Vec<&'s str>`). In an enum,
  `Int(&str from Ident.0)` (provisional).
- **Impls.** `impl Lexer` is generic over `Lexer`'s origins. `Self.src`
  names one of them. `-> Self` carries the impl's origins, so a constructor
  `fn new(src: &str) -> Self` is an error: write `-> Lexer` (elision) or
  `fn new(src: &str from Self.src) -> Self`. Builders return `-> &mut Self`.

### 3.4 `'static`, `Owned`, and `dyn`

- `'static` is kept: `&'static str`, string literals, consts.
- `Owned` is a compiler-known auto trait: no borrowed origins (Rust's
  `T: 'static`, accepted and rewritten to `T: Owned`). Work that outlives
  the current function requires it: `spawn_isolated` (7.1), children of a
  scope passed down (7.1), `Box<dyn Trait>` contents.
- `dyn Trait` behind `&`/`&mut` borrows from the reference's origin, as in
  Rust. Everywhere else (`Box`, `Rc`, `Arc`, fields, type arguments) it means
  `dyn Trait + Owned`. Write `Box<dyn Trait from x>` for a borrowing object,
  or `Box<dyn Fn(&Row) -> bool + '_>` in a field (one new origin).

### 3.5 Closures and returned `impl Trait`

Inside a body, closures and iterator chains work as in Rust. A closure that
captures a borrow is a borrowing value; it may be returned or stored only
where its origins are named.

- A returned `impl Trait` borrows exactly what the elided positions in its
  bounds name plus its `from` clause; with neither it is `Owned`. So
  `fn adults(p: &[Person]) -> impl Iterator<Item = &Person>` needs nothing,
  but `fn evens(xs: &[u32]) -> impl Iterator<Item = u32> from xs` and
  `fn over(min: &u32) -> impl Fn(&Person) -> bool from min` must name the
  borrow (`FX0204`, safe fix). This replaces Rust 2024's implicit capture
  and `+ '_`.
- Storing a closure in an `Owned` position (`Box<dyn Fn()>`,
  `spawn_isolated`) requires that it capture no borrows: `move` owned values
  in (`clone()`, `to_string()`), or write `from` (`FX0203`).

### 3.6 What is impossible, and what to write instead

A struct cannot borrow from its own field (`FX0210`):
`struct Doc { text: String, words: Vec<&str from text> }` is rejected.

| Need | Write |
| --- | --- |
| Parsed view of a buffer | Keep owner and view apart: `let text = fs.read_to_string(p)?; let doc = Doc::parse(&text);` |
| Store both together | Indices: `words: Vec<Range<usize>>` and `&self.text[self.words[i].clone()]` |
| Graphs, parent links, cycles | `Arena<T>` with `Id<T>` handles (below) |
| Shared ownership | `Rc<RefCell<T>>` and `Weak`, as Rust; `Arc` across tasks |

```ferox
use std::arena::{Arena, Id};

struct Node { name: String, edges: Vec<Id<Node>> }

let mut g: Arena<Node> = Arena::new();
let a = g.insert(Node { name: "a".into(), edges: vec![] });
let b = g.insert(Node { name: "b".into(), edges: vec![a] });
g[a].edges.push(b);               // a cycle: handles are values, not borrows
g.remove(b);
assert!(g.get(b).is_none());      // stale handle: None, never a dangling read
```

`Id<T>` is a `u32` index plus a `u32` generation. It is `Copy`, `Eq`,
`Hash`, `Owned`, `Pure`, `Send`, and `Sync` (02 §10.9), and also `Ord` and
`Debug` (provisional), so it works as a `HashSet`/`HashMap` key. `Arena`
methods: `new`, `insert(T) -> Id<T>`, `get(id) -> Option<&T>`, `get_mut`,
`remove(id) -> Option<T>`, `get_disjoint_mut([a, b]) -> Option<[&mut T; 2]>`,
`len`, `Index`/`IndexMut` (panic on a stale id); `contains(id) -> bool` and
`iter() -> impl Iterator<Item = (Id<T>, &T)>` (provisional).

## 4. Integers, floats, conversions

### 4.1 Overflow traps in every build

`+ - * / % << >>`, unary `-`, `abs`, and `pow` trap on overflow, division by
zero, `MIN / -1`, and over-wide shifts, in debug **and release**. A trap is a
panic (section 9), raised at the faulting operation. For modular or clamped
arithmetic say so, as in Rust: `wrapping_add`, `checked_add` (→ `Option`),
`saturating_sub`, `overflowing_add`, `Wrapping<T>`, `Saturating<T>`. Hashes,
checksums, and PRNGs must use these.

### 4.2 `as` is lossless only

| Allowed `as` | Examples |
| --- | --- |
| integer to wider integer, same signedness | `u8 as u32`, `i32 as i64` |
| unsigned to strictly wider signed | `u32 as i64` |
| `bool` to any integer; `u8` to `char`; `char` to `u32` and wider | `b as u8`, `c as u32` |
| integer to float when exact | `u16 as f32`, `u32 as f64` |
| `f32 as f64` | |
| `usize`/`isize`, treated as 32 to 64 bits | `u32 as usize`, `usize as u64`; not `u64 as usize` (provisional) |
| fieldless enum to an integer holding every discriminant | `Color::Red as u8` |

Lossy `as` is `FX0301` with these fixes:

| Rust | Ferox |
| --- | --- |
| `x as u8` (may not fit) | `u8::try_from(x)?`; keep the low bits: `let b: u8 = x.truncate();` |
| `i32 as u32` (reinterpret) | `x.cast_unsigned()`; back with `cast_signed()` |
| `n as f64` (`usize`, `u64`, `i64`) | `f64::round_from(n)` (nearest, ties to even) |
| `f as i64` | `i64::try_from(f.trunc())?` (errors on NaN, infinity, out of range) |
| `x as f32` (`x: f64`) | `f32::round_from(x)` |

`truncate()` takes no arguments on integers and gets its target type from
context. `round_from` and float `TryFrom` are provisional names.

### 4.3 Floats

IEEE 754 as Rust; `to_bits()` returns one canonical NaN pattern, so results
match across targets. `f64` is not `Eq` or `Ord`; use `total_cmp`.

## 5. Traits and generics

### 5.1 Power ceiling

Every item signature is annotated; inference is local to a body. Generics
are definition-checked: an instantiation never produces a type error. These
are rejected:

| Not allowed | Rust example | Code | Write instead |
| --- | --- | --- | --- |
| Specialization, overlapping impls | `default fn` | `FX0402` | one impl per type; a newtype |
| Generic associated types | `type Item<'a>` | `FX0403` | owned items or indices |
| Higher-ranked bounds beyond elision, HKTs | `for<'a> Fn(&'a T) -> &'a U` | `FX0404` | `Fn(&T) -> &U` |
| Negative impls, user auto traits | `impl !Send for T` | `FX0405` | a `PhantomData<Rc<()>>` field |
| Impl where-clause not smaller than its head | `impl<T> Foo for T where Vec<T>: Foo` | `FX0406` | bound `T` directly |
| Polymorphic recursion | `fn f<T>(x: T) { f(vec![x]) }` | `FX0407` | an enum or `Box<dyn ..>` |
| Two inherent methods with one name | two `fn new` | `FX0408` | distinct names |

Trait methods are namespaced by trait, as in Rust, so `Display::fmt` and
`Debug::fmt` coexist. A returned `impl Trait` exposes only the auto traits
it names: write `-> impl Iterator<Item = u8> + Send` if callers need `Send`.

### 5.2 Impl placement

An `impl` must live in the module that defines the type, or (for trait
impls) the trait. `impl Display for User` belongs in `User`'s file
(`FX0401`, fix moves the block). Foreign trait for foreign type: use a
newtype, as in Rust.

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
| `Env` (`std::env`) | `args() -> impl Iterator<Item = String>` (program name first), `var(k) -> Result<String, VarError>`, `vars()`, `current_dir() -> io::Result<PathBuf>` | `only(["HOME", ..])`, `args_only()` |
| `Stdin` (`std::io`) | `read_line(&mut String) -> io::Result<usize>`, `lines() -> impl Iterator<Item = io::Result<String>>`, `read_to_string(&mut String)`, `read_to_end(&mut Vec<u8>)`, `is_terminal()` | — |
| `Proc` (`std::process`) | `command(prog) -> Command` (Rust's builder: `.arg`, `.args`, `.env`, `.current_dir`, `.output()`, `.status()`, `.spawn()`), `exit(code: i32) -> !` | `only(["git", ..])` |
| `Tasks` (`std::task`) | `scope`, `spawn_isolated`, `shield`, `checkpoint`, `yield_now`, `fault_point` (7.1) | `limit(n) -> Tasks` |

Calling conventions:

- **Receivers.** Every capability method takes `&self` (provisional), so a
  `&Rand` or `&Stdin` parameter is enough: `rand.gen_range(0..n)`,
  `stdin.read_line(&mut line)`. For many draws, take `let mut rng =
  rand.rng();` once: a `Pcg64` is an ordinary `Pure` value with Rust's
  `&mut self` methods (`next_u64`, `gen_range`, `gen_bool`, `shuffle`).
  `Pcg64::seed_from_u64(n)` needs no capability.
- **Copying and threads.** Capabilities are not `Copy`. They are `Clone` (a
  clone has the same authority; provisional) and `Send + Sync`, so scoped
  children may borrow them. They are not `Pure` (6.6). Pass them by
  reference: `fs: &Fs`.
- **Attenuation** returns a narrower capability of the same type. Only
  `dir` can fail, so only `dir` returns `io::Result`. `only` takes any
  iterable of strings (provisional), so `main` can attenuate to hosts
  computed at run time.
- Every capability error type has a cancellation case whose
  `is_cancelled()` is true (7.2).

### 6.2 `main(root: Root)`

```ferox
fn main(root: Root) -> Result<(), AppError> {
    let data = root.fs.dir("/var/lib/app", Mode::ReadWrite)?;   // Fs below one directory
    let net = root.net.only(["db.internal:5432"]);               // Net to one host
    app::run(&data, &net, &root.clock)?;
    Ok(())
}
```

- `Root` has public fields `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
  `proc`, `tasks`, and `log` (sink configuration, 6.5).
- `main` takes `root: Root` or nothing, and returns `()`,
  `Result<(), E: Debug>`, or `std::process::ExitCode` (`ExitCode::SUCCESS`,
  `ExitCode::FAILURE`, `ExitCode::from(2u8)`), as in Rust. After `Err(e)`
  the program prints `Error: {e:?}` to stderr and exits with 1; to print
  `Display` and the `source()` chain, return `ExitCode` and log the error
  yourself. A panic exits with 101.
- `Root` can be named only in `main`'s signature. `root` cannot be passed,
  borrowed into a call, stored, returned, or captured (`FX0503`); move or
  borrow its fields (`root.fs`, `&root.clock`). Tests cannot call `main`, so
  put the logic in functions that take capabilities.
- `Root::sim(seed)` and everything in `std::sim` exist only in test code
  (`FX0504`, 6.8).

Attenuate in `main` and hand each subsystem only what it needs; review
shows where every capability reaching a function was attenuated.

### 6.3 Files: paths, append, durability

```ferox
impl Fs {                                         // every method takes &self; p: impl AsRef<Path>
    fn read(&self, p) -> io::Result<Vec<u8>>;
    fn read_to_string(&self, p) -> io::Result<String>;
    fn write(&self, p, data: impl AsRef<[u8]>) -> io::Result<()>;   // create or truncate; not durable
    fn write_synced(&self, p, data) -> io::Result<()>;              // durable when it returns
    fn open(&self, p) -> io::Result<File>;                          // read only
    fn create(&self, p) -> io::Result<File>;                        // write; create or truncate
    fn open_with(&self, p, opts: &OpenOptions) -> io::Result<File>; // append, create_new, read+write (provisional name)
    fn sync_dir(&self, p) -> io::Result<()>;                        // make p's entries durable
    fn create_dir_all(&self, p) -> io::Result<()>;
    fn exists(&self, p) -> bool;
    // read_dir, metadata, rename, remove_file, remove_dir_all: as std::fs, returning io::Result
    fn dir(&self, p, mode: Mode) -> io::Result<Fs>;    // Mode::Read | Mode::ReadWrite; fails if p is not a directory
    fn read_only(&self) -> Fs;
}
impl File {        // io::Read, io::Write, io::Seek, also for &File
    fn sync(&self) -> io::Result<()>;                  // D7; `sync_all`, `sync_data` accepted (provisional)
    fn set_len(&self, n: u64) -> io::Result<()>;       // (provisional)
    fn metadata(&self) -> io::Result<Metadata>;
    fn close(self) -> io::Result<()>;
}
```

- **`OpenOptions`** is Rust's builder (`read`, `write`, `append`,
  `truncate`, `create`, `create_new`) without its ambient `open` method
  (`FX0501`): `fs.open_with("kv.log",
  OpenOptions::new().read(true).append(true).create(true))?`.
- **A `File` carries the authority of the `Fs` that opened it**; its methods
  need no other capability.
- **Paths.** An attenuated `Fs` resolves every path beneath its directory.
  An absolute path, or a `..` or symlink that would leave the directory,
  fails with `ErrorKind::PermissionDenied` (provisional kind). `root.fs`
  accepts absolute paths and resolves relative ones against the starting
  working directory (provisional; `/` under simulation). A `Mode::Read` `Fs`
  returns `PermissionDenied` on writes. Missing files give `NotFound`, as in
  Rust. To attenuate to a directory that may not exist yet, call
  `create_dir_all` first.
- **Durability** (D7, D15). Writes are durable only once synced:
  `file.sync()` for a file's contents, `fs.sync_dir(dir)` for the
  directory entries changed by `create`, `rename`, `remove_file`, or a new
  subdirectory. `write_synced` syncs the file and its entry (provisional).
  After a crash, anything not synced may be lost or torn (6.8). An
  acknowledged write is therefore: write, `sync()`, and, the first time a
  file or directory is created, `sync_dir` of its parent.

```ferox
pub struct Log { file: File }

impl Log {
    pub fn open(dir: &Fs) -> io::Result<Log> {
        let file = dir.open_with("log", OpenOptions::new().read(true).append(true).create(true))?;
        dir.sync_dir(".")?;                           // the new entry survives a crash
        Ok(Log { file })
    }
    /// Durable when it returns Ok; acknowledge only after that.
    pub fn append(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.file, "{line}")?;
        self.file.sync()
    }
    /// Complete lines only: a torn last line is ignored.
    pub fn lines(&mut self) -> io::Result<Vec<String>> {
        let mut s = String::new();
        self.file.read_to_string(&mut s)?;
        let done = s.rfind('\n').map_or("", |i| &s[..i]);
        Ok(done.lines().map(String::from).collect())
    }
}
```

### 6.4 No globals; deterministic hashing

Safe code has no mutable globals and no thread-locals: no `static mut`, no
`static` holding atomics, `Mutex`, `OnceLock`, or `Cell`, no
`thread_local!`, no `lazy_static!` (`FX0502`). A `static` must be
`Owned + Sync + Pure`; `const` is as Rust. A lazily built table is allowed
when its builder is an authority-free `fn`:
`static TABLE: Lazy<Table> = Lazy::new(build_table);`
(`std::sync::Lazy`, provisional path). `Lazy<T>` is `Pure` when `T` is,
because forcing it yields the same value in every task. Otherwise build the
value in `main` and pass it down.

`HashMap::new()` uses a fixed hash key, so iteration order is deterministic
(but unspecified). For untrusted keys use
`HashMap::with_hasher(RandomState::from_rand(&rand))`. `Arc` has no
`strong_count`, `weak_count`, `get_mut`, `try_unwrap`, or `into_inner`,
because their results depend on other tasks; `make_mut` stays.

### 6.5 Write-only sinks are not authority

`print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`, `error!`..`trace!`,
`std::io::stdout()`/`stderr()` (write-only `impl Write` handles that always
return `Ok`), and `std::trace::span(name)` (a guard that closes on drop)
write to sinks owned by the current task. They read nothing back, need no
capability, never suspend, and keep a function authority-free.

```ferox
fn decode_user(row: &Row) -> Result<User, DecodeError> {      // still authority-free
    if row.version < 3 {
        warn!("legacy row format for user {}", row.id);
    }
    ..
}
```

- Format arguments are always evaluated, whatever level is enabled, so
  sink configuration never changes results or panics.
- Order within a task is program order; inside `join`/`par_map`, output is
  buffered and emitted in sequential order.
- `main` configures sinks through `root.log` (levels, text or JSON, and
  `root.log.to_file(file)`). Defaults: `println!` to stdout; `eprintln!` and
  `info!` and above to stderr; `dbg!` to stderr in debug builds.
- `dbg!` is an error in release builds (`FX0507`); `println!` in a library
  package is a lint (`FX0508`).
- `Secret<T>` implements neither `Debug` nor `Display`; read it with
  `expose()`.

### 6.6 `Pure` and authority-free functions

`Pure` is a compiler-known auto trait: no authority is reachable from the
type. **Not `Pure`**: the capability types and `File`, `TcpStream`, and other
handles derived from them; synchronization handles (`Shared`, channel ends,
`Scope`, `Task`, `Arc`, `sync::Weak`, atomics, `OnceLock`, and any `Sync`
type whose `&self` API can observe or cause mutation); raw pointers;
`dyn Trait` without `+ Pure`; an unbounded type parameter. **Everything else
is `Pure` exactly when its parts are**, including references, collections,
`Box`, `Rc`, `Cell`, `RefCell`, structs, enums, and closures. `Cell`,
`RefCell`, and `Rc` are `Pure` because they are not `Sync`: their state
never leaves one task. `Arc` is not, because drop timing can reveal
scheduling (D7): authority-free and parallel code shares data by reference
inside a scope instead.

A function or closure is **authority-free** when every parameter and capture
type is `Pure`. It cannot do I/O or observe time, randomness, or other
tasks; its result depends only on its arguments' memory; it never suspends.
Generic code is classified per instantiation; require it with a bound,
`F: Fn(&T) -> U + Pure`. `#[pure]` on a function makes losing
authority-freedom a compile error (`FX0511`).

| Use | Requires |
| --- | --- |
| `join`, `par_map`, `par_for` closures (8) | `Pure + Send`; shared captures `Pure + Sync` |
| `Shared<T>` payloads; `with` closures (7.4) | `T: Send + Pure`; closure `Pure` |
| `static` (6.4) | `Owned + Sync + Pure` |
| contract clauses (10) | authority-free |
| `const fn` | authority-free, no `&mut` parameters |

### 6.7 Network

`Net::get` performs an HTTP(S) GET and returns the body.
`NetError` has `Timeout` (the operation's deadline passed), `Cancelled`,
`Denied` (a host outside `only`), `Refused`, `Reset`, `Dns`, `Status(u16)`
(a non-2xx response), and `Io(io::Error)`; `is_cancelled()` and
`is_transient()` (timeouts, resets, refusals, 5xx) classify them
(provisional beyond `Timeout` and `Cancelled`). `Url::parse(&str) ->
Result<Url, UrlError>`; `Url` has the `url` crate's accessors (`host_str`,
`port_or_known_default`, `path`, `as_str`) and `Display`. `TcpStream`
implements `io::Read` and `io::Write`, also for `&TcpStream`.

### 6.8 Simulation

Every capability has a deterministic simulated version. `Root::sim(seed)`
returns a `SimRoot` (`std::sim`) with `Root`'s capability fields, all
simulated (pass `&sim.fs`, `&sim.net`, `&sim.clock`, `&sim.tasks` to the code
under test), plus `sim.world` (setup), `sim.faults` (fault
injection), `sim.node(name)` (crashable nodes), and inspection methods. The
test body runs as the simulation's main task.

| Item | Effect |
| --- | --- |
| `#[sim_test(seeds = 200)] fn t(sim: SimRoot)` | runs the test once per seed with a fresh `SimRoot`; plain `#[sim_test]` runs 100 seeds |
| `sim.world.file(path, contents)` | create a file (parents included) |
| `sim.world.args(["prog", "a.txt"])`, `env_var(k, v)`, `stdin(bytes)` | program inputs; `args` as `env.args()` returns them |
| `sim.world.http("host:port", \|req\| Reply::ok(body))` | a simulated HTTP peer; `Reply::ok(b).after(d)`, `Reply::status(503)`, `Reply::hang()` |
| `sim.net.drop_all() -> Net` | a `Net` whose every operation times out |
| `sim.faults.net_drop(p)`, `net_duplicate(p)`, `net_delay(min, max)`, `net_reorder(on)` | network faults; under `net_drop` an operation is delayed and fails with `NetError::Timeout` at its deadline (provisional) |
| `sim.faults.partition(&[&a], &[&b])`, `heal()` | cut links between node groups |
| `sim.faults.disk_error("/data", p, io::ErrorKind::Other)` | I/O errors on paths with that prefix |
| `sim.faults.torn_writes(on)` | whether a crash may keep a torn prefix of unsynced writes (provisional default: on, per D15) |
| `sim.faults.crash(&node)`, `restart(&node)`, `at(t, Fault::crash(&node))` | crash or restart now, or schedule any `Fault` at virtual time `t` |
| `sim.faults.fault_point(name, p)` | probability that `tasks.fault_point(name)` returns `true` |
| `sim.node("kv") -> SimNode` | a node with its own disk, address (`"kv"` resolves to it), clock, and tasks |
| `node.start(entry) -> NodeTask<T, E>` | runs `entry: FnOnce(NodeRoot) -> Result<T, E>`, which is `Owned + Send` and captures only `Pure` data; `NodeRoot` has `Root`'s fields |
| `node.fs() -> Fs` | the node's disk, for inspection after a crash |
| `sim.run_for(d)`, `sim.run_until(&node_task) -> Result<T, E>` | drive the world |
| `sim.elapsed() -> Duration`, `sim.records() -> Vec<Record>`, `sim.trace()` | virtual time; sink records (`node`, `level`, `message`); the event trace |

(`SimNode`, `NodeTask`, and `Record` are provisional names.)

Semantics:

- **Time.** Monotonic time starts at zero, wall time at 2000-01-01. Virtual
  time advances only when every task is blocked, then jumps to the next
  timer, so a 30-second timeout test runs in milliseconds. Computation takes
  no virtual time; capability operations take small seed-chosen latencies.
  If no task can ever run again, the run fails with a deadlock report.
- **Order.** Capability operations, `spawn`, `join`, channel operations,
  `Shared::with*`, and checkpoints are scheduling points; the seed picks the
  interleaving, so each seed explores a schedule and replays bit for bit.
- **Errors** have the host's `io::ErrorKind`s.
- **Crashes.** A crash stops a node's tasks at their current scheduling
  points without running destructors. Only synced data survives (6.3);
  unsynced writes and directory entries may be lost or torn, as the seed
  decides. `restart` reruns the entry over the surviving disk. The test body
  itself cannot crash: put crash-tested code in a node.

```ferox
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

`ferox test` reports a failing seed with a replay command
(`ferox test acked_lines_survive_a_crash --seed 1729 --trace`);
`ferox sim <bin> -- args` runs `main` under many seeds as one node.

### 6.9 Ambient Rust calls and their fixes

All are `FX0501`; the fix uses a capability in scope or proposes adding the
parameter.

| Rust | Ferox |
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

### 6.10 Patterns

**Thread a capability.** When a leaf starts needing authority, add the
parameter along the call chain; never reach for a global or `Root`.
`ferox fix --thread Fs --to decode_user` makes that edit along the shortest
path from a holder, test call sites included.

**App-context struct.** For service code, bundle attenuated capabilities
and clients built in `main`, and pass narrow pieces down:

```ferox
pub struct App { db: Db, files: Fs, clock: Clock }   // each already attenuated

impl App {
    pub fn handle(&self, req: &Request) -> Result<Response, AppError> {
        let user = load_user(&self.db, req.user_id)?;
        Ok(Response::ok(render(&user, self.clock.system_time())))   // pure core takes data
    }
}
```

Keep business logic in authority-free functions that take data; pass each
leaf the single capability it uses.

## 7. Concurrency

There is no `async`/`await`, `Future`, `Pin`, or executor choice in library
code. A function suspends only inside a capability or synchronization
operation; authority-free code never suspends. Write blocking-style code.

### 7.1 Scopes and tasks

```ferox
impl Tasks {
    fn scope<T, E>(&self, body: impl FnOnce(Scope<E>) -> Result<T, E>) -> Result<T, E>
        where E: From<TaskError> + Send;
    fn spawn_isolated<T: Send>(&self, f: impl FnOnce() -> T + Send + Owned) -> Result<T, Panicked>;
    fn limit(&self, n: usize) -> Tasks;               // attenuation: at most n live children
    fn shield<T>(&self, f: impl FnOnce() -> T) -> T;   // ignore enclosing cancellation
    fn checkpoint(&self) -> Result<(), Cancelled>;
}
impl<E> Scope<E> {                                    // Copy + Send + Sync: pass by value
    fn spawn<U: Send>(self, f: impl FnOnce() -> Result<U, E> + Send) -> Task<U, E>;
    fn set_deadline(self, at: Instant);
    fn cancel(self);
}
impl<U, E> Task<U, E> { fn join(self) -> Result<U, E>; }
pub enum TaskError { Cancelled, DeadlineExceeded }
pub struct Panicked { pub message: String }          // (provisional)
```

- **Structure.** Tasks are spawned into scopes and cannot outlive them.
  `scope` returns only after every child has finished; scope exit and `join`
  wait, are not checkpoints, and cannot be cancelled. Children may borrow
  anything that outlives the `scope` call, and every capability, `Shared`
  handle, and `Scope` is `Send + Sync`, so `s.spawn(|| Ok(net.get(u, d)))`
  may borrow `net`.
- **Failure.** A child fails when it returns `Err`; the body fails when it
  returns `Err`. The first failure cancels the scope (body, children, nested
  scopes), and `scope` returns that error after every child finishes. Later
  errors go to one `debug!` record. To collect per-item errors without
  cancelling siblings, return them as data:
  `s.spawn(move || Ok(net.get(u, deadline)))` gives
  `Task<Result<Vec<u8>, NetError>, E>`.
- **`join`** returns the child's `Ok` value, or `Err(TaskError::Cancelled
  .into())` if the child failed, panicked, or was cancelled (the scope
  reports the real cause). So `x.join()?` in the body is the normal form.
  Dropping a `Task` without joining is allowed.
- **Panics propagate** as in Rust's `std::thread::scope`: a child's panic
  cancels the scope, and once every child has finished, `scope` panics in
  the calling task with the child's message. A panic in the body cancels and
  joins the children, then continues unwinding. Nothing ever observes data a
  panicking child was mutating.
- **Isolation.** `tasks.spawn_isolated(move || ..)` runs an `Owned` closure
  as a new task, waits for it, and returns `Err(Panicked)` if it panicked;
  it inherits the caller's cancellation (both provisional). It is the only
  place a panic becomes a value (servers: one per request, called from a
  scoped child).
- **Bounded concurrency.** `let tasks = root.tasks.limit(4);` then spawn one
  child per item: scopes from that `Tasks` run at most 4 children at once,
  and `spawn` waits for a free slot (provisional; the wait is not a
  checkpoint).
- **A scope passed down** (`fn serve(bg: Scope<E>, ..)`) outlives the
  function's locals, so its children must own their data (`move` with owned
  values, `Owned`) or borrow what the signature ties to the scope
  (`cfg: &Config from bg`).
- The task, not the OS thread, is the isolation unit: values that never
  leave a task need not be `Send` (an `Rc` inside one task is fine).

```ferox
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
  authority-free code, `Shared::with*`, joins, and scope exit are not.
  Suspended work unwinds through ordinary `Err` returns, and destructors
  run. Operations that consume a value return it inside the error
  (`SendError(v)`).
- **Operation deadlines.** `net.get(url, deadline)` returns
  `NetError::Timeout` when `deadline` passes; `clock.sleep_until(t)` and
  `rx.recv_deadline(t)` likewise. Deadlines are `Instant`s from a `Clock`;
  under simulation they are virtual.
- **Scope deadlines** (D8). `s.set_deadline(t)` lowers the scope's deadline
  (a nested scope uses the earliest of its ancestors'). When it passes, the
  scope is cancelled, but that alone is not a failure: if the body and every
  child still return `Ok`, for example because children turned cancellation
  errors into values, `scope` returns `Ok` with the partial results. If any
  of them returns `Err` after the deadline, the scope fails and `scope`
  returns `Err(TaskError::DeadlineExceeded.into())` (provisional: the
  deadline is reported as the cause). `s.cancel()` behaves the same, with
  `TaskError::Cancelled`. When a scope deadline and an operation deadline
  coincide, the operation sees the cancellation.
- **Soft.** Authority-free computation, `Shared::with*`, and joins are never
  interrupted; cancellation is observed at the next checkpoint.
- **Cleanup.** `tasks.shield(|| ..)` ignores enclosing cancellation; give
  bounded cleanup its own inner scope with a deadline.

### 7.3 Channels

Channels are Rust's `std::sync::mpsc`: `channel()` (unbounded; a lint in
libraries, `FX0608`), `sync_channel(n)`, `Sender: Clone`, `send`,
`try_send`, `recv`, `try_recv`, `recv_deadline(t)`, and a `Receiver` that
iterates until every sender drops. `recv` and `send` on a full
`sync_channel` are checkpoints. Payloads must be `Send`. A `Receiver` has
one consumer; for N workers over a list, use `Tasks::limit` (7.1). To wait
on several sources (Rust's `select!`), send one enum through one channel.

### 7.4 `Shared<T>`

```ferox
impl<T: Send + Pure> Shared<T> {                   // std::sync
    fn new(v: T) -> Shared<T>;                     // no capability needed
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

- `Shared<T>` replaces `Arc<Mutex<T>>`. It is a handle: `clone()` names the
  same cell. It is `Send + Sync` but not `Pure`; children may borrow
  `&Shared<T>`. Payloads are `Send + Pure` (`FX0606`), so a cell never holds
  a capability, channel end, or another `Shared`.
- `with2` and `with_all` acquire all their cells atomically in a global
  order, so they cannot deadlock. If two operands are the same cell, the
  call returns `Err(LockError::Aliased)` without running `f`; test identity
  first with `Shared::same(&a, &b)`.
- `f` must be `Pure`. It therefore captures no capability, channel end, or
  `Shared` handle, so it cannot do I/O, suspend, or nest another `with`
  (`FX0603`). Logging is allowed; printing is a lint (`FX0607`). `return`
  and `?` inside `f` leave `f`, as in any closure.
- Acquisition waits FIFO per cell and is not a checkpoint: it still works in
  a cancelled scope.
- A panic in `f` poisons every cell it held; later calls return
  `Err(LockError::Poisoned)` without running `f`. `reset(v)` clears it.
- Atomics (`std::sync::atomic`, sequentially consistent only) exist for
  library authors and are not `Pure`. `Mutex` and `RwLock` live in
  `std::sync::raw` and need the `unsafe` grant.

```ferox
use std::sync::{LockError, Shared};

#[derive(Debug, Error)]
pub enum BankError {
    #[error("insufficient funds")]
    Insufficient,
    #[error("lock: {0}")]
    Lock(#[from] LockError),
}

pub struct Account { pub balance: u64 }

pub fn transfer(from: &Shared<Account>, to: &Shared<Account>, amount: u64) -> Result<(), BankError> {
    if Shared::same(from, to) {
        return Ok(());                                  // otherwise Err(Aliased)
    }
    Shared::with2(from, to, |a, b| {
        a.balance = a.balance.checked_sub(amount).ok_or(BankError::Insufficient)?;
        b.balance += amount;                            // traps on overflow: keep totals bounded
        Ok(())
    })?                                                 // LockError -> BankError via #[from]
}

pub fn total(accounts: &[Shared<Account>]) -> Result<u64, LockError> {
    Shared::with_all(accounts, |xs| xs.iter().map(|a| a.balance).sum())   // one consistent snapshot
}
```

### 7.5 Runtimes

The binary's manifest picks the runtime (`[bin] runtime = "threads"`, the
default, or `"blocking"`); tests and `ferox sim` use the simulator.
Libraries never choose and behave the same under all of them.

### 7.6 Porting async Rust

| Rust | Ferox |
| --- | --- |
| `async fn f(..) -> T` | `fn f(net: &Net, ..) -> T` |
| `x.await` | `x` (`FX0601`, safe fix deletes `.await` and `async`) |
| `#[tokio::main] async fn main()` | `fn main(root: Root)` |
| `tokio::spawn`, `thread::spawn` | `tasks.scope(\|s\| { s.spawn(..); .. })` (`FX0602`) |
| `std::thread::scope` | `tasks.scope` (children return `Result`) |
| `tokio::join!(a, b)` | two `spawn`s, two `join`s |
| `tokio::time::timeout(d, f)` | an operation deadline, or `s.set_deadline(clock.now() + d)` |
| `buffer_unordered(4)`, `Semaphore` | `tasks.limit(4)` |
| `tokio::select!` | one channel carrying an enum (`FX0605`) |
| `Arc<Mutex<T>>`, `m.lock().unwrap()` | `Shared<T>`, `m.with(\|x\| ..)?` (`FX0604`) |
| `catch_unwind`, `thread::spawn(..).join()` for isolation | `tasks.spawn_isolated(move \|\| ..)` |
| `rayon::join`, `par_iter().map()`, `.reduce(..)` | `join`, `par_map`, then a sequential fold (section 8) |

## 8. Parallelism

Deterministic fork-join for authority-free code:

```ferox
use std::par::join;

let (a, b) = join(|| parse(x), || parse(y));
let lens: Vec<usize> = words.par_map(|w| w.len());     // == words.iter().map(..).collect()
pixels.par_for(|p| *p = p.saturating_add(10));         // over &mut [u8], disjoint
let total: u64 = docs.par_map(|d| score(d)).iter().sum();         // reduce sequentially
```

- `join` is in `std::par`. `par_map`, `par_for`, and `par_sort` (stable) are
  inherent slice methods and need no import.
- Closures must be `Pure + Send` (`FX0610`); shared captures must be
  `Pure + Sync` (`FX0611`: not `&RefCell`, `&Rc`, `&Arc`, or a capability).
  Disjoint `&mut` splits (`split_at_mut`) are fine.
- `join`, `par_map`, and `par_for` equal their sequential versions,
  including sink output order. Under simulation they run on real cores.
- To reduce, `par_map` the expensive part and fold the results
  sequentially; `par_reduce` is v1 (Appendix A).
- If several branches panic, the first in sequential order is reported,
  buffered sink output from later branches is discarded, and the panic
  propagates in the enclosing task.

## 9. Errors and panics

- `Result<T, E>`, `?`, `From` conversion, `Box<dyn Error>`, `Option` and
  `?`, `unwrap`/`expect`, `map_err`, `ok_or`: as Rust.
- Results are must-use: an ignored `Result` is an error (`FX0702`); write
  `let _ = r;` to discard on purpose (provisional: Rust warns).
- `#[derive(Error)]` replaces `thiserror` (2.2).
- **Panics** (`panic!`, overflow, out-of-bounds, failed `assert!`, `unwrap`
  on `None`, a violated contract) unwind the current task and run
  destructors. A panicking child makes its scope panic (7.1). There is no
  `catch_unwind` (`FX0701`); isolate risky work with `tasks.spawn_isolated`.
  A panic in the main task exits with code 101.

## 10. Contracts

v0 contracts are executable: one spec, checked at run time in debug and test
builds, erased in release, and used to drive property tests.

### 10.1 Clauses

| Clause | Where | Meaning |
| --- | --- | --- |
| `requires e` | after a fn signature (and `where`) | precondition, checked at entry; a failure blames the caller |
| `ensures e` | after a fn signature | postcondition, checked at each return; a failure blames the callee |
| `invariant e` | between a loop header and its body | checked before the first iteration, after each, and at exit |

- **One expression per clause.** `requires a, b` is an error whose safe fix
  splits it into two clauses; repeated clauses are conjoined in order.
- **In `ensures`**, `result` is the return value (a contextual keyword) and
  `old(e)` is `e` evaluated at entry. A `&mut` parameter `v` (or `*v`)
  means its value at return; `old(*v)` its value at entry. `old` snapshots
  only what the clause reads and never calls `Clone`.
- **Clause expressions** are ordinary Rust boolean expressions that must be
  authority-free: they may call any function or trait method whose arguments
  are `Pure` (`xs.len()`, `x.cmp(y)`, your own `is_sorted(xs)`), never a
  capability or `Shared`. Type parameters need no `Pure` bound; at an
  instantiation with a non-`Pure` type, a clause that calls its methods is
  skipped and reported as unchecked (provisional).
  `==`, `<`, and arithmetic mean exactly what they mean in code, including
  overflow traps; widen (`a as u128 + b as u128`) when a sum might overflow.
- **Added syntax**, clauses only: `a ==> b` (implication, lowest precedence,
  right-associative); `forall |i: T| guard ==> p` and
  `exists |i: T| guard && p`, where `guard` bounds the binder as
  `lo <= i && i < hi` (an unsigned binder implies `0 <= i`) or
  `c.contains(&i)`. An unbounded quantifier is an error. A quantifier body
  extends as far right as possible, so write
  `result == (exists |i: usize| i < xs.len() && xs[i] == x)`. Rust's
  `(0..n).all(|i| ..)` works too.
- **Visibility.** A `pub` function's contract may call only `pub` functions
  (Rust's private-in-public rule; provisional for v0).
- Trait methods may carry contracts; every impl is checked against them.

```ferox
pub fn max_index(xs: &[i64]) -> usize
    requires xs.len() > 0
    ensures result < xs.len()
    ensures forall |i: usize| i < xs.len() ==> xs[result] >= xs[i]
{
    let mut best = 0;
    for i in 1..xs.len()
        invariant best < i
    {
        if xs[i] > xs[best] { best = i; }
    }
    best
}

pub fn push_all(dst: &mut Vec<i64>, src: &[i64])
    ensures dst.len() == old(dst.len()) + src.len()
    ensures forall |i: usize| i < src.len() ==> dst[old(dst.len()) + i] == src[i]
{
    for x in src { dst.push(*x); }
}
```

In a `for x in a..b` loop, an invariant sees `x` as the next iteration's
index: `x == a` before the first iteration and `x == b` at exit.

### 10.2 Property tests

- A `#[test]` function with parameters is a property test: `ferox test`
  generates and shrinks inputs. Generated types: integers, floats, `bool`,
  `char`, `String`, `Vec`, arrays, tuples, `Option`, `Result`, maps, sets,
  and structs and enums whose fields can all be generated, or that
  implement `Arbitrary`.
- A `seed: u64` parameter is an ordinary generated input that combines with
  the others; for one fresh simulator per seed, use `#[sim_test]` (6.8).
- `requires` on a test filters and guides generation. A run with fewer than
  100 accepted cases fails as `vacuous`; an early `return` counts as a pass,
  so prefer `requires`. A rare condition (a random `Vec` is seldom sorted)
  makes the test vacuous: construct such inputs instead (`xs.sort()`).
- `ferox test` also calls each authority-free function that has `ensures`
  on generated inputs that satisfy its `requires` (provisional for v0).

```ferox
#[test]
fn midpoint_stays_in_range(lo: u32, hi: u32)
    requires lo <= hi
{
    let mid = lo + (hi - lo) / 2;
    assert!(lo <= mid && mid <= hi);
}
```

### 10.3 Contracts in review

`ferox review` shows each public item's contract text. Weakening or
removing a contract of a public item, like widening authority, needs human
approval (11). If a contract or `#[protected]` test looks wrong, stop and
report it; never weaken it to get a green check.

## 11. Unsafe, trust holes, and review

| Hole | Spelling | Rule |
| --- | --- | --- |
| Unsafe code | `unsafe { .. }`, `unsafe fn`, `unsafe impl Send`/`Sync`/`Pure` | needs the package's `unsafe` grant (`FX0901`); each block needs a `// SAFETY:` comment (`FX0902`, provisional: error) |
| `std::sync::raw` locks, weaker atomic orderings | — | `unsafe` grant |

`ferox review <base>` prints the trust diff: public signatures, declared
authority (the capability types each public item can reach through its
parameters), contracts, trust holes, and grants, so a reviewer sees what
changed in what code *may do* before reading how. Changes that need a human
approval, recorded outside the agent's write scope and verified by CI:
widening authority, weakening or removing a contract, adding a trust hole,
and changing a `#[protected]` test. Never make one of these to get a green
check.

## 12. Modules and packages

### 12.1 Files are modules

- `src/main.fx` is a binary root, `src/lib.fx` a library root. Every `.fx`
  file under `src/` is a module; the directory tree is the module tree
  (`src/net/http.fx` is `crate::net::http`), with an optional `src/net.fx`
  beside `src/net/`; no `mod.fx`.
- You do not write `mod foo;`. If present, `mod foo;` is a no-op and
  `pub mod foo;` sets visibility; a file module without one is private to
  its parent. `mod foo;` with no file is `FX0910`; `#[path]` is `FX0911`.
  (provisional)
- `ferox check path.fx` checks one file as a crate root (provisional), for
  single-file programs and exercises.

### 12.2 Imports

- `use` as Rust: globs, `as`, nested groups, `pub use`. `extern crate` is
  `FX0912`.
- **Unused imports are errors** (`FX0913`, safe fix). **Import cycles
  between modules are errors** (`FX0914`): put shared types in leaf modules;
  a child's `use super::*` is fine while the parent does not import the
  child.

### 12.3 Prelude and std paths

The prelude is Rust's plus `Root`, `Pure`, `Owned`, `Secret`, and the
derives `Error`, `Encode`, `Decode` (provisional). Import everything else:

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
| `Shared`, `LockError`, `Lazy`, `mpsc` | `std::sync` |
| `join` | `std::par` |
| `Arena`, `Id` | `std::arena` |
| `Cow` | `std::borrow` |
| `json` (`encode`, `decode`, `json::Error`) | `std::json` |
| `span` | `std::trace` |
| `SimRoot`, `NodeRoot`, `SimNode`, `NodeTask`, `Faults`, `Fault`, `Reply`, `Record` | `std::sim` (test code only) |
| `Ordering`, `HashMap`, `BTreeMap`, `PathBuf`, ... | as Rust |

`Faults` is the type of `sim.faults`; you rarely name it, and there is no
`Faults { .. }` literal.

### 12.4 Manifest and grants

```toml
# ferox.toml (provisional file name and keys)
[package]
name = "acme/linecount"          # names are namespaced
version = "0.1.0"
grants = ["unsafe"]              # this package's own escape hatches

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
  capability types a function's parameters and captures can reach); a
  violation is an error (`FX0516`), and policy changes appear in the trust
  diff.

- Escape hatches are granted per dependency. `ferox.lock` is on by default,
  with content hashes, and records each package's public authority and
  trust holes; an upgrade that widens either fails until a human approves.
- The client rejects a dependency name that does not exist; it never
  guesses a near name.
- Ferox packages have no `build.rs`, install hooks, or code that runs at
  fetch or build time (`FX0915`). Code generators are programs you run
  explicitly (`ferox run --bin gen`); commit their output.

## 13. Toolchain loop

| Command | Use |
| --- | --- |
| `ferox check` | One verdict plus NDJSON diagnostics: stable codes, exact and related spans, fixes marked `safe` or `suggest`. |
| `ferox fix` | Apply safe fixes; `--thread <Cap> --to <fn>` threads a capability. |
| `ferox test [filter] [--seed N] [--trace]` | Tests, property tests, and sim tests: diff, panic span, seed, shrunk input, replay command, event trace. |
| `ferox sim <bin> -- args` | Run `main` under many seeds. |
| `ferox eval '<expr>'` | Evaluate an expression against the package in the interpreter. |
| `ferox explain FX0201` | Explanation with a before/after example. |
| `ferox review <base>` | Trust diff (11). |
| `ferox fmt` | The one canonical format; rewrites legacy lifetime spellings to `from`. |

**Typed holes**: `todo!()` or `_?name` in expression position. `ferox check`
checks the rest and reports each hole's expected type, bindings in scope,
and candidate functions; use holes to build a function top-down.

## 14. Rust habits and their fixes

| Rust habit | Ferox | Code |
| --- | --- | --- |
| `fn f<'a>(x: &'a str, y: &str) -> &'a str` | `fn f(x: &str, y: &str) -> &str from x` | accepted; `ferox fmt` rewrites |
| `struct P<'a> { s: &'a str }`, `impl<'a> P<'a>`, `P<'_>` | `struct P { s: &str }`, `impl P`, `P` | accepted; rewritten |
| `T: 'static` | `T: Owned` | accepted; rewritten |
| `'a: 'b`, `trait Foo<'a>`, `for<'a>` | a `from` clause, or restructure | `FX0201` |
| `fn new(s: &str) -> Self` in a borrowing type's impl | `-> Lexer`, or `s: &str from Self.src` | error with both fixes |
| `-> impl Iterator<Item = u32> + '_` | `-> impl Iterator<Item = u32> from xs` | `FX0204` safe |
| `Box<dyn Fn()>` capturing a borrow | `Box<dyn Fn() from x>`, or move owned data in | `FX0203` |
| Self-referential struct | indices, `Arena`, or owner and view apart | `FX0210` |
| `macro_rules!`, proc macros | functions, generics, generators | `FX0101` |
| `#[derive(Serialize, Deserialize)]`, `thiserror::Error` | `Encode`, `Decode`, `Error` | `FX0102` safe |
| `debug_assert_eq!`, `include_str!`, `env!` | `debug_assert!(a == b)`, a generated `const` | `FX0103` |
| `x as u8` (lossy), `n as f64` (`usize`) | `u8::try_from(x)?`, `x.truncate()`, `f64::round_from(n)` | `FX0301` |
| Relying on release-mode wrapping | `wrapping_add`, `Wrapping<T>` | run-time trap |
| `impl Display for T` in another module | move it to `T`'s module | `FX0401` |
| `std::fs::read`, `OpenOptions::open`, `env::args`, `Instant::now`, `thread_rng` | capability methods (6.9) | `FX0501` |
| `static COUNTER: AtomicUsize`, `OnceLock`, `lazy_static!`, `thread_local!` | values built in `main`; `Lazy` with a pure builder | `FX0502` |
| `fn helper(root: &Root)` | pass attenuated capabilities | `FX0503` |
| `Root::sim` outside tests | real capabilities from `main` | `FX0504` |
| `dbg!` left in code; `println!` in a library | remove, or `debug!`; return data or log | `FX0507`, `FX0508` |
| `async fn`, `.await`, `#[tokio::main]` | plain `fn` taking capabilities | `FX0601` safe |
| `thread::spawn`, `tokio::spawn` | `tasks.scope(\|s\| s.spawn(..))` | `FX0602` |
| I/O or a nested `with` inside a `Shared::with` closure | do it before or after | `FX0603` |
| `Arc<Mutex<T>>` + `.lock().unwrap()` | `Shared<T>` + `.with(\|x\| ..)?` | `FX0604` |
| `tokio::select!` | a channel of an enum, deadlines | `FX0605` |
| `Mutex<Sender<T>>`, a `Shared` inside a `Shared` | payloads are `Send + Pure`; pass handles separately | `FX0606` |
| Capturing a capability, `&RefCell`, or `Arc` in `join`/`par_map` | compute first, borrow `Pure + Sync` data | `FX0610`, `FX0611` |
| `catch_unwind` | `tasks.spawn_isolated` | `FX0701` |
| Ignoring a `Result` | `?`, handle it, or `let _ =` | `FX0702` |
| `requires a, b` | `requires a` `requires b` | safe fix |
| `unsafe` without a grant or `// SAFETY:` | add the grant; write the comment | `FX0901`, `FX0902` |
| `mod foo;` for a missing file, `#[path]` | create `foo.fx`; move the file | `FX0910`, `FX0911` |
| Unused `use`; `extern crate foo;` | delete | `FX0913`, `FX0912` safe |
| `a.fx` uses `b`, `b.fx` uses `a` | move shared items to a third module | `FX0914` |
| `build.rs` | a generator you run; commit its output | `FX0915` |

## 15. Example programs

### 15.1 CLI: arguments and files

`lc [--min N] FILE...` counts lines with at least `N` characters per file.

```ferox
// src/main.fx
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

```ferox
// src/main.fx
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

```ferox
// src/search.fx
use std::cmp::Ordering;

/// Public, so public contracts may call it.
pub fn is_sorted<T: Ord>(xs: &[T]) -> bool {
    xs.windows(2).all(|w| w[0] <= w[1])
}

/// Like `slice::binary_search`.
pub fn binary_search<T: Ord>(xs: &[T], target: &T) -> Result<usize, usize>
    requires is_sorted(xs)
    ensures match result {
        Ok(i) => i < xs.len() && xs[i] == *target,
        Err(i) => i <= xs.len()
            && (forall |j: usize| j < i ==> xs[j] < *target)
            && (forall |j: usize| i <= j && j < xs.len() ==> xs[j] > *target),
    }
{
    let (mut lo, mut hi) = (0, xs.len());
    while lo < hi
        invariant lo <= hi && hi <= xs.len()
    {
        let mid = lo + (hi - lo) / 2;                   // (lo + hi) / 2 could overflow
        match xs[mid].cmp(target) {
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
            Ordering::Equal => return Ok(mid),
        }
    }
    Err(lo)
}

pub fn insert_sorted(xs: &mut Vec<i64>, x: i64)
    requires is_sorted(xs)
    ensures is_sorted(xs)
    ensures xs.len() == old(xs.len()) + 1
    ensures xs.contains(&x)
{
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

The `ensures` of `binary_search` uses `==` and `<` on `T`, which are `Eq`
and `Ord` exactly as in code. In debug and test builds each call checks the
quantified `ensures` in linear time; release builds erase it.

## 16. Contradictions found

### 16.1 Chapters that still describe draft 2.1

01 draft 3 records these v0 changes, but the chapters have not caught up;
this reference follows 01:

1. `Frozen` is removed; `Pure + Sync` replaces it (02 §8.5, §9; 03 §3, §8).
2. The `lock` keyword and `sync::lock_all` are replaced by `Shared::with`
   and `with2` (03 §6.5, 02 §8.4, 04 §4.4). `with_all` (7.4) is this
   reference's provisional rename of `lock_all`.
3. A child's panic propagates through its scope, `spawn_isolated` is the
   only recovery point, and the moved-out rule is gone (02 §8.3, 03 §6.2).
   `TaskError` therefore loses 03's `Panicked(String)` variant.
4. v0 contracts are executable only (section 10). There `*v` of a `&mut`
   parameter in `ensures` means the value at return, which 04 §1.2 rule 4
   makes an error; it becomes an error again when `final` arrives in v1.
5. v0 overflow traps are precise; 04 §3.2 and 03 still assume deferred
   traps.

### 16.2 Contradictions between documents

1. **`par_reduce`.** The v0 scope note relayed during this revision keeps
   `par_reduce` in v0 with an unspecified grouping; 01's tier table puts it
   in v1. This reference follows 01.
2. **`Arc` and `Pure`.** 01 D7 makes `Arc` a synchronization handle (not
   `Pure`); 02 §9 and 03 §3.1 make `Arc` `Pure` and `Frozen` when its
   contents are. This reference follows 01.
3. **Sink arguments.** 01 D7 says format arguments are always evaluated;
   03 SINK-3 says only when the level is enabled. This reference follows 01.
4. **`#[pure]`.** 03 §3.2 settles it (`FX0511`); 01 still lists it as open
   question 4. Kept here.
5. **Scope deadlines.** D8 says a deadline fails the scope only if a child
   returns the cancellation as its error, so partial results are possible;
   03 TASK-4 and its §12.2 example fail the scope with `DeadlineExceeded`
   even when the body returns `Ok`. This reference follows D8 (7.2).
6. **File durability API.** D7 names `File::sync` and `Dir::sync_dir` and
   promises append; 03 §1.1 has `File::sync_all` and `Fs::write_synced`, but
   no append, no directory sync, and no `Dir` type. This reference uses
   `file.sync()` (accepting `sync_all`/`sync_data`), `Fs::sync_dir`, and
   `Fs::open_with` (6.3).
7. **Torn writes.** D15 says unsynced writes "may be lost or torn, as the
   seed decides"; 03 §9.4 makes tearing a fault enabled by
   `torn_writes(on)`. This reference keeps the knob, default on.
8. **Spec equality** (v1). D10 calls spec `==` "model equality", which 04
   §1.4 defines as view equality, while 04 §1.3 makes `==` structural and
   spells view equality `a@ == b@`. They differ for types with a custom
   `View`.
9. **`Pure` in generic spec fns** (v1). 04 §1.5 rule 3 and 03 §3.5 require
   `Pure` parameters, but 04's own `le<T: Ord>` and `sorted_by<T: Ord>`
   omit the bound. Resolved by the new 04 §1.7.
10. **`lock` aliasing.** 02 §8.4 said a duplicate operand panics and `lock`
    returns `Result<R, Poisoned>`, against D8 draft 2.1 and 03 LOCK-2. Fixed
    in 02 §8.4.
11. **Names.** 02 §8.2 calls the task handle `Handle<T>` and the scope
    error `ScopeError` (marked illustrative); 03 uses `Task<U, E>` and
    `TaskError`. The previous reference called the arena handle `Handle<T>`;
    02 §10.9 says `Id<T>`, adopted here.
12. **Node entry results.** 03 §9.5 types a node entry as
    `-> Result<(), E>`, but 03 §12.3 reads a value from
    `run_until(&client)`. This reference uses `-> Result<T, E>` (6.8).

## Appendix A. Later tiers (not in v0)

From the tier table of 01 (draft 3). The chapters specify most of these.

**v1:**

- **Proofs** (04): the `prove` mode and the `assert`/`test`/`prove` uses and
  clause classes; `spec fn`, `proof fn`, `#[spec]`, `ghost let`,
  `proof { }`, `assume`, `reveal`; views (`View`, `x@`) and spec types
  (`int`, `nat`, `Seq`, `Set`, `Map`, `Multiset`); `final`; `decreases` and
  `diverges`; type invariants; trait laws and abstract spec fns (`eq_spec`,
  `cmp_spec`, `le`); closure contracts (`f.requires`); triggers, `by(..)`,
  `#[opaque]`, `#[trusted]`; proof status and trust closures; `ferox verify`.
- **Authority**: exercised-authority tracking, the least-authority lint
  `FX0510`, and effect reporting for sinks; FFI (`extern`, the `ffi` grant,
  `#[trusted_pure]`, `#[sim_model]`, `ThreadBound`); imported Rust crates
  (`[rust-dependencies]`, symbol scan).
- **Runtime and backends**: the evented runtime and stackless lowering; the
  C-emitting release backend; production record and replay (D17).
- **Parallelism**: `par_reduce` with its canonical grouping, `Combine`,
  `Monoid`, `Sum`/`Product`/`Min`/`Max`.
- **Memory**: field-borrow declarations (`&mut self.{pos}`).

**Later:** the Bend tandem and `Accel` (D16); deferred overflow traps;
automatic parallelization; `Shape` reflection; Polonius-precision borrows;
interactive-proof export; snapshot-and-branch simulation.

**Removed** (01 draft 3): `Frozen`, the `lock` keyword, `par { }`, and the
moved-out rule. Not yet tiered by 01, and left out of v0 here: codec formats
other than JSON, the codec plug-in API, and the `flatten`,
`deny_unknown_fields`, and `try_from` keys; `pct` scheduling and
`--minimize` in the simulator.
