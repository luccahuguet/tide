# Enforcing the reference's FX rules in tide-admit (fx-u65.2 to fx-u65.6)
Drafted by Fable on 2026-11-11 for fx-u65.2 to fx-u65.6; decisions are recorded on fx-u65 and fx-u65.2.

Research report, 2026-11-11. Read-only: nothing in the repository or `br` was changed, no cargo build ran, and nothing under `trials/e1-checked/` was opened.

Evidence labels: **[repo]** read in this checkout; **[pin]** read in the pinned compiler's own sources (`~/.rustup/toolchains/nightly-2026-08-09*/lib/rustlib/rustc-src/rust/compiler`); **[web]** cited URL; **[unverified]** an inference that no build has tested.

## 0. What already exists

- **Codes and severity.** `diag.rs` owns `Code`, `Code::ALL` (printed by `--tide-print-codes`) and `is_lint()`. Errors go through `struct_span_err`, lints through `struct_span_warn`, each with the message prefix `FXnnnn:` [repo].
- **The lint mark is done.** fx-u65.1 (closed) added `//~ WARN FXnnnn` to `conformance/run.sh`, and the runner compares `line:severity:code`. Slices .2 to .6 contain no lint-level code (`FX0902` is "provisional: error"), so none of them needs runner work except the package-level proof for `FX0915` (§1.6).
- **Two check points.** `check_expanded` walks the expanded AST. `check_crate` walks HIR with typeck results. Both run in `after_expansion`, before rustc's analysis, so a crate with rustc errors still gets FX verdicts. `check_mono_items` runs in `after_analysis` only when nothing failed [repo]. On the pin, `after_expansion` is called right after `resolver_for_lowering()` and before `early_lint_checks` steals the lint buffer [pin: `rustc_driver_impl/src/lib.rs:294-297`, `rustc_ast_lowering/src/lib.rs:500`].
- **Existing hooks to reuse.** `check_expansions` (every expansion, including unresolved bang macros, as `lazy_static!` shows), `unresolved()` (a path Tide's std lacks, matched against Rust's std by key), `std_attr_made` (whether a std attribute macro made an item), `written_by_other_crate` and `generated_by_toolchain` (provenance) [repo].

