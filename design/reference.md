# Ferox for Rust Programmers

Status: core language reference, derived from `design/01-design.md` (draft
2.1), which wins on any conflict. Syntax is provisional until the E1
learnability gate (D0). Budget: under 25,000 tokens, counted in CI (D14). The
standard library reference is separate (`ferox doc`).

Items marked **(provisional)** are details the design leaves open; this
reference picks the most Rust-like option consistent with the decisions. They
are listed in section 16. Diagnostic codes (`FX0201`, ...) are placeholders
until the diagnostic catalog freezes.

## 0. How to use this document

You know Rust. Write Rust. Ferox is Rust's surface with a few changes in
meaning, and every place where your Rust habit would mean something different
fails loudly with a stable code and, usually, a machine-applicable fix. Read
section 1 for what is unchanged, sections 2–12 for the differences, section 14
for the habit table, and section 15 for complete programs.

The four differences you will hit first:

1. **No lifetime parameters.** Elision works as in Rust. Where Rust needs
   `'a`, write `from`: `fn pick(a: &str, b: &str) -> &str from a`. (A Rust
   `'a` signature that maps exactly onto origins is accepted, and `ferox fmt`
   rewrites it.)
2. **No ambient I/O.** Files, network, clock, randomness, environment,
   stdin, processes, and task spawning are methods on capability values that
   `main` receives in `root: Root` and passes down as parameters.
   `println!` and `info!` still work anywhere.
3. **No `async`.** Write blocking-style code; concurrency is structured
   scopes from the `Tasks` capability.
4. **Integers trap on overflow in every build**, and `as` only does lossless
   conversions.

## 1. What is unchanged

Everything not listed in sections 2–13 works exactly as in Rust (2024
edition). Precisely:

- **Syntax and items**: `fn`, `struct`, `enum`, `union` (unsafe), `impl`,
  `trait`, `type` aliases, `const`, immutable `static` (see 6.4), `use`,
  inline `mod name { }`, attributes, doc comments, `where` clauses,
  `pub`/`pub(crate)`/`pub(super)`/`pub(in path)`.
- **Expressions and statements**: blocks as expressions, `let`/`let else`,
  `if`/`if let` and let chains, `match` with guards, `loop`/`while`/
  `while let`/`for`, labeled `break`/`continue`, `break` with value, `return`,
  `?` on `Result` and `Option`, ranges, indexing, method calls, struct update
  syntax, closures (`|x| ..`, `move`), `unsafe` blocks (with a grant, 11).
- **Patterns**: all Rust patterns, exhaustiveness, binding modes, `@`, `..`,
  slice patterns, `ref`/`ref mut`.
- **Ownership and borrowing**: moves, `Copy`, `Clone`, `Drop`, shared xor
  mutable references, NLL precision, two-phase borrows, reborrowing, deref
  coercion, auto-ref in method calls, drop order, and Rust's conservative
  drop check. Only lifetime *syntax* differs (section 3).
- **Traits and generics**: trait definitions, default methods, supertraits,
  generic traits, associated types and consts, blanket impls, `impl Trait`
  in argument and return position (a borrowing return names its origins,
  3.5), `dyn Trait`
  with Rust's object-safety rules, operator traits (`Add`, `Index`, `Neg`,
  `AddAssign`, ...), `Deref`, `Iterator`/`IntoIterator` for `for` loops,
  `From`/`Into`/`TryFrom`/`TryInto`, `Display`/`Debug` in format strings,
  `Default`, `Fn`/`FnMut`/`FnOnce`, `Send`/`Sync` (structural), turbofish,
  const generics over integers, `bool`, and `char` (provisional). Limits in
  section 5.
- **Closures and iterators**: capture inference, `move`, all `Iterator`
  adapters and consumers, `collect` into any `FromIterator`.
- **Modules and visibility**: paths (`crate::`, `super::`, `self::`), `use`
  with globs, renames and `pub use` re-exports, privacy rules. File layout
  differs slightly (section 12).
- **Standard types** with their Rust API: integer and float scalars, `bool`,
  `char`, `str`, `String`, slices, arrays, tuples, `Option`, `Result`, `Vec`,
  `Box`, `Rc`, `Arc`, `Weak`, `Cell`, `RefCell`. Also, with their Rust API
  (provisional): `VecDeque`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`,
  `BinaryHeap`, `Cow`, `Path`, `PathBuf`, `Duration`, `Instant`, `SystemTime`,
  `std::cmp`, `std::ops`, `std::mem`, `std::fmt`, `std::convert`, and the
  `std::io` traits and types (`Read`, `Write`, `BufRead`, `BufReader`,
  `BufWriter`, `io::Error`, `io::ErrorKind`). What changes is how you *obtain*
  I/O objects (from capabilities) and the default hasher (6.9).
- **Testing**: `#[test]`, `#[should_panic(expected = "...")]`,
  `#[cfg(test)] mod tests { use super::*; ... }`, `assert!` family.

Ferox source files end in `.fx`.

## 2. Surface: `!` forms, macros, derives, attributes

### 2.1 The fixed built-in `!` forms

Users cannot define macros. The compiler implements exactly these forms, with
Rust's spelling and meaning:

| Form | Ferox notes |
| --- | --- |
| `print!`, `println!`, `eprint!`, `eprintln!` | Write to the current task's stdout/stderr sink. Need no capability (6.5). Lint in library packages. |
| `format!`, `write!`, `writeln!` | As Rust, including inline `{name}` arguments and all format specs. |
| `vec!` | As Rust. |
| `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` | Run-time checks as Rust. Under `prove` mode, each is a proof obligation (10.3). |
| `matches!` | As Rust. |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | As Rust. `todo!()` is also a typed hole (13). |
| `dbg!` | As Rust. Error in release builds (`FX0507`). |
| `error!`, `warn!`, `info!`, `debug!`, `trace!` | Write-only log sink (6.5), format-string checked. No `use` needed; `log::info!` and `tracing::info!` are accepted as the same forms. (provisional: path spellings) |

Anything else ending in `!` is an error (`FX0103`), including
`debug_assert_eq!` (write `debug_assert!(a == b)`), `include_str!`,
`include_bytes!`, `env!`, `concat!`, `stringify!`, `cfg!`, `format_args!`,
`thread_local!`, `lazy_static!`, and every crate macro. `macro_rules!` and
procedural macros are `FX0101`.

Replacing a macro you would have written:

- repeated code → a generic function or a trait;
- test tables → a `const` array of cases looped over in one `#[test]`;
- generated code → a generator program you run yourself; commit its `.fx`
  output (12.5);
- embedded data → a generated `const` (same route).

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

The derive set is closed and compiler-implemented:

| Derive | Meaning |
| --- | --- |
| `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Clone`, `Copy`, `Debug`, `Default` | As Rust. Derived impls carry generated proofs of their trait laws (5.3). |
| `Error` | `thiserror` semantics, built in: `#[error("...")]` on the type or each variant generates `Display`; `#[from]` on a single-field variant generates `From`; `#[source]` (or a field named `source`) sets `Error::source`; `#[error(transparent)]` forwards both. |
| `Encode`, `Decode` | Compiler-generated serialization through format plug-ins (`std::json`, ...). |

```ferox
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {path:?}")]
    Read { path: PathBuf, source: io::Error },
    #[error("bad config: {0}")]
    Parse(#[from] json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
pub struct Config {
    pub listen_addr: String,
    #[codec(default)]
    pub workers: u32,
    #[codec(skip)]
    pub cache: Option<Vec<u8>>,
}

let cfg: Config = json::decode(&bytes)?;      // Result<Config, json::Error>
let out: Vec<u8> = json::encode(&cfg);
```

`#[codec(...)]` keys follow serde's: `rename`, `rename_all`, `default`,
`skip`, `flatten`, `deny_unknown_fields`, and `try_from = "Raw"` (decode into
`Raw`, then validate through `TryFrom<Raw>`, so decoding cannot break a type's
invariant). (provisional)

Other derives fail with `FX0102` and a fix: `Serialize` → `Encode`,
`Deserialize` → `Decode`, `thiserror::Error` → `Error`. Remove
`use thiserror::Error;` and `use serde::...`; the built-ins are in the
prelude.

Every nominal type also exposes a read-only `Shape` (fields, variants, names,
layout) through `std::shape`. You rarely need it; capability types can never
be constructed through it.

### 2.3 `const fn`, attributes, `cfg`

- `const fn` as Rust, but authority-free, deterministic, and fuel-limited,
  with no `&mut` parameters. It computes values, never types or items.
- Rust attributes with the same meaning are kept: `#[derive]`, `#[test]`,
  `#[should_panic]`, `#[cfg]`, `#[allow]`/`#[warn]`/`#[deny]`, `#[inline]`,
  `#[must_use]`, `#[repr(..)]`, `#[non_exhaustive]`, `#[deprecated]`,
  `#[doc]`.
- `#[cfg(..)]` accepts `test`, `debug_assertions`, `target_os`,
  `target_arch`, `target_family`, `unix`, `windows`, and `feature = ".."`.
  (provisional beyond `test`)
- Ferox attributes: `#[error]`, `#[from]`, `#[source]`, `#[codec]`,
  `#[contracts(..)]`, `#[spec]`, `#[trusted(..)]`, `#[trusted_pure(..)]`,
  `#[trigger]`, `#[protected]`, `#[pure]` (6.6).
- `#[tokio::main]`, `#[async_trait]`, `#[path]`, `#[macro_export]`, and
  `#[proc_macro*]` are errors with fixes.

## 3. Origins instead of lifetimes

Ferox has Rust's borrow checker with no lifetime parameters. An **origin**
names where a borrow comes from: a parameter, a place path, or `'static`.

