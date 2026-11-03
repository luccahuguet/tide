//! The backstop (ADMIT-1, PURE-1): the instances that can run, checked whatever route reached
//! them. It runs after rustc's analysis, which runs only when rustc and the walk found no
//! errors. rustc collects the crate's mono items from its roots, the items a build exports, its
//! statics, and its entry point, through the uses it records, and makes its own instances of
//! upstream generics (`-Zshare-generics=no`). A library's `#[inline]` functions, like its
//! generics, are checked in the crates that instantiate them.
//!
//! A method of a non-local trait impl whose key a rule denies, or gives the `unsafe` grant the
//! package lacks, is rejected (`FX0520`); a key no rule lists does not count, since std's
//! internals instantiate unlisted impls. Where an item outside `core`, `alloc`, and Rust's std is
//! used, its where-clauses at its instance's args, and its trait method's for an impl's method,
//! must meet their `Pure` part, as at a use in the walk (`FX0517` and the codes
//! `clause_obligations` gives). Each failure is reported at its uses in the nearest local mono
//! items; a denied impl that no local item reaches this way, at the crate root.

use rustc_data_structures::fx::{FxHashMap, FxHashSet, FxIndexSet};
use rustc_hir::def_id::{CRATE_DEF_ID, DefId, LocalDefId};
use rustc_middle::mir::interpret::{AllocId, GlobalAlloc};
use rustc_middle::mono::{CollectionMode, MonoItem};
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{GenericArgsRef, InstanceKind, TyCtxt, TypingEnv};
use rustc_span::{DUMMY_SP, Span};

use super::{Admitted, Grants, Holes, Role, Walk, impl_key, policy};
use crate::crates::Crates;

