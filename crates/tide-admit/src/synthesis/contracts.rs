//! Ensures-bearing functions become property entries in a reserved child module.
use super::*;
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, ExprKind, PatKind, QPath};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::Upcast;
use rustc_middle::ty::print::{with_crate_prefix, with_no_trimmed_paths, with_no_visible_paths};
use rustc_middle::ty::{self, GenericArgsRef};
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt;
use rustc_trait_selection::traits::{Obligation, ObligationCause};

pub(super) fn plan(tcx: TyCtxt<'_>, crates: &Crates, arbitrary: DefId, edits: &mut Insertions) {
    let mut modules: BTreeMap<u32, (LocalModId, String)> = BTreeMap::new();
    let paths = exported_paths(tcx, crates.tide_std().unwrap().as_def_id());
    for def in tcx.hir_body_owners() {
        if !matches!(tcx.def_kind(def), DefKind::Fn | DefKind::AssocFn) {
            continue;
        }
        let clauses = Clauses::read(tcx, crates, def);
        if !clauses.ensures {
            continue;
        }
        let parent = tcx.local_parent(def);
        let owner = matches!(tcx.def_kind(parent), DefKind::Impl { .. }).then_some(parent);
        let module = tcx.parent_module_from_def_id(def);
        let name = test_name(tcx, def);
        let generator = Generator {
            tcx,
            arbitrary,
            crates,
            paths: &paths,
            module,
        };
        let result = generator.test(def, owner, &clauses);
        let (_, code) = modules
            .entry(module.to_local_def_id().local_def_index.as_u32())
            .or_insert_with(|| (module, String::new()));
        match result {
            Ok(test) => write!(code, " {test} ").unwrap(),
            Err(reason) => write!(
                code,
                " #[::core::prelude::v1::test] #[ignore = {:?}] fn r#{name}() {{}} ",
                format!("unchecked: {reason}")
            )
            .unwrap(),
        }
    }
    for (_, (module, tests)) in modules {
        edits.append(
            tcx,
            module,
            &format!("#[cfg(test)] mod __tide_contract_tests {{ #[allow(unused_imports)] use super::*; {tests} }}"),
        );
    }
}
fn test_name(tcx: TyCtxt<'_>, def: LocalDefId) -> String {
    format!(
        "{}_{}",
        tcx.item_name(def.to_def_id()),
        def.local_def_index.as_u32()
    )
}
// Assumes no exported std attribute emits marker calls carrying user-chosen strings.
fn contract_expansion(crates: &Crates, span: Span) -> bool {
    let expansion = span.ctxt().outer_expn_data();
    matches!(expansion.kind, ExpnKind::Macro(MacroKind::Attr, _))
        && expansion
            .macro_def_id
            .is_some_and(|mac| crates.exported(mac))
}

fn exported_paths(tcx: TyCtxt<'_>, root: DefId) -> FxHashMap<DefId, String> {
    let mut paths = FxHashMap::default();
    let mut visited = FxHashSet::default();
    let mut pending = std::collections::VecDeque::from([(root, "::std".to_owned())]);
    while let Some((module, path)) = pending.pop_front() {
        if !visited.insert(module) {
            continue;
        }
        for child in tcx.module_children(module) {
            if !child.vis.is_public() || child.ident.name.as_str() == "_" {
                continue;
            }
            let Res::Def(kind, def) = child.res else {
                continue;
            };
            let path = format!("{path}::{}", child.ident);
            paths.entry(def).or_insert_with(|| path.clone());
            if kind == DefKind::Mod {
                pending.push_back((def, path));
            }
        }
    }
    paths
}

