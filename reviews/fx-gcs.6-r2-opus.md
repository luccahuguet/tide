Reviewer: Opus (fresh subagent, converge round reporting) · Round: fx-gcs.6 r2 · Date: 2026-10-21

## Verdict

**Findings.** Two are blocking. Both are stub-vs-Rust verdict mismatches in `conformance/rustc/std/src/lib.rs`, the file this change edits. The first comes from this round's `const` fix. The second has been there since round 1, and round 1 missed it.

## Findings

**F1 (blocking): `with_hasher` is now `const fn`, but evaluating it panics.**
- **Where:** `lib.rs:148` and `lib.rs:262`.
- **Problem:** The body is `mem::forget(hasher); unimplemented!()`. Any `static` or `const` that uses it fails const evaluation.
- **Evidence:** probe `opus-probe2/p1_const.rs` defines `static M: HashMap<u32, u32, BuildHasherDefault<DefaultHasher>> = HashMap::with_hasher(BuildHasherDefault::new());` and the same for `HashSet`.
  - Plain rustc 1.96 accepts it.
  - The stub gives `E0080: evaluation panicked: not implemented` on both lines. Before this round it gave E0015, so the round-1 fix swapped one wrong verdict for another.
  - `--emit metadata` still evaluates statics.
- **Smallest fix:** build the real value, which is const in Rust since 1.85:
  - `HashMap { base: std::collections::HashMap::with_hasher(hasher) }`
  - `HashSet { base: std::collections::HashSet::with_hasher(hasher) }`
  - I checked this shape in `opus-probe2/fixcheck.rs`: the static is accepted.

**F2 (blocking): the stub lacks Rust's by-reference `Extend` impls.**
- **Where:** `lib.rs:218` (`HashMap`) and `lib.rs:307` (`HashSet`).
- **Problem:** Rust has these as well as the by-value impls:
  - `impl<'a, K: Eq + Hash + Copy, V: Copy, S: BuildHasher> Extend<(&'a K, &'a V)> for HashMap<K, V, S>`
  - `impl<'a, T: 'a + Eq + Hash + Copy, S: BuildHasher> Extend<&'a T> for HashSet<T, S>`
- With only one impl, the verdicts are wrong in both directions. Evidence is probe `opus-probe2/p2_extend.rs`:
  - `let mut s = HashSet::new(); s.extend(v)` (with `v: &[u32]`) and the `HashMap` equivalent: Rust 1.96 gives `E0282` at lines 5 and 10, but the stub accepts both. That is a silent false accept.
  - `let mut s: HashSet<u32> = …; s.extend(v.iter())`: Rust accepts, but the stub gives `16:E0271`.
- This is the same class as round 1's F1: an impl set that differs from Rust's changes inference.
- **Smallest fix:** add the two impls with Rust's bounds. In `fixcheck.rs` this shape restores the ambiguity error (E0282/E0283) and accepts the annotated call.

**F3 (non-blocking, nit, predates the change):** the stub's `RecvError` still has no `Display` or `Error` impl, so `rx.recv()?` into a boxed error fails, where Rust's equivalent works. The stale `GUESS` round 1 noted on it is also still there. This could go on fx-djp.

## Checked and fine

- **Candidate:** `git diff | sha256sum` matches `4e79b520…`.
- **New cases:** `run.sh` gives PASS for `admit_collections`, `admit_dir_entry` and `admit_hardware`, and AWAITS CHECKER (FX0501) for `cap_timed_wait`.
- **Round-1 F1 is resolved.**
  - `Default` (`S: Default`) and `FromIterator` (`S: BuildHasher + Default`) on both types now match Rust 1.96.
  - Tide's `RandomState: Default` is marked GUESS.
  - `admit_collections` covers a caller's `BuildHasherDefault` through `collect` and `default`.
- **The `From` impls match Rust's:** `From<[(K, V); N]> for HashMap<K, V>` with `K: Eq + Hash`, and `From<[T; N]> for HashSet<T>` with `T: Eq + Hash`, both fixed to the default `RandomState` as Rust's are. The case's `HashMap::from` and `HashSet::from` calls pass both runs.
- **`get_key_value`** has Rust's signature (`Q: ?Sized + Hash + Eq`, `K: Borrow<Q>`).
- **`Debug`** on `ReadDir` and `DirEntry` matches Rust's derives.
- **`RecvTimeoutError`** now has `Display`, `Error` and `From<RecvError>`, as 03 §6.4 requires.
  - `cap_timed_wait` propagates the error with `?` into `Box<dyn Error + Send + Sync>`.
  - My probe `p3_err.rs` passes: `rx.recv()?` into `Result<_, RecvTimeoutError>`, `e.to_string()`, and matching on `Timeout`.
- **Round-1 F5:** the `collections` doc notes that `std::hash::RandomState` is still Rust's.
- **Round-1 F3:** the README states the x86_64 host requirement and names the case.
- **Driver test comment:** `driver.rs:74` is corrected.
- **Round-1 F4:** the companion's 14.2 table has the row `rx.recv_timeout(d)` → `rx.recv_deadline(clock.now() + d)`, which matches reference 6.9 and 7.3.
- **Left out on purpose:** F2's `HashSet` set operations fail loudly (E0599), and F6 is out of scope for the epic. Both are acceptable.

Probes are in `/tmp/claude-1000/-home-lucca-pjs-lab-tide/e82b34d5-9d7e-4e10-b5a8-e5e6171b6c40/scratchpad/opus-probe2/`. No worktree files were edited.
