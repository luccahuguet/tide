//! Test-build additions planned from resolved HIR and compiled through ordinary admission.
mod contracts;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::GenericParamKind;
use rustc_hir::def::{CtorKind, DefKind, Res};
use rustc_hir::def_id::{CrateNum, DefId, LocalModId};
use rustc_middle::ty::{Adt, Array, Ref, Slice, Tuple};
use rustc_middle::ty::{
    AdtDef, TraitRef, Ty, TyCtxt, TypeVisitableExt, TypingEnv, Unnormalized, VariantDef,
};
use rustc_span::{BytePos, DUMMY_SP, ExpnKind, FileName, MacroKind, Span, Symbol, sym};

use crate::crates::Crates;
use crate::walk::{MAX_OBLIGATION_DEPTH, Selected, select, where_clauses};

#[derive(Default)]
pub struct Insertions(BTreeMap<PathBuf, BTreeMap<usize, String>>);

impl Insertions {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn apply(&self, path: &Path, source: &str) -> Option<String> {
        let edits = self.0.get(path)?;
        let mut result = source.to_owned();
        for (&offset, code) in edits.iter().rev() {
            result.insert_str(offset, code);
        }
        Some(result)
    }

    /// Only bytes inserted by this compiler run are trusted, never a source attribute.
    pub fn contains(&self, tcx: TyCtxt<'_>, span: Span) -> bool {
        let span = span.source_callsite();
        let file = tcx.sess.source_map().lookup_source_file(span.lo());
        let FileName::Real(name) = &file.name else {
            return false;
        };
        let Some(edits) = name.local_path().and_then(|path| self.0.get(path)) else {
            return false;
        };
        let lo = file.original_relative_byte_pos(span.lo()).0 as usize;
        let hi = file.original_relative_byte_pos(span.hi()).0 as usize;
        let mut shift = 0;
        for (&offset, code) in edits {
            let start = offset + shift;
            if start <= lo && hi <= start + code.len() {
                return true;
            }
            shift += code.len();
        }
        false
    }
    fn append(&mut self, tcx: TyCtxt<'_>, module: LocalModId, code: &str) {
        let (body, declaration, _) = tcx.hir_get_module(module);
        let end = body.spans.inner_span.hi();
        // Inline modules include their closing brace. Other modules end at EOF.
        let inline = !module.is_top_level_module() && declaration.hi() == end;
        let position = if inline { end - BytePos(1) } else { end };
        // HIR's printer wraps long types; keep generated code on one physical line.
        let code = code.replace('\n', " ");
        let code = if inline {
            format!(" {code} ")
        } else {
            format!("\n{code}\n")
        };
        self.insert(tcx, position, &code);
    }

    fn insert(&mut self, tcx: TyCtxt<'_>, position: BytePos, code: &str) {
        let file = tcx.sess.source_map().lookup_source_file(position);
        let FileName::Real(name) = &file.name else {
            return;
        };
        let Some(path) = name.local_path() else {
            return;
        };
        let offset = file.original_relative_byte_pos(position).0 as usize;
        let insertion = self
            .0
            .entry(path.into())
            .or_default()
            .entry(offset)
            .or_default();
        insertion.push_str(code);
    }
}