### 3.1 Elision is Rust's

Rust's three elision rules apply unchanged. Every signature that compiles in
Rust without lifetimes means the same in Ferox:

```ferox
fn first_word(s: &str) -> &str                       // from s
fn get(&self, key: &str) -> Option<&V>               // from self (the &self rule)
fn split(&mut self) -> (&mut [u8], &mut [u8])        // from self
```

### 3.2 `from` clauses: where Rust needs `'a`

Where elision does not decide, name the origin with `from` directly after a
type. `from p` means "borrows from what the value at place `p` borrows". If
`p` is a reference parameter, that is its referent.

| Ferox | Rust |
| --- | --- |
| `fn longest(a: &str, b: &str) -> &str from a, b` | `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str` |
| `fn pick(a: &str, b: &str) -> &str from a` | `fn pick<'a>(a: &'a str, b: &str) -> &'a str` |
| `fn next(&mut self) -> Option<&str from self.src>` | `fn next(&mut self) -> Option<&'a str>` in `impl<'a> Lexer<'a>` |
| `fn longest(words: &[&str]) -> &str from *words` | `fn longest<'a>(words: &[&'a str]) -> &'a str` |
| `fn add(m: &mut HashMap<&str from w, u32>, w: &str)` | `fn add<'a>(m: &mut HashMap<&'a str, u32>, w: &'a str)` |
| `fn merge(a: &str, b: &str) -> Vec<&str from (a, b)>` | `fn merge<'a>(a: &'a str, b: &'a str) -> Vec<&'a str>` |
| `fn name() -> &'static str` | same |

Forms:

- **Parameter**: `from a`. **Several**: `from a, b` after the whole type;
  inside generic arguments, parenthesize: `Vec<&str from (a, b)>`.
- **Place path**: `from self.src`, `from cfg.name`, `from *words`. A path
  through a field names that field's origin, not the whole struct. This is
  how a token returned from `&mut self` stays usable after the next call.
- **In parameter types**, to relate parameters (the `add` row).
- **Inside a type**: `Option<&str from self.src>`, `Vec<&str from text>`.
- **`Self.field`** names `Self`'s origins inside an impl, where there is no
  `self` value: `type Item = &str from Self.src;`.

`from` is a contextual keyword, a keyword only directly after a type in a
signature, a field declaration, or a `dyn` type, so `String::from`,
`PathBuf::from`, and `impl From<X>` are unaffected.

**Rust lifetimes.** A named lifetime whose use maps exactly onto origins is
accepted as a legacy spelling and `ferox fmt` rewrites it to `from`
(`fn pick<'a>(a: &'a str, b: &str) -> &'a str` becomes
`fn pick(a: &str, b: &str) -> &str from a`; `struct P<'a> { s: &'a str }`
becomes `struct P { s: &str }`). Other uses are `FX0201` errors that suggest
origins: outlives bounds (`'a: 'b`, `T: 'a`), lifetime parameters on traits,
`for<'a>`, and lifetimes that no parameter or field carries.

```
error[FX0201]: lifetime bound `'a: 'b` has no origin equivalent
 --> src/text.fx:4:9
help: drop the bound and name the source: `-> &str from a`
```

A borrow that elision ties to the wrong origin is `FX0202`; its fix suggests
the `from` clause (often `from self.field`). `'_` is accepted and means
"elided". (provisional)

### 3.3 Borrowing structs

A struct with reference fields declares no parameters. Each field whose type
contains a borrow gets its own origin, named by the field. Type arguments
carry their origins (`Vec<&str>` borrows what its elements borrow). Variance
is Rust's: `&T` covariant, `&mut T`, `Cell`, `RefCell`, and `Shared` invariant.

```ferox
pub struct Lexer {          // Rust: struct Lexer<'a> { src: &'a str, pos: usize }
    src: &str,              // origin `src`
    pos: usize,
}

impl Lexer {                // Rust: impl<'a> Lexer<'a>
    pub fn new(src: &str) -> Lexer {          // elision: from src
        Lexer { src, pos: 0 }
    }
    pub fn next_word(&mut self) -> Option<&str from self.src> {
        let rest = &self.src[self.pos..];
        let start = rest.find(|c: char| !c.is_whitespace())?;
        let len = rest[start..].find(char::is_whitespace).unwrap_or(rest.len() - start);
        self.pos += start + len;
        Some(&rest[start..start + len])
    }
}

impl Iterator for Lexer {
    type Item = &str from Self.src;           // Self's origin, named by field
    fn next(&mut self) -> Option<&str from self.src> { self.next_word() }
}

fn two_words(text: &str) -> Option<(&str, &str)> {
    let mut lx = Lexer::new(text);
    let a = lx.next_word()?;
    let b = lx.next_word()?;                  // fine: `a` borrows text, not lx
    Some((a, b))
}
```

**Sharing an origin between fields.** Fields get distinct origins by default.
When one field must hold data borrowed from another field's origin (an
invariant position such as `&mut Vec<&str>`), write `from` on the field:

```ferox
struct Collector {
    src: &str,
    out: &mut Vec<&str from src>,   // Rust: struct Collector<'a, 'o> { src: &'a str, out: &'o mut Vec<&'a str> }
}
```

`ferox fmt` rewrites `struct S<'a> { x: &'a str, y: &'a str }` to
`struct S { x: &str, y: &str from x }`.

**Returning a multi-origin struct** built from several parameters:
`-> Pair from a, b` lets every origin of `Pair` borrow from either. There is
no per-field binding in signatures. (provisional)

Using a borrowing type never needs parameters: write `Lexer`, never
`Lexer<'a>` or `Lexer<'_>` (the formatter deletes them).

### 3.4 `'static` and `Owned`

- `'static` is kept: `&'static str`, string literals, `const` data.
- `Owned` is a compiler-known auto trait: a type with no borrowed origins.
  It is Rust's `T: 'static`, which is accepted as a legacy spelling and
  rewritten to `T: Owned`. Detached work (a closure spawned into a scope
  that outlives the current function, 7.1) requires `Owned`.
- `dyn Trait` in `Box`, `Rc`, `Arc`, and other owning positions means
  `dyn Trait + Owned`. Write `dyn Trait from x` for a borrowing trait object
  (Rust's `dyn Trait + 'a`). Behind `&`/`&mut`, the object borrows from the
  reference's origin, as in Rust's defaults. (provisional)

```ferox
fn boxed(text: &str) -> Box<dyn Iterator<Item = &str> from text> {
    Box::new(text.split(','))
}
fn errors() -> Vec<Box<dyn Error>> { Vec::new() }     // Owned: no borrows inside
```

### 3.5 Closures and iterator chains

Inside a function body, as Rust: capture inference, `move`, and iterator
chains that borrow work unchanged. A closure that captures a borrow is itself
a borrowing value. **A borrowing value may be returned or stored only where
its origins are named**: write `from` on a returned `impl Trait` or `dyn`
type, or on the field that stores it. This replaces Rust 2024's implicit
capture and `+ '_`. Plain references and borrowing structs still use
elision (3.1). (provisional: where "named" ends)

```ferox
struct Person { name: String, score: u32 }

fn top_names(people: &[Person], min: u32) -> Vec<&str> {        // elision
    people.iter().filter(|p| p.score > min).map(|p| p.name.as_str()).collect()
}

fn top_iter(people: &[Person], min: u32) -> impl Iterator<Item = &str> from people {
    people.iter().filter(move |p| p.score > min).map(|p| p.name.as_str())
}

fn over(min: &u32) -> impl Fn(&Person) -> bool from min {
    move |p| p.score > *min
}

fn boxed(min: &u32) -> Box<dyn Fn(&Person) -> bool from min> {
    Box::new(move |p| p.score > *min)
}
```

In `impl Fn(A) -> R from x` and `dyn Fn(A) -> R from x`, `from` applies to
the closure type, as Rust's `+ 'a` does; to put an origin on `R`, write
`-> impl Fn(A) -> (&U from x)`. (provisional) A borrowing `impl Trait`
return without `from` is `FX0204` (safe fix: add `from` naming the parameter
elision would pick). Storing a closure in an
`Owned` position (`Box<dyn Fn()>`, a scope that outlives the function, an
`Owned` bound) requires that it capture no borrows: move owned values in
(`move` plus `clone()`) or write `from`. Violation: `FX0203`.

### 3.6 Field-borrow declarations on methods

A method may declare which fields of `self` it touches. Callers then keep
borrows of the *other* fields alive across the call, which Rust rejects
(E0502/E0499). Syntax: `&self.{a, b}` or `&mut self.{a}`. (provisional)

```ferox
struct Parser { toks: Vec<Token>, pos: usize, errors: Vec<String> }

impl Parser {
    fn peek(&self.{toks, pos}) -> Option<&Token> { self.toks.get(self.pos) }
    fn note(&mut self.{errors}, msg: String) { self.errors.push(msg); }

    fn check(&mut self) {
        if let Some(t) = self.peek() {                  // borrows toks, pos
            if t.is_bad() {
                self.note(format!("bad token {t:?}"));  // Rust: E0502. Ferox: disjoint, OK.
            }
        }
    }
}
```

Rules: the body may use only the listed fields (`FX0220`); an elided result
borrows from the listed fields; on a `pub` method the list is part of the
public signature, and changing it is a breaking change shown by
`ferox review`. (provisional) Without a declaration, `&self`/`&mut self`
borrows all of `self` as in Rust. Splitting borrows of fields inside one
function body works as in Rust without declarations.

### 3.7 What is impossible, and what to write instead

A struct cannot borrow from its own owned field (`FX0210`):

```ferox
struct Doc { text: String, words: Vec<&str from text> }   // FX0210: self-referential
```

Alternatives, in order of preference:

| Need | Write |
| --- | --- |
| Parsed view of a buffer | Keep owner and view apart: `let text = fs.read_to_string(p)?; let doc = Doc::parse(&text);` with `struct Doc { words: Vec<&str> }`. |
| Store both together | Indices: `struct Doc { text: String, words: Vec<Range<usize>> }` and `fn word(&self, i: usize) -> &str { &self.text[self.words[i].clone()] }`. |
| Graphs, trees with parent links, cycles | `Arena<T>` with generational `Handle<T>` (below). |
| Shared ownership | `Rc<RefCell<T>>`, `Weak` for back edges, as Rust. `Arc` across tasks. |
| Last resort | Raw pointers in `unsafe` (needs the `unsafe` grant; a trust hole). |

```ferox
use std::arena::{Arena, Handle};

