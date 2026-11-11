//! D3/D4's trait power ceiling; rustc owns coherence and instance failures.
use rustc_ast::{self as ast, visit as ast_visit};
use rustc_data_structures::fx::FxHashMap;

use super::*;

/// Surface syntax must be checked before lowering, including unstable forms rustc rejects.
pub fn check_trait_surface(tcx: TyCtxt<'_>, krate: &ast::Crate, diags: &mut Diagnostics) {
    struct Surface<'a, 'tcx> {
        tcx: TyCtxt<'tcx>,
        diags: &'a mut Diagnostics,
    }
    impl Surface<'_, '_> {
        fn report(&mut self, span: Span, code: Code, message: &str) {
            if !span.in_external_macro(self.tcx.sess.source_map()) {
                self.diags
                    .report(self.tcx, span, code, message.into(), None);
            }
        }
        fn specialization(&mut self, defaultness: ast::Defaultness) {
            if let ast::Defaultness::Default(span) = defaultness {
                self.report(
                    span,
                    Code::Fx0402,
                    "specialization is not admitted; use one impl or a newtype",
                );
            }
        }
    }
    impl<'ast> ast_visit::Visitor<'ast> for Surface<'_, '_> {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            match &item.kind {
                ast::ItemKind::Impl(imp) => {
                    if let Some(header) = &imp.of_trait {
                        self.specialization(header.defaultness);
                        if let ast::ImplPolarity::Negative(span) = header.polarity {
                            self.report(
                                span,
                                Code::Fx0405,
                                "negative impls are not admitted; use a marker field",
                            );
                        }
                    }
                }
                ast::ItemKind::Trait(tr) if matches!(tr.is_auto, ast::IsAuto::Yes) => {
                    self.report(
                        item.span,
                        Code::Fx0405,
                        "user auto traits are not admitted; use an ordinary trait",
                    );
                }
                _ => {}
            }
            ast_visit::walk_item(self, item);
        }
        fn visit_assoc_item(&mut self, item: &'ast ast::AssocItem, ctxt: ast_visit::AssocCtxt) {
            self.specialization(item.kind.defaultness());
            ast_visit::walk_assoc_item(self, item, ctxt);
        }
    }
    ast_visit::Visitor::visit_crate(&mut Surface { tcx, diags }, krate);
}

pub(super) struct Call {
    caller: (LocalDefId, u32),
    callee: (LocalDefId, u32),
    span: Span,
    grows: bool,
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    pub(super) fn ceiling_item(&mut self, item: &'tcx hir::Item<'tcx>) -> bool {
        if !self.user_written(item.span) {
            return true;
        }
        let mut valid_bounds = true;
        let def = item.owner_id.def_id;
        match item.kind {
            hir::ItemKind::Impl(imp) => {
                let module = self.tcx.parent_module_from_def_id(def);
                let self_ty = self
                    .tcx
                    .type_of(def)
                    .instantiate_identity()
                    .skip_normalization();
                let type_module = match self_ty.kind() {
                    ty::Adt(adt, _) if adt.did().is_local() => {
                        Some(self.tcx.parent_module_from_def_id(adt.did().expect_local()))
                    }
                    _ => None,
                };
                let trait_ref = imp.of_trait.map(|_| {
                    self.tcx
                        .impl_trait_ref(def)
                        .instantiate_identity()
                        .skip_normalization()
                });
                let trait_module = trait_ref
                    .and_then(|tr| tr.def_id.as_local())
                    .map(|tr| self.tcx.parent_module_from_def_id(tr));
                if Some(module) != type_module && Some(module) != trait_module {
                    let message = if type_module.is_none() && trait_module.is_none() {
                        "no module can hold this impl; implement a trait on the local type (for example, Into<u16> for Local instead of From<Local> for u16) or use a newtype"
                    } else {
                        "an impl must live in the exact module defining its type or trait; move the impl"
                    };
                    self.report(item.span, Code::Fx0401, message.into());
                }
                let heads: Vec<_> =
                    trait_ref.map_or_else(|| vec![self_ty], |tr| tr.args.types().collect());
                for (clause, span) in self.tcx.explicit_clauses_of(def).clauses.iter().copied() {
                    let bounded = match clause.kind().skip_binder() {
                        ty::ClauseKind::Trait(predicate) => predicate.self_ty(),
                        ty::ClauseKind::TypeOutlives(predicate) => predicate.0,
                        _ => continue,
                    };
                    if !bounded.references_error() && !small_bound(bounded, &heads) {
                        valid_bounds = false;
                        self.report(span, Code::Fx0406, "an impl bound must be on a parameter, its projection, a concrete type, or a strict subterm of the impl head; bound the parameter directly".into());
                    }
                }
            }
            hir::ItemKind::Struct(..) | hir::ItemKind::Enum(..) | hir::ItemKind::Union(..) => {
                let mut methods = Vec::new();
                for &imp in self.tcx.inherent_impls(def) {
                    for method in self.tcx.associated_items(imp).in_definition_order() {
                        if method.is_fn() && self.user_written(self.tcx.def_span(method.def_id)) {
                            methods.push((self.tcx.def_span(method.def_id), method.name()));
                        }
                    }
                }
                methods.sort_by_key(|(span, _)| span.lo());
                let mut names = FxHashSet::default();
                for (span, name) in methods {
                    if !names.insert(name) {
                        self.report(span, Code::Fx0408, format!("inherent method `{name}` is defined more than once for this type; use distinct names"));
                    }
                }
            }
            _ => {}
        }
        valid_bounds
    }

