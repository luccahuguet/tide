Model: gpt-6.1-sol (high) · Date: 2026-11-16

Verdict: request_changes

## Scope

Read-only supervisor review of fx-iwjq at e87ffe040d4b5e22e920ffad7053fb34e1a99a99, based on main 3f3d417d2692f84a8fafb22b21237b9e4dfab2bf. `git diff main...HEAD` has SHA-256 0502db666a2a5e2f8cd05e2701d3bb2059741baace24cb9ad316014774c78c39, matching the brief. The worktree was clean before and after inspection; `git diff --check main...HEAD` passes.

Read AGENTS.md, GOVERNANCE.md's Merge Rule, `br --no-db show fx-iwjq`, its comments, the author's report, the complete diff, the surrounding admission walk and trait-selection code, the six CLOSURE-3 cases, the affected design sections, and the installed pinned nightly's relevant compiler source. No repository files were changed. This report is the only output written.

Contract reviewed: reject the chosen conservative class of function-item-to-pointer conversions at the conversion site, including external items; preserve a callable closure workaround; expose FX0206; carry the regressions; and record RUN-3's v0 placement and the interim limit on public safety claims. This pass does not implement RUN-3 or close unrelated rustc soundness holes.

Confidence: high in the structural findings below. Minimal programs are inspection-derived examples, not executed probes. Cargo, rustc, tide-admit, and conformance were not run, as instructed. Full-suite proof remains the coordinator's responsibility.

## Blocking findings

### B1 — Trait method pointers omit the selected impl's implied bounds

`crates/tide-admit/src/walk/reify.rs:18` and `:55`.

The check discards the `FnDef` arguments and analyzes only its definition. For a trait method expression, that definition belongs to the trait declaration, not to the selected implementation. Its parent is a trait, so the `DefKind::Impl` branch never executes. The existing `ceiling_call` machinery explicitly resolves this distinction (`walk/traits.rs:218`), but the new check does not.

Minimal conservative-rule counterexample:

```rust
struct Pair<'a, 'b>(&'a &'b ());
trait Select { fn select(value: u32) -> u32; }
impl<'a, 'b> Select for Pair<'a, 'b> {
    fn select(value: u32) -> u32 { value }
}
pub fn demo() {
    let _: fn(u32) -> u32 =
        <Pair<'static, 'static> as Select>::select;
}
```

`Select::select`'s signature is `fn(u32) -> u32`, which produces no outlives bounds. The implementation's self type implies `'b: 'a`, but is never queried. This conversion is therefore admitted. The corresponding inherent method is rejected by the new check, even at the same all-static instantiation. That difference contradicts the implementation report and CLOSURE-3's inclusion of the method body's impl-header assumptions (`design/02-memory.md:546`). The all-static example isolates the missing conservative rejection; it is not itself a memory-safety exploit.

Smallest repair: include the selected implementation's definition and its assumed header bounds when checking a trait method conversion, using the existing trait-selection mechanism. Explicitly handle a conversion through an unresolved generic trait bound rather than treating the absence of a selected implementation as proof of no implied bounds. Add a regression for this route. Inspect the implementation under its identity parameters, so the all-static instantiation does not accidentally relax the stated conservative policy. This finding concerns v0 enforcement and blocks under the Merge Rule.

### B2 — The required interim public safety qualification is absent from the README

`README.md:5`, `README.md:12`, and `design/01-design.md:580`.

D7 now requires public claims, until RUN-3 lands, to say that safety against a malicious dependency rests on rustc's soundness and that rustc has open I-unsound issues. The public README still says that code cannot overreach and that no code in any dependency can touch files, the network, or the clock without being handed authority. Its new FX0206 paragraph explains one closure of a hole but adds no qualification about the remaining holes or the missing sandbox.

Failure scenario: someone evaluating the implemented driver from the README is still promised containment of a malicious dependency, although D7 and RUN-4 expressly acknowledge that remaining compiler holes can forge capabilities. The design-phase label does not communicate this particular limitation of the present implementation.

Smallest repair: add the required rustc-soundness/open-I-unsound qualification beside the README's authority claim, explicitly saying that RUN-3 is pending. Keep the existing D7 decision and tier placement. This is within the bead's public-claim requirement and does not require implementing the sandbox.

