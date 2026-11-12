Source: `/home/lucca/.cache/eon-agents/tide-research-yg5/report.md`, 2026-11-10; copied 2026-11-12.

# Research: lossless casts, Arena and `#[pure]`, allowlist audit (fx-yg5.3, fx-yg5.4, fx-yg5.5)

Research agent report, 2026-11-10, against `main` d98ae1a and the pinned `nightly-2026-08-09`.
Nothing in the repository was edited, no `br` write was made, and nothing was built.
`trials/e1-checked/` was not opened. The brief's "fx-08m" is read as these three beads.

**Evidence labels.** *Local* means read in this checkout or in the pinned toolchain's
`rust-src`. *Searched* means a web search this session returned the page. *Memory* means a
well-known documentation URL that I cite without having re-fetched it; treat those as pointers
to verify, not as proof.

## 0. Findings that change the work

1. **The pinned core already has an unstable inherent `truncate` on every integer.**
   `library/core/src/num/uint_macros.rs:4161` (local):
   `pub const fn truncate<Target>(self) -> Target where Self: TruncateTarget<Target>`, feature
   `integer_widen_truncate`, issue 154330. It keeps the low bits, works only within one
   signedness, and has no `u64 -> usize` or `usize -> u32` pair (`core/src/num/traits.rs`,
   `impl_truncate!`). Searched: [tracking issue 154330 in the unstable
   book](https://doc.rust-lang.org/unstable-book/library-features/integer-widen-truncate.html),
   [PR 154356 commit](https://rust.googlesource.com/rust/+/ea421bf991b797fbd574aa138ce4b31ae28cf483).
   The driver passes `-Zallow-features=` (`invocation.rs:246`), so a Tide crate cannot enable it.
2. **`cast_signed`/`cast_unsigned` need no work beyond a test.** They are stable since 1.87
   (local: `#[stable(feature = "integer_sign_cast", since = "1.87.0")]`) and their keys
   (`core::num::<impl i32>::cast_unsigned`) already match `allow("core::num::*")`.
   `conformance/cases/authority/address_casts_lossy.rs` already uses `cast_unsigned`.
3. **`i64::try_from(f.trunc())?` cannot compile.** Pinned core has no `TryFrom<f64>` for any
   integer; `core/src/convert/num.rs` holds only the unstable `FloatToInt` (local). Rust's orphan
   rule forbids the impl outside core
   ([Reference, orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules), memory).
   The spelling must change (§1.4).
4. **`#[pure]` is already enforced in the driver.** `walk.rs:1075` reads
   `std_attr_made(tcx, crates, def, "pure")` and reports `FX0511`; four conformance cases
   (`authority/afree_pure_{accept,generic,capability,nested}.rs`) pass through the stub's
   identity macro. Production only lacks the macro and its prelude export.
5. **`Id<T>` would not be `Pure` for every `T` today.** `pure.rs` decides a std ADT as "`Pure`
   exactly when its type arguments are" (`pure.rs:414-419`). 02 §10.9 and reference §3.6 say
   `Id<T>` is `Pure` for every `T`. `Id<File>` would be rejected. This needs one entry in
   `pure.rs` (§2.3).
6. **Tide's std's own items need no allowlist entry.** `admit_def` admits an unlisted key whose
   role is `TideStd` (`walk.rs:267`), and `admit_impl` admits any impl defined in Tide's std
   (`walk.rs:299`). "Allowlist entries" for `.3` and `.4` therefore reduce to tests.
7. **No conformance case mentions `Arena`, `truncate()` or `round_from`** (local grep). The stub
   `arena` module lacks `get_disjoint_mut`, `len`, `Default`, `contains` and `iter`.

## 1. fx-yg5.3: cast replacements

### 1.1 `truncate()`

Options:

- **A. A prelude trait method with core's exact signature and pair set.** *(recommended)*
- B. An inherent method added by Tide's std with `#[rustc_allow_incoherent_impl]`, the way
  Rust's std adds `f64::floor` (local: `std/src/num/f64.rs:44`). It would collide with core's
  own inherent `truncate` and needs `rustc_attrs`; the stub std already records that it cannot
  add inherent methods (its `ParSlice` comment). Rejected.
- C. Let Tide crates use core's unstable method by enabling the feature in the driver. The
  conformance oracle is stable rustc (`conformance/run.sh:17-19`), where the method cannot be
  enabled. Rejected.
- D. A Tide trait with a wider pair set (cross-sign, `u64 -> usize`). When Rust stabilizes its
  inherent method, the inherent one wins method lookup and every extra pair stops compiling.
  Rejected.

Recommended shape, in a file shared with the stub as `float.rs` is:

```rust
pub trait Truncate: Sized {                       // sealed; in the prelude
    fn truncate<Target>(self) -> Target where Self: TruncateTarget<Target>;
}
```

`TruncateTarget` is a sealed helper, implemented by macro for core's pairs only: each integer to
itself and to every strictly narrower fixed-width integer of the same signedness, and
`usize`/`isize` to themselves and to 16 and 8 bits. The body is `self as Target`. It never fails.

Semantics: two's-complement low bits, as Rust's `as` between integers
([Reference, numeric cast](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast), memory).
`let b: u8 = x.truncate();` infers the target from context, as reference §4.2 says, and
`x.truncate::<u8>()` also works because the parameter is on the method.

Why it resolves today: when an unstable inherent method and a stable trait method share a name,
rustc sets the unstable candidate aside, picks the trait method and emits the
`unstable_name_collisions` lint (searched:
[lint docs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/builtin/static.UNSTABLE_NAME_COLLISIONS.html),
[PR 99898](https://github.com/rust-lang/rust/pull/99898)). I did not run this on the pin; it is
the first thing to falsify (§4). If the lint fires in user crates, have the driver allow that one
lint: Tide pins std and the toolchain together, so the collision is Tide's to resolve at a pin
bump, not the user's.

Consequence for the reference: with core's pair set, `i32 -> u8` is
`x.cast_unsigned().truncate()` and `u64 -> usize` has no `truncate` (use `usize::try_from(n)?`).
Reference §4.2's row reads as if any `x` works. This is silent in the design; I recommend the
same-signedness reading because it is the only one that survives Rust's stabilization. The
`FX0301` fix text belongs to fx-u65.2 and must emit the two-step form for a sign change.

Prior art: Zig separates `@truncate` (low bits) from `@intCast` (checked) and `@bitCast`
([Zig language reference](https://ziglang.org/documentation/master/#truncate), memory); Swift has
`init(truncatingIfNeeded:)`, `init(clamping:)` and `init(exactly:)`
([BinaryInteger](https://developer.apple.com/documentation/swift/binaryinteger), memory); clippy
splits the same hazards into `cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`
and `cast_precision_loss`
([clippy lint list](https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_truncation), memory).

### 1.2 `cast_signed()` / `cast_unsigned()`

Admit Rust's stable inherent methods, as the bead says. No policy change. Semantics: same-width
bit reinterpretation, never fails ([tracking issue 125882](https://github.com/rust-lang/rust/issues/125882), searched).
Add them to the executing test and to `.5`'s admitted list.

### 1.3 `round_from`

An inherent `f64::round_from` is impossible outside core without option B's machinery, and no
`round_from` exists in the pinned core or std (local grep), so a prelude trait has no collision:

```rust
pub trait RoundFrom<T>: Sized { fn round_from(value: T) -> Self; }   // sealed; in the prelude
```

`f64::round_from(n)` and `f32::round_from(x)` resolve as trait associated functions. The key is
`tide_std::…::RoundFrom::round_from`, so the `deny("core::f*::<impl f*>::*")` rule does not
touch it.

Impls (recommended: the pairs where `as` is `FX0301`, no others): `f64` from `i64`, `u64`,
`i128`, `u128`, `isize`, `usize`; `f32` from `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`,
`usize` and `f64`. Exact pairs keep `From` and `as`. Adding them later is compatible.

Semantics: IEEE 754 round to nearest, ties to even; the body is `value as Self`, which Rust
defines the same way (Reference, numeric cast, as above). It never fails and never panics. Two
results to document because "nearest" surprises:

- `f32::round_from(u128::MAX)` is `+infinity`: the Reference notes integer-to-float overflow
  happens only for `u128 as f32` at or above `f32::MAX + 0.5 ULP`.
- `f32::round_from(1e39_f64)` is `+infinity`; NaN stays NaN, with unspecified payload, which
  Tide's observers canonicalize anyway (reference §4.3).

### 1.4 Float to integer (the provisional "float `TryFrom`")

Options:

- A. Keep `i64::try_from(x)` by replacing the prelude's `TryFrom` with a Tide trait. A blanket
  impl over Rust's `TryFrom` plus a float impl overlaps (E0119: upstream may add the impl), and
  user and std `TryFrom` impls would split in two. Rejected.
- B. An incoherent inherent `i64::try_from` that shadows the trait for every call. Same overlap
  problem, changes the error type, internal attribute. Rejected.
- C. A wrapper argument, `i64::try_from(Exact(f.trunc()))`. Legal under the orphan rule, since
  the wrapper is local, but it needs an import and reads worse than D. Rejected.
- **D. A new prelude trait method: `i64::try_from_float(f.trunc())?`.** *(recommended)*
- E. Wait for Rust. Proposals since 2018 have not landed
  ([PR 47857](https://github.com/rust-lang/rust/pull/47857),
  [internals: TryFrom for f64](https://internals.rust-lang.org/t/tryfrom-for-f64/9793),
  [pre-ACP: exact float-int TryFrom](https://internals.rust-lang.org/t/pre-acp-tryfrom-for-exact-float-int-conversions/23828),
  all searched). Rejected.

```rust
pub trait TryFromFloat<F>: Sized {                // sealed; in the prelude
    fn try_from_float(value: F) -> Result<Self, TryFromFloatError>;
}
```

Implemented for every integer type from `f32` and `f64`. `TryFromFloatError` is a field-less
struct with `Debug`, `Display`, `Error`, `Clone`, `Copy`, `PartialEq`, `Eq`, so `?` into
`Box<dyn Error>` works and it is `Pure`.

Semantics (recommended): `Ok` exactly when the value is finite, has no fractional part, and
lies in the target's range; `-0.0` gives `0`. NaN, both infinities, out-of-range and fractional
values are `Err`. The reference names NaN, infinity and range; it is **silent on a fractional
input**. I recommend the exact reading because the taught spelling writes `.trunc()` first,
which would be redundant if the conversion rounded. Swift's `Int(exactly:)` behaves this way
(searched: [init(exactly:)](https://developer.apple.com/documentation/swift/int/init(exactly:)-7qdwf)),
and so does the pre-ACP. For contrast, Rust's `as` saturates and maps NaN to 0 (Reference, as
above), Kotlin's `toInt()` does the same
([Double.toInt](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin/-double/to-int.html), memory),
and Zig's `@intFromFloat` is checked illegal behavior
([Zig reference](https://ziglang.org/documentation/master/#intFromFloat), memory).

Range check: compare against powers of two, which are exact in both float types:
`value >= -2^(N-1) && value < 2^(N-1)` for signed, `value >= 0.0 && value < 2^N` for unsigned,
then `value as Int`, which is exact inside the range. For `f32 -> u128` the upper bound is
infinite, so every finite non-negative integral value passes.

### 1.5 Where the items live, and the design edits

Reference §12.3 gives no path for them and they are prelude items. Recommended: Tide's std
replaces the plain `convert` re-export in `root.rs` with
`pub mod convert { pub use std::convert::*; … }`, as `marker` already does for `Pure`, and adds
`Truncate`, `RoundFrom`, `TryFromFloat` to `declare_prelude!`. One shared source file serves
production and the stub, so fx-nkv's equivalence holds by construction.

Design edits the spelling change needs (a governed, docs-only change; `design/reference.md` is
a single-holder file under the 25,000-token budget): reference §4.2 row 4 and its "provisional
names" sentence; a same-signedness clause on the `truncate` row. `01-design.md` D3 ("`TryFrom`,
`truncate()`, or explicit float rounding methods") stays true if "rounding" becomes
"conversion". Companion line 33 needs no change.

### 1.6 Proof

Conformance stub bodies are `unimplemented!()`, so conformance proves only that the spellings
type-check under stable rustc and are admitted. Execution proof goes in
`crates/tide-admit/tests/build.rs` (a program built through `tide-admit` with the production
std, then run), plus plain unit tests in `tide-std`. One accepted conformance case with every
documented spelling; fx-u65.2 owns the violating `FX0301` cases. `conformance/rules.md` has no
row for the lossless rule except GLOBAL-4, so the case's `//@ rules:` tag needs fx-u65.2's row.

## 2. fx-yg5.4: `std::arena` and `#[pure]`

### 2.1 Representation

The design already fixes it: `Id<T>` is a `u32` index and a `u32` generation, and handles are
generational (01 D2 line 265, 02 §10.9). Not open. What is open:

**Generation overflow.** slotmap's version wraps after about 2^31 reuses of one slot and can
then return the same key twice; `slotmap-careful` exists to retire such slots (searched:
[slotmap-careful](https://lib.rs/crates/slotmap-careful)). generational-arena bumps with a
wrapping add (searched; [docs](https://docs.rs/generational-arena), memory). Recommended: when a
slot's generation reaches `u32::MAX`, `remove` retires the slot instead of putting it on the
free list. A stale `Id` then can never become valid again, which is what "stale handle: `None`,
never a dangling read" promises. Cost: one comparison in `remove`.

**Layout.** `Vec<Slot<T>>`, where a slot holds its generation and either the value or the next
free index; a free-list head; a live count. Reuse is last-freed-first, which is deterministic.
`insert` panics when the slot count would pass `u32::MAX`. No `unsafe` is needed:
`get_disjoint_mut` can validate generations and then call the stable
`slice::get_disjoint_mut` (stable since 1.86, local).

**Cross-arena misuse.** An `Id` from another `Arena<T>` of the same `T` is indistinguishable.
id-arena detects it with a per-arena `u32` taken from a global counter (searched:
[id-arena source](https://docs.rs/id-arena/2.3.0/src/id_arena/lib.rs.html)). la-arena, slotmap
and generational-arena do not (searched:
[la-arena `Idx`](https://docs.rs/la-arena/latest/la_arena/struct.Idx.html);
[slotmap](https://docs.rs/slotmap), memory). The design is **silent**. Recommended: no
detection. A global counter is ambient state in a type that must be `Pure`, and it would widen
`Id` past the two `u32`s the design states. Document it in rustdoc as a logic error with safe
results (`None`, a panic, or another element), never undefined behavior.

### 2.2 API

Reference §3.6 lists it. Exact signatures recommended:

| Item | Signature and behavior |
| --- | --- |
| `new`, `Default` | empty; no `T: Default` bound |
| `insert` | `(&mut self, T) -> Id<T>` |
| `get`, `get_mut`, `contains` | take `Id<T>` by value (it is `Copy`); stale gives `None` / `false` |
| `remove` | `(&mut self, Id<T>) -> Option<T>`; stale gives `None` |
| `get_disjoint_mut` | `<const N: usize>(&mut self, [Id<T>; N]) -> Option<[&mut T; N]>`; `None` if any id is stale or two are equal |
| `len`, `is_empty` | live elements; `is_empty` is not in the reference, it follows Rust's convention |
| `Index`, `IndexMut` | panic on a stale id, with a fixed message a `#[should_panic(expected)]` test can match |
| `iter` | `(Id<T>, &T)` in slot-index order |

The reference shows `get_disjoint_mut([a, b]) -> Option<[&mut T; 2]>`; the const-generic form
is the Rust-like generalization (it matches `slice::get_disjoint_mut` and slotmap) and admits
that call unchanged. `iter` order is **not insertion order** once a slot is reused; the
reference marks `iter` provisional and is silent on order.

`Id<T>`: hand-written `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Debug`
with no bound on `T` (derives would add one; the stub already does this by hand). Marker
`PhantomData<fn() -> T>`, which gives `Send + Sync` for every `T`, covariance, no drop-check
ownership, and `'static` exactly when `T` is, as la-arena does (searched, link above). `Ord`
compares index, then generation.

The stub's `arena` module should become the shared production file (as `float.rs` and `math.rs`
are), which also gives the stub the five missing items.

### 2.3 `Pure`

- `Arena<T>` needs nothing: it is not in `NOT_PURE`, so it is `Pure` exactly when `T` is.
- `Id<T>` needs a `pure.rs` entry that makes it `Pure` whatever `T` is, checked before the
  type-argument walk. Without it, `HashSet<Id<File>>` in a `#[pure]` function is `FX0511`,
  against 02 §10.9. This is data in PURE-1's std table (it is "versioned with Tide's std", like
  `NOT_PURE`), not a change to `FX0511`'s rule, so I read it as inside fx-yg5.4's scope. A
  conformance case should pin it both ways: `Id<Fs>` meets a `Pure` bound, `Arena<Fs>` does not.

### 2.4 `#[pure]`

Meaning (03 §3.2, AFREE-1): it asserts that the marked function is authority-free, that is,
every parameter type including `self` is `Pure`, for every instantiation its bounds allow. It
does not cover items nested in the body. On a foreign function it is always `FX0511`. Nothing
here is open.

Mechanism: move the identity attribute from `conformance/rustc/macros` into `tide-macros` and
export it from the shared prelude in `root.rs`, so both std surfaces get one definition
(AP-OWNERSHIP-001). The driver recognizes it by name and by Tide's std exporting it
(`is_std_attr`), so no driver change is expected. The macro must return its tokens unchanged so
spans survive; `std_attr_made` relies on the expansion data, and already excludes nested items.

Why checking types is enough, compared with other languages: D's `pure` forbids touching
global mutable state and calling impure functions, with "weak" purity allowing mutation through
parameters (searched: [D spec, functions](https://docarchives.dlang.io/v2.096.0/spec/function.html);
[current spec](https://dlang.org/spec/function.html#pure-functions), memory). SPARK states it
as `Global => null` and proves it by flow analysis (searched:
[AdaCore, data dependencies](https://blog.adacore.com/spark-2014-rationale-data-dependencies)).
Koka infers an effect row, with `total` as the empty row (searched:
[Leijen 2013](https://microsoft.com/en-us/research/wp-content/uploads/2016/02/koka-effects-2013.pdf)).
Fortran's `PURE` restricts the body (no I/O, no saved state;
[Fortran wiki](https://fortranwiki.org/fortran/show/pure), memory). All four inspect bodies or
calls because authority is ambient there. Tide removes ambient authority (`FX0501`, `FX0520`,
`Pure` statics), so authority can only arrive through a parameter or capture, and a signature
check is complete. Tide's notion is closest to D's weak purity: a `&mut` parameter is allowed.

**Silent in the design:** `#[pure]` on a non-function (a struct, a module) and `#[pure(args)]`.
Today the driver only looks at functions, so the attribute would be ignored silently.
Recommended: the macro emits a plain `compile_error!` for a target that is not a function or
for a non-empty argument list. That is input validation, with no new FX code.

Out of scope, reported: production also lacks `#[trusted_pure]`, which the driver reads the
same way; the bead names only `#[pure]`.

### 2.5 Proof

- Executing (build.rs, through `tide-admit`, production std): 02 §10.9's graph with a cycle
  runs; `g[stale]` panics; `get(stale)` is `None`; a `#[pure]` function compiles and runs; one
  taking `&Fs` fails with `FX0511`.
- Conformance: a new accepted case from 02 §10.9's own example (design text, not E1); the four
  `afree_pure_*` cases must still pass with the moved macro; the `Id`/`Arena` purity case above.

## 3. fx-yg5.5: the allowlist audit

### 3.1 What the allowlist is

`policy.rs` is about 150 glob rules over rustc's canonical paths, first match wins, with whole
modules allowed (`core::num::*`, `core::iter::*`, `alloc::vec::*`, …) and narrow denies placed
before them. It is not an item list, so "enumerate the kept items" means: for each item family
the reference keeps, confirm that one real use produces a key that an `allow` rule decides.

### 3.2 Method (recommended)

1. **Build the admit list from the reference only**: §1 (standard types and `std::{cmp, ops,
   mem, fmt, convert}`, every `Iterator` adapter), §2.1 (the fixed `!` forms, whose expansions
   name internals), §3–§5 (lifetimes, `Rc`/`Cell`/`RefCell`, `wrapping_*`/`checked_*`/
   `saturating_*`/`overflowing_*`, `Wrapping`, `Saturating`, `try_from`, `cast_signed`, the ten
   kept float methods, `from_bits`, trait and closure forms). Write it as one test crate with
   one use per item, built through `tide-admit` with the production std, expecting no `FX0520`.
   Group it by reference section so a failure names the section.
2. **Build the deny list from §6.9 and §2.1**: `core::arch` intrinsics; a raw pointer and an
   address observer without the grant; `core::sync::atomic`; `OnceLock`, `LazyLock`; each float
   observer and a sample of math methods; `Path::exists`, `Path::metadata`;
   `io::Error::last_os_error`; `include_str!`, `env!`, `option_env!`; `asm!` without the grant.
   Expect `FX0520` with the exact key. `tests/driver.rs` already covers several
   (`rejects_each_use_outside_the_allowlist_at_its_span`); extend that table, do not duplicate.
3. Fix what step 1 finds by adding the narrowest `allow` that covers it, above any deny it must
   carve out of, and add a `first_matching_rule_decides` assertion for each new rule.

The E1 half of Acceptance ("every Tide-arm reference solution that compiles reports no
`FX0520`") is the coordinator's harness run; under the held-out rule it may confirm, never
supply items.

### 3.3 Mechanical enumeration: considered, not recommended now

Rustdoc JSON for `core`, `alloc` and `std` ships as the nightly-only `rust-docs-json` component
(searched: [rustdoc JSON in 2022](https://adotinthevoid.github.io/posts/rustdoc-json-2022/);
[rustdoc-json crate](https://docs.rs/rustdoc-json/latest/src/rustdoc_json/builder.rs.html)). It
is not installed for the pin (local: no `share/doc/rust/json`). `cargo-public-api` consumes the
same JSON to list and diff one crate's public API
([repository](https://github.com/cargo-public-api/cargo-public-api), memory).

Neither fits this audit. Policy keys are rustc's `def_path_str` output
(`core::num::<impl u32>::truncate`, `impl core::fmt::Debug for fn()`), not rustdoc paths, so a
JSON list would need a reimplementation of rustc's path printer before it could be matched. The
JSON format is unstable. Macro-expansion internals (`alloc::intrinsics::write_box_via_move`,
`core::panicking::*`) are not public API and would be missed. A faithful dump would be a driver
mode that prints key and verdict for every item it visits, which is a new feature; nobody needs
it yet. `cargo-public-api` could later guard `tide-std`'s own surface, a different question.

### 3.4 What reading the rules against the reference found

No kept core or alloc family is missing a rule: I traced scalars, `str`/`String`, slices,
arrays, `Option`/`Result`, `Vec`, `Box`, `Rc`, `Cell`/`RefCell`, the ordered collections, `Cow`,
`Path`/`OsStr`, `Duration`, iterator adapters, and the `vec!`/`format!`/`assert!`/`panic!`
expansions to an `allow`. This is a reading, not a run; step 1 decides.

Places the reference is **silent** and the policy denies, to report rather than change:

- Float methods outside the ten kept ones, for example `is_infinite`, `clamp`, `fract`,
  `to_le_bytes`, `from_le_bytes`. §4.3 ends its kept list without "…", so they stay `FX0520`.
- Items owned by other beads: `Instant`/`SystemTime` methods (`std::time::*` has no rule and
  Tide's `time` has only `Clock`), `core::net` types, `core::pin`. They are std, not
  core/alloc, or not in the reference.

## 4. Pitfalls

1. **`truncate` after stabilization.** The inherent method wins silently. With core's pair set
   mirrored the behavior is identical and the trait becomes dead code to delete at that pin
   bump. With any wider pair set it becomes a compile break.
2. **`T::MAX as f64` in a range check.** `i64::MAX as f64` is `2^63`, so `x <= i64::MAX as f64`
   accepts `2^63` and the following `as` saturates silently. Use the exclusive power-of-two
   bound.
3. **Rounding.** `f64::round_from(2^53 + 1)` is `2^53` (tie to even); `f32::round_from(u128::MAX)`
   is infinity. `f.round()` in Rust rounds ties away from zero, so `round()` then
   `try_from_float` is not ties-to-even.
4. **NaN and infinities.** Every comparison with NaN is false, so a check written as
   "reject if `x < lo || x >= hi`" accepts NaN. Write the accepting condition, or test
   `is_finite` first. `-0.0` must convert to `0`, including for unsigned targets.
5. **Stale ids.** Generation wrap (§2.1). An `Id` from another arena. `iter` order after reuse.
   `get_disjoint_mut` with a repeated id must be `None`, not two `&mut` to one value.
6. **Purity holes.** A generic `F: Fn` without `+ Pure` (already `FX0511`,
   `afree_pure_generic.rs`). A `#[pure]` trait method whose `Self` is unbounded. `Id<T>` judged
   by its type argument (§2.3). `#[pure]` on a non-function ignored. A macro that re-spans its
   output would break `std_attr_made` or move the diagnostic.
7. **Allowlist drift at a pin bump.** Keys follow where rustc defines an item. On this pin
   `floor`, `round`, `trunc`, `mul_add`, `sqrt` are std inherent methods that call free
   functions in `core::f64::math` (local), which is why their rules say `std::f*::…`. If a later
   pin moves them into core's inherent impl, the key becomes `core::f64::<impl f64>::floor` and
   the catch-all deny rejects a kept method. The same holds for expansion internals of `vec!`
   and the derives. Section 3.2's two tests are the detector.
8. **Rule order.** A new `allow` placed above a deny it overlaps silently admits denied items;
   placed below, it does nothing.
9. **Shared files.** `.3` and `.4` both touch `root.rs`, the stub std, `tide-macros` and
   `tests/build.rs`; `.3` may also need `design/reference.md`, a single-holder file.

## 5. Cheapest falsifiers

| Recommendation | Test that would falsify it |
| --- | --- |
| Trait `truncate` resolves despite core's unstable inherent one | First, before any other work: a 5-line crate with the trait, on the pin and on stable; `let b: u8 = 376u32.truncate();` must compile and give `120`. Record whether `unstable_name_collisions` fires. |
| `truncate` keeps the low bits | `0x1234_5678u32 -> u16` is `0x5678`; `-1i64 -> i8` is `-1`; `i16::MIN -> i8` is `0`; `300u16 -> u8` is `44`. |
| `cast_*` admitted and bit-preserving | `(-1i32).cast_unsigned() == u32::MAX`; `u8::MAX.cast_signed() == -1`; no `FX0520`. |
| `round_from` ties to even | `f64`: `2^53+1 -> 2^53`, `2^53+3 -> 2^53+4`, `u64::MAX -> 2^64`. `f32`: `16_777_217 -> 16_777_216`, `16_777_219 -> 16_777_220`, `u128::MAX -> inf`, `1e39_f64 -> inf`. |
| `try_from_float` boundaries | `Err`: NaN, `+inf`, `-inf`, `2^63` to `i64`, the next `f64` below `-2^63`, `-1.0` to `u8`, `256.0` to `u8`, `0.5`. `Ok`: `-2^63` to `i64`, the largest `f64` below `2^63`, `-0.0` to `u8` is `0`, `255.0`. |
| Old spelling is really impossible | `i64::try_from(1.0_f64)` fails with E0277 on the pin (so the design edit is needed, not optional). |
| Arena semantics | 02 §10.9's cycle runs; `remove` then `get` is `None`; index panics; reuse of a slot gives a different `Id` and the old one stays stale; `get_disjoint_mut([a, a])` is `None`; a slot forced to generation `u32::MAX` (unit test with a test-only constructor) is not reused. |
| `Id` is `Pure` for every `T` | A `Pure` bound on `Id<Fs>` passes and on `Arena<Fs>` fails; without the `pure.rs` entry the first one fails. |
| Production `#[pure]` reaches the driver | The four `afree_pure_*` conformance cases unchanged, plus one build.rs program through the production std with one `FX0511` and one passing function. |
| The allowlist matches the reference | §3.2's two tests; then delete one `allow` (for example `core::num::*`) locally and confirm the admit test fails, so it is not vacuous. |

## 6. Where the design is silent, in one list

1. `truncate` across signedness or to `usize`: recommend core's pairs only (§1.1).
2. Whether float-to-integer conversion rejects a fractional value: recommend yes (§1.4).
3. The float-to-integer spelling: must change; recommend `i64::try_from_float(x)` (§1.4).
4. Which pairs `round_from` covers: recommend the lossy ones only (§1.3).
5. The module path of the new traits and the error type: recommend `std::convert` (§1.5).
6. Generation overflow: recommend retiring the slot (§2.1).
7. Cross-arena ids: recommend no detection, documented (§2.1).
8. `iter` order, `Ord` on `Id`, `is_empty`, const-generic `get_disjoint_mut` (§2.2).
9. `#[pure]` on a non-function: recommend a plain compile error from the macro (§2.4).
10. Float methods beyond the ten kept: recommend they stay `FX0520` (§3.4).

None of these adds a language rule; 2, 3 and 1 change reference §4.2's wording and go through
GOVERNANCE.md's design review.

DONE