struct Node { name: String, edges: Vec<Handle<Node>> }

let mut g: Arena<Node> = Arena::new();
let a = g.insert(Node { name: "a".into(), edges: vec![] });
let b = g.insert(Node { name: "b".into(), edges: vec![a] });
g[a].edges.push(b);               // cycle: handles are Copy values, not borrows
g.remove(b);
assert!(g.get(b).is_none());      // stale handle: None, never a dangling read
```

`Arena` API (provisional names): `new`, `insert(T) -> Handle<T>`,
`get(h) -> Option<&T>`, `get_mut`, `remove(h) -> Option<T>`, `Index` (panics
on a stale handle), `iter`, `len`.

No hidden reference counting exists: values are never shared implicitly.

## 4. Integers, floats, conversions

### 4.1 Overflow traps in every build

`+ - * / % << >>`, unary `-`, `abs`, and `pow` trap on overflow, division by
zero, `MIN / -1`, and shifts by `>=` the bit width, in debug **and release**.
A trap is a panic (section 9). The trap may be reported later than the
faulting operation, but before the bad value affects any observable effect or
leaves the enclosing loop or function, so do not rely on the exact point.
Proved contracts delete checks.

When you want modular or clamped arithmetic, say so, exactly as in Rust:
`wrapping_add`, `wrapping_mul`, `wrapping_shl`, `checked_add` (→ `Option`),
`saturating_sub`, `overflowing_add`, `Wrapping<T>`, `Saturating<T>`. Hash
functions, checksums, and PRNGs must use these.

### 4.2 `as` is lossless only

`as` compiles only when the conversion is exact for every value:

| Allowed `as` | Examples |
| --- | --- |
| integer to wider integer, same signedness | `u8 as u32`, `i32 as i64` |
| unsigned to strictly wider signed | `u32 as i64` |
| `bool` to any integer; `u8` to `char`; `char` to `u32`/`u64`/`i64`/`u128`/`i128` | `b as u8`, `c as u32` |
| integer to float when exact | `u8`/`i8`/`u16`/`i16` → `f32`; up to `u32`/`i32` → `f64` |
| `f32 as f64` | |
| `usize`/`isize`, treated as 32 to 64 bits | `u32 as usize`, `usize as u64`; not `u64 as usize` (provisional) |
| fieldless enum to an integer that holds every discriminant | `Color::Red as u8` |
| pointer casts inside `unsafe` | as Rust |

Lossy `as` is `FX0301` with these fixes:

| Rust | Ferox |
| --- | --- |
| `x as u8` (may not fit) | `u8::try_from(x)?` (or `.expect("..")`); keep low bits: `let b: u8 = x.truncate();` |
| `i32 as u32` (reinterpret bits) | `x.cast_unsigned()`; `cast_signed()` back |
| `len as f64` (`usize`, `u64`, `i64`) | `f64::round_from(len)` (nearest, ties to even) |
| `f as i64` | `i64::try_from(f.trunc())?`; pick `round()`, `floor()`, `ceil()` first. Errors on NaN, infinity, out of range, or a non-integral value. |
| `x as f32` (`f64`) | `f32::round_from(x)` |

`truncate()` takes no arguments on integers (unlike `Vec::truncate(n)`), and
its target type comes from context, like `into()`. `u64::from(x)`,
`x.into()`, `T::try_from`, and `try_into()` work as in Rust.
(`round_from` and float `TryFrom` are provisional names.)

### 4.3 Floats

IEEE 754 as Rust. `to_bits()` returns the canonical NaN bit pattern for every
NaN, so results match across targets. `total_cmp` as Rust. `f64` is not `Eq`
or `Ord`, as in Rust.

## 5. Traits and generics

### 5.1 Power ceiling

Every item signature is annotated; inference is local to a function body.
Generics are definition-checked: an instantiation never produces a type error,
and there are no post-monomorphization errors. These are rejected:

| Not allowed | Rust example | Code | Write instead |
| --- | --- | --- | --- |
| Specialization, overlapping impls | `default fn`, two blanket impls | `FX0402` | one impl per type; a newtype |
| Generic associated types | `type Item<'a> where Self: 'a` | `FX0403` | owned items, indices, or a method returning a borrow |
| Higher-ranked bounds, rank-N | `for<'a> Fn(&'a T) -> &'a U` | `FX0404` | `Fn(&T) -> &U` (elision covers it) |
| Higher-kinded types | `trait Functor<F<_>>` | `FX0404` | concrete traits |
| Negative bounds and impls | `impl !Send for T` | `FX0405` | a `PhantomData<Rc<()>>` field |
| User auto traits | `auto trait Foo {}` | `FX0405` | an ordinary trait |
| Impl where-clause not smaller than its head | `impl<T> Foo for T where Vec<T>: Foo` | `FX0406` | bound on `T` directly |
| Polymorphic recursion | `fn f<T>(x: T) { f(vec![x]) }` | `FX0407` | an enum or `Box<dyn ..>` |
| Overloading | two inherent `fn new` | `FX0408` | distinct names |

Associated types are flat (no parameters of their own). `dyn Trait` is always
written with `dyn`. Trait methods are namespaced by trait as in Rust, so
`Display::fmt` and `Debug::fmt` coexist; "one method per name per type"
applies to inherent methods. (provisional reading of D3)

### 5.2 Impl placement

An `impl` block must live in the module that defines the type (inherent and
trait impls) or the trait (trait impls). `impl Display for User` belongs in
the file that defines `User`, not in a separate `fmt.fx`. Violation:
`FX0401`, with a suggested fix that moves the block. Implementing a foreign
trait for a foreign type is impossible, as in Rust: use a newtype.

### 5.3 Laws

A trait may declare bodiless `law fn`s: universally quantified properties
every impl must satisfy. Their clauses may call the trait's own methods,
which must then be authority-free. (provisional)

```ferox
pub trait Combine: Sized {                 // std::par: what par_reduce needs
    fn empty() -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait Monoid: Combine {                // adds the laws
    law fn associative(a: Self, b: Self, c: Self)
        ensures a.combine(&b).combine(&c) == a.combine(&b.combine(&c));
    law fn identity(a: Self)
        ensures Self::empty().combine(&a) == a && a.combine(&Self::empty()) == a;
}
```

Std traits carry laws: `PartialEq`/`Eq` (equivalence), `PartialOrd`/`Ord`
(order consistent with `Eq`), `Hash` (consistent with `Eq`), `Monoid`
(above; trait names provisional). Derived impls come with generated proofs.
A hand-written impl is property-tested under `test` mode, proved under
`prove` mode, and counts as an unproved law impl in the trust surface until
one of those happens (11).

## 6. Authority: capabilities

### 6.1 The rule

Code touches the outside world only through capability values it is handed.
The standard library has no free functions that do I/O, read the time, draw
randomness, read the environment, or spawn. A function's parameters bound
what it can do, in every dependency.

