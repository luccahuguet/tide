# Tide for Rust Programmers

Status: the agent-facing reference for the **v0 core** of Tide, as the
tier table of `design/01-design.md` (draft 3.7, the decision record) defines
it. It summarizes 01 and the normative chapters `02-memory.md`,
`03-authority-concurrency.md`, and `04-contracts.md`. Appendix A lists what
is not in v0; section 16 lists where the documents still disagree and which
reading this reference takes. Syntax is provisional until the E1 gate (D0).
Budget: under 25,000 tokens (D14; CI counting is planned). Measured on
this text, 2026-10-19: 39,330 tokens, over budget (01 open question 8).
Method: input tokens that `claude -p --model claude-opus-5-5 --tools ""
--system-prompt "Reply with OK." --output-format json` reports with this
file as the prompt, minus those for the prompt `.`.

**(provisional)** marks a detail no normative document decides; this
reference picks the most Rust-like option consistent with the decisions.
Diagnostic codes (`FX0201`, ...) are placeholders.

## 0. How to use this document

You know Rust. Write Rust. A Tide program is a valid Rust 2024 crate: Tide's
checker admits it, then a pinned nightly rustc builds it against Tide's std
(D18). Meaning changes in a few places, and every Rust habit that would mean
something different fails loudly with a stable code and, usually, a
machine-applicable fix. Section 1
lists what is unchanged, sections 2–11 the differences, 12.3 every std
import path, 14 the habit table, and 15 complete programs.

The differences you will hit first:

1. **No ambient I/O.** Files, network, clock, randomness, environment,
   stdin, processes, and task spawning are methods on capability values that
   `main` receives in `root: Root` and passes down as parameters.
   `println!` and `info!` still work anywhere.
2. **No `async`.** Write blocking-style code; concurrency is structured
   scopes from the `Tasks` capability.
3. **Lifetimes are Rust 2024's**, except lifetime parameters on traits,
   explicit `for<'a>`, and generic associated types (3.2).
4. **Integers trap on overflow in every build**, and `as` only does lossless
   conversions.
5. **Shared mutable state is `Shared<T>`** with closure methods
   (`cell.with(|x| ..)`), not `Arc<Mutex<T>>`.
6. **Only admitted library items.** `core`, `alloc`, and std methods outside
   Tide's allowlist are errors (6.9): write `math::sin(x)`, not `x.sin()`.

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
  auto-ref, drop order, Rust's drop check, and Rust 2024's lifetimes,
  minus three forms (section 3).
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
  arrays, tuples, `Option`, `Result`, `Vec`, `Box`, `Rc`, `Arc` (created
  with `&Tasks`, minus count observers, 6.4), `Weak`, `Cell`, `RefCell`, `VecDeque`, `HashMap`,
  `HashSet`, `BTreeMap`, `BTreeSet`, `BinaryHeap`, `Cow`, `Path`, `PathBuf`,
  `Duration`, `Instant`, `SystemTime`, `std::cmp`, `std::ops`, `std::mem`,
  `std::fmt`, `std::convert`, and the `std::io` traits and types (`Read`,
  `Write`, `Seek`, `BufRead`, `BufReader`, `BufWriter`, `io::Error`,
  `io::ErrorKind`, `io::Result`). What changes is how you *obtain* I/O
  objects (from capabilities, section 6) and the default hasher (6.4).
- **Testing**: `#[test]` (Tide's std supplies it, so a test may take
  parameters, 10.2), `#[should_panic(expected = "..")]`,
  `#[cfg(test)] mod tests { use super::*; .. }`, the `assert!` family.

Source files end in `.rs`, and modules are declared with `mod` (12.1).

## 2. Surface: `!` forms, derives, attributes

### 2.1 The fixed built-in `!` forms

Users cannot define macros. Tide's std supplies exactly these, with Rust's
spelling and meaning; the last row is Tide's own forms:

| Form | Tide notes |
| --- | --- |
| `print!`, `println!`, `eprint!`, `eprintln!` | Write to the task's stdout/stderr sink; need no capability (6.5). Lint in library packages. |
| `format!`, `write!`, `writeln!`, `vec!`, `matches!` | As Rust, including inline `{name}` arguments. |
| `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` | As Rust. |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | As Rust. `todo!()` is also a typed hole (13). |
| `dbg!` | As Rust. Error in release builds (`FX0507`). |
| `error!`, `warn!`, `info!`, `debug!`, `trace!` | Write-only log sink (6.5). No `use` needed; `log::info!` and `tracing::info!` are accepted spellings (provisional). |
| `invariant!`, `hole!` | Loop invariant (10.1), rewritten by its function's attribute, not a macro you import; typed hole (13). |

Anything else ending in `!` is `FX0103`, including `debug_assert_eq!`
(write `debug_assert!(a == b)`), `concat!`, `thread_local!`,
`lazy_static!`, and every crate macro; `include_str!`, `env!`, and
`option_env!` are `FX0520` (6.9). `macro_rules!` and
procedural macros are `FX0101`. Instead of a macro, write a generic function
or trait; for test tables, loop over a `const` array; for generated code or
embedded data, run a generator program yourself and commit its `.rs` output.

```rust
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
All are in the prelude, and so is the `Error` trait itself (12.3); remove
`use thiserror::Error;`, `use std::error::Error;`, and `use serde::..`.

- **`Error`** has `thiserror` semantics, built in: `#[error("..")]` on a
  struct, an enum, or each variant generates `Display`; `#[from]` on a
  single-field variant generates `From` and, as in thiserror, also marks the
  field as the `source`; `#[source]` (or a field named `source`) sets
  `Error::source`; `#[error(transparent)]` forwards both. If you print the
  `source()` chain, leave the source out of each message, or it prints
  twice.
- **`Encode`/`Decode`** are compiler-generated serialization. v0 supports
  JSON only: `std::json::encode(&v) -> Vec<u8>` and
  `std::json::decode::<T>(&bytes) -> Result<T, json::Error>`. Keys:
  `#[codec(rename = "..")]`, `#[codec(rename_all = "camelCase")]`,
  `#[codec(default)]`, `#[codec(skip)]`. Spellings and defaults are serde's
  (provisional): a missing `Option` field decodes as `None` without
  `#[codec(default)]`, and enums are externally tagged
  (`{"Set":{"key":"a"}}`, `"Clear"`).
- `Serialize`/`Deserialize`/`thiserror::Error` are `FX0102` with safe fixes.

```rust
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
- Tide attributes: `#[error]`, `#[from]`, `#[source]`, `#[codec]`,
  `#[tide::main]` (6.2), `#[pure]` (6.6), `#[sim_test]` (6.8),
  `#[requires]`, `#[ensures]`, `#[invariants]` (10.1), `#[protected]` and
  `#[trusted_pure]` (11). They are toolchain proc macros and their derives'
  helper attributes, not user macros.
- `#[tokio::main]`, `#[async_trait]`, `#[path]`, `#[macro_export]`, and
  `#[proc_macro*]` are errors with fixes.

## 3. Lifetimes