#[derive(Default)]
struct Clauses {
    ensures: bool,
    requires: Vec<String>,
}
impl Clauses {
    fn read(tcx: TyCtxt<'_>, crates: &Crates, def: LocalDefId) -> Self {
        struct Scan<'a, 'tcx> {
            tcx: TyCtxt<'tcx>,
            crates: &'a Crates,
            clauses: Clauses,
        }
        impl<'tcx> Visitor<'tcx> for Scan<'_, 'tcx> {
            type NestedFilter = nested_filter::OnlyBodies;
            fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
                self.tcx
            }
            fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
                if let ExprKind::Call(callee, args) = expr.kind
                    && let ExprKind::Path(QPath::Resolved(_, path)) = callee.kind
                    && let Res::Def(_, def) = path.res
                    && contract_expansion(self.crates, expr.span)
                {
                    let key = self.crates.key(self.tcx, def);
                    if key == "tide_std::__contract::ensures"
                        || key == "tide_std::__contract::const_ensures_marker"
                    {
                        self.clauses.ensures = true;
                    }
                    if key == "tide_std::__contract::const_requires_marker"
                        && let Some(first) = args.first()
                        && let ExprKind::Lit(lit) = first.kind
                        && let rustc_ast::LitKind::Str(text, _) = lit.node
                    {
                        self.clauses.requires.push(text.to_string());
                    }
                    if key == "tide_std::__contract::requires"
                        && let Some(first) = args.first()
                        && let ExprKind::Struct(_, fields, _) = first.kind
                    {
                        for field in fields {
                            if field.ident.name.as_str() == "text"
                                && let ExprKind::Lit(lit) = field.expr.kind
                                && let rustc_ast::LitKind::Str(text, _) = lit.node
                            {
                                self.clauses.requires.push(text.to_string());
                            }
                        }
                    }
                }
                intravisit::walk_expr(self, expr);
            }
        }
        let mut scan = Scan {
            tcx,
            crates,
            clauses: Self::default(),
        };
        scan.visit_body(tcx.hir_body_owned_by(def));
        scan.clauses
    }
}
struct Generator<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    arbitrary: DefId,
    crates: &'a Crates,
    paths: &'a FxHashMap<DefId, String>,
    module: LocalModId,
}
impl<'tcx> Generator<'_, 'tcx> {
    fn holds(&self, predicate: ty::Predicate<'tcx>) -> bool {
        let infcx = self
            .tcx
            .infer_ctxt()
            .build(ty::TypingMode::non_body_analysis());
        infcx.predicate_must_hold_modulo_regions(&Obligation::new(
            self.tcx,
            ObligationCause::dummy(),
            ty::ParamEnv::empty(),
            predicate,
        ))
    }
    fn arbitrary(&self, ty: Ty<'tcx>) -> bool {
        self.holds(TraitRef::new(self.tcx, self.arbitrary, [ty]).upcast(self.tcx))
    }
    fn arguments(&self, def: DefId) -> Result<GenericArgsRef<'tcx>, String> {
        let tcx = self.tcx;
        let mut current = Some(def);
        while let Some(id) = current {
            if tcx
                .generics_of(id)
                .own_params
                .iter()
                .any(|p| matches!(p.kind, ty::GenericParamDefKind::Const { .. }))
            {
                return Err("const parameters cannot be instantiated".into());
            }
            current = tcx.generics_of(id).parent;
        }
        let count = tcx.generics_of(def).count();
        let type_count = (0..count)
            .filter(|&i| {
                matches!(
                    tcx.generics_of(def).param_at(i, tcx).kind,
                    ty::GenericParamDefKind::Type { .. }
                )
            })
            .count();
        let combinations = 1usize
            .checked_shl(type_count as u32)
            .ok_or("too many type parameters")?;
        let budget = combinations.min(4096);
        for choices in 0..budget {
            let mut next = type_count;
            let mut missing_string = false;
            let args = ty::GenericArgs::for_item(tcx, def, |param, _| match param.kind {
                ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
                ty::GenericParamDefKind::Type { .. } => {
                    next -= 1;
                    if choices & (1 << next) == 0 {
                        tcx.types.i64.into()
                    } else if let Some(string) = self.string_type() {
                        string.into()
                    } else {
                        missing_string = true;
                        tcx.types.i64.into()
                    }
                }
                ty::GenericParamDefKind::Const { .. } => unreachable!(),
            });
            if !missing_string
                && tcx
                    .clauses_of(def)
                    .instantiate(tcx, args)
                    .clauses
                    .iter()
                    .all(|p| self.holds(p.skip_normalization().as_predicate()))
            {
                return Ok(args);
            }
        }
        Err(if combinations > budget {
            "generic instantiation search budget exceeded"
        } else {
            "neither i64 nor String meets the bounds"
        }
        .into())
    }
    fn string_type(&self) -> Option<Ty<'tcx>> {
        self.tcx.lang_items().string().map(|id| {
            self.tcx
                .type_of(id)
                .instantiate_identity()
                .skip_normalization()
        })
    }
    fn text(&self, ty: Ty<'tcx>) -> String {
        match *ty.kind() {
            Adt(adt, args) => {
                let args = self
                    .tcx
                    .generics_of(adt.did())
                    .own_args_no_defaults(self.tcx, args)
                    .iter()
                    .filter_map(|arg| match arg.kind() {
                        ty::GenericArgKind::Type(ty) => Some(self.text(ty)),
                        ty::GenericArgKind::Const(value) => Some(match value.kind() {
                            ty::ConstKind::Param(_) => value.to_string(),
                            ty::ConstKind::Value(v) if v.ty.is_integral() => {
                                let scalar = v.try_to_leaf().unwrap();
                                if v.ty.is_signed() {
                                    scalar.to_int(scalar.size()).to_string()
                                } else {
                                    scalar.to_bits(scalar.size()).to_string()
                                }
                            }
                            ty::ConstKind::Value(v) if v.ty.is_bool() || v.ty.is_char() => {
                                value.to_string()
                            }
                            _ => format!("{{ {value} }}"),
                        }),
                        ty::GenericArgKind::Lifetime(_) => None,
                    })
                    .collect::<Vec<_>>();
                let path = self.path(adt.did());
                if args.is_empty() {
                    path
                } else {
                    format!("{path}<{}>", args.join(","))
                }
            }
            Ref(_, referent, mutability) => format!(
                "&{}{}",
                if mutability.is_mut() { "mut " } else { "" },
                self.text(referent)
            ),
            Slice(element) => format!("[{}]", self.text(element)),
            Array(element, length) => format!("[{}; {length}]", self.text(element)),
            Tuple(elements) => {
                let elements = elements.iter().map(|ty| self.text(ty)).collect::<Vec<_>>();
                if elements.is_empty() {
                    "()".into()
                } else {
                    format!("({},)", elements.join(","))
                }
            }
            _ => {
                with_no_visible_paths!(with_no_trimmed_paths!(with_crate_prefix!(format!("{ty}"))))
            }
        }
    }
    fn path(&self, def: DefId) -> String {
        if def.is_local() {
            with_no_visible_paths!(with_no_trimmed_paths!(with_crate_prefix!(
                self.tcx.def_path_str(def)
            )))
        } else {
            self.paths
                .get(&def)
                .cloned()
                .unwrap_or_else(|| self.tcx.def_path_str(def))
        }
    }
    fn reachable(&self, mut def: DefId) -> bool {
        if !def.is_local() {
            return self.paths.contains_key(&def);
        }
        while let Some(parent) = self.tcx.opt_parent(def) {
            if !matches!(self.tcx.def_kind(parent), DefKind::Mod)
                || !self
                    .tcx
                    .visibility(def)
                    .is_accessible_from(self.module.to_def_id(), self.tcx)
            {
                return false;
            }
            def = parent;
        }
        true
    }
    fn nameable(&self, ty: Ty<'tcx>) -> Result<(), String> {
        match *ty.kind() {
            Adt(adt, args) => {
                if !self.reachable(adt.did()) {
                    return Err("type not nameable from the generated module".into());
                }
                for arg in self
                    .tcx
                    .generics_of(adt.did())
                    .own_args_no_defaults(self.tcx, args)
                {
                    if let Some(ty) = arg.as_type() {
                        self.nameable(ty)?;
                    }
                }
            }
            Ref(_, referent, _) | Slice(referent) | Array(referent, _) => {
                self.nameable(referent)?
            }
            Tuple(elements) => {
                for ty in elements {
                    self.nameable(ty)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn signature(&self, def: DefId, args: GenericArgsRef<'tcx>) -> ty::FnSig<'tcx> {
        let sig = self
            .tcx
            .fn_sig(def)
            .instantiate(self.tcx, args)
            .skip_normalization();
        let sig = self.tcx.instantiate_bound_regions_with_erased(sig);
        self.tcx
            .try_normalize_erasing_regions(TypingEnv::fully_monomorphized(), Unnormalized::new(sig))
            .unwrap_or(sig)
    }
    fn temporary_inputs(&self, def: DefId) -> Result<ty::FnSig<'tcx>, String> {
        // Erasure is safe only for independent, function-bound outer borrows.
        let signature = self
            .tcx
            .fn_sig(def)
            .instantiate_identity()
            .skip_normalization();
        let signature = signature.skip_binder();
        for &input in signature.inputs() {
            let supported = match *input.kind() {
                Ref(region, referent, _) => {
                    matches!(region.kind(), ty::ReBound(..)) && !referent.has_regions()
                }
                _ => !input.has_regions(),
            };
            if !supported {
                return Err("input lifetime requirements cannot be satisfied by generated temporary storage".into());
            }
        }
        Ok(signature)
    }
    fn temporary_shape(declared: ty::FnSig<'tcx>, resolved: ty::FnSig<'tcx>) -> bool {
        declared
            .inputs()
            .iter()
            .zip(resolved.inputs())
            .all(|(before, after)| match *after.kind() {
                Ref(_, referent, _) => matches!(before.kind(), Ref(..)) && !referent.has_regions(),
                _ => !after.has_regions(),
            })
    }
    fn test(
        &self,
        def: LocalDefId,
        owner: Option<LocalDefId>,
        clauses: &Clauses,
    ) -> Result<String, String> {
        let tcx = self.tcx;
        let parent = tcx.local_parent(def);
        if self.crates.is_main(def.to_def_id())
            || (self.crates.is_main(parent.to_def_id())
                && tcx.item_name(def.to_def_id()).as_str() == "__tide_main")
        {
            return Err("entry point".into());
        }
        if matches!(tcx.def_kind(parent), DefKind::Trait)
            || owner.is_some_and(|o| matches!(tcx.def_kind(o), DefKind::Impl { of_trait: true }))
        {
            return Err("trait method".into());
        }
        let container = owner.map_or(parent, |owner| tcx.local_parent(owner));
        if !matches!(tcx.def_kind(container), DefKind::Mod) {
            return Err("body-local function".into());
        }
        if owner.is_none() && !self.reachable(def.to_def_id()) {
            return Err("type not nameable from the generated module".into());
        }
        let declared = self.temporary_inputs(def.to_def_id())?;
        let args = self.arguments(def.to_def_id())?;
        let sig = self.signature(def.to_def_id(), args);
        if !Self::temporary_shape(declared, sig) {
            return Err("projected input lifetime requirements cannot be satisfied by generated temporary storage".into());
        }
        if sig.safety().is_unsafe() {
            return Err("unsafe function".into());
        }
        if sig.c_variadic() {
            return Err("variadic function".into());
        }
        let body = tcx.hir_body_owned_by(def);
        let self_ty = owner.map(|owner| {
            tcx.type_of(owner)
                .instantiate(tcx, args)
                .skip_normalization()
        });
        if let Some(ty) = self_ty {
            self.nameable(ty)?;
        }
        let mut parameters = Vec::new();
        let mut values = Vec::new();
        let mut attrs = String::new();
        for (index, arg) in args.iter().enumerate() {
            if let Some(ty) = arg.as_type() {
                let param = tcx.generics_of(def).param_at(index, tcx);
                if matches!(
                    param.kind,
                    ty::GenericParamDefKind::Type {
                        synthetic: true,
                        ..
                    }
                ) {
                    continue;
                }
                write!(
                    attrs,
                    "#[doc = {:?}] ",
                    format!("__tide_type:{}:{}", param.name, self.text(ty))
                )
                .unwrap();
            }
        }
        for (&ty, param) in sig.inputs().iter().zip(body.params) {
            self.nameable(ty)?;
            let PatKind::Binding(_, _, ident, None) = param.pat.kind else {
                return Err("destructured parameter".into());
            };
            let label = if ident.name.as_str() == "self" {
                "__tide_self".to_owned()
            } else {
                tcx.sess
                    .source_map()
                    .span_to_snippet(ident.span)
                    .unwrap_or_else(|_| ident.name.to_string())
            };
            let (owned, borrow) = match *ty.kind() {
                Ref(_, inner, mutability) => {
                    (inner, if mutability.is_mut() { "&mut " } else { "&" })
                }
                _ => (ty, ""),
            };
            let index = parameters.len();
            parameters.push(format!("{label}: {}", self.text(ty)));
            values.push(label);
            if let Some(capability) = self.capability(owned) {
                if capability == "root" {
                    return Err("Root parameter".into());
                }
                let value = format!("__tide_sim.{capability}.clone()");
                let prefix = if borrow.is_empty() {
                    "__tide_draw"
                } else {
                    "__tide_storage_draw"
                };
                write!(
                    attrs,
                    "#[doc = \"__tide_capabilities:\"] #[doc = {:?}] #[doc = {:?}] #[doc = {:?}] ",
                    format!("{prefix}:{index}:{value}"),
                    format!("__tide_storage_type:{index}:{}", self.text(owned)),
                    format!("__tide_opaque:{index}:")
                )
                .unwrap();
                continue;
            }
            let draw_ty = if matches!(owned.kind(), ty::Str) {
                self.string_type().ok_or("String unavailable")?
            } else {
                owned
            };
            let draw = if let Slice(element) = *draw_ty.kind() {
                format!(
                    "__tide_gen.collection_with(|__tide_gen| {})",
                    self.draw(element, &mut Vec::new())?
                )
            } else {
                self.draw(draw_ty, &mut Vec::new())?
            };
            let standard_slice = matches!(owned.kind(), Slice(element) if self.arbitrary(*element));
            if !self.arbitrary(draw_ty) && !standard_slice {
                let prefix = if borrow.is_empty() {
                    "__tide_draw"
                } else {
                    "__tide_storage_draw"
                };
                write!(attrs, "#[doc = {:?}] ", format!("{prefix}:{index}:{draw}")).unwrap();
                let element = match *draw_ty.kind() {
                    Slice(element) => Some(element),
                    Adt(adt, args)
                        if self.crates.is_std(adt.did())
                            && matches!(tcx.item_name(adt.did()).as_str(), "Vec" | "Option") =>
                    {
                        Some(args.type_at(0))
                    }
                    _ => None,
                };
                if let Some(element) = element {
                    write!(
                        attrs,
                        "#[doc = {:?}] ",
                        format!(
                            "__tide_element:{index}:{}",
                            self.draw(element, &mut Vec::new())?
                        )
                    )
                    .unwrap();
                }
                if !borrow.is_empty() {
                    let storage = match *owned.kind() {
                        Slice(element) => format!("::std::vec::Vec<{}>", self.text(element)),
                        _ => self.text(draw_ty),
                    };
                    write!(
                        attrs,
                        "#[doc = {:?}] ",
                        format!("__tide_storage_type:{index}:{storage}")
                    )
                    .unwrap();
                }
            }
            let debug = tcx
                .get_diagnostic_item(sym::Debug)
                .is_some_and(|debug| self.holds(TraitRef::new(tcx, debug, [owned]).upcast(tcx)));
            if !debug {
                write!(attrs, "#[doc = {:?}] ", format!("__tide_opaque:{index}:")).unwrap();
            }
        }
        let mut bounds = String::new();
        for requires in &clauses.requires {
            write!(
                bounds,
                "#[doc = {:?}] ",
                format!("__tide_requires:{requires}")
            )
            .unwrap();
        }
        for attr in tcx.hir_attrs(tcx.local_def_id_to_hir_id(def)) {
            if attr.has_name(sym::allow)
                && let Some(parts) = attr.meta_item_list()
                && parts
                    .iter()
                    .any(|part| part.has_name(Symbol::intern("vacuous")))
            {
                let reason = parts
                    .iter()
                    .find(|part| part.has_name(sym::reason))
                    .and_then(|part| part.value_str());
                let Some(reason) = reason.filter(|reason| !reason.as_str().trim().is_empty())
                else {
                    tcx.dcx()
                        .span_err(attr.span(), "allow(vacuous) needs a nonempty reason");
                    continue;
                };
                let allow = format!("#[allow(vacuous, reason = {:?})]", reason.as_str());
                write!(bounds, "#[doc = {:?}] ", format!("__tide_allow:{allow}")).unwrap();
            }
        }
        let types: Vec<_> = tcx
            .generics_of(def)
            .own_params
            .iter()
            .filter_map(|p| match p.kind {
                ty::GenericParamDefKind::Type {
                    synthetic: false, ..
                } => Some(self.text(args.type_at(p.index as usize))),
                _ => None,
            })
            .collect();
        let turbofish = if types.is_empty() {
            String::new()
        } else {
            format!("::<{}>", types.join(","))
        };
        let target = if let Some(ty) = self_ty {
            format!("<{}>::{}", self.text(ty), tcx.item_name(def.to_def_id()))
        } else {
            self.path(def.to_def_id())
        };
        let name = test_name(tcx, def);
        Ok(format!(
            "#[::std::test::test] #[doc = {:?}] {attrs} {bounds} fn r#{name}({}) {{ let _ = {target}{turbofish}({}); }}",
            format!(
                "__tide_generated_contract:{}",
                self_ty.map_or(String::new(), |ty| self.text(ty))
            ),
            parameters.join(","),
            values.join(",")
        ))
    }
    fn capability(&self, ty: Ty<'tcx>) -> Option<&'static str> {
        let Adt(adt, _) = *ty.kind() else {
            return None;
        };
        let key = self.crates.key(self.tcx, adt.did());
        match key.as_str() {
            "tide_std::Root" => Some("root"),
            "tide_std::net::Net" => Some("net"),
            "tide_std::rand::Rand" => Some("rand"),
            "tide_std::env::Env" => Some("env"),
            "tide_std::io::Stdin" => Some("stdin"),
            "tide_std::process::Proc" => Some("proc"),
            "tide_std::fs::Fs" => Some("fs"),
            "tide_std::task::Tasks" => Some("tasks"),
            "tide_std::time::Clock" => Some("clock"),
            _ => None,
        }
    }
    fn draw(&self, ty: Ty<'tcx>, stack: &mut Vec<DefId>) -> Result<String, String> {
        let tcx = self.tcx;
        self.nameable(ty)?;
        if self.capability(ty) == Some("root") {
            return Err("Root parameter".into());
        }
        if self.arbitrary(ty) {
            return Ok(format!(
                "<{} as ::std::test::Arbitrary>::arbitrary(__tide_gen)",
                self.text(ty)
            ));
        }
        match *ty.kind() {
            Tuple(elements) => {
                let fields = elements
                    .iter()
                    .map(|t| self.draw(t, stack))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("({},)", fields.join(",")))
            }
            Array(element, _) => Ok(format!(
                "::std::array::from_fn(|_| {})",
                self.draw(element, stack)?
            )),
            Adt(adt, args) if !adt.did().is_local() && self.crates.exported(adt.did()) => {
                let name = tcx.item_name(adt.did());
                let mut element = || self.draw(args.type_at(0), stack);
                match name.as_str() {
                    "Vec" => Ok(format!(
                        "__tide_gen.collection_with(|__tide_gen| {})",
                        element()?
                    )),
                    "Option" => Ok(format!(
                        "if __tide_gen.index(2) == 0 {{ None }} else {{ Some({}) }}",
                        element()?
                    )),
                    "Result" => {
                        let ok = element()?;
                        let err = self.draw(args.type_at(1), stack)?;
                        Ok(format!(
                            "if __tide_gen.index(2) == 0 {{ Ok({ok}) }} else {{ Err({err}) }}"
                        ))
                    }
                    "HashMap" | "BTreeMap" => {
                        let key = element()?;
                        let value = self.draw(args.type_at(1), stack)?;
                        Ok(format!(
                            "__tide_gen.collection_with(|__tide_gen| ({key}, {value})).into_iter().collect::<{}>()",
                            self.text(ty)
                        ))
                    }
                    "HashSet" | "BTreeSet" => Ok(format!(
                        "__tide_gen.collection_with(|__tide_gen| {}).into_iter().collect::<{}>()",
                        element()?,
                        self.text(ty)
                    )),
                    _ => Err(format!(
                        "cannot generate {} (closure, dyn, handle, or missing Arbitrary)",
                        self.text(ty)
                    )),
                }
            }
            Adt(adt, args) if adt.did().is_local() => {
                if adt.is_union() {
                    return Err("union parameter".into());
                }
                if stack.contains(&adt.did()) {
                    return Err("recursive type without Arbitrary".into());
                }
                stack.push(adt.did());
                let result = (|| {
                    if adt.all_fields().all(|f| f.vis.is_public()) {
                        let mut variants = Vec::new();
                        for variant in adt.variants() {
                            let fields = variant
                                .fields
                                .iter()
                                .map(|f| self.draw(f.ty(tcx, args).skip_normalization(), stack))
                                .collect::<Result<Vec<_>, _>>()?;
                            let path = if adt.is_enum() {
                                format!("{}::{}", self.path(adt.did()), variant.name)
                            } else {
                                self.path(adt.did())
                            };
                            variants.push(match variant.ctor_kind() {
                                Some(CtorKind::Const) => path,
                                Some(CtorKind::Fn) => format!("{path}({})", fields.join(",")),
                                None => format!(
                                    "{path} {{ {} }}",
                                    variant
                                        .fields
                                        .iter()
                                        .zip(fields)
                                        .map(|(f, v)| format!("r#{}: {v}", f.name))
                                        .collect::<Vec<_>>()
                                        .join(",")
                                ),
                            });
                        }
                        if variants.is_empty() {
                            Err("uninhabited type".into())
                        } else {
                            Ok(choice(&variants))
                        }
                    } else {
                        self.private(ty, adt, args, stack)
                    }
                })();
                stack.pop();
                result
            }
            _ => Err(format!(
                "cannot generate {} (closure, dyn, handle, or missing Arbitrary)",
                self.text(ty)
            )),
        }
    }
    fn private(
        &self,
        ty: Ty<'tcx>,
        adt: AdtDef<'tcx>,
        args: GenericArgsRef<'tcx>,
        stack: &mut Vec<DefId>,
    ) -> Result<String, String> {
        let tcx = self.tcx;
        let mut constructors = Vec::new();
        let mut methods = Vec::new();
        let mut const_constructor_requires = false;
        for &imp in tcx.inherent_impls(adt.did()).iter() {
            for item in tcx.associated_items(imp).in_definition_order() {
                if tcx.def_kind(item.def_id) != DefKind::AssocFn
                    || !tcx.visibility(item.def_id).is_public()
                {
                    continue;
                }
                let Ok(declared) = self.temporary_inputs(item.def_id) else {
                    continue;
                };
                if tcx.is_const_fn(item.def_id)
                    && !Clauses::read(tcx, self.crates, item.def_id.expect_local())
                        .requires
                        .is_empty()
                {
                    const_constructor_requires |= !item.is_method();
                    continue;
                }
                if tcx
                    .generics_of(item.def_id)
                    .own_params
                    .iter()
                    .any(|p| !matches!(p.kind, ty::GenericParamDefKind::Lifetime))
                {
                    continue;
                }
                if tcx.generics_of(imp).count() != args.len() {
                    continue;
                }
                let method_args = ty::GenericArgs::for_item(tcx, item.def_id, |param, _| {
                    if matches!(param.kind, ty::GenericParamDefKind::Lifetime) {
                        tcx.lifetimes.re_erased.into()
                    } else {
                        args[param.index as usize]
                    }
                });
                if !tcx
                    .clauses_of(item.def_id)
                    .instantiate(tcx, method_args)
                    .clauses
                    .iter()
                    .all(|p| self.holds(p.skip_normalization().as_predicate()))
                {
                    continue;
                }
                let sig = self.signature(item.def_id, method_args);
                if !Self::temporary_shape(declared, sig) {
                    continue;
                }
                if sig.safety().is_unsafe() {
                    continue;
                }
                let receiver = item.is_method();
                let is_method = receiver
                    && matches!(sig.inputs().first().map(|t| t.kind()), Some(Ref(_, inner, mutability)) if *inner == ty && mutability.is_mut());
                if !(is_method || (!receiver && sig.output() == ty)) {
                    continue;
                }
                let mut values = Vec::new();
                let mut supported = true;
                for &param in sig.inputs().iter().skip(usize::from(receiver)) {
                    match self.draw(param, stack) {
                        Ok(v) => values.push(v),
                        Err(_) => {
                            supported = false;
                            break;
                        }
                    }
                }
                if !supported {
                    continue;
                }
                let expression = if receiver {
                    format!(
                        "{{ let _ = __tide_value.{}({}); }}",
                        tcx.item_name(item.def_id),
                        values.join(",")
                    )
                } else {
                    format!(
                        "<{}>::{}({})",
                        self.text(ty),
                        tcx.item_name(item.def_id),
                        values.join(",")
                    )
                };
                let name = tcx.item_name(item.def_id);
                let call = format!("::std::test::attempt({name:?}, || {expression})");
                if receiver {
                    methods.push(format!("{{ let _ = {call}; }}"));
                } else {
                    constructors.push(call);
                }
            }
        }
        if constructors.is_empty() {
            if const_constructor_requires {
                return Err(format!(
                    "{}: const constructor requires lacks runtime rejection metadata",
                    self.text(ty)
                ));
            }
            return Err(format!(
                "{} has private fields and no generatable public constructor",
                self.text(ty)
            ));
        }
        let mutate = if methods.is_empty() {
            String::new()
        } else {
            format!("for _ in 0..__tide_gen.index(9) {{ {} }}", choice(&methods))
        };
        Ok(format!(
            "{{ let mut __tide_value = match {} {{ Some(value) => value, None => ::std::test::reject_generation() }}; {mutate} __tide_value }}",
            choice(&constructors)
        ))
    }
}
fn choice(values: &[String]) -> String {
    format!(
        "match __tide_gen.index({}) {{ {} }}",
        values.len(),
        values
            .iter()
            .enumerate()
            .map(|(i, v)| if i + 1 == values.len() {
                format!("_ => {v},")
            } else {
                format!("{i} => {v},")
            })
            .collect::<String>()
    )
}
