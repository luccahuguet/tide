//! The package interface's trust artifact (D11; 03 §4.1): each exported item's declared
//! authority, with the access path that carries each kind, and the crate's trust holes. It is
//! written beside the metadata of every crate admitted against Tide's std, as
//! `<crate>.interface.json`; a crate whose `std` is neither Tide's nor Rust's fails the check
//! instead. v0 reports declared authority only: no exercised authority, effects, contracts, or
//! proof status.
//!
//! Declared authority comes from types alone. A function's is what each parameter's type
//! reaches, `self` included, under the parameter's name (`#<n>` for one a pattern binds); a
//! struct's, enum's, or union's is what its fields reach; a type alias's, `const`'s, or
//! `static`'s is what its type reaches. Another crate's item that the crate re-exports is listed
//! under the exporting path. A `Pure` type reaches nothing (`pure.rs` decides it).
//!
//! Paths follow fields (`app.db.net`), tuple positions (`pair.0`), and enum variants
//! (`event.Sent.0`); the empty path is the value itself. References, arrays, slices, and std's
//! generic types, handles included (`Arc<Net>` carries `sync` and `net`), keep their holder's
//! path. Authority a type does not show is `opaque(<type>)`: through a type parameter or
//! `dyn Trait` that is not `Pure`, an opaque type without `Pure` in its bounds, a raw pointer, an
//! erased payload, recursion whose type arguments grow twice, or a type too deep or too large to
//! search. A foreign type or function pointer is `foreign`. So an item the artifact lists with
//! no authority has none.
//!
//! The holes are every site the `unsafe` grant admits, by the innermost item that holds it
//! (`walk.rs` and its backstop find them; `Holes`). A granted impl that a generic item reaches
//! only through std's internals is held where an instance runs: by that item when a local
//! instance of it does, unless an item that instantiates it, whose types may have chosen the
//! impl, holds it already; else in the crate that instantiates it, as the backstop checks it
//! there. One that no local item reaches, as a `global_asm!` operand's, is the crate root's.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;

use rustc_abi::ExternAbi;
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingEnv, Unnormalized};

use crate::crates::Crates;
use crate::pure::{self, Carries, Pure};
use crate::walk::{Holes, crate_pure};

mod diff;
pub use diff::diff;

/// The artifact's format, which a reader checks before it reads the rest.
const FORMAT: &str = "tide-interface/1";

/// How many types the search expands one inside another before it reports `opaque`, as the
/// `Pure` walk gives up (`pure.rs`).
const MAX_DEPTH: usize = 64;

/// How many types the search of one item visits before it reports the rest as `opaque`: a
/// type whose recursion changes its type arguments (`Poly<(T, T)>`) doubles at each level.
const BUDGET: usize = 4096;

/// Access path to the kinds it carries.
type Authority = BTreeMap<String, BTreeSet<String>>;

/// Writes the artifact into `out`, except for a crate built on Rust's std, which has no
/// capabilities: an operator that passes Rust's std as `--tide-std`, as the driver's cache tests
/// do. It fails when Tide's std, which alone defines them, is not the crate's `std` otherwise, as
/// when the operator's `--tide-std` is `core` or `alloc`, since no authority can then be decided;
/// the check rejects `#![no_std]` before this runs.
pub fn write(tcx: TyCtxt<'_>, holes: &Holes, out: &Path) -> std::io::Result<()> {
    let crates = Crates::new(tcx);
    let Some(pure) = crate_pure(tcx, &crates) else {
        if crates.rust_std_injected() {
            return Ok(());
        }
        let why = "Tide's std is not the crate's `std`, so no authority can be decided";
        return Err(std::io::Error::other(why));
    };
    let search = Search {
        tcx,
        crates: &crates,
        pure: &pure,
        budget: Cell::new(0),
    };
    let visibilities = tcx.effective_visibilities(());
    let mut items: BTreeMap<(String, String), Authority> = BTreeMap::new();
    for def in tcx.hir_crate_items(()).definitions() {
        if visibilities.is_exported(def)
            && let Some(authority) = search.item(def.to_def_id())
        {
            let key = (
                crates.key(tcx, def.to_def_id()),
                tcx.def_descr(def.to_def_id()).into(),
            );
            items.insert(key, authority);
        }
    }
    // Another crate's items the crate re-exports, under the path that exports them. A
    // re-exported module's items stay its crate's.
    let modules = tcx.hir_crate_items(()).definitions();
    for module in modules.filter(|&d| tcx.def_kind(d) == DefKind::Mod) {
        for child in tcx.module_children_local(module) {
            let Res::Def(_, def) = child.res else {
                continue;
            };
            let exported = child
                .reexport_chain
                .first()
                .and_then(|r| r.id())
                .and_then(|id| id.as_local())
                .is_some_and(|id| visibilities.is_exported(id));
            if def.is_local() || !exported {
                continue;
            }
            if let Some(authority) = search.item(def) {
                let path = format!("{}::{}", crates.key(tcx, module.to_def_id()), child.ident);
                items.insert(
                    (path, format!("re-exported {}", tcx.def_descr(def))),
                    authority,
                );
            }
        }
    }
    let mut json = format!(
        "{{\n  \"format\": {},\n  \"crate\": {},\n  \"items\": [",
        string(FORMAT),
        string(tcx.crate_name(LOCAL_CRATE).as_str())
    );
    for (n, ((item, kind), authority)) in items.iter().enumerate() {
        let sep = if n == 0 { "" } else { "," };
        let _ = write!(
            json,
            "{sep}\n    {{\"item\": {}, \"kind\": {}, \"authority\": {{",
            string(item),
            string(kind)
        );
        for (m, (path, kinds)) in authority.iter().enumerate() {
            let kinds: Vec<_> = kinds.iter().map(|k| string(k)).collect();
            let sep = if m == 0 { "" } else { ", " };
            let _ = write!(json, "{sep}{}: [{}]", string(path), kinds.join(", "));
        }
        json.push_str("}}");
    }
    json.push_str("\n  ],\n  \"holes\": [");
    for (n, ((item, kind, what), count)) in holes.iter().enumerate() {
        let sep = if n == 0 { "" } else { "," };
        let _ = write!(
            json,
            "{sep}\n    {{\"item\": {}, \"kind\": {}, \"hole\": {}, \"count\": {count}}}",
            string(item),
            string(kind),
            string(what)
        );
    }
    json.push_str("\n  ]\n}\n");
    let name = format!("{}.interface.json", tcx.crate_name(LOCAL_CRATE));
    std::fs::write(out.join(name), json)
}