## Non-blocking findings

### N1 — State the generic callback cost and the stronger-than-necessary rejection accurately

`crates/tide-admit/src/walk/reify.rs:75`; `design/02-memory.md:542`.

Covering type parameters is needed to cover the same underlying hole: substituting a reference for `T` converts `T: 'a` into a lifetime relation. The new type-parameter regression supplies the corresponding failure scenario. Unresolved associated types need the same treatment because their concrete types can contain references. Omitting both categories would leave the conservative rule incomplete.

However, the rule now rejects routine safe monomorphizations, not only the old `same` example:

```rust
fn first<T>(v: &[T]) -> &T { &v[0] }
pub fn demo() {
    let _: fn(&[u8]) -> &u8 = first::<u8>;
}
```

It even rejects a definition whose parameter is explicitly owned:

```rust
fn owned<T: 'static>(_: &T) {}
pub fn demo() {
    let _: fn(&u8) = owned::<u8>;
}
```

The unconditional `RegionSubParam` arm rejects both. In the second example, no legal instantiation breaks `T`'s outlives requirement: `T: 'static` already entails it. An associated type with an explicit `'static` bound has the analogous cost. The implementation does not prove the prose claim that some instantiation can break each rejected bound.

The design does state rejection of type-parameter bounds regardless of instantiation, so the first example follows its mechanical policy. It does not explicitly illustrate the common generic callback cost, and its introductory “some instantiation can break” description overstates the precision. The bead's original Decision still names lifetime parameters and only `same` as the cost. Record the broadened technical decision and its actual cost there, and describe the implementation as a syntactic conservative rejection unless it is changed to account for universally sufficient declared bounds. I do not require a precise solver to land this bead.

### N2 — Most coercion-route evidence is outside the committed corpus

`conformance/cases/memory/closure_fnptr_type_param_reject.rs:12` and the other CLOSURE-3 fixtures.

The committed negative cases all use explicit local pointer initializers and free functions. They do not protect `as`, an unannotated `if`/`match` least-upper-bound conversion, an external function, a constructor, an alias, or an impl-header method route. The author's scratch probes support some of these but do not remain as regression tests. A compact fixture covering distinct compiler adjustment paths would make the “every coercion” claim substantially easier to maintain. B1's regression is required independently of this suggestion.

The existing `closure_fnptr_reject.rs:5` comment still says the checker re-checks bounds at the coercion; it should describe the conservative rejection instead.

## Reasoning

### Conversion coverage

- **Ordinary implicit coercion and `as`:** the hook is at the right HIR/typeck boundary. In the pinned compiler, `coerce_from_fn_item` emits `ReifyFnPointer`; cast checking also calls coercion on the source expression. Tracking each adjustment's source type is correct, including additional safe-to-unsafe-pointer adjustments.
- **`if`/`match` common types:** the pinned `try_find_coercion_lub` records `ReifyFnPointer` on the contributing function-item expressions, so the visitor sees those conversions. If the expressions retain the same function-item type, no pointer conversion has occurred yet; a later conversion is checked.
- **Inherent methods:** the immediate impl-parent branch reads self type assumptions. Trait-ref type arguments are also collected if given an impl method definition. The defect is obtaining that definition for a normal trait method expression (B1).
- **Constructors:** function-like constructors have `FnDef` signatures and use the same adjustment. The implementation treats their argument/output types conservatively; there is no explicit constructor exemption.
- **Std/external free items:** the new check does not apply the allowlist's external-item exemptions. `fn_sig`, `param_env`, impl self type, and trait-ref metadata queries can operate on external definitions. Generic arguments are deliberately ignored, so `first::<u8>` still has the generic definition's rejection.
- **Noncapturing closure to pointer:** uses `ClosureFnPointer`, intentionally outside this hook. A closure calling `pick` is the explicit accepted workaround, and that positive case is in the corpus. This is a direct-call workaround, not a proof that every closure-based compiler soundness hole is closed.
- **`Fn` generics, `impl Fn`, and `dyn Fn`:** they do not require a function-pointer conversion. For example, the following lawful same-lifetime route remains admitted by FX0206:

```rust
fn pick<'a, 'b>(_: &'a &'b (), v: &'b u32) -> &'a u32 { v }
pub fn via_dyn<'a>(unit: &'a &'a (), x: &'a u32) -> &'a u32 {
    let f: &dyn Fn(&'a &'a (), &'a u32) -> &'a u32 = &pick;
    f(unit, x)
}
pub fn callable<'a>()
    -> impl Fn(&'a &'a (), &'a u32) -> &'a u32 + use<'a>
{
    pick
}
```

  Passing `pick` directly to a matching generic `F: Fn(...)` likewise avoids the hook. These examples do not forge a lifetime. The pinned Fn-trait candidate confirmation equates argument trait references; it does not simply reuse the function-pointer subtyping operation. I found no demonstrated #25860 exploit through these routes in this inspection, and do not claim that admitting them violates the bead's pointer-only scope. Other implied-bound/closure bugs must remain within the public caveat; upstream discusses the separate closure/opaque/type-identity mechanism in [rust#112905](https://github.com/rust-lang/rust/issues/112905).
- **Macros/toolchain spans:** ordinary macro expansions are traversed and are not exempt merely for being external. The skip recognizes compiler AST passes, core test descriptors, the actual Tide main expansion, and bytes inserted by the trusted synthesis pass. The inspected Tide main and test macros preserve user-body token spans. I found no concrete candidate-controlled coercion hidden by this exemption. The blanket exemption is not, by itself, evidence that arbitrary generated conversions satisfy CLOSURE-3.

### Pinned rustc API use and failure behavior

The installed source for nightly-2026-08-09 supports the APIs used here. `liberate_late_bound_regions` replaces the outer signature binder's regions with free late parameters, while identity instantiation keeps early parameters symbolic. This matches `assumed_wf_types` for ordinary functions. Nested binders stay nested; rustc's implied-bounds computation skips predicates with bound variables and escaping-alias components rather than blindly liberating them. I found no concrete escaping-bound-variable panic introduced by these calls.

`TypingEnv::non_body_analysis(tcx, def_id)` is appropriate for examining the callee's declared signature under its own predicates without revealing body-defined opaque types. The caller body ID supplied to `implied_bounds_tys` scopes normalization obligations and diagnostic spans; it does not substitute caller generic arguments for the callee's identity parameters. No concrete mismatch from that choice was found.

The impl-header reconstruction resembles `assumed_wf_types`, but it is not a complete reproduction of method resolution (B1). Borrowck also normalizes impl-header types before deriving their bounds, while this code feeds unnormalized types to the implied-bounds routine, which examines both normalized and unnormalized forms. That can be more restrictive; the report should not describe the two processes as identical.

The underlying `implied_outlives_bounds` helper returns an empty vector on `NoSolution` or response-instantiation failure, and filters unexpected placeholders. Thus “no reported bound” is not a fail-closed proof. Its assertion against non-region inference variables is compatible with identity-instantiated signatures. Error-containing items can still reach this hook through recovered typeck, but identity queries do not import the erroneous use's inference arguments. I found no concrete valid admitted input that produces a query failure or panic here. Malformed-input crash behavior remains unexecuted, not proven safe by this review.

### Design and conformance

D7 records the delegated RUN-3 decision, the pending fx-ip4x, deployed-binary enforcement remaining v1, and the requested interim public-claim policy. The v0 row already included the D13 test sandbox; narrowing the v1 row to deployed binaries reconciles that existing split. These changes do not move another feature between tiers. ORIGIN-6's replacement of “re-check” with “rule” is appropriate. RUN-4 continues to acknowledge other compiler bugs; its stronger statement that the pointer route is closed depends on completing B1. Public-facing application of the caveat is B2.

The five negative fixture verdicts follow the implemented conservative rule: original variance case, known surviving rustc case, capability case, formerly accepted `same`, and type-parameter case. Each has an FX0206 mark at the coercion and no rustc rejection mark. Removing FX0206 would therefore make their expected diagnostics fail, as the runner compares exact driven marks and rejection status. This is an inspection of the fixture/runner contract, not an independent reproduction of the author's passing run. The positive fixture's `add` and closure-wrapper examples correctly remain accepted; positive controls naturally continue passing if the check is removed.

Not checked: compilation, full suites, runtime exploits, and execution of the review examples. Risk: trait-impl header coverage and the public safety qualification require correction before approval.

DONE