Tide has Rust's borrow checker and Rust 2024's lifetimes: syntax, elision,
outlives bounds, `+ use<..>`, implied bounds, object-lifetime defaults, and
return-position `impl Trait` capture all mean what they mean in Rust 2024.
Only lifetime parameters on traits, explicit `for<'a>`, and generic
associated types are left out (3.2). Each lifetime names an **origin**, the
set of loans a borrow may come from.

### 3.1 Elision is Rust's

Rust's elision rules apply unchanged: with one input lifetime, every elided
output position gets it; with `&self` or `&mut self`, every elided output
position gets `self`'s; otherwise an elided output lifetime is an error. A
borrowing type in a signature elides its lifetimes as `'_` (`Token<'_>`);
the bare name (`Token`) is also accepted, as in Rust. In an impl header,
write `impl Collector<'_, '_>`; the bare `impl Collector` is an error, as in
Rust.

```rust
fn first_word(s: &str) -> &str                          // borrows s
fn get(&self, key: &str) -> Option<&V>                  // borrows self
fn split_csv(line: &str) -> Result<Vec<Cow<'_, str>>, CsvError>   // Cow borrows line
fn lex(src: &str) -> Vec<Token<'_>>                     // each Token borrows src
```

In `const` and `static` types, elided lifetimes are `'static`:
`const CASES: [(&str, &[&str]); 2] = ..;`.

### 3.2 Named lifetimes: where elision is not enough

| Need | Write |
| --- | --- |
| result borrows from either input | `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str` |
| result borrows from one input only | `fn pick<'a>(a: &'a str, b: &str) -> &'a str` |
| a token that outlives the `&mut self` call | `fn next_word(&mut self) -> Option<&'a str>` in `impl<'a> Lexer<'a>` |
| what a borrowing value borrows, not `&self` | `fn text(&self) -> &'a str` in `impl<'a> Token<'a>` |
| an element of a slice of borrows | `fn longest<'a>(words: &[&'a str]) -> &'a str` |
| map keys borrowed from an argument | `fn add<'a>(m: &mut HashMap<&'a str, u32>, w: &'a str)` |
| results from two inputs | `fn merge<'a>(a: &'a str, b: &'a str) -> Vec<&'a str>` |
| an associated type that borrows | `type Item = Result<Token<'a>, LexError>;` in `impl<'a> Iterator for Lexer<'a>` |
| a value with two independent borrows | `fn find<'n, 't>(n: &'n str, t: &'t str) -> Option<Match<'n, 't>>` |

Rules:

- Lifetimes are declared first in the item's `<..>` list: `fn f<'a, T>`,
  `struct S<'a>`, `enum E<'a>`, `impl<'a> S<'a>`,
  `impl<'a> Iterator for S<'a>`.
- One name on several inputs means the result may borrow from any of them;
  at a call, the borrows shrink to the shortest, as in Rust.
- `&'a mut Vec<&'b str>` keeps the `&mut` borrow apart from the data's
  lifetime. Writing one name for both (`&'a mut Vec<&'a str>`) is legal
  but ties the mutable borrow to the data, Rust's classic trap: callers
  then fail to borrow-check, as in Rust.
- In a body, `let src = self.src;` copies the reference out, and
  `&src[a..b]` keeps lifetime `'a`, as in Rust. A plain elided `-> &str`
  on `&self` borrows the reference itself, not `'a`.
- Outlives bounds are Rust's: `'a: 'b`, `T: 'a`, `T: Trait + 'a`,
  `impl Trait + 'a`, `dyn Trait + 'a`. A result can get a lifetime of its
  own: `fn pick<'r, 'a: 'r, 'b: 'r>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str`.
- Relations that well-formed signature types imply need no bound:
  `&'m BTreeMap<char, Vec<&'t str>>` lets a result be `&'m [&'t str]`.
- Not in Tide, as beyond the power ceiling (5.1): lifetime parameters on
  traits (`trait Parse<'a>`, `FX0201`), explicit `for<'a>` (`FX0404`), and
  generic associated types (`FX0403`).
