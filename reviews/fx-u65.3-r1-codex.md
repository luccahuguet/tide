Model: gpt-6.1-sol · Effort: high · Date: 2026-11-11
Verdict: request_changes

Confidence: high. The direction and ownership are appropriate, but three FX0407 defects block this candidate. Opus should wait for the fixes and this supervisor's re-check.

Reviewed HEAD b0ca910268aaec2409f055404646f77b96dcdac8 against d9a02c0; the diff SHA-256 matches d94a7e6d739f383f4738abdc0d2ee79180b272454d6ca48aac9f0f1ee81940f0. The checkout remained clean. Read AGENTS.md, GOVERNANCE.md, the implementation report, fx-u65.3 and its binding decisions, fx-u65/fx-u65.2, research/27, the relevant design/reference and D3/D4 decisions, conformance documentation, the complete diff, and connected walker, synthesis, emitter and test code. No held-out files were opened.

1. **Blocking — A projection is mistaken for constructor-wrapped growth.** Evidence: `crates/tide-admit/src/walk/traits.rs:194-196` and `:231-233`; research/27 C6 and the bead's binding decision require a caller parameter inside a constructor. The implementation instead treats every non-parameter type containing a parameter as growth, including associated-type projections.

   Reproducer:

   ```rust
   pub trait K { type A: K<A = Self::A>; }
   pub fn f<T: K>(n: u8) {
       if n > 0 { f::<T::A>(n - 1); }
   }
   pub struct S;
   impl K for S { type A = S; }
   pub fn entry() { f::<S>(2); }
   ```

   The equality makes projection idempotent: after the first call, the type stays fixed. No call wraps a caller parameter in a constructor. Nevertheless, the driver rejects the recursive call with FX0407, both on Rust's std and production Tide std. The pinned rustc separately links this exact source successfully. This is an admission false positive, not a missing supplementary label.

   Fix: distinguish actual constructor wrapping from a projection or other unresolved alias; normalize when the caller's constraints establish the result. Do not use “not Param” as the constructor test. Add this accepted regression alongside the existing rejected Vec/reference/closure growth cases.

2. **Blocking — Statically selectable local trait implementations disappear from the definition call graph.** Evidence: `crates/tide-admit/src/walk/traits.rs:167-181` records the type checker's trait-method DefId directly, while `:197-208` compares it with the caller implementation method's different DefId. No selected-implementation resolution connects them.

   Reproducer:

   ```rust
   pub struct S<T>(T);
   pub trait K { fn f(self); }
   impl<T> K for S<T> {
       fn f(self) { S(Some(self.0)).f(); }
   }
   ```

   Both the Rust-std and production-Tide-std driver checks exit 0 with only rustc's unconditional-recursion warning. The fully qualified call `<S<Option<T>> as K>::f(S(Some(self.0)))` also passes. The receiver is the known local ADT `S<Option<T>>`, with one selectable local implementation; this is not the binding decision's deferred trait-method call on a type parameter. Its constructor-wrapped recursive call must fail even when the generic method is uncalled. The instance fallback cannot help an uninstantiated definition.

   Fix: when local trait dispatch can be selected in the caller's environment, record the selected implementation method and its substituted implementation arguments. Preserve the documented fallback for genuinely unresolved parameter/dynamic dispatch. Add rejected dot-call and qualified-call coverage, plus an accepted same-type recursive trait method so the implementation's fixed `S<T>` receiver is not itself mistaken for growth.

3. **Blocking — The recursion-limit label falsely identifies finite concrete instantiation as FX0407.** Evidence: `crates/tide-admit/src/diag/labels.rs:140-145` uses `Site::None` with only rustc's message prefix. That does not prove the language's forbidden growth. The label policy at `conformance/README.md:89-92` and the binding bead comment explicitly require labels never to be false.

   A concrete reproducer is a finite chain: declare `trait K { fn step(); }` and `fn f<T: K>() { T::step(); }`; declare S0 through S131, each with a separate concrete K impl. S0's method is empty; Si's method calls `f::<S(i-1)>()`. A public entry calls `f::<S131>()`. There is no constructor-wrapped caller parameter, and the selected implementations form a finite chain ending at S0.

   With the default recursion limit, the driver emits rustc's instance-depth failure and adds FX0407. Adding only `#![recursion_limit="256"]` makes the exact finite chain pass. The failure and false label also reproduce with production Tide std. A nine-type version with `#![recursion_limit="4"]` is a smaller reproducer. The pinned compiler's `rustc_monomorphize/src/collector.rs:656-695` confirms the check merely counts repeated function DefIds and assumes excessive depth is infinite; it does not establish growth.

   Fix: require evidence of forbidden constructor growth before adding this supplementary label. Where that evidence is uncertain, omit FX0407 and preserve rustc's original error/rejection, as the precision policy permits. Add this finite-chain no-label regression while retaining coverage of actual hidden growth. An unconditional mapping from this resource-limit message cannot satisfy the never-false contract.

The remaining review found no additional blocker. FX0401 uses the decided exact-module policy; FX0406 matches D4's adopted bounded-type/subterm sentence. The cycle search terminates through its visited set. Error diagnostics bypass warning allowances. The table keeps rustc diagnostics and removes the old emitter fallback panic; E0119's mapping names actual overlap. The FX0210 secondary-span change stays paired with the recorded borrow. The insertion ranges account for preceding insertions and original byte positions; an extra production-std probe combining BOM, UTF-8, CRLF, two inline modules, inserted parameter metadata and generated impls passed, while its adjacent illegal user impl still produced FX0406. No dependency or broader architectural replacement is needed for these findings.

Proof: focused probes used the existing candidate driver and pinned rustc 1.99.0-nightly (771916f90), serially under nice -n 19 and CARGO_BUILD_JOBS=4, with closed stdin and 45-second per-process timeouts. Load, available memory and the quiet-window deadline were checked before each batch; load stayed below 12 and available memory above 6 GB. Reproducers and captured diagnostics are under `/home/lucca/.cache/eon-agents/tide-fx-u65.3-work/r1-probes/` (projection_idempotent_linkroot, local_trait_recursion, local_trait_recursion_ufcs, finite_default_chain, finite_default_chain_higher_limit, finite_instance_chain, provenance_crlf and provenance_crlf_reject). Three optional stub runs exited 2 because that stub's dependency directory was absent; they are not verdict evidence. Equivalent production-std probes succeeded in reproducing all three findings.

Minimal approval path: correct the growth predicate and resolvable trait-call targets, make the instance label certain-only, and add the focused regressions above. No unrelated refactor is requested.

Not checked: no full-workspace build or suite was run by this reviewer; rebased suite proof remains with the coordinator. Risk: the current tree both rejects legal definitions and admits forbidden uncalled recursion, so it is not Codex-clean.
DONE
