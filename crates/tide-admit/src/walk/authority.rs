//! Ambient world operations, entry-root use, test-only simulation, and handle creation.
use rustc_ast::attr::data_structures::CfgEntry;
use rustc_span::Symbol;

use super::*;

impl<'a, 'tcx> Walk<'a, 'tcx> {
    fn is_root(&self, ty: Ty<'tcx>) -> bool {
        matches!(ty.peel_refs().kind(), ty::Adt(adt, _) if self.key(adt.did()) == "tide_std::Root")
    }

    /// Test mode alone does not make ordinary production helpers test code.
    fn test_code(&self, span: Span) -> bool {
        if !self.tcx.sess.opts.test {
            return false;
        }
        if self.test_suite {
            return true;
        }
        let mut owner = match (self.owner, self.item) {
            (Some(body), Some(item))
                if self
                    .tcx
                    .is_descendant_of(item.to_def_id(), body.to_def_id()) =>
            {
                Some(item)
            }
            (body, item) => body.or(item),
        }
        .or_else(|| self.holder_at(span));
        while let Some(def) = owner {
            let id = self.tcx.local_def_id_to_hir_id(def);
            if self.tcx.hir_attrs(id).iter().any(|attr| {
                matches!(attr, hir::Attribute::Parsed(AttributeKind::CfgTrace(entries))
                    if entries.iter().any(|(entry, _)| requires_test(self.tcx, entry)))
            }) || self.test_attribute(def)
            {
                return true;
            }
            owner = self.tcx.opt_local_parent(def);
        }
        false
    }

    fn test_attribute(&self, def: LocalDefId) -> bool {
        let mut expn = self.tcx.expn_that_defined(def.to_def_id());
        while expn != ExpnId::root() {
            let data = expn.expn_data();
            if matches!(data.kind, ExpnKind::Macro(MacroKind::Attr, _))
                && data.macro_def_id.is_some_and(|mac| {
                    (self.crates.role(mac.krate) == Role::Core
                        && self.tcx.item_name(mac) == sym::test)
                        || self.crates.macro_exported_as(mac, &["test", "sim_test"])
                })
            {
                return true;
            }
            expn = data.parent;
        }
        false
    }

    pub(super) fn authority_use(&mut self, def: DefId, span: Span) {
        if !self.user_written(span) {
            return;
        }
        let key = self.key(def);
        if (key == "tide_std::sim"
            || key.starts_with("tide_std::sim::")
            || key == "tide_std::Root::sim")
            && !self.test_code(span)
        {
            self.report(
                span,
                Code::Fx0504,
                "simulation is available only in test code; move this use into a test".into(),
            );
        }
        if matches!(self.tcx.def_kind(def), DefKind::Fn | DefKind::AssocFn)
            && self
                .tcx
                .opt_item_name(def)
                .is_some_and(|name| name.as_str() == "thread_rng")
            && self.admit_def(def).is_err()
        {
            self.denied_key("thread_rng".into(), span);
        }
    }

    pub(super) fn root_name(&mut self, def: DefId, path: &hir::Path<'tcx>, id: HirId) {
        if self.key(def) != "tide_std::Root" || !self.user_written(path.span) {
            return;
        }
        if matches!(self.tcx.hir_node(id), hir::Node::Pat(pat) if root_fields(self, pat)) {
            return;
        }
        if self.tcx.hir_parent_iter(id).take(2).any(|(_, node)| {
            matches!(node, hir::Node::LetStmt(binding) if root_fields(self, binding.pat)
                && binding.ty.is_some_and(|ty| ty.span.contains(path.span)))
        }) {
            return;
        }
        // Type-relative Root::sim names Root as its self type, including an imported alias.
        if self.tcx.hir_parent_iter(id).take(2).any(|(_, node)| {
            matches!(node, hir::Node::Expr(hir::Expr {
                kind: hir::ExprKind::Path(hir::QPath::TypeRelative(_, segment)), ..
            }) if segment.ident.name.as_str() == "sim")
        }) {
            return;
        }
        let owner = id.owner.def_id;
        let entry = self.crates.is_main(owner.to_def_id())
            || (self
                .tcx
                .opt_item_name(owner)
                .is_some_and(|name| name.as_str() == "__tide_main")
                && self.tcx.opt_local_parent(owner).is_some_and(|parent| {
                    self.crates.is_main(parent.to_def_id())
                        && self
                            .crates
                            .is_entry(self.tcx.def_span(parent).ctxt().outer_expn())
                }));
        if self
            .tcx
            .hir_node_by_def_id(owner)
            .fn_decl()
            .is_some_and(|decl| {
                decl.inputs.iter().enumerate().any(|(i, input)| {
                    (entry && input.span == path.span)
                        || (entry
                            && input.span.contains(path.span)
                            && self
                                .tcx
                                .hir_maybe_body_owned_by(owner)
                                .and_then(|body| body.params.get(i))
                                .is_some_and(|param| root_fields(self, param.pat)))
                })
            })
        {
            return;
        }
        self.report(path.span, Code::Fx0503,
            "`Root` may be named only as main's parameter type, in a field pattern destructuring `root`, or as `Root::sim`; pass a capability instead".into());
    }