    pub(super) fn ceiling_associated_type(&mut self, generics: &hir::Generics<'_>, span: Span) {
        if !generics.params.is_empty() && self.user_written(span) {
            self.report(
                span,
                Code::Fx0403,
                "generic associated types are not admitted; use an owned item or index".into(),
            );
        }
    }

    pub(super) fn ceiling_parameter(&mut self, param: &hir::GenericParam<'_>) {
        if matches!(param.source, hir::GenericParamSource::Binder)
            && !param.is_elided_lifetime()
            && self.user_written(param.span)
        {
            self.report(param.span, Code::Fx0404, "explicit higher-ranked binders are not admitted; use elided Fn or fn-pointer lifetimes".into());
        }
    }

    pub(super) fn ceiling_call(
        &mut self,
        expr: &'tcx hir::Expr<'tcx>,
        typeck: &ty::TypeckResults<'tcx>,
    ) {
        if !self.user_written(expr.span) {
            return;
        }
        let callee = match expr.kind {
            hir::ExprKind::Call(fun, _) => match typeck.expr_ty_opt(fun).map(|t| t.kind()) {
                Some(ty::FnDef(def, args)) => {
                    Some((*def, self.tcx.instantiate_bound_regions_with_erased(*args)))
                }
                _ => None,
            },
            hir::ExprKind::MethodCall(..) => typeck
                .type_dependent_def_id(expr.hir_id)
                .map(|def| (def, typeck.node_args(expr.hir_id))),
            _ => None,
        };
        let Some((mut callee, mut args)) = callee else {
            return;
        };
        let Some(mut caller) = self.owner else { return };
        // A call in a closure still runs from the enclosing function's definition.
        while self.tcx.def_kind(caller) == DefKind::Closure {
            let Some(parent) = self.tcx.opt_local_parent(caller) else {
                return;
            };
            caller = parent;
        }
        if !matches!(self.tcx.def_kind(caller), DefKind::Fn | DefKind::AssocFn) {
            return;
        }
        let env = TypingEnv::non_body_analysis(self.tcx, caller.to_def_id());
        let Ok(normalized) = self
            .tcx
            .try_normalize_erasing_regions(env, Unnormalized::new(args))
        else {
            return;
        };
        args = normalized;
        if let Some(trait_id) = self.tcx.trait_of_assoc(callee) {
            let tr = ty::TraitRef::from_assoc(self.tcx, trait_id, args);
            let Some(Selected::Impl(impl_id, impl_args)) = select(self.tcx, env, tr) else {
                return;
            };
            if !impl_id.is_local() {
                return;
            }
            if let Some(method) = self
                .tcx
                .associated_items(impl_id)
                .in_definition_order()
                .find(|item| item.trait_item_def_id() == Some(callee))
            {
                args = args.rebase_onto(self.tcx, trait_id, impl_args);
                callee = method.def_id;
            }
        }
        let Some(callee) = callee.as_local() else {
            return;
        };
        for (index, arg) in args.iter().enumerate() {
            let Some(ty) = arg.as_type() else { continue };
            let grows = !matches!(ty.kind(), ty::Param(_));
            let mut parameters = FxHashSet::default();
            // Unresolved aliases do not prove that an input parameter survives in their output.
            let mut parts = ty.walk();
            while let Some(part) = parts.next() {
                match part.as_type().map(|t| t.kind()) {
                    Some(ty::Alias(..)) => parts.skip_current_subtree(),
                    Some(ty::Param(param)) if parameters.insert(param.index) => {
                        self.trait_calls.push(Call {
                            caller: (caller, param.index),
                            callee: (callee, index as u32),
                            span: expr.span,
                            grows,
                        })
                    }
                    _ => {}
                }
            }
        }
    }

