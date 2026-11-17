Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 6a30b7c

# Opus review r2: fx-qlek.5 (red-team admission and the RUN-3 sandbox)

## Verdict

request_changes

One blocking finding, B1. FX0402 only checks impls whose self type is a bare type parameter
(`ty::Param`). An impl whose self type is a projection is also generic over an unsized type, and
it can reach the same overlap with the built-in `dyn` impl. The docs this change edits say it
closes "the trait-object route of rust-lang/rust#57893", so the claim would be wrong as written.
I could not compile the probe (no rustc runs, per the brief), so B1 is PLAUSIBLE, not CONFIRMED.
It costs one conformance case to settle, and widening the check is a one-line change.

## Scope

- I reviewed `git diff fab99b3...6a30b7c`. Its SHA-256 matches the brief
  (`50a6ba5a…aee97f9`), and `git diff --check` is clean.
- I read the bead (Scope and Acceptance), AGENTS.md's governance summary, the red-team record,
  Codex r1 and its re-check, and the builder report's FX0402 section.
- I read every new conformance case, `walk/traits.rs` (the whole `ceiling_item` impl arm) and
  its caller (`walk.rs` `visit_item`, `nested_filter::All`), and `policy.rs` (key glob and
  `lookup`). I also read `crates.rs` `key`, `walk.rs` `impl_key`, `tide-std` `root.rs`
  re-exports, the `?Sized` blanket impls in `tide-std`, the conformance stub std's root, and
  `invocation/sandbox.rs` (`close_range` and the ioctl allowlist).
- I read the pinned nightly's `library/std/src/path.rs`, one file under
  `~/.rustup/toolchains/nightly-2026-08-09-…`, to check the `std::path` list.
- No cargo, rustc, tide-admit or conformance runs. I changed no file other than this report.

## Blocking findings

### B1. FX0402 misses impls whose self type is a projection (PLAUSIBLE)

`crates/tide-admit/src/walk/traits.rs:138`
(`&& matches!(self_ty.kind(), ty::Param(_))`)

**Mechanism.** The 57893 overlap needs an impl whose self type can become `dyn Trait`. Today's
check covers a bare parameter. A projection self type such as `<P as Tr>::Assoc` can also
normalize to `dyn Object<…>`:

- The parameter is constrained through the trait's arguments, so rustc reports no E0207.
- rustc's E0371 check (`coherence/mod.rs::check_object_overlap`) inspects the unnormalized
  self type, so it does not see the `dyn`.
- Coherence never compares impls against the built-in object impl. That gap is the bug itself.

**Failure scenario** (a minimal program I expect to be admitted at 6a30b7c):

```rust
use std::marker::PhantomData;

pub trait Tr { type Assoc: ?Sized; type Out; }
pub trait Object<P: ?Sized> { type Output; }

// Self type is a projection, not ty::Param, so FX0402 skips it. P is constrained by Object<P>.
impl<P: Tr + ?Sized> Object<P> for <P as Tr>::Assoc {
    type Output = <P as Tr>::Out;
}

// In this generic context only the impl applies: Output = <P as Tr>::Out.
fn convert<P: Tr + ?Sized>(x: <<P as Tr>::Assoc as Object<P>>::Output) -> <P as Tr>::Out { x }

pub struct Key<T, U>(PhantomData<(T, U)>);
impl<T, U> Tr for Key<T, U> {
    type Assoc = dyn Object<Key<T, U>, Output = T>;
    type Out = U;
}

// At the call, the object candidate wins projection (project.rs prefers a single object
// candidate), so the argument type is T, while the return type is U.
pub fn transmute<T, U>(x: T) -> U { convert::<Key<T, U>>(x) }
pub fn forge(fs: &u64) -> &std::fs::Fs { transmute(fs) }
```

The other checks do not catch it:

- FX0401: the self type is not an ADT, and the trait is local, so placing the impl in the
  trait's module passes.
- FX0406: the bound `P: Tr` is on a parameter.
- FX0403 (GATs): nothing generic here.

**Uncertainty.** I could not check two steps without running rustc. First, that rustc accepts
the impl header. Second, that the generic `convert` resolves the impl candidate against the
rigid projection `<P as Tr>::Assoc` in the old solver. A concrete variant,
`impl Object<K> for <K as Tr>::Assoc { type Output = u8; }`, at least shows the E0371 bypass, but
it needs the generic form to become a transmute.