/// A JSON string literal.
fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `path.segment`, or `segment` at a type's root.
fn join(path: &str, segment: impl std::fmt::Display) -> String {
    if path.is_empty() {
        segment.to_string()
    } else {
        format!("{path}.{segment}")
    }
}

struct Search<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    crates: &'a Crates,
    pure: &'a Pure<'a, 'tcx>,
    /// The types the current item's search may still visit.
    budget: Cell<usize>,
}

impl<'a, 'tcx> Search<'a, 'tcx> {
    /// The declared authority of `def`, or none for an item that has none to declare (a trait,
    /// a module, a `use`).
    fn item(&self, def: DefId) -> Option<Authority> {
        let tcx = self.tcx;
        let kind = tcx.def_kind(def);
        if !matches!(
            kind,
            DefKind::Fn
                | DefKind::AssocFn
                | DefKind::Struct
                | DefKind::Enum
                | DefKind::Union
                | DefKind::TyAlias
                | DefKind::Const { .. }
                | DefKind::AssocConst { .. }
                | DefKind::Static { .. }
        ) {
            return None;
        }
        self.budget.set(BUDGET);
        let env = TypingEnv::non_body_analysis(tcx, def);
        let mut authority = Authority::new();
        if matches!(kind, DefKind::Fn | DefKind::AssocFn) {
            let sig = tcx.fn_sig(def).instantiate_identity().skip_normalization();
            let sig = tcx.instantiate_bound_regions_with_erased(sig);
            let names = tcx.fn_arg_idents(def);
            for (i, &input) in sig.inputs().iter().enumerate() {
                let name = match names.get(i) {
                    Some(Some(ident)) if ident.name != rustc_span::kw::Underscore => {
                        ident.to_string()
                    }
                    _ => format!("#{i}"),
                };
                self.ty(env, input, name, &mut authority);
            }
        } else {
            let ty = tcx.type_of(def).instantiate_identity().skip_normalization();
            self.ty(env, ty, String::new(), &mut authority);
        }
        Some(authority)
    }

    /// Adds what `ty`, held at `path`, reaches to `authority`.
    fn ty(&self, env: TypingEnv<'tcx>, ty: Ty<'tcx>, path: String, authority: &mut Authority) {
        let ty = self.tcx.erase_and_anonymize_regions(ty);
        self.walk(env, ty, path, authority, &mut FxHashMap::default(), 0);
    }

