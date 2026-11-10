Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence.** The implementation direction is sound, and the executed acceptance fixture passes. Four confirmed compatibility defects remain in the derive. They reject valid thiserror inputs, reject ordinary named-field errors when warnings are denied, or accept malformed enum attributes.

What this change does: it adds the prelude Error derive, its shared runtime helpers, and execution tests against production std. It also repairs generic argument recovery in the admission walk. I found no admission bypass in that repair or in the new allowlist rule.

## Scope

- Reviewed `07e104b55ca8e97bc9e2d9ea8d76f6894a62c9f9..cadea79bbbb959431cf3cff186a27851e7a99848`, all 16 changed files, and their relevant callers and supporting code. Read `AGENTS.md`, `GOVERNANCE.md`, README, both beads with `br --no-db`, the implementation report and supplied proof logs, reference §1/§2.2 and FX0102, D18/the tier table, and memory §9.
- Inspected the macro parser/generator, production root/prelude/error helpers, existing io/task error implementations, conformance std wiring, both changed test builders, and the admission policy, generic-obligation/Pure paths, and mono-item backstop. Compared thiserror 2.0.21's cached `attr.rs`, `ast.rs`, `fmt.rs`, `expand.rs`, `valid.rs`, `prop.rs`, `generics.rs`, and runtime helpers. The public reference was [thiserror 2.0.21 documentation](https://docs.rs/thiserror/2.0.21/thiserror/).
- Created the allowed private archive at `/tmp/fx-3iu-codex-r1.BHIwzY`. Independently recomputed the changed-file content digest: `7227cd9552754986e93a035332187ee56a7e8bc7b1987dd915fe18e0ad3a06c7`, matching the implementation report.
- All compilation probes used private source/output paths, `nice -n 19`, closed stdin, and a timeout, sequentially. Before each probe I checked the specified quiet window, load below 12, and available memory at least 6 GB. Some early attempts were skipped by that gate. Completed probes below started with load approximately 4.6–8.4 and at least 14 GB available.
- **Artifact qualification:** no Cargo build or test was run. I copied the existing candidate `target/tmp/build/std/{libtide_std.rlib,libtide_macros.so}`, `target/debug/tide-admit`, and thiserror oracle artifacts into the private archive. These probes independently exercise those binaries; they do not independently establish a clean rebuild of their sources. The supplied logs report 77 workspace tests and 198 conformance PASS / 25 AWAITS / zero failures; those remain implementer evidence.
- No repository/worktree or Beads writes, no E1 held-out source reads, and no E1 task execution. The worktree was clean at candidate HEAD at the end of inspection.

Probe commands and results used in this report follow. Let `R=/tmp/fx-3iu-codex-r1.BHIwzY`, `A=$R/probes/artifacts`, and `C=/home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc`. The differential probes used this command template, once per side, after the resource gate:

```sh
nice -n 19 timeout 40 "$C" --edition=2024 --error-format=short \
  --extern "std=$A/libtide_std.rlib" -Ldependency="$A" \
  "$R/probes/CASE.rs" -o "$R/probes/CASE_tide" </dev/null
nice -n 19 timeout 40 "$C" --edition=2024 --error-format=short \
  --extern "thiserror=$A/libthiserror-b5b9aa9aa9ac69e9.rlib" -Ldependency="$A" \
  "$R/probes/CASE_oracle.rs" -o "$R/probes/CASE_oracle" </dev/null
```

Oracle sources add `use thiserror::Error;` and otherwise retain the tested behavior. Successful binaries ran under `nice -n 19 timeout 10` with closed stdin. The `deny_warnings` oracle keeps its inner attribute first.