| Capability | From | Main methods (Rust analogue) | Attenuation |
| --- | --- | --- | --- |
| `Fs` | `root.fs` | `read`, `read_to_string`, `write`, `open`, `create`, `create_dir_all`, `read_dir`, `metadata`, `exists`, `rename`, `remove_file` (as `std::fs::*`, returning `io::Result`) | `dir(path, Mode::Read \| Mode::ReadWrite) -> io::Result<Fs>`; `read_only()` |
| `Net` | `root.net` | `connect(addr) -> TcpStream`, `bind(addr) -> TcpListener`, `bind_udp`, `resolve(host)`, `get(&Url, deadline) -> Result<Vec<u8>, NetError>` | `only(["host:port", ..])` |
| `Clock` | `root.clock` | `now() -> Instant`, `system_time() -> SystemTime`, `sleep(d)`, `sleep_until(t)` (both `-> Result<(), Cancelled>`) | — |
| `Rand` | `root.rand` | `next_u64()`, `gen_range(a..b)`, `gen_bool(p)`, `fill_bytes(&mut buf)`, `shuffle(&mut xs)` | — |
| `Env` | `root.env` | `args()` (iterator of `String`), `var(k) -> Result<String, VarError>`, `vars()`, `current_dir()` | `only(["HOME", ..])` |
| `Stdin` | `root.stdin` | `read_line(&mut s)`, `lines()`, `read_to_string(&mut s)`, `read_to_end(&mut v)` | — |
| `Proc` | `root.proc` | `command(prog) -> Command` (Rust's builder: `.arg`, `.env`, `.current_dir`, `.output()`, `.status()`, `.spawn()`), `exit(code)` | `only(["git", ..])` |
| `Tasks` | `root.tasks` | `scope(\|s\| ..)`; `Scope::spawn`, `set_deadline`, `cancel` (7.1) | — |
| `Shared<T>` | `Shared::new(v)`, `T: Send + Pure` | `lock (c) \|x\| { .. }` (7.3); `clone()` makes another handle | — |
| `Accel` | `root.accel` | Bend kernel calls (D16) | thread budget |

Method names other than those in the design (`dir`, `only`, `get`, `now`) are
provisional; the std reference is authoritative. `Instant`, `Duration`,
`SystemTime`, `Path`, `TcpStream`, `File` are ordinary types, as in Rust.
Capability values are not `Copy`; they are `Clone` (a clone has the same
authority). Pass them by reference: `fs: &Fs`.

### 6.2 `main(root: Root)`

```ferox
fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let data = root.fs.dir("/var/lib/app", Mode::ReadWrite)?;   // Fs below one directory
    let net = root.net.only(["db.internal:5432"]);               // Net to one host
    app::run(&data, &net, &root.clock)?;
    Ok(())
}
```

- `main` may take `root: Root` (or nothing). `main` returns `()`,
  `Result<(), E: Debug>`, or `ExitCode`, as in Rust.
- `Root` can be named only in `main`'s signature. It cannot be passed,
  borrowed into a call, or stored (`FX0503`); move or borrow its fields
  instead (`root.fs`, `&root.clock`).
- `Root::sim(seed)` exists only in test code and under `ferox sim`
  (`FX0504`).

### 6.3 Attenuation

Attenuation is a method returning a narrower capability of the same type. Do
it in `main` and hand each subsystem only what it needs. Paths given to an
attenuated `Fs` are relative to its directory; `..` cannot escape it. A
`Mode::Read` `Fs` returns `PermissionDenied` on writes. `ferox review` shows
where each capability reaching a function was attenuated.

### 6.4 No globals

Safe code has no mutable globals and no thread-locals: no `static mut`, no
`static` of a type with interior mutability (`AtomicUsize`, `Mutex`,
`OnceLock`, `LazyLock`, `Cell`), no `thread_local!`, no `lazy_static!`
(`FX0502`). `static` is allowed for `Frozen + Pure` data; `const` as Rust. Fix:
build the value in `main` and pass it down, or put it in an app-context
struct (6.10).

### 6.5 Write-only sinks are not authority

`print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`, `error!`..`trace!`, and
`std::trace::span` write to sinks owned by the current task and configured by
`main`. They cannot read anything back, they need no capability, and they do
not affect purity. Inside `par`, their output is buffered and emitted in
sequential order.

```ferox
fn decode_user(row: &Row) -> Result<User, DecodeError> {      // still authority-free
    if row.version < 3 {
        warn!("legacy row format for user {}", row.id);
    }
    ...
}

let _span = std::trace::span("load_user");   // closes on drop (provisional)
```

- Sink use is reported as an exercised effect in the trust surface, so a
  dependency that starts logging is visible in review.
- `Secret<T>` refuses `Debug` and `Display`, so a secret cannot be logged or
  printed by accident; read it with `expose()`. (provisional method name)
- `dbg!` is an error in release builds (`FX0507`).
- `println!` in a library package is a lint (`FX0508`); return data or log.
- `main` configures sinks through `root.log` (level, format, destination);
  default: `info` and above to stderr. (provisional)
- `std::io::stdout()` and `stderr()` return write-only `impl Write` handles to
  the same sinks, for raw bytes. (provisional)

### 6.6 `Pure` and `Frozen`

Two compiler-known auto traits, usable as bounds:

- **`Pure`**: no capability, synchronization handle (channel ends, `Shared<T>`
  handles, `Task`/`Scope` handles), atomic (provisional), or foreign handle is
  reachable from the type.
- **`Frozen`**: no interior mutability reachable through shared references
  (no `Cell`, `RefCell`, atomics).

A function is **authority-free** if all its parameter and capture types are
`Pure`. It cannot observe nondeterminism, never suspends, and can run in
`par` (with `Frozen` captures) and be proved. Memoization and `const fn` also
forbid `&mut` parameters. A call through a closure or `dyn` value may suspend
exactly when that value's type is not `Pure`. Generic functions are
authority-free per instantiation; require it with `F: Fn(&T) -> U + Pure`.
`#[pure]` on a function asserts authority-freedom and makes losing it a
compile error, which is useful on public APIs. (provisional)

### 6.7 Foreign calls carry authority

`extern` blocks are allowed only in packages granted `ffi` (12.4). Raw
foreign functions are `unsafe` to call. Every safe function that calls one
must either take the capabilities the foreign code may use, or be marked
`#[trusted_pure(reason = "...")]`, which counts as a trust hole (`FX0509`).

```ferox
extern "C" {
    fn ZSTD_compress(dst: *mut u8, cap: usize, src: *const u8, len: usize, level: i32) -> usize;
    fn fsync(fd: i32) -> i32;
}

#[trusted_pure(reason = "zstd compression is a function of its input buffer")]
pub fn compress(src: &[u8], level: i32) -> Vec<u8> {
    let mut dst = vec![0u8; src.len() + 1024];
    // SAFETY: dst and src are valid for the given lengths.
    let n = unsafe { ZSTD_compress(dst.as_mut_ptr(), dst.len(), src.as_ptr(), src.len(), level) };
    dst.truncate(n);
    dst
}

pub fn sync(_fs: &Fs, file: &File) -> io::Result<()> {      // takes the authority it uses
    // SAFETY: the descriptor is owned by `file` and open.
    let rc = unsafe { fsync(file.as_raw_fd()) };
    if rc == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}
```

### 6.8 Testing with `Root::sim(seed)`

`Root::sim(seed)` returns a `SimRoot` with the same capability fields
(`sim.fs`, `sim.net`, `sim.clock`, ...), all deterministic, plus `sim.world`
for setting up the simulated world (provisional API):

| `sim.world` method | Effect |
| --- | --- |
| `file(path, contents)` | create a file in the in-memory filesystem |
| `args([..])`, `env_var(k, v)`, `stdin(bytes)` | program inputs |
| `http("host:port", \|req\| Reply::ok(body).after(d))` | a simulated HTTP peer; `Reply::status(503)`, `Reply::hang()` |
| `faults(Faults { net_drop: 0.05, disk_error: 0.01, ..Faults::none() })` | fault injection |

`sim.net.drop_all()` returns a `Net` whose packets are all dropped. Virtual
time advances only when every task is blocked, so a 30-second timeout test
runs instantly. A global deadlock is reported with the blocked-task graph.

```ferox
#[test]
fn times_out_when_network_is_down() {
    let sim = Root::sim(42);
    let net = sim.net.drop_all();
    let url = Url::parse("https://example.com/").unwrap();
    let result = fetch_feed(&net, &sim.clock, &url);
    assert!(matches!(result, Err(FetchError::Net(NetError::Timeout))));
}
```

A `#[test]` function may take parameters; it becomes a property test whose
inputs are generated (and shrunk). A `seed: u64` parameter runs the test
under many seeds. (provisional) `ferox test` reports the failing seed and a
replay command; `ferox test --trace` prints the capability event log;
`ferox sim <binary> -- args` runs `main` under many seeds.

### 6.9 Ambient Rust calls and their fixes

All are `FX0501`; the fix uses a capability already in scope, or proposes
adding the parameter.

| Rust | Ferox |
| --- | --- |
| `std::fs::read_to_string(p)` | `fs.read_to_string(p)` |
| `File::open(p)` / `File::create(p)` | `fs.open(p)` / `fs.create(p)` |
| `std::env::args()` / `env::var(k)` | `env.args()` / `env.var(k)` |
| `Instant::now()` / `SystemTime::now()` | `clock.now()` / `clock.system_time()` |
| `std::thread::sleep(d)` | `clock.sleep(d)?` |
| `rand::thread_rng().gen_range(0..n)` | `rand.gen_range(0..n)` |
| `std::io::stdin()` | `root.stdin` / `stdin: &Stdin` |
| `Command::new("git")` | `proc.command("git")` |
| `TcpStream::connect(a)` | `net.connect(a)` |
| `std::process::exit(1)` | return `ExitCode::FAILURE` from `main`, or `proc.exit(1)` |

`HashMap::new()` uses a fixed hash key, so iteration order is deterministic
(but unspecified). For untrusted keys, use
`HashMap::with_hasher(RandomState::from_rand(&rand))`. A pure seeded PRNG,
`std::rand::Pcg64::seed_from_u64(n)`, needs no capability. (provisional)

### 6.10 Patterns

**Thread a capability.** When a leaf starts needing authority, add the
parameter along the call chain. Do not reach for a global or `Root`.

```ferox
// decode_user now reads a schema file:
fn handle(req: &Request, db: &Db, fs: &Fs) -> Result<Response, AppError> { .. load_user(db, fs, id)? .. }
fn load_user(db: &Db, fs: &Fs, id: UserId) -> Result<User, DbError> { .. decode_user(fs, &row)? .. }
fn decode_user(fs: &Fs, row: &Row) -> Result<User, DecodeError> { .. }
```

`ferox fix --thread Fs --to decode_user` makes this edit along the shortest
path from a holder, including test call sites (which get `sim` capabilities).
Logging never needs threading (6.5).

**App-context struct.** For service code, bundle *attenuated* capabilities
and clients built in `main`:

```ferox
pub struct App {
    db: Db,          // holds its own attenuated Net
    files: Fs,       // /var/lib/app only
    clock: Clock,
}

impl App {
    pub fn handle(&self, req: &Request) -> Result<Response, AppError> {
        let user = load_user(&self.db, req.user_id)?;       // pass narrow pieces down
        let body = render(&user, self.clock.system_time());  // pure core takes data
        Ok(Response::ok(body))
    }
}
```

Rules of thumb: never store `Root` (impossible); keep business logic in
authority-free functions that take data; pass leaves the single capability
they use. The bundle does not hide authority: tooling computes the
**exercised** authority of every function (which capabilities its body
reaches), shows changes in `ferox review`, and lints a parameter that grants
more than is exercised (`FX0510`, fix: narrow `&App` to `&Clock`).

## 7. Concurrency

There is no `async`/`await`, no `Future`, no `Pin`, and no executor choice in
library code. A function suspends only inside a capability operation (or a
call through a non-`Pure` closure or `dyn` value); an authority-free function
never suspends. Write ordinary blocking-style code.

### 7.1 Scopes and tasks

Tasks are spawned into scopes from `Tasks` and cannot outlive them. Children
may borrow from the parent. The API (signatures provisional):

| Item | Signature |
| --- | --- |
| `Tasks::scope` | `fn scope<T, E: From<TaskError>>(&self, f: impl FnOnce(&Scope<E>) -> Result<T, E>) -> Result<T, E>` |
| `Scope::spawn` | `fn spawn<U: Send>(&self, f: impl FnOnce() -> Result<U, E> + Send) -> Task<U, E>` |
| `Task::join` | `fn join(self) -> Result<U, E>` |
| `Scope::set_deadline` | `fn set_deadline(&self, at: Instant)` |
| `Scope::cancel` | `fn cancel(&self)` |
| `TaskError` | `enum TaskError { Panicked(String), Cancelled }` (implements `Error`) |

Semantics:

- `scope` returns only after every child has finished. Joining a scope
  cannot be cancelled.
- Children share the scope's error type `E`. A child **fails** when it
  returns `Err` or panics. The first failure cancels the scope (siblings and
  the scope body), and `scope` returns that error after all children finish.
  A panic arrives as `TaskError::Panicked(msg)` converted into `E`.
- To collect per-task errors without cancelling siblings, return them as
  data: `s.spawn(|| Ok(fetch(url)))` gives `Task<Result<Page, NetError>, E>`.
- `join` returns the child's result; dropping a `Task` without joining is
  allowed (its failure still propagates through the scope).
- The task, not the OS thread, is the isolation unit. Values that never
  leave a task need not be `Send` (an `Rc` inside one task is fine). Values
  moved or borrowed into `spawn` must be `Send`/`Sync` as in Rust.
- A task holding a `ThreadBound` value is pinned to its thread while it holds
  it. Foreign callbacks cannot suspend.

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

**Long-lived background work** goes into a scope created high up and passed
down as `&Scope<E>`. A closure spawned into a scope that outlives the current
function is detached work and must be `Owned`: move owned data in
(`FX0203`).

### 7.2 Cancellation and deadlines

Cancellation is level-triggered. Once a scope is cancelled (first failure,
`cancel()`, or its deadline passing), every capability operation inside it
returns an error whose `is_cancelled()` is true (`NetError::Cancelled`,
`io::ErrorKind::Cancelled`, `RecvError::Cancelled`, `Cancelled` from
`clock.sleep`) (provisional names). Suspended work unwinds through ordinary
`Err` returns and destructors run. Operations that consume a value return it
inside the error (`tx.send(v)` → `Err(SendError(v))`). Pure computation has no
cancellation points; it observes cancellation at its next capability call.
Deadlines are `Instant`s from a `Clock`, enforced by the runtime's clock;
under `Root::sim` they are virtual.

### 7.3 Channels and `lock`

Shared mutable state is either message passing or `lock` over `Shared<T>`.

**Channels** are Rust's `std::sync::mpsc` (`channel()`, bounded
`sync_channel(n)`, `Sender: Clone`, `Receiver` iterates until all senders
drop), with cancellation as in 7.2. (provisional) Under simulation the
interleaving of senders is chosen by the seed. To wait on several sources
(Rust's `select!`), send one enum through one channel.

**`lock`** replaces `Arc<Mutex<T>>`:

```ferox
struct Account { balance: i64 }

fn transfer(a: &Shared<Account>, b: &Shared<Account>, amount: i64) -> Result<(), BankError> {
    lock (a, b) |x, y| {                     // x: &mut Account, y: &mut Account
        if x.balance < amount {
            return Err(BankError::Insufficient);   // returns from the body
        }
        x.balance -= amount;
        y.balance += amount;
        Ok(())
    }?                                       // outer ?: Err(Poisoned) → BankError via #[from]
}
```

- `lock (c1, c2, ..) |x1, x2, ..| body` acquires all cells atomically in a
  global order, so it cannot deadlock. Acquisition is not a cancellation
  point. Payloads must be `T: Send + Pure`, so a cell never holds a
  capability, channel end, or `Shared` handle (`FX0606`).
- The body behaves like a closure: `return` and `?` leave the body, not the
  function. `lock` evaluates to `Result<R, Poisoned>` where `R` is the body's
  type.
- The body may call only authority-free functions and may not contain
  `lock`: no capability operations, no channel operations, no suspension
  (`FX0603`). Logging is allowed. A cell's invariant is its payload type's
  invariant.
- Passing two handles to the same cell panics (aliasing is checked at run
  time). (provisional)
- A panicking body poisons every cell it held; later `lock`s return
  `Err(Poisoned)`. `cell.reset(value)` replaces the contents and clears the
  poison. (provisional)
- `Shared<T>` is a cloneable handle (like `Arc<Mutex<T>>`); children can also
  borrow `&Shared<T>`. `lock` is a contextual keyword: `.lock()` method names
  stay legal.

Atomics exist in `std::sync::atomic` for library authors; they are not
`Pure`. There is no `Mutex`/`RwLock` in safe std. (provisional)

### 7.4 Runtimes

The binary chooses the runtime in its manifest: `threads` (default),
`blocking`, `evented`, or the simulator under tests and `ferox sim`.
(provisional default) Libraries never choose and compile the same way for all.

### 7.5 Porting async Rust

| Rust | Ferox |
| --- | --- |
| `async fn f(..) -> T` | `fn f(net: &Net, ..) -> T` (take the capabilities it uses) |
| `x.await` | `x` (`FX0601` safe fix deletes `.await` and `async`) |
| `#[tokio::main] async fn main()` | `fn main(root: Root)` |
| `tokio::spawn(fut)`, `thread::spawn` | `tasks.scope(\|s\| { s.spawn(\|\| ..); .. })` (`FX0602`) |
| `tokio::join!(a, b)` | two `spawn`s and two `join`s (7.1) |
| `tokio::time::timeout(d, f)` | `s.set_deadline(clock.now() + d)` or an operation deadline |
| `tokio::select!` | one channel carrying an enum; deadlines for timeouts (`FX0605`) |
| `Arc<Mutex<T>>`, `m.lock().unwrap()` | `Shared<T>`, `lock (m) \|x\| { .. }?` (`FX0604`) |
| `rayon::join`, `par_iter().map()` | `join`, `par_map` (section 8) |

## 8. Parallelism

Deterministic parallelism for authority-free code:

```ferox
use std::par::{join, Sum};

let (l, r) = par { checksum(&data[..mid]), checksum(&data[mid..]) };   // tuple of results
let (a, b) = join(|| parse(x), || parse(y));
let lens: Vec<usize> = words.par_map(|w| w.len());                    // == words.iter().map(..).collect()
pixels.par_for(|p| *p = p.saturating_add(10));                       // over &mut [u8], disjoint
let total: Sum<u64> = sizes.par_map(|&n| Sum(n)).par_reduce();
```

- `par { e1, e2, .. }` evaluates the expressions potentially in parallel and
  yields a tuple. (provisional) `join`, `par_map`, `par_for`, `par_reduce`,
  `par_chunks_mut` are std functions and slice methods. (`par_for` on
  `&mut [T]`, provisional)
- Closures and expressions must be authority-free (`FX0610`) and `Send`, and
  may capture shared data only if it is `Frozen` (`FX0611`), or hold disjoint
  `&mut` splits (`split_at_mut`, `par_chunks_mut`).
- `par_map` and `par_for` equal their sequential versions.
- `par_reduce` works on a slice of `M: Combine` (5.3) and is *defined* by a
  canonical balanced grouping that depends only on the input length, so
  results are bitwise identical on any core count. When `M: Monoid` and its
  associativity law is proved or tested, the result also equals the
  sequential fold. Std provides `Sum`, `Product`, `Min`, `Max`; `Sum<f64>` is
  only `Combine`, so a float sum is the canonical grouping and can differ
  from `iter().sum()`.
- Granularity is the runtime's job: `par` costs about a branch until promoted.
  Under simulation, `par` runs on real cores.
- A panic in a parallel branch abandons the enclosing task. If several
  branches panic, the one first in sequential order is reported.

## 9. Errors and panics

- `Result<T, E>`, `?`, `From` conversion, `Box<dyn Error>`, `Option` and `?`,
  `unwrap`/`expect`, `map_err`, `ok_or`: as Rust.
- Results are must-use: an ignored `Result` is an error (`FX0702`). Write
  `let _ = r;` to discard on purpose. (provisional: Rust warns)
- `#[derive(Error)]` (2.2) replaces `thiserror`; `#[from]` replaces
  hand-written `impl From<X> for E` (`ferox fix` rewrites trivial ones).
- **Panics** (`panic!`, overflow, out-of-bounds, failed `assert!`, `unwrap`
  on `None`) abandon the current task: it unwinds and runs destructors. There
  is no `catch_unwind` (`FX0701`); isolate risky work in a child task. A
  panic in the main task ends the program with exit code 101.
- A scope observes a child's panic as `Err(TaskError::Panicked(msg).into())`.
- **Moved-out rule.** On the `Err` path of a scope or `join` (where a child
  may have panicked), every place a child borrowed mutably is treated as moved
  out: reassign it before use (`FX0703`). Every `Shared` cell the child held
  locked is poisoned. No half-updated data is ever observed.

```ferox
let mut stats = Stats::default();
let r = tasks.scope(|s| {
    s.spawn(|| { tally(&mut stats, &rows); Ok(()) });
    Ok(())
});
match r {
    Ok(()) => println!("{}", stats.total),
    Err(e) => {
        // println!("{}", stats.total);   // FX0703: `stats` may be half-updated
        stats = Stats::default();          // reassign, then it is usable again
        error!("tally failed: {e}");
    }
}
```

- `#[should_panic(expected = "...")]` works as in Rust.

## 10. Contracts

Contracts are written in the grammar, not in macros. One spec serves three
uses: run-time checks, property tests, and SMT proofs.

### 10.1 Clauses

Clauses go after the signature (and after any `where` clause), before the
body. Loop clauses go between the loop header and its body. Repeated clauses
are conjoined.

| Clause | On | Meaning |
| --- | --- | --- |
| `requires e` | fn | precondition: the caller's obligation |
| `ensures e` | fn | postcondition |
| `invariant e` | loop | holds on entry, after each iteration, and at exit |
| `decreases e` | fn, loop | termination measure (`nat`, or a tuple compared lexicographically) |
| `diverges` | fn, loop | may run forever (servers, event loops); a trust hole that needs approval |

Inside `ensures`:

- `result` is the return value (a contextual keyword; `let result` elsewhere
  is fine);
- `old(x)` (or `old(*x)`) is the entry value behind a `&mut` parameter `x`;
- `final(x)` (or `final(*x)`) is the value a `&mut` holds when its borrow
  ends: at return for a parameter, later for a returned or stored `&mut`.
- A bare `x` or `*x` of a `&mut` parameter in `ensures` is an error
  (`FX0801`): it is ambiguous, so write `old(..)` or `final(..)`. Methods
  auto-deref as usual: `old(v).len()`, `final(v)[i]`.

```ferox
pub fn push_all(dst: &mut Vec<i64>, src: &[i64])
    ensures final(dst).len() == old(dst).len() + src.len()
    ensures forall |i: usize| i < src.len() ==> final(dst)[old(dst).len() + i] == src[i]
{
    for x in src { dst.push(*x); }
}

pub fn slot(v: &mut Vec<u32>, i: usize) -> &mut u32
    requires i < v.len()
    ensures *result == old(v)[i]
    ensures final(v).len() == old(v).len() && final(v)[i] == final(*result)
{
    &mut v[i]
}
```

Trait methods may carry `requires`/`ensures`. An impl may weaken
preconditions and strengthen postconditions, never the reverse (`FX0802`).

### 10.2 Spec language

- Spec expressions are Rust expressions without side effects: `if`, `match`,
  `let`, field access, indexing, slicing, `&&`, `||`, comparisons.
- `==>` is implication; `forall |x: T| e` and `exists |x: T| e` are
  quantifiers. Specs only.
- Integers in specs are mathematical (`int`, `nat`): `a + b` cannot overflow.
  `/` and `%` truncate as in Rust. `x as int` is allowed.
- `==` is model equality: structural, for any type, no `PartialEq` needed.
  Floats in specs are IEEE values, not reals.
- Specs call only `spec fn`s (which must terminate), the built-in spec types
  (`int`, `nat`, `Seq<T>`, `Set<T>`, `Map<K, V>`), and functions marked
  `#[spec]`, which must be authority-free and proved total. Std marks its
  pure accessors (`len`, `is_empty`, indexing, `get`, `contains`, and
  similar), so `xs.len()` and `xs[i]` mean the same in specs as in code.
  `Vec`, slices, and `String` are viewed as `Seq`. (provisional: the `Seq`
  view)
- Spec expressions carry well-formedness obligations, as in Dafny: no
  division by zero, indices in bounds (guard them: `i < xs.len() && xs[i] > 0`).
- Closures appear only as `f.requires((args,))` and `f.ensures((args,), r)`.
- Capabilities have no spec model. A function that uses one may state
  postconditions that hold for every possible response of the world.

```ferox
spec fn sorted(xs: &[i64]) -> bool {
    forall |i: usize, j: usize| i <= j && j < xs.len() ==> xs[i] <= xs[j]
}

spec fn total(xs: &[i64]) -> int
    decreases xs.len()
{
    if xs.len() == 0 { 0 } else { xs[0] as int + total(&xs[1..]) }
}

pub fn read_port(fs: &Fs, p: &Path) -> Result<u16, ConfigError>
    ensures match result { Ok(port) => port >= 1024, Err(_) => true }
{ .. }
```

Executable code cannot call a `spec fn`. Ghost proof steps go in
`proof { .. }` blocks, erased at run time: `assert e;` (no `!`; a proof step)
and calls to `proof fn` lemmas, which have contracts and must terminate.
Quantifier triggers are inferred; when the solver asks, mark one with
`#[trigger]` (`forall |i: usize| i < n ==> #[trigger] xs[i] > 0`).
Nonlinear and bit-vector reasoning goes in marked blocks
(`proof { assert x * y >= 0 by(nonlinear); }`). (provisional spellings)

### 10.3 Modes

Chosen per package in the manifest (`contracts = "test"`) and overridable per
item with `#[contracts(assert | test | prove)]`. (provisional spelling)

| Mode | Run-time checks | Property tests | SMT |
| --- | --- | --- | --- |
| `assert` (default) | `checked` clauses in debug and test builds | — | — |
| `test` | plus `sampled` clauses and loop invariants | `requires` filters and guides input generation; fuzzing; trait laws | — |
| `prove` | as `test` | as `test` | **total correctness**: every panic site (overflow, indexing, `unwrap`, `assert!`) is an obligation; every loop and recursion needs `decreases` or `diverges` |

The compiler classifies every clause and reports the counts; nothing is
skipped silently:

- `checked`: cheap to execute (`xs.len() > 0`);
- `sampled`: executable but expensive (a quantifier over a collection),
  checked only in `test` mode;
- `proof-only`: unbounded quantifiers, never executed.

`for` loops over ranges and slices get their `decreases` inferred.
(provisional) Release builds erase specs unless the manifest keeps them.
Specs never affect typing or code generation, except that proved facts
delete run-time checks; a `pub` precondition that a deleted check relies on
stays checked at function entry. Function bodies are opaque to callers: a
caller sees only the contract.

Spec quality is checked: an unsatisfiable `requires`, an `ensures` that is
trivially true, and contradictory assumptions are reported (`FX0810`,
`FX0811`, `FX0812`), and CI mutation-tests specs.

**Approval.** These changes need a human approval, signed outside the
agent's write scope and verified by CI: weakening or removing a contract,
widening authority, adding a trust hole (including `diverges`), downgrading
an item or package from `prove` to `test` or `assert`, and changing a
`#[protected]` test. Never do any of these to get a green check. If a spec or
protected test looks wrong, stop and report it.

`ferox verify` returns NDJSON: a status per item and obligation (`proved`,
`refuted` with a counterexample, `timeout`, `assumed`, `trusted`).

## 11. Unsafe and trust holes

| Hole | Spelling | Rule |
| --- | --- | --- |
| Unsafe code | `unsafe { .. }`, `unsafe fn`, `unsafe impl Send` | needs the `unsafe` grant (`FX0901`); each block needs a `// SAFETY:` comment (`FX0902`, provisional: error) |
| Trusted contract | `#[trusted(reason = "...")]` on a fn | contract assumed, body not proved |
| Assumption | `assume e;` inside `proof { }` | fact assumed |
| Trusted pure FFI | `#[trusted_pure(reason = "...")]` | 6.7 |
| Foreign call | `extern` | needs the `ffi` grant |
| Possible non-termination | `diverges` | 10.1 |
| Rust crate build script | — | runs sandboxed (12.5) |
| Unproved law impl or dependency contract a proof relies on | — | counted automatically |

The compiler counts holes transitively, per package. `ferox review <base>`
prints the trust diff: signatures, declared and exercised authority,
contracts with proof status, spec coverage, and trust holes, so a reviewer
sees what changed in what code *may do*.

## 12. Modules and packages

### 12.1 Files are modules

- `src/main.fx` is a binary root, `src/lib.fx` a library root.
- Every `.fx` file under `src/` is a module; the directory tree is the module
  tree: `src/net/http.fx` is `crate::net::http`. A directory module may have a
  sibling body file (`src/net.fx` next to `src/net/`), which is optional.
  (provisional) `mod.fx` files are not used.
- You do not write `mod foo;`. If present, it is accepted: plain `mod foo;`
  is a no-op, and `pub mod foo;` / `pub(crate) mod foo;` set the module's
  visibility. A file module without a declaration is private to its parent,
  as `mod foo;` is in Rust. `mod foo;` with no matching file is `FX0910`.
  `#[path]` is `FX0911`. (provisional)
- Inline `mod name { .. }` blocks, including `#[cfg(test)] mod tests`, work
  as in Rust.

### 12.2 Imports

- `use` as Rust, including globs, `as`, nested groups, and `pub use`
  re-exports. `extern crate` is `FX0912` (fix: delete).
- **Unused imports are errors** (`FX0913`, safe fix: delete them).
- **Import cycles between modules are errors** (`FX0914`). Idiom: put shared
  types in leaf modules (`error.fx`, `types.fx`), keep `lib.fx` to re-exports,
  and keep types that refer to each other in one file. A child's
  `use super::*` is fine while the parent does not import the child.

### 12.3 Prelude

Rust's prelude, plus `Root`, the `Error` trait and derive, `Secret`, `Pure`,
`Frozen`, and `Owned`. (provisional) Capability types are imported from std:
`std::fs::{Fs, Mode}`, `std::net::{Net, NetError, Url}`,
`std::time::{Clock, Duration, Instant}`, `std::rand::Rand`, `std::env::Env`,
`std::io::Stdin`, `std::process::Proc`, `std::task::{Tasks, Scope, Task,
TaskError}`, `std::sync::{Shared, mpsc}`.

### 12.4 Manifest and grants

```toml
# ferox.toml (provisional file name and keys)
[package]
name = "acme/linecount"          # names are namespaced
version = "0.1.0"
contracts = "test"               # assert | test | prove
grants = ["unsafe"]              # this package's own escape hatches

[bin]
runtime = "threads"              # threads | blocking | evented

[dependencies]
"acme/json-ext" = "1.2"                                       # no grants: no unsafe, no FFI
"zlib/zstd" = { version = "0.13", grants = ["unsafe", "ffi"] }

[rust-dependencies]
regex = { version = "1", authority = [] }                     # synchronous crate via C-ABI shim

[features]
fast = []
```

- Escape hatches (`unsafe`, `ffi`) are granted per dependency. A dependency
  without a grant cannot use them.
- `ferox.lock` is on by default, with content hashes, and records each
  package's public authority and trust holes. An upgrade that widens either
  fails until a human approves it.
- The client rejects a dependency name that does not exist; it never guesses
  a near name.
- Rust crates are imported through generated shims for synchronous APIs; a
  crate's declared `authority` becomes capability parameters on the shim,
  and a symbol scan of the compiled crate checks the declaration. Async Rust
  crates and lifetime-heavy APIs are out of scope.

### 12.5 No build scripts

Ferox packages have no `build.rs`, install hooks, or code that runs at fetch
or build time (`FX0915`). The one exception is an imported Rust crate's build
script, which runs sandboxed without network, with recorded outputs, and
counts as a trust hole. Code generators are ordinary programs you run
explicitly (for example `src/bin/gen.fx` via `ferox run --bin gen`); commit
the `.fx` files they write. (provisional)

## 13. Toolchain loop

| Command | Use |
| --- | --- |
| `ferox check` | One verdict plus NDJSON diagnostics with stable codes, exact and related spans, and fixes marked `safe` or `suggest`. |
| `ferox fix` | Apply safe fixes; `--thread <Cap> --to <fn>` threads a capability. |
| `ferox test [filter] [--seed N] [--trace]` | Structured results: diff, panic span, seed, shrunk input, replay command, event trace. |
| `ferox sim <bin> -- args` | Run `main` under many seeds. |
| `ferox eval '<expr>'` | Evaluate an expression in the interpreter against the package. Use it to answer "what does this return?" in one call. |
| `ferox verify` | Proofs; NDJSON status per item and obligation, counterexamples on failure. |
| `ferox explain FX0201` | Explanation with a before/after example. |
| `ferox review <base>` | Trust diff (11). |
| `ferox fmt` | The one canonical format; no options. Rewrites legacy lifetime spellings to `from`. |

`#[protected]` tests are part of the trust surface: you cannot change them
without a human approval (10.3).

**Typed holes**: `todo!()` or `_?name` in expression position. `ferox check`
checks the rest of the program and reports each hole's expected type,
in-scope bindings, and candidate functions. Use holes to build a function
top-down.

## 14. Rust habits and their fixes

| Rust habit | Ferox | Code |
| --- | --- | --- |
| `fn f<'a>(x: &'a str, y: &str) -> &'a str` | `fn f(x: &str, y: &str) -> &str from x` | accepted; `ferox fmt` rewrites |
| `struct P<'a> { s: &'a str }`, `impl<'a> P<'a>`, `P<'_>` | `struct P { s: &str }`, `impl P`, `P` | accepted; `ferox fmt` rewrites |
| Two fields sharing `'a` | `y: &str from x` | accepted; `ferox fmt` rewrites |
| `T: 'static` | `T: Owned` | accepted; `ferox fmt` rewrites |
| `'a: 'b`, `trait Foo<'a>`, `for<'a>` | a `from` clause, or restructure | `FX0201` |
| `-> impl Iterator<Item = &T> + '_` | `-> impl Iterator<Item = &T> from xs` | `FX0204` safe |
| `Box<dyn Fn()>` capturing a borrow | `Box<dyn Fn() from x>`, or move owned data in | `FX0203` |
| Self-referential struct | indices, `Arena`, or owner and view apart | `FX0210` |
| `macro_rules!`, proc macros | functions, generics, generators | `FX0101` |
| `#[derive(Serialize, Deserialize)]`, `thiserror::Error` | `Encode`, `Decode`, `Error` | `FX0102` safe |
| `debug_assert_eq!`, `include_str!`, `env!`, other `!` | `debug_assert!(a == b)`, generated `const` | `FX0103` |
| `x as u8` (lossy) | `u8::try_from(x)?` or `x.truncate()` | `FX0301` |
| `n as f64` (`usize`) | `f64::round_from(n)` | `FX0301` |
| `f as i64` | `i64::try_from(f.trunc())?` | `FX0301` |
| Relying on release-mode wrapping | `wrapping_add`, `Wrapping<T>` | run-time trap |
| `impl Display for T` in another module | move it to `T`'s module | `FX0401` |
| Specialization, GATs, `for<'a>`, `impl !Send` | see 5.1 | `FX0402`–`FX0405` |
| `std::fs::read`, `env::args`, `Instant::now`, `thread_rng` | capability methods (6.9) | `FX0501` |
| `static COUNTER: AtomicUsize`, `OnceLock`, `lazy_static!`, `thread_local!` | values built in `main`, passed down | `FX0502` |
| `fn helper(root: &Root)` | pass attenuated capabilities | `FX0503` |
| `Root::sim` outside tests | real capabilities from `main` | `FX0504` |
| `dbg!` left in code | remove, or `debug!` | `FX0507` (release) |
| `println!` in a library | return data, or `info!` | `FX0508` lint |
| `async fn`, `.await`, `#[tokio::main]` | plain `fn` taking capabilities | `FX0601` safe |
| `thread::spawn`, `tokio::spawn` | `tasks.scope(\|s\| s.spawn(..))` | `FX0602` |
| Capability call inside a `lock` body | do it before or after the `lock` | `FX0603` |
| `Arc<Mutex<T>>` + `.lock().unwrap()` | `Shared<T>` + `lock (m) \|x\| { .. }?` | `FX0604` |
| `tokio::select!` | channel of an enum, deadlines | `FX0605` |
| `Mutex<Sender<T>>`, a `Shared` inside a `Shared` | payloads are `Send + Pure`; pass the handle separately | `FX0606` |
| Capturing a capability or `RefCell` in `par` | compute first, pass `Frozen` data | `FX0610`, `FX0611` |
| `catch_unwind` | a child task in a scope | `FX0701` |
| Ignoring a `Result` | `?`, handle it, or `let _ =` | `FX0702` |
| Using a place after a failed child that borrowed it | reassign first | `FX0703` |
| `ensures v.len() > 0` on `v: &mut Vec<T>` | `final(v).len() > 0` or `old(v)` | `FX0801` |
| `unsafe` without a grant or `// SAFETY:` | add the grant in the manifest; write the comment | `FX0901`, `FX0902` |
| `mod foo;` for a missing file, `#[path]` | create `foo.fx`; move the file | `FX0910`, `FX0911` |
| `extern crate foo;` | delete | `FX0912` safe |
| Unused `use` | delete | `FX0913` safe |
| `a.rs` uses `b`, `b.rs` uses `a` | move shared items to a third module | `FX0914` |
| `build.rs` | a generator you run; commit its output | `FX0915` |

## 15. Example programs

### 15.1 CLI: argument parsing and file I/O

`lc [--min N] [--out FILE] FILE...` counts lines with at least `N` characters
in each file and optionally writes the report to a file.

```ferox
// src/main.fx
use std::fs::{Fs, Mode};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Error)]
enum CliError {
    #[error("usage: lc [--min N] [--out FILE] FILE...")]
    Usage,
    #[error("not a number: {0:?}")]
    BadNumber(String),
    #[error("cannot read {path:?}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Default, PartialEq)]
struct Options {
    min: usize,
    out: Option<PathBuf>,
    files: Vec<PathBuf>,
}

/// Authority-free: parses arguments, touches nothing.
fn parse_args(args: &[String]) -> Result<Options, CliError> {
    let mut opts = Options::default();
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--min" => {
                let v = it.next().ok_or(CliError::Usage)?;
                opts.min = v.parse().map_err(|_| CliError::BadNumber(v.clone()))?;
            }
            "--out" => opts.out = Some(PathBuf::from(it.next().ok_or(CliError::Usage)?)),
            s if s.starts_with("--") => return Err(CliError::Usage),
            s => opts.files.push(PathBuf::from(s)),
        }
    }
    if opts.files.is_empty() {
        return Err(CliError::Usage);
    }
    Ok(opts)
}

/// Authority-free.
fn count_lines(text: &str, min: usize) -> usize {
    text.lines().filter(|l| l.chars().count() >= min).count()
}

/// Authority: reads and writes files inside `fs`, nothing else.
fn run(fs: &Fs, opts: &Options) -> Result<String, CliError> {
    let mut report = String::new();
    let mut total = 0;
    for path in &opts.files {
        let text = fs
            .read_to_string(path)
            .map_err(|source| CliError::Read { path: path.clone(), source })?;
        let n = count_lines(&text, opts.min);
        total += n;
        report.push_str(&format!("{n:>8} {}\n", path.display()));
    }
    report.push_str(&format!("{total:>8} total\n"));
    if let Some(out) = &opts.out {
        fs.write(out, report.as_bytes())?;
    }
    Ok(report)
}

fn main(root: Root) -> Result<(), CliError> {
    let args: Vec<String> = root.env.args().collect();
    let opts = parse_args(&args)?;
    let cwd = root.env.current_dir()?;
    let fs = root.fs.dir(&cwd, Mode::ReadWrite)?;     // nothing outside the working directory
    print!("{}", run(&fs, &opts)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_flags() {
        let opts = parse_args(&args(&["lc", "--min", "3", "a.txt"])).unwrap();
        assert_eq!(opts.min, 3);
        assert_eq!(opts.files, vec![PathBuf::from("a.txt")]);
        assert!(matches!(parse_args(&args(&["lc"])), Err(CliError::Usage)));
    }

    #[test]
    fn counts_and_writes_report() {
        let sim = Root::sim(7);
        sim.world.file("/w/a.txt", "one\ntwo\n\nthree\n");
        let fs = sim.fs.dir("/w", Mode::ReadWrite).unwrap();
        let opts = Options {
            min: 1,
            out: Some(PathBuf::from("out.txt")),
            files: vec![PathBuf::from("a.txt")],
        };
        let report = run(&fs, &opts).unwrap();
        assert!(report.ends_with("       3 total\n"));
        assert_eq!(fs.read_to_string("out.txt").unwrap(), report);
    }
}
```

### 15.2 Concurrent fetcher with a deadline and a simulation test

```ferox
// src/main.fx
use std::net::{Net, NetError, Url};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Fetched { bytes: usize },
    Failed(String),
    TimedOut,
}

/// Authority: tasks, network, clock. One task per URL; all share one deadline.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url], budget: Duration)
    -> Result<Vec<Outcome>, TaskError>
{
    let deadline = clock.now() + budget;
    tasks.scope(|s| {
        s.set_deadline(deadline);
        // Each child reports its outcome as data, so one slow host
        // does not cancel the others.
        let handles: Vec<_> = urls
            .iter()
            .map(|url| s.spawn(move || Ok(fetch_one(net, url, deadline))))
            .collect();
        let mut out = Vec::new();
        for h in handles {
            out.push(h.join()?);
        }
        Ok(out)
    })
}

fn fetch_one(net: &Net, url: &Url, deadline: Instant) -> Outcome {
    match net.get(url, deadline) {
        Ok(body) => Outcome::Fetched { bytes: body.len() },
        Err(e) if e.is_cancelled() => Outcome::TimedOut,
        Err(NetError::Timeout) => Outcome::TimedOut,
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let mut urls = Vec::new();
    for arg in root.env.args().skip(1) {
        urls.push(Url::parse(&arg)?);
    }
    let outcomes = fetch_all(&root.tasks, &root.net, &root.clock, &urls, Duration::from_secs(5))?;
    for (url, outcome) in urls.iter().zip(&outcomes) {
        println!("{url}: {outcome:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::Reply;

    #[test]
    fn slow_host_times_out(seed: u64) {          // property over seeds
        let sim = Root::sim(seed);
        sim.world.http("fast.test:443", |_req| Reply::ok("hello").after(Duration::from_millis(20)));
        sim.world.http("slow.test:443", |_req| Reply::ok("late").after(Duration::from_secs(30)));
        let urls = [
            Url::parse("https://fast.test/").unwrap(),
            Url::parse("https://slow.test/").unwrap(),
        ];
        let start = sim.clock.now();
        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls, Duration::from_secs(2)).unwrap();
        assert_eq!(got, vec![Outcome::Fetched { bytes: 5 }, Outcome::TimedOut]);
        assert!(sim.clock.now() - start <= Duration::from_secs(2));   // virtual time: instant
    }
}
```

A failure prints the seed and a command such as
`ferox test slow_host_times_out --seed 1729 --trace`, which replays it bit for
bit and prints the ordered capability events.

### 15.3 Verified pure functions with contracts and a property test

```ferox
// src/search.fx
spec fn sorted(xs: &[i64]) -> bool {
    forall |i: usize, j: usize| i <= j && j < xs.len() ==> xs[i] <= xs[j]
}

/// Proved: total correctness, no overflow, no out-of-bounds.
#[contracts(prove)]
pub fn search(xs: &[i64], target: i64) -> Option<usize>
    requires sorted(xs)
    ensures match result {
        Some(i) => i < xs.len() && xs[i] == target,
        None => forall |i: usize| i < xs.len() ==> xs[i] != target,
    }
{
    let mut lo = 0;
    let mut hi = xs.len();
    while lo < hi
        invariant lo <= hi && hi <= xs.len()
        invariant forall |i: usize| i < lo ==> xs[i] < target
        invariant forall |i: usize| hi <= i && i < xs.len() ==> xs[i] > target
        decreases hi - lo
    {
        let mid = lo + (hi - lo) / 2;        // (lo + hi) / 2 could overflow: an obligation
        if xs[mid] < target {
            lo = mid + 1;
        } else if xs[mid] > target {
            hi = mid;
        } else {
            return Some(mid);
        }
    }
    None
}

/// Property-tested, not proved: `ferox test` generates sorted inputs from
/// `requires` and checks every `ensures`.
#[contracts(test)]
pub fn insert_sorted(xs: &mut Vec<i64>, x: i64)
    requires sorted(xs)                         // entry value: no old() needed
    ensures sorted(final(xs))
    ensures final(xs).len() == old(xs).len() + 1
    ensures exists |i: usize| i < final(xs).len() && final(xs)[i] == x
{
    let at = xs.partition_point(|&y| y < x);
    xs.insert(at, x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_agrees_with_contains(mut xs: Vec<i64>, x: i64) {   // generated inputs
        xs.sort();
        assert_eq!(search(&xs, x).is_some(), xs.contains(&x));
    }

    #[test]
    fn finds_inserted(mut xs: Vec<i64>, x: i64) {
        xs.sort();
        insert_sorted(&mut xs, x);
        assert_eq!(xs[search(&xs, x).unwrap()], x);
    }
}
```

Clause classes here: `sorted` and the loop invariants are `sampled` (checked
in `test` mode, proved in `prove` mode); `lo <= hi && hi <= xs.len()` is
`checked`. `ferox verify` reports each `search` obligation as
`proved`; `insert_sorted` shows as `tested` in the trust surface.

## 16. Choices made in this reference

Each item is marked (provisional) above: open in draft 2.1 and chosen as the
most Rust-like option consistent with its decisions.

1. Surface: const generics over integers, `bool`, `char`; the wider std type
   list in section 1 keeps its Rust API; `log::info!` and `tracing::info!`
   accepted as the built-in logging forms; `std::trace::span` returns a drop
   guard; `#[cfg]` predicates beyond `test`; prelude contents.
2. `#[codec(..)]` with serde's keys for `Encode`/`Decode`, including
   `try_from` for validated decoding.
3. Origins: `'_` accepted as "elided"; a returned multi-origin struct takes
   `from a, b`, with no per-field binding; `&dyn Trait` borrows from the
   reference's origin (Rust's default), and only owning positions default to
   `Owned`; elision still covers plain references and borrowing structs,
   while borrowing `impl Trait` and `dyn` returns need `from` (`FX0204`); in
   `impl Fn(A) -> R from x`, `from` binds to the closure type.