- An elided output that cannot be resolved, or that elision ties to the
  wrong input, is `FX0202`; its fix suggests the named lifetime (often the
  impl's `'a`).

### 3.3 Borrowing structs and enums

A struct or enum that holds references declares lifetimes for them,
exactly as in Rust: `Ident(&'a str)` in `enum Token<'a>`, a field
`lx: Lexer<'a>`. Variance is Rust's: `&T` covariant; `&mut T`, `Cell`,
`RefCell`, `Shared` invariant.

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

- **Several lifetimes.** Fields from different sources get different
  names; one name means one shared lifetime:
  `struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }`.
- **Impls.** `impl<'a> Lexer<'a>` is generic over the lifetime, and `Self`
  is `Lexer<'a>`, so a constructor `fn new(src: &str) -> Self` is an error:
  write `fn new(src: &'a str) -> Self`. Builders return `-> &mut Self`.

### 3.4 `'static` and `dyn`

- `'static` is Rust's: `&'static str`, string literals, consts. A type is
  *owned* when it borrows nothing except `'static` data (`T: 'static`).
  Work that outlives the current function requires it: `spawn_isolated`
  (7.1), `Box<dyn Trait>` contents in a signature or field, and children of
  a scope passed down unless they borrow only what the signature ties to
  the scope (7.1).
- `dyn Trait` follows Rust's object-lifetime defaults: behind `&'a` or
  `&'a mut` it is `dyn Trait + 'a`. Elsewhere outside a body
  (`Box`, `Rc`, `Arc`, type arguments) it means `dyn Trait + 'static`, unless
  the container's own `T: 'a` bound sets `'a`; inside a body it is
  inferred. Write
  `Box<dyn Trait + 'a>` for a borrowing object, for example
  `Box<dyn Fn(&Row) -> bool + 'a>` in a field of `struct Filter<'a>`.

### 3.5 Closures and returned `impl Trait`

Inside a body, closures and iterator chains work as in Rust. A closure that
captures a borrow is a borrowing value; it may be returned or stored only
where its lifetime is named or captured.

- A returned `impl Trait` captures every generic parameter in scope,
  lifetimes included, as in Rust 2024: `fn evens(xs: &[u32]) -> impl
  Iterator<Item = u32>` and `fn over(min: &u32) -> impl Fn(&Person) -> bool`
  need no annotation. To capture less, list what it captures:
  `-> impl Fn() -> usize + use<'a>`.
- Storing a closure in a `'static` position (`Box<dyn Fn()>` in a signature
  or field, `spawn_isolated`) requires that it capture no borrows: `move` owned values
  in (`clone()`, `to_string()`), or write `+ 'a` (`FX0203`).

### 3.6 What is impossible, and what to write instead

A struct cannot borrow from its own field (`FX0210`):
`struct Doc<'a> { text: String, words: Vec<&'a str> }` cannot be built with
`words` borrowing its own `text`.

| Need | Write |
| --- | --- |
| Parsed view of a buffer | Keep owner and view apart: `let text = fs.read_to_string(p)?; let doc = Doc::parse(&text);` |
| Store both together | Indices: `words: Vec<Range<usize>>` and `&self.text[self.words[i].clone()]` |
| Graphs, parent links, cycles | `Arena<T>` with `Id<T>` handles (below) |
| Shared ownership | `Rc<RefCell<T>>` and `Weak`, as Rust; `Arc` across tasks |

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

`Id<T>` is a `u32` index plus a `u32` generation. It is `Copy`, `Eq`,
`Hash`, `'static`, `Pure`, `Send`, and `Sync` (02 §10.9), and also `Ord` and
`Debug` (provisional), so it works as a `HashSet`/`HashMap` key. `Arena`
methods: `new`, `insert(T) -> Id<T>`, `get(id) -> Option<&T>`, `get_mut`,
`remove(id) -> Option<T>`, `get_disjoint_mut([a, b]) -> Option<[&mut T; 2]>`,
`len`, `Index`/`IndexMut` (panic on a stale id); `Default`,
`contains(id) -> bool`, and `iter() -> impl Iterator<Item = (Id<T>, &T)>`
(provisional). `Arena<T>` is `Pure` when `T` is.

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

| Rust | Tide |
| --- | --- |
| `x as u8` (may not fit) | `u8::try_from(x)?`; keep the low bits: `let b: u8 = x.truncate();` |
| `i32 as u32` (reinterpret) | `x.cast_unsigned()`; back with `cast_signed()` |
| `n as f64` (`usize`, `u64`, `i64`) | `f64::round_from(n)` (nearest, ties to even) |
| `f as i64` | `i64::try_from(f.trunc())?` (errors on NaN, infinity, out of range) |
| `x as f32` (`x: f64`) | `f32::round_from(x)` |

`truncate()` takes no arguments on integers and gets its target type from
context. `round_from` and float `TryFrom` are provisional names.

### 4.3 Floats

IEEE 754 as Rust, with native arithmetic. Rust leaves NaN bits and math
precision unspecified, so the methods that read a float's sign or bits
(`to_bits`, `is_sign_positive`, `is_sign_negative`, `copysign`, `signum`,
`total_cmp`, `min`, `max`) and the math methods (`sin`, `exp`, `powf`,
`ln`, ...) are `FX0520` (6.9). Use std's functions:
`float::bits(x)` (any NaN reads as one canonical NaN), `float::min(a, b)`
and `float::max(a, b)` (-0.0 below +0.0), and `math::sin(x)`,
`math::exp(x)`, ..., from one library on every target (`use std::{float,
math};`), each generic over `f32` and `f64` (`float::bits` of an `f32` is
a `u32`; signatures provisional). `f64::from_bits(u)`, `x.sqrt()`, and
`x.mul_add(a, b)` are admitted: they observe nothing or are correctly
rounded. Rounding and classification (`floor`, `ceil`, `round`, `trunc`,
`abs`, `is_nan`, `is_finite`) stay methods (provisional). `f64` is not `Eq`
or `Ord`; use `float::total_cmp(a, b)` (provisional name).

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

A missing auto or marker trait never makes impls disjoint: `impl<T: Pure> K for T`
and `impl K for Net` overlap (`FX0402`), though `Net` is not `Pure`. Trait
methods are namespaced by trait, as in Rust, so `Display::fmt` and
`Debug::fmt` coexist. A returned `impl Trait` is `Send`, `Sync`,
or `Pure` only if it names them: write `-> impl Iterator<Item = u8> + Send`
if callers need `Send`. It is `'static` when it captures no borrow, or when
it states `+ 'static`.

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
| `Tasks` (`std::task`) | `scope`, `spawn_isolated`, `shield`, `checkpoint`, `yield_now`, `fault_point` (7.1); creating synchronization handles (7.3, 7.4) | `limit(n) -> Tasks` |

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
- **Attenuation** returns a narrower capability of the same type, so
  attenuations chain: `root.net.only([..]).no_bind()`. Only `dir` can fail,
  so only `dir` returns `io::Result`. `Net::only` takes `"host:port"`
  entries in any iterable of strings (provisional), so `main` can
  attenuate to hosts computed at run time.
- Every capability error type has a cancellation case whose
  `is_cancelled()` is true (7.2).

### 6.2 `#[tide::main] fn main(root: Root)`

```rust
#[tide::main]
fn main(root: Root) -> Result<(), AppError> {
    let data = root.fs.dir("/var/lib/app", Mode::ReadWrite)?;   // Fs below one directory
    let net = root.net.only(["db.internal:5432"]);               // Net to one host
    app::run(&data, &net, &root.clock)?;
    Ok(())
}
```

- `Root` has public fields `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
  `proc`, `tasks`, and `log` (sink configuration, 6.5).
- The attribute generates Rust's parameterless entry point and mints
  `root`; every binary's `main` needs it (`FX0512`, safe fix). `main` takes
  `root: Root` or nothing, and returns `()`,
  `Result<(), E: Debug>`, or `std::process::ExitCode` (`ExitCode::SUCCESS`,
  `ExitCode::FAILURE`, `ExitCode::from(2u8)`), as in Rust. After `Err(e)`
  the program prints `Error: {e:?}` to stderr and exits with 1; to print
  `Display` and the `source()` chain, return `ExitCode` and log the error
  yourself. A panic exits with 101.
- `Root` can be named only in `main`'s signature. `root` cannot be passed,
  borrowed into a call, stored, returned, or captured (`FX0503`); move or
  borrow its fields (`root.fs`, `&root.clock`). No code may call `main` (`FX0506`),
  so a test cannot mint a real `Root`: put the logic in functions that take
  capabilities.
- `Root::sim(seed)` and everything in `std::sim` exist only in test code
  (`FX0504`, 6.8).

Attenuate in `main` and hand each subsystem only what it needs; review
shows where every capability reaching a function was attenuated.

### 6.3 Files: paths, append, durability

```rust
impl Fs {                                         // every method takes &self; p: impl AsRef<Path>
    fn read(&self, p) -> io::Result<Vec<u8>>;
    fn read_to_string(&self, p) -> io::Result<String>;
    fn write(&self, p, data: impl AsRef<[u8]>) -> io::Result<()>;   // create or truncate; not durable
    fn write_synced(&self, p, data: impl AsRef<[u8]>) -> io::Result<()>;   // durable when it returns
    fn open(&self, p) -> io::Result<File>;                          // read only
    fn create(&self, p) -> io::Result<File>;                        // write; create or truncate
    fn open_with(&self, p, opts: &OpenOptions) -> io::Result<File>; // append, create_new, read+write
    fn sync_dir(&self, p) -> io::Result<()>;                        // make p's entries durable
    fn create_dir_all(&self, p) -> io::Result<()>;
    fn exists(&self, p) -> bool;
    // read_dir, metadata, rename, remove_file, remove_dir_all: as std::fs, returning io::Result
    fn dir(&self, p, mode: Mode) -> io::Result<Fs>;    // Mode::Read | Mode::ReadWrite; fails if p is not a directory
    fn read_only(&self) -> Fs;
}
impl File {        // io::Read, io::Write, io::Seek, also for &File
    fn sync(&self) -> io::Result<()>;                  // D7; `sync_all`, `sync_data` accepted
    fn set_len(&self, n: u64) -> io::Result<()>;       // truncate or extend; works in append mode
    fn metadata(&self) -> io::Result<Metadata>;
    fn close(self) -> io::Result<()>;
}
```

- **`OpenOptions`** is Rust's builder (`read`, `write`, `append`,
  `truncate`, `create`, `create_new`) without its ambient `open` method
  (`FX0501`): `fs.open_with("events",
  OpenOptions::new().read(true).append(true).create(true))?`. As in POSIX,
  such a file reads from offset 0, and every write goes to the end.
- **A `File` carries the authority of the `Fs` that opened it**; its methods
  need no other capability.
- **Paths.** `root.fs` is rooted at `/`; a relative path resolves against
  the starting working directory (provisional; `/` under simulation). `read_only()` keeps the
  same base. `fs.dir(p, mode)` resolves `p` like any path and returns an
  `Fs` rooted at it, which resolves relative paths beneath `p`; an absolute
  path, or a `..` or symlink that would leave `p`, fails with
  `ErrorKind::PermissionDenied` (provisional kind). A `Mode::Read` `Fs`
  returns `PermissionDenied` on writes. Missing files give `NotFound`, as in
  Rust. To attenuate to a directory that may not exist yet, call
  `create_dir_all` first.
- **Durability** (D7, D15). Writes are durable only once synced:
  `file.sync()` for a file's contents, `fs.sync_dir(dir)` for the
  directory entries changed by `create`, `rename`, `remove_file`, or a new
  subdirectory. `write_synced` syncs the file and its entry.
  After a crash, anything not synced may be lost or torn (6.8). An
  acknowledged write is therefore: write, `sync()`, and, the first time a
  file or directory is created, `sync_dir` of its parent.
- **Torn tails.** A crash can leave a partial last record. Cut it off
  before appending again, or the next acknowledged record is glued to the
  torn bytes and lost on the next read.

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

### 6.4 No globals; deterministic hashing

Safe code has no mutable globals and no thread-locals: no `static mut`, no
`static` holding atomics, `Mutex`, `OnceLock`, or `Cell`, no
`thread_local!`, no `lazy_static!` (`FX0502`). A `static` must be
`'static + Sync + Pure`, and its value is computed at compile time by
`const fn`; `const` is as Rust. There are no lazy statics: build a value
that needs run-time work in `main` and pass it down.

`HashMap::new()` uses a fixed hash key, so iteration order is deterministic
(but unspecified). For untrusted keys use
`HashMap::with_hasher(RandomState::from_rand(&rand))`. `Arc` has no
`strong_count`, `weak_count`, `get_mut`, `try_unwrap`, or `into_inner`,
because their results depend on other tasks; `make_mut` stays. `Arc`,
`sync::Weak`, and atomics are synchronization handles, so creating one takes
`&Tasks` (7.4): `Arc::new(v, tasks)`, `Arc::downgrade(&a, tasks)`,
`AtomicU64::new(0, tasks)`; `core::sync::atomic` is `FX0520`. Cloning
needs nothing. `Rc` and `Cell` need no
capability, because they never leave their task.

### 6.5 Write-only sinks are not authority

`print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`, `error!`..`trace!`,
`std::io::stdout()`/`stderr()` (write-only `impl Write` handles that always
return `Ok`), and `std::trace::span(name)` (a guard that closes on drop)
write to sinks owned by the current task. They read nothing back, need no
capability, never suspend, and keep a function authority-free.

```rust
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

`Pure` (prelude) means no authority is reachable from the type. Std declares
it with a blanket impl, so rustc accepts every bound; Tide's checker decides
it structurally wherever a type meets a `Pure` bound or is coerced to a
`Pure` trait object (`FX0517`, unless the rule that needs `Pure` has its
own code). **Not `Pure`**: the capability types and `File`, `TcpStream`, and
other handles derived from them; synchronization handles (`Shared`, channel
ends,
`Scope`, `Task`, `Arc`, `sync::Weak`, atomics, `OnceLock`, and any `Sync`
type whose `&self` API can observe or cause mutation); raw pointers;
a `dyn Trait` whose trait lacks a `Pure` supertrait; an unbounded type
parameter. **Everything else
is `Pure` exactly when its parts are**, including references, collections,
`Arena`, `Box`, `Rc`, `Cell`, `RefCell`, structs, enums, and closures. `Cell`,
`RefCell`, and `Rc` are `Pure` because they are not `Sync`: their state
never leaves one task. `Arc` is not, because drop timing can reveal
scheduling (D7): authority-free and parallel code shares data by reference
inside a scope instead.

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
authority-freedom a compile error (`FX0511`).

| Use | Requires |
| --- | --- |
| `join` closures (8) | `Pure + Send`; shared captures `Pure + Sync` |
| `par_map`, `par_for`, `par_sort` (8) | closures `Fn + Pure + Send + Sync`; shared captures `Pure + Sync`; elements `Pure + Sync` (`par_map`) or `Pure + Send` |
| `Shared<T>` payloads; `with` closures (7.4) | `T: Send + Pure`; closure `Pure` |
| `static` (6.4) | `'static + Sync + Pure` |
| functions a contract clause calls (10.1) | authority-free, `Pure + Sync` arguments, `&mut` only to clause-created temporaries |
| `const fn` | authority-free, no `&mut` parameters |

### 6.7 Network

`Net::get` performs an HTTP(S) GET and returns the body.
`NetError` has `Timeout` (the operation's deadline passed), `Cancelled`,
`Denied` (a host outside `only`), `Refused`, `Reset`, `Dns`, `Status(u16)`
(a non-2xx response), and `Io(io::Error)`; `is_cancelled()` and
`is_transient()` (timeouts, resets, refusals, 5xx) classify them
(provisional beyond `Timeout` and `Cancelled`; every variant but `Status`
and `Io` is a unit variant). `NetError` is not `PartialEq`, since it can
hold an `io::Error`: test it with `matches!`. `Url::parse(&str) ->
Result<Url, UrlError>`; `Url` has the `url` crate's accessors (`host_str`,
`port_or_known_default`, `path`, `as_str`) and `Display`. `TcpStream`
implements `io::Read` and `io::Write`, also for `&TcpStream`.

### 6.8 Simulation

Every capability has a deterministic simulated version. `Root::sim(seed)`
returns a `SimRoot` (`std::sim`) with `Root`'s capability fields, all
simulated (pass `&sim.fs`, `&sim.net`, `&sim.clock`, `&sim.tasks` to the code
under test), plus `sim.world` (setup), `sim.faults` (fault
injection), `sim.node(name)` (crashable nodes), and inspection methods.
`Root::sim` works in any test, plain or property: the test body runs as the
simulation's main task, so it can spawn, wait, and read virtual time.

| Item | Effect |
| --- | --- |
| `#[sim_test(seeds = 200)] fn t(sim: SimRoot)` | runs the test once per seed with a fresh `SimRoot`; plain `#[sim_test]` runs 100 seeds; more parameters are generated (10.2) |
| `sim.world.file(path: impl AsRef<Path>, contents: impl AsRef<[u8]>)` | create a file (parents included) |
| `sim.world.args(["prog", "a.txt"])`, `env_var(k, v)`, `stdin(bytes)` | program inputs; `args` as `env.args()` returns them |
| `sim.world.http(addr: &str, f: impl Fn(&Request) -> Reply + 'static)` | a simulated HTTP peer at `"host:port"`; `req.url() -> &Url`; `Reply::ok(body: impl Into<Vec<u8>>)`, `.after(d)`, `Reply::status(503)`, `Reply::hang()` |
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
| `sim.elapsed() -> Duration`, `sim.records() -> Vec<Record>`, `sim.trace()` | virtual time; sink records; the event trace |

(`SimNode`, `NodeTask`, `Request`, `Record`, and the setup parameter types
are provisional.)

Semantics:

- **Time.** Monotonic time starts at zero, wall time at 2000-01-01. Virtual
  time advances only when every task is blocked, jumping to the next timer. Computation and
  `Clock`, `Rand`, `Env`, and `Tasks` calls take no virtual time; each file
  operation takes up to 1 ms and each network packet 1–10 ms, drawn from the
  seed (provisional). Bound `sim.elapsed()` by a deadline your code sets, or
  with slack. If no task can ever run again, the run fails with a deadlock
  report.
- **Order.** Capability operations, `spawn`, `join`, channel operations,
  `Shared::with*`, and checkpoints are scheduling points; the seed picks the
  interleaving, so each seed explores a schedule. Parallel regions (8) run
  sequentially. Each run writes a replay manifest: seed, content hashes of
  the source and every dependency, compiler version, target,
  semantics-affecting build options, simulated setup, and injected inputs.
  Replay refuses to run if the source or build differs; otherwise it
  reproduces the run bit for bit.
- **Packet loss.** `net_drop(p)` drops each packet independently with
  probability `p`. A UDP datagram is simply lost. TCP streams and
  `Net::get` retransmit, so loss shows up as delay: each loss of the same
  packet waits a retransmission timeout, 200 ms and doubling (provisional).
  An operation fails only when its own deadline passes (`Timeout`) or its
  scope is cancelled (`Cancelled`).
- **Records.** Every sink record, `println!` output included, enters
  `sim.records()` with its node, and stays there if the node later crashes.
  A `Record` has `node: String`, `time: Duration`, `sink: Sink` (`Out`,
  `Err`, `Dbg`, or `Log(Level)`), and `message: String`, one line without
  its newline (provisional).
- **Node entries.** `restart` calls the entry again, so it is a `fn` item or
  an `Fn` closure, never `FnOnce`. It is `Send`, and its captures are
  `'static`, `Pure`, and `Sync`, so a run cannot change them; clone inside it
  what each run consumes. Unlike `Root`, a `NodeRoot` is an ordinary value: pass it or its
  fields to helpers.
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
`tide sim <bin> -- args` runs `main` under many seeds as one node.

### 6.9 Ambient Rust calls and the admission allowlist

Ambient std calls are `FX0501`; the fix uses a capability in scope or
proposes adding the parameter.

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

The checker admits only listed `core`, `alloc`, and std inherent items;
everything else is `FX0520`, notably: hardware
intrinsics (`core::arch`); raw pointer types and address observers
(`addr()`, `{:p}`) without the `unsafe` grant; `core::sync::atomic`; float
observers and math methods (4.3); I/O methods on std types (`Path::exists`,
`metadata`; use `fs`); `io::Error::last_os_error`; timed waits such as
`recv_timeout` (use `recv_deadline`); and `include_str!`, `env!`,
`option_env!`.

### 6.10 Patterns

**Thread a capability.** When a leaf starts needing authority, add the
parameter along the call chain; never reach for a global or `Root`.
`tide fix --thread Fs --to decode_user` makes that edit along the shortest
path from a holder, test call sites included.

**App-context struct.** For service code, bundle attenuated capabilities
and clients built in `main`, and pass narrow pieces down:

```rust
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

```rust
impl Tasks {
    fn scope<'env, T, E>(&'env self, body: impl FnOnce(Scope<'_, 'env, E>) -> Result<T, E>) -> Result<T, E>
        where E: From<TaskError> + Send;
    fn spawn_isolated<T: Send>(&self, f: impl FnOnce() -> T + Send + 'static) -> Result<T, Panicked>;
    fn limit(&self, n: usize) -> Tasks;               // attenuation: at most n live children
    fn shield<T>(&self, f: impl FnOnce() -> T) -> T;   // ignore enclosing cancellation
    fn checkpoint(&self) -> Result<(), Cancelled>;
}
impl<'s, 'env, E> Scope<'s, 'env, E> {               // Copy + Send + Sync: pass by value
    fn spawn<U: Send + 's>(self, f: impl FnOnce() -> Result<U, E> + Send + 's) -> Task<'s, U, E>;
    fn set_deadline(self, at: Instant);
    fn cancel(self);
}
impl<U, E> Task<'_, U, E> { fn join(self) -> Result<U, E>; }
pub enum TaskError { Cancelled, DeadlineExceeded }  // Debug, Clone, Copy, PartialEq, Eq, Error (provisional)
pub struct Panicked { pub message: String }          // (provisional)
```

- **Structure.** Tasks are spawned into scopes and cannot outlive them.
  `scope` returns only after every child has finished; scope exit and `join`
  wait, are not checkpoints, and cannot be cancelled. Children may borrow
  anything that outlives the `scope` call, and every capability, `Shared`
  handle, and `Scope` is `Send + Sync`, so `s.spawn(|| Ok(net.get(u, d)))`
  may borrow `net`.
- **Failure.** A child or the body fails when it returns `Err`. The first
  failure cancels the scope (body, children, nested scopes). Once every
  child has finished, `scope` re-raises the panic of the earliest-spawned
  panicking child, if any; otherwise it returns the error of the
  earliest-spawned failing child, if any (`DeadlineExceeded` if the scope's
  own deadline cancelled it, 7.2); otherwise the body's result, `Ok` or
  `Err`. An error a child returns after seeing a cancellation that the
  body's `Err` or a sibling's failure caused is an echo and never counts,
  so the root cause is reported: the body's error when the children only
  echo it, and `x`'s error when the body fails through `x.join()?`. Other
  errors go to one `debug!` record. To collect per-item errors without
  cancelling siblings, return them as data:
  `s.spawn(move || Ok(net.get(u, deadline)))` gives
  `Task<'_, Result<Vec<u8>, NetError>, E>`. Since `E: From<TaskError>`, a
  scope's error enum needs a `#[from] TaskError` variant.
- **`join`** returns the child's `Ok` value, or `Err(TaskError::Cancelled
  .into())` if the child failed, panicked, or was cancelled (the scope
  reports the real cause). So `x.join()?` in the body is the normal form.
  Dropping a `Task` without joining is allowed.
- **Panics propagate** as in Rust's `std::thread::scope`: a child's panic
  cancels the scope, and `scope` re-raises it in the calling task (above). A
  panic in the body cancels and joins the children, then continues
  unwinding. Only destructors observe data a panicking child was mutating.
- **Isolation.** `tasks.spawn_isolated(move || ..)` runs a `'static` closure
  as a new task, waits for it, and returns `Err(Panicked)` if it panicked;
  it inherits the caller's cancellation. It is the only place a panic
  becomes a value (servers: one per request).