pub fn plan(tcx: TyCtxt<'_>, test_library: Option<CrateNum>) -> Insertions {
    let mut edits = Insertions::default();
    let crates = Crates::new(tcx);
    let Some(std) = crates.tide_std() else {
        return edits;
    };
    let Some(test) = child(tcx, std.as_def_id(), sym::test, DefKind::Mod) else {
        return edits;
    };
    let Some(arbitrary) = child(tcx, test, Symbol::intern("Arbitrary"), DefKind::Trait) else {
        return edits;
    };
    let mut properties = Properties {
        tcx,
        arbitrary,
        test_library,
        property: DUMMY_SP,
        visited: FxHashSet::default(),
        discovered: FxHashSet::default(),
        recursive: FxHashSet::default(),
        adts: Vec::new(),
    };
    for id in tcx.hir_free_items() {
        let def = id.owner_id.def_id;
        if tcx.def_kind(def) != DefKind::Fn {
            continue;
        }
        // Only the macro can mark the function whose arguments it draws.
        let mut simulation = false;
        let property = tcx.hir_attrs(id.hir_id()).iter().any(|attr| {
            if attr
                .doc_str()
                .is_none_or(|text| text.as_str() != "__tide_property_body")
            {
                return false;
            }
            let expansion = attr.span().ctxt().outer_expn_data();
            matches!(expansion.kind, ExpnKind::Macro(MacroKind::Attr, _))
                && expansion.macro_def_id.is_some_and(|mac| {
                    simulation = tcx.item_name(mac).as_str() == "sim_test";
                    crates.exported(mac) && (simulation || tcx.item_name(mac) == sym::test)
                })
        });
        if !property {
            continue;
        }
        // This pass sees only original source; driver metadata arrives in the second compile.
        let forged = tcx.hir_attrs(id.hir_id()).iter().any(|attr| {
            let Some(text) = attr.doc_str() else {
                return false;
            };
            if [
                "__tide_property_parameters:",
                "__tide_generated_contract:",
                "__tide_capabilities:",
                "__tide_draw:",
                "__tide_storage_draw:",
                "__tide_storage_type:",
                "__tide_element:",
                "__tide_opaque:",
                "__tide_type:",
                "__tide_requires:",
                "__tide_allow:",
            ]
            .iter()
            .any(|prefix| text.as_str().starts_with(prefix))
            {
                tcx.dcx().span_err(
                    attr.span(),
                    "reserved property metadata is supplied only by tide-admit",
                );
                true
            } else {
                false
            }
        });
        if forged {
            continue;
        }
        let sig = tcx.fn_sig(def).instantiate_identity().skip_normalization();
        let sig = tcx.instantiate_bound_regions_with_erased(sig);
        properties.property = tcx.def_span(def);
        let mut kinds = String::new();
        for (index, &ty) in sig.inputs().iter().enumerate() {
            if simulation && index == 0 {
                // The macro supplies this argument from the case's simulator.
                kinds.push('v');
                continue;
            }
            let ty = properties.normalize(ty);
            let ty = if let Ref(_, referent, mutability) = *ty.kind() {
                kinds.push(if mutability.is_mut() { 'm' } else { 'r' });
                referent
            } else {
                kinds.push('v');
                ty
            };
            properties.discover(ty, &mut Vec::new(), None, 0);
        }
        let mut result = properties.normalize(sig.output());
        let mut results = String::new();
        while let Adt(adt, args) = *result.kind()
            && tcx.is_diagnostic_item(sym::Result, adt.did())
        {
            results.push('r');
            result = properties.normalize(args.type_at(0));
        }
        if kinds.contains(['r', 'm']) || !results.is_empty() {
            edits.insert(
                tcx,
                tcx.def_span(def).lo(),
                &format!("#[doc = \"__tide_property_parameters:{kinds}:{results}\"] "),
            );
        }
    }
    contracts::plan(tcx, &crates, arbitrary, &mut edits);
    for &adt in &properties.adts {
        let module = LocalModId::new_unchecked(tcx.local_parent(adt.did().expect_local()));
        edits.append(tcx, module, &properties.implementation(adt));
    }
    edits
}

/// The child of `module` named `name` whose kind is `kind`: a name can stand for items of
/// different kinds (a `fn task` beside a `mod task`).
pub(crate) fn child(tcx: TyCtxt<'_>, module: DefId, name: Symbol, kind: DefKind) -> Option<DefId> {
    tcx.module_children(module)
        .iter()
        .find_map(|child| match child.res {
            Res::Def(found, id) if found == kind && child.ident.name == name => Some(id),
            _ => None,
        })
}

struct Properties<'tcx> {
    tcx: TyCtxt<'tcx>,
    arbitrary: DefId,
    test_library: Option<CrateNum>,
    /// The property whose parameter types are being followed, for diagnostics.
    property: Span,
    visited: FxHashSet<(TraitRef<'tcx>, Option<DefId>)>,
    discovered: FxHashSet<DefId>,
    recursive: FxHashSet<DefId>,
    adts: Vec<AdtDef<'tcx>>,
}

impl<'tcx> Properties<'tcx> {
    fn normalize(&self, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.tcx
            .try_normalize_erasing_regions(TypingEnv::fully_monomorphized(), Unnormalized::new(ty))
            .unwrap_or(ty)
    }

