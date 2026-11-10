Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-yg5.1 round-1 Opus review (converge round, report only)

## Verdict

**request_changes**. I am highly confident in the facts below and moderately confident that B1 should block rather than go to a follow-up bead.

The hashing core is correct. `new()`, `default()`, `with_capacity()`, `collect`/`FromIterator`, `From<[_; N]>` and the `HashSet` equivalents all use the same fixed builder: a pristine `DefaultHasher`, which is SipHash-1-3 with key (0, 0), cloned for each hash. No per-process randomness enters through any of these routes. `from_rand` really does draw OS entropy. Admission and Pure behave as claimed. One gap blocks: production `HashMap` and `HashSet` expose only the old type-check stub's method subset, not Rust's API, which reference §1 and the bead's Scope promise.

## Scope

- **Read:** AGENTS.md (Governance and Merge Rule summary), the bead fx-yg5.1, its parent fx-yg5, fx-nkv's Acceptance, the implementer's report, and the full `git diff b4f1302 e581f25`.
- **Code read in full:** the new `crates/tide-std/src/collections.rs`, both new conformance cases, and the build.rs tests.
- **Code read in part:** `policy.rs` (whole rule list; RandomState and DefaultHasher rules), `walk.rs` (`decide`, `admit_def`, `admit_impl`, the Pure key path), and the doc header of `walk/mono.rs`.
- **Spec:** reference §1 (lines 34–45), §6.4, the §12.3 import table and §6.x Rand row; 03 GLOBAL-5 and RUST-6; 01-design (Rust consumers / Tide-owned `HashMap`); and the collection mentions in 02.
- **Rust source:** the pinned rust-src `library/std/src/hash/random.rs`, lines 55–73 and 106–107. It confirms that `DefaultHasher::new()` is `SipHasher13::new_with_keys(0, 0)`. It also confirms that `RandomState::new()` takes a thread-local key from `sys::random::hashmap_random_keys()` and adds 1 to k0 on each call.
- **Logs:** `workspace-test.log` (both new tests ok; 49 passed), `conformance.log` (`collections_executable`, `collections_pure`, `admit_collections` and `pure_std_types` PASS), and `e1-summary.json`. I also ran a regex scan of `target/e1-checked/tide-results.json` for `error[E…]`: no remaining error mentions a collection method or path.
- **Held-out check:** I grepped `trials/e1-checked` for the fixture's string literals (`"oak"`, `"elm"`, `"fir"`) and found no hits. The fixtures are generic and contain no E1 code.
- **Probes:** none run (no rustc or cargo).

## Blocking

### B1. Production HashMap/HashSet expose only a subset of Rust's API

**Where:** `crates/tide-std/src/collections.rs:54-223` and `:236-348`, against `design/reference.md:34-43`.

**Spec:** Reference §1 says "Standard types keep their Rust API: … `HashMap`, `HashSet` … What changes is how you obtain I/O objects (6) and the default hasher (6.4)." The bead's Scope says "otherwise keep Rust's API (entry API, iterators, …)".

**Implementation:** It copies the old stub's list. That stub said it covered only "the methods listed here" because it only decided type verdicts. Now that `root.rs` makes this module the production owner, the stub's subset has become the production API.

**Missing from `HashMap`:**
- `with_capacity_and_hasher`, `hasher`, `capacity`, `reserve`, `try_reserve`, `shrink_to_fit`, `shrink_to`
- `drain`, `extract_if`, `remove_entry`, `get_disjoint_mut`
- the matching `Drain` / `ExtractIf` re-exports

**Missing from `HashSet`:**
- `union`, `intersection`, `difference`, `symmetric_difference`
- `is_subset`, `is_superset`, `is_disjoint`
- `get`, `take`, `replace`, `drain`, `extract_if`
- `with_capacity_and_hasher`, `hasher`, `capacity`, `reserve`, `try_reserve`, `shrink_to*`
- the `BitOr` / `BitAnd` / `BitXor` / `Sub` impls on `&HashSet`
- the `Union` / `Intersection` / `Difference` / `SymmetricDifference` / `Drain` / `ExtractIf` re-exports

**Failure scenario:** A Tide program written from the reference fails with E0599 when it uses ordinary Rust idioms such as `let common: HashSet<_> = a.intersection(&b).cloned().collect();`, `if seen.is_subset(&all)`, `map.drain()`, `&a | &b`, or `HashMap::with_capacity_and_hasher(n, RandomState::from_rand(&rand))`. The last one is the natural sized form of the very constructor §6.4 teaches. In the E1 gate (fx-ic2), each such failure would count against Tide's learnability when the cause is an incomplete std, not the design.

The seven reference programs happen to avoid these methods. However, B7 and B9 still stop at resolution errors (E0425/E0433), so their typeck-stage method uses are not proven either.

**Smallest fix:** Forward each stable inherent method and operator impl of Rust's `HashMap`/`HashSet` (pinned toolchain) to `base`. Use the same one-line bodies the file already uses, with Rust's signatures and bounds, and re-export the iterator types they return. Then extend `collections_executable.rs` with one call per added item, so each is shown to execute.

I prefer explicit forwarding to `Deref`/`DerefMut` onto the std map. Forwarding keeps the public surface in one checkable list. It also avoids error messages and `*map` that name a `std::collections::hash::map::HashMap` type, which RUST-6 says is distinct.

**Fallback:** If the coordinator lands the bead first anyway, it must file this as a bead that blocks fx-yg5.5 and fx-ic2. It cannot be a comment, because §1 is v0-core text that this bead claims to implement.

## Non-blocking

