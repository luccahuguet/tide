//! Closed surface forms (D6), lifetime diagnostics (D2), and lossless casts (D3).
use rustc_span::ExpnData;

use super::*;

const RUST_DERIVES: &[&str] = &[
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
    "Clone",
    "Copy",
    "Debug",
    "Default",
];
const TIDE_DERIVES: &[&str] = &["Error", "Encode", "Decode"];
const BANG_FORMS: &[&str] = &[
    "print",
    "println",
    "eprint",
    "eprintln",
    "format",
    "write",
    "writeln",
    "vec",
    "matches",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "panic",
    "todo",
    "unimplemented",
    "unreachable",
    "dbg",
    "error",
    "warn",
    "info",
    "debug",
    "trace",
    "invariant",
    "hole",
];

impl<'a, 'tcx> Walk<'a, 'tcx> {
    /// Returns true when this form has its own verdict instead of the allowlist's.
    pub(super) fn surface_expansion(&mut self, data: &ExpnData) -> bool {
        let ExpnKind::Macro(kind, written_name) = data.kind else {
            return false;
        };
        if !self.user_written(data.call_site) {
            return false;
        }
        let name = data
            .macro_def_id
            .map_or(written_name, |mac| self.tcx.item_name(mac));
        let name = name.as_str();
        let name = name.rsplit("::").next().unwrap();
        let code = match kind {
            MacroKind::Derive => {
                let allowed = data.macro_def_id.is_some_and(|mac| {
                    (self.crates.role(mac.krate) == Role::Core && RUST_DERIVES.contains(&name))
                        || self.crates.macro_exported_as(mac, TIDE_DERIVES)
                });
                if allowed {
                    return false;
                }
                Code::Fx0102
            }
            MacroKind::Bang => {
                // These forms have other owners, including select! in the habits slice.
                if matches!(
                    name,
                    "include"
                        | "include_str"
                        | "include_bytes"
                        | "env"
                        | "option_env"
                        | "thread_local"
                        | "lazy_static"
                        | "asm"
                        | "global_asm"
                        | "naked_asm"
                        | "select"
                ) {
                    return false;
                }
                let allowed = data.macro_def_id.is_some_and(|mac| {
                    (BANG_FORMS.contains(&name)
                        && matches!(
                            self.crates.role(mac.krate),
                            Role::Core | Role::Alloc | Role::RustStd
                        ))
                        || self.crates.macro_exported_as(mac, BANG_FORMS)
                });
                if allowed {
                    return false;
                }
                Code::Fx0103
            }
            MacroKind::Attr => return false,
        };
        // Proc macros can put generated tokens at an input span without expansion context.
        if kind == MacroKind::Bang
            && data.parent != ExpnId::root()
            && !written_bang(self.tcx, data.call_site, written_name)
        {
            return false;
        }
        let message = if code == Code::Fx0102 {
            format!(
                "derive `{name}` is outside Tide's closed derive set; use Error, Encode or Decode"
            )
        } else {
            format!(
                "`{name}!` is outside Tide's fixed built-in forms; use a function or the fixed forms"
            )
        };
        self.report(data.call_site, code, message);
        true
    }