    fn discover(
        &mut self,
        ty: Ty<'tcx>,
        stack: &mut Vec<DefId>,
        field: Option<(DefId, DefId)>,
        depth: usize,
    ) {
        self.requirement(
            TraitRef::new(self.tcx, self.arbitrary, [ty]),
            stack,
            field,
            depth,
        );
    }

    fn requirement(
        &mut self,
        tr: TraitRef<'tcx>,
        stack: &mut Vec<DefId>,
        field: Option<(DefId, DefId)>,
        depth: usize,
    ) {
        if depth > MAX_OBLIGATION_DEPTH {
            self.tcx.dcx().span_err(
                field.map_or(self.property, |(_, field)| self.tcx.def_span(field)),
                "property generation requirements nest too deeply",
            );
            return;
        }
        let tr = self
            .tcx
            .try_normalize_erasing_regions(TypingEnv::fully_monomorphized(), Unnormalized::new(tr))
            .unwrap_or(tr);
        let ty = tr.self_ty();
        if tr.def_id == self.arbitrary
            && let Adt(adt, _) = *ty.kind()
            && let Some(index) = stack.iter().position(|&id| id == adt.did())
        {
            self.recursive.extend(&stack[index..]);
            return;
        }
        if !self.visited.insert((tr, field.map(|(_, field)| field))) {
            return;
        }
        let requirements = match select(self.tcx, TypingEnv::fully_monomorphized(), tr) {
            Some(Selected::Impl(id, args)) => Some(where_clauses(self.tcx, id, args)),
            Some(Selected::Builtin(nested)) => Some(nested),
            None => None,
        };
        if let Some(requirements) = requirements {
            for requirement in requirements {
                self.requirement(requirement, stack, field, depth + 1);
            }
            return;
        }
        if tr.def_id != self.arbitrary {
            return;
        }
        match *ty.kind() {
            Array(element, _) => self.discover(element, stack, field, depth),
            Slice(element) if field.is_none() => self.discover(element, stack, field, depth),
            Tuple(elements) => {
                for element in elements {
                    self.discover(element, stack, field, depth);
                }
            }
            Adt(adt, args) => {
                let has_impl = self
                    .tcx
                    .non_blanket_impls_for_ty(self.arbitrary, ty)
                    .next()
                    .is_some();
                let Some(local) = adt.did().as_local() else {
                    if !has_impl && Some(adt.did().krate) == self.test_library {
                        // Only the defining crate may implement the trait for its type, and
                        // the paired library is built before its suite's properties are known.
                        self.tcx.dcx().span_err(
                            self.property,
                            format!(
                                "this property generates `{}`, which the paired library defines \
                                 without implementing `std::test::Arbitrary`; a suite cannot \
                                 generate a library type unless the library implements it",
                                self.tcx.def_path_str(adt.did())
                            ),
                        );
                    } else {
                        self.cannot_generate(ty, field);
                    }
                    return;
                };
                if adt.is_union()
                    || adt.variants().is_empty()
                    || self.tcx.def_kind(self.tcx.local_parent(local)) != DefKind::Mod
                    || has_impl
                {
                    self.cannot_generate(ty, field);
                    return;
                }
                if self.discovered.insert(adt.did()) {
                    self.adts.push(adt);
                }
                stack.push(adt.did());
                for field in adt.all_fields() {
                    self.discover(
                        field.ty(self.tcx, args).skip_normalization(),
                        stack,
                        Some((adt.did(), field.did)),
                        0,
                    );
                }
                stack.pop();
            }
            rustc_middle::ty::Str if field.is_none() => {}
            _ => self.cannot_generate(ty, field),
        }
    }

    fn cannot_generate(&self, ty: Ty<'tcx>, field: Option<(DefId, DefId)>) {
        if let Some((adt, field)) = field
            && !ty.has_param()
        {
            self.tcx.dcx().span_err(
                self.tcx.def_span(field),
                format!(
                    "field `{}` of `{}` has type `{ty}`, which cannot be generated; \
                     implement `std::test::Arbitrary` for `{}`",
                    self.tcx.item_name(field),
                    self.tcx.item_name(adt),
                    self.tcx.item_name(adt)
                ),
            );
        }
    }

