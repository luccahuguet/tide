//! The admission walk (ADMIT-1): every item the local crate uses, as rustc resolved it, checked
//! against the allowlist. It covers paths, type-relative paths, method calls, overloaded
//! operators and derefs, the types of expressions and patterns, impls selected for the trait
//! obligations that a use, a type's where-clauses, an autoderef, an unsizing coercion to
//! `dyn Trait`, a local trait impl, or the entry point's `Termination` incurs (and their
//! where-clauses, or a builtin impl's obligations on a type's parts), function pointers' integer
//! casts and builtin comparisons, and macro expansions, including ones that land in attributes.

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, HirId};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::adjustment::{Adjust, DerefAdjustKind, PointerCoercion};
use rustc_middle::ty::print::{
    PrintTraitRefExt, with_no_trimmed_paths, with_no_visible_paths, with_resolve_crate_name,
};
use rustc_middle::ty::{
    self, GenericArgsRef, Ty, TyCtxt, TypeVisitableExt, TypingEnv, Unnormalized,
};
use rustc_span::hygiene::LocalExpnId;
use rustc_span::{ExpnKind, MacroKind, Span, kw, sym};
use rustc_trait_selection::traits::{ImplSource, Obligation, ObligationCause, SelectionContext};

use crate::crates::{Crates, Role};
use crate::diag::{Code, Diagnostics};
use crate::policy::{self, Rule, Verdict};

/// What the package's manifest grants it (03 RUN-4).
#[derive(Clone, Copy, Default)]
pub struct Grants {
    pub unsafe_: bool,
}

/// Checks the local crate and emits its diagnostics.
pub fn check_crate(tcx: TyCtxt<'_>, grants: Grants) {
    let crates = Crates::new(tcx);
    let mut walk = Walk {
        tcx,
        crates: &crates,
        grants,
        diags: Diagnostics::default(),
        typeck: None,
        owner: None,
        seen_obligations: FxHashSet::default(),
    };
    tcx.hir_walk_toplevel_module(&mut walk);
    walk.entry_point();
    walk.check_expansions();
    walk.diags.emit(tcx);
}

/// Why a use is not admitted.
struct Rejection {
    key: String,
    help: Option<&'static str>,
}

/// Obligations are followed through impl where-clauses at most this deep; a deeper chain is
/// rejected, since what it would select is unchecked.
const MAX_OBLIGATION_DEPTH: usize = 64;

