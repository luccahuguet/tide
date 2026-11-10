Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-3iu round 1: Opus review (converge round, reports only)

Candidate `cadea79` on base `07e104b` (`git diff 07e104b cadea79`, 16 files).

## Verdict

**approve**. Confidence: 0.8.

Nothing blocks. The derive matches thiserror 2.0.21 for every form the Acceptance
names. Each form runs as an admitted binary and as an admitted test harness. The
walk.rs change is safe: it replaces a driver panic and weakens no check. The
non-blocking items below are small cleanups, diagnostic-quality gaps, and one test
that claims more than it proves.

## Scope

Read:
- AGENTS.md (AP-DEPENDENCY-001, AP-MINIMAL-001), the GOVERNANCE.md Merge Rule,
  bead fx-3iu with its comments, parent fx-nkv's acceptance, the brief, and
  impl-report.md.
- The full diff, plus the code it relies on:
  - `crates/tide-macros/src/{lib,error}.rs`, `crates/tide-std/src/{error,root,lib}.rs`
  - `crates/tide-admit/src/walk.rs`: `use_ty`, `use_generic`, `clause_obligations`,
    `generated_by_toolchain`, `check`/`granted`/`written_by_other_crate`,
    `visit_path`/`visit_qpath`/`visit_expr`/`visit_pat`, `check_expansions`
  - the policy.rs header and the address/`dyn Any` rules
  - `crates/tide-admit/tests/build.rs`, `crates/tide-std/tests/entry.rs`
  - `conformance/cases/errors/derive.rs`, `conformance/rustc/std/{Cargo.toml,src/lib.rs}`
- Spec: reference.md §2.2, including FX0102 and the line on std error types being
  `Error + Send + Sync + Pure`, and design/01-design.md D5.
- Upstream, as the reference for thiserror's behavior: the locally cached
  thiserror-impl 2.0.21 (`valid.rs`, `expand.rs`, `fmt.rs`, `prop.rs`,
  `ast.rs`, `attr.rs::parse_token_expr`).

Ran: four single-file driver probes with the candidate's built driver
(`target/debug/tide-admit`, 23:22, after the last walk.rs edit at 23:21). Each
used the candidate's built std (`target/tmp/build/std/libtide_std.rlib`,
`libtide_macros.so`) and `nice -n 19`, and each ran only after checking the
quiet-window, load < 12 and ≥ 6 GB gate. Sources and outputs are in my
scratchpad, and nothing was written to the repository:
- P1 (positive): a generic enum mixing a `#[error("a {0}")]` variant with a
  `#[error(transparent)] B(std::io::Error)` variant. Admitted, ran, printed `OK`.
- P2: `Bounded<T: Pure>` deriving Error, with `main` doing only
  `let _b = Bounded::Value { value: root.net };` (no `inspect` call). Rejected:
  `8:14: FX0517: Net is never Pure`, at the constructor.
- P3: an enum without an enum-level message, where variant `A` has `#[error]` and
  `B` has none. Rejected with "missing #[error] attribute on variant" at `1:17`,
  the `derive` attribute, not at variant `B`.
- P4: `{r:p}` in a message. Rejected: `FX0520 core::fmt::rt::Argument::new_pointer`,
  reported at `1:17`. A second file had `#[error("tuple {1}", 9)] struct Amb(u8);`
  (rustc rejects it as an invalid positional reference) and
  `#[source] inner: &'a io::Error` (rustc's "lifetime may not live long enough",
  at `5:17`, the derive).

Not run: no cargo builds or tests (heavy slot). The implementer's 77-pass workspace
run and 198 PASS / 0 FAIL conformance run are their claims, not my proof.

## Blocking

None.

## Non-blocking

**N1. `generic_error_patterns_preserve_pure_obligations` does not exercise the
walk.rs fix** (`crates/tide-admit/tests/build.rs:533`).
- The test's FX0517 comes from `use_ty` on the resolved-path constructor
  `Bounded::Value { value: root.net }`. P2 shows that site rejects on its own,
  without the derive's `Self::Value {..}` patterns and without `inspect`.
  `inspect<T: Pure>` would reject too.
