//! The admission walk (ADMIT-1): every item the local crate uses, as rustc resolved it, checked
//! against the allowlist. It covers paths, type-relative paths, method calls, overloaded
//! operators and derefs, the types of expressions and patterns, impls selected for the trait
//! obligations that a use, a type's where-clauses, an autoderef, an unsizing coercion to
//! `dyn Trait`, a local trait impl, or the entry point's `Termination` incurs (and their
//! where-clauses, or a builtin impl's obligations on a type's parts), function pointers' integer
//! casts and builtin comparisons, and macro expansions, including ones that land in attributes.
//!
//! The same walk finds the `Pure` obligations those uses incur, those of the types that
//! declarations name (fields, signatures, impl headers, associated types) and that a use's
//! generic args name, those of a trait impl item's own where-clauses, and those of an opaque
//! type's bounds, decided by `pure.rs`; the reads of raw memory that ROOT-4 bounds by `Pure`;
//! the rules on items: `static`s (GLOBAL-1 to 3), `const fn` (AFREE-2), `#[pure]` (AFREE-1),
//! and what needs the `unsafe` grant (UNSAFE-1); the paths that name the binary's `main`
//! (ROOT-5); and fn items coerced to `fn` pointers (CLOSURE-3, `reify.rs`).

use std::collections::BTreeMap;

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::def::{CtorOf, DefKind, Res};
use rustc_hir::def_id::{CrateNum, DefId, LocalDefId};
use rustc_hir::intravisit::{self, FnKind, Visitor};
use rustc_hir::{self as hir, HirId};
use rustc_hir_typeck::expr_use_visitor::{Delegate, ExprUseVisitor, PlaceBase, PlaceWithHirId};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_lint::LateContext;
use rustc_middle::hir::nested_filter;
use rustc_middle::mir::FakeReadCause;
use rustc_middle::ty::adjustment::{Adjust, DerefAdjustKind, PointerCoercion};
use rustc_middle::ty::print::{
    PrintTraitRefExt, with_forced_trimmed_paths, with_no_trimmed_paths, with_no_visible_paths,
    with_resolve_crate_name,
};
use rustc_middle::ty::{
    self, GenericArgsRef, Ty, TyCtxt, TypeFoldable, TypeVisitableExt, TypingEnv, Unnormalized,
};
use rustc_span::fatal_error::catch_fatal_errors;
use rustc_span::hygiene::LocalExpnId;
use rustc_span::{ExpnId, ExpnKind, MacroKind, Span, kw, sym};
use rustc_trait_selection::traits::{
    ImplSource, Obligation, ObligationCause, SelectionContext, normalize_param_env_or_error,
};

use crate::crates::{Crates, Role};
use crate::diag::{Code, Diagnostics, RustcLabels};
use crate::policy::{self, Rule, Verdict};
use crate::pure::{self, Need, Pure};

mod authority;
mod contracts;
mod habits;
mod modules;
mod mono;
mod reify;
mod sinks;
mod surface;
mod traits;
pub use habits::check_habit_surface;
pub use modules::check_module_surface;
pub use mono::check_mono_items;
pub use traits::check_trait_surface;

/// What the package's manifest grants it (03 RUN-4).
#[derive(Clone, Copy, Default)]
pub struct Grants {
    pub unsafe_: bool,
}

/// The crate's trust holes (D11): each site the `unsafe` grant admits, as (the key of the item
/// that holds it, that item's kind, what the hole is) and how many distinct sites there are. A
/// use of an item the grant admits counts once per holding item, however often it recurs.
pub type Holes = BTreeMap<(String, String, String), usize>;

/// Checks the local crate, emits its diagnostics, and returns its trust holes. With `tide_std`,
/// the operator supplied Tide's std, so a binary's `main` needs its `#[tide::main]` (ROOT-1).
/// Without `obligations`, a switch for the backstop's tests, the walk skips its obligation steps
/// (`obligation`, `pure_obligation`).
pub fn check_crate(
    tcx: TyCtxt<'_>,
    grants: Grants,
    tide_std: bool,
    obligations: bool,
    test_library: Option<CrateNum>,
    test_suite: bool,
    labels: &RustcLabels,
    insertions: &crate::synthesis::Insertions,
    surface_errors: usize,
    unused_imports: &[Span],
) -> Holes {
    let crates = Crates::new(tcx);
    let mut walk = Walk::new(tcx, &crates, grants, test_library);
    walk.test_suite = test_suite;
    walk.labels = labels.clone();
    walk.insertions = Some(insertions);
    walk.obligations = obligations;
    tcx.hir_walk_toplevel_module(&mut walk);
    walk.ceiling_cycles();
    walk.module_rules(surface_errors, unused_imports);
    walk.visit_boundary(false, |walk| walk.entry_point(tide_std));
    walk.check_expansions();
    for def in tcx.hir_crate_items(()).definitions() {
        if std_attr_made(tcx, &crates, def, "trusted_pure") {
            walk.holes
                .insert((def, "`#[trusted_pure]`".into(), Some(tcx.def_span(def))));
        }
    }
    // Rejected crates have no artifact; formatting an unresolved impl's key also lowers its trait.
    let holes = if !walk.caught_fatal {
        walk.take_holes()
    } else {
        Holes::new()
    };
    walk.diags.emit(tcx);
    holes
}

/// How an admitted use is admitted: freely, or by the `unsafe` grant, under this key.
#[derive(Clone)]
enum Admitted {
    Free,
    Granted(String),
}

/// Why a use is not admitted.
#[derive(Clone)]
struct Rejection {
    key: String,
    code: Code,
    help: Option<&'static str>,
}

/// Obligations are followed through impl where-clauses at most this deep; a deeper chain is
/// rejected, since what it would select is unchecked.
pub(crate) const MAX_OBLIGATION_DEPTH: usize = 64;

const GRANT: &str = "needs the package's `unsafe` grant";

/// `io::Error::new` and `io::Error::other`, Tide's std's own (03 §1.1), by `Crates::key`.
const IO_ERROR_NEW: &[&str] = &["tide_std::io::Error::new", "tide_std::io::Error::other"];