struct Walk<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    crates: &'a Crates,
    grants: Grants,
    diags: Diagnostics,
    typeck: Option<&'tcx ty::TypeckResults<'tcx>>,
    owner: Option<LocalDefId>,
    seen_obligations: FxHashSet<(ty::TraitRef<'tcx>, Option<LocalDefId>)>,
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    // ------------------------------------------------------------ verdicts

    /// The allowlist key of an item: its canonical path in its defining crate, with Tide's std
    /// printed as `tide_std` whatever its crate is named.
    fn key(&self, def_id: DefId) -> String {
        let path = with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(
            self.tcx.def_path_str(def_id)
        )));
        if self.crates.role(def_id.krate) == Role::TideStd {
            let name = self.tcx.crate_name(def_id.krate);
            if let Some(rest) = path
                .strip_prefix(name.as_str())
                .and_then(|p| p.strip_prefix("::"))
            {
                return format!("tide_std::{rest}");
            }
        }
        path
    }

    fn decide(
        &self,
        key: String,
        rule: Option<&'static Rule>,
        unlisted_admitted: bool,
    ) -> Result<(), Rejection> {
        match rule.map(|r| r.verdict) {
            Some(Verdict::Allow) => Ok(()),
            Some(Verdict::Unsafe) if self.grants.unsafe_ => Ok(()),
            None if unlisted_admitted => Ok(()),
            _ => Err(Rejection {
                key,
                help: rule.and_then(|r| r.help),
            }),
        }
    }

    /// Items of the local crate are admitted, as is whatever Tide's std re-exports and the
    /// items of the traits it re-exports; Tide's std's own items unless a rule says otherwise;
    /// `core`, `alloc`, and Rust's std only where a rule allows; any other crate's items never.
    fn admit_def(&self, def_id: DefId) -> Result<(), Rejection> {
        let role = self.crates.role(def_id.krate);
        let exported = |d| self.crates.exported(d);
        if role == Role::Local
            || exported(def_id)
            || self.tcx.trait_of_assoc(def_id).is_some_and(exported)
        {
            return Ok(());
        }
        let key = self.key(def_id);
        match role {
            Role::Other => Err(Rejection { key, help: None }),
            _ => self.decide(key.clone(), policy::lookup(&key), role == Role::TideStd),
        }
    }

    /// An impl is admitted when a rule allows its key, or else when it comes from Tide's std or
    /// both the trait and the self type's head are admitted.
    fn admit_impl(&self, impl_def_id: DefId, used_as: ty::TraitRef<'tcx>) -> Result<(), Rejection> {
        let tcx = self.tcx;
        let role = self.crates.role(impl_def_id.krate);
        if role == Role::Local {
            return Ok(());
        }
        let generic = tcx
            .impl_trait_ref(impl_def_id)
            .instantiate_identity()
            .skip_normalization();
        // core's impls for every function pointer (`impl<F: FnPtr> Hash for F`) are keyed
        // `for fn()`, so the rules can tell them from other blanket impls.
        let fn_ptr = tcx.lang_items().fn_ptr_trait();
        let for_fn_ptrs = tcx
            .clauses_of(impl_def_id)
            .instantiate_identity(tcx)
            .clauses
            .into_iter()
            .any(|c| {
                c.skip_normalization()
                    .as_trait_clause()
                    .is_some_and(|tp| Some(tp.def_id()) == fn_ptr)
            });
        let key =
            with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(format!(
                "impl {} for {}",
                generic.print_only_trait_path(),
                if for_fn_ptrs {
                    "fn()".to_string()
                } else {
                    generic.self_ty().to_string()
                }
            ))));
        if let Some(rule) = policy::lookup(&key) {
            return self.decide(key, Some(rule), false);
        }
        let head = match *used_as.self_ty().kind() {
            ty::Adt(def, _) => self.admit_def(def.did()),
            ty::Foreign(def) => self.admit_def(def),
            ty::RawPtr(_, m) => self.admit_pseudo(raw_pointer_key(m)),
            _ => Ok(()),
        };
        if role == Role::TideStd || (self.admit_def(used_as.def_id).is_ok() && head.is_ok()) {
            Ok(())
        } else {
            Err(Rejection { key, help: None })
        }
    }

    fn admit_pseudo(&self, key: &str) -> Result<(), Rejection> {
        self.decide(key.to_string(), policy::lookup(key), false)
    }

    /// A function pointer's address observed without an impl: a cast to an integer, or a
    /// builtin ordering comparison, which rustc does not route through `PartialOrd`.
    fn fn_pointer_address(&mut self, e: &hir::Expr<'tcx>, t: &ty::TypeckResults<'tcx>) {
        let is_fn = |e: &hir::Expr<'_>| {
            matches!(
                t.expr_ty_adjusted_opt(e).map(|ty| ty.kind()),
                Some(ty::FnDef(..) | ty::FnPtr(..))
            )
        };
        let key = match e.kind {
            hir::ExprKind::Cast(src, _)
                if is_fn(src) && t.node_type_opt(e.hir_id).is_some_and(|ty| ty.is_integral()) =>
            {
                "fn() as integer"
            }
            hir::ExprKind::Binary(op, l, _)
                if matches!(
                    op.node,
                    hir::BinOpKind::Lt
                        | hir::BinOpKind::Le
                        | hir::BinOpKind::Gt
                        | hir::BinOpKind::Ge
                ) && is_fn(l) =>
            {
                "impl core::cmp::PartialOrd for fn()"
            }
            _ => return,
        };
        self.check(e.span, self.admit_pseudo(key));
    }

    /// Whether `span` is in code the toolchain generates: the harness's `main` under `--test`,
    /// the descriptors of `core`'s `#[test]`, which use the sysroot's `test` crate, and the
    /// entry point that Tide's std's `#[tide::main]` generates, which starts the runtime.
    fn generated_by_toolchain(&self, mut span: Span) -> bool {
        while span.from_expansion() {
            let data = span.ctxt().outer_expn_data();
            let toolchain_attr = matches!(data.kind, ExpnKind::Macro(MacroKind::Attr, _))
                && data.macro_def_id.is_some_and(|d| {
                    let name = self.tcx.item_name(d);
                    (self.crates.role(d.krate) == Role::Core && name == sym::test)
                        || (self.crates.exported(d) && name == sym::main)
                });
            if toolchain_attr || matches!(data.kind, ExpnKind::AstPass(_)) {
                return true;
            }
            span = data.call_site;
        }
        false
    }

    /// The typeck results that cover `id`: none in the signature of an item nested in a body.
    fn typeck_of(&self, id: HirId) -> Option<&'tcx ty::TypeckResults<'tcx>> {
        self.typeck.filter(|t| t.hir_owner == id.owner)
    }

    // ------------------------------------------------------------ uses

    fn reject(&mut self, span: Span, rejection: Rejection, note: &str) {
        // Report at the outermost call site, so an expansion points at the user's code.
        let span = span.source_callsite();
        let message = format!("`{}` is not admitted{note}", rejection.key);
        self.diags
            .report(self.tcx, span, Code::Fx0520, message, rejection.help);
    }

    fn check(&mut self, span: Span, verdict: Result<(), Rejection>) {
        if let Err(r) = verdict {
            if !self.generated_by_toolchain(span) {
                self.reject(span, r, "");
            }
        }
    }

    fn use_def(&mut self, def_id: DefId, span: Span) {
        // Naming a module grants nothing; the items used through it are checked.
        if self.tcx.def_kind(def_id) != DefKind::Mod {
            self.check(span, self.admit_def(def_id));
        }
    }

    /// Every item a type names: ADTs, fn items, foreign types, aliases, trait objects, and raw
    /// pointers, whose ordering, hashing, and formatting observe addresses. The where-clauses of
    /// an ADT, which are all a `Drop` impl's bounds can be, and of a fn item are obligations too:
    /// a constructor passed as a function (`.map(Logged)`) builds values no expression's type
    /// shows.
    fn use_ty(&mut self, ty: Ty<'tcx>, span: Span) {
        for arg in ty.walk() {
            let Some(t) = arg.as_type() else { continue };
            match *t.kind() {
                ty::Adt(def, args) => {
                    self.use_def(def.did(), span);
                    self.use_generic(def.did(), args, span);
                }
                ty::FnDef(def, args) => {
                    self.use_def(def, span);
                    self.use_generic(
                        def,
                        self.tcx.instantiate_bound_regions_with_erased(args),
                        span,
                    );
                }
                ty::Foreign(def) => self.use_def(def, span),
                ty::RawPtr(_, m) => self.check(span, self.admit_pseudo(raw_pointer_key(m))),
                ty::Alias(_, alias) => {
                    let def_id: DefId = match alias.kind {
                        ty::AliasTyKind::Projection { def_id } => def_id.into(),
                        ty::AliasTyKind::Inherent { def_id } => def_id.into(),
                        ty::AliasTyKind::Opaque { def_id } => def_id.into(),
                        ty::AliasTyKind::Free { def_id } => def_id.into(),
                    };
                    self.use_def(def_id, span)
                }
                ty::Dynamic(preds, _) => {
                    for p in preds.iter() {
                        self.use_def(p.def_id(), span)
                    }
                }
                _ => {}
            }
        }
    }

    fn typing_env(&self) -> TypingEnv<'tcx> {
        match self.owner {
            Some(o) => TypingEnv::post_analysis(self.tcx, o.to_def_id()),
            None => TypingEnv::fully_monomorphized(),
        }
    }

    /// The trait obligations a use of `def_id` at `args` incurs: its trait, if it is a trait
    /// item, and its where-clauses.
    fn use_generic(&mut self, def_id: DefId, args: GenericArgsRef<'tcx>, span: Span) {
        let tcx = self.tcx;
        let mut trait_refs = Vec::new();
        if let Some(tr) = tcx.trait_of_assoc(def_id) {
            let n = tcx.generics_of(tr).count();
            if args.len() >= n {
                trait_refs.push(ty::TraitRef::new(tcx, tr, args.iter().take(n)));
            }
        }
        if tcx.def_kind(def_id) != DefKind::Field {
            trait_refs.extend(where_clauses(tcx, def_id, args));
        }
        for tr in trait_refs {
            self.obligation(tr, span, 0);
        }
    }

    /// What rustc proves where a local trait impl is written, which a use of the impl relies on
    /// but selecting it does not show: the trait's where-clauses and supertraits (a `dyn Error`
    /// formats with `E: Debug`) and the bounds of its associated types.
    fn trait_impl(&mut self, impl_id: LocalDefId) {
        let tcx = self.tcx;
        let tr = tcx
            .impl_trait_ref(impl_id)
            .instantiate_identity()
            .skip_normalization();
        let outer = self.owner.replace(impl_id);
        let span = tcx.def_span(impl_id);
        for t in where_clauses(tcx, tr.def_id, tr.args) {
            self.obligation(t, span, 0);
        }
        let types = tcx.associated_items(impl_id).in_definition_order();
        for item in types.filter(|i| i.is_type()) {
            let (Some(trait_item), Some(local)) =
                (item.trait_item_def_id(), item.def_id.as_local())
            else {
                continue;
            };
            // As rustc's `check_type_bounds` does, with the impl's types for the trait's.
            let args = ty::GenericArgs::identity_for_item(tcx, item.def_id).rebase_onto(
                tcx,
                impl_id.to_def_id(),
                tr.args,
            );
            self.owner = Some(local);
            let span = tcx.def_span(item.def_id);
            for bound in tcx
                .explicit_item_bounds(trait_item)
                .iter_instantiated_copied(tcx, args)
            {
                if let Some(tp) = bound.skip_normalization().0.as_trait_clause() {
                    let t = tcx.instantiate_bound_regions_with_erased(tp).trait_ref;
                    self.obligation(t, span, 0);
                }
            }
        }
        self.owner = outer;
    }

    /// Checks the impl selected for `tr`, then the obligations of its where-clauses, or those of
    /// the builtin impl that meets `tr`. It checks nothing where rustc reports an error, where
    /// `tr` is met by a where-clause of the caller (checked where the caller is used), by an
    /// object candidate, or by an auto trait's builtin impl.
    fn obligation(&mut self, tr: ty::TraitRef<'tcx>, span: Span, depth: usize) {
        let tcx = self.tcx;
        if depth > MAX_OBLIGATION_DEPTH {
            let key = with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(
                format!("{}: {}", tr.self_ty(), tr.print_only_trait_path())
            )));
            let rejection = Rejection { key, help: None };
            self.reject(span, rejection, " (its obligations nest too deep to check)");
            return;
        }
        if tr.references_error() || tr.has_non_region_infer() || tr.has_escaping_bound_vars() {
            return;
        }
        let env = self.typing_env();
        let tr = tcx.erase_and_anonymize_regions(tr);
        let Ok(tr) = tcx.try_normalize_erasing_regions(env, Unnormalized::new(tr)) else {
            return;
        };
        if !self.seen_obligations.insert((tr, self.owner)) {
            return;
        }
        let nested = match select(tcx, env, tr) {
            Some(Selected::Impl(impl_def_id, impl_args)) => {
                // Checked under the toolchain's expansions too, since the user's types choose
                // the impl: `fn main() -> Result<(), fn()>` makes `#[tide::main]`'s
                // `Termination::report` select `fn(): Debug`.
                if let Err(r) = self.admit_impl(impl_def_id, tr) {
                    self.reject(span, r, "");
                }
                where_clauses(tcx, impl_def_id, impl_args)
            }
            Some(Selected::Builtin(nested)) => nested,
            None => return,
        };
        for tr in nested {
            self.obligation(tr, span, depth + 1);
        }
    }

    /// rustc's entry point reports what a user's `main` returns through its `Termination` impl,
    /// which no expression shows: `fn main() -> Result<(), fn()>` formats the error. A `main`
    /// the toolchain generates returns a type it fixes (`()` for the test harness).
    fn entry_point(&mut self) {
        let tcx = self.tcx;
        let (Some((main, _)), Some(termination)) =
            (tcx.entry_fn(()), tcx.lang_items().termination())
        else {
            return;
        };
        let Some(local) = main.as_local() else { return };
        if self.generated_by_toolchain(tcx.def_span(main)) {
            return;
        }
        let sig = tcx.fn_sig(main).instantiate_identity().skip_normalization();
        let ret = tcx.instantiate_bound_regions_with_erased(sig.output());
        let outer = self.owner.replace(local);
        let tr = ty::TraitRef::new(tcx, termination, [ret]);
        self.obligation(tr, tcx.def_span(main), 0);
        self.owner = outer;
    }

    /// Every macro expansion in the crate, by the macro's definition, including expansions whose
    /// output lands in attributes (`#[doc = include_str!(..)]`).
    fn check_expansions(&mut self) {
        // rustc has no public iterator over a crate's expansions; ids are dense, so a fresh
        // empty one bounds them.
        let end = LocalExpnId::fresh_empty().as_u32();
        for i in 1..end {
            let data = LocalExpnId::from_u32(i).expn_data();
            if let (ExpnKind::Macro(..), Some(mac)) = (&data.kind, data.macro_def_id) {
                self.use_def(mac, data.call_site);
            }
        }
    }

    /// A path that did not resolve because Tide's std does not export it, such as
    /// `std::sync::LazyLock`, is reported when the rules deny the item Rust's std has there.
    fn unresolved(&mut self, path: &hir::Path<'tcx>) {
        let tcx = self.tcx;
        let (Some(rust_std), Some(tide_std)) = (self.crates.rust_std(), self.crates.tide_std())
        else {
            return;
        };
        let mut names = path
            .segments
            .iter()
            .map(|s| s.ident.name)
            .skip_while(|n| *n == kw::PathRoot);
        if names.next() != Some(sym::std) {
            return;
        }
        // The same module in both crates, while Tide's std has it.
        let (mut rust, mut tide) = (rust_std.as_def_id(), Some(tide_std.as_def_id()));
        for name in names {
            let children = move |m: DefId| {
                tcx.module_children(m)
                    .iter()
                    .filter(move |c| c.ident.name == name)
            };
            if tide.is_some_and(|m| children(m).next().is_none()) {
                tide = None;
            }
            let mut next = None;
            for child in children(rust) {
                match child.res {
                    Res::Def(DefKind::Mod, def_id) => next = Some(def_id),
                    Res::Def(_, def_id) if tide.is_none() => {
                        let key = self.key(def_id);
                        if let Some(rule) =
                            policy::lookup(&key).filter(|r| r.verdict == Verdict::Deny)
                        {
                            let rejection = Rejection {
                                key,
                                help: rule.help,
                            };
                            self.reject(path.span, rejection, " (Tide's std does not export it)");
                        }
                    }
                    _ => {}
                }
            }
            let Some(next) = next else { return };
            rust = next;
            tide = tide.and_then(|m| {
                children(m).find_map(|c| match c.res {
                    Res::Def(DefKind::Mod, def_id) => Some(def_id),
                    _ => None,
                })
            });
        }
    }
}