**Fix.**

- Widen the self-type test to `ty::Param(_) | ty::Alias(..)`. Keep the `is_sized`,
  dyn-compatible and own-associated-type conditions. An impl on a projection is rare, and
  rejecting one costs little.
- Add the program above to `memory/escape_dyn_overlap_transmute` with `//~ FX0402` on the impl.
  If rustc rejects it, keep the case as a witness that the route is closed.
- If the check widens, add "or a projection" to the rule text in README:94,
  conformance/README.md:146-152, rules.md REF-0402 and 03 RUN-4 hole 5.

**Why it blocks under the tier rule.** The fix changes REF-0402's check and the v0-core claims
that this change itself edits: README:18 and :94, 01 D7:584, 02 R3:1442, 03 RUN-4:883,
rules.md:171, and the record's "Every other admission route is rejected" (line 12). So it is not
later-tier text the change leaves alone. The bead exists because "a reader who finds the next
one at launch sinks the alpha". A curious reader who knows the issue will try a variant of the
reproducer within an afternoon, and moving the self type behind a projection is the obvious
variant to try.

## Non-blocking findings

**N1. FX0402 rejects more than it needs to, and its message overstates the overlap.**
`traits.rs:146`.

- An impl such as `impl<T: ?Sized + Hash> Key for T { type Digest = u64; }` is rejected. No
  `dyn` type can satisfy its bound, because `Hash` is not dyn-compatible, so no built-in impl
  overlaps it.
- Failing closed is defensible here. A user subtrait can make any dyn-compatible bound
  satisfiable, so a more precise rule would have to check that every bound is a dyn-compatible
  trait or an auto trait.
- The diagnostic still says "overlaps the built-in impl" when no overlap may exist. Suggested
  wording: "may overlap". This is text only.

**N2. Many `std::path` keys are tested only as strings, not through a driver run.**

- The pinned `path.rs` agrees with the list: every stable inherent item, type, variant, const
  and free function is there, and every unlisted item is unstable. This matches the record.
- Real-std driver runs exercise `join`, `with_extension`, `push`, `file_name`, `components`,
  `ancestors`, `strip_prefix`, `is_absolute`, `starts_with`, `is_empty` and `into_string`. They
  come from `escape_path_io::pure`, through the stub's `pub use std::path`, and from the
  reference allowlist row.
- The variant keys (`Component::Normal`, `Prefix::*`) and the `Prefix*::is_verbatim` and
  `PrefixComponent*::*` globs are tested only by string `lookup`. If rustc prints a key in
  another shape, a correct `match c { Component::Normal(s) => … }` gets FX0520.
- Fix: add one `match` on `Component` to `pure` (or to the reference row).
- Trait-impl uses (`PathBuf::from`, `AsRef<Path>`, `Deref`) are keyed `impl … for …` and never
  matched `std::path::*`, so removing the glob does not change their verdict.

**N3. `design/reference.md` §5.1 does not name the narrowed FX0402 rule.**

- The record defers this because the file is single-holder. That is correct, but nothing durable
  tracks it.
- The new rejection changes a v0-core language boundary that 01 D4 owns, and 01 D4's text was not
  updated.
- Suggestion: record both as a comment on fx-djp or on fx-qlek.7's launch-claims audit, so the
  holder of `reference.md` picks them up.

**N4. README.md:18 is now about 150 characters wide.** The neighbouring lines wrap near 75
characters. Cosmetic only.

**N5. Record route 35 and the test comment overstate what the `tgkill` probe shows.**

- They attribute the `tgkill` denial to "Landlock's signal scope". The probe sees `EPERM`, which
  seccomp would also return.
- The test does not tell the two layers apart. No control shows `tgkill` reaching Landlock.
- Fix: say "denied (`EPERM`)", or add a control. This is text only.

## Checks that held

**FX0402 for bare parameters.** The check is sound and does not reject the accepted controls:

- A trait's own associated types are the only ones a blanket impl defines.
- A supertrait's associated type is checked at the supertrait's own blanket impl. A direct
  `impl Super for dyn Sub` gets E0371.
- `generics_require_sized_self` is the compiler's own test for which associated types a `dyn`
  type omits.