    pub(super) fn ceiling_cycles(&mut self) {
        let mut nodes = FxHashMap::default();
        let mut edges = Vec::<Vec<usize>>::new();
        for call in &self.trait_calls {
            for node in [call.caller, call.callee] {
                let next = nodes.len();
                if let std::collections::hash_map::Entry::Vacant(entry) = nodes.entry(node) {
                    entry.insert(next);
                    edges.push(Vec::new());
                }
            }
            edges[nodes[&call.caller]].push(nodes[&call.callee]);
        }
        let components = strongly_connected_components(&edges);
        for call in std::mem::take(&mut self.trait_calls) {
            if call.grows && components[nodes[&call.caller]] == components[nodes[&call.callee]] {
                self.report(call.span, Code::Fx0407, "a type parameter expands under a constructor on a recursive cycle; use a fixed recursive representation".into());
            }
        }
    }
}

fn strongly_connected_components(edges: &[Vec<usize>]) -> Vec<usize> {
    let mut reverse = vec![Vec::new(); edges.len()];
    for (from, next) in edges.iter().enumerate() {
        for &to in next {
            reverse[to].push(from);
        }
    }
    // Kosaraju's two traversals use explicit stacks so graph depth cannot exhaust the call stack.
    let mut seen = vec![false; edges.len()];
    let mut order = Vec::with_capacity(edges.len());
    for root in 0..edges.len() {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut pending = vec![(root, 0)];
        while let Some((node, next)) = pending.last_mut() {
            if let Some(&to) = edges[*node].get(*next) {
                *next += 1;
                if !seen[to] {
                    seen[to] = true;
                    pending.push((to, 0));
                }
            } else {
                order.push(*node);
                pending.pop();
            }
        }
    }
    let mut components = vec![usize::MAX; edges.len()];
    for root in order.into_iter().rev() {
        if components[root] != usize::MAX {
            continue;
        }
        components[root] = root;
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            for &from in &reverse[node] {
                if components[from] == usize::MAX {
                    components[from] = root;
                    pending.push(from);
                }
            }
        }
    }
    components
}

fn has_type_parameter(ty: Ty<'_>) -> bool {
    ty.walk()
        .any(|arg| matches!(arg.as_type().map(|t| t.kind()), Some(ty::Param(_))))
}

fn parameter_projection(ty: Ty<'_>) -> bool {
    match ty.kind() {
        ty::Param(_) => true,
        ty::Alias(_, alias) => {
            alias.try_to_projection().is_some() && parameter_projection(alias.self_ty())
        }
        _ => false,
    }
}

fn small_bound<'tcx>(bounded: Ty<'tcx>, heads: &[Ty<'tcx>]) -> bool {
    parameter_projection(bounded)
        || !has_type_parameter(bounded)
        || heads.iter().any(|head| {
            head.walk()
                .skip(1)
                .any(|arg| arg.as_type() == Some(bounded))
        })
}

#[cfg(test)]
mod tests {
    use super::strongly_connected_components;

    #[test]
    fn components_match_reachability_and_handle_deep_graphs() {
        assert!(strongly_connected_components(&[]).is_empty());
        for mask in 0..512 {
            let mut edges = vec![vec![]; 3];
            let mut reachable = [[false; 3]; 3];
            for from in 0..3 {
                reachable[from][from] = true;
                for to in 0..3 {
                    if mask & (1 << (from * 3 + to)) != 0 {
                        edges[from].push(to);
                        reachable[from][to] = true;
                    }
                }
            }
            for via in 0..3 {
                for from in 0..3 {
                    for to in 0..3 {
                        reachable[from][to] |= reachable[from][via] && reachable[via][to];
                    }
                }
            }
            let components = strongly_connected_components(&edges);
            for from in 0..3 {
                for to in 0..3 {
                    assert_eq!(
                        components[from] == components[to],
                        reachable[from][to] && reachable[to][from],
                        "graph {mask}: {from} -> {to}"
                    );
                }
            }
        }
        let mut edges: Vec<_> = (1..20_000).map(|next| vec![next, next]).collect();
        edges.push(vec![]);
        let components = strongly_connected_components(&edges);
        assert_eq!(
            components
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            edges.len()
        );
        edges.last_mut().unwrap().push(0);
        let components = strongly_connected_components(&edges);
        assert!(
            components
                .iter()
                .all(|&component| component == components[0])
        );
    }
}