struct Walk<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    crates: &'a Crates,
    grants: Grants,
    pure: Option<Pure<'a, 'tcx>>,
    diags: Diagnostics,
    labels: RustcLabels,
    trait_calls: Vec<traits::Call>,
    insertions: Option<&'a crate::synthesis::Insertions>,
    typeck: Option<&'tcx ty::TypeckResults<'tcx>>,
    owner: Option<LocalDefId>,
    /// A fatal query prevents serialization of rejected-crate trust holes.
    caught_fatal: bool,
    /// The current item still gets HIR checks, without querying its unknown types.
    recovering: bool,
    seen_obligations: FxHashSet<(ty::TraitRef<'tcx>, Option<LocalDefId>)>,
    /// Like `seen_obligations`, per rule: a failed `Pure` obligation is reported once per item
    /// and rule, at its first use.
    seen_pure: FxHashSet<(ty::TraitRef<'tcx>, Option<LocalDefId>, Need)>,
    /// The `Pure` decision's environment in place of the owner's, while `impl_item_clauses`
    /// decides a trait impl item's where-clauses, or the backstop an instance's.
    item_env: Option<TypingEnv<'tcx>>,
    /// Whether the obligation steps run (`check_crate`).
    obligations: bool,
    /// The exact artifact this driver just admitted for a separate test crate.
    test_library: Option<CrateNum>,
    /// The separate suite is test code even if it never loads the candidate artifact.
    test_suite: bool,
    /// The item being visited, which holds the trust holes found outside bodies.
    item: Option<LocalDefId>,
    /// Trust holes: (holding item, what, the site's span, or none for a use counted once).
    holes: FxHashSet<(LocalDefId, String, Option<Span>)>,
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    fn new(
        tcx: TyCtxt<'tcx>,
        crates: &'a Crates,
        grants: Grants,
        test_library: Option<CrateNum>,
    ) -> Self {
        Walk {
            tcx,
            crates,
            grants,
            pure: crate_pure(tcx, crates),
            diags: Diagnostics::default(),
            labels: RustcLabels::default(),
            trait_calls: Vec::new(),
            insertions: None,
            typeck: None,
            owner: None,
            caught_fatal: false,
            recovering: false,
            seen_obligations: FxHashSet::default(),
            seen_pure: FxHashSet::default(),
            item_env: None,
            obligations: true,
            test_library,
            test_suite: false,
            item: None,
            holes: FxHashSet::default(),
        }
    }

    /// A poisoned query raises the same fatal again; retain known facts and retry only HIR.
    fn visit_boundary(&mut self, recovering: bool, mut visit: impl FnMut(&mut Self)) {
        let outer = (
            self.typeck,
            self.owner,
            self.item,
            self.recovering,
            self.item_env,
        );
        self.recovering = recovering;
        if self.tcx.dcx().has_errors().is_none() {
            visit(self);
        } else {
            // `impl_item_clauses` temporarily takes this cache, so unwinding must restore it too.
            let seen_pure = self.seen_pure.clone();
            // rustc's helper resumes every payload other than FatalErrorMarker unchanged.
            if catch_fatal_errors(|| visit(self)).is_err() {
                self.caught_fatal = true;
                (
                    self.typeck,
                    self.owner,
                    self.item,
                    self.recovering,
                    self.item_env,
                ) = outer;
                self.seen_pure.clone_from(&seen_pure);
                if !recovering {
                    self.typeck = None;
                    self.recovering = true;
                    if catch_fatal_errors(|| visit(self)).is_err() {
                        self.seen_pure = seen_pure;
                    }
                }
            }
        }
        (
            self.typeck,
            self.owner,
            self.item,
            self.recovering,
            self.item_env,
        ) = outer;
    }
}

/// The crate's `Pure` decision, or none without Tide's std (the driver's own tests run against
/// Rust's).
pub fn crate_pure<'a, 'tcx>(tcx: TyCtxt<'tcx>, crates: &'a Crates) -> Option<Pure<'a, 'tcx>> {
    let krate = crates.tide_std()?;
    let pure_trait = tcx
        .traits(krate)
        .iter()
        .copied()
        .find(|&d| crates.key(tcx, d) == "tide_std::marker::Pure")?;
    let trusted = tcx
        .hir_crate_items(())
        .definitions()
        .filter(|&d| {
            matches!(
                tcx.def_kind(d),
                DefKind::Struct | DefKind::Enum | DefKind::Union
            )
        })
        .filter(|&d| std_attr_made(tcx, crates, d, "trusted_pure"))
        .map(LocalDefId::to_def_id)
        .collect();
    Some(Pure::new(tcx, crates, pure_trait, trusted))
}

/// Whether the attribute macro `name` that Tide's std exports (`pure`, `trusted_pure`) was
/// applied to the item `def`, directly or through other macros. An attribute's expansion also
/// defines the items nested in its target, such as a helper `fn` in a `#[pure]` body; those are
/// not its target, since their parent comes from the same expansion.
fn std_attr_made(tcx: TyCtxt<'_>, crates: &Crates, def: LocalDefId, name: &str) -> bool {
    let from = |d: LocalDefId, target: ExpnId| {
        let mut expn = tcx.expn_that_defined(d.to_def_id());
        while expn != ExpnId::root() {
            if expn == target {
                return true;
            }
            expn = expn.expn_data().parent;
        }
        false
    };
    let mut expn = tcx.expn_that_defined(def.to_def_id());
    while expn != ExpnId::root() {
        let data = expn.expn_data();
        if matches!(data.kind, ExpnKind::Macro(MacroKind::Attr, _))
            && data
                .macro_def_id
                .is_some_and(|m| is_std_attr(tcx, crates, m, name))
        {
            return tcx.opt_local_parent(def).is_none_or(|p| !from(p, expn));
        }
        expn = data.parent;
    }
    false
}

fn is_std_attr(tcx: TyCtxt<'_>, crates: &Crates, mac: DefId, name: &str) -> bool {
    crates.exported(mac) && tcx.item_name(mac).as_str() == name
}

impl<'a, 'tcx> Walk<'a, 'tcx> {
    // ------------------------------------------------------------ verdicts

    /// The allowlist key of an item (`Crates::key`).
    fn key(&self, def_id: DefId) -> String {
        self.crates.key(self.tcx, def_id)
    }

    fn decide(
        &self,
        key: String,
        rule: Option<&'static Rule>,
        unlisted_admitted: bool,
    ) -> Result<Admitted, Rejection> {
        match rule.map(|r| r.verdict) {
            Some(Verdict::Allow) => Ok(Admitted::Free),
            Some(Verdict::Unsafe) if self.grants.unsafe_ => Ok(Admitted::Granted(key)),
            None if unlisted_admitted => Ok(Admitted::Free),
            _ => Err(Rejection {
                key,
                code: rule.map_or(Code::Td0520, |r| r.code),
                help: rule.and_then(|r| r.help),
            }),
        }
    }

    /// Items of the local crate are admitted, as is whatever Tide's std re-exports and the
    /// items of the traits it re-exports; Tide's std's own items unless a rule says otherwise;
    /// `core`, `alloc`, and Rust's std only where a rule allows. The paired test library's
    /// definitions are admitted, with their visibility decided by rustc.
    fn admit_def(&self, def_id: DefId) -> Result<Admitted, Rejection> {
        let role = self.crates.role(def_id.krate);
        let exported = |d| self.crates.exported(d);
        if role == Role::Local
            || (self.test_library == Some(def_id.krate)
                && !matches!(self.tcx.def_kind(def_id), DefKind::Macro(_)))
            || exported(def_id)
            || self.tcx.trait_of_assoc(def_id).is_some_and(exported)
        {
            return Ok(Admitted::Free);
        }
        if let Some(rejection) = self.habit_def(def_id) {
            return Err(rejection);
        }
        let key = self.key(def_id);
        match role {
            Role::Other => Err(Rejection {
                key,
                code: Code::Td0520,
                help: None,
            }),
            _ => self.decide(key.clone(), policy::lookup(&key), role == Role::TideStd),
        }
    }

    /// The driver checks this impl in its defining crate during this invocation.
    fn owns_impl(&self, impl_def_id: DefId) -> bool {
        impl_def_id.is_local() || self.test_library == Some(impl_def_id.krate)
    }

    /// For other crates, a rule can decide an impl by key. An unlisted impl is admitted when it
    /// comes from Tide's std or both the trait and the self type's head are admitted.
    fn admit_impl(
        &self,
        impl_def_id: DefId,
        used_as: ty::TraitRef<'tcx>,
    ) -> Result<Admitted, Rejection> {
        let role = self.crates.role(impl_def_id.krate);
        if self.owns_impl(impl_def_id) {
            return Ok(Admitted::Free);
        }
        let key = impl_key(self.tcx, impl_def_id);
        if let Some(rule) = policy::lookup(&key) {
            return self.decide(key, Some(rule), false);
        }
        let head = match *used_as.self_ty().kind() {
            ty::Adt(def, _) => self.admit_def(def.did()),
            ty::Foreign(def) => self.admit_def(def),
            ty::RawPtr(_, m) => self.admit_pseudo(raw_pointer_key(m)),
            _ => Ok(Admitted::Free),
        };
        if role == Role::TideStd {
            return Ok(Admitted::Free);
        }
        self.admit_def(used_as.def_id)
            .and(head)
            .map(|_| Admitted::Free)
            .map_err(|rejection| Rejection { key, ..rejection })
    }

    fn admit_pseudo(&self, key: &str) -> Result<Admitted, Rejection> {
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
    /// entry point that Tide's std's `#[tide::main]` generates as the binary's crate-root
    /// `main`, which starts the runtime, and structural test impls inserted by this run.
    fn generated_by_toolchain(&self, mut span: Span) -> bool {
        if self
            .insertions
            .is_some_and(|edits| edits.contains(self.tcx, span))
        {
            return true;
        }
        while span.from_expansion() {
            let expn = span.ctxt().outer_expn();
            let data = expn.expn_data();
            let toolchain_attr = matches!(data.kind, ExpnKind::Macro(MacroKind::Attr, _))
                && data.macro_def_id.is_some_and(|d| {
                    let name = self.tcx.item_name(d);
                    (self.crates.role(d.krate) == Role::Core && name == sym::test)
                        || (self.crates.exported(d)
                            && name == sym::main
                            && self.crates.is_entry(expn))
                });
            if toolchain_attr || matches!(data.kind, ExpnKind::AstPass(_)) {
                return true;
            }
            span = data.call_site;
        }
        false
    }

    /// Candidate-written code, including tokens passed through a std attribute macro.
    fn user_written(&self, span: Span) -> bool {
        !written_by_other_crate(span) && !self.generated_by_toolchain(span)
    }

    /// The typeck results that cover `id`: none in the signature of an item nested in a body.
    fn typeck_of(&self, id: HirId) -> Option<&'tcx ty::TypeckResults<'tcx>> {
        self.recovered_typeck_of(id)
            .filter(|t| t.tainted_by_errors.is_none())
    }

    /// Recorded resolutions/types, including partial results; consumers validate query inputs.
    fn recovered_typeck_of(&self, id: HirId) -> Option<&'tcx ty::TypeckResults<'tcx>> {
        self.typeck.filter(|t| t.hir_owner == id.owner)
    }

    fn can_select(&self) -> bool {
        !self.recovering && self.typeck.is_none_or(|t| t.tainted_by_errors.is_none())
    }

    // ------------------------------------------------------------ uses

    fn reject(&mut self, span: Span, rejection: Rejection, note: &str) {
        // Report at the outermost call site, so an expansion points at the user's code.
        let span = span.source_callsite();
        let message = format!("`{}` is not admitted{note}", rejection.key);
        self.diags
            .report(self.tcx, span, rejection.code, message, rejection.help);
    }

    fn check(&mut self, span: Span, verdict: Result<Admitted, Rejection>) {
        if self.generated_by_toolchain(span) {
            return;
        }
        // Written async/await is rejected before lowering; its lang-item uses are not user code.
        if matches!(
            span.desugaring_kind(),
            Some(rustc_span::DesugaringKind::Async | rustc_span::DesugaringKind::Await)
        ) {
            return;
        }
        match verdict {
            Err(r) => self.reject(span, r, ""),
            Ok(admitted) => self.admitted(span, admitted),
        }
    }

    /// Records a use the `unsafe` grant admitted as a trust hole, once per holding item.
    fn admitted(&mut self, span: Span, admitted: Admitted) {
        if let Admitted::Granted(key) = admitted {
            self.hole(span, format!("uses `{key}`"), false);
        }
    }

    /// Records a trust hole at `span`: one per `site` when `site` is true, else once per holding
    /// item, the innermost of the body and the item being visited, with a closure's held by its
    /// function.
    fn hole(&mut self, span: Span, what: String, site: bool) {
        let tcx = self.tcx;
        let inner = match (self.owner, self.item) {
            (Some(body), Some(item))
                if tcx.is_descendant_of(item.to_def_id(), body.to_def_id()) =>
            {
                Some(item)
            }
            (body, item) => body.or(item),
        };
        // After the walk, as for a macro's use: the definition it is written in.
        let inner = inner.or_else(|| self.holder_at(span));
        let holder = inner.map_or(rustc_hir::def_id::CRATE_DEF_ID, |d| {
            tcx.typeck_root_def_id(d.to_def_id()).expect_local()
        });
        let span = site.then(|| span.source_callsite());
        self.holes.insert((holder, what, span));
    }

    /// UNSAFE-1: what needs the `unsafe` grant is a trust hole with it and `TD0901` without it.
    fn granted(&mut self, span: Span, what: &str) {
        if !self.user_written(span) {
            return;
        }
        if self.grants.unsafe_ {
            self.hole(span, what.into(), true);
        } else {
            self.report(span, Code::Td0901, format!("{what} {GRANT}"));
        }
    }

    /// The trust holes found so far, by holding item's key and what each is (`Holes`).
    fn take_holes(&mut self) -> Holes {
        let mut holes = Holes::new();
        for (def, what, _) in std::mem::take(&mut self.holes) {
            let key = self.crates.key(self.tcx, def.to_def_id());
            let kind = self.tcx.def_descr(def.to_def_id()).to_string();
            *holes.entry((key, kind, what)).or_default() += 1;
        }
        holes
    }

    /// The innermost local definition whose source contains `span`.
    fn holder_at(&self, span: Span) -> Option<LocalDefId> {
        let tcx = self.tcx;
        let span = span.source_callsite();
        tcx.hir_crate_items(())
            .definitions()
            .filter(|&d| tcx.source_span(d).contains(span))
            .min_by_key(|&d| {
                let s = tcx.source_span(d);
                s.hi() - s.lo()
            })
    }

    /// Reports `code` at the outermost call site of `span`, unless the toolchain generated it.
    fn flag(&mut self, span: Span, code: Code, message: String) {
        if !self.generated_by_toolchain(span) {
            self.report(span, code, message);
        }
    }

    /// Reports `code` at the outermost call site of `span`.
    fn report(&mut self, span: Span, code: Code, message: String) {
        self.diags
            .report(self.tcx, span.source_callsite(), code, message, None);
    }

    fn use_def(&mut self, def_id: DefId, span: Span) {
        // Naming a module grants nothing; the items used through it are checked.
        if self.tcx.def_kind(def_id) != DefKind::Mod {
            self.check(span, self.admit_def(def_id));
        }
    }

    /// Every item a type names: ADTs, fn items, foreign types, aliases, trait objects, and raw
    /// pointers, whose ordering, hashing, and formatting observe addresses. The where-clauses of
    /// an ADT and of a fn item are obligations too: a constructor passed as a function
    /// (`.map(Logged)`) builds values no expression's type shows. So are a local `Drop` impl's
    /// `Pure` bounds, which rustc lets go beyond the ADT's (`local_drop_impl`).
    fn use_ty(&mut self, ty: Ty<'tcx>, span: Span) {
        for arg in ty.walk() {
            let Some(t) = arg.as_type() else { continue };
            match *t.kind() {
                ty::Adt(def, args) => {
                    self.use_def(def.did(), span);
                    self.use_generic(def.did(), args, span, true);
                }
                ty::FnDef(def, args) => {
                    self.use_def(def, span);
                    let args = self.tcx.instantiate_bound_regions_with_erased(args);
                    self.use_generic(def, args, span, true);
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
    /// item, its where-clauses, and those of its local `Drop` impl (`local_drop_impl`). `admit`
    /// is false for a type that is only named (`named`), whose obligations run no code: only
    /// their `Pure` part is checked.
    fn use_generic(&mut self, def_id: DefId, args: GenericArgsRef<'tcx>, span: Span, admit: bool) {
        if self.recovering {
            return;
        }
        let tcx = self.tcx;
        let complete = self.can_select();
        // A failed bound can leave known arguments; each Pure check validates its own types.
        if !complete && (self.pure.is_none() || args.len() != tcx.generics_of(def_id).count()) {
            return;
        }
        if let Some(tr) = tcx.trait_of_assoc(def_id) {
            let n = tcx.generics_of(tr).count();
            if args.len() >= n {
                let tr = ty::TraitRef::new(tcx, tr, args.iter().take(n));
                if admit && complete {
                    self.obligation(tr, span, 0);
                }
                self.pure_obligation(tr, Need::Bound, span, 0);
            }
        }
        if tcx.def_kind(def_id) != DefKind::Field {
            self.clause_obligations(def_id, args, span, admit && complete);
        }
        if complete && let Some((impl_def_id, impl_args)) = local_drop_impl(tcx, def_id, args) {
            self.clause_obligations(impl_def_id, impl_args, span, false);
        }
    }

    /// Follows the obligation `tr` for admission and for `Pure`.
    fn follow(&mut self, tr: ty::TraitRef<'tcx>, span: Span) {
        self.obligation(tr, span, 0);
        self.pure_obligation(tr, Need::Bound, span, 0);
    }

    // ------------------------------------------------------------ Pure

    /// The obligations of `def_id`'s where-clauses at `args`, for admission when `admit`, and for
    /// `Pure`, each under the rule `pure::needs` gives for the item and the parameter it bounds.
    fn clause_obligations(
        &mut self,
        def_id: DefId,
        args: GenericArgsRef<'tcx>,
        span: Span,
        admit: bool,
    ) {
        let tcx = self.tcx;
        let clauses = where_clauses(tcx, def_id, args);
        if admit {
            for &tr in &clauses {
                self.obligation(tr, span, 0);
            }
        }
        let Some(pure_trait) = self.pure.as_ref().map(|p| p.trait_def) else {
            return;
        };
        let role = self.crates.role(def_id.krate);
        let key = if self.crates.is_std(def_id) {
            self.key(def_id)
        } else {
            String::new()
        };
        if let Some(&(_, at)) = pure::FORGES.iter().find(|(p, _)| policy::matches(p, &key))
            && let Some(made) = args.types().nth(at)
        {
            self.forged(made, span);
        }
        let (for_closures, for_others) = match role {
            Role::TideStd if IO_ERROR_NEW.contains(&key.as_str()) => {
                let into = tcx.get_diagnostic_item(sym::Into);
                if let Some(&tr) = clauses.iter().find(|t| Some(t.def_id) == into) {
                    self.io_error_payload(tr, span);
                }
                (Need::Bound, Need::Bound)
            }
            Role::TideStd => pure::needs(&key),
            _ => (Need::Bound, Need::Bound),
        };
        let lang = tcx.lang_items();
        let fn_traits = [lang.fn_trait(), lang.fn_mut_trait(), lang.fn_once_trait()];
        // Each where-clause's self type as written, and the parameters with an `Fn` bound.
        let written: Vec<_> = tcx
            .clauses_of(def_id)
            .instantiate_identity(tcx)
            .clauses
            .into_iter()
            .filter_map(|c| c.skip_normalization().as_trait_clause())
            .map(|tp| (tp.def_id(), tp.self_ty().skip_binder()))
            .collect();
        let closures: Vec<_> = written
            .iter()
            .filter(|(d, _)| fn_traits.contains(&Some(*d)))
            .map(|&(_, t)| t)
            .collect();
        for (&tr, &(_, param)) in clauses.iter().zip(&written) {
            let need = if closures.contains(&param) {
                for_closures
            } else {
                for_others
            };
            if tr.def_id == pure_trait && matches!(need, Need::JoinClosure | Need::SharedClosure) {
                // The closure's `Fn` bound, which a shared closure must meet (PAR-6).
                let fn_bound = clauses.iter().zip(&written).find_map(|(&t, &(d, p))| {
                    (p == param && Some(d) == lang.fn_trait()).then_some(t)
                });
                self.parallel_closure(tr.self_ty(), need, fn_bound, span);
            } else {
                self.pure_obligation(tr, need, span, 0);
            }
        }
    }

    /// ROOT-4: a value of type `made` that raw bytes or a raw pointer make, under the `unsafe`
    /// grant too, is `Pure`.
    fn forged(&mut self, made: Ty<'tcx>, span: Span) {
        if let Some(pure) = &self.pure {
            let tr = ty::TraitRef::new(self.tcx, pure.trait_def, [made]);
            self.pure_obligation(tr, Need::Forged, span, 0);
        }
    }

    /// ROOT-4: a place that reads raw memory makes a value of its type: a raw pointer's deref
    /// (`*p`, `&*p`, `(*p).field`), or a union's field, which reinterprets another field's bytes
    /// as `transmute` does. A place only assigned to or borrowed raw (`&raw const (*p).field`),
    /// directly or through a field or a builtin array or slice index, reads nothing. An
    /// overloaded `Index` borrows its base, and an implicit deref of the base (of a reference, a
    /// `Box`, or an overloaded `Deref`) loads or borrows it, so both read.
    fn raw_place(&mut self, e: &hir::Expr<'tcx>, t: &ty::TypeckResults<'tcx>) {
        let ty_of = |e: &hir::Expr<'tcx>| t.expr_ty_adjusted_opt(e).map(|t| *t.kind());
        let derefs = |id: hir::HirId| {
            t.adjustments()
                .get(id)
                .is_some_and(|a| a.iter().any(|a| matches!(a.kind, Adjust::Deref(_))))
        };
        let made = match e.kind {
            hir::ExprKind::Unary(hir::UnOp::Deref, inner) => match ty_of(inner) {
                Some(ty::RawPtr(pointee, _)) => pointee,
                _ => return,
            },
            hir::ExprKind::Field(base, _) => match (ty_of(base), t.node_type_opt(e.hir_id)) {
                (Some(ty::Adt(def, _)), Some(field)) if def.is_union() => field,
                _ => return,
            },
            _ => return,
        };
        let mut id = e.hir_id;
        while let hir::Node::Expr(parent) = self.tcx.parent_hir_node(id) {
            match parent.kind {
                hir::ExprKind::Field(..) if !derefs(id) => id = parent.hir_id,
                hir::ExprKind::Index(base, ..)
                    if base.hir_id == id && !derefs(id) && !t.is_method_call(parent) =>
                {
                    id = parent.hir_id
                }
                hir::ExprKind::AddrOf(hir::BorrowKind::Raw, ..) => return,
                hir::ExprKind::Assign(lhs, ..) if lhs.hir_id == id => return,
                _ => break,
            }
        }
        self.forged(made, e.span);
    }

    /// The typing environment the `Pure` decision runs in: the owner's where-clauses, with opaque
    /// types left opaque, so no `Pure` leaks from a hidden type (02 §5.2).
    fn pure_env(&self) -> TypingEnv<'tcx> {
        if let Some(env) = self.item_env {
            return env;
        }
        let owner = self.owner.unwrap_or(rustc_hir::def_id::CRATE_DEF_ID);
        TypingEnv::non_body_analysis(self.tcx, owner.to_def_id())
    }

    /// The type `ty` normalized in the `Pure` environment, unless it holds an error or an
    /// inference variable.
    fn pure_ty(&self, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
        if self.recovering
            || ty.references_error()
            || ty.has_non_region_infer()
            || ty.has_escaping_bound_vars()
        {
            return None;
        }
        let tcx = self.tcx;
        let ty = tcx.erase_and_anonymize_regions(ty);
        let ty = tcx
            .try_normalize_erasing_regions(self.pure_env(), Unnormalized::new(ty))
            .ok()?;
        (!ty.references_error() && !ty.has_non_region_infer() && !ty.has_escaping_bound_vars())
            .then_some(ty)
    }

    /// Follows `tr` through the impls that meet it, as `obligation` does but with opaque types
    /// left opaque, and decides each `Pure` obligation on the way, reporting a failure, or a chain
    /// too deep to follow, under `need`'s code. It reports under the toolchain's expansions too,
    /// since the user's types choose the impl: `#[tide::main]` and `#[test]` report a result
    /// through its `Termination` impl.
    fn pure_obligation(&mut self, tr: ty::TraitRef<'tcx>, need: Need, span: Span, depth: usize) {
        let tcx = self.tcx;
        let Some(pure) = self
            .pure
            .as_ref()
            .filter(|_| self.obligations && !self.recovering)
        else {
            return;
        };
        if tr.references_error() || tr.has_non_region_infer() || tr.has_escaping_bound_vars() {
            return;
        }
        if depth > MAX_OBLIGATION_DEPTH {
            let message = format!(
                "`{}: {}` nests obligations too deep to decide `Pure`",
                with_forced_trimmed_paths!(tr.self_ty().to_string()),
                with_forced_trimmed_paths!(tr.print_only_trait_path().to_string())
            );
            self.report(span, need.code(), need.message(message));
            return;
        }
        let env = self.pure_env();
        let tr = tcx.erase_and_anonymize_regions(tr);
        let Ok(tr) = tcx.try_normalize_erasing_regions(env, Unnormalized::new(tr)) else {
            return;
        };
        if !self.seen_pure.insert((tr, self.owner, need)) {
            return;
        }
        if tr.def_id == pure.trait_def {
            if let Err(e) = pure.check(env, tr.self_ty()) {
                let message = need.message(e.message(tr.self_ty()));
                self.report(span, need.code(), message);
            }
            return;
        }
        let nested = match select(tcx, env, tr) {
            Some(Selected::Impl(impl_def_id, impl_args)) => {
                where_clauses(tcx, impl_def_id, impl_args)
            }
            Some(Selected::Builtin(nested)) => nested,
            None => return,
        };
        for tr in nested {
            self.pure_obligation(tr, need, span, depth + 1);
        }
    }

    /// PAR-5 to PAR-7: a closure that `join` (`JoinClosure`) or a slice method (`SharedClosure`)
    /// runs in parallel branches, passed as it is or through `&`, `&mut`, or `Box`. Shared data
    /// it captures, a `&T` borrowed or moved in, must be `Pure + Sync` (`TD0611`, in place of
    /// `TD0610`); the callable must be `Pure + Send`, and a shared one also `Sync` and meet its
    /// `Fn` bound `fn_bound` (`TD0610`), as rustc checks too except for `Pure`.
    fn parallel_closure(
        &mut self,
        ty: Ty<'tcx>,
        need: Need,
        fn_bound: Option<ty::TraitRef<'tcx>>,
        span: Span,
    ) {
        let tcx = self.tcx;
        let Some(pure) = &self.pure else { return };
        let ty = tcx.erase_and_anonymize_regions(ty);
        let tr = ty::TraitRef::new(tcx, pure.trait_def, [ty]);
        if !self.seen_pure.insert((tr, self.owner, need)) {
            return;
        }
        let env = self.pure_env();
        let show = |t: Ty<'tcx>| with_forced_trimmed_paths!(t.to_string());
        let holds = |tr: Option<DefId>, t: Ty<'tcx>| {
            tr.is_none_or(|tr| pure::implements(tcx, env, ty::TraitRef::new(tcx, tr, [t])))
        };
        let (send, sync) = (
            tcx.get_diagnostic_item(sym::Send),
            tcx.lang_items().sync_trait(),
        );
        let shared = need == Need::SharedClosure;
        // The closure behind the adapters: `&F`, `&mut F`, and `Box<F>` call `F`.
        let (mut callee, mut through_mut) = (ty, false);
        loop {
            callee = match *callee.kind() {
                ty::Ref(_, t, m) => {
                    through_mut |= m.is_mut();
                    t
                }
                ty::Adt(def, args) if def.is_box() => args.type_at(0),
                _ => break,
            };
        }
        if let ty::Closure(def, args) = *callee.kind()
            && let Some(local) = def.as_local()
            && !callee.references_error()
            && !callee.has_non_region_infer()
            && !callee.has_escaping_bound_vars()
        {
            // Partial typeck may omit closure_typeinfo's signature entry; captures are optional.
            for cap in tcx.typeck(local).closure_min_captures_flattened(local) {
                // The capture's type at the closure's args, which the backstop gives concretely.
                let place = ty::EarlyBinder::bind(tcx, cap.place.ty())
                    .instantiate(tcx, args.as_closure().parent_args())
                    .skip_normalization();
                let place = tcx.erase_and_anonymize_regions(place);
                if place.references_error()
                    || place.has_non_region_infer()
                    || place.has_escaping_bound_vars()
                {
                    continue;
                }
                let data = match cap.info.capture_kind {
                    ty::UpvarCapture::ByRef(ty::BorrowKind::Immutable) => place,
                    ty::UpvarCapture::ByValue | ty::UpvarCapture::ByUse => match *place.kind() {
                        ty::Ref(_, t, hir::Mutability::Not) => t,
                        _ => continue,
                    },
                    ty::UpvarCapture::ByRef(_) => continue,
                };
                let why = match pure.check(env, data) {
                    Err(e) => e.message(data),
                    Ok(()) if !holds(sync, data) => format!("`{}` is not `Sync`", show(data)),
                    Ok(()) => continue,
                };
                let message = format!(
                    "data a parallel closure shares must be `Pure + Sync`: it shares `{}`, and \
                     {why}",
                    cap.to_string(tcx)
                );
                self.flag(span, Code::Td0611, message);
                return;
            }
        }
        // A callable that calls through a `&mut` is at most `FnMut`, whatever `F` is; rustc's
        // error on the bound can leave `F`'s type unknown, so this is decided first.
        if shared && through_mut {
            let why = format!(
                "`{}` is not `Fn`: a callable through a `&mut` is at most `FnMut`",
                show(ty)
            );
            self.flag(span, need.code(), need.message(why));
            return;
        }
        let Some(ty) = self.pure_ty(ty) else { return };
        let not_fn = fn_bound.is_some_and(|tr| {
            !tr.references_error()
                && !tr.has_non_region_infer()
                && !tr.has_escaping_bound_vars()
                && !pure::implements(tcx, env, tr)
        });
        let why = if let Err(e) = pure.check(env, ty) {
            e.message(ty)
        } else if !holds(send, ty) {
            format!("`{}` is not `Send`", show(ty))
        } else if shared && !holds(sync, ty) {
            format!("`{}` is not `Sync`", show(ty))
        } else if shared && not_fn {
            format!("`{}` is not `Fn`", show(ty))
        } else if shared && !self.reads_captures_only(callee) {
            format!("`{}` is not `Fn`: it moves or mutates a capture", show(ty))
        } else {
            return;
        };
        self.flag(span, need.code(), need.message(why));
    }

    /// Whether a closure's body only reads its captures, so it is `Fn` (02 §5.3). rustc infers a
    /// closure's kind from these uses only when no bound fixes it; under an `Fn` bound it leaves
    /// a move or a mutation to the borrow checker (E0507, E0594, E0596).
    fn reads_captures_only(&self, ty: Ty<'tcx>) -> bool {
        let tcx = self.tcx;
        let ty::Closure(def, _) = *ty.kind() else {
            return true;
        };
        let Some(closure) = def.as_local() else {
            return true;
        };
        // The visitor assumes well-typed code; rustc reports a body that is not.
        let typeck = tcx.typeck(closure);
        if typeck.tainted_by_errors.is_some() {
            return true;
        }
        let body = tcx.hir_body_owned_by(closure);
        let cx = LateContext {
            tcx,
            enclosing_body: Some(body.id()),
            typeck_results: Some(typeck),
            param_env: tcx.param_env(closure),
            effective_visibilities: tcx.effective_visibilities(()),
            last_node_with_lint_attrs: tcx.local_def_id_to_hir_id(closure),
            generics: None,
            only_module: false,
        };
        let mut uses = CaptureUses {
            closure,
            reads_only: true,
        };
        let _ = ExprUseVisitor::for_clippy(&cx, closure, &mut uses).consume_body(body);
        uses.reads_only
    }

    /// 02 §9 (ADMIT-1): `io::Error::new` and `io::Error::other` box their payload as a
    /// `Box<dyn Error + Send + Sync>`, which is `Pure` only because the conversion their
    /// `E: Into<..>` bound performs must reach, through core's `impl<T, U: From<T>> Into<U> for
    /// T`, one of alloc's four `From` impls for that box: from an `E: Error + Send + Sync` that is
    /// `Pure`, or from `String`, `&str`, or `Cow<'_, str>`. Anything else is `TD0517`.
    ///
    /// `tr` is the constructor's `E: Into<Box<dyn Error + Send + Sync>>`.
    fn io_error_payload(&mut self, tr: ty::TraitRef<'tcx>, span: Span) {
        let tcx = self.tcx;
        if tr.references_error() || tr.has_non_region_infer() || tr.has_escaping_bound_vars() {
            return;
        }
        let (Some(pure), Some(from)) = (&self.pure, tcx.get_diagnostic_item(sym::From)) else {
            return;
        };
        let env = self.pure_env();
        let tr = tcx.erase_and_anonymize_regions(tr);
        let Ok(tr) = tcx.try_normalize_erasing_regions(env, Unnormalized::new(tr)) else {
            return;
        };
        if tr.references_error() || tr.has_non_region_infer() {
            return;
        }
        let payload = tr.self_ty();
        let role = |d: DefId| self.crates.role(d.krate);
        let converted = match select(tcx, env, tr) {
            Some(Selected::Impl(blanket, _)) if role(blanket) == Role::Core => {
                let conversion = ty::TraitRef::new(tcx, from, [tr.args.type_at(1), payload]);
                match select(tcx, env, conversion) {
                    Some(Selected::Impl(imp, _)) if role(imp) == Role::Alloc => Some(imp),
                    _ => None,
                }
            }
            _ => None,
        };
        let why = match converted {
            None => format!(
                "`{}` does not convert to `Box<dyn Error + Send + Sync>` through one of alloc's \
                 four `From` impls",
                with_forced_trimmed_paths!(payload.to_string())
            ),
            // The impl from any `E: Error + Send + Sync` boxes `E` unchanged.
            Some(imp)
                if matches!(
                    tcx.impl_trait_ref(imp)
                        .instantiate_identity()
                        .skip_normalization()
                        .args
                        .type_at(1)
                        .kind(),
                    ty::Param(_)
                ) =>
            {
                match pure.check(env, payload) {
                    Err(e) => e.message(payload),
                    Ok(()) => return,
                }
            }
            Some(_) => return,
        };
        self.flag(
            span,
            Code::Td0517,
            format!("an `io::Error` payload must be `Pure`: {why}"),
        );
    }

    /// An opaque type's bounds hold of its hidden type, `Pure` included (PURE-1): the hidden type
    /// meets each `Pure`-implying obligation they incur, those of its associated types' bounds
    /// (`impl Iterator<Item: Show>`) too, which a use meets through the opaque type's bounds
    /// without selecting an impl. As rustc does, each bound is read with the hidden type for the
    /// opaque type.
    fn opaque_bounds(&mut self, opaque: &'tcx hir::OpaqueTy<'tcx>) {
        if !self.can_select() {
            return;
        }
        let tcx = self.tcx;
        let def = opaque.def_id;
        // An RPITIT in a trait has no hidden type unless its method has a default body.
        if let hir::OpaqueTyOrigin::FnReturn {
            parent,
            in_trait_or_impl: Some(hir::RpitContext::Trait),
        }
        | hir::OpaqueTyOrigin::AsyncFn {
            parent,
            in_trait_or_impl: Some(hir::RpitContext::Trait),
        } = opaque.origin
            && !tcx.defaultness(parent).has_value()
        {
            return;
        }
        if self.pure.is_none() {
            return;
        }
        let hidden = tcx.type_of(def).instantiate_identity().skip_normalization();
        if hidden.references_error() {
            return;
        }
        let outer = self.owner.replace(def);
        let span = tcx.def_span(def);
        let itself = ty::AliasTyKind::Opaque {
            def_id: def.to_def_id(),
        };
        let mut to_hidden = ty::BottomUpFolder {
            tcx,
            ty_op: |t: Ty<'tcx>| match *t.kind() {
                ty::Alias(_, alias) if alias.kind == itself => hidden,
                _ => t,
            },
            lt_op: |r| r,
            ct_op: |c| c,
        };
        for bound in tcx
            .explicit_item_bounds(def)
            .iter_identity_copied()
            .filter_map(|b| b.skip_normalization().0.as_trait_clause())
        {
            let tr = tcx.instantiate_bound_regions_with_erased(bound).trait_ref;
            self.pure_obligation(tr.fold_with(&mut to_hidden), Need::Bound, span, 0);
        }
        self.owner = outer;
    }

    // ------------------------------------------------------------ items

    /// GLOBAL-1: a `static` is `Sync + Pure` (rustc checks `Sync`), and safe code has no
    /// `static mut`.
    fn static_item(&mut self, def: LocalDefId, mutability: hir::Mutability) {
        let tcx = self.tcx;
        let span = tcx.def_span(def);
        if mutability.is_mut() {
            self.flag(
                span,
                Code::Td0502,
                "a `static mut` is a mutable global".into(),
            );
            return;
        }
        if !self.can_select() {
            return;
        }
        let Some(pure) = &self.pure else { return };
        let ty = tcx.type_of(def).instantiate_identity().skip_normalization();
        let env = TypingEnv::non_body_analysis(tcx, def.to_def_id());
        if let Err(e) = pure.check(env, ty) {
            let message = format!("a `static` must be `Pure`: {}", e.message(ty));
            self.flag(span, Code::Td0502, message);
        }
    }

    /// The `Pure` obligations of the types a declaration of `owner` names, such as a field's
    /// `NeedsPure<Clock>`, which no expression shows (PURE-1).
    fn declared(&mut self, owner: LocalDefId, ty: Ty<'tcx>, span: Span) {
        let outer = self.owner.replace(owner);
        self.named([ty], span);
        self.owner = outer;
    }

    /// The `Pure` obligations of the ADTs and associated types that `tys` name, which hold
    /// wherever they are named: those of a declaration's types, and of a use's generic args,
    /// which no expression's type shows for a method call or a constant
    /// (`s.make::<Vec<Guard<Clock>>>()` drops a `Guard<Clock>` inside `make`).
    fn named(&mut self, tys: impl IntoIterator<Item = Ty<'tcx>>, span: Span) {
        if self.pure.is_none() {
            return;
        }
        for arg in tys.into_iter().flat_map(|t| t.walk()) {
            let Some(t) = arg.as_type() else { continue };
            match *t.kind() {
                ty::Adt(def, args) => self.use_generic(def.did(), args, span, false),
                ty::Alias(_, alias) => {
                    if let ty::AliasTyKind::Projection { def_id } = alias.kind {
                        self.use_generic(def_id.into(), alias.args, span, false);
                    }
                }
                _ => {}
            }
        }
    }

    /// A function, with a body or not: `unsafe fn` needs the grant (UNSAFE-1; a foreign one, as a
    /// foreign item); a `const fn` takes `Pure + Sync` parameters and none holds a `&mut`
    /// (AFREE-2); a `#[pure]` function's parameter types are `Pure` (AFREE-1), for every
    /// instantiation its bounds allow, so a type parameter needs the bound, and a foreign function
    /// is never authority-free (03 §3.2).
    fn fn_item(&mut self, def: LocalDefId, header: &hir::FnHeader, decl: &'tcx hir::FnDecl<'tcx>) {
        let tcx = self.tcx;
        let span = tcx.def_span(def);
        if header.is_unsafe() && !tcx.is_foreign_item(def) {
            self.granted(span, "an `unsafe fn`");
        }
        if self.recovering {
            return;
        }
        let sig = tcx.fn_sig(def).instantiate_identity().skip_normalization();
        let sig = tcx.instantiate_bound_regions_with_erased(sig);
        for (&input, hir_ty) in sig.inputs().iter().zip(decl.inputs) {
            self.declared(def, input, hir_ty.span);
        }
        if let hir::FnRetTy::Return(hir_ty) = decl.output {
            self.declared(def, sig.output(), hir_ty.span);
        }
        let is_const = matches!(header.constness, hir::Constness::Const { .. });
        let marked = std_attr_made(tcx, self.crates, def, "pure");
        let Some(pure) = &self.pure else { return };
        if marked && tcx.is_foreign_item(def) {
            let message = "a foreign function is never authority-free, so it cannot be `#[pure]`";
            self.flag(span, Code::Td0511, message.into());
            return;
        }
        if !is_const && !marked {
            return;
        }
        if !self.can_select() {
            return;
        }
        let env = TypingEnv::non_body_analysis(tcx, def.to_def_id());
        let sync = tcx.lang_items().sync_trait();
        let show = |t: Ty<'tcx>| with_forced_trimmed_paths!(t.to_string());
        let mut failures = Vec::new();
        for &input in sig.inputs() {
            let input = tcx
                .try_normalize_erasing_regions(env, Unnormalized::new(input))
                .unwrap_or(input);
            if input.references_error() {
                continue;
            }
            let not_pure = pure.check(env, input).err().map(|e| e.message(input));
            if marked && let Some(why) = &not_pure {
                failures.push((
                    Code::Td0511,
                    format!("a `#[pure]` function's parameter types must be `Pure`: {why}"),
                ));
            }
            if is_const {
                let why = if let Some(part) = pure.mut_ref(env, input) {
                    Some(
                        if matches!(part.kind(), ty::Ref(_, _, hir::Mutability::Mut)) {
                            format!("`{}` holds a `{}`", show(input), show(part))
                        } else {
                            format!(
                                "`{}` nests types too deep to search for a `&mut`",
                                show(input)
                            )
                        },
                    )
                } else if let Some(why) = not_pure {
                    Some(why)
                } else if sync.is_some_and(|s| {
                    !pure::implements(tcx, env, ty::TraitRef::new(tcx, s, [input]))
                }) {
                    Some(format!("`{}` is not `Sync`", show(input)))
                } else {
                    None
                };
                if let Some(why) = why {
                    failures.push((
                        Code::Td0518,
                        format!(
                            "a `const fn`'s parameters must be `Pure + Sync` and not `&mut`: {why}"
                        ),
                    ));
                }
            }
        }
        for (code, message) in failures {
            self.flag(span, code, message);
        }
    }

    /// UNSAFE-1: Rust 2024's unsafe attributes need the grant.
    fn unsafe_attrs(&mut self, id: HirId, item_span: Span) {
        for attr in self.tcx.hir_attrs(id) {
            let span = match attr {
                hir::Attribute::Parsed(
                    AttributeKind::NoMangle(span) | AttributeKind::Naked(span),
                )
                | hir::Attribute::Parsed(AttributeKind::ExportName { span, .. }) => *span,
                hir::Attribute::Parsed(AttributeKind::LinkSection { .. }) => item_span,
                _ => continue,
            };
            self.granted(span, "an unsafe attribute");
        }
    }

    /// What rustc proves where a local trait impl is written, which a use of the impl relies on
    /// but selecting it does not show: the trait's where-clauses and supertraits (a `dyn Error`
    /// formats with `E: Debug`), the bounds of its associated types, and its items' own
    /// where-clauses (`impl_item_clauses`).
    fn trait_impl(&mut self, impl_id: LocalDefId) {
        if !self.can_select() {
            return;
        }
        let tcx = self.tcx;
        let tr = tcx
            .impl_trait_ref(impl_id)
            .instantiate_identity()
            .skip_normalization();
        let outer = self.owner.replace(impl_id);
        let span = tcx.def_span(impl_id);
        // An impl of `Pure` itself overlaps std's blanket impl (E0119); only `#[trusted_pure]`
        // declares a type `Pure` (UNSAFE-1).
        let of_pure = self.pure.as_ref().is_some_and(|p| p.trait_def == tr.def_id);
        for t in where_clauses(tcx, tr.def_id, tr.args) {
            if of_pure {
                self.obligation(t, span, 0);
            } else {
                self.follow(t, span);
            }
        }
        for item in tcx.associated_items(impl_id).in_definition_order() {
            let (Some(trait_item), Some(local)) =
                (item.trait_item_def_id(), item.def_id.as_local())
            else {
                continue;
            };
            // rustc reports an item whose generics do not match the trait's (E0049).
            if tcx.compare_impl_item(local).is_err() {
                continue;
            }
            // As rustc's `check_type_bounds` and `compare_impl_item` do, with the impl's types
            // for the trait's.
            let args = ty::GenericArgs::identity_for_item(tcx, item.def_id).rebase_onto(
                tcx,
                impl_id.to_def_id(),
                tr.args,
            );
            self.owner = Some(local);
            let span = tcx.def_span(item.def_id);
            if item.is_type() {
                for bound in tcx
                    .explicit_item_bounds(trait_item)
                    .iter_instantiated_copied(tcx, args)
                {
                    if let Some(tp) = bound.skip_normalization().0.as_trait_clause() {
                        let t = tcx.instantiate_bound_regions_with_erased(tp).trait_ref;
                        self.follow(t, span);
                    }
                }
            }
            self.impl_item_clauses(impl_id, local, trait_item, args);
        }
        self.owner = outer;
    }

    /// A trait impl item's own where-clauses, which its users rely on but no use shows: a call of
    /// a trait's method, or of a `dyn` object's, selects the impl, not the method; drop glue
    /// calls `Drop::drop`; and the bounds of an associated type, checked where its own
    /// where-clauses hold, are met through the trait in generic code (`T::Out<U>: Show`). rustc
    /// proves them from the impl's where-clauses and the trait item's own (E0276), and std's
    /// blanket impl meets any `T: Pure`, so `fn drop(&mut self) where T: Pure` or `type Out<U> =
    /// W<U> where U: Pure` can go beyond both. Their `Pure` obligations are decided where only
    /// those hold, and a clause that repeats one of them (`where Self: Pure`) holds as written;
    /// what they decide is not remembered for the item's own checks, which have its environment.
    fn impl_item_clauses(
        &mut self,
        impl_id: LocalDefId,
        item: LocalDefId,
        trait_item: DefId,
        trait_args: GenericArgsRef<'tcx>,
    ) {
        let tcx = self.tcx;
        let mut assumed = tcx.clauses_of(impl_id).instantiate_identity(tcx).clauses;
        assumed.extend(
            tcx.clauses_of(trait_item)
                .instantiate_own(tcx, trait_args)
                .map(|(c, _)| c),
        );
        let assumed = tcx.mk_clauses_from_iter(assumed.into_iter().map(|c| c.skip_normalization()));
        let param_env =
            normalize_param_env_or_error(tcx, ty::ParamEnv::new(assumed), ObligationCause::dummy());
        let erased: FxHashSet<_> = param_env
            .caller_bounds()
            .iter()
            .map(|c| tcx.erase_and_anonymize_regions(c))
            .collect();
        self.item_env = Some(TypingEnv::new(
            param_env,
            ty::TypingMode::non_body_analysis(),
        ));
        let seen = std::mem::take(&mut self.seen_pure);
        for (clause, span) in tcx.clauses_of(item).instantiate_own_identity() {
            let clause = clause.skip_normalization();
            if erased.contains(&tcx.erase_and_anonymize_regions(clause)) {
                continue;
            }
            if let Some(tp) = clause.as_trait_clause() {
                let tr = tcx.instantiate_bound_regions_with_erased(tp).trait_ref;
                self.pure_obligation(tr, Need::Bound, span, 0);
            }
        }
        self.item_env = None;
        self.seen_pure = seen;
    }

    /// Checks the impl selected for `tr`, then the obligations of its where-clauses, or those of
    /// the builtin impl that meets `tr`. It checks nothing where rustc reports an error, where
    /// `tr` is met by a where-clause of the caller (checked where the caller is used), by an
    /// object candidate, or by an auto trait's builtin impl.
    fn obligation(&mut self, tr: ty::TraitRef<'tcx>, span: Span, depth: usize) {
        let tcx = self.tcx;
        if !self.obligations || !self.can_select() {
            return;
        }
        if depth > MAX_OBLIGATION_DEPTH {
            let key = with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(
                format!("{}: {}", tr.self_ty(), tr.print_only_trait_path())
            )));
            let rejection = Rejection {
                key,
                code: Code::Td0520,
                help: None,
            };
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
                match self.admit_impl(impl_def_id, tr) {
                    Err(r) => self.reject(span, r, ""),
                    Ok(admitted) => self.admitted(span, admitted),
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

    /// With `require_attr`, a binary's `main` must be the one `#[tide::main]` generates, which
    /// starts the runtime (ROOT-1), so an imported `main` never is; `check_expanded` rejects
    /// `#![no_main]`. rustc's entry point reports what a user's `main` returns through its
    /// `Termination` impl, which no expression shows: `fn main() -> Result<(), fn()>` formats the
    /// error. A `main` the toolchain generates returns a type it fixes (`()` for the test harness).
    fn entry_point(&mut self, require_attr: bool) {
        let tcx = self.tcx;
        let Some((main, _)) = tcx.entry_fn(()) else {
            return;
        };
        if self.generated_by_toolchain(tcx.def_span(main)) {
            return;
        }
        if require_attr {
            let span = match tcx.resolutions(()).main_def {
                Some(def) if def.is_import => def.span,
                _ => tcx.def_span(main),
            };
            let message = "`main` does not carry `#[tide::main]`, which starts the runtime".into();
            let help = Some("write `#[tide::main] fn main` in the crate root");
            self.diags
                .report(tcx, span.source_callsite(), Code::Td0512, message, help);
        }
        let Some(local) = main.as_local() else { return };
        if self.recovering {
            return;
        }
        let Some(termination) = tcx.lang_items().termination() else {
            return;
        };
        let sig = tcx.fn_sig(main).instantiate_identity().skip_normalization();
        let ret = tcx.instantiate_bound_regions_with_erased(sig.output());
        let outer = self.owner.replace(local);
        let tr = ty::TraitRef::new(tcx, termination, [ret]);
        self.follow(tr, tcx.def_span(main));
        self.owner = outer;
    }

    /// ROOT-5: no code names the binary's `main`, to call it, take it as a value, or import it,
    /// so nothing but the entry point mints a real `Root`. The toolchain's expansions count: the
    /// entry wiring never names `main`, but a `#[test]` on it would call it from a test.
    fn main_named(&mut self, def_id: DefId, span: Span) {
        if self.crates.is_main(def_id) {
            let message = "no code may call or name `main`, which starts the runtime and mints its \
                           `Root`";
            let help = Some("tests call the functions that take capabilities, with `Root::sim`");
            self.diags.report(
                self.tcx,
                span.source_callsite(),
                Code::Td0506,
                message.into(),
                help,
            );
        }
    }

    /// Every macro expansion in the crate, by the macro's definition, including expansions whose
    /// output lands in attributes (`#[doc = include_str!(..)]`).
    fn check_expansions(&mut self) {
        // rustc has no public iterator over a crate's expansions; ids are dense, so a fresh
        // empty one bounds them.
        let end = LocalExpnId::fresh_empty().as_u32();
        for i in 1..end {
            let data = LocalExpnId::from_u32(i).expn_data();
            if self.habit_expansion(&data) || self.surface_expansion(&data) {
                continue;
            }
            match (data.kind, data.macro_def_id) {
                (ExpnKind::Macro(kind, _), Some(mac)) => {
                    self.use_def(mac, data.call_site);
                    if kind == MacroKind::Bang {
                        self.sink_expansion(mac, data.call_site);
                    }
                    // UNSAFE-1: `#[trusted_pure]` is a trust hole.
                    if kind == MacroKind::Attr
                        && !self.grants.unsafe_
                        && is_std_attr(self.tcx, self.crates, mac, "trusted_pure")
                    {
                        let message = format!("`#[trusted_pure]` {GRANT}");
                        self.flag(data.call_site, Code::Td0901, message);
                    }
                }
                // A macro that did not resolve: Rust's lazy globals and thread-locals, which
                // Tide's std has no form of (GLOBAL-2, GLOBAL-3).
                (ExpnKind::Macro(MacroKind::Bang, name), None) => {
                    let what = match name.as_str().rsplit("::").next() {
                        Some("lazy_static") => "`lazy_static!` declares a lazy global",
                        Some("thread_local") => "`thread_local!` declares a thread-local",
                        _ => continue,
                    };
                    let message = format!("{what}; Tide has none");
                    self.flag(data.call_site, Code::Td0502, message);
                }
                _ => {}
            }
        }
    }
}

/// Whether a macro of another crate wrote the code at `span`, such as the `unsafe impl` that
/// `derive(Clone)` emits: its unsafety is that crate's trust hole, not the package's (UNSAFE-1).
/// Code a local macro writes, or that a macro's caller passes in, is the package's.
fn written_by_other_crate(span: Span) -> bool {
    span.from_expansion()
        && span
            .ctxt()
            .outer_expn_data()
            .macro_def_id
            .is_some_and(|m| !m.is_local())
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

/// The `Drop` impl of the local type `def_id`, or of the type it constructs (`.map(Guard)` builds
/// values no expression's type shows), at the args that `args` give its parameters. Drop glue
/// relies on the impl's where-clauses, but no obligation shows them, and they can bound a
/// parameter by `Pure` where the type does not: rustc proves a `Drop` impl's bounds from the
/// type's (E0367), and std's blanket impl meets `T: Pure`. The impl's self type is the type at
/// distinct parameters (E0366), so each impl parameter is one of the type's.
fn local_drop_impl<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
    args: GenericArgsRef<'tcx>,
) -> Option<(DefId, GenericArgsRef<'tcx>)> {
    if !def_id.is_local() {
        return None;
    }
    let adt = match tcx.def_kind(def_id) {
        DefKind::Struct | DefKind::Enum | DefKind::Union => def_id,
        DefKind::Ctor(CtorOf::Struct, _) => tcx.parent(def_id),
        DefKind::Ctor(CtorOf::Variant, _) => tcx.parent(tcx.parent(def_id)),
        _ => return None,
    };
    let impl_def_id = tcx.parent(tcx.adt_destructor(adt)?.did);
    let ty::Adt(_, params) = *tcx
        .type_of(impl_def_id)
        .instantiate_identity()
        .skip_normalization()
        .kind()
    else {
        return None;
    };
    let mut impl_args = vec![None; tcx.generics_of(impl_def_id).count()];
    for (param, arg) in params.iter().zip(args) {
        let index = match param.kind() {
            ty::GenericArgKind::Type(t) => match *t.kind() {
                ty::Param(p) => p.index,
                _ => return None,
            },
            ty::GenericArgKind::Lifetime(r) => match r.kind() {
                ty::ReEarlyParam(p) => p.index,
                _ => return None,
            },
            ty::GenericArgKind::Const(c) => match c.kind() {
                ty::ConstKind::Param(p) => p.index,
                _ => return None,
            },
        };
        *impl_args.get_mut(index as usize)? = Some(arg);
    }
    let impl_args: Option<Vec<_>> = impl_args.into_iter().collect();
    Some((impl_def_id, tcx.mk_args(&impl_args?)))
}

/// A trait impl's allowlist key, `impl <Trait> for <SelfTy>`. core's impls for every function
/// pointer (`impl<F: FnPtr> Hash for F`) are keyed `for fn()`, so the rules can tell them from
/// other blanket impls.
fn impl_key(tcx: TyCtxt<'_>, impl_def_id: DefId) -> String {
    let generic = tcx
        .impl_trait_ref(impl_def_id)
        .instantiate_identity()
        .skip_normalization();
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
    with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(format!(
        "impl {} for {}",
        generic.print_only_trait_path(),
        if for_fn_ptrs {
            "fn()".to_string()
        } else {
            generic.self_ty().to_string()
        }
    ))))
}

fn raw_pointer_key(m: hir::Mutability) -> &'static str {
    if m.is_mut() { "*mut T" } else { "*const T" }
}

/// Struct/variant expressions and patterns can omit node_args; their ADT still owns them.
fn path_args<'tcx>(
    tcx: TyCtxt<'tcx>,
    typeck: &ty::TypeckResults<'tcx>,
    def: DefId,
    id: HirId,
) -> GenericArgsRef<'tcx> {
    let args = typeck.node_args(id);
    if args.len() < tcx.generics_of(def).count()
        && matches!(
            tcx.def_kind(def),
            DefKind::Struct | DefKind::Variant | DefKind::Ctor(..)
        )
        && let Some(ty) = typeck.node_type_opt(id)
        && let ty::Adt(_, args) = ty.kind()
    {
        return args;
    }
    args
}