    pub(super) fn surface_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        if self.user_written(item.span)
            && let hir::ItemKind::Trait { generics, .. } = item.kind
            && let Some(parameter) = generics.params.iter().find(|p| p.is_lifetime())
        {
            self.report(
                parameter.span,
                Code::Fx0201,
                "a trait cannot have lifetime parameters; put the lifetime on a method or impl"
                    .into(),
            );
        }
    }

    pub(super) fn surface_output(&mut self, output: &'tcx hir::Ty<'tcx>) {
        struct Elision(Vec<Span>);
        impl<'tcx> Visitor<'tcx> for Elision {
            fn visit_lifetime(&mut self, lifetime: &'tcx hir::Lifetime) {
                if matches!(lifetime.kind, hir::LifetimeKind::Error(_)) && lifetime.is_anonymous() {
                    self.0.push(lifetime.ident.span);
                }
            }
        }
        let mut elision = Elision(Vec::new());
        if let Some(output) = output.try_as_ambig_ty() {
            elision.visit_ty(output);
        }
        for span in elision.0 {
            if self.user_written(span) {
                self.report(
                    span,
                    Code::Fx0202,
                    "output lifetime cannot be elided; name the input lifetime the result borrows"
                        .into(),
                );
            }
        }
    }

    pub(super) fn surface_expr(&mut self, e: &'tcx hir::Expr<'tcx>, t: &ty::TypeckResults<'tcx>) {
        if !self.user_written(e.span) {
            return;
        }
        if let hir::ExprKind::Cast(source, _) = e.kind
            && let (Some(from), Some(to)) = (t.expr_ty_opt(source), t.expr_ty_opt(e))
            && !from.references_error()
            && !to.references_error()
            && !lossless_cast(self.tcx, from, to)
        {
            self.report(e.span, Code::Fx0301, format!(
                "`{from} as {to}` can lose information; use try_from, T::try_from_float(f.trunc()), truncate (same signedness, fixed widths), cast_signed/cast_unsigned, or round_from"));
        }
        if let hir::ExprKind::Struct(_, fields, _) = e.kind {
            self.record_self_borrows(fields, t);
        }
        if let hir::ExprKind::Ret(Some(value)) = e.kind
            && let Some(output) = self.surface_return_type()
        {
            self.owned_value(value, Some(output), &[], &mut FxHashSet::default());
        }
        if let hir::ExprKind::Assign(place, value, _) = e.kind
            && let hir::ExprKind::Field(base, _) = place.kind
            && let Some(base) = t.expr_ty_adjusted_opt(base)
            && let ty::Adt(adt, args) = base.peel_refs().kind()
            && adt.is_struct()
            && let Some(index) = t.opt_field_index(place.hir_id)
        {
            let destination = adt.non_enum_variant().fields[index]
                .ty(self.tcx, args)
                .skip_normalization();
            if static_object(destination) {
                self.owned_value(
                    value,
                    Some(destination),
                    &[place.span],
                    &mut FxHashSet::default(),
                );
            }
        }
        // Typeck retains explicit static object lifetimes at coercions and casts. Return
        // expectations below also carry static destinations through inferred local objects.
        for target in t.expr_adjustments(e).iter().map(|adj| adj.target).chain(
            t.expr_ty_opt(e)
                .filter(|_| matches!(e.kind, hir::ExprKind::Cast(..))),
        ) {
            if static_object(target) {
                self.owned_value(e, Some(target), &[], &mut FxHashSet::default());
            }
        }
        if matches!(e.kind, hir::ExprKind::Struct(..))
            && let Some(ty) = t.expr_ty_opt(e)
        {
            self.owned_value(e, Some(ty), &[], &mut FxHashSet::default());
        }
        let (called, args, offset, generic) = match e.kind {
            hir::ExprKind::MethodCall(_, _, args, _) => {
                let Some(def) = t.type_dependent_def_id(e.hir_id) else {
                    return;
                };
                (def, args, 1, t.node_args(e.hir_id))
            }
            hir::ExprKind::Call(callee, args) => {
                let Some(ty) = t.expr_ty_opt(callee) else {
                    return;
                };
                let ty::FnDef(def, generic) = *ty.kind() else {
                    return;
                };
                (
                    def,
                    args,
                    0,
                    self.tcx.instantiate_bound_regions_with_erased(generic),
                )
            }
            _ => return,
        };
        let isolated = self.key(called) == "tide_std::task::Tasks::spawn_isolated";
        let signature = self
            .tcx
            .fn_sig(called)
            .instantiate(self.tcx, generic)
            .skip_normalization()
            .skip_binder();
        for (index, arg) in args.iter().enumerate() {
            if isolated {
                self.owned_value(arg, None, &[e.span], &mut FxHashSet::default());
            } else if let Some(&expected) = signature.inputs().get(index + offset) {
                self.owned_value(arg, Some(expected), &[], &mut FxHashSet::default());
            }
        }
    }

    pub(super) fn surface_return_type(&self) -> Option<Ty<'tcx>> {
        let owner = self.owner?;
        matches!(self.tcx.def_kind(owner), DefKind::Fn | DefKind::AssocFn).then(|| {
            self.tcx
                .fn_sig(owner)
                .instantiate_identity()
                .skip_normalization()
                .skip_binder()
                .output()
        })
    }

    /// Follow the value to its static destination, without treating other statements or
    /// closure bodies as part of that flow. None is spawn_isolated's static callable bound.
    pub(super) fn owned_value(
        &self,
        e: &'tcx hir::Expr<'tcx>,
        expected: Option<Ty<'tcx>>,
        targets: &[Span],
        seen: &mut FxHashSet<HirId>,
    ) {
        if !seen.insert(e.hir_id) {
            return;
        }
        let Some(t) = self.typeck_of(e.hir_id) else {
            return;
        };
        if t.expr_ty_opt(e).is_none_or(|ty| ty.references_error()) {
            return;
        }
        // A projected/desugared value is not its enclosing initializer. Never look up
        // a variant of one ADT in another while following a destination's expectation.
        if let Some(expected) = expected
            && let ty::Adt(wanted, _) = expected.kind()
            && let Some(actual) = t.expr_ty_opt(e)
            && let ty::Adt(actual, _) = actual.kind()
            && wanted != actual
        {
            return;
        }
        let mut targets = targets.to_vec();
        targets.push(e.span);
        if expected.is_none_or(static_object)
            && let Some(ty) = t.expr_ty_opt(e)
        {
            let mut callee = ty;
            loop {
                callee = match *callee.kind() {
                    ty::Ref(_, inner, _) => inner,
                    ty::Adt(adt, args) if adt.is_box() => args.type_at(0),
                    _ => break,
                };
            }
            if let ty::Closure(def, _) = *callee.kind()
                && let Some(local) = def.as_local()
                && let hir::Node::Expr(expression) = self.tcx.hir_node_by_def_id(local)
                && let hir::ExprKind::Closure(closure) = expression.kind
                && self.user_written(closure.fn_decl_span)
            {
                let mut uses = Vec::new();
                let mut origins = Vec::new();
                for capture in self.tcx.closure_captures(local) {
                    // This evidence is only for non-E0373 errors. Rustc's E0373 spans
                    // identify the borrowed place even when its provenance is unknown.
                    if capture
                        .place
                        .projections
                        .iter()
                        .any(|p| !matches!(p.kind, rustc_middle::hir::place::ProjectionKind::Deref))
                    {
                        continue;
                    }
                    let ty = capture.place.ty();
                    if !matches!(capture.info.capture_kind, ty::UpvarCapture::ByRef(_))
                        && !ty.has_param()
                        && !ty
                            .walk()
                            .filter_map(|arg| arg.as_region())
                            .any(|r| !r.is_static())
                    {
                        continue;
                    }
                    let mut source = Vec::new();
                    if !self.capture_origins(
                        capture.get_root_variable(),
                        closure.fn_decl_span,
                        None,
                        &mut FxHashSet::default(),
                        &mut source,
                    ) {
                        continue;
                    }
                    uses.push(capture.get_path_span(self.tcx));
                    self.region_origins(ty, local, &mut origins);
                    origins.extend(source);
                }
                self.labels.owned(crate::diag::OwnedClosure {
                    declaration: closure.fn_decl_span,
                    expression: expression.span,
                    requirements: targets.clone(),
                    uses,
                    origins,
                });
            }
        }
        match e.kind {
            hir::ExprKind::Path(hir::QPath::Resolved(_, path)) => {
                if let Res::Local(binding) = path.res
                    && let Some(init) = self.binding_source(binding, e.span, None)
                {
                    self.owned_value(init, expected, &targets, seen);
                }
            }
            hir::ExprKind::Block(block, _) => {
                if let Some(value) = block.expr {
                    self.owned_value(value, expected, &targets, seen);
                }
            }
            hir::ExprKind::If(_, then, otherwise) => {
                self.owned_value(then, expected, &targets, seen);
                if let Some(otherwise) = otherwise {
                    self.owned_value(otherwise, expected, &targets, seen);
                }
            }
            hir::ExprKind::Match(_, arms, _) => {
                for arm in arms {
                    self.owned_value(arm.body, expected, &targets, seen);
                }
            }
            hir::ExprKind::Cast(value, _)
            | hir::ExprKind::DropTemps(value)
            | hir::ExprKind::Ret(Some(value)) => self.owned_value(value, expected, &targets, seen),
            hir::ExprKind::Array(values) => {
                if let Some(expected) = expected
                    && let ty::Array(element, _) = expected.kind()
                {
                    for value in values {
                        self.owned_value(value, Some(*element), &targets, seen);
                    }
                }
            }
            hir::ExprKind::Tup(values) => {
                if let Some(expected) = expected
                    && let ty::Tuple(types) = expected.kind()
                {
                    for (value, ty) in values.iter().zip(types.iter()) {
                        self.owned_value(value, Some(ty), &targets, seen);
                    }
                }
            }
            hir::ExprKind::Struct(path, fields, _) => {
                if let Some(expected) = expected
                    && let ty::Adt(adt, args) = expected.kind()
                {
                    let res = t.qpath_res(path, e.hir_id);
                    if !matches!(res, Res::Err) {
                        let variant = adt.variant_of_res(res);
                        for field in fields {
                            if let Some(index) = t.opt_field_index(field.hir_id) {
                                self.owned_value(
                                    field.expr,
                                    Some(
                                        variant.fields[index]
                                            .ty(self.tcx, args)
                                            .skip_normalization(),
                                    ),
                                    &targets,
                                    seen,
                                );
                            }
                        }
                    }
                }
            }
            hir::ExprKind::Call(callee, values) => {
                if let Some(expected) = expected
                    && let ty::Adt(adt, args) = expected.kind()
                    && let Some(callee) = t.expr_ty_opt(callee)
                    && let ty::FnDef(def, _) = callee.kind()
                    && matches!(self.tcx.def_kind(*def), DefKind::Ctor(..))
                {
                    for (value, field) in values.iter().zip(&adt.variant_with_ctor_id(*def).fields)
                    {
                        self.owned_value(
                            value,
                            Some(field.ty(self.tcx, args).skip_normalization()),
                            &targets,
                            seen,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn binding_source(
        &self,
        binding: HirId,
        before: Span,
        borrow: Option<HirId>,
    ) -> Option<&'tcx hir::Expr<'tcx>> {
        for (_, node) in self.tcx.hir_parent_iter(binding) {
            match node {
                hir::Node::LetStmt(local) => {
                    if !matches!(local.pat.kind, hir::PatKind::Binding(_, id, _, None) if id == binding)
                    {
                        return None;
                    }
                    let writes: Vec<_> = self
                        .binding_assignments(binding)
                        .into_iter()
                        .filter(|e| Some(e.hir_id) != borrow)
                        .collect();
                    return match (local.init, writes.as_slice()) {
                        (Some(init), []) => Some(init),
                        (None, [assignment]) if assignment.span.hi() < before.lo() => {
                            let owner = self.tcx.hir_enclosing_body_owner(binding);
                            let body = self.tcx.hir_body_owned_by(owner);
                            if let hir::ExprKind::Block(block, _) = body.value.kind
                                && block.stmts.iter().any(|stmt| matches!(stmt.kind,
                                    hir::StmtKind::Semi(e) | hir::StmtKind::Expr(e) if e.hir_id == assignment.hir_id))
                                && let hir::ExprKind::Assign(place, value, _) = assignment.kind
                                && matches!(place.kind, hir::ExprKind::Path(hir::QPath::Resolved(_, path)) if path.res == Res::Local(binding))
                            { Some(value) } else { None }
                        }
                        _ => None,
                    };
                }
                hir::Node::Arm(_) | hir::Node::Param(_) | hir::Node::Item(_) => return None,
                _ => {}
            }
        }
        None
    }

    fn binding_assignments(&self, binding: HirId) -> Vec<&'tcx hir::Expr<'tcx>> {
        struct Writes<'tcx> {
            tcx: TyCtxt<'tcx>,
            binding: HirId,
            writes: Vec<&'tcx hir::Expr<'tcx>>,
        }
        impl<'tcx> Visitor<'tcx> for Writes<'tcx> {
            type NestedFilter = nested_filter::OnlyBodies;
            fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
                self.tcx
            }
            fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
                let place = match e.kind {
                    hir::ExprKind::Assign(place, ..) | hir::ExprKind::AssignOp(_, place, _) => {
                        Some(place)
                    }
                    hir::ExprKind::AddrOf(_, hir::Mutability::Mut, place) => Some(place),
                    _ => None,
                };
                if let Some(place) = place {
                    struct Uses(HirId, bool);
                    impl<'tcx> Visitor<'tcx> for Uses {
                        fn visit_path(&mut self, path: &hir::Path<'tcx>, _: HirId) {
                            self.1 |= path.res == Res::Local(self.0);
                        }
                    }
                    let mut uses = Uses(self.binding, false);
                    uses.visit_expr(place);
                    if uses.1 {
                        self.writes.push(e);
                    }
                }
                intravisit::walk_expr(self, e);
            }
        }
        let owner = self.tcx.hir_enclosing_body_owner(binding);
        let mut writes = Writes {
            tcx: self.tcx,
            binding,
            writes: Vec::new(),
        };
        writes.visit_body(self.tcx.hir_body_owned_by(owner));
        writes.writes
    }

    fn region_origins(&self, ty: Ty<'tcx>, owner: LocalDefId, spans: &mut Vec<Span>) {
        for region in ty.walk().filter_map(|arg| arg.as_region()) {
            let def = match region.kind() {
                ty::ReEarlyParam(param) => Some(
                    self.tcx
                        .generics_of(owner)
                        .param_at(param.index as usize, self.tcx)
                        .def_id,
                ),
                ty::ReLateParam(param) => match param.kind {
                    ty::LateParamRegionKind::Named(def) => Some(def),
                    _ => None,
                },
                ty::ReBound(_, bound) => match bound.kind {
                    ty::BoundRegionKind::Named(def) => Some(def),
                    _ => None,
                },
                _ => None,
            };
            if let Some(def) = def {
                spans.push(self.tcx.def_span(def));
            }
        }
    }

    fn capture_origins(
        &self,
        binding: HirId,
        before: Span,
        borrow: Option<HirId>,
        seen: &mut FxHashSet<HirId>,
        spans: &mut Vec<Span>,
    ) -> bool {
        if !seen.insert(binding) {
            return false;
        }
        for (_, node) in self.tcx.hir_parent_iter(binding) {
            match node {
                hir::Node::Param(param) => {
                    if !matches!(param.pat.kind, hir::PatKind::Binding(_, id, _, None) if id == binding)
                        || self
                            .binding_assignments(binding)
                            .iter()
                            .any(|e| Some(e.hir_id) != borrow)
                    {
                        return false;
                    }
                    spans.push(self.tcx.hir_span(binding));
                    // The lifetime label can be on the parameter's type rather than its name.
                    let owner = self.tcx.hir_enclosing_body_owner(param.hir_id);
                    let body = self.tcx.hir_body_owned_by(owner);
                    if let Some(index) = body.params.iter().position(|p| p.hir_id == param.hir_id)
                        && let Some(decl) = self.tcx.hir_node_by_def_id(owner).fn_decl()
                        && let Some(ty) = decl.inputs.get(index)
                    {
                        spans.push(ty.span);
                        if matches!(self.tcx.def_kind(owner), DefKind::Fn | DefKind::AssocFn) {
                            let sig = self
                                .tcx
                                .fn_sig(owner)
                                .instantiate_identity()
                                .skip_normalization()
                                .skip_binder();
                            if let Some(&ty) = sig.inputs().get(index) {
                                self.region_origins(ty, owner, spans);
                            }
                        }
                    }
                    return true;
                }
                hir::Node::LetStmt(local) => {
                    let Some(init) = self.binding_source(binding, before, borrow) else {
                        return false;
                    };
                    // An owned call/literal is a storage leaf, not a reason to inspect
                    // its arguments. Only borrowing values continue the alias chain.
                    if matches!(
                        init.kind,
                        hir::ExprKind::Call(..)
                            | hir::ExprKind::MethodCall(..)
                            | hir::ExprKind::Lit(_)
                    ) && self
                        .typeck_of(binding)
                        .and_then(|t| t.node_type_opt(binding))
                        .is_some_and(|ty| {
                            !ty.has_param() && !ty.walk().any(|arg| arg.as_region().is_some())
                        })
                    {
                        spans.push(local.pat.span);
                        return true;
                    }
                    if !self.source_origins(init, before, seen, spans) {
                        return false;
                    }
                    spans.push(local.pat.span);
                    spans.push(init.span);
                    return true;
                }
                hir::Node::Arm(_) | hir::Node::Item(_) => return false,
                _ => {}
            }
        }
        false
    }

    fn source_origins(
        &self,
        e: &'tcx hir::Expr<'tcx>,
        before: Span,
        seen: &mut FxHashSet<HirId>,
        spans: &mut Vec<Span>,
    ) -> bool {
        let (place, borrow) = if matches!(e.kind, hir::ExprKind::Path(_)) {
            (e, None)
        } else if let Some(source) = self.direct_borrow(e) {
            source
        } else {
            return false;
        };
        if let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = place.kind
            && let Res::Local(binding) = path.res
        {
            self.capture_origins(binding, before, borrow, seen, spans)
        } else {
            false
        }
    }

    fn direct_borrow(
        &self,
        e: &'tcx hir::Expr<'tcx>,
    ) -> Option<(&'tcx hir::Expr<'tcx>, Option<HirId>)> {
        match e.kind {
            hir::ExprKind::AddrOf(hir::BorrowKind::Ref, _, place) => Some((place, Some(e.hir_id))),
            hir::ExprKind::MethodCall(_, receiver, [], _) => {
                let t = self.typeck_of(e.hir_id)?;
                let def = t.type_dependent_def_id(e.hir_id)?;
                let sig = self
                    .tcx
                    .fn_sig(def)
                    .instantiate_identity()
                    .skip_normalization()
                    .skip_binder();
                if !matches!((sig.inputs().first().map(|t| t.kind()), sig.output().kind()),
                    (Some(ty::Ref(input, _, _)), ty::Ref(output, _, _)) if input == output && !input.is_static())
                {
                    return None;
                }
                Some((receiver, None))
            }
            _ => None,
        }
    }

    fn record_self_borrows(
        &self,
        fields: &'tcx [hir::ExprField<'tcx>],
        t: &ty::TypeckResults<'tcx>,
    ) {
        for (index, owner_field) in fields.iter().enumerate() {
            let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = owner_field.expr.kind else {
                continue;
            };
            let Res::Local(owner) = path.res else {
                continue;
            };
            let Some(ty) = t.expr_ty_opt(owner_field.expr) else {
                continue;
            };
            // Copies and adjusted reborrows do not move this local into the field.
            if ty.references_error()
                || !t.expr_adjustments(owner_field.expr).is_empty()
                || self.tcx.type_is_copy_modulo_regions(self.pure_env(), ty)
            {
                continue;
            }
            for (other, field) in fields.iter().enumerate() {
                if other == index {
                    continue;
                }
                let Some((place, _)) = self.direct_borrow(field.expr) else {
                    continue;
                };
                if !matches!(
                    t.expr_ty_adjusted_opt(field.expr).map(|ty| ty.kind()),
                    Some(ty::Ref(..))
                ) || !Self::retains_loan_root(field.expr, t, true)
                {
                    continue;
                }
                // A reference-valued receiver/place can borrow foreign data instead of
                // this owner's storage after deref coercion; omit that uncertain case.
                if t.expr_ty_opt(place).is_none_or(|ty| {
                    matches!(ty.kind(), ty::Ref(..) | ty::RawPtr(..)) || ty.references_error()
                }) {
                    continue;
                }
                if self.borrow_owner(place, t) == Some(owner) {
                    self.labels
                        .record_self_borrow(owner_field.expr.span, field.expr.span);
                }
            }
        }
    }

    fn borrow_owner(
        &self,
        place: &'tcx hir::Expr<'tcx>,
        t: &ty::TypeckResults<'tcx>,
    ) -> Option<HirId> {
        if !Self::retains_loan_root(place, t, false) {
            return None;
        }
        match place.kind {
            hir::ExprKind::Path(hir::QPath::Resolved(_, path)) => match path.res {
                Res::Local(id) => Some(id),
                _ => None,
            },
            hir::ExprKind::Field(base, _) | hir::ExprKind::Index(base, ..) => {
                if matches!(t.expr_ty_opt(base)?.kind(), ty::Ref(..) | ty::RawPtr(..)) {
                    return None;
                }
                self.borrow_owner(base, t)
            }
            hir::ExprKind::Unary(hir::UnOp::Deref, base) => {
                if let ty::Adt(adt, _) = t.expr_ty_opt(base)?.kind()
                    && adt.is_box()
                {
                    self.borrow_owner(base, t)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn retains_loan_root(
        e: &'tcx hir::Expr<'tcx>,
        t: &ty::TypeckResults<'tcx>,
        mut new_borrow: bool,
    ) -> bool {
        let Some(mut ty) = t.expr_ty_opt(e) else {
            return false;
        };
        for adjustment in t.expr_adjustments(e) {
            if matches!(adjustment.kind, Adjust::Deref(_)) {
                // Dereferencing a stored reference changes the loan root to foreign
                // data. Only a reference just created by this borrow may be crossed.
                if matches!(ty.kind(), ty::Ref(..) | ty::RawPtr(..)) && !new_borrow {
                    return false;
                }
                new_borrow = false;
            } else {
                new_borrow = matches!(adjustment.kind, Adjust::Borrow(_));
            }
            ty = adjustment.target;
        }
        true
    }
}

fn written_bang(tcx: TyCtxt<'_>, span: Span, name: rustc_span::Symbol) -> bool {
    use rustc_ast::token::TokenKind;
    use rustc_parse::parser::PathStyle;
    use rustc_session::parse::ParseSess;
    use rustc_span::FileName;

    let Ok(source) = tcx.sess.source_map().span_to_snippet(span) else {
        return false;
    };
    let psess = ParseSess::new();
    psess.dcx().make_silent();
    let mut parser = match rustc_parse::new_parser_from_source_str(
        &psess,
        FileName::Custom("<tide macro provenance>".into()),
        source,
        rustc_parse::lexer::StripTokens::ShebangAndFrontmatter,
    ) {
        Ok(parser) => parser,
        Err(errors) => {
            for error in errors {
                error.emit();
            }
            return false;
        }
    };
    let path = match parser.parse_path(PathStyle::Mod) {
        Ok(path) => path,
        Err(error) => {
            error.cancel();
            return false;
        }
    };
    parser.token.kind == TokenKind::Bang
        && path
            .segments
            .iter()
            .filter(|s| s.ident.name != kw::PathRoot)
            .map(|s| s.ident.name.as_str())
            .eq(name.as_str().trim_start_matches("::").split("::"))
}

fn static_object(ty: Ty<'_>) -> bool {
    match ty.kind() {
        ty::Dynamic(_, region) => region.is_static(),
        ty::Adt(_, args) => args.types().any(static_object),
        ty::Array(inner, _) | ty::Slice(inner) | ty::Ref(_, inner, _) | ty::RawPtr(inner, _) => {
            static_object(*inner)
        }
        ty::Tuple(types) => types.iter().any(static_object),
        _ => false,
    }
}

/// (signed, minimum width, maximum width): pointer sizes are portable, not host-sized.
fn integer(ty: Ty<'_>) -> Option<(bool, u64, u64)> {
    let (signed, width) = match ty.kind() {
        ty::Int(int) => (true, int.bit_width()),
        ty::Uint(int) => (false, int.bit_width()),
        ty::RawPtr(..) | ty::FnPtr(..) | ty::FnDef(..) => (false, None),
        _ => return None,
    };
    Some((signed, width.unwrap_or(32), width.unwrap_or(64)))
}

fn lossless_cast<'tcx>(tcx: TyCtxt<'tcx>, from: Ty<'tcx>, to: Ty<'tcx>) -> bool {
    let address = |ty: Ty<'_>| matches!(ty.kind(), ty::RawPtr(..) | ty::FnPtr(..) | ty::FnDef(..));
    // Non-numeric casts retain their existing owners (e.g. reference/pointer coercions).
    if from == to
        || address(from) && address(to)
        || address(from) && to == tcx.types.usize
        || from == tcx.types.usize && address(to)
    {
        return true;
    }
    if let Some((to_signed, to_min, _)) = integer(to) {
        if let Some((from_signed, _, from_max)) = integer(from) {
            return if from_signed == to_signed {
                from_max <= to_min
            } else {
                !from_signed && to_signed && from_max < to_min
            };
        }
        return match from.kind() {
            ty::Bool => true,
            ty::Char => {
                if to_signed {
                    to_min > 32
                } else {
                    to_min >= 32
                }
            }
            ty::Float(_) => false,
            ty::Adt(adt, _)
                if adt.is_enum() && adt.variants().iter().all(|v| v.fields.is_empty()) =>
            {
                adt.discriminants(tcx).all(|(_, discr)| {
                    let (size, signed) = discr.ty.int_size_and_signed(tcx);
                    if signed {
                        let value = size.sign_extend(discr.val) as i128;
                        if to_signed {
                            to_min == 128
                                || (-(1i128 << (to_min - 1))..(1i128 << (to_min - 1)))
                                    .contains(&value)
                        } else {
                            value >= 0 && (to_min == 128 || (value as u128) < (1u128 << to_min))
                        }
                    } else if to_signed {
                        discr.val < (1u128 << (to_min - 1))
                    } else {
                        to_min == 128 || discr.val < (1u128 << to_min)
                    }
                })
            }
            _ => true,
        };
    }
    if let ty::Float(float) = to.kind() {
        let mantissa = match float {
            ty::FloatTy::F32 => 24,
            ty::FloatTy::F64 => 53,
            _ => return true,
        };
        if let Some((signed, _, width)) = integer(from) {
            return width - u64::from(signed) <= mantissa;
        }
        if let ty::Float(src) = from.kind() {
            return src.bit_width() <= float.bit_width();
        }
    }
    if to.is_char() && from.is_integral() {
        return from == tcx.types.u8;
    }
    true
}
