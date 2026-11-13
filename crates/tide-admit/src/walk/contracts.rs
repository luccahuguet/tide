//! Contract placement and the signature boundary for mutable postcondition parameters.
use std::ops::ControlFlow;

use rustc_infer::infer::outlives::env::OutlivesEnvironment;
use rustc_middle::ty::{TypeSuperVisitable, TypeVisitable, TypeVisitor};
use rustc_trait_selection::regions::OutlivesEnvironmentBuildExt;

use super::*;

struct ContainsRegion<'tcx> {
    tcx: TyCtxt<'tcx>,
    region: ty::Region<'tcx>,
}
impl<'tcx> TypeVisitor<TyCtxt<'tcx>> for ContainsRegion<'tcx> {
    type Result = ControlFlow<()>;
    fn visit_region(&mut self, region: ty::Region<'tcx>) -> Self::Result {
        if region == self.region {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
    fn visit_ty(&mut self, value: Ty<'tcx>) -> Self::Result {
        if let ty::Alias(_, alias) = value.kind()
            && let ty::Opaque { def_id } = alias.kind
        {
            // Opaque args include uncaptured parent lifetimes; rustc marks those bivariant.
            for (arg, variance) in alias.args.iter().zip(self.tcx.variances_of(def_id)) {
                if *variance != ty::Bivariant {
                    arg.visit_with(self)?;
                }
            }
            ControlFlow::Continue(())
        } else {
            value.super_visit_with(self)
        }
    }
}
fn contains<'tcx>(
    tcx: TyCtxt<'tcx>,
    value: impl TypeVisitable<TyCtxt<'tcx>>,
    region: ty::Region<'tcx>,
) -> bool {
    value
        .visit_with(&mut ContainsRegion { tcx, region })
        .is_break()
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    fn trait_method(&self, def: LocalDefId) -> bool {
        let parent = self.tcx.local_parent(def);
        matches!(self.tcx.def_kind(parent), DefKind::Trait)
            || matches!(self.tcx.def_kind(parent), DefKind::Impl { of_trait: true })
    }

    pub(super) fn contract_placement(&mut self, def: LocalDefId) {
        if self.trait_method(def)
            && ["requires", "ensures"]
                .iter()
                .any(|name| std_attr_made(self.tcx, self.crates, def, name))
        {
            self.report(
                self.tcx.def_span(def).source_callsite(),
                Code::Fx0802,
                "requires and ensures belong on free functions or inherent methods".into(),
            );
        }
    }

    pub(super) fn contract_function(&mut self, def: LocalDefId, body: hir::BodyId) {
        if !matches!(self.tcx.def_kind(def), DefKind::Fn | DefKind::AssocFn) {
            return;
        }
        let body = self.tcx.hir_body(body);
        struct Clauses<'w, 'a, 'tcx> {
            walk: &'w mut Walk<'a, 'tcx>,
            uses: Vec<(HirId, Span)>,
            trait_method: bool,
            post: bool,
            post_span: Option<Span>,
        }
        impl<'tcx> Visitor<'tcx> for Clauses<'_, '_, 'tcx> {
            type NestedFilter = nested_filter::All;
            fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
                self.walk.tcx
            }
            fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
                if let hir::ExprKind::Call(callee, args) = expr.kind
                    && let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = callee.kind
                    && let Res::Def(_, helper) = path.res
                    && expr.span.from_expansion()
                {
                    let key = self.walk.crates.key(self.walk.tcx, helper);
                    if matches!(
                        key.as_str(),
                        "tide_std::__contract::requires"
                            | "tide_std::__contract::ensures"
                            | "tide_std::__contract::const_entry"
                            | "tide_std::__contract::entry"
                    ) {
                        let entry = key.ends_with("::entry") || key.ends_with("::const_entry");
                        if self.trait_method && !entry {
                            self.walk.report(
                                expr.span.source_callsite(),
                                Code::Fx0802,
                                "requires and ensures belong on free functions or inherent methods"
                                    .into(),
                            );
                        }
                        let outer = (self.post, self.post_span);
                        self.post_span = entry.then_some(expr.span.source_callsite());
                        self.post = key.ends_with("::ensures") || entry;
                        // old snapshots are hoisted before this closure by the contract macro.
                        if let Some(clause) = args.last() {
                            self.visit_expr(clause);
                        }
                        (self.post, self.post_span) = outer;
                        return;
                    }
                }
                intravisit::walk_expr(self, expr);
            }
            fn visit_path(&mut self, path: &hir::Path<'tcx>, _: HirId) {
                if self.post
                    && let Res::Local(id) = path.res
                {
                    self.uses.push((
                        id,
                        self.post_span
                            .unwrap_or_else(|| path.span.source_callsite()),
                    ));
                }
            }
            fn visit_item(&mut self, _: &'tcx hir::Item<'tcx>) {}
        }
        let trait_method = self.trait_method(def);
        let mut clauses = Clauses {
            walk: self,
            uses: Vec::new(),
            trait_method,
            post: false,
            post_span: None,
        };
        clauses.visit_body(body);
        let uses = clauses.uses;
        if uses.is_empty() {
            return;
        }
        let signature = self.tcx.liberate_late_bound_regions(
            def.to_def_id(),
            self.tcx
                .fn_sig(def)
                .instantiate_identity()
                .skip_normalization(),
        );
        let candidates: Vec<_> = signature
            .inputs()
            .iter()
            .enumerate()
            .filter_map(|(i, input)| {
                let ty::Ref(region, _, hir::Mutability::Mut) = input.kind() else {
                    return None;
                };
                if !region.is_free() {
                    return None;
                }
                let mut spans = Vec::new();
                body.params[i].pat.each_binding(|_, id, _, _| {
                    let typeck = self.tcx.typeck(def);
                    let Some(binding) = typeck.node_type_opt(id) else {
                        return;
                    };
                    let borrowed = matches!(
                        typeck.pat_binding_modes().get(id),
                        Some(hir::BindingMode(hir::ByRef::Yes(..), _))
                    );
                    // Copied scalars/shared references do not retain the argument's mutable borrow.
                    if matches!(binding.kind(), ty::Ref(_, _, hir::Mutability::Mut))
                        || borrowed
                            && binding
                                .walk()
                                .filter_map(|arg| arg.as_type())
                                .any(|ty| matches!(ty.kind(), ty::Ref(_, _, hir::Mutability::Mut)))
                    {
                        spans.extend(uses.iter().filter(|u| u.0 == id).map(|u| u.1));
                    }
                });
                (!spans.is_empty()).then_some((i, *region, spans))
            })
            .collect();
        if candidates.is_empty() {
            return;
        }
        let env = TypingEnv::non_body_analysis(self.tcx, def);
        let (infcx, param_env) = self.tcx.infer_ctxt().build_with_typing_env(env);
        let outlives =
            OutlivesEnvironment::new(&infcx, def, param_env, signature.inputs_and_output.iter());
        for (i, region, spans) in candidates {
            let escapes = signature
                .inputs()
                .iter()
                .enumerate()
                .any(|(j, other)| i != j && contains(self.tcx, *other, region))
                || contains(self.tcx, signature.output(), region)
                || self
                    .tcx
                    .clauses_of(def)
                    .instantiate_identity(self.tcx)
                    .clauses
                    .iter()
                    .any(
                        |predicate| match predicate.skip_normalization().kind().skip_binder() {
                            ty::ClauseKind::RegionOutlives(ty::OutlivesClause(left, _)) => {
                                left == region
                            }
                            ty::ClauseKind::TypeOutlives(ty::OutlivesClause(left, _)) => {
                                contains(self.tcx, left, region)
                            }
                            other => contains(self.tcx, other, region),
                        },
                    )
                || outlives.free_region_map().elements().any(|other| {
                    other != region
                        && outlives
                            .free_region_map()
                            .sub_free_regions(self.tcx, other, region)
                });
            if escapes {
                for span in spans {
                    self.report(span, Code::Fx0803,
                        "this mutable parameter can escape the call; name it only inside `old(..)` in ensures".into());
                }
            }
        }
    }
}
