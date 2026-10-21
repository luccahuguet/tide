Reviewer: Opus (fresh subagent, converge round reporting) · Round: fx-gcs.6 r3 · Date: 2026-10-21

## Verdict

**Clean.** Nothing blocks. The three notes below are nits.

## Findings

- **N1 (non-blocking): the stub's header comment is now out of date.**
  - `conformance/rustc/std/src/lib.rs:9-10` says "Every body is `unimplemented!()`, so it decides type and borrow verdicts only."
  - But the two `const fn with_hasher` bodies (`:149` and `:266`) now build real values, and the static verdict depends on that.
  - Fix: add "except `with_hasher`, which builds the value so a `static` evaluates."
- **N2 (non-blocking, fails loudly): some of Rust's re-exports and operators are missing.**
  - Missing re-exports: `collections::Bound`, `hash_map::{DefaultHasher, Drain, ExtractIf}`, and `hash_set::{Difference, Intersection, Union, SymmetricDifference, Drain, ExtractIf}`. These give E0432.
  - Missing operators: `&HashSet` `|`, `&`, `^` and `-`. Probe `opus-probe3/p2_loud.rs` gives `2:E0432 6:E0369` where Rust accepts.
  - Both fail loudly, so neither blocks. `DefaultHasher` is FX0520 in the driver anyway.
- **N3 (nit):** `git diff --check` flags trailing spaces at `reviews/fx-gcs.6-r1-codex.md:13, 23, 41`. They are Markdown hard line breaks.

## Checked and fine

- **Candidate:** `git diff | sha256sum` = `78ce97e7…`. On it, `conformance/run.sh` exits 0 with 143 PASS and 59 AWAITS.
  - The three admit cases PASS.
  - `cap_timed_wait` is AWAITS CHECKER (FX0501).
- **Impl sets against Rust 1.96:** I checked against the 1.96.0 `rust-src`, `collections/hash/{map,set}.rs`, and every `for HashMap`/`for HashSet` impl in `library/`.
  - `HashMap`: every trait impl has Rust's bounds. That covers `Clone`, `PartialEq`, `Eq`, `Debug`, `Default` (`S: Default`), `Index<&Q>`, `From<[(K,V);N]>` (fixed to `RandomState`), `FromIterator` (`S: BuildHasher + Default`), and both `Extend` impls, including `(&'a K, &'a V)` with `K: Copy, V: Copy`. All three `IntoIterator` impls are there too.
  - `HashSet`: the same set, plus `Extend<&'a T>` with `T: 'a + Eq + Hash + Copy`.
  - The only trait impls left out are the four `HashSet` operators (N2).
  - `panic.rs`'s explicit `UnwindSafe` for `HashMap` carries through the wrapped field.
  - Rust's `const Default` is `rustc_const_unstable`, so stable code can't tell the difference.
- **Inherent methods:** these have Rust's receivers and bounds, in the right impl blocks: `new`/`with_capacity` (default hasher), the unbounded block (`len` through `retain`), and the bounded block (`entry`, `get*`, `insert`, `remove`, `contains`). `with_hasher` is const in Rust since 1.85, as here.
- **Probe `opus-probe3/p1_ok.rs`.** Plain rustc 1.96 and the stub's rustc run both accept it. It covers:
  - inference through `collect` into `HashMap<_, _>`;
  - `&mut` iteration and `Extend` over mapped tuples;
  - `Index` with `&i32` and with `str`;
  - `==` and `assert_eq!`;
  - `Send`, `Sync`, `Unpin`, `UnwindSafe` and `RefUnwindSafe` for both types;
  - covariance in `K` and `T`;
  - drop check, with a borrowed key that outlives nothing;
  - `extend(&[u32])` on a typed set, and `with_hasher(Default::default())`.

  Its only errors are the driver's FX0520 on `DefaultHasher`, which is driver policy that this diff does not touch.
- **`fs`:**
  - `Metadata` is `Clone` and `Debug`, not `Copy`.
  - `FileType` is `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash` and `Debug`.
  - `ReadDir` and `DirEntry` are `Debug`, and `ReadDir`'s `Iterator` item is right.
  - The method signatures match Rust's `fs.rs`.
- **`mpsc`:**
  - `RecvError` now has `Display` and `Error`.
  - `RecvTimeoutError` has the derives, `Display`, `Error` and `From<RecvError>`, as in Rust's `mpsc.rs`; `Cancelled` is the documented Tide addition.
  - `recv_deadline(&self, Instant)` matches Rust's unstable signature.
- **Leftovers:**
  - Outside `reviews/`, `archive/` and the historical `research/15`, no text still classes timed waits as FX0520.
  - The driver's `policy.rs` doesn't mention them.
  - The README and `driver.rs` comments are consistent with the cases.

No worktree files were edited. Probes and `run.log` are in `/tmp/claude-1000/-home-lucca-pjs-lab-tide/e82b34d5-9d7e-4e10-b5a8-e5e6171b6c40/scratchpad/opus-probe3/`.