    /// `holding` holds the crate's types on the way to `ty`, by definition, each with whether it
    /// grew: one met again adds nothing new, and one whose type arguments grow out of a held
    /// instance's that itself grew, as polymorphic recursion makes them (`Poly<((T, T), (T, T))>`
    /// in `Poly<T>`), is `opaque`. One growth step expands (`Node<Arc<Net>>` in `Node<Net>`);
    /// a second is `opaque`, even where the recursion would end.
    fn walk(
        &self,
        env: TypingEnv<'tcx>,
        ty: Ty<'tcx>,
        path: String,
        authority: &mut Authority,
        holding: &mut FxHashMap<DefId, Vec<(Ty<'tcx>, bool)>>,
        depth: usize,
    ) {
        let tcx = self.tcx;
        let opaque = |t: Ty<'tcx>| with_forced_trimmed_paths!(format!("opaque({t})"));
        let mut left = self.budget.get();
        match self.pure.check(env, ty) {
            Ok(()) => return,
            // Its parts may each be shallow enough to decide; it is not.
            Err(e) if e.too_deep() => left = 0,
            Err(_) => {}
        }
        let mut add = |path: String, kind: String| {
            authority.entry(path).or_default().insert(kind);
        };
        if left == 0 {
            add(path, opaque(ty));
            return;
        }
        self.budget.set(left - 1);
        match *ty.kind() {
            ty::Ref(_, t, _) | ty::Slice(t) | ty::Array(t, _) | ty::Pat(t, _) => {
                self.walk(env, t, path, authority, holding, depth)
            }
            ty::Tuple(ts) => {
                for (i, t) in ts.iter().enumerate() {
                    self.walk(env, t, join(&path, i), authority, holding, depth);
                }
            }
            ty::Foreign(_) => add(path, "foreign".into()),
            ty::FnPtr(_, header) if header.abi() != ExternAbi::Rust => add(path, "foreign".into()),
            ty::Alias(..) => match tcx.try_normalize_erasing_regions(env, Unnormalized::new(ty)) {
                Ok(norm) if norm != ty => self.walk(env, norm, path, authority, holding, depth),
                _ => add(path, opaque(ty)),
            },
            ty::Adt(def, args) if self.crates.is_std(def.did()) => {
                match pure::carries(&self.crates.key(tcx, def.did())) {
                    // A handle reaches what its type arguments do too (`Arc<Net>`).
                    Some(Carries::Kinds(kinds)) => {
                        for &kind in kinds {
                            add(path.clone(), kind.into());
                        }
                        for t in args.types() {
                            self.walk(env, t, path.clone(), authority, holding, depth);
                        }
                    }
                    Some(Carries::Fields) => {
                        self.fields(env, def, args, &path, authority, holding, depth)
                    }
                    Some(Carries::Hidden) => add(path, opaque(ty)),
                    None => {
                        for t in args.types() {
                            self.walk(env, t, path.clone(), authority, holding, depth);
                        }
                    }
                }
            }
            // A trusted type's author vouches for its fields: it reaches what its type
            // arguments do.
            ty::Adt(def, args) if self.pure.is_trusted(def.did()) => {
                for t in args.types() {
                    self.walk(env, t, path.clone(), authority, holding, depth);
                }
            }
            ty::Adt(..) if depth >= MAX_DEPTH => add(path, opaque(ty)),
            ty::Adt(def, args) => {
                let held = holding.get(&def.did()).cloned().unwrap_or_default();
                if held.iter().any(|&(h, _)| h == ty) {
                    return;
                }
                // A held instance's type argument properly inside one of this instance's
                // (`T` in `Poly<(T, T)>`): the arguments grew from it.
                let grew_from: Vec<bool> = held
                    .iter()
                    .filter(|(h, _)| {
                        h.walk().skip(1).filter_map(|a| a.as_type()).any(|a| {
                            args.types()
                                .any(|b| b != a && b.walk().any(|x| x.as_type() == Some(a)))
                        })
                    })
                    .map(|&(_, grew)| grew)
                    .collect();
                if grew_from.contains(&true) {
                    add(path, opaque(ty));
                    return;
                }
                holding
                    .entry(def.did())
                    .or_default()
                    .push((ty, !grew_from.is_empty()));
                self.fields(env, def, args, &path, authority, holding, depth + 1);
                holding.entry(def.did()).or_default().pop();
            }
            // Type parameters, `dyn Trait`, raw pointers, and closures and coroutines, which no
            // signature names: `Pure` failed, and the type does not show why.
            _ => add(path, opaque(ty)),
        }
    }

    /// Adds what the fields of `def` at `args` reach, each under `path.field`, and under
    /// `path.Variant.field` for an enum.
    fn fields(
        &self,
        env: TypingEnv<'tcx>,
        def: ty::AdtDef<'tcx>,
        args: ty::GenericArgsRef<'tcx>,
        path: &str,
        authority: &mut Authority,
        holding: &mut FxHashMap<DefId, Vec<(Ty<'tcx>, bool)>>,
        depth: usize,
    ) {
        for variant in def.variants() {
            let base = if def.is_enum() {
                join(path, variant.name)
            } else {
                path.to_string()
            };
            for field in &variant.fields {
                let t = self.pure.field_ty(env, field, args);
                self.walk(env, t, join(&base, field.name), authority, holding, depth);
            }
        }
    }
}