    fn implementation(&self, adt: AdtDef<'tcx>) -> String {
        let tcx = self.tcx;
        let id = adt.did().expect_local();
        let name = tcx.item_name(adt.did());
        let generics = tcx.hir_get_generics(id).unwrap();
        let mut parameters = Vec::new();
        let mut arguments = Vec::new();
        let mut bounds: Vec<_> = generics
            .predicates
            .iter()
            .map(|predicate| rustc_hir_pretty::id_to_string(&tcx, predicate.hir_id))
            .collect();
        // A DFS cross edge may reach a recursive type without marking this ADT itself.
        let recursive_bounds = self.recursive.contains(&adt.did())
            || adt.all_fields().any(|field| {
                let ty = tcx
                    .type_of(field.did)
                    .instantiate_identity()
                    .skip_normalization();
                ty.has_param() && self.mentions_recursive(ty)
            });
        for parameter in generics.params {
            let name = parameter.name.ident().name;
            let argument = if parameter.is_lifetime() {
                name.to_string()
            } else {
                format!("r#{name}")
            };
            let declaration = match parameter.kind {
                GenericParamKind::Const { ty, .. } => {
                    format!(
                        "const {argument}: {}",
                        rustc_hir_pretty::id_to_string(&tcx, ty.hir_id)
                    )
                }
                GenericParamKind::Type { .. } if recursive_bounds => {
                    bounds.push(format!("{argument}: ::std::test::Arbitrary"));
                    argument.clone()
                }
                _ => argument.clone(),
            };
            parameters.push(declaration);
            arguments.push(argument);
        }
        for field in adt.all_fields() {
            let ty = tcx
                .type_of(field.did)
                .instantiate_identity()
                .skip_normalization();
            if ty.has_param() && !self.mentions_recursive(ty) {
                let rustc_hir::Node::Field(field) =
                    tcx.hir_node_by_def_id(field.did.expect_local())
                else {
                    unreachable!("ADT field has no field declaration");
                };
                let ty = rustc_hir_pretty::id_to_string(&tcx, field.ty.hir_id);
                bounds.push(format!("{ty}: ::std::test::Arbitrary"));
            }
        }

        let mut code = String::from("impl");
        if !parameters.is_empty() {
            write!(code, "<{}>", parameters.join(", ")).unwrap();
        }
        write!(code, " ::std::test::Arbitrary for r#{name}").unwrap();
        if !arguments.is_empty() {
            write!(code, "<{}>", arguments.join(", ")).unwrap();
        }
        if !bounds.is_empty() {
            write!(code, " where {}", bounds.join(", ")).unwrap();
        }
        code.push_str(" { fn arbitrary(__tide_gen: &mut ::std::test::Gen) -> Self { ");
        if adt.is_enum() {
            write!(code, "match __tide_gen.index({}) {{ ", adt.variants().len()).unwrap();
            for (index, variant) in adt.variants().iter().enumerate() {
                if index + 1 == adt.variants().len() {
                    code.push_str("_ => ");
                } else {
                    write!(code, "{index} => ").unwrap();
                }
                write!(code, "Self::r#{}", variant.name).unwrap();
                constructor(&mut code, variant);
                code.push_str(", ");
            }
            code.push_str("} ");
        } else {
            code.push_str("Self");
            constructor(&mut code, adt.non_enum_variant());
        }
        code.push_str(" } }");
        code
    }

    fn mentions_recursive(&self, ty: Ty<'tcx>) -> bool {
        ty.walk()
            .filter_map(|arg| arg.as_type())
            .any(|ty| matches!(ty.kind(), Adt(inner, _) if self.recursive.contains(&inner.did())))
    }
}

fn constructor(code: &mut String, variant: &VariantDef) {
    let draw = "::std::test::Arbitrary::arbitrary(__tide_gen)";
    match variant.ctor_kind() {
        Some(CtorKind::Const) => (),
        Some(CtorKind::Fn) => {
            code.push('(');
            for _ in &variant.fields {
                write!(code, "{draw}, ").unwrap();
            }
            code.push(')');
        }
        None => {
            code.push_str(" { ");
            for field in &variant.fields {
                write!(code, "r#{}: {draw}, ", field.name).unwrap();
            }
            code.push('}');
        }
    }
}
