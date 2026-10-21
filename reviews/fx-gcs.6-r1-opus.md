Reviewer: Opus (fresh subagent, converge round reporting) · Round: fx-gcs.6 r1 · Date: 2026-10-21

## Verdict

**Findings.** One blocking, five non-blocking.

## Findings

**F1 (blocking): `Default` and `FromIterator` on the stub's `HashMap`/`HashSet` do not follow Rust's impl signatures.**
- **Where:** `conformance/rustc/std/src/lib.rs`, the `impl<K, V> Default for HashMap<K, V>` and `impl<K: Eq + Hash, V> FromIterator<(K, V)> for HashMap<K, V>` blocks, and the same two impls in `hash_set`.
- **Problem:** Rust's impls are `impl<K, V, S: Default> Default` and `impl<K: Eq + Hash, V, S: BuildHasher + Default> FromIterator`. The stub fixes `S = RandomState`. That contradicts the module's own doc comment ("impls keep Rust's signatures") and reference §Standard types, which says `HashMap` and `HashSet` keep their Rust API. The result is wrong verdicts in both directions:
  - It rejects programs the design admits. My probe `opus-probe/p2_div.rs` uses a deterministic `BuildHasherDefault<DefaultHasher>` map through `HashMap::default()`, `.collect()` and `#[derive(Default)]`. Plain rustc 1.96 accepts it; the stub gives `7:E0308 10:E0277 13:E0277 17:E0277`.
  - It also changes inference. In `p3_infer.rs`, Rust gives E0283 on both `let m = HashMap::default()` and `let m = HashMap::from_iter(v)`. The stub gives E0282 on the first and accepts the second.
- **Smallest fix:**
  - Make both impls generic over `S` as Rust does, for both types.
  - Add `impl Default for RandomState`, a fixed key like `new()`, marked `GUESS`.
  - I checked this shape in a standalone file (`opus-probe/fixcheck.rs`). Custom hashers are accepted, and both inference lines give E0283, as in Rust. The driver still blocks Rust's random key through this route: picking `S = std::hash::RandomState` selects `impl Default for std::hash::random::RandomState`, which `policy.rs` denies.

**F2 (non-blocking): common Rust-API items are missing from the stub.**
- **Problem:** These fail loudly (E0599/E0277), so they won't give a silently wrong verdict. But the first case to use them will fail.
  - `HashMap::from([(k, v)])` gives a confusing E0308. Under Tide's fixed key it should be admitted. `policy.rs` denies it only for Rust's maps.
  - `HashSet` lacks `union`, `difference` and `is_subset`. `HashMap` lacks `get_key_value`, and `hash_map` lacks `DefaultHasher`.
  - `DirEntry` and `ReadDir` have no `Debug`.
  - `RecvTimeoutError` lacks the `Display`/`Error` impls and the `From<RecvError>` conversion that 03 §6.4 gives it.
  - `RecvTimeoutError`'s doc says "`RecvError`'s variants plus `Timeout`", but the stub's `RecvError` is still a `GUESS` unit struct. 03 §6.4 fixes it as `enum { Disconnected, Cancelled }`, so that `GUESS` is stale. This predates the change.
- **Fix:** Add at least `From<[(K, V); N]>` / `From<[T; N]>` and `DirEntry: Debug`, or leave them to whichever case first needs them.

**F3 (non-blocking): `admit_hardware.rs` runs only on x86_64.**
- **Where:** `conformance/cases/authority/admit_hardware.rs:3-5`.
- **Problem:** The case says it is x86_64-only, but nothing enforces that. The runner rejects unknown `//@` lines, and `#[cfg]` would leave the `FX0520` mark unmatched. On an aarch64 host the case gets E0433 and `run.sh` exits 1.
- **Fix:** Add "an x86_64 host" to the requirements sentence in `conformance/README.md` (Running).

**F4 (non-blocking): the companion's porting table has no row for timed waits.**
- **Where:** `design/reference-companion.md` §14.2 (lines 70-83).
- **Problem:** Reference 6.9 now lists `rx.recv_timeout(d)` as FX0501 and sends readers to companion 14.2 for the fix. That table has rows for `Instant::now` and `thread::sleep`, but none for `recv_timeout`.
- **Fix:** Add the row `rx.recv_timeout(d)` → `rx.recv_deadline(clock.now() + d)`. This costs no core tokens.