    fn root_destructured(&self, mut id: HirId) -> bool {
        loop {
            match self.tcx.parent_hir_node(id) {
                hir::Node::Expr(parent) => match parent.kind {
                    hir::ExprKind::AddrOf(..)
                    | hir::ExprKind::Unary(hir::UnOp::Deref, _)
                    | hir::ExprKind::DropTemps(_) => id = parent.hir_id,
                    hir::ExprKind::Let(binding) if binding.init.hir_id == id => {
                        return root_fields(self, binding.pat);
                    }
                    hir::ExprKind::Match(scrutinee, arms, _) if scrutinee.hir_id == id => {
                        return arms.iter().any(|arm| root_fields(self, arm.pat))
                            && arms.iter().all(|arm| {
                                root_fields(self, arm.pat)
                                    || matches!(arm.pat.kind, hir::PatKind::Wild)
                            });
                    }
                    _ => return false,
                },
                hir::Node::LetStmt(binding)
                    if binding.init.is_some_and(|init| init.hir_id == id) =>
                {
                    return root_fields(self, binding.pat);
                }
                _ => return false,
            }
        }
    }

    pub(super) fn authority_expr(
        &mut self,
        e: &'tcx hir::Expr<'tcx>,
        t: &'tcx ty::TypeckResults<'tcx>,
    ) {
        if !self.user_written(e.span) {
            return;
        }
        let field_base = matches!(self.tcx.parent_hir_node(e.hir_id),
            hir::Node::Expr(hir::Expr { kind: hir::ExprKind::Field(base, _), .. })
            if base.hir_id == e.hir_id);
        if t.node_type_opt(e.hir_id).is_some_and(|ty| self.is_root(ty))
            && !field_base
            && !self.root_destructured(e.hir_id)
        {
            self.report(e.span, Code::Fx0503,
                "the whole `root` cannot be passed, borrowed, stored, returned or captured; move or borrow its fields".into());
        }
        if let hir::ExprKind::Closure(closure) = e.kind {
            for capture in self.tcx.closure_captures(closure.def_id) {
                if self.is_root(capture.place.ty()) {
                    self.report(
                        capture.get_capture_kind_span(self.tcx),
                        Code::Fx0503,
                        "a closure cannot capture the whole `root`; capture its fields instead"
                            .into(),
                    );
                }
            }
        }
        if let hir::ExprKind::MethodCall(segment, receiver, _, _) = e.kind
            && t.type_dependent_def_id(e.hir_id).is_none()
            && let Some(ty) = t.expr_ty_opt(receiver)
        {
            self.missing_method(ty, segment.ident.name, e.span);
        }
        let (def, supplied) = match e.kind {
            hir::ExprKind::Call(callee, args) => {
                let Some(ty) = t.expr_ty_opt(callee) else {
                    return;
                };
                let ty::FnDef(def, _) = *ty.kind() else {
                    return;
                };
                (def, args.len())
            }
            hir::ExprKind::MethodCall(_, _, args, _) => {
                let Some(def) = t.type_dependent_def_id(e.hir_id) else {
                    return;
                };
                (def, args.len() + 1)
            }
            _ => return,
        };
        if self.crates.role(def.krate) != Role::TideStd {
            return;
        }
        let sig = self
            .tcx
            .fn_sig(def)
            .instantiate_identity()
            .skip_normalization()
            .skip_binder();
        if sig.inputs().len() == supplied + 1
            && sig.inputs().last().is_some_and(|ty| {
                matches!(ty.kind(), ty::Ref(_, inner, hir::Mutability::Not)
                    if matches!(inner.kind(), ty::Adt(adt, _) if self.key(adt.did()) == "tide_std::task::Tasks"))
            })
        {
            self.report(e.span, Code::Fx0505,
                "creating this handle needs a final `&Tasks` argument; thread `Tasks` through the parameters".into());
        }
    }

    /// Resolve the same public type path in Rust's std; never guess from a bare type name.
    fn rust_counterpart(&self, def: DefId) -> Option<DefId> {
        if self.crates.role(def.krate) == Role::RustStd {
            return Some(def);
        }
        let key = self.key(def);
        let path = key.strip_prefix("tide_std::")?;
        let mut current = self.crates.rust_std()?.as_def_id();
        for name in path.split("::") {
            current =
                self.tcx
                    .module_children(current)
                    .iter()
                    .find_map(|child| match child.res {
                        Res::Def(_, def) if child.ident.name.as_str() == name => Some(def),
                        _ => None,
                    })?;
        }
        Some(current)
    }