- **Bounded concurrency.** `let tasks = root.tasks.limit(4);` then spawn one
  child per item: scopes from that `Tasks` run at most 4 children at once,
  and `spawn` waits for a free slot (not a checkpoint). A child whose
  `spawn` waited past its scope's cancellation still starts, and sees the
  cancellation at its first checkpoint (provisional).
- **A scope passed down** (`fn serve<'s>(bg: Scope<'s, '_, E>, ..)`) outlives
  the function's locals, so its children must own their data (`move` with
  owned values) or borrow what the signature ties to the scope
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
  errors into values, `scope` returns `Ok` with the partial results. A child
  that returns `Err` after the deadline is reported as
  `Err(TaskError::DeadlineExceeded.into())` (7.1); the body's own `Err` is
  returned unchanged. `s.cancel()` cancels the same way but reports the
  child's own error. When a scope deadline and an operation deadline
  coincide, the operation sees the cancellation.
- **Soft.** Authority-free computation, `Shared::with*`, and joins are never
  interrupted; cancellation is observed at the next checkpoint.
- **Cleanup.** `tasks.shield(|| ..)` ignores enclosing cancellation; give
  bounded cleanup its own inner scope with a deadline.

### 7.3 Channels

Channels are Rust's `std::sync::mpsc`, except that creating one takes
`&Tasks` (7.4): `channel(tasks)` (unbounded; a lint in libraries, `FX0608`),
`sync_channel(n, tasks)`, `Sender: Clone`, `send`,
`try_send`, `recv`, `try_recv`, `recv_deadline(t)`, and a `Receiver` that
iterates until every sender drops. `recv` and `send` on a full
`sync_channel` are checkpoints. Payloads must be `Send`. A `Receiver` has
one consumer; for N workers over a list, use `Tasks::limit` (7.1). To wait
on several sources (Rust's `select!`), send one enum through one channel.

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

- `Shared<T>` replaces `Arc<Mutex<T>>`. It is a handle: `clone()` names the
  same cell. It is `Send + Sync` but not `Pure`; children may borrow
  `&Shared<T>`. Payloads are `Send + Pure` (`FX0606`), so a cell never holds
  a capability, channel end, or another `Shared`.
- **Creating a handle takes `&Tasks`.** Cells, channel ends, `Arc`, and
  atomics let tasks observe each other's timing (D7), so every constructor
  takes `&Tasks`, last: `Shared::new(v, tasks)`, `channel(tasks)`,
  `Arc::new(v, tasks)`. Cloning needs nothing. Authority-free code can
  neither create nor receive a handle. In a test, take them from a
  simulator: `let sim = Root::sim(0);`, then `Shared::new(v, &sim.tasks)`.
- `with2` and `with_all` acquire all their cells atomically in a global
  order, so they cannot deadlock. If two operands are the same cell, the
  call returns `Err(LockError::Aliased)` without running `f`; test identity
  first with `Shared::same(&a, &b)`. On an empty slice, `with_all` runs `f`
  on an empty slice.
- `f` must be `Pure`. It therefore captures no capability, channel end, or
  `Shared` handle, so it cannot do I/O, suspend, or nest another `with`
  (`FX0603`). Logging is allowed; printing is a lint (`FX0607`). `return`
  and `?` inside `f` leave `f`, as in any closure.
- Acquisition waits FIFO per cell and is not a checkpoint: it still works in
  a cancelled scope.
- A panic in `f`, an overflow trap included, poisons every cell it held;
  later calls return `Err(LockError::Poisoned)` without running `f`.
  `reset(v)` clears it. Check everything that can fail before the first
  write.
- Atomics (`std::sync::atomic`, sequentially consistent only) exist for
  library authors and are not `Pure`. `Mutex` and `RwLock` live in
  `std::sync::raw` and need the `unsafe` grant.

```rust
use std::sync::{LockError, Shared};

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
        return Ok(());                                  // otherwise Err(Aliased)
    }
    Shared::with2(from, to, |a, b| {
        let debited = a.balance.checked_sub(amount).ok_or(BankError::Insufficient)?;
        let credited = b.balance.checked_add(amount).ok_or(BankError::Overflow)?;
        a.balance = debited;                            // no write until both checks pass,
        b.balance = credited;                           // and nothing here can trap
        Ok(())
    })?                                                 // LockError -> BankError via #[from]
}

pub fn total(accounts: &[Shared<Account>]) -> Result<u128, LockError> {
    Shared::with_all(accounts, |xs| xs.iter().map(|a| a.balance as u128).sum())   // one snapshot; cannot overflow
}
```

### 7.5 Runtimes

The binary's manifest picks the runtime (`[bin] runtime = "threads"`, the
default, or `"blocking"`); tests and `tide sim` use the simulator.
Libraries never choose and behave the same under all of them. `threads`
runs one OS thread per task, so blocked tasks never starve runnable ones;
bound concurrency with `Tasks::limit` (7.1).

