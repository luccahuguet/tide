Model: Opus 5.5. Date: 2026-10-31. Round 2, candidate 4b7e14d7 (uncommitted on 5472a93).

# fx-kw8.4 round 2 (Opus): mono-item backstop

**Verdict: not clean; two defects fixed. The candidate is now `05701262…4875` and needs another round.**

### Fixed

1. **The driver crashed on a static that points at an upstream non-generic function** (`walk/mono.rs`, `pointees` and the BFS over `live`). `pub static LOWER: fn(&str) -> String = str::to_lowercase;` against the stub std gave exit 101 with an ICE in `optimized_mir` for `alloc::str::<impl str>::to_lowercase`, reached through `items_of_instance`. `pointees` added every function and static an allocation names, and the BFS asked for the uses of items that rustc links rather than instantiates; rustc's `collect_alloc` filters these with `should_codegen_locally`. Fixed by fix 2: `uses` runs only on rustc's collected mono items.
2. **Eager collection made rustc reject legitimate code with post-monomorphization errors in code that never runs** (`invocation.rs`, `-Clink-dead-code`). A trait whose provided `fn first_index(&self)` holds `const { assert!(Self::LEN > 0) }`, with `impl Nonempty for Nothing { const LEN: usize = 0; }` and no caller, builds with stable rustc but failed with `E0080 … while instantiating <Nothing as Nonempty>::first_index`. Round 1's root filter cannot prevent this, since rustc's collector raises the error; eager drop glue for unused ADTs has the same problem.
   - Dropped `-Clink-dead-code`; rustc collects lazily.
   - `Admission::config` overrides the `reachable_non_generics` query with rustc's `reachable_non_generics_helper` (exposed for Miri), so a metadata check has a full build's roots: what the crate exports, its statics, and its entry point. Metadata output is unchanged: `exported_non_generic_symbols` keeps its own early return for metadata builds.
   - `mono.rs` lost its root filter and BFS: every collected item is live, and users come from each item's uses. `main.rs` gains `extern crate rustc_codegen_ssa`.
   - Docs: the `mono.rs` header (a library's `#[inline]` functions, like its generics, are checked in the crates that instantiate them), the invocation comments, ADMIT-1's roots sentence, and ADMIT-2, which no longer names `-Clink-dead-code`.
   - Tests: the denied-impl driver test runs the source as a library (the same 11 sites), then as a `--test` build, which reports only `lib.rs:122:1`, since a test build runs only what its tests reach. `admit_backstop_accept.rs` gains the `Nonempty`/`Nothing` regression.

### Not fixed

- **Extra reports in backstop-only mode:** a failure inside a generic instance whose own precondition already fails is reported again inside it (`self.cmp(o)` inside `PartialOrd::partial_cmp::<W<Clock>>`, next to the real site). It appears only when the walk misses a route, and the program is rejected anyway; suppressing reports in failing users would let a root with a trivially false `where` clause through.
- **Coverage now matches "can run":** dead private functions are no longer checked by the backstop (the walk still checks them); `pure_routes.rs` as a bin gets only `154:1` from the backstop alone. Intended.
- **Vtable route for `where Self: Pure` provided methods:** not an issue; rustc rejects such traits as not dyn-compatible (E0038).
- **Opaque types revealed in the backstop's environment:** the walk checks hidden types where they are defined, so revealing them cannot turn a walk acceptance into a backstop rejection.
- No regression of round 1's fixes; `--tide-test-backstop` stays operator-trusted, as decided. Nothing for fx-djp.

### Verification

- At 4b7e14d7: `cargo test --workspace` and `conformance/run.sh` exit 0.
- After the fixes: `cargo test --workspace` exit 0 (2 + 7 + 13); `conformance/run.sh` exit 0, 181 PASS, 28 AWAITS CHECKER; rustfmt and `git diff HEAD --check` clean. A last comment-only edit in `mono.rs` was followed by a rebuild and `cargo test --workspace`, passing.
- Probes against the stub std, normal and `--tide-test-backstop`: the upstream-function static (was ICE 101, now exit 0), the post-mono provided method (was E0080, now exit 0), the accept case (exit 0), and the sort-through-`Ord` and opaque-closure routes (still FX0517 at local spans).
- `trials/e1-dialect/programs` all pass except `backup.rs`, which fails with an unrelated rustc E0600.