- So the test still passes if `path_args` returns any args that do not panic. It
  proves the end-to-end property, not "the fix preserves obligations" as the
  report says.
- Not blocking, because `path_args` cannot weaken admission (see Decision 5).
- Fix: rename it to what it shows, for example `generic_error_types_still_require_pure`,
  or drop it. The crash regression is already covered by the fixture's `Choice<T>`:
  generic `Self::Variant {..}` patterns and the `Self::Converted { 0: .. }` constructor.

**N2. Diagnostic spans diverge from thiserror** (`crates/tide-macros/src/error.rs:263-267`
and `:487-492`).
- "missing #[error] attribute on variant" and "#[from] requires no fields other
  than the source" use `new_spanned(fields, ..)`. For a unit variant that is an
  empty token stream, so the error points at the derive (P3, `1:17`).
- thiserror spans the variant and the `#[from]` attribute respectively.
- A non-`'static` lifetime in a source field (P4) gets rustc's generic lifetime
  error at the derive. thiserror reports "non-static lifetimes are not allowed in
  the source of an error" at the field type.
- Fix: span the variant (keep `&v` in `bodies`) and the `#[from]` attribute.
  Optionally port thiserror's `contains_non_static_lifetime` check.

**N3. Small permissiveness differences from thiserror 2** (`error.rs:309-315`, `:208-220`).
- The positional-ambiguity error fires only when a `{N}` names an existing tuple
  field. thiserror rejects any `{N}` in a container with unnamed fields once extra
  positional arguments are present. Example: `#[error("{} {1}", .0, 7)]` on a
  1-field tuple struct is accepted here and rejected by thiserror.
- `option_inner` accepts `Option<A, ..>` by taking the first argument. thiserror
  requires exactly one argument.
- A malformed enum-level `#[error(..)]` on an empty enum is never parsed, so it
  goes unreported.
- An enum with some `#[error(transparent)]` variants, some unmarked ones and a
  hand-written Display is rejected here. thiserror accepts it and generates no
  Display.
- All are rare and none is unsound. Fix if cheap; otherwise list them in the
  tide-std docs (N4).

**N4. Unsupported thiserror features are not labelled** (`crates/tide-std/src/lib.rs:16`, README.md:162).
- The docs say "the `Error` derive with thiserror semantics".
- `#[error(fmt = path)]` is rejected as "expected a string literal or `transparent`".
- `#[backtrace]` is an unknown attribute.
- `Backtrace` fields and `provide` are not detected.
- reference §2.2 lists only the supported forms, so the spec is fine. AP-CONTRACT-001
  asks the supported-surface docs to label partial behavior: add one clause, for
  example "(no `fmt =`, backtrace or `provide`)".

**N5. The test-time macro build is less hermetic and less portable than before**
(`crates/tide-admit/tests/build.rs:27-35`, `crates/tide-std/tests/entry.rs:38-48`).
- `build.rs` dropped `env_clear()`, so inherited `RUSTFLAGS`, `CARGO_*` and
  ancestor `.cargo/config.toml` now shape the trusted macro build.
- `--offline` without `--locked` lets a stale lock be rewritten in the repository
  during a test run.
- Both files hard-code `libtide_macros.so`. entry.rs then derives the name from
  `DLL_PREFIX`/`DLL_SUFFIX` a few lines later, which disagrees on any non-Linux host.
- Fix: use `--frozen` instead of `--offline`, and use one computed file name in
  both places. This is acceptable for the supported Linux host.

**N6. Unrelated rustfmt churn** (`crates/tide-std/tests/entry.rs:168-171`): the
`assert!` reflow. Drop it, or accept it as harmless.

**N7. The `{:p}` path is untested** (`error.rs:333,342-343`; `tide-std/src/error.rs:47-52`).
- Pointer formatting needs the `unsafe` grant (reference "Addresses" row, P4), so
  this path matters only in granted packages, where it is a recorded trust hole.