### 7.6 Porting async Rust

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

## 8. Parallelism

Deterministic fork-join for authority-free code:

```rust
use std::par::join;

let (a, b) = join(|| parse(x), || parse(y));
let lens: Vec<usize> = words.par_map(|w| w.len());     // == words.iter().map(..).collect()
pixels.par_for(|p| *p = p.saturating_add(10));         // over &mut [u8], disjoint
let total: u64 = docs.par_map(|d| score(d)).iter().sum();         // reduce sequentially
```

- `join` is in `std::par`. `par_map`, `par_for`, and `par_sort` (stable) are
  inherent slice methods and need no import:
  `fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U> where T: Pure + Sync`
  and `fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync) where T: Pure + Send`;
  `par_sort` needs `T: Pure + Send` too. These element bounds (`FX0612`)
  keep every `Shared` cell, channel, and capability out of reach of a branch.
- `join` closures must be `Pure + Send`; `par_map`, `par_for`, and
  `par_sort` closures are shared by branches, so they must also be `Fn` and
  `Sync` (no owned `Cell`) (`FX0610`). Shared captures must be
  `Pure + Sync` (`FX0611`: not `&RefCell`, `&Rc`, `&Arc`, or a capability).
  Disjoint `&mut` splits (`split_at_mut`) are fine.
