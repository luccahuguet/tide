//! Crate identity (ADMIT-1 keys the allowlist on it, not on crate names): which crate in the
//! graph is `core`, `alloc`, Rust's std, and Tide's std, what Tide's std exports, and which
//! expansion generated the binary's entry point.

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::ItemKind;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{CRATE_DEF_ID, CrateNum, DefId, LOCAL_CRATE};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::print::{
    with_no_trimmed_paths, with_no_visible_paths, with_resolve_crate_name,
};
use rustc_session::config::CrateType;
use rustc_span::hygiene::{AstPass, ExpnId};
use rustc_span::{ExpnKind, sym};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The crate being checked.
    Local,
    /// The crate rustc's std injection loads as `std`, unless that is Rust's std, `core`, or
    /// `alloc`.
    TideStd,
    /// The crate that defines the `Sized` lang item.
    Core,
    /// The crate that defines the `Box` lang item.
    Alloc,
    /// The crate that defines the `start` lang item.
    RustStd,
    /// Anything else, such as the toolchain's proc-macro crates.
    Other,
}

pub struct Crates {
    tide_std: Option<CrateNum>,
    core: Option<CrateNum>,
    alloc: Option<CrateNum>,
    rust_std: Option<CrateNum>,
    /// Whether the injected std is Rust's, which has no capabilities.
    rust_std_injected: bool,
    /// Items of other crates that Tide's std re-exports publicly.
    exports: FxHashSet<DefId>,
    /// The expansion that generated the crate-root `fn main` of an executable crate.
    entry: Option<ExpnId>,
}

impl Crates {
    pub fn new(tcx: TyCtxt<'_>) -> Crates {
        let lang = tcx.lang_items();
        let mut crates = Crates {
            tide_std: injected_std(tcx),
            core: lang.sized_trait().map(|d| d.krate),
            alloc: lang.owned_box().map(|d| d.krate),
            rust_std: lang.start_fn().map(|d| d.krate),
            rust_std_injected: false,
            exports: FxHashSet::default(),
            entry: entry_expansion(tcx),
        };
        // Built without Tide's std, the injected std is Rust's, and the allowlist checks it; an
        // `--extern std=` naming `core` or `alloc` does not make them Tide's std either.
        crates.rust_std_injected = crates.rust_std.is_some() && crates.tide_std == crates.rust_std;
        if [crates.rust_std, crates.core, crates.alloc].contains(&crates.tide_std) {
            crates.tide_std = None;
        }
        if let Some(krate) = crates.tide_std {
            collect_exports(
                tcx,
                krate.as_def_id(),
                krate,
                &mut FxHashSet::default(),
                &mut crates.exports,
            );
        }
        crates
    }

    pub fn role(&self, krate: CrateNum) -> Role {
        let is = |c: Option<CrateNum>| c == Some(krate);
        if krate == LOCAL_CRATE {
            Role::Local
        } else if is(self.tide_std) {
            Role::TideStd
        } else if is(self.core) {
            Role::Core
        } else if is(self.alloc) {
            Role::Alloc
        } else if is(self.rust_std) {
            Role::RustStd
        } else {
            Role::Other
        }
    }

    /// Whether the injected std is Rust's, which has no capabilities.
    pub fn rust_std_injected(&self) -> bool {
        self.rust_std_injected
    }

    pub fn tide_std(&self) -> Option<CrateNum> {
        self.tide_std
    }

    /// Rust's std, where a path Tide's std does not export is looked up.
    pub fn rust_std(&self) -> Option<CrateNum> {
        self.rust_std
    }

    /// Whether Tide's std re-exports `def_id`, which makes it part of Tide's std's API.
    pub fn exported(&self, def_id: DefId) -> bool {
        self.exports.contains(&def_id)
    }

    /// Whether `expn` generated the crate-root `fn main` of an executable crate, the one place
    /// where `#[tide::main]`'s expansion is the binary's entry point (ROOT-1): applied to any
    /// other `fn main`, it would mint a `Root` that some code can call.
    pub fn is_entry(&self, expn: ExpnId) -> bool {
        self.entry == Some(expn)
    }

    /// Whether `def_id` is an item of `core`, `alloc`, Rust's std, or Tide's std.
    pub fn is_std(&self, def_id: DefId) -> bool {
        matches!(
            self.role(def_id.krate),
            Role::TideStd | Role::Core | Role::Alloc | Role::RustStd
        )
    }

    /// The key the rules match an item by: its canonical path in its defining crate, with Tide's
    /// std printed as `tide_std` whatever its crate is named.
    pub fn key(&self, tcx: TyCtxt<'_>, def_id: DefId) -> String {
        let path = with_no_visible_paths!(with_no_trimmed_paths!(with_resolve_crate_name!(
            tcx.def_path_str(def_id)
        )));
        if self.role(def_id.krate) == Role::TideStd {
            let name = tcx.crate_name(def_id.krate);
            if let Some(rest) = path
                .strip_prefix(name.as_str())
                .and_then(|p| p.strip_prefix("::"))
            {
                return format!("tide_std::{rest}");
            }
        }
        path
    }
}

/// The crate behind the `extern crate std` that rustc's std injection adds to the local crate.
/// A user's `extern crate core as std;` is not it, so it cannot make `core` Tide's std.
fn injected_std(tcx: TyCtxt<'_>) -> Option<CrateNum> {
    tcx.hir_free_items().find_map(|id| {
        let item = tcx.hir_item(id);
        let injected = matches!(
            item.span.ctxt().outer_expn_data().kind,
            ExpnKind::AstPass(AstPass::StdImports)
        );
        match item.kind {
            ItemKind::ExternCrate(_, ident) if ident.name == sym::std && injected => {
                tcx.extern_mod_stmt_cnum(item.owner_id.def_id)
            }
            _ => None,
        }
    })
}

/// The expansion that generated the crate-root `fn main`, if the crate is executable (a binary,
/// or a test harness) and an expansion generated it.
fn entry_expansion(tcx: TyCtxt<'_>) -> Option<ExpnId> {
    if !tcx.crate_types().contains(&CrateType::Executable) {
        return None;
    }
    tcx.hir_free_items().find_map(|id| {
        let def_id = id.owner_id.def_id;
        let span = tcx.def_span(def_id);
        (tcx.def_kind(def_id) == DefKind::Fn
            && tcx.item_name(def_id) == sym::main
            && tcx.opt_local_parent(def_id) == Some(CRATE_DEF_ID)
            && span.from_expansion())
        .then(|| span.ctxt().outer_expn())
    })
}

/// Every item of another crate that a public module of `krate`, reached from `module`,
/// re-exports publicly.
fn collect_exports(
    tcx: TyCtxt<'_>,
    module: DefId,
    krate: CrateNum,
    visited: &mut FxHashSet<DefId>,
    out: &mut FxHashSet<DefId>,
) {
    if !visited.insert(module) {
        return;
    }
    for child in tcx.module_children(module) {
        let Res::Def(kind, def_id) = child.res else {
            continue;
        };
        if !child.vis.is_public() {
            continue;
        }
        if def_id.krate != krate {
            out.insert(def_id);
        } else if kind == DefKind::Mod {
            collect_exports(tcx, def_id, krate, visited, out);
        }
    }
}
