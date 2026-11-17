//! CLOSURE-3 (`TD0206`, 02 §5.4): a function item coerced to a `fn` pointer, under a syntactic
//! conservative rule. A `fn` pointer type does not carry the outlives bounds that the item's
//! body may assume, and rustc does not check them at the coercion (rust-lang/rust#25860), so the
//! coercion is rejected whenever the item implies an outlives bound that does not hold
//! trivially, whatever the instantiation. A closure that calls the item is accepted: rustc checks
//! the call's argument types, though not bounds an impl header implies (rust-lang/rust#100051).

use rustc_middle::traits::query::OutlivesBound;
use rustc_trait_selection::traits::outlives_bounds::InferCtxtExt as _;

use super::*;

const HELP: &str = "call the function from a closure instead, such as `|x, y| f(x, y)`: a call \
                    checks its argument types against the signature, though not the bounds an \
                    impl header implies";

impl<'a, 'tcx> Walk<'a, 'tcx> {
    /// `e`, of type `source`, is reified to a `fn` pointer in `owner`'s body.
    pub(super) fn reify(&mut self, e: &hir::Expr<'tcx>, source: Ty<'tcx>, owner: LocalDefId) {
        let ty::FnDef(def_id, args) = *source.kind() else {
            return;
        };
        if self.generated_by_toolchain(e.span) {
            return;
        }
        let tcx = self.tcx;
        let name = tcx.item_name(def_id);
        // A trait method runs the selected impl's body, which also assumes that impl's header.
        let mut items = vec![def_id];
        // A body with type errors is rejected already, and selection there is unreliable.
        if let Some(trait_id) = tcx.trait_of_assoc(def_id)
            && self.can_select()
        {
            let args = tcx.instantiate_bound_regions_with_erased(args);
            match selected_impl(tcx, owner, trait_id, def_id, args) {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(()) => {
                    let message = format!(
                        "`{name}` cannot become a `fn` pointer here: the impl it runs is not \
                         known, so the bounds that impl implies cannot be checked (CLOSURE-3)"
                    );
                    self.td0206(e.span, message);
                    return;
                }
            }
        }
        if let Some(bound) = items
            .iter()
            .find_map(|&item| implied_bound(tcx, item, owner))
        {
            let message = format!(
                "`{name}` cannot become a `fn` pointer: it implies `{bound}`, which a `fn` \
                 pointer does not keep (CLOSURE-3)"
            );
            self.td0206(e.span, message);
        }
    }

    fn td0206(&mut self, span: Span, message: String) {
        self.diags.report(
            self.tcx,
            span.source_callsite(),
            Code::Td0206,
            message,
            Some(HELP),
        );
    }
}

/// The item whose implied bounds the trait method `method` at `args` adds when called from
/// `owner`: the selected impl's method, or the impl itself when the method is the trait's
/// default. `None` for a builtin impl, which has no header; an error when selection does not
/// find one impl, as for a method of a type parameter's bound.
fn selected_impl<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    trait_id: DefId,
    method: DefId,
    args: GenericArgsRef<'tcx>,
) -> Result<Option<DefId>, ()> {
    let env = TypingEnv::non_body_analysis(tcx, owner.to_def_id());
    let args = tcx
        .try_normalize_erasing_regions(env, Unnormalized::new(args))
        .map_err(|_| ())?;
    match select(tcx, env, ty::TraitRef::from_assoc(tcx, trait_id, args)) {
        Some(Selected::Impl(impl_id, _)) => Ok(Some(
            tcx.associated_items(impl_id)
                .in_definition_order()
                .find(|item| item.trait_item_def_id() == Some(method))
                .map_or(impl_id, |item| item.def_id),
        )),
        Some(Selected::Builtin(_)) => Ok(None),
        None => Err(()),
    }
}

/// The first outlives bound that `def_id` implies, other than one that always holds, under its
/// identity parameters. `def_id` is a function, whose body assumes its argument and return types
/// and, for a method, its impl header, or an impl, whose header alone counts. A lifetime outlives
/// itself, `'static` outlives every lifetime, a bound on a concrete type reduces to these, and a
/// type parameter declared `'static` outlives every lifetime too. rustc computes the bounds
/// (`body` scopes its normalization).
fn implied_bound<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId, body: LocalDefId) -> Option<String> {
    let header = |imp: DefId, tys: &mut Vec<Ty<'tcx>>| {
        let DefKind::Impl { of_trait } = tcx.def_kind(imp) else {
            return;
        };
        tys.push(tcx.type_of(imp).instantiate_identity().skip_normalization());
        if of_trait {
            let tr = tcx
                .impl_trait_ref(imp)
                .instantiate_identity()
                .skip_normalization();
            tys.extend(tr.args.types());
        }
    };
    let mut tys = Vec::new();
    if matches!(tcx.def_kind(def_id), DefKind::Impl { .. }) {
        header(def_id, &mut tys);
    } else {
        let sig = tcx.liberate_late_bound_regions(
            def_id,
            tcx.fn_sig(def_id)
                .instantiate_identity()
                .skip_normalization(),
        );
        tys.extend(sig.inputs_and_output);
        if let Some(parent) = tcx.opt_parent(def_id) {
            header(parent, &mut tys);
        }
    }
    let env = TypingEnv::non_body_analysis(tcx, def_id);
    let (infcx, param_env) = tcx.infer_ctxt().build_with_typing_env(env);
    let is_static = |param: ty::ParamTy| {
        param_env.caller_bounds().iter().any(|clause| {
            matches!(
                clause.kind().skip_binder(),
                ty::ClauseKind::TypeOutlives(ty::OutlivesClause(t, r))
                    if r.is_static() && t == param.to_ty(tcx)
            )
        })
    };
    infcx
        .implied_bounds_tys(body, param_env, tys, false)
        .find_map(|bound| match bound {
            OutlivesBound::RegionSubRegion(short, long) => {
                (short != long && !long.is_static()).then(|| format!("{long}: {short}"))
            }
            OutlivesBound::RegionSubParam(region, param) => {
                (!is_static(param)).then(|| format!("{param}: {region}"))
            }
            OutlivesBound::RegionSubAlias(region, alias) => {
                Some(format!("{}: {region}", alias.to_ty(tcx, ty::IsRigid::No)))
            }
        })
}