/// The trait ref an unsizing coercion from `src` to `target` needs where it makes a `dyn Trait`:
/// `fn(): Debug` for `&fn()` to `&dyn Debug`, through references, raw pointers, and the type
/// arguments of a smart pointer or struct (`Box`, `Rc`).
fn unsized_trait_ref<'tcx>(
    tcx: TyCtxt<'tcx>,
    src: Ty<'tcx>,
    target: Ty<'tcx>,
) -> Option<ty::TraitRef<'tcx>> {
    match (*src.kind(), *target.kind()) {
        (ty::Dynamic(..), _) => None,
        (_, ty::Dynamic(preds, _)) => {
            let principal = preds.principal()?;
            Some(
                tcx.instantiate_bound_regions_with_erased(principal)
                    .with_self_ty(tcx, src),
            )
        }
        (ty::Ref(_, a, _) | ty::RawPtr(a, _), ty::Ref(_, b, _) | ty::RawPtr(b, _)) => {
            unsized_trait_ref(tcx, a, b)
        }
        (ty::Adt(da, aa), ty::Adt(db, ab)) if da == db => aa
            .types()
            .zip(ab.types())
            .find(|(a, b)| a != b)
            .and_then(|(a, b)| unsized_trait_ref(tcx, a, b)),
        _ => None,
    }
}

