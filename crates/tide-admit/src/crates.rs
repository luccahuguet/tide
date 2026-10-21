//! Crate identity (ADMIT-1 keys the allowlist on it, not on crate names): which crate in the
//! graph is `core`, `alloc`, Rust's std, and Tide's std, and what Tide's std exports.

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::ItemKind;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{CrateNum, DefId, LOCAL_CRATE};
use rustc_middle::ty::TyCtxt;
use rustc_span::hygiene::AstPass;
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
    /// Items of other crates that Tide's std re-exports publicly.
    exports: FxHashSet<DefId>,
}

impl Crates {
    pub fn new(tcx: TyCtxt<'_>) -> Crates {
        let lang = tcx.lang_items();
        let mut crates = Crates {
            tide_std: injected_std(tcx),
            core: lang.sized_trait().map(|d| d.krate),
            alloc: lang.owned_box().map(|d| d.krate),
            rust_std: lang.start_fn().map(|d| d.krate),
            exports: FxHashSet::default(),
        };
        // Built without Tide's std, the injected std is Rust's, and the allowlist checks it; an
        // `--extern std=` naming `core` or `alloc` does not make them Tide's std either.
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
