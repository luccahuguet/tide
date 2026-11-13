//! Concurrency surface (D8) and explicit Result handling (D5).
use rustc_ast::{self as ast, visit as ast_visit};
use rustc_span::ExpnData;

use super::*;

/// Check written async syntax before lowering replaces it with lang-item uses.
pub fn check_habit_surface(tcx: TyCtxt<'_>, krate: &ast::Crate, diags: &mut Diagnostics) {
    struct Surface<'a, 'tcx> {
        tcx: TyCtxt<'tcx>,
        diags: &'a mut Diagnostics,
    }
    impl Surface<'_, '_> {
        fn asynchronous(&mut self, span: Span) {
            if !span.in_external_macro(self.tcx.sess.source_map()) {
                self.diags.report(
                    self.tcx,
                    span,
                    Code::Fx0601,
                    "async/await is not admitted; use ordinary functions and structured tasks"
                        .into(),
                    None,
                );
            }
        }
    }
    impl<'ast> ast_visit::Visitor<'ast> for Surface<'_, '_> {
        fn visit_fn(
            &mut self,
            kind: ast_visit::FnKind<'ast>,
            _: &ast::AttrVec,
            _: Span,
            _: ast::NodeId,
        ) {
            let coroutine = match kind {
                ast_visit::FnKind::Fn(_, _, f) => f.sig.header.coroutine_kind,
                ast_visit::FnKind::Closure(_, coroutine, ..) => *coroutine,
            };
            if let Some(
                coroutine
                @ (ast::CoroutineKind::Async { .. } | ast::CoroutineKind::AsyncGen { .. }),
            ) = coroutine
            {
                self.asynchronous(coroutine.span());
            }
            ast_visit::walk_fn(self, kind);
        }
        fn visit_expr(&mut self, expr: &'ast ast::Expr) {
            match expr.kind {
                ast::ExprKind::Gen(
                    _,
                    _,
                    ast::GenBlockKind::Async | ast::GenBlockKind::AsyncGen,
                    span,
                )
                | ast::ExprKind::Await(_, span) => self.asynchronous(span),
                _ => {}
            }
            ast_visit::walk_expr(self, expr);
        }
    }
    ast_visit::Visitor::visit_crate(&mut Surface { tcx, diags }, krate);
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    pub(super) fn habit_def(&self, def: DefId) -> Option<Rejection> {
        // Tokio's public spawn is re-exported from an internal module.
        if def.is_local()
            || self.tcx.crate_name(def.krate).as_str() != "tokio"
            || !self
                .tcx
                .opt_item_name(def)
                .is_some_and(|name| name.as_str() == "spawn")
        {
            return None;
        }
        let rule = policy::lookup("tokio::spawn").filter(|rule| rule.verdict == Verdict::Deny)?;
        Some(Rejection {
            key: self.key(def),
            code: rule.code,
            help: rule.help,
        })
    }

    pub(super) fn habit_expansion(&mut self, data: &ExpnData) -> bool {
        let ExpnKind::Macro(MacroKind::Bang, written_name) = data.kind else {
            return false;
        };
        let name = data
            .macro_def_id
            .map_or(written_name, |mac| self.tcx.item_name(mac));
        if (name.as_str().rsplit("::").next() != Some("select")
            && written_name.as_str().rsplit("::").next() != Some("select"))
            || !self.user_written(data.call_site)
        {
            return false;
        }
        if data.parent != ExpnId::root()
            && !super::surface::written_bang(self.tcx, data.call_site, written_name)
        {
            return false;
        }
        self.report(data.call_site, Code::Fx0605,
            "`select!` is not admitted; send one enum through one channel, or use deadlines and cancellation".into());
        true
    }

    pub(super) fn habit_stmt(&mut self, stmt: &'tcx hir::Stmt<'tcx>) {
        let (hir::StmtKind::Semi(expr) | hir::StmtKind::Expr(expr)) = stmt.kind else {
            return;
        };
        // A macro's returned value belongs to its calling statement, not its inner blocks.
        let span = stmt.span.source_callsite();
        let written = self.user_written(stmt.span)
            || matches!(self.tcx.parent_hir_node(stmt.hir_id), hir::Node::Block(block)
                if self.user_written(block.span));
        if written
            && self.user_written(span)
            && let Some(t) = self.recovered_typeck_of(expr.hir_id)
            && let Some(ty) = t.expr_ty_opt(expr)
            && let ty::Adt(adt, _) = ty.kind()
            && self.tcx.is_diagnostic_item(sym::Result, adt.did())
        {
            self.report(span, Code::Fx0702,
                "a Result cannot be ignored; handle it, propagate it with `?`, or explicitly discard it with `let _ =`".into());
        }
    }
}