This matches how clippy and dylint work: a driver wraps rustc and runs checks over HIR with type information, and "late" passes are the usual choice when types are needed ([clippy lint passes](https://doc.rust-lang.org/clippy/development/lint_passes.html), [how dylint works](https://trailofbits.github.io/dylint/docs/how_dylint_works.html), [`Callbacks`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver_impl/trait.Callbacks.html)). The runner's marks follow rustc's UI tests, which annotate the expected diagnostic on its source line with `//~ ERROR` or `//~ WARN` ([rustc-dev-guide, UI tests](https://rustc-dev-guide.rust-lang.org/tests/ui.html)).

## 1. Check-by-check plan

Provenance for every check unless stated: report only code the candidate wrote, judged from the span's expansion data as clippy does (`from_expansion`, `in_external_macro`, `ctxt`; [clippy: macro expansions](https://doc.rust-lang.org/clippy/development/macro_expansions.html)). Tide has no user proc macros, so clippy's proc-macro span problem applies only to Tide's own `tide-macros`.

Case shape unless stated: a `<rule>_reject.rs` under `conformance/cases/<area>/` with one `//~ FXnnnn` per violating line (plus rustc's code where rustc also rejects), and an accepted twin showing the reference's "write instead".

### 1.1 fx-u65.2: surface and casts

| Code | Level and mechanism | Notes |
| --- | --- | --- |
| FX0102 | `check_expansions`: a `MacroKind::Derive` expansion whose macro is unresolved, or is neither one of core's nine kept derives nor exported by Tide's std. Today a resolved one falls to `use_def` and gets FX0520; this recodes it. | Unresolved derives must produce an expansion record, as unresolved bang macros do [unverified]. Fallback: scan `#[derive(..)]` names in `Guard`'s pre-expansion AST. Case line carries rustc's uncoded `error` too. |
| FX0103 | `check_expansions`: a `MacroKind::Bang` expansion whose call site is user-written and whose macro is not in §2.1's fixed list. Keep the reference's carve-outs: `include_str!`/`env!`/`option_env!` FX0520, `thread_local!`/`lazy_static!` FX0502, `asm!` family FX0520/grant, `select!` FX0605 (.5). | Decide on the macro the user wrote, never on nested expansions inside an admitted std macro (`println!` expands to `format_args!`). `debug_assert_eq!`, `concat!`, `stringify!`, `cfg!`, `line!`, `file!` are rejected when user-written; `cfg!` written by a contract macro is not. |
| FX0201 | HIR item walk: `ItemKind::Trait` with a lifetime generic parameter. | Existing case `memory/life_ceiling_reject.rs`. |
| FX0202 | HIR: in a fn signature's return type, a `hir::Lifetime` whose kind is `LifetimeKind::Error` [pin: `rustc_hir/src/hir.rs`]. rustc reports E0106 during resolution, before the walk, so the error kind is already there. | Existing case `memory/life_elision_reject.rs` (`//~ E0106 FX0202`). See choice C3. |
| FX0203 | Label on rustc's borrow-check rejection (choice C1). The walk records candidate sites: a closure coerced to a `dyn` object in a signature or field position, or passed to `spawn_isolated`. | Existing cases mark `//~ error FX0203`. |
| FX0210 | Label on rustc's rejection inside a struct literal of a local type with a lifetime parameter (choice C4). | No rule row in `rules.md` and no existing case. |
| FX0301 | HIR `ExprKind::Cast` with typeck: source type from `expr_ty(src)`, target from `node_type(e.hir_id)`; a table of the pairs §4.2 allows; everything else numeric is FX0301. | See below. |

**FX0301 detail.** The allowed set is closed and small, so write it as a function over `(from, to)` type kinds, the way clippy's cast lints decide from types and bit widths ([clippy casts](https://rust.googlesource.com/rust-clippy/+/refs/heads/beta/clippy_lints/src/casts/mod.rs), [cast_possible_wrap](https://rust.googlesource.com/rust-clippy/+/refs/heads/beta/clippy_lints/src/casts/cast_possible_wrap.rs)). Unlike clippy, which evaluates `usize` at the target's width and so misses `usize as u64` hazards on other targets ([clippy issue via goodfirstissue](https://www.goodfirstissue.org/rust-lang/rust-clippy/issues/9231)), Tide's rule is target-independent: `usize`/`isize` count as "32 to 64 bits", so widening to them needs at most 32 source bits and widening from them needs at least 64 target bits.

- Integer to float "when exact": the integer's value bits must fit the mantissa (24 for `f32`, 53 for `f64`), so `u16`/`i16` and narrower to `f32`, `u32`/`i32` and narrower to `f64`.
- Fieldless enum to integer "holding every discriminant": read the discriminant range from `tcx.adt_def(..).discriminants(tcx)`.
- Under the `unsafe` grant, a raw or function pointer on either side counts as `usize` (existing case `authority/address_casts_lossy.rs`). Without the grant, FX0520 already fires on the pointer; report both, since they are different rules.
- Interplay recorded on fx-yg5.3: the replacements' bodies (`self as Target`, `value as Self`) live in Tide's std, which the walk never checks, so no exemption is needed. The accepted twin should use each replacement spelling once fx-yg5.3 has landed; until then use `try_from`, `cast_unsigned`, and widening casts only.
- Non-numeric casts (`&T as &dyn Trait`, pointer-to-pointer, `fn` item to `fn` pointer) are not lossy `as`; leave them to their own rules.
- A cast whose operand has an error type is skipped.

### 1.2 fx-u65.3: trait power ceiling

| Code | Level and mechanism | Notes |
| --- | --- | --- |
| FX0401 | HIR: for each local `impl`, `tcx.parent_module_from_def_id(impl)` must equal the module of the self type's definition (local ADT) or, for a trait impl, the trait's module. | Skip impls a std macro wrote (`written_by_other_crate`). An impl in a fn body counts as its enclosing module. Exact module equality (design silent on descendants; choice C8). |
| FX0402 | AST (`check_expanded`): `default` on an impl or impl item. Overlap: label rustc's E0119 (choice C1), or call `overlapping_trait_impls` [pin: `rustc_trait_selection/src/traits/coherence.rs:130`]. | Existing case `authority/pure_overlap.rs` (`//~ E0119 FX0402`). `default fn` also gets E0658 under `-Zallow-features=`. |
| FX0403 | HIR: an associated type (trait item or impl item) with generic parameters. | Existing case. |
| FX0404 | HIR: a written `for<..>` binder: `PolyTraitRef` or where-predicate `bound_generic_params` with an explicitly written lifetime parameter, and a `fn` pointer type's explicit binder. Elided ones from `Fn(&T) -> &U` sugar are synthesized and allowed. | Distinguish written from elided by the parameter's kind (explicit vs elided) or by `span.from_expansion()`/source text [unverified which field the pin exposes]. |
| FX0405 | AST: negative impl polarity; `auto trait`. | Both are unstable, so rustc adds E0658. |
| FX0406 | HIR + `tcx.predicates_of(impl)`: a trait predicate whose bounded type is neither a type parameter, a projection of one, nor a strict subterm of the impl head (choice C5). | No existing case. |
| FX0407 | Definition check for expanding cycles between local function type parameters (choice C6, adopted with changes below); no instance-limit label. | |
| FX0408 | `tcx.inherent_impls(adt)` for each local ADT: collect associated fn names across all its inherent impls; a repeated name is FX0408 at the later definition. | rustc's E0592 covers only overlapping impls; `impl S<u8> { fn new }` and `impl S<u16> { fn new }` is what rustc accepts and Tide rejects. An overlapping pair carries both codes. |

### 1.3 fx-u65.4: authority

| Code | Level and mechanism | Notes |
| --- | --- | --- |
| FX0501 | `Rule` gains a `code` (default FX0520). `unresolved()` reports the rule's code, so denied Rust-std keys for ambient operations (`std::fs::*` free fns, `std::env::*`, `std::thread::sleep`, `std::process::exit`, `std::io::stdin`) become FX0501. For type-relative paths and method calls that typeck cannot resolve (`Instant::now()`, `t.elapsed()`, `File::open`, `Command::new`, `rx.recv_timeout(d)`): find the Tide type (HIR path res or receiver type), map it to the same-path Rust std type, look for an inherent item of that name, and apply the rule. | Existing cases `authority/cap_ambient.rs`, `cap_elapsed.rs`, `cap_timed_wait.rs`. `thread_rng` is not std: match by name (choice C7). Clippy's `disallowed_methods` is the same idea, a configured path list ([clippy config](https://doc.rust-lang.org/nightly/nightly-rustc/clippy_config/struct.Conf.html)). |
| FX0503 | (a) Naming: a path or type resolving to Tide's `Root`, anywhere but the parameter type of the entry `main` and the qualified path `Root::sim`. (b) Value: any expression whose type, after peeling references, is `Root` and that is not the base of a field access; and a closure capture whose place is the whole `root`. | Existing cases `root_borrowed.rs`, `root_captured.rs`. Edition 2024 closures capture `root.fs` disjointly, so `|| root.fs.read(..)` passes. |
| FX0504 | In `use_def`: a key under `tide_std::sim::` or `Root::sim`, used outside test code. Test code is: an ancestor item carrying a `cfg` trace for `test` [pin: `AttributeKind::CfgTrace`], a fn made by `#[test]`/`#[sim_test]`, or the separate test crate (`--tide-test-library` suite). | In a non-`--test` build every such use is outside test code by construction. Existing case `root_sim_outside_test.rs`. |
| FX0505 | Typed HIR call: the callee is a Tide std fn whose last parameter is `&Tasks`, and the call has exactly one argument fewer than the signature. | No constructor table to maintain. Existing case `cap_handle_no_tasks.rs` (`//~ E0061 FX0505`). Needs typeck to record the callee despite E0061 [unverified]. |

### 1.4 fx-u65.5: concurrency and errors

| Code | Level and mechanism | Notes |
| --- | --- | --- |
| FX0601 | AST (`check_expanded`): `async fn`, `async` block, `async` closure, `.await`. Policy: `core::future::*` and `core::pin::*` under code FX0601 (choice C9). | The HIR walk will also see rustc's async desugaring and may add FX0520 for `core::future` lang items; skip spans whose `desugaring_kind()` is `Async` or `Await` [pin: `rustc_span/src/hygiene.rs`]. |
| FX0602 | `unresolved()` with rules coded FX0602 for `std::thread::spawn`, `std::thread::Builder::*`, `std::thread::scope`; name match for `tokio::spawn`. | |
| FX0604 | `unresolved()` with rules coded FX0604 for Rust's `Mutex` and `RwLock`. | Keys are canonical defining paths, which for std's locks is not `std::sync::Mutex`; print the key on the pin before writing the pattern. `std::sync::raw::*` stays FX0520 without the grant (CAP-3). |
| FX0605 | `check_expansions`: an unresolved bang macro whose last path segment is `select`, exactly as `lazy_static` is matched today. | |
| FX0701 | `unresolved()` with a rule coded FX0701 for `catch_unwind`. | Only the item the reference names. |
| FX0702 | Typed HIR: a `StmtKind::Semi` whose expression has type `core::result::Result<..>`. | rustc's own lint is a late pass over statements that reads the expression's type from typeck ([rustc `must_use.rs`](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_lint/unused/must_use.rs.html)); `Result` carries `#[must_use]` ([std `result`](https://doc.rust-lang.org/std/result/)). Judge provenance by the statement: `writeln!(f, "x");` is the user's ignored `Result`. `let _ = r;` passes. Choice C10. |

### 1.5 fx-u65.6: contracts, unsafe, modules

| Code | Level and mechanism | Notes |
| --- | --- | --- |
| FX0802 | HIR: a trait item or trait-impl item for which `std_attr_made(.., "requires" or "ensures")` holds. Report at the attribute (the expansion's call site). | Needs fx-mhd's macros to re-emit a bodiless trait method instead of failing to parse it (pitfall 8). Existing case `contracts/form_trait_reject.rs`. |
| FX0803 | Two halves. Typed: from `tcx.fn_sig`, the outer region of each `&mut` parameter, and whether it occurs in another parameter's type, the result, or a bound other than the right of an outlives bound. Naming: whether an `ensures` names that parameter outside `old(..)`. | The naming half depends on fx-mhd's lowering (its decision 2 hoists `old(e)` out of the clause closure, so a use of the parameter left inside the `ensures` closure is outside `old`). Read 04 §1.8 CLAUSE-4 and §4.1 for the exact "appears in" list. Existing case `contracts/clause_param_reject.rs`. |
| FX0902 | HIR `visit_block` for a user-provided `unsafe` block, then source text: the lines directly above the block (or above the statement that holds it) must be `//` comments, one starting with `SAFETY:`. | This is how clippy's `undocumented_unsafe_blocks` works: user-written blocks only, a comment search in the source, with known edge cases around attributes ([clippy source](https://rust.googlesource.com/rust-clippy/+/refs/heads/beta/clippy_lints/src/undocumented_unsafe_blocks.rs), [issue 15754](https://github.com/rust-lang/rust-clippy/issues/15754)). Case needs `//@ grants: unsafe`; without the grant the block is also FX0901. Choice C11. |
| FX0910 | An out-of-line `mod foo;` whose file the snapshot loader (`Sources`) could not supply. rustc reports E0583 during expansion. Record loader misses and report in `check_expanded` for modules that survived `cfg`. | A `#[cfg(test)] mod t;` with no file is not an error in a non-test build, as in rustc. Case: `mod missing; //~ E0583 FX0910`. |
| FX0911 | Already reported. Case: `#[path = "x"] mod m {} //~ FX0911`. | An inline module with `#[path]` loads nothing, so stable rustc accepts it and no second file enters `cases/`. |
| FX0912 | AST: a user-written `ItemKind::ExternCrate`. | rustc's injected `extern crate std` has a compiler-generated span. |
| FX0913 | Read rustc's own results (choice C2): the buffered `UNUSED_IMPORTS` early lints in `resolver_for_lowering().0.lint_buffer`, plus the trait-import half from `maybe_unused_trait_imports` and `used_trait_imports`. | |
| FX0914 | HIR `ItemKind::Use`: an edge from the importing module to the module that defines the imported item (or the module itself for a module import); report each `use` that lies on a cycle (choice C12). | |
| FX0915 | Already reported; no source line. | Cite `crates/tide-admit/tests/invocation.rs` (it asserts the code at lines 325 and 336) after making that test also assert `error` severity and exit status 1, which the bead's Acceptance allows. Cheaper than teaching `run.sh` directory cases. |

### 1.6 Shared infrastructure and who builds it

1. **A rules table for reference-only rules** in `conformance/rules.md`. Every case needs a `//@ rules:` ID that the file lists, and FX0102, FX0103, FX0210, FX0401, FX0405 to FX0408, FX0601, FX0602, FX0604, FX0605, FX0701, FX0702, FX0902, FX0910, FX0912 to FX0914 have no row [repo]. Add a section sourced from `01-design.md` (D3, D4, D5, D6, D8, D12) and the reference. **fx-u65.2 adds the section**; later slices add rows.
2. **A provenance helper** `user_written(span)` unifying `written_by_other_crate` and `generated_by_toolchain`. **fx-u65.2**.
3. **The rustc-verdict label layer** (choice C1), if adopted. **fx-u65.2**, since FX0203 needs it first; .3 reuses it.
4. **`Rule.code`** in `policy.rs` and its use in `reject`/`unresolved`. **fx-u65.4**; .5 reuses it. If .5 runs first, it builds it.
5. **A test-code predicate** (FX0504). **fx-u65.4**.
6. **New checks in new files** under `walk/` (`surface.rs`, `traits.rs`, `authority.rs`, `habits.rs`, `modules.rs`), as `sinks.rs` and `mono.rs` already are. All five slices edit `walk.rs`, `diag.rs` and `rules.md`; keeping each slice's body in its own file limits each rebase to the registry rows, which the AP-PLAN-001 exception requires reviewers to re-check.

Suggested landing order: .2, .4, .5, .3, .6 (.6 waits on fx-mhd).

## 2. Open design choices

**C1. Rules that rustc already decides ("both" rows): re-decide or label?** Affected: FX0203, FX0210, FX0402 (overlap), FX0407 (instances, excluded by C6's adoption note), and optionally FX0910.

- A. Re-decide in the walk. Fails for FX0203: typeck results erase regions, so a capture of `&'static str` cannot be told from `&'a str`, and rejecting every reference capture would reject code the reference allows (§3.4: owned means borrowing nothing except `'static` data).
- B. Label rustc's verdict. On the pin a driver can install its own emitter (`DiagCtxt::set_emitter`, and `Emitter::emit_diagnostic(DiagInner)`) [pin: `rustc_errors/src/lib.rs:496`, `emitter.rs:53`]. A wrapper forwards every diagnostic and, when an error matches a mapped code and lies within a site the walk recorded, emits one extra `FXnnnn` error at the same span.
- C. Leave rustc's code and drop the FX mark. That weakens the bead's Acceptance, which is the owner's call.

Recommend **B**, scoped narrowly: E0119 to FX0402 with no site test; no recursion-limit label for FX0407 (see C6's adoption note); region and borrow errors (E0373, E0521, E0597, the uncoded "lifetime may not live long enough") to FX0203 only inside recorded candidate sites; E0505/E0515/E0597 to FX0210 only inside recorded struct literals. Reason: rustc stays the single owner of the borrow and coherence verdicts (AP-OWNERSHIP-001) and the driver only names them. The rejection itself never depends on the label, so a missed label costs a code, not soundness. If the wrapper proves fragile, FX0402 has a clean fallback (`overlapping_trait_impls`); FX0203 does not.

**Adopted with changes (fx-u65.2)**. Labels prefer precision over recall; uncertain labels are omitted and rustc retains its error and rejection. E0373 uses rustc's closure/borrowed-place spans.
Other FX0203 errors require a certain whole-binding chain (`&`/`&mut` or receiver-tied borrow methods); projections, destructuring and reassignment invalidate it.
FX0210 records only literals moving a whole local into one field and borrowing it in another, including a borrow projection rooted in that local. See [conformance/README.md](../conformance/README.md) for the current rules and limits.

**C2. FX0913: rustc's lint or a reimplementation?** rustc computes unused imports in two places: `rustc_resolve/check_unused.rs` buffers `UNUSED_IMPORTS` as an early lint, and defers trait imports to `rustc_hir_analysis/check_unused.rs`, because whether a trait import is used is known only after method resolution ([resolve check_unused](https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_resolve/check_unused.rs.html)) [pin: both files].

- A. Pass `-D unused_imports`. The crate is rejected, but the diagnostic reads `error: unused import`, with no FX code, and `#[allow]` or `--cap-lints` can silence it ([rustc lint levels](https://doc.rust-lang.org/rustc/lints/levels.html)).
- B. Reimplement usage tracking. Duplicates the resolver.
- C. Read rustc's results and re-report: iterate `lint_buffer.map` for `lint_id == UNUSED_IMPORTS` in `check_expanded` (the buffer is public and not yet stolen there) [pin: `rustc_errors/src/decorate_diag.rs:39-57`, `rustc_middle/src/ty/mod.rs:269`]; for trait imports repeat the 20 lines of `check_unused_traits` in the walk.

Recommend **C**, with `-A unused_imports` passed to rustc so the user sees one diagnostic. It ignores lint levels, which fits an error. The trait half matters: the companion's own example (`use std::io::Read;` beside `stdin.read_to_string`) is an unused trait import.

**C3. FX0202's second half.** The reference says an elided output "that elision ties to the wrong input" is also FX0202; 02 LIFE-4 and `rules.md` cover only the unresolved case. The wrong-input case is a borrow error in the body, which rustc reports as a region error. Recommend: FX0202 for LIFE-4 only in this slice; rustc still rejects the other case. Say so on the bead. Labelling it would need C1's layer with fn bodies as sites, which over-labels unrelated region errors.

**C4. FX0210 has no rejectable form.** `struct Doc<'a> { text: String, words: Vec<&'a str> }` is a legal type; only building it self-referentially fails, in rustc's borrow checker. No chapter gives a rule ID. Options: (A) label rustc's error inside a struct literal of a local lifetime-parameterized type; (B) record FX0210 as an explanation code (`tide explain FX0210`) with no driver check, by an owner-approved change to the epic's Acceptance. Recommend **A** as the only reading that meets "a violating case gets that code" without a new rule; it is a heuristic and the bead should say so.

**C5. FX0406 "structurally smaller" is undefined.** The one example is `impl<T> Foo for T where Vec<T>: Foo`, fix "bound `T` directly". Haskell's equivalent, the Paterson conditions, requires each constraint to have fewer constructors and variables than the head and no variable more often ([GHC user guide, instance termination](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)). Applied literally that rejects `impl<T: Pure> K for T`, a blanket impl the reference itself writes. Recommend the subterm reading: a where-clause or bound on an impl is accepted when its bounded type is a type parameter, a projection of one, contains no type parameter, or is a strict subterm of the impl's self type or trait arguments; otherwise FX0406. It accepts the stated fix and rejects the stated example. Record it in 01 D4 as one sentence.

**C6. FX0407 scope.** rustc detects polymorphic recursion only when an instance is collected, as "reached the recursion limit while instantiating" ([rust issue 43520 mirror](https://google.18789066745.workers.dev/rust-lang/rust/issues/43520), [issue 88925 mirror](https://github.colasdn.workers.dev/rust-lang/rust/issues/88925)), and some cases take hours to hit the limit ([issue 132558 mirror](https://scriptagc.wasmer.app/https_www_github_com/rust-lang/rust/issues/132558)). The reference's example, an uncalled generic fn in a library, is never instantiated, so rustc accepts it. Recommend a definition check: among local generic fns, in each call-graph cycle, a call that passes a generic argument containing one of the caller's type parameters inside a constructor (`Vec<T>`, `&T`, a closure type) is FX0407. Permuting parameters (`f::<B, A>`) stays legal because it cannot grow. Calls through a trait method on a type parameter are not resolvable at the definition; leave those to rustc's unlabelled instantiation checks.

**Adopted with changes (fx-u65.3)**. No instance-limit label is added: finite
chains hit the same rustc limit, so the error alone cannot prove forbidden
growth. The definition check uses the expanding-cycle refinement from
[ECMA-335 §II.9.2](https://carlwa.com/ecma-335/#ii.9.2-generics-and-recursive-inheritance-graphs),
applied to (function, type parameter) nodes. Each occurrence of a caller type
parameter in a callee argument adds an edge to that callee parameter; an
occurrence under a constructor marks it expanding. FX0407 fires only when an
expanding edge lies on a cycle, so resets and idempotent wrapping remain legal.
Projections normalize in the caller's environment; unresolved aliases prove no
parameter flow. Trait dispatch contributes edges only for selected local impls:
to their overriding method with substituted impl arguments, or to the default
body when that impl has no override. Parameter, object, builtin and ambiguous
dispatch add no edge; rustc retains its checks.

**C7. Which calls are "ambient" (FX0501)?** §6.9's list ends with "...". Recommend a closed table in `policy.rs`: the named items plus the other items of the same Rust std modules that perform an operation in §6.1's capability table (file openers, `std::env` readers, `Instant`/`SystemTime::now` and `elapsed`, `thread::sleep`, `process::exit`/`abort`, `Command::new`, `io::stdin`, socket constructors), and by name `thread_rng`. Anything else keeps today's verdict. Reason: a closed table is testable row by row.

**C8. FX0401 details.** Design: "Impls live in the module of the type or the trait." Recommend exact module equality; for a trait impl either module qualifies; when the self type is not a local ADT, the trait's module is the only choice.

**C9. `Future` and `Pin`: FX0601 or FX0520?** Reference §7 names "`async`/`await`, `Future`, `Pin`" under FX0601; fx-yg5.5's decision 3 lists `core::pin` as a reference-silent FX0520 denial; `rules.md` fixes `Waker` at FX0520. Recommend FX0601 for `core::future::*` and `core::pin::*`, FX0520 for `core::task::*`, and a one-line amendment to fx-yg5.5's note. The reference is the text writers learn from.

**C10. FX0702 breadth.** Options: (A) exactly `Result` in statement position; (B) everything rustc's `unused_must_use` covers, through `rustc_lint::unused::must_use::is_ty_must_use`, which is public on the pin and already exempts `Result<(), Uninhabited>` [pin: `must_use.rs:137-170`]. Recommend **A**: the reference says "an ignored `Result`", and B would reject unused iterator adapters and other `#[must_use]` values the reference never mentions. Under A, `Result<(), Infallible>;` is also an error; the design is silent and "ignored `Result`" is flat.

**C11. FX0902 placement and scope.** The reference says each `unsafe { .. }` block; a comment in `memory/drop_may_dangle.rs` suggests its author expected `unsafe impl` to need one too. Recommend blocks only, with or without the grant, comment on the lines directly above the block or its statement (clippy's default); fix that case's comment in the same change.

**C12. FX0914 edge definition.** The design gives two facts: cycles between modules are errors, and a child's `use super::*` is fine while the parent does not import the child. Recommend: edges come only from `use` items (`pub use` and globs included), never from `mod` declarations or qualified paths in bodies; the target is the module defining the imported item; external crates give no edge. A parent's `pub use child::T` together with the child's `use super::*` is therefore a cycle, as the text says.

**C13. Do `#[allow(..)]` attributes silence FX errors?** §2.3 keeps `#[allow]`. Recommend no: FX0702 and FX0913 are errors, not lints, and the three lint codes keep fx-u65.1's fixed severities.

## 3. Pitfalls

1. **Adding a code to `Code::ALL` enforces it corpus-wide.** The runner then requires every case's marks in that code to match exactly, and fails any case where the code fires unmarked. FX0913 and FX0702 will fire in existing accepted cases, driver test fixtures and std-facing examples. Budget a corpus sweep in .5 and .6.
2. **Two builds per case.** The runner unions a library or binary build and a `--test` build. An import used only by `#[cfg(test)]` code is unused in the first; rustc warns there too, so FX0913 firing is consistent, but accepted cases must import inside `mod tests`.
3. **Typeck with errors.** Skip a check when the expression's type is an error type. Skip the trait-import half of FX0913 when the crate already has errors: `used_trait_imports` is incomplete after a failed method lookup and would add a false FX0913.
4. **Async desugaring** adds `core::future` uses with `DesugaringKind::Async`/`Await` spans; without a skip, one `async fn` yields FX0601 plus stray FX0520s and the exact-match runner fails.
5. **Deduplication.** `Diagnostics` keeps one diagnostic per code, message and line. Two violations of one code on one line collapse, which is fine for marks but not for tests that count.
6. **Canonical keys.** Policy keys are defining paths, not re-export paths. Print the key for each new deny rule on the pin before writing its pattern.
7. **Label layer ordering.** Analysis errors arrive after the walk's diagnostics. The wrapper must hold the walk's site sets before analysis starts, must not label warnings, and must emit through the inner emitter so the error count stays right.
8. **Contract macros.** FX0802 and FX0803 see only what `tide-macros` emits. The macros must keep user tokens' spans and must not swallow a bodiless trait method. Tell fx-mhd's implementer before .6 starts.
9. **Held-out rule.** Cases and diagnostic text come from the reference and companion examples only.

## 4. Cheapest falsifying tests

| Recommendation | Falsifier |
| --- | --- |
| C1 label layer | One driver test: a file with overlapping impls prints both `E0119` and `FX0402` on one line and exits 1; a file with an unrelated E0597 prints no FX code. |
| FX0203 sites | `closure_escape_reject.rs` passes as marked, including its `counter()` line (E0373, no FX0203). A twin moving a `&'static str` into `spawn_isolated` is accepted. |
| C2 lint buffer | Driver test: `use std::collections::BTreeMap;` unused gives FX0913; an unused `use std::io::Read;` gives FX0913; a `pub use` and an import used once give none. If the buffer is empty at `check_expanded`, the first assertion fails at once. |
| FX0202 by `LifetimeKind::Error` | `life_elision_reject.rs` passes; `fn f(x: &str) -> &str` gives no FX0202. |
| FX0301 table | A table test, one row per §4.2 sentence: `u8 as u16`, `u32 as usize`, `usize as u64`, `u16 as f32`, `u32 as f64`, `bool as u8`, `u8 as char`, `char as u32`, `f32 as f64` accepted; `u64 as usize`, `i32 as u32`, `u32 as f32`, `usize as f64`, `f64 as f32`, `f64 as i64`, `u32 as char`-style lossy rows rejected; plus `address_casts_lossy.rs`. |
| FX0103 on the user's macro only | `println!("{}", 1)` and `assert_eq!(a, b)` accepted; `debug_assert_eq!(a, b)` and `concat!("a", "b")` give exactly one FX0103 each and no FX0520. |
| C5 subterm rule | `impl<T: Pure> K for T` accepted; `impl<T> Foo for T where Vec<T>: Foo` rejected; `impl<T> Foo for Vec<Vec<T>> where Vec<T>: Foo` accepted. |
| C6 definition check | `fn f<T>(x: T) { f(vec![x]) }` uncalled in a library is FX0407; `fn g<A, B>(a: A, b: B) { g(b, a) }` and plain `fn h<T>(x: T) { h(x) }` are accepted. |
| FX0408 | Two `fn new` in `impl S<u8>` and `impl S<u16>`: FX0408 with no rustc error. |
| FX0503 typed rule | `root_borrowed.rs` and `root_captured.rs` pass; `let fs = root.fs; let c = &root.clock; let f = || root.fs.read("x");` accepted. |
| FX0504 predicate | `root_sim_outside_test.rs` passes; the same call inside `#[cfg(test)] mod tests` and inside a `#[test]` fn is accepted in the `--test` build. |
| FX0505 arity rule | `cap_handle_no_tasks.rs` passes; if typeck drops the callee on E0061 this fails immediately and the fallback is a HIR path match. |
| FX0601 skip | One `async fn` with one `.await`: exactly two FX0601 and nothing else from the driver. |
| C10 statement rule | `r;` and `writeln!(s, "x");` rejected; `let _ = r;`, `r?;`, `r.ok();` accepted. |
| FX0902 | With the grant: block without comment rejected; `// SAFETY: ..` on the line above accepted; a trailing same-line comment rejected. |
| FX0910 | `mod missing;` gives `E0583 FX0910`; `#[cfg(test)] mod missing;` in the library build gives nothing. |
| C12 | Two inline modules importing each other: each `use` line marked; `mod a { pub struct T; } mod b { use super::a::T; }` accepted; a child with `use super::*` accepted. |
| FX0915 | The strengthened invocation test: code, `error`, exit 1. |

## 5. Where the design is silent or ambiguous

Choices C3 to C13 each answer a silent or ambiguous spot (FX0202's second half, FX0210, FX0406, FX0407, FX0501's list, FX0401, `Future`/`Pin`, FX0702's breadth, FX0902, FX0914's edges, `#[allow]`). None adds a language rule. Four more:

1. FX0503: whether destructuring `root` in a pattern counts as moving its fields. Recommend reject, by the uniform typed rule. `SimRoot` is unconstrained by ROOT-2 as written.
2. FX0604 names only `Mutex` habits; `Condvar`, `Barrier` and `Once` fall to rustc's unresolved-name errors. Recommend leaving them.
3. §2.3 calls `#[tokio::main]`, `#[async_trait]`, `#[macro_export]` and `#[proc_macro*]` "errors with fixes" without codes. No slice lists them; today they surface as rustc errors, FX0101 or FX0520. Reported for the coordinator; no change recommended.
4. Nineteen of these codes have no row of their own in `conformance/rules.md` (§1.6 item 1). That traceability gap is separate from enforcement, and closing it is the first shared step.