    fn missing_method(&mut self, ty: Ty<'tcx>, name: Symbol, span: Span) {
        let ty::Adt(adt, _) = ty.peel_refs().kind() else {
            return;
        };
        let Some(rust) = self.rust_counterpart(adt.did()) else {
            return;
        };
        for &imp in self.tcx.inherent_impls(rust) {
            for item in self.tcx.associated_items(imp).in_definition_order() {
                if item.is_fn() && item.name() == name {
                    self.denied_key(self.key(item.def_id), span);
                }
            }
        }
    }

    pub(super) fn authority_qpath(&mut self, path: &'tcx hir::QPath<'tcx>, id: HirId) {
        let hir::QPath::TypeRelative(base, segment) = path else {
            return;
        };
        if !self.user_written(path.span()) {
            return;
        }
        let Some(t) = self.typeck_of(id) else { return };
        if t.type_dependent_def_id(id).is_some() {
            return;
        }
        let ty = t
            .node_type_opt(base.hir_id)
            .filter(|ty| !ty.references_error())
            .or_else(|| {
                let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = base.kind else {
                    return None;
                };
                let def = match path.res {
                    Res::Def(DefKind::Struct | DefKind::Enum | DefKind::TyAlias, def) => def,
                    Res::Err => self.rust_path(path.segments, id, &mut FxHashSet::default())?,
                    _ => return None,
                };
                if !matches!(
                    self.tcx.def_kind(def),
                    DefKind::Struct | DefKind::Enum | DefKind::TyAlias
                ) {
                    return None;
                }
                Some(
                    self.tcx
                        .type_of(def)
                        .instantiate_identity()
                        .skip_normalization(),
                )
            });
        if let Some(ty) = ty {
            self.missing_method(ty, segment.ident.name, path.span());
        }
    }

    /// One recovery path for missing std items, including aliases and imports.
    pub(super) fn authority_unresolved(&mut self, path: &hir::Path<'tcx>, id: HirId) {
        if !self.user_written(path.span) {
            return;
        }
        if path
            .segments
            .last()
            .is_some_and(|s| s.ident.name.as_str() == "thread_rng")
        {
            self.denied_key("thread_rng".into(), path.span);
        }
        if let Some(def) = self.rust_path(path.segments, id, &mut FxHashSet::default()) {
            self.denied_key(self.key(def), path.span);
        }
    }