fn raw_pointer_key(m: hir::Mutability) -> &'static str {
    if m.is_mut() { "*mut T" } else { "*const T" }
}

/// The trait refs among `def_id`'s where-clauses (a trait's include its supertraits), at `args`.
fn where_clauses<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
    args: GenericArgsRef<'tcx>,
) -> Vec<ty::TraitRef<'tcx>> {
    tcx.clauses_of(def_id)
        .instantiate(tcx, args)
        .clauses
        .into_iter()
        .filter_map(|c| c.skip_normalization().as_trait_clause())
        .map(|tp| tcx.instantiate_bound_regions_with_erased(tp).trait_ref)
        .collect()
}

enum Selected<'tcx> {
    /// A user-written impl, at its args.
    Impl(DefId, GenericArgsRef<'tcx>),
    /// A builtin impl's obligations on the type's parts: a tuple's or a closure's `Clone`
    /// clones each element or capture.
    Builtin(Vec<ty::TraitRef<'tcx>>),
}

/// What satisfies `tr`, if selection finds exactly one impl. An auto trait's builtin impl is
/// not followed: it runs no code, and its obligations reach the private fields of std's types.
fn select<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: TypingEnv<'tcx>,
    tr: ty::TraitRef<'tcx>,
) -> Option<Selected<'tcx>> {
    let (infcx, param_env) = tcx
        .infer_ctxt()
        .ignoring_regions()
        .build_with_typing_env(env);
    let mut selcx = SelectionContext::new(&infcx);
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), param_env, tr);
    match selcx.select(&obligation) {
        Ok(Some(ImplSource::UserDefined(d))) => {
            let args = tcx.erase_and_anonymize_regions(infcx.resolve_vars_if_possible(d.args));
            (!args.has_non_region_infer()).then_some(Selected::Impl(d.impl_def_id, args))
        }
        Ok(Some(ImplSource::Builtin(_, nested))) if !tcx.trait_is_auto(tr.def_id) => {
            Some(Selected::Builtin(
                nested
                    .iter()
                    .filter_map(|o| o.predicate.as_trait_clause())
                    .map(|tp| {
                        let tp = infcx.resolve_vars_if_possible(tp);
                        tcx.instantiate_bound_regions_with_erased(tp).trait_ref
                    })
                    .collect(),
            ))
        }
        _ => None,
    }
}