| Probe | CASE / behavior | Observed result |
| --- | --- | --- |
| P1 | `keyword`: conditional expression using `.0.is_empty()` after `if` | Tide compilation exits 1 at the shorthand; thiserror compiles and executes the exact-string assertion, exits 0. |
| P2 | `override`: enum-wide `#[error("{}", .0)]`, tuple variant, overridden unit variant | Tide exits 1 parsing the unused default for the unit variant; thiserror compiles and executes assertions for both variants, exits 0. |
| P3 | `empty_bad`: `#[error(12)] enum E {}` | Tide compiles and runs, exits 0; thiserror exits 1 at the malformed attribute. |
| P4 | `deny_warnings`: named-field interpolation under `#![deny(warnings)]` | Tide exits 1 for `non_shorthand_field_patterns`; thiserror compiles and executes, exits 0. |
| P5 | `zero_pad`: `#[error("{00}")] struct E(u8)` | Tide exits 1, “invalid reference to positional argument 0”; thiserror compiles and executes `E(7).to_string() == "7"`, exits 0. |
| P6 | `missing_attr`: one formatted variant followed by an unannotated unit variant | Both reject; Tide points at the derive on line 1, thiserror at the missing variant on its line 6 (line 5 without the oracle import). |
| P7 | `dynamic_width`, `precision`: named dynamic format width and precision | Both implementations compile and execute the exact-string assertions, exit 0. |

The admission probes used this template, again resource-gated individually:

```sh
nice -n 19 timeout 50 "$A/tide-admit" --crate-type=bin --emit=link \
  --crate-name=probe --error-format=short --tide-std "$A/libtide_std.rlib" \
  --tide-dep-dir "$A" --out-dir "$R/probes/admission/CASE/out" \
  "$R/probes/admission/CASE/package/src/main.rs" </dev/null
```

Backstop variants add `--tide-test-backstop` and use a separate `out_backstop` directory. The fixture probes use the archived `conformance/cases/errors/derive.rs`, crate name `fixture`, separate `probes/fixture-bin`/`probes/fixture-test` outputs, and `--crate-type=lib --test` for the test build.

| Probe | Source / modes | Observed result |
| --- | --- | --- |
| P8 | Exact archived acceptance fixture, binary and test harness | Both builds exit 0. Binary exits 0 with `ERROR_DERIVE_RAN`; harness exits 0 with one test passed. Each build emits 13 redundant named-pattern warnings. |
| P9 | `pure_pattern`: derived `Bounded<T: Pure>` enum, generic variant pattern, instantiated with `root.net`; ordinary and backstop-only obligations | Both exit 1 with FX0517; neither leaves an executable. This reproduces the committed negative test's program. |
| P10 | `handwritten_pattern`: same generic enum/pattern and Net instantiation, without the derives | Both modes exit 1 with FX0517; neither leaves an executable. |
| P11 | `denied_expr`: `Path::new("/").exists()` inside an error message's expression argument | Exits 1 with FX0520 at the attribute argument; no executable. |
| P12 | `debug_generic`: `#[error("{0:?}")] struct DebugAddr<T>(T)`, formatted with `T = fn()` | Both ordinary and backstop-only obligations exit 1 with FX0520 for the function-pointer Debug impl; neither leaves an executable. The latter diagnostic explicitly names the running Debug instance. |
| P13 | `positive_generic`: generic named enum variant, LowerHex interpolation, construction and a handwritten generic pattern | Builds and executes assertions successfully, exit 0. |

## Blocking

### B1. Field shorthand fails after expression-introducing keywords

**Location:** `crates/tide-macros/src/error.rs:120` (`rewrite_fields`).

The scanner recognizes expression starts only after punctuation. It misses keywords such as `if`, `match`, and `return`. A valid thiserror expression therefore remains invalid Rust:

```rust
#[derive(Debug, Error)]
#[error("{}", if .0.is_empty() { "empty" } else { "full" })]
struct Conditional(String);
```

`Conditional("x".into()).to_string()` must produce `"full"`. P1 confirms thiserror compiles and executes that assertion; Tide reports “expected an expression” at `.0`. This is within the documented arbitrary-expression/field-shorthand behavior and the accepted interpolation surface.

**Smallest fix:** recognize Rust's actual expression-start contexts, including the relevant keywords, while preserving ordinary member accesses and turbofish boundaries. Upstream `attr.rs::parse_token_expr` is a useful concrete reference. Add this executed regression to the shared fixture; add another context only if it exercises a distinct scanner boundary.

### B2. Enum-level attributes have no independent parse/validation owner

**Location:** `crates/tide-macros/src/error.rs:377`, especially line 385.

The enum's attributes are parsed separately against every variant's fields, including variants that override the default, because `Option::or` evaluates its argument eagerly. This rejects a valid default-plus-override enum:

```rust
#[derive(Debug, Error)]
#[error("{}", .0)]
enum E {
    A(u8),
    #[error("other")]
    B,
}
```

P2 confirms thiserror executes `A(1) -> "1"` and `B -> "other"`; Tide fails while parsing `.0` against B's empty fields, even though B never uses that default.

The same ownership gap has the opposite effect for an empty enum: the iterator executes no times, so **none of its attributes are validated**. P3 confirms Tide accepts `#[derive(Debug, Error)] #[error(12)] enum E {}`, while thiserror rejects the malformed helper. Misplaced `#[source]`/`#[from]` and duplicate enum helpers likewise have no validation path there by source inspection.

**Smallest fix:** parse and validate enum-level helper syntax once independently of variant fields, then perform field rewriting for the selected default/override only. Retain normal validation for empty enums. Merely replacing `or` with a lazy choice would leave the empty-enum defect and must not suppress malformed common attributes. Add one executed inherited/overridden default case and one empty-enum rejection test at the attribute's line.

### B3. Every named-field derive emits avoidable warnings that can become errors

**Location:** `crates/tide-macros/src/error.rs:170` (`pattern`), emitted in both Display and source implementations.

Named fields are always emitted as `{ value: value }` rather than `{ value }`, preserving user spans. A normal program using the allowed `#[deny]` attribute then fails:

```rust
#![deny(warnings)]
#[derive(Debug, Error)]
#[error("{value}")]
struct Named { value: u8 }
```

P4 confirms Tide fails at the field for `non_shorthand_field_patterns`; thiserror compiles and executes the same formatting assertion. This is a compile failure caused by generated code, not a style preference. P8 also shows 13 such warnings in the acceptance fixture itself.

**Smallest fix:** emit shorthand named-field patterns, since the generated binding already has the field's identifier. Keep the existing tuple/unit branches. Protect an ordinary named-field example with `deny(warnings)` rather than globally suppressing user warnings.

### B4. Positional format indices are compared as strings instead of numbers

**Location:** `crates/tide-macros/src/error.rs:306`.

Rust and thiserror interpret `{00}` as positional argument zero. The lookup compares it to `0u32.to_string()`, misses the field, and forwards the placeholder to `write!` without supplying an argument:

```rust
#[derive(Debug, Error)]
#[error("{00}")]
struct E(u8);
```

P5 confirms thiserror executes `E(7).to_string() == "7"`; Tide fails compilation. This is an uncommon spelling, but it is a confirmed mismatch in the accepted positional-interpolation form.

**Smallest fix:** parse decimal positional keys to their numeric index before comparing with `Member::Unnamed`; preserve Rust's diagnostics for invalid/overflowing keys. Add one executed leading-zero index assertion to the positional fixture.

## Non-blocking

### N1. Missing unit-variant messages point at the derive rather than the variant

**Location:** `crates/tide-macros/src/error.rs:488`.

The error is attached to `Fields::Unit`, which has no tokens. P6 demonstrates that a missing attribute on `Two` points at the top-level derive instead. Carry the variant identifier/span alongside its fields and attach this diagnostic to the missing variant. The rejection itself is correct, so this does not independently block approval.

### N2. Correct the dependency license record

**Location:** `impl-report.md`, Decision 1 / AP-DEPENDENCY-001 record.

The “four host-only crates, MIT/Apache-2.0” summary omits a license term: cached `unicode-ident-1.0.27/Cargo.toml:36` declares `(MIT OR Apache-2.0) AND Unicode-3.0`. Correct that factual record when the dependency decision is finalized. I found no project rule making this dependency unacceptable; this is not a legal assessment.

## Decisions

