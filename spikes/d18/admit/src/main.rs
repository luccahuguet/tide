//! tide-admit: D18 spike admission checker (throwaway; agent-written).
//!
//! A rustc driver that, after analysis, walks the local crate's HIR with typeck results and
//! checks every resolved use of a non-local, non-trusted item (paths, method calls, overloaded
//! operators and derefs, types of expressions and patterns, impls selected for trait
//! obligations, and macro expansions) against a policy file. It also runs the structural
//! `Pure` check (FX0510) at `T: Pure`-implying obligations, `dyn` coercions, and impls.
//!
//! Usage: tide-admit --tide-policy=FILE [--tide-report=FILE] <rustc args...>
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_trait_selection;

use std::collections::{BTreeMap, HashSet};

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, HirId};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::adjustment::{Adjust, PointerCoercion};
use rustc_middle::ty::print::{PrintTraitRefExt, with_no_trimmed_paths, with_no_visible_paths, with_resolve_crate_name};
use rustc_middle::ty::{self, GenericArgsRef, Ty, TyCtxt, TypeVisitableExt, TypingEnv, Unnormalized};
use rustc_span::{ExpnKind, Span};
use rustc_trait_selection::traits::{ImplSource, Obligation, ObligationCause, SelectionContext};

// ---------------------------------------------------------------- policy

#[derive(Default)]
struct Policy {
    allow: Vec<String>,
    deny: Vec<String>,
    impure: Vec<String>,
    trusted: Vec<String>,
    pure_trait: Option<String>,
}

/// Glob match where `*` matches any run of characters (including `::`).
fn pat_match(pat: &str, s: &str) -> bool {
    match pat.split_once('*') {
        None => pat == s,
        Some((head, rest)) => {
            let Some(s) = s.strip_prefix(head) else { return false };
            (0..=s.len()).any(|i| s.is_char_boundary(i) && pat_match(rest, &s[i..]))
        }
    }
}

impl Policy {
    fn load(path: &str) -> Policy {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("tide-admit: cannot read policy {path}: {e}"));
        let mut p = Policy::default();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap().trim();
            if line.is_empty() {
                continue;
            }
            let (kind, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            let rest = rest.trim().to_string();
            match kind {
                "allow" => p.allow.push(rest),
                "deny" => p.deny.push(rest),
                "impure" => p.impure.push(rest),
                "trusted" => p.trusted.push(rest),
                "pure-trait" => p.pure_trait = Some(rest),
                _ => panic!("tide-admit: {path}:{}: unknown directive `{kind}`", n + 1),
            }
        }
        p
    }
    fn denied(&self, s: &str) -> bool {
        self.deny.iter().any(|p| pat_match(p, s))
    }
    fn admitted(&self, s: &str) -> bool {
        !self.denied(s) && self.allow.iter().any(|p| pat_match(p, s))
    }
    fn impure(&self, s: &str) -> bool {
        self.impure.iter().any(|p| pat_match(p, s))
    }
}

// ---------------------------------------------------------------- driver

struct Admit {
    policy: Policy,
    report: Option<String>,
}

impl Callbacks for Admit {
    fn after_analysis<'tcx>(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        let mut ck = Ck {
            tcx,
            pol: &self.policy,
            typeck: None,
            owner: None,
            uses: BTreeMap::new(),
            errors: Vec::new(),
            reported: HashSet::new(),
            defer_params: false,
            seen_oblig: HashSet::new(),
        };
        tcx.hir_walk_toplevel_module(&mut ck);
        ck.check_pure_impls();
        ck.check_expansions();
        if let Some(path) = &self.report {
            let mut out = String::new();
            for (p, ok) in &ck.uses {
                out.push_str(if *ok { "admit  " } else { "REJECT " });
                out.push_str(p);
                out.push('\n');
            }
            std::fs::write(path, out).expect("write report");
        }
        ck.errors.sort_by(|a, b| (a.0.lo(), &a.1).cmp(&(b.0.lo(), &b.1)));
        for (span, msg) in ck.errors {
            tcx.dcx().span_err(span, msg);
        }
        Compilation::Continue
    }
}

