//! Sink and unbounded-channel diagnostics (SINK-8, LOCK-4, reference 7.3).

use rustc_session::config::CrateType;

use super::*;

impl<'a, 'tcx> Walk<'a, 'tcx> {
    fn library_target(&self) -> bool {
        // A test harness is executable even when the source target is a library.
        !self.tcx.crate_types().contains(&CrateType::Executable)
    }

    pub(super) fn sink_expansion(&mut self, mac: DefId, span: Span) {
        if !self.crates.is_std(mac) && !self.crates.exported(mac) {
            return;
        }
        match self.tcx.item_name(mac).as_str() {
            "dbg" if !self.tcx.sess.opts.debug_assertions => self.flag(
                span,
                Code::Td0507,
                "`dbg!` is not allowed in release builds".into(),
            ),
            "print" | "println" if self.library_target() => self.flag(
                span,
                Code::Td0508,
                "printing to stdout from a library; use a log sink".into(),
            ),
            _ => {}
        }
    }

    pub(super) fn lint_sink_expr(&mut self, e: &'tcx hir::Expr<'tcx>, t: &ty::TypeckResults<'tcx>) {
        let (def, args) = match e.kind {
            _ if t.expr_ty_adjusted_opt(e).is_some_and(|ty| ty.is_fn_ptr()) => {
                // Reification preserves the constructor's identity at the coercion site.
                let Some(ty) = t.node_type_opt(e.hir_id) else {
                    return;
                };
                let ty::FnDef(def, _) = *ty.kind() else {
                    return;
                };
                (def, &[][..])
            }
            hir::ExprKind::Call(callee, args) => {
                let Some(ty) = t.expr_ty_adjusted_opt(callee) else {
                    return;
                };
                let ty::FnDef(def, _) = *ty.kind() else {
                    return;
                };
                (def, args)
            }
            hir::ExprKind::MethodCall(_, _, args, _) => {
                let Some(def) = t.type_dependent_def_id(e.hir_id) else {
                    return;
                };
                (def, args)
            }
            _ => return,
        };
        let key = self.key(def);
        if key == "tide_std::sync::mpsc::channel" && self.library_target() {
            self.flag(
                e.span,
                Code::Td0608,
                "an unbounded channel in a library; use `sync_channel` for backpressure".into(),
            );
        }
        if !matches!(
            key.as_str(),
            "tide_std::sync::Shared::<T>::with"
                | "tide_std::sync::Shared::<T>::with2"
                | "tide_std::sync::Shared::<T>::with_all"
        ) {
            return;
        }
        let Some(f) = args.last() else { return };
        let Some(mut ty) = t.expr_ty_adjusted_opt(f) else {
            return;
        };
        loop {
            ty = match *ty.kind() {
                ty::Ref(_, inner, _) => inner,
                ty::Adt(def, args) if def.is_box() => args.type_at(0),
                _ => break,
            };
        }
        let (ty::Closure(def, _) | ty::FnDef(def, _)) = *ty.kind() else {
            return;
        };
        let Some(def) = def.as_local() else { return };
        let Some(body) = self.tcx.hir_maybe_body_owned_by(def) else {
            return;
        };
        LockPrints { walk: self }.visit_body(body);
    }
}

/// Inspect f and its nested closures, excluding nested fn items. This lint does not infer
/// transitive effects of arbitrary callees.
struct LockPrints<'w, 'a, 'tcx> {
    walk: &'w mut Walk<'a, 'tcx>,
}

impl<'tcx> Visitor<'tcx> for LockPrints<'_, '_, 'tcx> {
    type NestedFilter = nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.walk.tcx
    }

    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        let mut span = e.span;
        while span.from_expansion() {
            let data = span.ctxt().outer_expn_data();
            if let Some(mac) = data.macro_def_id
                && (self.walk.crates.is_std(mac) || self.walk.crates.exported(mac))
                && matches!(
                    self.walk.tcx.item_name(mac).as_str(),
                    "print" | "println" | "eprint" | "eprintln" | "dbg"
                )
            {
                self.walk.flag(
                    data.call_site,
                    Code::Td0607,
                    "printing inside a `Shared::with` closure; use a log sink".into(),
                );
            }
            span = data.call_site;
        }
        intravisit::walk_expr(self, e);
    }
}