### N1. The new admission rule is dead and over-broad (`crates/tide-admit/src/policy.rs:72-73`)

`allow("tide_std::collections::*")` changes no verdict:
- Tide-std items are already admitted when they are unlisted (`walk.rs:229, 252`) or exported (`walk.rs:243-247`).
- Impl keys begin with `impl ` and so never match this pattern.
- No later rule matches a `tide_std::collections::` key. I checked every pattern in `RULES`.

Because the rule is first in the list, it would also silently override any future deny on a collection item. Remove it, unless a focused rerun without it shows a failing verdict. In that case, replace the comment with the concrete reason.

The `std::hash::random::DefaultHasher` / `DefaultHasher::*` allows are needed (`RandomState::Hasher = DefaultHasher`, and `use std::hash::DefaultHasher`), and they are harmless: the key is fixed and nothing is ambient.

### N2. Under a future simulator, `from_rand` ignores `Rand` (`collections.rs:33-35`)

`from_rand` draws OS entropy whatever `rand` is. That is correct today, because `Rand` has no draws and `Root::sim` does not exist. Once a simulated `Rand` lands (reference §6.8: every capability has a deterministic simulated version), a `from_rand` map under `Root::sim(seed)` would iterate differently on each replay.

The constraint: `DefaultHasher` has no public keyed constructor. So the bead that adds `Rand` draws must either:
- seed by absorbing drawn words into a pristine `DefaultHasher` (`h.write_u64(k0); h.write_u64(k1)`, then clone `h`), or
- change `RandomState::Hasher`. The stub marked this `GUESS`, so changing it is allowed.

Record this as an Acceptance line on the Rand-draws/sim bead. It is not a fix here.

### N3. Iteration-order determinism is per toolchain and target, not portable

The determinism holds for one toolchain and one target. hashbrown's group width (SSE2 vs generic), `usize` hashing width and toolchain updates all change the order. The module doc says this correctly. GLOBAL-5 and §6.4 say only "deterministic (but unspecified)", which is consistent with that.

If the owner wants the scope stated in the design, add a clause to 03 GLOBAL-5, for fx-djp or a docs bead. It needs no code change.

### N4. Nit: stale ownership wording in the module doc (`collections.rs:2`)

"retain the fixture's Rust signatures" is stale. Production now owns these definitions and the fixture includes them. Say "keep Rust's signatures". Text only; no re-check needed.

### N5. Test strength is adequate for the hashing risk

The repeatability test compares stdout across two processes. That stdout includes the iteration orders of 64-key maps and sets built five ways each, so any per-process randomness would show with overwhelming probability. The `from_rand` check compares hash outputs from two processes, and an accidental match has probability about 2^-64. The negative test covers the generic `S = std::hash::RandomState` route for both maps and sets.

No change needed beyond B1's added calls.

## Decisions

1. **Single shared `pub mod collections;` in `root.rs`:** Agree. It gives one owner and no stub drift, which meets fx-nkv's "no two unreconciled std surfaces". Module resolution relative to the included file is proven by both builds (conformance PASS and the workspace tests).
2. **Pristine `DefaultHasher` clone as the builder:** Agree. It is the minimum: no dependency and no branch. I confirmed the key (0, 0) in pinned rust-src lines 106–107. N2 records its limit once `Rand` gains a backend.
3. **Entropy from Rust's `RandomState::new().build_hasher()`:** Agree. In pinned rust-src, `KEYS` is a thread-local seeded from `hashmap_random_keys()` (OS getrandom on Linux), and k0 is incremented on each call. That is Rust's own HashDoS posture. Fixed-key maps intentionally have no HashDoS resistance, and §6.4/GLOBAL-5 send untrusted keys to `from_rand`, as the code docs say.
4. **Gate on the existing `&Rand` token, with no draw API:** Agree. This is within scope, and `RandomState`'s Debug correctly hides the key.
5. **API boundary limited to the stub's listed methods:** Disagree. See B1. The stub's subset reflected a type-check fixture's needs, not the reference's contract. Do the full forwarding instead.
6. **Reuse the generic-argument Pure owner, with narrow admission:** Agree. Data `K`/`V`/`S` (including a seeded builder) is Pure, a capability in any argument is FX0517, and the nine negative cases cover every collection. The ambient Rust `RandomState` stays denied through `RandomState::new` (path rule) and `impl Default` (mono backstop), so neither `collect` nor `default` with `S = std::hash::RandomState` can reach the OS key. Drop the dead `tide_std::collections::*` allow (N1).

## Reasoning

- **Determinism:** Every fixed-key route ends in `RandomState::default()`, either directly or through `S::default()`. The value it builds is `DefaultHasher::new()`, a const SipHash-1-3 with key (0, 0), and `build_hasher` clones it. hashbrown adds no per-instance randomness, so for a fixed toolchain and target the order depends only on the mutation history and capacity.
- **API fidelity of what exists:** For the items present, the signatures and bounds match Rust exactly, including `Index<&Q>`, `Extend<(&K, &V)>` with `Copy`, `From<[_; N]>` fixed at the default `S`, Debug and Default with `S` unbounded, and `Clone` requiring `S: Clone`. Because `new`/`with_capacity` sit on `impl<K, V> HashMap<K, V>` and `S = RandomState` is a type default, inference behaves as in Rust.
- **Equivalence (fx-nkv):** It holds trivially, because one definition is shared.
- **What blocks:** Only the API surface. §1 makes "Rust's API, different default hasher" the contract that learners and the E1 participants rely on. The missing methods are common and the fix is mechanical, which makes the change worth fixing before landing.

DONE