fn main() -> std::process::ExitCode {
    rustc_driver::install_ice_hook("tide-admit spike", |_| ());
    let mut policy = None;
    let mut report = None;
    let mut args = Vec::new();
    for a in std::env::args() {
        if let Some(v) = a.strip_prefix("--tide-policy=") {
            policy = Some(Policy::load(v));
        } else if let Some(v) = a.strip_prefix("--tide-report=") {
            report = Some(v.to_string());
        } else {
            args.push(a);
        }
    }
    let mut cb = Admit { policy: policy.expect("tide-admit: --tide-policy=FILE is required"), report };
    rustc_driver::catch_with_exit_code(|| rustc_driver::run_compiler(&args, &mut cb))
}

// ---------------------------------------------------------------- checker

struct Ck<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pol: &'a Policy,
    typeck: Option<&'tcx ty::TypeckResults<'tcx>>,
    owner: Option<LocalDefId>,
    /// every checked path seen, with its verdict (for --tide-report)
    uses: BTreeMap<String, bool>,
    errors: Vec<(Span, String)>,
    reported: HashSet<(String, usize)>,
    defer_params: bool,
    seen_oblig: HashSet<(ty::TraitRef<'tcx>, Option<LocalDefId>)>,
}


impl<'a, 'tcx> Ck<'a, 'tcx> {
    /// Canonical (defining-crate) path, independent of re-exports and of whether std is present.
    fn path(&self, def_id: DefId) -> String {
        with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(
            self.tcx.def_path_str(def_id)
        )))
    }

    fn checked(&self, def_id: DefId) -> bool {
        if def_id.is_local() {
            return false;
        }
        let krate = self.tcx.crate_name(def_id.krate);
        !self.pol.trusted.iter().any(|t| t.as_str() == krate.as_str())
    }

    fn reject(&mut self, span: Span, code: &str, what: String) {
        // Report at the outermost call site so expansions point at user code.
        // One diagnostic per item per source line (a `let` pattern and its initializer would
        // otherwise both report the same use).
        let span = span.source_callsite();
        let line = self.tcx.sess.source_map().lookup_char_pos(span.lo()).line;
        if self.reported.insert((what.clone(), line)) {
            self.errors.push((span, format!("{code} {what}")));
        }
    }

    fn use_def(&mut self, def_id: DefId, span: Span) {
        // Naming a module grants nothing; only the items used through it are checked.
        if !self.checked(def_id) || self.tcx.def_kind(def_id) == DefKind::Mod {
            return;
        }
        let path = self.path(def_id);
        self.use_pseudo(&path, span)
    }

    /// Record and check one item key (a canonical def path or a pseudo-item).
    fn use_pseudo(&mut self, path: &str, span: Span) {
        let ok = self.pol.admitted(path);
        self.uses.insert(path.to_string(), ok);
        if !ok {
            self.reject(span, "FX0520", format!("not admitted: {path}"));
        }
    }

    /// Every item named by a type: ADTs, fn items, trait objects, aliases, foreign types.
    fn use_ty(&mut self, ty: Ty<'tcx>, span: Span) {
        for arg in ty.walk() {
            let Some(t) = arg.as_type() else { continue };
            match *t.kind() {
                ty::Adt(def, _) => self.use_def(def.did(), span),
                ty::FnDef(def, _) => self.use_def(def, span),
                ty::Foreign(def) => self.use_def(def, span),
                // Raw pointers are checked as pseudo-items: casts to them are safe, and their
                // address, ordering, hashing and formatting observe layout.
                ty::RawPtr(_, m) => self.use_pseudo(if m.is_mut() { "*mut T" } else { "*const T" }, span),
                ty::Alias(_, alias) => {
                    let d: DefId = match alias.kind {
                        ty::AliasTyKind::Projection { def_id } => def_id.into(),
                        ty::AliasTyKind::Inherent { def_id } => def_id.into(),
                        ty::AliasTyKind::Opaque { def_id } => def_id.into(),
                        ty::AliasTyKind::Free { def_id } => def_id.into(),
                    };
                    self.use_def(d, span)
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

    /// The obligations a use of `def_id` with `args` incurs: check each selected impl (and its
    /// own where-clauses, recursively) for admission, and each `Pure`-implying bound structurally.
    fn use_generic(&mut self, def_id: DefId, args: GenericArgsRef<'tcx>, span: Span) {
        let tcx = self.tcx;
        let mut trait_refs = Vec::new();
        if let Some(tr) = tcx.trait_of_assoc(def_id) {
            let n = tcx.generics_of(tr).count();
            if args.len() >= n {
                trait_refs.push(ty::TraitRef::new(tcx, tr, args.iter().take(n)));
            }
        }
        if matches!(tcx.def_kind(def_id), DefKind::Ctor(..) | DefKind::Field) {
            return;
        }
        for clause in tcx.clauses_of(def_id).instantiate(tcx, args).clauses {
            if let Some(tp) = clause.skip_normalization().as_trait_clause() {
                trait_refs.push(tcx.instantiate_bound_regions_with_erased(tp).trait_ref);
            }
        }
        for tr in trait_refs {
            self.obligation(tr, span, 0);
        }
    }

    fn obligation(&mut self, tr: ty::TraitRef<'tcx>, span: Span, depth: usize) {
        let tcx = self.tcx;
        if depth > 12 || tr.references_error() || tr.has_non_region_infer() || tr.has_escaping_bound_vars() {
            return;
        }
        let env = self.typing_env();
        let tr = tcx.erase_and_anonymize_regions(tr);
        let Ok(tr) = tcx.try_normalize_erasing_regions(env, Unnormalized::new(tr)) else { return };
        if !self.seen_oblig.insert((tr, self.owner)) {
            return;
        }
        if self.implies_pure(tr.def_id) {
            self.pure_check_root(tr.self_ty(), span, "");
        }
        let Some((impl_def_id, impl_args)) = select(tcx, env, tr) else { return };
        if self.checked(impl_def_id) {
            self.use_impl(impl_def_id, tr, span);
        }
        for clause in tcx.clauses_of(impl_def_id).instantiate(tcx, impl_args).clauses {
            if let Some(tp) = clause.skip_normalization().as_trait_clause() {
                let tr = tcx.instantiate_bound_regions_with_erased(tp).trait_ref;
                self.obligation(tr, span, depth + 1);
            }
        }
    }

    /// An impl from a checked crate is admitted unless a deny pattern matches its key, and either
    /// an allow pattern matches the key or both its trait and its self type's head are admitted.
    fn use_impl(&mut self, impl_def_id: DefId, used_as: ty::TraitRef<'tcx>, span: Span) {
        let tcx = self.tcx;
        let generic = tcx.impl_trait_ref(impl_def_id).instantiate_identity().skip_normalization();
        let key = with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(format!(
            "impl {} for {}",
            generic.print_only_trait_path(),
            generic.self_ty()
        ))));
        let trait_path = self.path(used_as.def_id);
        let head_ok = match *used_as.self_ty().kind() {
            ty::Adt(d, _) => !self.checked(d.did()) || self.pol.admitted(&self.path(d.did())),
            ty::Foreign(d) => !self.checked(d) || self.pol.admitted(&self.path(d)),
            _ => true, // primitives, refs, tuples, closures: deny patterns name them explicitly
        };
        let trait_ok = !self.checked(used_as.def_id) || self.pol.admitted(&trait_path);
        let ok = !self.pol.denied(&key)
            && (self.pol.allow.iter().any(|p| pat_match(p, &key)) || (trait_ok && head_ok));
        self.uses.insert(key.clone(), ok);
        if !ok {
            self.reject(span, "FX0520", format!("not admitted: {key}"));
        }
    }

    /// Every macro expansion in this crate (including ones whose output lands in attributes, such
    /// as `#[doc = include_str!(..)]`), checked by the macro's definition path.
    fn check_expansions(&mut self) {
        use rustc_span::hygiene::LocalExpnId;
        // rustc has no public iterator over expansions; allocating an empty one reveals the count.
        let end = LocalExpnId::fresh_empty().as_u32();
        for i in 1..end {
            let data = LocalExpnId::from_u32(i).expn_data();
            if let (ExpnKind::Macro(..), Some(mac)) = (&data.kind, data.macro_def_id) {
                self.use_def(mac, data.call_site);
            }
        }
    }

    // ------------------------------------------------------------ Pure (FX0510)

    fn pure_trait(&self, def_id: DefId) -> bool {
        self.pol.pure_trait.as_deref() == Some(self.path(def_id).as_str())
    }

    fn implies_pure(&self, trait_def_id: DefId) -> bool {
        self.pol.pure_trait.is_some()
            && rustc_middle::ty::elaborate::supertrait_def_ids(self.tcx, trait_def_id)
                .any(|d| self.pure_trait(d))
    }

    fn pure_check_root(&mut self, ty: Ty<'tcx>, span: Span, ctx: &str) {
        let mut seen = HashSet::new();
        if let Some(why) = self.impure_reason(ty, &mut seen) {
            let shown = with_no_trimmed_paths!(ty.to_string());
            self.reject(span, "FX0510", format!("not Pure{ctx}: {shown}: {why}"));
        }
    }

    /// `Some(trail)` if an impure type is structurally reachable from `ty`.
    fn impure_reason(&self, ty: Ty<'tcx>, seen: &mut HashSet<Ty<'tcx>>) -> Option<String> {
        let tcx = self.tcx;
        if !seen.insert(ty) {
            return None;
        }
        let show = |t: Ty<'tcx>| with_no_trimmed_paths!(t.to_string());
        match *ty.kind() {
            ty::Bool | ty::Char | ty::Int(_) | ty::Uint(_) | ty::Float(_) | ty::Str | ty::Never => None,
            ty::Error(_) => None,
            ty::RawPtr(..) => Some(format!("raw pointer {}", show(ty))),
            ty::Foreign(_) => Some(format!("foreign type {}", show(ty))),
            ty::FnPtr(_, hdr) => {
                if hdr.abi() == rustc_abi::ExternAbi::Rust {
                    None
                } else {
                    Some(format!("foreign fn pointer {}", show(ty)))
                }
            }
            ty::FnDef(d, _) => tcx.is_foreign_item(d).then(|| format!("foreign fn {}", self.path(d))),
            ty::Ref(_, t, _) | ty::Slice(t) | ty::Array(t, _) | ty::Pat(t, _) => {
                self.impure_reason(t, seen)
            }
            ty::Tuple(ts) => ts.iter().find_map(|t| self.impure_reason(t, seen)),
            ty::Adt(def, args) => {
                let path = self.path(def.did());
                if self.pol.impure(&path) {
                    return Some(path);
                }
                if def.did().is_local() {
                    for f in def.all_fields() {
                        let fty = f.ty(tcx, args).skip_normalization();
                        if let Some(r) = self.impure_reason(fty, seen) {
                            return Some(format!("field {}: {r}", f.name));
                        }
                    }
                    None
                } else {
                    // Library ADTs are audited: impure ones are listed; others are Pure when
                    // their type arguments are.
                    args.types().find_map(|t| self.impure_reason(t, seen))
                }
            }
            ty::Closure(_, args) => args
                .as_closure()
                .upvar_tys()
                .iter()
                .find_map(|t| self.impure_reason(t, seen).map(|r| format!("capture: {r}"))),
            ty::Dynamic(preds, _) => match preds.principal_def_id() {
                Some(p) if self.implies_pure(p) => None,
                _ => Some(format!("{} has no Pure supertrait", show(ty))),
            },
            // At an impl, parameters are checked where they are substituted (every use of the
            // impl incurs a Pure-implying obligation, which `obligation` checks).
            ty::Param(_) if self.defer_params => None,
            ty::Param(_) => {
                let env = self.typing_env();
                let bounded = env.param_env.caller_bounds().iter().any(|c| {
                    c.as_trait_clause().is_some_and(|tp| {
                        tp.skip_binder().self_ty() == ty && self.implies_pure(tp.skip_binder().def_id())
                    })
                });
                (!bounded).then(|| format!("type parameter {} has no Pure bound", show(ty)))
            }
            ty::Alias(..) => {
                match tcx.try_normalize_erasing_regions(self.typing_env(), Unnormalized::new(ty)) {
                    Ok(n) if n != ty => self.impure_reason(n, seen),
                    _ => Some(format!("unnormalized {}", show(ty))),
                }
            }
            _ => Some(format!("unsupported type {}", show(ty))),
        }
    }

    /// Impls of traits that imply `Pure` (e.g. `impl Job for Holder` where `trait Job: Pure`).
    fn check_pure_impls(&mut self) {
        let tcx = self.tcx;
        for id in tcx.hir_crate_items(()).definitions() {
            if !matches!(tcx.def_kind(id), DefKind::Impl { of_trait: true }) {
                continue;
            }
            let tr = tcx.impl_trait_ref(id).instantiate_identity().skip_normalization();
            if self.implies_pure(tr.def_id) && !self.pure_trait(tr.def_id) {
                (self.owner, self.defer_params) = (Some(id), true);
                self.pure_check_root(tr.self_ty(), tcx.def_span(id), " (impl)");
                (self.owner, self.defer_params) = (None, false);
            }
        }
    }
}

/// Like rustc's `codegen_select_candidate`, but returns None instead of ICEing on odd errors.
fn select<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: TypingEnv<'tcx>,
    tr: ty::TraitRef<'tcx>,
) -> Option<(DefId, GenericArgsRef<'tcx>)> {
    let (infcx, param_env) = tcx.infer_ctxt().ignoring_regions().build_with_typing_env(env);
    let mut selcx = SelectionContext::new(&infcx);
    let ob = Obligation::new(tcx, ObligationCause::dummy(), param_env, tr);
    match selcx.select(&ob) {
        Ok(Some(ImplSource::UserDefined(d))) => {
            let args = infcx.resolve_vars_if_possible(d.args);
            let args = tcx.erase_and_anonymize_regions(args);
            (!args.has_non_region_infer()).then_some((d.impl_def_id, args))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------- HIR walk

impl<'a, 'tcx> Visitor<'tcx> for Ck<'a, 'tcx> {
    type NestedFilter = nested_filter::All;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn visit_nested_body(&mut self, id: hir::BodyId) {
        let old = (self.typeck, self.owner);
        self.typeck = Some(self.tcx.typeck_body(id));
        self.owner = Some(self.tcx.hir_body_owner_def_id(id));
        let body = self.tcx.hir_body(id);
        self.visit_body(body);
        (self.typeck, self.owner) = old;
    }

    fn visit_path(&mut self, path: &hir::Path<'tcx>, id: HirId) {
        if let Res::Def(_, def_id) = path.res {
            self.use_def(def_id, path.span);
            if let Some(args) = self.typeck.and_then(|t| t.node_args_opt(id)) {
                self.use_generic(def_id, args, path.span);
            }
        }
        intravisit::walk_path(self, path)
    }

    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: HirId, _span: Span) {
        if let hir::QPath::TypeRelative(..) = qpath {
            if let Some(t) = self.typeck {
                if let Some(def_id) = t.type_dependent_def_id(id) {
                    let span = qpath.span();
                    self.use_def(def_id, span);
                    self.use_generic(def_id, t.node_args(id), span);
                }
            }
        }
        intravisit::walk_qpath(self, qpath, id)
    }

    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        if let Some(t) = self.typeck {
            // Method calls, overloaded operators and indexing.
            if let Some(def_id) = t.type_dependent_def_id(e.hir_id) {
                self.use_def(def_id, e.span);
                self.use_generic(def_id, t.node_args(e.hir_id), e.span);
            }
            if let Some(ty) = t.node_type_opt(e.hir_id) {
                self.use_ty(ty, e.span);
            }
            for adj in t.expr_adjustments(e) {
                self.use_ty(adj.target, e.span);
                match adj.kind {
                    Adjust::Deref(rustc_middle::ty::adjustment::DerefAdjustKind::Overloaded(od)) => {
                        let tcx = self.tcx;
                        let tr = match od.mutbl {
                            hir::Mutability::Not => tcx.lang_items().deref_trait(),
                            hir::Mutability::Mut => tcx.lang_items().deref_mut_trait(),
                        };
                        if let Some(tr) = tr {
                            self.use_def(tr, e.span);
                        }
                    }
                    Adjust::Pointer(PointerCoercion::Unsize) => {
                        let src = t.expr_ty(e);
                        self.unsize_pure(src, adj.target, e.span);
                    }
                    _ => {}
                }
            }
        }
        intravisit::walk_expr(self, e)
    }

    fn visit_pat(&mut self, p: &'tcx hir::Pat<'tcx>) {
        if let Some(ty) = self.typeck.and_then(|t| t.node_type_opt(p.hir_id)) {
            self.use_ty(ty, p.span);
        }
        intravisit::walk_pat(self, p)
    }

}

impl<'a, 'tcx> Ck<'a, 'tcx> {
    /// Coercing `&S`/`Box<S>`/... to a `dyn Trait` whose principal implies `Pure` checks `S`.
    fn unsize_pure(&mut self, src: Ty<'tcx>, dst: Ty<'tcx>, span: Span) {
        let pointee = |t: Ty<'tcx>| t.builtin_deref(true).or_else(|| t.boxed_ty());
        let (Some(s), Some(d)) = (pointee(src), pointee(dst)) else { return };
        if let ty::Dynamic(preds, _) = d.kind() {
            if let Some(p) = preds.principal_def_id() {
                if self.implies_pure(p) && !matches!(s.kind(), ty::Dynamic(..)) {
                    let what = with_no_trimmed_paths!(format!(" (as {d})"));
                    self.pure_check_root(s, span, &what);
                }
            }
        }
    }
}