1. **Parsing dependencies — agree with syn/quote/proc-macro2, subject to the fixes above.** Their capability is Rust item/expression/generic parsing and token construction. The standard library provides token trees but no equivalent Rust AST parser. Extending the existing small handwritten parser would move substantially more Rust grammar into Tide; vendoring thiserror would bring a larger implementation and a private-support/path maintenance obligation. A direct thiserror re-export does not solve its generated `::thiserror` paths under the current injected-std/extern boundary. The exact host-only pins and lockfiles are present, MSRV fits the pinned compiler, and replacement remains local to the macro crate. Correct N2. The policy scanner bugs are reasons to strengthen its small boundary tests, not reasons to abandon the parser dependency.
2. **Transparent source — agree.** Delegating to the wrapped error's own `source()` matches upstream expansion. Returning the wrapper's field itself would add an extra link and change leaf behavior. P8 executes both leaf and non-leaf forwarding.
3. **Source representation — agree for the accepted forms.** Method resolution through AsDynError, including Option handling, supports sized and boxed dynamic errors without a fragile direct cast. The helper corresponds to upstream's runtime mechanism, and P8 executes the optional, concrete, generic and boxed cases. No additional representation is needed for this bead.
4. **Production/conformance wiring — agree.** `root.rs` owns the derive export and includes the runtime support once; conformance removes only the old Error derive import and keeps its still-used test attributes. The shared fixture is admitted and executed against production std (P8). There is no second runtime implementation to reconcile.
5. **Generic admission recovery — agree; retain the shared fix.** `path_args` only replaces insufficient arguments for a struct/variant/constructor, using that expression/pattern's compiler-resolved ADT arguments. It does not filter clauses, change the admission flag, bypass `named`, or add a generated-code exemption. Both call sites still run `use_def`, `use_generic(..., true)`, and `named(args.types())`. ADT and constructor parameters come from the same definition hierarchy; source code cannot supply arbitrary TypeckResults. Avoiding enum patterns would hide the driver defect, and skipping obligations would be unacceptable. P9/P10 reject derived and handwritten non-Pure uses, P11 rejects an ambient operation carried through an attribute expression, P12 rejects a denied generic formatting impl even through the backstop, and P13 accepts the corresponding ordinary generic construction/pattern. These are concrete checks, not a claim of exhaustive soundness proof.
6. **Test artifact building — agree.** Adding host dependencies makes the old bare-rustc macro build incomplete in both test owners. Cargo resolves that trusted macro package, and explicit `--emit=link=...` avoids reconstructing Cargo's changing dependency/artifact layout. The consumer std and driver boundaries remain separate. The `.so` spelling is consistent with this slice's documented supported Linux host. No new package-side Cargo invocation enters production admission. The supplied workspace log covers the changed entry tests; I did not rerun their Cargo builds.
7. **Std error scope — agree.** The bead comment assigns future errors to their owning beads, and existing io::Error, IntoInnerError, and TaskError already implement Display/Error. P8 executes existing io/TaskError behavior. Adding planned types here would expand the chosen scope.

These are recommendations for the designated Opus decision-maker, not a substitute for that decision.

## Reasoning

The architecture keeps the intended owners: Tide's macro owns emitted policy, syn owns Rust parsing, production std owns runtime support, and the admission driver checks user-selected obligations. The one-line dyn-Error policy addition mirrors the existing dyn-Any method family under the pinned core API; it does not authorize new ambient operations. I found no need for a new allowlist exception for macro output. In particular, generated Display/source/From implementations are not broadly treated as trusted entry-point code.

The acceptance fixture provides useful execution evidence for ordinary named/tuple/unit errors, enum variants, generics, format specs, explicit arguments, From/`?`, marked/conventional sources, and transparent forwarding. My P8 run confirms its behavior. Its successful assertions do not establish the stronger compatibility claim: P1–P5 expose untested parser, inheritance, validation, and generated-code boundaries. Fixing B1–B4 and adding their small regressions is the minimum patch for approval; no architecture rewrite or expanded std-error feature is required.

The locks are consistent with their dependency edges. The retained oracle macro package still supplies test/sim_test attributes, so removing its Error import does not orphan that package. The entry.rs builder change is necessary for the new dependencies rather than unrelated drift. I saw no held-out E1 task code in the diff, and read no held-out task sources to construct these probes; source provenance beyond that is the author's recorded claim.

Backtrace/provide, the separate `error(fmt = ...)` facility, wider FX0102 migration enforcement, and future std errors were not promoted into extra acceptance requirements here. This pass identifies no later-tier-text-only finding for fx-djp.

Not checked: clean Cargo rebuilds, a fresh full workspace/conformance run, cross-platform behavior, or the held-out E1 gate. Runtime/admission evidence uses copied prebuilt artifacts; compatibility failures B1–B4 remain the merge risk until fixed. DONE