- When no branch panics, `join`, `par_map`, and `par_for` equal their
  sequential versions, including sink output order. Under simulation they
  run sequentially.
- To reduce, `par_map` the expensive part and fold the results
  sequentially; `par_reduce` is v1 (Appendix A).
- Branches cannot do I/O. For concurrent I/O, spawn children in a
  `tasks.limit(n)` scope (7.1).
- If a branch panics, the panic propagates in the enclosing task, and data
  the region could write is left unspecified (but memory-safe); only
  destructors see it. If several panic, the first in sequential order is
  reported, and buffered sink output from later branches is discarded.

## 9. Errors and panics

- `Result<T, E>`, `?`, `From` conversion, `Box<dyn Error>`, `Option` and
  `?`, `unwrap`/`expect`, `map_err`, `ok_or`: as Rust.
- Results are must-use: an ignored `Result` is an error (`FX0702`); write
  `let _ = r;` to discard on purpose (provisional: Rust warns).
- `#[derive(Error)]` replaces `thiserror` (2.2).
- **Panics** (`panic!`, overflow, out-of-bounds, failed `assert!`, `unwrap` on
  `None`, a violated contract) unwind the current task and run destructors.
  A panic raised during that unwinding, for example in a destructor, aborts
  the process, and so does stack overflow, as in Rust. `tide test` and
  `tide sim` then rerun each test and seed in flight alone, report each
  that aborts (a seed with its replay command), and go on in a fresh
  process. A panicking child makes its scope panic
  (7.1). There is no `catch_unwind` (`FX0701`); isolate risky work with
  `tasks.spawn_isolated`. A panic in the main task exits with code 101.