impl<'a, 'tcx> Visitor<'tcx> for Walk<'a, 'tcx> {
    type NestedFilter = nested_filter::All;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        if let hir::ItemKind::Impl(hir::Impl {
            of_trait: Some(_), ..
        }) = item.kind
        {
            self.trait_impl(item.owner_id.def_id);
        }
        intravisit::walk_item(self, item)
    }

    fn visit_nested_body(&mut self, id: hir::BodyId) {
        let outer = (self.typeck, self.owner);
        self.typeck = Some(self.tcx.typeck_body(id));
        self.owner = Some(self.tcx.hir_body_owner_def_id(id));
        self.visit_body(self.tcx.hir_body(id));
        (self.typeck, self.owner) = outer;
    }

    fn visit_path(&mut self, path: &hir::Path<'tcx>, id: HirId) {
        match path.res {
            Res::Def(_, def_id) => {
                self.use_def(def_id, path.span);
                // Typeck records no args for the lang-item calls that `?` and `for` desugar to
                // (`FromResidual::from_residual`, `IntoIterator::into_iter`); their type has them.
                let tcx = self.tcx;
                let args = self.typeck_of(id).and_then(|t| {
                    t.node_args_opt(id)
                        .or_else(|| match *t.node_type_opt(id)?.kind() {
                            ty::FnDef(d, args) if d == def_id => {
                                Some(tcx.instantiate_bound_regions_with_erased(args))
                            }
                            _ => None,
                        })
                });
                if let Some(args) = args {
                    self.use_generic(def_id, args, path.span);
                }
            }
            Res::Err => self.unresolved(path),
            _ => {}
        }
        intravisit::walk_path(self, path)
    }

    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: HirId, _span: Span) {
        if let (hir::QPath::TypeRelative(..), Some(t)) = (qpath, self.typeck_of(id)) {
            if let Some(def_id) = t.type_dependent_def_id(id) {
                let span = qpath.span();
                self.use_def(def_id, span);
                self.use_generic(def_id, t.node_args(id), span);
            }
        }
        intravisit::walk_qpath(self, qpath, id)
    }

    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        if let Some(t) = self.typeck_of(e.hir_id) {
            // Method calls, overloaded operators, and indexing.
            if let Some(def_id) = t.type_dependent_def_id(e.hir_id) {
                self.use_def(def_id, e.span);
                self.use_generic(def_id, t.node_args(e.hir_id), e.span);
            }
            if let Some(ty) = t.node_type_opt(e.hir_id) {
                self.use_ty(ty, e.span);
            }
            self.fn_pointer_address(e, t);
            let mut source = t.node_type_opt(e.hir_id);
            for adj in t.expr_adjustments(e) {
                self.use_ty(adj.target, e.span);
                // An unsizing coercion to `dyn Trait` selects the source's impl of `Trait`.
                if let (Adjust::Pointer(PointerCoercion::Unsize), Some(src)) = (&adj.kind, source) {
                    if let Some(tr) = unsized_trait_ref(self.tcx, src, adj.target) {
                        self.obligation(tr, e.span, 0);
                    }
                }
                // An autoderef selects the source's impl of `Deref` or `DerefMut`.
                if let Adjust::Deref(DerefAdjustKind::Overloaded(od)) = adj.kind {
                    let lang = self.tcx.lang_items();
                    let deref = match od.mutbl {
                        hir::Mutability::Not => lang.deref_trait(),
                        hir::Mutability::Mut => lang.deref_mut_trait(),
                    };
                    if let Some(tr) = deref {
                        self.use_def(tr, e.span);
                        if let Some(src) = source {
                            self.obligation(ty::TraitRef::new(self.tcx, tr, [src]), e.span, 0);
                        }
                    }
                }
                source = Some(adj.target);
            }
        }
        intravisit::walk_expr(self, e)
    }

    fn visit_pat(&mut self, p: &'tcx hir::Pat<'tcx>) {
        if let Some(ty) = self
            .typeck_of(p.hir_id)
            .and_then(|t| t.node_type_opt(p.hir_id))
        {
            self.use_ty(ty, p.span);
        }
        intravisit::walk_pat(self, p)
    }

    /// Raw pointer types written in signatures and fields, which no expression's type shows.
    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx, hir::AmbigArg>) {
        if let hir::TyKind::Ptr(mt) = t.kind {
            self.check(t.span, self.admit_pseudo(raw_pointer_key(mt.mutbl)));
        }
        intravisit::walk_ty(self, t)
    }
}