4. Field-borrow syntax `&self.{a, b}` / `&mut self.{a}` (OQ2); the body is
   limited to the listed fields; the list is part of a `pub` method's
   signature.
5. `Arena`/`Handle` method names.
6. Conversions: `usize` treated as 32–64 bits for `as`; `f64::round_from`,
   `f32::round_from`, and `TryFrom<f64>` for integers; `truncate()` takes its
   target from context.
7. "One method per name per type" applies to inherent methods; trait methods
   are namespaced as in Rust.
8. Law clauses may call the trait's own authority-free methods; std splits
   `Combine` (what `par_reduce` needs) from `Monoid` (the laws).
9. Capabilities: method names beyond the design's; capabilities are `Clone`;
   `Fs::dir` returns `io::Result<Fs>`; `clock.sleep` returns
   `Result<(), Cancelled>`; `Shared::new` and channels need no capability;
   `Secret::expose`.
10. Atomics are not `Pure`; no `Mutex`/`RwLock` in safe std.
11. `#[pure]` on functions (OQ4).
12. Sink configuration through `root.log`; `io::stdout()`/`stderr()` as
    write-only sinks.
13. Simulation: `SimRoot` with `sim.world` setup methods, `Reply`, `Faults`;
    `#[test]` functions with parameters are property tests; a `seed: u64`
    parameter runs many seeds.