## 10. Contracts

v0 contracts are executable: one spec, checked at run time in debug and test
builds, erased in release, and used to drive property tests.

### 10.1 Clauses

| Clause | Where | Meaning |
| --- | --- | --- |
| `#[requires(e)]` | attribute on a fn or inherent method, not a trait or trait-impl method (`FX0802`) | precondition, checked at entry; a failure blames the caller |
| `#[ensures(e)]` | as `requires` | postcondition, checked at each return; a failure blames the callee |
| `invariant!(e);` | first statements of a loop body | checked at loop entry (even for zero iterations) and exit; under `tide test`, also after every iteration |

- **Loops.** The enclosing fn must carry `#[invariants]` or any contract
  attribute, which rewrites its loops; elsewhere `invariant!` is an error.
- **One expression per clause.** `#[requires(a, b)]` is an error whose safe
  fix splits it into two attributes. Repeated clauses of one kind,
  `invariant!` included, are conjoined in order.
- **In `ensures`**, `result` is the return value (a contextual keyword): a
  place the clause reads but cannot move out of, so match on `&result` or
  use `result.as_ref()` unless it is `Copy` (provisional). A `&mut`
  parameter `v` (or `*v`) means the caller's referent at return, even if
  the body reassigns `v` (`v = &mut n.next`); `old(*v)` its value at
  entry. If `v`'s lifetime appears in another parameter's type, the result
  type, or a bound, `v` may be named only inside `old(..)` (`FX0803`); the
  usual fix gives the other parameter its own lifetime. `old(e)` evaluates
  `e` at
  entry and snapshots the value: `Copy`
  values are copied, others cloned, and a reference snapshots its referent.
  So `old(v.len())` costs one word, `old(*v)` clones the `Vec`, and `old`
  of a type that is neither `Copy` nor `Clone` is an error. `e` cannot
  mention `result` or a quantifier's variable: write `old(*v)[i]`. A panic
  while snapshotting is a violation of that clause. Other parameters mean
  their entry values: a `Copy` one is copied and a `Clone + Pure + Sync`
  one cloned at entry (in generic code, if so bounded); any other cannot be
  named if the body assigns, moves, or mutably borrows it, through a
  `&mut` it holds included (narrow it, `old(x.len())`, or add the bound).
  This does not change the clause's class.
- **Clause expressions** are ordinary Rust boolean expressions, evaluated
  as code: `==`, `<`, and arithmetic mean exactly what they mean in code,
  including overflow traps; widen (`a as u128 + b as u128`) when a sum might
  overflow. Evaluation is read-only: a clause may call only authority-free
  functions whose arguments are `Pure + Sync` and that take `&mut` only to
  temporaries the clause creates (`xs.len()`, `x.cmp(y)`,
  `xs.iter().all(|x| *x > 0)`, your own `is_valid(x)`, closures capturing
  such data). So a clause cannot reach a capability, `Shared`, `Cell`, or
  `RefCell`, not even `cells.len()` on a slice of handles. In generic code,
  a clause that uses a value of a type parameter `T` needs `T: Pure + Sync`
  (the fix adds the bound); no instantiation skips a clause.
- **Added syntax**, clauses only: `a ==> b` (implication, lowest precedence,
  right-associative); `forall |i: T| guard ==> p` and
  `exists |i: T| guard && p`, where `guard` bounds the binder as
  `lo <= i && i < hi` (an unsigned binder implies `0 <= i`) or
  `c.contains(&i)`. An unbounded quantifier is an error. A quantifier body
  extends as far right as possible: to the end of the clause, or of the
  enclosing parentheses or `match` arm. So write
  `result == (exists |i: usize| i < xs.len() && xs[i] == x)`.
- **When clauses run.** One rule: a clause built only from operators on std
  types, literals, `result`, `old` of `Copy` values, and std's accessors
  (`len`, `is_empty`, indexing, `get`, `contains`, `is_some`) is *checked*;
  any other clause, such as one with a quantifier, a closure, or a call to
  your own function, is *sampled*. Debug and test builds evaluate
  checked clauses at the points in the table; `tide test` also checks
  sampled clauses on every call, each under a step budget, so the outcome
  depends only on the inputs. One that runs over budget is
  reported as unchecked, not failed. Release builds erase clauses.