type Users<'tcx> = FxHashMap<MonoItem<'tcx>, Vec<(MonoItem<'tcx>, Span)>>;

/// Checks the crate's mono items, emits its diagnostics, and adds the trust holes it finds to the
/// walk's `holes`: the impls the `unsafe` grant admits, held by the local items that use them.
pub fn check_mono_items(tcx: TyCtxt<'_>, grants: Grants, holes: &mut Holes) {
    let crates = Crates::new(tcx);
    let mut walk = Walk::new(tcx, &crates, grants);
    walk.item_env = Some(TypingEnv::fully_monomorphized());
    let items: FxIndexSet<MonoItem<'_>> = tcx
        .collect_and_partition_mono_items(())
        .codegen_units
        .iter()
        .flat_map(|cgu| cgu.items().keys().copied())
        .collect();
    let mut generic = Vec::new();
    let mut users = Users::default();
    for &item in &items {
        for (used, span) in uses(tcx, item) {
            users.entry(used).or_default().push((item, span));
        }
    }
    for &item in &items {
        let MonoItem::Fn(instance) = item else {
            continue;
        };
        let def_id = instance.def_id();
        let role = crates.role(def_id.krate);
        if role != Role::Local
            && let Some(impl_def_id) = tcx.trait_impl_of_assoc(def_id)
        {
            let key = impl_key(tcx, impl_def_id);
            let verdict = walk.decide(key.clone(), policy::lookup(&key), true);
            if !matches!(verdict, Ok(Admitted::Free)) {
                let mut sites = local_uses(tcx, &users, item);
                if sites.is_empty() {
                    sites.push((CRATE_DEF_ID, tcx.def_span(CRATE_DEF_ID)));
                }
                let note = with_forced_trimmed_paths!(format!(" (it runs as `{instance}`)"));
                for (owner, span) in sites {
                    match &verdict {
                        Err(rejection) => walk.reject(span, rejection.clone(), &note),
                        // The walk may hold a generic user's impl at the item whose types
                        // chose it, so a generic user's holes are decided after the others.
                        Ok(admitted) if is_generic(tcx, owner) => {
                            let held = std::mem::take(&mut walk.holes);
                            walk.owner = Some(owner);
                            walk.admitted(span, admitted.clone());
                            generic.push((owner, std::mem::replace(&mut walk.holes, held)));
                        }
                        Ok(admitted) => {
                            walk.owner = Some(owner);
                            walk.admitted(span, admitted.clone());
                        }
                    }
                }
            }
        }
        if let InstanceKind::Item(_) = instance.def
            && matches!(role, Role::Local | Role::TideStd | Role::Other)
            && walk.pure.is_some()
        {
            // Decided once, and only a failure is looked for at the uses: in the fully
            // monomorphized environment, the verdict does not depend on the user.
            let found = std::mem::take(&mut walk.diags);
            walk.owner = None;
            walk.seen_pure.clear();
            item_clauses(&mut walk, def_id, instance.args, DUMMY_SP);
            let failed = !std::mem::replace(&mut walk.diags, found).is_empty();
            if failed {
                for (owner, span) in local_uses(tcx, &users, item) {
                    walk.owner = Some(owner);
                    item_clauses(&mut walk, def_id, instance.args, span);
                }
            }
        }
    }
    // The walk and the backstop can find the same use; each counts it once.
    for (hole, n) in walk.take_holes() {
        let count = holes.entry(hole).or_default();
        *count = (*count).max(n);
    }
    // A generic user holds an impl unless an item that instantiates it, which may have chosen
    // the impl, holds it already; one reached through std's internals is the generic user's.
    let mut kept = Holes::new();
    let mut by_root: FxHashMap<LocalDefId, Vec<MonoItem<'_>>> = FxHashMap::default();
    if !generic.is_empty() {
        for &item in &items {
            if let Some(def) = local_def(item) {
                by_root.entry(root(tcx, def)).or_default().push(item);
            }
        }
    }
    let mut paths: FxHashMap<LocalDefId, FxHashSet<String>> = FxHashMap::default();
    for (owner, found) in generic {
        let path = paths.entry(root(tcx, owner)).or_insert_with(|| {
            instantiators(tcx, &users, &by_root, owner)
                .into_iter()
                .map(|def| crates.key(tcx, def.to_def_id()))
                .collect()
        });
        walk.holes = found;
        for (hole, n) in walk.take_holes() {
            if !holes
                .keys()
                .any(|(key, _, what)| *what == hole.2 && path.contains(key))
            {
                let count = kept.entry(hole).or_default();
                *count = (*count).max(n);
            }
        }
    }
    for (hole, n) in kept {
        let count = holes.entry(hole).or_default();
        *count = (*count).max(n);
    }
    walk.diags.emit(tcx);
}

/// Whether `def`'s instances depend on type arguments, a closure's on its function's. The crate
/// root, which holds what no local item reaches, has none.
fn is_generic(tcx: TyCtxt<'_>, def: LocalDefId) -> bool {
    def != CRATE_DEF_ID
        && tcx
            .generics_of(root(tcx, def))
            .requires_monomorphization(tcx)
}

/// The item whose body `def` is, or whose closure it is: a function, `const`, or `static`.
fn root(tcx: TyCtxt<'_>, def: LocalDefId) -> LocalDefId {
    tcx.typeck_root_def_id(def.to_def_id()).expect_local()
}

/// The local item that `item` is or belongs to, if any.
fn local_def(item: MonoItem<'_>) -> Option<LocalDefId> {
    match item {
        MonoItem::Fn(instance) => instance.def_id().as_local(),
        MonoItem::Static(def_id) => def_id.as_local(),
        MonoItem::GlobalAsm(id) => Some(id.owner_id.def_id),
    }
}

/// The item holding `owner` and the local items that run its instances, through generic ones
/// up to the first non-generic one on each path: those whose types can choose what a generic
/// `owner` uses. Each is a `root`, not a closure.
fn instantiators<'tcx>(
    tcx: TyCtxt<'tcx>,
    users: &Users<'tcx>,
    by_root: &FxHashMap<LocalDefId, Vec<MonoItem<'tcx>>>,
    owner: LocalDefId,
) -> FxHashSet<LocalDefId> {
    let mut found = FxHashSet::default();
    let mut queue = vec![root(tcx, owner)];
    while let Some(next) = queue.pop() {
        if !found.insert(next) || !is_generic(tcx, next) {
            continue;
        }
        for &item in by_root.get(&next).into_iter().flatten() {
            queue.extend(
                local_uses(tcx, users, item)
                    .into_iter()
                    .map(|(def, _)| root(tcx, def)),
            );
        }
    }
    found
}

