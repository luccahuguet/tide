//! Module syntax, rustc-owned import usage, import cycles, and unsafe documentation.
use rustc_ast::{self as ast, visit as ast_visit};
use rustc_data_structures::unord::{ExtendUnord, UnordSet};
use rustc_session::lint::{LintId, builtin::UNUSED_IMPORTS};
use rustc_span::Pos;

use super::*;

pub fn check_module_surface(
    tcx: TyCtxt<'_>,
    krate: &ast::Crate,
    diags: &mut Diagnostics,
    insertions: &crate::synthesis::Insertions,
) -> Vec<Span> {
    struct Surface<'a, 'tcx> {
        tcx: TyCtxt<'tcx>,
        diags: &'a mut Diagnostics,
    }
    impl<'ast> ast_visit::Visitor<'ast> for Surface<'_, '_> {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            if matches!(item.kind, ast::ItemKind::ExternCrate(..)) && !item.span.from_expansion() {
                self.diags.report(
                    self.tcx,
                    item.span,
                    Code::Fx0912,
                    "`extern crate` is not admitted; use an import".into(),
                    None,
                );
            }
            ast_visit::walk_item(self, item);
        }
    }
    ast_visit::Visitor::visit_crate(&mut Surface { tcx, diags }, krate);
    // A failed resolution cannot establish an unused import.
    if tcx.dcx().has_errors().is_some() {
        return Vec::new();
    }
    let mut unused = Vec::new();
    let resolver = tcx.resolver_for_lowering().0.borrow();
    let buffer = resolver.lint_buffer.borrow();
    for lint in buffer.map.values().flatten() {
        if lint.lint_id == LintId::of(UNUSED_IMPORTS)
            && let Some(span) = lint.span.as_ref()
        {
            for &span in span.primary_spans() {
                if !span.from_expansion() && !insertions.contains(tcx, span) {
                    unused.push(span);
                }
            }
        }
    }
    unused
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    pub(super) fn module_rules(&mut self, surface_errors: usize, unused: &[Span]) {
        // Collect edges before module findings so unused and cyclic imports can both report.
        let mut edges = Vec::new();
        for id in self.tcx.hir_crate_items(()).free_items() {
            let item = self.tcx.hir_item(id);
            if let hir::ItemKind::Use(path, kind) = item.kind
                && kind != hir::UseKind::ListStem
                && self.user_written(item.span)
                && !self.diags.has_error_in(item.span)
            {
                let source = self
                    .tcx
                    .parent_module_from_def_id(item.owner_id.def_id)
                    .to_local_def_id();
                for res in path.res.iter().flatten() {
                    if let Res::Def(kind, target) = res
                        && let Some(target) = target.as_local()
                    {
                        let target = if *kind == DefKind::Mod {
                            target
                        } else {
                            self.tcx.parent_module_from_def_id(target).to_local_def_id()
                        };
                        if source != target {
                            edges.push((source, target, item.span));
                        }
                    }
                }
            }
        }
        for &span in unused {
            let rejected = self.tcx.hir_crate_items(()).free_items().any(|id| {
                let item = self.tcx.hir_item(id);
                matches!(item.kind, hir::ItemKind::Use(_, kind) if kind != hir::UseKind::ListStem)
                    && item.span.contains(span)
                    && self.diags.has_error_in(item.span)
            });
            if !rejected {
                self.report(span, Code::Fx0913, "unused import; remove it".into());
            }
        }
        // Only late trait usage is incomplete after rustc errors, not our surface errors.
        if self.tcx.dcx().err_count() == surface_errors {
            let mut used = UnordSet::default();
            for def in self.tcx.hir_body_owners() {
                used.extend_unord(self.tcx.used_trait_imports(def).items().copied());
            }
            for &def in &self.tcx.resolutions(()).maybe_unused_trait_imports {
                let item = self.tcx.hir_expect_item(def);
                if !used.contains(&def)
                    && !self.tcx.visibility(def).is_public()
                    && self.user_written(item.span)
                    && !item.span.is_dummy()
                    && !self.diags.has_error_in(item.span)
                {
                    self.report(
                        item.expect_use().0.span,
                        Code::Fx0913,
                        "unused import; remove it".into(),
                    );
                }
            }
        }
        for &(source, target, span) in &edges {
            let mut pending = vec![target];
            let mut seen = FxHashSet::default();
            while let Some(module) = pending.pop() {
                if module == source {
                    self.report(
                        span,
                        Code::Fx0914,
                        "import cycle between modules; put shared items in a leaf module".into(),
                    );
                    break;
                }
                if seen.insert(module) {
                    pending.extend(edges.iter().filter(|e| e.0 == module).map(|e| e.1));
                }
            }
        }
    }

    pub(super) fn safety_comment(&mut self, block: &'tcx hir::Block<'tcx>) {
        if !self.user_written(block.span) {
            return;
        }
        let sm = self.tcx.sess.source_map();
        let documented = |span: Span| {
            let file = sm.lookup_source_file(span.lo());
            let Some(src) = &file.src else {
                return false;
            };
            let before = &src[..(span.lo() - file.start_pos).to_usize()];
            let mut lines = before.rsplit('\n');
            // Only preceding lines count, regardless of code before the block on its line.
            lines.next();
            lines
                .map(str::trim)
                .take_while(|line| line.starts_with("//"))
                .any(|line| {
                    line.strip_prefix("//")
                        .unwrap()
                        .trim_start()
                        .starts_with("SAFETY:")
                })
        };
        if documented(block.span) {
            return;
        }
        for (_, node) in self.tcx.hir_parent_iter(block.hir_id) {
            match node {
                hir::Node::Stmt(stmt) if documented(stmt.span) => return,
                hir::Node::Expr(expr) if documented(expr.span) => return,
                hir::Node::Item(item) if documented(item.span) => return,
                hir::Node::ImplItem(item) if documented(item.span) => return,
                hir::Node::TraitItem(item) if documented(item.span) => return,
                hir::Node::Block(_)
                | hir::Node::Item(_)
                | hir::Node::ImplItem(_)
                | hir::Node::TraitItem(_) => break,
                _ => (),
            }
        }
        self.report(
            block.span,
            Code::Fx0902,
            "an unsafe block needs a `// SAFETY:` comment directly above it or its statement"
                .into(),
        );
    }
}