- **Visibility.** A `pub` function's contract may call only `pub` functions
  (Rust's private-in-public rule; provisional for v0).
- **Violations.** A clause that is false, or whose evaluation panics, is
  violated. Violations panic with `requires violated: <clause text>` (or
  `ensures`, `invariant`), the function's name, and the blamed span
  (provisional), so
  `#[should_panic(expected = "requires violated")]` can target them.

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
#[ensures(forall |i: usize| i < src.len() ==> dst[old(dst.len()) + i] == src[i])]
pub fn push_all(dst: &mut Vec<i64>, src: &[i64]) {
    for x in src { dst.push(*x); }
}
```

In a `for x in a..b` loop, an invariant sees `x` as the next iteration's
index: `x == a` before the first iteration and `x == max(a, b)` at exit.
A quantified invariant costs a linear scan each time it is checked.

### 10.2 Property tests

- A `#[test]` function with parameters is a property test: `tide test`
  generates and shrinks inputs. Generated types: integers, floats, `bool`,
  `char`, `String`, `Vec`, arrays, tuples, `Option`, `Result`, maps, sets,
  and structs and enums whose fields can all be generated, or that
  implement `Arbitrary`. A `&T` or `&mut T` parameter is generated as a `T`
  and borrowed (`&str` from a `String`).
- **With a simulator.** A `seed: u64` parameter is an ordinary generated
  input, and `Root::sim(seed)` in the body gives each case its own
  simulator (6.8). `#[sim_test] fn t(sim: SimRoot, sizes: Vec<u16>)` is
  shorthand for that: each case gets a fresh `SimRoot` plus generated values
  for the other parameters, `seeds = N` sets the number of cases, and
  shrinking keeps the seed (provisional). `requires` and the vacuity floor
  apply to both forms.
- **`#[requires]`**, on a test or a tested function, filters and guides
  generation. Conjuncts in a recognized shape are generated directly; the
  rest filter. Shapes: scalar
  ranges (`n <= 64`, `lo <= hi`), length bounds (`xs.len() > 0`), an index
  below a length (`i < xs.len()`), `xs.is_sorted()`, `xs.contains(&x)`,
  `o.is_some()`, and `x == c` (provisional spellings). Your own predicate,
  such as `is_valid(x)`, only filters. An early `return` counts as a pass,
  so prefer `requires`.
- **Vacuity.** `tide test` runs a property test until 100 cases (or `N`,
  for `seeds = N`) satisfy its `requires`. If under 1% of generated cases
  do, it fails the test as `vacuous` and names the conjunct that rejected
  most. A rare condition (a random `Vec` is seldom sorted) causes it:
  construct such inputs (`xs.sort()`), use a recognized shape, or give the
  type an `Arbitrary` impl.
- **Generated contract tests** (04 §2.4). `tide test` also tests every
  function with an `ensures` on generated inputs satisfying its `requires`,
  under the same vacuity floor. Capabilities come from a fresh
  `Root::sim(seed)`; a type with private fields is built by random public
  constructor and method calls. Type parameters become `i64` if it meets
  the bounds, else `String`. A function that cannot be generated (closure,
  `dyn`, or handle parameters; unmet bounds) is skipped as unchecked.
  `#[allow(vacuous, reason = "..")]` reports a vacuous generated test as
  unchecked, visibly in review. (Instantiation, skipping, and `allow` are
  provisional.)

```rust
#[test]
#[requires(lo <= hi)]
fn midpoint_stays_in_range(lo: u32, hi: u32) {
    let mid = lo + (hi - lo) / 2;
    assert!(lo <= mid && mid <= hi);
}
```

### 10.3 Contracts in review

`tide review` shows each public item's contract text. Weakening or
removing a contract of a public item, like widening authority, needs human
approval (11). A contract's hash covers the bodies of every function its
clauses call, transitively, and v0 has no solver to tell weakening from
strengthening, so any such change needs approval, except adding a contract
where there was none. If a contract or `#[protected]` test looks wrong, stop and
report it; never weaken it to get a green check.

## 11. Unsafe, trust holes, and review

| Hole | Spelling | Rule |
| --- | --- | --- |
| Unsafe code | `unsafe { .. }`, `unsafe fn`, `unsafe impl Send`/`Sync` | needs the package's `unsafe` grant (`FX0901`); each block needs a `// SAFETY:` comment (`FX0902`, provisional: error) |
| A type the checker finds not `Pure` but that is authority-free | `#[trusted_pure]` with a reason (`unsafe impl Pure` conflicts with std's blanket impl) | `unsafe` grant |
| `std::sync::raw` locks, weaker atomic orderings | — | `unsafe` grant |
| Raw pointers | `*const T`, `*mut T`, `p as usize`, `{:p}`, comparing, hashing, or printing them | `unsafe` grant (`FX0520` without it) |

`tide review <base>` prints the trust diff: public signatures, declared
authority (the capability types each public item can reach through its
parameters), contracts, trust holes, and grants, so a reviewer sees what
changed in what code *may do* before reading how. Changes that need a human
approval, recorded outside the agent's write scope and verified by CI:
widening authority, weakening or removing a contract, adding a trust hole,
and changing a `#[protected]` test. Never make one of these to get a green
check.

## 12. Modules and packages

### 12.1 Files are modules

- As in Rust: `src/main.rs` is a binary root, `src/lib.rs` a library root,
  and `mod net;` in a parent declares `src/net.rs` (or `src/net/mod.rs`);
  `pub mod` sets visibility. The directory tree is the module tree
  (`src/net/http.rs` is `crate::net::http`). `mod foo;` with no file is
  `FX0910`; `#[path]` is `FX0911`.
- `tide check path.rs` and `tide test path.rs` treat one file as a
  crate root, for single-file programs and exercises: a binary root if it
  defines `main`, otherwise a library root, where `println!` is lint
  `FX0508` (provisional).

### 12.2 Imports

- `use` as Rust: globs, `as`, nested groups, `pub use`. `extern crate` is
  `FX0912`.
- **Unused imports are errors** (`FX0913`, safe fix). **Import cycles
  between modules are errors** (`FX0914`): put shared types in leaf modules;
  a child's `use super::*` is fine while the parent does not import the
  child.

### 12.3 Prelude and std paths

The prelude is Rust's plus `Root`, the `Error` trait (`std::error::Error`,
so `Box<dyn Error>` and `e.source()` need no import), and the derives
`Error`, `Encode`, `Decode`, as 01's own example assumes, and the contract
attributes; `Pure` and `Secret` are in it too (provisional). Import everything else:

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

`Faults` is the type of `sim.faults`; you rarely name it, and there is no
`Faults { .. }` literal.

### 12.4 Manifest and grants

```toml
# tide.toml (provisional file name and keys)
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

- Escape hatches are granted per dependency. `tide.lock` is on by default,
  with content hashes, and records each package's public authority and
  trust holes; an upgrade that widens either fails until a human approves.
- The client rejects a dependency name that does not exist; it never
  guesses a near name.
- Tide packages have no `build.rs`, install hooks, or code that runs at
  fetch or build time (`FX0915`). Code generators are programs you run
  explicitly (`tide run --bin gen`); commit their output.
- Tide's driver runs rustc itself, not Cargo; `RUSTFLAGS` and unstable
  features do not apply.

## 13. Toolchain loop

| Command | Use |
| --- | --- |
| `tide check` | One verdict plus NDJSON diagnostics: stable codes, exact and related spans, fixes marked `safe` or `suggest`. |
| `tide fix` | Apply safe fixes; `--thread <Cap> --to <fn>` threads a capability. |
| `tide test [filter] [--seed N] [--trace]` | Tests, property tests, and sim tests: diff, panic span, seed, shrunk input, replay command, event trace. |
| `tide sim <bin> -- args` | Run `main` under many seeds; `tide sim --replay <manifest>` replays a run (6.8). |
| `tide eval '<expr>'` | Evaluate an expression against the package (a scratch crate built by rustc). |
| `tide explain FX0201` | Explanation with a before/after example. |
| `tide review <base>` | Trust diff (11). |
| `tide fmt` | The one canonical format. |

**Typed holes**: `todo!()` or `hole!(name)` in expression position. `tide check`
checks the rest and reports each hole's expected type, bindings in scope,
and candidate functions; use holes to build a function top-down.

## 14. Rust habits and their fixes

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
| `std::fs::read`, `OpenOptions::open`, `env::args`, `Instant::now`, `thread_rng` | capability methods (6.9) | `FX0501` |
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