/// Decides the `Pure` part of what a use of `def_id` at `args` needs: its where-clauses, and its
/// trait method's for a trait impl's method, whose own clauses rustc does not copy to the impl.
fn item_clauses<'tcx>(
    walk: &mut Walk<'_, 'tcx>,
    def_id: DefId,
    args: GenericArgsRef<'tcx>,
    span: Span,
) {
    let tcx = walk.tcx;
    walk.use_generic(def_id, args, span, false);
    if let Some(impl_def_id) = tcx.trait_impl_of_assoc(def_id)
        && let Some(trait_item) = tcx.associated_item(def_id).trait_item_def_id()
    {
        let trait_ref = tcx
            .impl_trait_ref(impl_def_id)
            .instantiate(tcx, args)
            .skip_normalization();
        let trait_args = args.rebase_onto(tcx, impl_def_id, trait_ref.args);
        walk.clause_obligations(trait_item, trait_args, span, false);
    }
}

/// The items `item` uses, with the span of each use: a function's from rustc's collector, a
/// static's from the pointers its value holds, which have no span.
fn uses<'tcx>(tcx: TyCtxt<'tcx>, item: MonoItem<'tcx>) -> Vec<(MonoItem<'tcx>, Span)> {
    match item {
        // The collector computed these; the query returns its cached result.
        MonoItem::Fn(instance) => tcx
            .items_of_instance((instance, CollectionMode::UsedItems))
            .map_or_else(
                |_| Vec::new(),
                |(used, _)| used.iter().map(|u| (u.node, u.span)).collect(),
            ),
        MonoItem::Static(def_id) => {
            let mut found = Vec::new();
            if let Ok(alloc) = tcx.eval_static_initializer(def_id) {
                for prov in alloc.inner().provenance().ptrs().values() {
                    pointees(tcx, prov.alloc_id(), &mut found);
                }
            }
            found.into_iter().map(|item| (item, DUMMY_SP)).collect()
        }
        MonoItem::GlobalAsm(_) => Vec::new(),
    }
}

/// The functions and statics an allocation points to, through vtables and nested memory, as
/// rustc's collector finds them.
fn pointees<'tcx>(tcx: TyCtxt<'tcx>, id: AllocId, found: &mut Vec<MonoItem<'tcx>>) {
    match tcx.global_alloc(id) {
        GlobalAlloc::Static(def_id) => found.push(MonoItem::Static(def_id)),
        GlobalAlloc::Function { instance, .. } => found.push(MonoItem::Fn(instance)),
        GlobalAlloc::Memory(alloc) => {
            for prov in alloc.inner().provenance().ptrs().values() {
                pointees(tcx, prov.alloc_id(), found);
            }
        }
        GlobalAlloc::VTable(ty, dyn_ty) => {
            let principal = dyn_ty
                .principal()
                .map(|p| tcx.instantiate_bound_regions_with_erased(p));
            pointees(tcx, tcx.vtable_allocation((ty, principal)), found);
        }
        GlobalAlloc::TypeId { .. } => {}
    }
}

/// The uses of `item` in the nearest local mono items: each local item that uses it directly or
/// through non-local items only, at the use when it is in the local crate's source and at the
/// local item otherwise. rustc's entry point calls `main` through the `start` lang item's
/// instance, which stands for `main`.
fn local_uses<'tcx>(
    tcx: TyCtxt<'tcx>,
    users: &Users<'tcx>,
    item: MonoItem<'tcx>,
) -> Vec<(LocalDefId, Span)> {
    let main = tcx.entry_fn(()).and_then(|(main, _)| main.as_local());
    let start = tcx.lang_items().start_fn();
    let mut found = Vec::new();
    let mut seen = FxHashSet::default();
    let mut queue = vec![item];
    while let Some(next) = queue.pop() {
        if let (MonoItem::Fn(instance), Some(main)) = (next, main)
            && Some(instance.def_id()) == start
        {
            found.push((main, tcx.def_span(main)));
        }
        for &(user, span) in users.get(&next).into_iter().flatten() {
            match local_def(user) {
                Some(def) => {
                    let span = span.source_callsite();
                    let span = if span.is_dummy() || tcx.sess.source_map().is_imported(span) {
                        tcx.def_span(def)
                    } else {
                        span
                    };
                    found.push((def, span));
                }
                None if seen.insert(user) => queue.push(user),
                None => {}
            }
        }
    }
    found
}