    fn rust_path(
        &self,
        segments: &[hir::PathSegment<'tcx>],
        id: HirId,
        seen: &mut FxHashSet<LocalDefId>,
    ) -> Option<DefId> {
        let (index, mut current) = segments
            .iter()
            .enumerate()
            .find_map(|(i, segment)| {
                let Res::Def(DefKind::Mod, def) = segment.res else {
                    return None;
                };
                self.rust_counterpart(def)
                    .or_else(|| def.is_local().then_some(def))
                    .map(|def| (i, def))
            })
            .or_else(|| {
                let index = segments.iter().position(|s| s.ident.name != kw::PathRoot)?;
                if segments[index].ident.name == sym::std {
                    return Some((index, self.crates.rust_std()?.as_def_id()));
                }
                self.imported_type(segments[index].ident.name, id, seen)
                    .map(|def| (index, def))
            })?;
        for segment in &segments[index + 1..] {
            let name = segment.ident.name;
            current = if self.tcx.def_kind(current) == DefKind::Mod {
                self.imported_member(current, name, seen).flatten()?
            } else if matches!(self.tcx.def_kind(current), DefKind::Struct | DefKind::Enum) {
                self.tcx.inherent_impls(current).iter().find_map(|&imp| {
                    self.tcx
                        .associated_items(imp)
                        .in_definition_order()
                        .find(|item| item.name() == name)
                        .map(|item| item.def_id)
                })?
            } else {
                return None;
            };
        }
        Some(current)
    }

    /// Prefer real module members before recovering missing imports.
    fn imported_member(
        &self,
        module: DefId,
        name: Symbol,
        seen: &mut FxHashSet<LocalDefId>,
    ) -> Option<Option<DefId>> {
        let children = match module.as_local() {
            Some(local) => self.tcx.module_children_local(local),
            None => self.tcx.module_children(module),
        };
        children
            .iter()
            .find_map(|child| match child.res {
                Res::Def(_, def) if child.ident.name == name => Some(Some(def)),
                _ => None,
            })
            .or_else(|| {
                module.as_local().and_then(|local| {
                    self.import_in_scope(name, self.tcx.local_def_id_to_hir_id(local), seen)
                })
            })
    }

    fn imported_type(
        &self,
        name: Symbol,
        id: HirId,
        seen: &mut FxHashSet<LocalDefId>,
    ) -> Option<DefId> {
        for (scope, node) in self.tcx.hir_parent_iter(id) {
            if lexical_scope(node) {
                if let Some(resolved) = self.import_in_scope(name, scope, seen) {
                    return resolved;
                }
                if !matches!(node, hir::Node::Block(_)) {
                    break;
                }
            }
        }
        None
    }

    /// Some(None) is a shadowing binding whose source cannot be recovered.
    fn import_in_scope(
        &self,
        name: Symbol,
        scope: HirId,
        seen: &mut FxHashSet<LocalDefId>,
    ) -> Option<Option<DefId>> {
        let imports = self
            .tcx
            .hir_crate_items(())
            .definitions()
            .filter_map(|def| {
                let hir::Node::Item(item) = self.tcx.hir_node_by_def_id(def) else {
                    return None;
                };
                let hir::ItemKind::Use(path, kind) = item.kind else {
                    return None;
                };
                (self
                    .tcx
                    .hir_parent_iter(item.hir_id())
                    .find(|(_, node)| lexical_scope(*node))
                    .is_some_and(|(parent, _)| parent == scope))
                .then_some((def, path, kind, item.hir_id()))
            })
            .collect::<Vec<_>>();
        let mut named = imports.iter().filter(
            |(_, _, kind, _)| matches!(kind, hir::UseKind::Single(ident) if ident.name == name),
        );
        if let Some(&(def, path, _, id)) = named.next() {
            if named.next().is_some() || !seen.insert(def) {
                return Some(None);
            }
            let result = self.rust_path(path.segments, id, seen);
            seen.remove(&def);
            return Some(result);
        }
        let mut result = None;
        for (def, path, kind, id) in imports {
            if !matches!(kind, hir::UseKind::Glob) || !seen.insert(def) {
                continue;
            }
            let found = self
                .rust_path(path.segments, id, seen)
                .filter(|module| self.tcx.def_kind(*module) == DefKind::Mod)
                .and_then(|module| self.imported_member(module, name, seen));
            seen.remove(&def);
            if let Some(found) = found {
                if found.is_none() || result.is_some_and(|previous| previous != found) {
                    return Some(None);
                }
                result = Some(found);
            }
        }
        result
    }

    fn denied_key(&mut self, key: String, span: Span) {
        if let Some(rule) = policy::lookup(&key).filter(|r| r.verdict == Verdict::Deny) {
            self.reject(
                span,
                Rejection {
                    key,
                    code: rule.code,
                    help: rule.help,
                },
                if rule.code == Code::Fx0501 {
                    ""
                } else {
                    " (Tide's std does not export it)"
                },
            );
        }
    }
}

fn lexical_scope(node: hir::Node<'_>) -> bool {
    matches!(
        node,
        hir::Node::Block(_)
            | hir::Node::Crate(_)
            | hir::Node::Item(hir::Item {
                kind: hir::ItemKind::Mod(..),
                ..
            })
    )
}

fn root_fields(walk: &Walk<'_, '_>, pat: &hir::Pat<'_>) -> bool {
    match pat.kind {
        hir::PatKind::Struct(hir::QPath::Resolved(_, path), ..) => {
            matches!(path.res, Res::Def(_, def) if walk.key(def) == "tide_std::Root")
        }
        hir::PatKind::Ref(inner, ..) => root_fields(walk, inner),
        hir::PatKind::Or(patterns) => patterns.iter().all(|pat| root_fields(walk, pat)),
        _ => false,
    }
}

/// Keep the target configuration fixed and disable test: a test-only cfg becomes false.
fn requires_test(tcx: TyCtxt<'_>, entry: &CfgEntry) -> bool {
    fn without_test(tcx: TyCtxt<'_>, entry: &CfgEntry) -> Option<bool> {
        match entry {
            CfgEntry::NameValue {
                name, value: None, ..
            } if *name == sym::test => Some(false),
            CfgEntry::NameValue { name, value, .. } => {
                Some(tcx.sess.config.contains(&(*name, *value)))
            }
            CfgEntry::All(entries, _) => {
                if entries.iter().any(|e| without_test(tcx, e) == Some(false)) {
                    Some(false)
                } else if entries.iter().all(|e| without_test(tcx, e) == Some(true)) {
                    Some(true)
                } else {
                    None
                }
            }
            CfgEntry::Any(entries, _) => {
                if entries.iter().any(|e| without_test(tcx, e) == Some(true)) {
                    Some(true)
                } else if entries.iter().all(|e| without_test(tcx, e) == Some(false)) {
                    Some(false)
                } else {
                    None
                }
            }
            CfgEntry::Not(entry, _) => without_test(tcx, entry).map(|v| !v),
            CfgEntry::Bool(value, _) => Some(*value),
            _ => None,
        }
    }
    without_test(tcx, entry) == Some(false)
}