- The `Pointer` wrapper mirrors thiserror's `Var` and is correct by inspection,
  but nothing runs it. Add a granted positive, or accept the gap.

**N8. Diagnostic location for policy rejections inside messages.**
- A rejected item reached through a message's format machinery is reported at the
  derive (P4 `1:17`), not at the format literal, because the `write!` call site is
  the derive expansion.
- This is a property of `reject`'s `source_callsite`, not of this change. Not a
  hole. Mentioned only for awareness.

**N9. Rebase note.** main has since gained fx-u65.1's lint codes and fx-yg5.1's
collections. The derive's generated impls should run under those lints once. The
re-proof after the rebase covers this, as long as it includes
`error_derive_builds_and_executes_against_production_std` and
`conformance/cases/errors/derive.rs`.

## Decisions

**1. syn/quote/proc-macro2 (AP-DEPENDENCY-001). Agree.**
- Capability: parsing Rust items, generics, where-clauses and expressions with
  spans, and building tokens. The record names this, the rejected alternatives,
  and the replacement cost.
- The crates are host-only, toolchain-trusted, MIT/Apache, and pinned with `=`.
  The versions are already locked by the conformance oracle, so no new crates enter
  the tree.
- A hand-written parser for full generics, where-clauses and arbitrary argument
  expressions would be more owned code, and riskier, than the whole derive.
- Vendoring thiserror would duplicate roughly 2k lines and still need its
  `::thiserror` paths rewritten.
- fx-08m and fx-mhd (Encode/Decode, further macros) will reuse the dependency.
  That is a real second caller, not speculation.
- Cost: syn's build time on the toolchain's critical path (research/03), host-side
  only. Accepted.
- The `visit` feature exists only for `uses_generic`. Keep it, since it is cheap
  and correct.

**2. Transparent source delegates to the inner error's `source()`. Agree.** This
is thiserror's exact expansion: `Error::source(transparent.as_dyn_error())`. The
fixture runs both the non-leaf and the leaf case.

**3. Source access through `AsDynError` method resolution, plus Option handling. Agree.**
- This copies thiserror's `AsDynError`, including the four `dyn Error + ..`
  impls, its Option handling and its From-wraps-`Some`.
- Method probing picks `Self = FieldTy` before the `&T: Error` blanket impl.
- The `downcast_ref::<Leaf>()` assertions prove the source is `Leaf`, not
  `&Leaf`.

**4. Shared definitions through root.rs, with `__error` support. Agree.**
- Production and conformance get the same derive and the same support items by
  `include!`, so there is one owner and fx-nkv's equivalence holds by
  construction.
- The conformance oracle's E1 `Error` import is removed and the Cargo.toml comment
  corrected.
- `std::__error` items are user-nameable under the TideStd default allow. They are
  safe, authority-free helpers, the same as `__sink`, so this is acceptable.

**5. walk.rs `path_args`. Agree: sound, no check weakened.** Reasons:
- Where it applies. It changes args only when typeck recorded fewer args than the
  def's generics and the def is a `Struct`, `Variant` or `Ctor`, which is
  type-relative struct paths and patterns such as `Self::V { .. }`. rustc's
  `check_struct_path` records the resolution but writes no `node_args` there.
- What it changed before. Before this change, `use_generic` instantiated the
  where-clauses with empty args and panicked. That was a driver crash on any
  generic `Self::V {..}`, not a check.
- Where the new args come from. They are the ADT args of the node's own type, as
  typeck computed it. The variant/ctor generics are the parent ADT's, so the
  counts match. A user cannot choose them independently of the type that rustc
  itself checks.
- Obligations are still enforced elsewhere. `visit_expr` and `visit_pat` already
  call `use_ty` on the same node's type, and `use_ty` runs
  `use_generic(adt, args, admit=true)` with the same ADT args. The ADT's
  where-clauses, including Pure, were already enforced at that node regardless of
  `path_args`, and P2 confirms `use_ty` rejects there. So `path_args` adds checks
  (and `named()` now sees real types) rather than removing any.
