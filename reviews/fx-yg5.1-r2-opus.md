Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-yg5.1 round-2 Opus re-check (narrow; B1 and N1–N4 only)

## Verdict

**approve**. All five findings are resolved. I found no new defect in the changed lines.

## What I checked

- **Diff:** `git diff e581f25 fx-yg5.1` (HEAD `f434d13`), 6 files.
- **Inputs:** the author's report `fix1-report.md` and the coordinator's decisions `fix1-brief.md`.
- **Proof identity:** `git diff e581f25 f434d13 | sha256sum` gives `677c02a5…c8ed`. That equals the author's frozen pre-suite diff hash, so the final suites ran on exactly the committed tree.
- **Proof logs** (`target/fix1-proof/`):
  - `workspace.log`: all three `collections_*` tests ok.
  - `conformance.log`: 198 PASS, 0 FAIL. `admit_collections`, `collections_executable`, `collections_pure` and `pure_std_types` all PASS.
- **Independent inventory:** I compared the pinned rust-src `library/std/src/collections/hash/{map,set}.rs` (nightly-2026-08-09) with `crates/tide-std/src/collections.rs`. I did this with my own script, not the author's.
- **Probes:** I ran no driver or rustc probes. The executable conformance case and the build tests already exercise every added item, and the logs above match the committed tree.

## Per-finding status

### B1: Production HashMap/HashSet expose only a subset of Rust's API. **Resolved.**

**Inherent methods.** My script lists the stable, non-allocator-API inherent methods in the owner impls. Both maps match Tide's exactly: 33 map methods and 30 set methods. The only differences were artefacts of my own regex: it missed `pub const fn with_hasher`, which is present at `collections.rs:71` and `:322`, and it counted `from_rand`, which is Tide's intended extra.

**Bound placement matches Rust:**
- These are in the unbounded `impl<K, V, S>` / `impl<T, S>` blocks:
  - `with_capacity_and_hasher`, `capacity` and `hasher` (`collections.rs:76-86`, `:327-337`)
  - `drain` and `extract_if` (`:117-125`, `:350-358`)
- These are under `K/T: Eq + Hash, S: BuildHasher`, as in rust-src lines 890/562:
  - `reserve`, `try_reserve`, `shrink_to*` (`:131-142`, `:364-375`)
  - the set relations and operations (`:376-399`)
  - `get`, `replace`, `take` (`:409-431`)
  - `get_disjoint_mut` and `get_disjoint_unchecked_mut`, whose bounds are `K: Borrow<Q>, Q: ?Sized + Hash + Eq` (`:165-188`)
  - `remove_entry` (`:204-210`)

**Trait and operator impls.** Every `impl … for HashMap`/`HashSet` in rust-src has a Tide counterpart. That includes:
- `Clone`, now with `clone_from` (`:218`, `:439`)
- `BitOr`/`BitAnd`/`BitXor`/`Sub` on `&HashSet` with Rust's bounds `T: Eq + Hash + Clone, S: BuildHasher + Default` (`:496-535`)

**Re-exports.** `hash_map::{Drain, ExtractIf}` (`:18-19`) and `hash_set::{Difference, Drain, ExtractIf, Intersection, SymmetricDifference, Union}` (`:298-300`) are now re-exported.

**Tests.** `collections_executable.rs:75-110` (maps), `:142-206` (sets) and `:265-270` (seeded sized constructors) call every added item and every operator. They also annotate each returned iterator type, so a re-export regression fails to compile. `crates/tide-admit/tests/build.rs:111-156` covers both `get_disjoint` methods:
- the `unsafe` variant is gated by FX0901 and runs once granted;
- the overlapping safe lookup panics, as Rust's does.

**Determinism.** `&a | &b` and the other operators build `S::default()`. For Tide's builder that is the fixed key. For `std::hash::RandomState` it is still blocked by the existing `impl Default` mono backstop, which the existing negative test covers.

### N1: The admission rule is dead and over-broad. **Resolved.**

`policy.rs:72-73` (`allow("tide_std::collections::*")` and its comment) is removed. No replacement is needed. The final conformance run has `admit_collections`, `collections_executable` and `collections_pure` passing without the rule. That confirms my round-1 analysis that existing ownership/export admission covers these items. The `DefaultHasher` allows and the ambient `RandomState` denials are unchanged.

### N2: Under a future simulator, `from_rand` ignores `Rand`. **Resolved, as the coordinator scoped it** (one doc sentence, no behaviour change).

`collections.rs:25` now says `from_rand` must use the simulated `Rand` backend for replay once one exists.

Nit (non-blocking, needs no re-check): the implementation constraint is not recorded anywhere. `DefaultHasher` has no public keyed constructor, so the future backend must either absorb drawn words into a pristine hasher or change `RandomState::Hasher`. The bead that adds `Rand` draws and the simulator is the durable place for that note. The coordinator may add it there when that bead is shaped.

### N3: Iteration-order determinism is per toolchain and target. **Resolved.**

`design/03-authority-concurrency.md:413-414` now limits repeatability to the same mutation history, toolchain and target, and says the order is not portable. This is consistent with the module doc at `collections.rs:4-6` and with reference §6.4's "deterministic (but unspecified)".

### N4: Stale ownership wording. **Resolved.**

`collections.rs:2-3` now reads "keep the pinned toolchain's full stable Rust API". `lib.rs:15-17` states the same surface and the fixed default keys. Both claims match my inventory above.

## New findings in the changed lines

None. I checked the following, and none is a defect:
- `hasher().clone()` can duplicate a seeded builder. This matches Rust, where `RandomState: Clone`, and it is still data reached only through `&Rand`.
- `try_reserve(usize::MAX).is_err()` relies on Rust's documented capacity-overflow error.
- The `unsafe` forwarding body keeps Rust's safety contract in its doc.

DONE