**F5 (non-blocking): the stub has two different `RandomState` types.**
- **Problem:** `std::hash` is still Rust's re-export, so `std::hash::RandomState` is Rust's type. `std::collections::hash_map::RandomState` is now Tide's. In Rust they are the same type. In my probe `opus-probe/p5_hashmod.rs`:
  - `HashMap<K, V, std::collections::hash_map::RandomState>` → `HashMap<K, V, std::hash::RandomState>` gives E0308.
  - `std::hash::RandomState::from_rand` gives E0599.
- **Impact:** Nothing leaks. `policy.rs` denies Rust's `new` and `Default`, and the reference puts `RandomState` only in `std::collections::hash_map`.
- **Fix:** A one-line note in the `collections` doc comment, or give `hash` a module that re-exports Tide's `RandomState`.

**F6 (non-blocking, out of scope, predates the change): `Instant::elapsed` and `SystemTime::elapsed` are not covered.**
- Both read the clock with no `Clock`. Reference §Standard types says `Instant` and `SystemTime` keep their Rust API.
- The new CAP-1 wording ("no … method that does what a capability method does") now covers them in principle. But neither CAP-1's examples nor reference 6.9 names them, and the stub's `Instant` doesn't list them.
- This could go on fx-djp or a follow-up bead. Note the reference has only 8 tokens of headroom.

## Checked and fine

- **Candidate:** `git diff | sha256sum` matches `d24ba5d7…`.
- **New cases:** `run.sh` on the four new cases plus `cap_ambient`:
  - PASS for `admit_collections`, `admit_dir_entry` and `admit_hardware`.
  - AWAITS CHECKER (FX0501) for `cap_timed_wait` and `cap_ambient`.
- **Borrow and type behaviour match Rust.** My probe `opus-probe/p1_borrow.rs` PASSes both passes with the same marks plain rustc 1.96 gives. It covers:
  - all of 02 §10.2, including NLL problem case #3 (E0499 at the same two lines) and the `or_default` idiom;
  - matching on `Entry`, `values_mut`, `&mut` iteration, `Index` with `&str` and `&String`, and a held `get` across `insert` (E0502);
  - drop check (`m.insert(&s[..])` with `s` declared after `m`), covariance of both types, `Send + Sync`, `retain`, and `into_iter().collect()`.
- **Signatures:** inherent methods, `Index`, the three `IntoIterator` impls, `Clone`, `Debug`, `PartialEq`/`Eq` and `Extend` match Rust's bounds and impl split. The two-phase borrow in `admit_collections:14` works as claimed.
- **Fs types:** `ReadDir`, `DirEntry`, `Metadata` and `FileType` have Rust's derives (apart from F2's missing `Debug`) and auto traits. `Fs::metadata` and `read_dir` now return the local types.
- **Leftovers:** no other `design/` text, `conformance/rules.md` row, or companion line still classes timed waits as FX0520. Both D18 lists, ADMIT-1 and reference 6.9 are consistent. `research/15` keeps the old claim, but it is a historical evidence record. rules.md's CAP-1 row (E0425/E0599 → FX0501) still fits.
- **CAP-1 reclassification is sound.** Tide's `Receiver` is its own type (03 §1.2, §6.4), so `recv_timeout` cannot be named and there is nothing for the allowlist to deny. The case parallels `Instant::now` → `E0599 FX0501`. `recv_deadline` takes a `Clock`'s `Instant`, matching reference 7.2 and 7.3.
- **Token budget:** reference is at 24,992 tokens, within the 25k budget.
- **Acceptance:**
  - `HashMap::new`, `DirEntry::path` and `Metadata::is_file` cases pass both passes.
  - The timed-wait and hardware cases cite 03 CAP-1/§6.4 and ADMIT-1.
- **Driver:** `policy.rs`'s rules for Rust's `HashMap`/`From` are still exercised by `tests/driver.rs` against Rust's std, so they are not orphaned.
