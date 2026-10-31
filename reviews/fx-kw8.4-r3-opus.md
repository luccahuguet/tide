Model: Opus 5.5. Date: 2026-10-31. Round 3, candidate c9b31142 (uncommitted on 5472a93).

# fx-kw8.4 round 3 (Opus): mono-item backstop

**Verdict: clean round.** No defect found; no file changed.

### Not fixed

**The `reachable_non_generics` override (`invocation.rs`, `Admission::config`) is sound**, checked against the pinned source (`rustc_codegen_ssa/src/back/symbol_export.rs`, `rustc_monomorphize/src/{collector,partitioning}.rs`, `rustc_metadata/src/rmeta/{encoder,decoder/cstore_impl}.rs`, `rustc_passes/src/reachable.rs`):
- Metadata and `exported_symbols` are unchanged. The encoder (`encoder.rs:701-705`) writes only `exported_non_generic_symbols` and `exported_generic_symbols`, whose local providers still return `&[]` when `!should_codegen()`, as for `--emit=metadata`; a downstream crate's `reachable_non_generics` comes from those lists.
- In a metadata build the override has two consumers: the collector's lazy `is_root` (`collector.rs:1692`), the intended effect, and partitioning's linkage and visibility (`partitioning.rs:784/903/970`), which matter only for codegen. `should_codegen_locally` returns `true` for local items before asking `is_reachable_non_generic`.
- The roots are the ones claimed: statics, the entry fn, and what `is_reachable_non_generic` gives at the export threshold (Rust level for a lib: every reachable non-generic, non-`#[inline]` item; C level for a bin or `--test` build: only `#[no_mangle]`/`export_name` items). At opt-level 0 without `-Zinline-mir`, `cross_crate_inlinable` holds only for `#[inline]` items, closures, and constructors. `reachable_set` marks every trait impl item reachable, even for private types: a slight over-approximation that matches what a real build codegens.

Probes against the stub std with `--tide-test-backstop`:

| Case | lib | bin | `--test` |
| --- | --- | --- | --- |
| exported type's `Display` impl | reported | reported (when `main` calls it) | not reached |
| private never-constructed type's impl | reported (rustc's conservative reachable set; the walk rejects it too) | — | — |
| private fn called from a `pub` generic fn | reported | — | — |
| `pub` fn in a private module, never called | not reported (cannot run) | — | — |
| `#[inline] pub fn` | not reported (as documented) | — | — |
| closure returned as `impl Fn` from a `pub` fn | not reported | — | — |
| `pub const F: fn() = private_fn` | reported | — | — |
| private static holding a closure | reported | reported | reported |

A returned closure is like an `#[inline]` function: rustc does not export it, so a downstream crate instantiates it. A static of a `Drop` type has its drop glue collected and reported in the local `drop` (the walk rejects it too). A `deny`-lint error raises fatal in analysis, so the backstop does not run. Common std `Debug` formatting (`Vec<Box<_>>`, `Rc`, `Weak`, `RefCell`, `BTreeMap`, `VecDeque`, iterators, `Box<dyn Error>`, `Duration`, `TypeId`) passes in backstop-only mode, so the granted `impl Pointer for *` and `impl * for fn()` rules give no false positive there.

Nothing that can run is missed, apart from the known exceptions: generic-only code, which the walk owns; `#[inline(never)]` upstream instances an upstream crate already made (ADMIT-2); and non-generic, non-inline upstream denied-impl methods, which the crate links. No current deny rule names one; all are generic or `#[inline]`.

Nits, not blocking:
1. The `mono.rs` header's "a library's `#[inline]` functions, like its generics" also covers closures a library returns uncalled.
2. The `invocation.rs` comment "a metadata check exports nothing, which leaves a library's mono items without roots" overstates a little: statics stay roots.
3. README.md line 51 runs past the paragraph's wrap width; ADMIT-1 has an odd line break.
4. README, D18, and `conformance/rules.md` list `-Zallow-features=` and `-Zpolonius=no` but not `-Zshare-generics=no`; ADMIT-2, which owns the flag list, has it.
5. `check_mono_items` formats `impl_key` and looks up the rule per non-local trait-impl instance, uncached; the cost is acceptable (conformance 2m25s).

The bead's decisions comment is stale: it describes eager collection (`-Clink-dead-code`), roots filtered to local non-generic fns, statics, and `start`, two ADMIT-2 flags, and costs measured with eager collection. Its report-site description (`local_uses`) is still accurate. The docs match the code. Nothing for fx-djp.

### Verification

- `cargo test --workspace` at c9b31142: exit 0 (2 + 7 + 13).
- `conformance/run.sh`: exit 0; 181 PASS, 28 AWAITS CHECKER, 0 FAIL; 2m25s.
- `git diff HEAD --check`: clean. No `rustc-ice-*` or stray files in the repository.
- Final `git diff HEAD | sha256sum` = `41f864af…7c5a`; it differs from c9b31142 only by `.beads/issues.jsonl`, which another session changed during the round. Without `.beads/`: `68199b25…1e15`, unchanged by this round.