- Otherwise nothing changes. Any other short-args case keeps the old behavior.
- The derive gains no trust. Its expansion is a `MacroKind::Derive`, so
  `generated_by_toolchain` does not trust it; only `#[test]` and the entry
  `#[tide::main]` are trusted. `written_by_other_crate` affects only `granted()`,
  and the derive emits no unsafe code. User expressions inside `#[error(..)]`
  keep their own spans, so `unsafe` blocks and denied items in them are still
  attributed to the package (P4 shows a policy rejection through a message).
  Hand-written code shaped like the derive's output gets exactly the same checks.
- Only the accompanying test is weak (N1).

**6. Cargo builds the trusted macro at an explicit path in tests. Agree, with N5's
hardening.** Once the crate has dependencies, resolving them by hand would be
brittle. Cargo with an explicit `--emit=link=` path is layout-independent, and the
README's demo already uses Cargo's `compiler-artifact` filenames.

**7. Std error types: only the existing ones (io::Error, IntoInnerError,
TaskError). Agree.** This follows the 2026-11-09 plan-review comment. The fixture
asserts `io::Error` and `TaskError` are `Error + Send + Sync + Pure`, with their
Display text.

**policy.rs.** `allow("<(dyn core::error::Error*)>::*")` follows the existing
`dyn Any` rule. It admits `is`, `downcast_ref` and `downcast_mut`; the unstable
inherent methods stay feature-gated. The derive's own output does not need it;
the fixture's downcasts do, and so does normal user code on `source()`. One line,
justified.

## Reasoning

**Acceptance, checked against thiserror-impl 2.0.21's source.**
- The format rewrite is the same as thiserror's `expand_shorthand`:
  - the trait comes from the spec's last character
  - bare `{field}` uses `as_display`, so Path and PathBuf print without `.display()`
  - a spec without a `p` passes the raw binding
  - `{:p}` goes through the `Var`/`Pointer` wrapper
  - explicit named arguments shadow fields and get no inferred bound
  - `{r#x}` passes through to rustc
  - the tuple/positional ambiguity error exists (narrower, N3)
- Enum-level messages and transparent are inherited by variants as in `ast.rs`.
  `has_display` agrees with thiserror, including for empty and all-transparent
  enums.
- Inferred bounds are the same: Display bounds on generic fields, `Error + 'static`
  on generic sources, `Error` on transparent fields, and `Self: Debug + Display`
  on generic Error impls.
- Source selection matches: `#[from]`/`#[source]` first, then a field named
  `source`.
- From is generated, including the Option-wrapping `Some`, with the original
  generics.

**Fixture.**
- It asserts exact strings for: field, positional and format-spec interpolation,
  escaped braces, expression arguments (including `.0.len()`, `.1 + 2` and `.0.0`),
  explicit named arguments, collision-shaped names, raw identifiers, paths, const
  generics, lifetimes and where-clauses.
- It asserts `source()` identity by downcast for `#[source]`, a `source` field,
  Option, `Box<dyn Error + Send + Sync>`, transparent and generic sources.
- It covers From both through `?` and through `.into()`, plus manual Display and
  the empty enum.
- It runs as a binary and under the test harness on the production std, and it
  passes in conformance on both the stable oracle and the driver (implementer's
  claim).
- The implementer also ran it against real thiserror 2.0.21, so this one fixture
  serves as a differential oracle. That is a strong test for the risk.
- The nine malformed-attribute negatives assert the message and line.
- I saw no E1 material. The fixture's types are generic, and the bead's held-out
  rule is respected as far as I can tell.

**Leftovers.**
- Both lockfiles are consistent with the `=` pins.
- The conformance comment is corrected and the tide-std docs moved the derive from
  unavailable to supported. README updated.
- The only stray is the rustfmt reflow (N6).
- No orphaned code. `Pointer` is reachable under the `unsafe` grant (N7).

DONE