- The sized-subtrait split is consistent: `impl<T> Super for T` with
  `impl<T: ?Sized + Super> Sub for T` gives `dyn Sub` only the built-in `Super`.
- `tide-std` has no `?Sized` blanket impl with an associated type. Its `Pure` and
  `ToSocketAddrs` blankets have none, so the std side opens no route.
- The check runs on every user-written impl, including impls nested in bodies and impls a local
  macro generates. It is skipped only while rustc recovers from errors, and then the crate is
  rejected anyway.

**Each test fails without its fix.**

- `escape_dyn_overlap_transmute` has `//~ FX0402` on the impl, and rustc accepts the rest, so at
  fab99b3 the case would miss that mark.
- `escape_dyn_overlap_accept` would get FX0402 under the first, broader predicate.
- `std_path_admits_listed_items_only` asserts `lookup(..normalize_lexically).is_none()`, which
  the old `std::path::*` glob would fail.
- The reference row would get FX0520 for `is_empty` and `into_string` without their entries.

**Each rejected route fails for the stated reason.**

- The stable oracle compares the exact line and code sets, so a stray error cannot stand in for
  an expected FX code.
- E0433 and E0599 are the right rejections for the surfaces Tide's std leaves out.
- The linkage routes rely on the driver forcing `--edition=2024` (`invocation.rs:241,328`), which
  makes `unsafe(..)` mandatory on these attributes.

**RUN-3 test.**

- `call` now passes six arguments, so `process_vm_readv`'s flags argument is 0.
- Two direct controls make the vmread and loader denials meaningful.
- `.init_array` evidence: inside the sandbox `ctor_read` is 13 (`EACCES`) and `ctor_fork` is 1
  (`EPERM`), so the constructor was confined.
- The `interpreter` ELF parser assumes 64-bit little-endian, which is fine on the x86_64-only
  host.
- Inherited descriptors are closed by `close_range(3.., CLOEXEC)` at `sandbox.rs:134`. `TIOCSTI`
  is absent from the ioctl allowlist, and an allowlist cannot be bypassed with high bits set.

## Missed routes

| Route | Program | Likely result | Status |
| --- | --- | --- | --- |
| M1: projection self type, rust#57893 | The program in B1 | Likely admitted | Blocking |
| M2: `cfg_attr` hiding `#[path]` on an out-of-line module | `#[cfg_attr(all(), path = "/etc/hostname")] mod m;` | Probably rejected (FX0911, if the AST check runs after `cfg_attr` expansion) | Add a case |
| M3: implicit `#[global_allocator]` | `#[global_allocator] static A: Counting = Counting;` without the full path | Probably rejected (unresolved, or FX0520) | Add next to the full-path form |
| M4: `#[unsafe(no_mangle)]` | `#[unsafe(no_mangle)] pub extern "C" fn malloc(_: usize) -> *mut u8 { core::ptr::null_mut() }` | Probably FX0901 (`unsafe_ungranted` covers `no_mangle`) | Worth one row in the record |
| M5: `vfork` and `execveat` in the RUN-3 probe | `call(58, …)` and `call(322, AT_FDCWD, c"/bin/sh", …, 0)` | Probably denied (default `EPERM`, and Landlock checks execute on `execveat`) | Two lines in the existing test |

M2 also matters on its own: rustc reads the file at parse time. As with `include_str!`, the
crate is rejected and no artifact is produced.

Recording M2 to M5 in the existing case and test would make the "at least 15 routes" record
harder to fault at launch.

## Reasoning

Most of the change holds up:

- The admission cases, the item-by-item `std::path` list, and the RUN-3 test each test what they
  claim.
- Codex's r1 fixes are correct and have accept cases.
- The doc edits match the code as written.

The gap is B1. The record states that the trait-object route is closed. The rule recognizes
that route by the self type's syntactic shape, `ty::Param`, while the real condition is wider:
a self type that may be unsized and that can become a `dyn` type. On stable Rust that means a
parameter or a projection. Weak aliases, inherent associated types and TAITs are unstable, so
the driver already rejects them. Widening the match covers the class at no real cost to correct
programs. Settling B1 takes one case in the conformance run that is already scheduled.

I found nothing else that blocks. N1 to N5 are precision and text items, and none changes a tier
the change edits in a way that would restart the round.

DONE