/// The trait refs among `def_id`'s where-clauses (a trait's include its supertraits), at `args`.
pub(crate) fn where_clauses<'tcx>(
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

pub(crate) enum Selected<'tcx> {
    /// A user-written impl, at its args.
    Impl(DefId, GenericArgsRef<'tcx>),
    /// A builtin impl's obligations on the type's parts: a tuple's or a closure's `Clone`
    /// clones each element or capture.
    Builtin(Vec<ty::TraitRef<'tcx>>),
}

/// What satisfies `tr`, if selection finds exactly one impl. An auto trait's builtin impl is
/// not followed: it runs no code, and its obligations reach the private fields of std's types.
pub(crate) fn select<'tcx>(
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

/// The uses of one closure's captures that make it not `Fn`: a move out of one, a mutation, or
/// a mutable or unique borrow.
struct CaptureUses {
    closure: LocalDefId,
    reads_only: bool,
}

impl CaptureUses {
    fn note(&mut self, place: &PlaceWithHirId<'_>) {
        if matches!(place.place.base, PlaceBase::Upvar(id) if id.closure_expr_id == self.closure) {
            self.reads_only = false;
        }
    }
}

impl<'tcx> Delegate<'tcx> for CaptureUses {
    fn consume(&mut self, place: &PlaceWithHirId<'tcx>, _: HirId) {
        self.note(place);
    }
    fn use_cloned(&mut self, _: &PlaceWithHirId<'tcx>, _: HirId) {}
    fn borrow(&mut self, place: &PlaceWithHirId<'tcx>, _: HirId, kind: ty::BorrowKind) {
        if kind != ty::BorrowKind::Immutable {
            self.note(place);
        }
    }
    fn mutate(&mut self, place: &PlaceWithHirId<'tcx>, _: HirId) {
        self.note(place);
    }
    fn fake_read(&mut self, _: &PlaceWithHirId<'tcx>, _: FakeReadCause, _: HirId) {}
}

impl<'a, 'tcx> Visitor<'tcx> for Walk<'a, 'tcx> {
    type NestedFilter = nested_filter::All;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        self.visit_boundary(false, |walk| {
            // The item holds the trust holes found outside its bodies.
            walk.item = Some(item.owner_id.def_id);
            let def = item.owner_id.def_id;
            walk.surface_item(item);
            let valid_bounds = !walk.recovering && walk.ceiling_item(item);
            match item.kind {
                hir::ItemKind::Impl(hir::Impl {
                    of_trait: Some(header),
                    ..
                }) => {
                    // Do not ask selection to solve a bound that already exceeds the ceiling.
                    if valid_bounds {
                        walk.trait_impl(def);
                    }
                    if header.safety.is_unsafe() && !written_by_other_crate(item.span) {
                        walk.granted(walk.tcx.def_span(def), "an `unsafe impl`");
                    }
                }
                hir::ItemKind::Static(mutability, ..) => walk.static_item(def, mutability),
                // An empty block still links the native libraries its `#[link]` names (UNSAFE-1).
                hir::ItemKind::ForeignMod { .. } => {
                    walk.granted(walk.tcx.def_span(def), "an `unsafe extern` block")
                }
                hir::ItemKind::Struct(..) | hir::ItemKind::Enum(..) | hir::ItemKind::Union(..) => {
                    if !walk.recovering {
                        for field in walk.tcx.adt_def(def).all_fields() {
                            let ty = walk.tcx.type_of(field.did).instantiate_identity();
                            let span = walk.tcx.def_span(field.did);
                            walk.declared(def, ty.skip_normalization(), span);
                        }
                    }
                }
                _ => {}
            }
            if let hir::ItemKind::Impl(imp) = item.kind
                && !walk.recovering
            {
                let ty = walk.tcx.type_of(def).instantiate_identity();
                walk.declared(def, ty.skip_normalization(), imp.self_ty.span);
                if imp.of_trait.is_some() {
                    let tr = walk.tcx.impl_trait_ref(def).instantiate_identity();
                    for t in tr.skip_normalization().args.types().skip(1) {
                        walk.declared(def, t, walk.tcx.def_span(def));
                    }
                }
            }
            walk.unsafe_attrs(item.hir_id(), item.span);
            intravisit::walk_item(walk, item);
        });
    }

    fn visit_impl_item(&mut self, item: &'tcx hir::ImplItem<'tcx>) {
        self.visit_boundary(false, |walk| {
            // The item holds the trust holes found outside its bodies.
            walk.item = Some(item.owner_id.def_id);
            if let hir::ImplItemKind::Type(hir_ty) = item.kind {
                walk.ceiling_associated_type(item.generics, item.span);
                let def = item.owner_id.def_id;
                if !walk.recovering {
                    let ty = walk.tcx.type_of(def).instantiate_identity();
                    walk.declared(def, ty.skip_normalization(), hir_ty.span);
                }
            }
            walk.contract_placement(item.owner_id.def_id);
            walk.unsafe_attrs(item.hir_id(), item.span);
            intravisit::walk_impl_item(walk, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'tcx hir::TraitItem<'tcx>) {
        self.visit_boundary(false, |walk| {
            // The item holds the trust holes found outside its bodies.
            walk.item = Some(item.owner_id.def_id);
            if let hir::TraitItemKind::Type(..) = item.kind {
                walk.ceiling_associated_type(item.generics, item.span);
            }
            // A method with a body is checked from `visit_fn`.
            if let hir::TraitItemKind::Fn(sig, hir::TraitFn::Required(_)) = &item.kind {
                walk.fn_item(item.owner_id.def_id, &sig.header, sig.decl);
            }
            walk.contract_placement(item.owner_id.def_id);
            walk.unsafe_attrs(item.hir_id(), item.span);
            intravisit::walk_trait_item(walk, item);
        });
    }

    fn visit_foreign_item(&mut self, item: &'tcx hir::ForeignItem<'tcx>) {
        self.visit_boundary(false, |walk| {
            // The item holds the trust holes found outside its bodies.
            walk.item = Some(item.owner_id.def_id);
            // A `safe` item reaches foreign code too, without an `unsafe` block.
            walk.granted(item.span, "a foreign item");
            if let hir::ForeignItemKind::Fn(sig, ..) = &item.kind {
                walk.fn_item(item.owner_id.def_id, &sig.header, sig.decl);
            }
            walk.unsafe_attrs(item.hir_id(), item.span);
            intravisit::walk_foreign_item(walk, item);
        });
    }

    fn visit_fn(
        &mut self,
        kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: hir::BodyId,
        _: Span,
        def: LocalDefId,
    ) {
        if let Some(header) = kind.header() {
            self.fn_item(def, header, decl);
        }
        self.contract_function(def, body);
        intravisit::walk_fn(self, kind, decl, body, def)
    }

    /// An unsafe block needs the grant (UNSAFE-1) and a SAFETY comment (TD0902).
    fn visit_block(&mut self, block: &'tcx hir::Block<'tcx>) {
        if let hir::BlockCheckMode::UnsafeBlock(hir::UnsafeSource::UserProvided) = block.rules {
            self.granted(block.span, "an `unsafe` block");
            self.safety_comment(block);
        }
        intravisit::walk_block(self, block)
    }

    fn visit_nested_body(&mut self, id: hir::BodyId) {
        self.visit_boundary(self.recovering, |walk| {
            let owner = walk.tcx.hir_body_owner_def_id(id);
            walk.typeck = if walk.recovering {
                None
            } else {
                Some(walk.tcx.typeck_body(id))
            };
            walk.owner = Some(owner);
            let body = walk.tcx.hir_body(id);
            if walk.can_select()
                && walk.typeck_of(body.value.hir_id).is_some()
                && let Some(output) = walk.surface_return_type()
            {
                walk.owned_value(body.value, Some(output), &[], &mut FxHashSet::default());
            }
            walk.visit_body(body);
        });
    }

    fn visit_path(&mut self, path: &hir::Path<'tcx>, id: HirId) {
        match path.res {
            Res::Def(_, def_id) => {
                self.authority_use(def_id, path.span);
                self.root_name(def_id, path, id);
                self.use_def(def_id, path.span);
                self.main_named(def_id, path.span);
                // Typeck records no args for the lang-item calls that `?` and `for` desugar to
                // (`FromResidual::from_residual`, `IntoIterator::into_iter`); their type has them.
                let tcx = self.tcx;
                let args = self.recovered_typeck_of(id).and_then(|t| {
                    t.node_args_opt(id)
                        .or_else(|| match *t.node_type_opt(id)?.kind() {
                            ty::FnDef(d, args) if d == def_id => {
                                Some(tcx.instantiate_bound_regions_with_erased(args))
                            }
                            _ => None,
                        })
                });
                if let Some(args) = args {
                    self.use_generic(def_id, args, path.span, true);
                    self.named(args.types(), path.span);
                }
            }
            Res::Err => self.authority_unresolved(path, id),
            _ => {}
        }
        intravisit::walk_path(self, path)
    }

    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: HirId, _span: Span) {
        if let (hir::QPath::TypeRelative(..), Some(t)) = (qpath, self.recovered_typeck_of(id)) {
            if let Some(def_id) = t.type_dependent_def_id(id) {
                let span = qpath.span();
                self.authority_use(def_id, span);
                self.use_def(def_id, span);
                let args = path_args(self.tcx, t, def_id, id);
                self.use_generic(def_id, args, span, true);
                self.named(args.types(), span);
            }
        }
        self.authority_qpath(qpath, id);
        intravisit::walk_qpath(self, qpath, id)
    }

    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        if let Some(t) = self.recovered_typeck_of(e.hir_id) {
            self.authority_expr(e, t);
            self.lint_sink_expr(e, t);
            self.surface_expr(e, t);
            self.fn_pointer_address(e, t);
            if let Some(ty) = t
                .node_type_opt(e.hir_id)
                .filter(|ty| !ty.references_error())
            {
                self.use_ty(ty, e.span);
            }
            // Method calls, overloaded operators, and indexing.
            let callee = t.type_dependent_def_id(e.hir_id);
            if let Some(def_id) = callee {
                self.authority_use(def_id, e.span);
                self.use_def(def_id, e.span);
            }
            self.ceiling_call(e, t);
            if let Some(def_id) = callee {
                let args = path_args(self.tcx, t, def_id, e.hir_id);
                self.use_generic(def_id, args, e.span, true);
                self.named(args.types(), e.span);
            }
            self.raw_place(e, t);
            let mut source = t.node_type_opt(e.hir_id);
            for adj in t.expr_adjustments(e) {
                self.use_ty(adj.target, e.span);
                // An unsizing coercion to `dyn Trait` selects the source's impl of `Trait`.
                if let (Adjust::Pointer(PointerCoercion::Unsize), Some(src)) = (&adj.kind, source) {
                    if let Some(tr) = unsized_trait_ref(self.tcx, src, adj.target) {
                        self.follow(tr, e.span);
                    }
                }
                // A fn item coerced to a `fn` pointer (CLOSURE-3).
                if let (Adjust::Pointer(PointerCoercion::ReifyFnPointer(_)), Some(src)) =
                    (&adj.kind, source)
                {
                    self.reify(e, src, t.hir_owner.def_id);
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
                            self.follow(ty::TraitRef::new(self.tcx, tr, [src]), e.span);
                        }
                    }
                }
                source = Some(adj.target);
            }
        }
        intravisit::walk_expr(self, e)
    }

    fn visit_stmt(&mut self, stmt: &'tcx hir::Stmt<'tcx>) {
        self.habit_stmt(stmt);
        intravisit::walk_stmt(self, stmt);
    }

    fn visit_pat(&mut self, p: &'tcx hir::Pat<'tcx>) {
        if let Some(t) = self.recovered_typeck_of(p.hir_id)
            && let Some(ty) = t.node_type_opt(p.hir_id)
        {
            self.use_ty(ty, p.span);
            // ROOT-4: a union pattern reads its fields' bytes, as a field expression does.
            if let (hir::PatKind::Struct(_, fields, _), ty::Adt(def, _)) = (p.kind, ty.kind())
                && def.is_union()
            {
                for field in fields {
                    if let Some(made) = t.node_type_opt(field.pat.hir_id) {
                        self.forged(made, field.span);
                    }
                }
            }
        }
        intravisit::walk_pat(self, p)
    }

    fn visit_fn_ret_ty(&mut self, output: &'tcx hir::FnRetTy<'tcx>) {
        if let hir::FnRetTy::Return(output) = output {
            self.surface_output(output);
        }
        intravisit::walk_fn_ret_ty(self, output);
    }

    fn visit_generic_param(&mut self, param: &'tcx hir::GenericParam<'tcx>) {
        self.ceiling_parameter(param);
        intravisit::walk_generic_param(self, param);
    }

    fn visit_generic_args(&mut self, args: &'tcx hir::GenericArgs<'tcx>) {
        if args.parenthesized == hir::GenericArgsParentheses::ParenSugar {
            for constraint in args.constraints {
                if let Some(output) = constraint.ty() {
                    self.surface_output(output);
                }
            }
        }
        intravisit::walk_generic_args(self, args);
    }

    /// Raw pointer types written in signatures and fields, which no expression's type shows.
    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx, hir::AmbigArg>) {
        match t.kind {
            hir::TyKind::Ptr(mt) => {
                self.check(t.span, self.admit_pseudo(raw_pointer_key(mt.mutbl)))
            }
            hir::TyKind::OpaqueDef(opaque) => self.opaque_bounds(opaque),
            _ => {}
        }
        intravisit::walk_ty(self, t)
    }
}