14. Deterministic default `HashMap` hasher; `RandomState::from_rand`; pure
    `Pcg64`.
15. Scope API: one error type `E: From<TaskError>` per scope, a child `Err`
    counts as a failure, `set_deadline`, `TaskError`, cancellation error
    names.
16. Channels are Rust's `std::sync::mpsc`.
17. `lock` on aliased handles panics; `Shared::reset` clears poison; `lock`
    is a contextual keyword.
18. Default runtime `threads`, chosen in the manifest.
19. `par { .. }` yields a tuple; `par_for` over `&mut [T]`.
20. An ignored `Result` is an error, not a warning.
21. Contracts: the `Seq` view of `Vec`, slices, and `String`; `proof { }`
    blocks, `proof fn`, `assume e;`, `#[trigger]`, `by(nonlinear)`; `for`
    loops infer `decreases`; `#[contracts(mode)]` and `contracts = ".."`
    spellings.
22. A missing `// SAFETY:` comment is an error.
23. File layout (`foo.fx` beside `foo/`, no `mod.fx`); `mod foo;` accepted,
    with `pub mod foo;` setting visibility; `#[path]` rejected.
24. Manifest file name and keys (`ferox.toml`, `grants`, `runtime`,
    `contracts`, `[rust-dependencies]`, `[features]`).
25. Code generators run explicitly; outputs are committed.
26. Diagnostic codes are placeholders.
