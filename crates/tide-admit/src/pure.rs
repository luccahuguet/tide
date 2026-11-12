//! The structural `Pure` decision (`design/02-memory.md` §9, PURE-1; 03 §3.1) and the rules
//! that need it, each with its D18 code.
//!
//! Tide's std declares `Pure` with a blanket impl, so rustc proves every `Pure` bound; the
//! checker decides each `T: Pure` obligation here instead (`walk.rs` finds them). A type is
//! `Pure` when no authority is reachable from it:
//!
//! - std's types (`core`, `alloc`, Rust's std, Tide's std): those [`NOT_PURE`] names are not
//!   `Pure`; `Id<T>` is `Pure` for every `T`; any other is `Pure` exactly when its
//!   type arguments are. That is 02 §9's walk with
//!   its raw-pointer rule (std owns or borrows values only of its type arguments or of fixed
//!   `Pure` types), read off the arguments rather than off fields that change with the pinned
//!   nightly; an erased payload that is not `Pure` (`fmt::Arguments`), and a type that holds
//!   one with no type argument to show it (`fmt::DebugStruct`), is named;
//! - the crate's own types: every field, private ones included, with type arguments
//!   substituted; recursive types take the greatest fixed point, and one nested more than
//!   [`MAX_TYPE_DEPTH`] deep is rejected; a `#[trusted_pure]` type is `Pure` exactly when its
//!   type arguments are, as std's types are;
//! - raw pointers, `extern` types, and foreign function pointers are not `Pure`; Rust-ABI
//!   function pointers, fn items, and primitives are; references, arrays, tuples, and closures
//!   are `Pure` exactly when their parts (a closure's captures) are;
//! - `dyn Trait` is `Pure` only through a `Pure` supertrait, an opaque type only through its
//!   bounds, and a type parameter or a projection that does not normalize only through a bound
//!   in scope.
//!
//! Opaque types stay opaque (no `Pure` leaks from a hidden type, 02 §5.2), so the decision
//! runs in a typing environment that does not reveal them.

use rustc_abi::ExternAbi;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::elaborate::supertrait_def_ids;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingEnv, Unnormalized};
use rustc_trait_selection::infer::InferCtxtExt;

use crate::crates::Crates;
use crate::diag::Code;
use crate::policy::matches;

/// What a std type that is never `Pure` carries, which declared authority reports (03 §4.1).
#[derive(Clone, Copy)]
pub enum Carries {
    /// These authority kinds (03 §0).
    Kinds(&'static [&'static str]),
    /// What its fields carry: `Root` and the simulated roots.
    Fields,
    /// Authority it does not show, as an erased payload or a raw pointer hides it.
    Hidden,
}

const FS: Carries = Carries::Kinds(&["fs.read", "fs.write"]);
const FS_READ: Carries = Carries::Kinds(&["fs.read"]);
const NET: Carries = Carries::Kinds(&["net"]);
const PROC: Carries = Carries::Kinds(&["proc"]);
const SYNC: Carries = Carries::Kinds(&["sync"]);
const HIDDEN: Carries = Carries::Hidden;

/// std's handles with no payload ownership, `Pure` for every type argument (02 §10.9).
/// Versioned with Tide's std, like `NOT_PURE`.
const ALWAYS_PURE: &[&str] = &["tide_std::arena::Id"];

/// std's types that are not `Pure` whatever their type arguments (02 §9's table), by
/// `Crates::key` patterns (`policy.rs`'s glob), with what each carries. It is versioned with
/// Tide's std, as the allowlist is with the rustc pin.
const NOT_PURE: &[(&str, Carries)] = &[
    // Capabilities and their simulated roots (03 §9.2, §9.5).
    ("tide_std::Root", Carries::Fields),
    ("tide_std::fs::Fs", FS),
    ("tide_std::net::Net", NET),
    ("tide_std::time::Clock", Carries::Kinds(&["clock"])),
    ("tide_std::rand::Rand", Carries::Kinds(&["rand"])),
    ("tide_std::env::Env", Carries::Kinds(&["env"])),
    ("tide_std::io::Stdin", Carries::Kinds(&["stdin"])),
    ("tide_std::io::StdinReader", Carries::Kinds(&["stdin"])),
    ("tide_std::process::Proc", PROC),
    ("tide_std::task::Tasks", Carries::Kinds(&["tasks"])),
    ("tide_std::accel::Accel", Carries::Kinds(&["accel"])),
    ("tide_std::sim::SimRoot", Carries::Fields),
    ("tide_std::sim::NodeRoot", Carries::Fields),
    // Resource handles, the simulator's handles, and what can hold one.
    ("tide_std::fs::File", FS),
    ("tide_std::fs::ReadDir", FS_READ),
    ("tide_std::fs::DirEntry", FS_READ),
    ("tide_std::net::TcpStream", NET),
    ("tide_std::net::TcpListener", NET),
    ("tide_std::net::UdpSocket", NET),
    ("tide_std::process::Command", PROC),
    ("tide_std::process::Child", PROC),
    ("tide_std::process::ChildStdin", PROC),
    ("tide_std::process::ChildStdout", PROC),
    ("tide_std::process::ChildStderr", PROC),
    ("tide_std::process::Stdio", PROC),
    ("tide_std::process::Signals", PROC),
    ("tide_std::log::LogConfig", HIDDEN),
    ("tide_std::sim::World", HIDDEN),
    ("tide_std::sim::Faults", HIDDEN),
    ("tide_std::sim::SimNode", HIDDEN),
    ("tide_std::sim::NodeTask", HIDDEN),
    // Synchronization handles (03 §1.2, GLOBAL-6), Rust's included.
    ("tide_std::sync::Shared", SYNC),
    ("tide_std::sync::Arc", SYNC),
    ("tide_std::sync::Weak", SYNC),
    ("tide_std::sync::atomic::*", SYNC),
    ("tide_std::sync::raw::*", SYNC),
    ("tide_std::sync::mpsc::Sender", SYNC),
    ("tide_std::sync::mpsc::SyncSender", SYNC),
    ("tide_std::sync::mpsc::Receiver", SYNC),
    ("tide_std::sync::mpsc::Iter", SYNC),
    ("tide_std::sync::mpsc::IntoIter", SYNC),
    ("tide_std::sync::mpsc::TryIter", SYNC),
    ("tide_std::task::Scope", SYNC),
    ("tide_std::task::Task", SYNC),
    ("alloc::sync::*", SYNC),
    ("core::sync::atomic::Atomic*", SYNC),
    ("std::sync::*", SYNC),
    // Foreign handles.
    ("core::ptr::non_null::NonNull", HIDDEN),
    // Erased payloads that follow the `dyn` rule (02 §9), and the std types that hold one with
    // no type argument to show it: `debug_struct` and the other builders hold the `Formatter`,
    // `fmt::rt::Argument` a `NonNull` to an erased value, and `PanicMessage` an `Arguments`.
    ("core::fmt::Arguments", HIDDEN),
    ("core::fmt::Formatter", HIDDEN),
    ("core::fmt::builders::Debug*", HIDDEN),
    ("core::fmt::rt::*", HIDDEN),
    ("core::panic::panic_info::*", HIDDEN),
    ("std::panic::PanicHookInfo", HIDDEN),
];

/// What the std type keyed `key` carries, if it is never `Pure`.
pub fn carries(key: &str) -> Option<Carries> {
    NOT_PURE
        .iter()
        .find(|(pattern, _)| matches(pattern, key))
        .map(|&(_, carries)| carries)
}

/// The std functions that make a `T` from raw bytes or a raw pointer, by `Crates::key` pattern,
/// with the index of that `T` among a use's type arguments. ROOT-4 bounds it by `Pure` under the
/// `unsafe` grant too, wherever a use names the function; `walk.rs` bounds the places that read
/// raw memory (a raw pointer's deref, a union's field) the same way.
pub const FORGES: &[(&str, usize)] = &[
    // Values from raw bytes: zeroed or uninitialized memory, by any owner, and reinterpreted
    // bytes (`align_to::<U>` makes `U`s of a slice's bytes).
    ("core::mem::zeroed", 0),
    ("core::mem::uninitialized", 0),
    ("core::mem::maybe_uninit::MaybeUninit::<T>::assume_init*", 0),
    ("alloc::boxed::Box::<*>::assume_init", 0),
    ("alloc::rc::*::assume_init", 0),
    ("alloc::vec::Vec::<T*>::set_len", 0),
    ("core::intrinsics::transmute", 1),
    ("core::mem::transmute_copy", 1),
    ("core::slice::<impl [T]>::align_to*", 1),
    // Reads through a raw pointer, and its conversions to a reference or an owner.
    ("core::ptr::read*", 0),
    ("core::ptr::replace", 0),
    ("core::ptr::const_ptr::<impl *const T>::read*", 0),
    ("core::ptr::const_ptr::<impl *const T>::as_*", 0),
    ("core::ptr::mut_ptr::<impl *mut T>::read*", 0),
    ("core::ptr::mut_ptr::<impl *mut T>::replace", 0),
    ("core::ptr::mut_ptr::<impl *mut T>::as_*", 0),
    ("core::ptr::non_null::NonNull::<T>::read*", 0),
    ("core::ptr::non_null::NonNull::<T>::replace", 0),
    ("core::ptr::non_null::NonNull::<T>::as_ref", 0),
    ("core::ptr::non_null::NonNull::<T>::as_mut", 0),
    ("core::ptr::non_null::NonNull::<T>::as_uninit_*", 0),
    ("core::slice::raw::from_raw_parts*", 0),
    ("core::slice::raw::from_*ptr_range", 0),
    ("alloc::boxed::Box::<T*>::from_raw*", 0),
    ("alloc::boxed::Box::<T*>::from_non_null*", 0),
    ("alloc::rc::*::from_raw*", 0),
    ("alloc::vec::Vec::<T*>::from_raw_parts*", 0),
    ("alloc::vec::Vec::<T*>::from_parts*", 0),
];

/// How many types the `Pure` walk and the `&mut` search expand one inside another before they
/// give up: a type whose recursion changes its type arguments (`Cons(T, Box<Poly<Vec<T>>>)`)
/// never repeats, so no fixed point stops it. Giving up rejects the type (02 §9).
const MAX_TYPE_DEPTH: usize = 64;

/// The rule a `Pure` obligation a use incurs serves, which picks the code a failure is reported
/// under (D18; 03 §3.1). `walk.rs` reports `static`s (`FX0502`), `const fn`s (`FX0518`), and
/// `#[pure]` functions (`FX0511`) itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Need {
    /// A user-written bound, a `Pure` supertrait's impl, a coercion to a `Pure` trait object,
    /// an opaque type's bounds, or a std bound no rule below owns.
    Bound,
    /// ROOT-4: a value made from raw bytes or read through a raw pointer ([`FORGES`], a deref,
    /// a union's field).
    Forged,
    /// LOCK-1: a `Shared` payload.
    Payload,
    /// LOCK-4: a `Shared::with` closure.
    LockClosure,
    /// PAR-5: a `join` closure.
    JoinClosure,
    /// PAR-6: a closure the slice methods share between branches.
    SharedClosure,
    /// PAR-4: the slice methods' elements.
    Elements,
}

impl Need {
    pub fn code(self) -> Code {
        match self {
            Need::Bound | Need::Forged => Code::Fx0517,
            Need::Payload => Code::Fx0606,
            Need::LockClosure => Code::Fx0603,
            Need::JoinClosure | Need::SharedClosure => Code::Fx0610,
            Need::Elements => Code::Fx0612,
        }
    }

    /// The diagnostic for a failure `why` under this rule.
    pub fn message(self, why: String) -> String {
        let rule = match self {
            Need::Bound => return why,
            Need::Forged => "what raw bytes or a raw pointer make must be `Pure`",
            Need::Payload => "a `Shared` payload must be `Send + Pure`",
            Need::LockClosure => "a `Shared::with` closure must be `Pure`",
            Need::JoinClosure => "a `join` closure must be `FnOnce + Pure + Send`",
            Need::SharedClosure => {
                "a closure the slice methods share must be `Fn + Pure + Send + Sync`"
            }
            Need::Elements => "the slice methods' elements must be `Pure`",
        };
        format!("{rule}: {why}")
    }
}

/// The std items whose `Pure` bounds a rule with its own code owns: (key pattern, the rule for
/// a parameter with an `Fn` bound, the rule for any other parameter). Any other bound is
/// [`Need::Bound`].
const NEEDS: &[(&str, Need, Need)] = &[
    ("tide_std::sync::Shared*", Need::LockClosure, Need::Payload),
    ("tide_std::par::join", Need::JoinClosure, Need::JoinClosure),
    (
        "tide_std::par::ParSlice::par_*",
        Need::SharedClosure,
        Need::Elements,
    ),
];

/// The rule that owns the `Pure` bounds of the item keyed `key`, for a closure parameter and
/// for any other.
pub fn needs(key: &str) -> (Need, Need) {
    NEEDS
        .iter()
        .find(|(pattern, ..)| matches(pattern, key))
        .map_or((Need::Bound, Need::Bound), |&(_, closure, other)| {
            (closure, other)
        })
}

/// Why a type is not `Pure`: the part that is not, and the reason.
pub struct NotPure<'tcx> {
    part: Ty<'tcx>,
    why: Why,
}

#[derive(Clone, Copy)]
enum Why {
    Named,
    RawPointer,
    Foreign,
    ForeignFn,
    Coroutine,
    Dyn,
    Opaque,
    Unbounded,
    TooDeep,
}

impl<'tcx> NotPure<'tcx> {
    /// Whether the type nests too deep to decide, rather than reaching what is not `Pure`.
    pub fn too_deep(&self) -> bool {
        matches!(self.why, Why::TooDeep)
    }

    /// "`ty` is not `Pure`: ..." for a failure found in `ty`.
    pub fn message(&self, ty: Ty<'tcx>) -> String {
        let show = |t: Ty<'tcx>| with_forced_trimmed_paths!(t.to_string());
        let why = match self.why {
            Why::Named => "is never `Pure`",
            Why::RawPointer => "is a raw pointer",
            Why::Foreign => "is an `extern` type",
            Why::ForeignFn => "is a foreign function",
            Why::Coroutine => "is a coroutine",
            Why::Dyn => "has no `Pure` supertrait",
            Why::Opaque => "does not state `Pure` in its bounds",
            Why::Unbounded => "is not bounded by `Pure`",
            // The part is not shown: it can be exponentially large (`Poly<(T, T)>`).
            Why::TooDeep => {
                return format!(
                    "`{}` is not `Pure`: it nests types more than {MAX_TYPE_DEPTH} deep, too \
                     deep to decide",
                    show(ty)
                );
            }
        };
        if self.part == ty && matches!(self.why, Why::Named) {
            format!("`{}` is never `Pure`", show(ty))
        } else if self.part == ty {
            format!("`{}` is not `Pure`: it {why}", show(ty))
        } else {
            format!(
                "`{}` is not `Pure`: it reaches `{}`, which {why}",
                show(ty),
                show(self.part)
            )
        }
    }
}

/// The `Pure` decision for one crate.
pub struct Pure<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    crates: &'a Crates,
    /// std's `Pure` trait.
    pub trait_def: DefId,
    /// The crate's types marked `#[trusted_pure]`.
    trusted: FxHashSet<DefId>,
}

impl<'a, 'tcx> Pure<'a, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        crates: &'a Crates,
        trait_def: DefId,
        trusted: FxHashSet<DefId>,
    ) -> Self {
        Pure {
            tcx,
            crates,
            trait_def,
            trusted,
        }
    }

    /// Whether `trait_def` is `Pure` or has it as a supertrait.
    fn implied_by(&self, trait_def: DefId) -> bool {
        supertrait_def_ids(self.tcx, trait_def).any(|d| d == self.trait_def)
    }

    /// Decides `ty: Pure` in `env`, which must not reveal opaque types.
    pub fn check(&self, env: TypingEnv<'tcx>, ty: Ty<'tcx>) -> Result<(), NotPure<'tcx>> {
        let ty = self.tcx.erase_and_anonymize_regions(ty);
        self.walk(env, ty, &mut FxHashSet::default(), 0)
    }

    /// `assumed` holds the types being decided or decided `Pure`: a type met again is `Pure`
    /// (the greatest fixed point), and any failure fails the whole decision. `depth` counts the
    /// crate's types expanded into their fields on the way to `ty`.
    fn walk(
        &self,
        env: TypingEnv<'tcx>,
        ty: Ty<'tcx>,
        assumed: &mut FxHashSet<Ty<'tcx>>,
        depth: usize,
    ) -> Result<(), NotPure<'tcx>> {
        let tcx = self.tcx;
        let not = |why| Err(NotPure { part: ty, why });
        if !assumed.insert(ty) {
            return Ok(());
        }
        let mut walk = |t| self.walk(env, t, assumed, depth);
        match *ty.kind() {
            ty::Ref(_, t, _) | ty::Slice(t) | ty::Array(t, _) | ty::Pat(t, _) => walk(t),
            ty::Tuple(ts) => ts.iter().try_for_each(walk),
            ty::RawPtr(..) => not(Why::RawPointer),
            ty::Foreign(_) => not(Why::Foreign),
            ty::FnPtr(_, header) if header.abi() != ExternAbi::Rust => not(Why::ForeignFn),
            ty::FnDef(def, _) if tcx.is_foreign_item(def) => not(Why::ForeignFn),
            ty::Closure(_, args) => args.as_closure().upvar_tys().iter().try_for_each(walk),
            ty::Coroutine(..) | ty::CoroutineClosure(..) | ty::CoroutineWitness(..) => {
                not(Why::Coroutine)
            }
            ty::Dynamic(preds, ..) => match preds.principal_def_id() {
                Some(p) if self.implied_by(p) => Ok(()),
                _ => not(Why::Dyn),
            },
            ty::Param(_) if self.bounded(env, ty) => Ok(()),
            ty::Param(_) => not(Why::Unbounded),
            ty::Alias(_, alias) => {
                match tcx.try_normalize_erasing_regions(env, Unnormalized::new(ty)) {
                    Ok(norm) if norm != ty => return walk(norm),
                    _ => {}
                }
                if self.bounded(env, ty) {
                    return Ok(());
                }
                let (def, opaque): (DefId, bool) = match alias.kind {
                    ty::AliasTyKind::Opaque { def_id } => (def_id.into(), true),
                    ty::AliasTyKind::Projection { def_id } => (def_id.into(), false),
                    _ => return not(Why::Unbounded),
                };
                // The opaque type's or associated type's own bounds.
                let stated = tcx
                    .item_bounds(def)
                    .iter_instantiated(tcx, alias.args)
                    .filter_map(|c| c.skip_normalization().as_trait_clause())
                    .any(|tp| {
                        self.implied_by(tp.def_id())
                            && tcx.erase_and_anonymize_regions(tp.self_ty().skip_binder()) == ty
                    });
                match (stated, opaque) {
                    (true, _) => Ok(()),
                    (false, true) => not(Why::Opaque),
                    (false, false) => not(Why::Unbounded),
                }
            }
            // A trusted type's author vouches for its fields, not for the type arguments its users
            // pass: it is `Pure` exactly when they are, as std's types are.
            ty::Adt(def, args) if self.trusted.contains(&def.did()) => {
                args.types().try_for_each(walk)
            }
            ty::Adt(def, args) if self.crates.is_std(def.did()) => {
                let key = self.crates.key(tcx, def.did());
                if ALWAYS_PURE.contains(&key.as_str()) {
                    return Ok(());
                }
                if carries(&key).is_some() {
                    return not(Why::Named);
                }
                args.types().try_for_each(walk)
            }
            ty::Adt(..) if depth >= MAX_TYPE_DEPTH => not(Why::TooDeep),
            ty::Adt(def, args) => def.all_fields().try_for_each(|field| {
                self.walk(env, self.field_ty(env, field, args), assumed, depth + 1)
            }),
            // Primitives, Rust-ABI function pointers and fn items, `!`, and the types of code
            // rustc rejects.
            _ => Ok(()),
        }
    }

    /// Whether `def` is a crate's type marked `#[trusted_pure]`.
    pub fn is_trusted(&self, def: DefId) -> bool {
        self.trusted.contains(&def)
    }

    /// The type of `field` of an ADT at `args`, normalized where it can be.
    pub fn field_ty(
        &self,
        env: TypingEnv<'tcx>,
        field: &ty::FieldDef,
        args: ty::GenericArgsRef<'tcx>,
    ) -> Ty<'tcx> {
        let tcx = self.tcx;
        let t = field.ty(tcx, args).skip_normalization();
        let t = tcx
            .try_normalize_erasing_regions(env, Unnormalized::new(t))
            .unwrap_or(t);
        tcx.erase_and_anonymize_regions(t)
    }

    /// AFREE-2: the `&mut` that `ty` names or holds, through type arguments and the fields of
    /// every type, std's included (`slice::IterMut` holds a `PhantomData<&mut T>`), or `ty`
    /// itself if it nests too deep to search.
    pub fn mut_ref(&self, env: TypingEnv<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
        let ty = self.tcx.erase_and_anonymize_regions(ty);
        let mut seen = FxHashSet::default();
        self.find_mut(env, ty, &mut seen, 0).err()
    }

    fn find_mut(
        &self,
        env: TypingEnv<'tcx>,
        ty: Ty<'tcx>,
        seen: &mut FxHashSet<Ty<'tcx>>,
        depth: usize,
    ) -> Result<(), Ty<'tcx>> {
        for t in ty.walk().filter_map(|a| a.as_type()) {
            match *t.kind() {
                ty::Ref(_, _, hir::Mutability::Mut) => return Err(t),
                ty::Adt(def, args) if seen.insert(t) => {
                    if depth >= MAX_TYPE_DEPTH {
                        return Err(ty);
                    }
                    for field in def.all_fields() {
                        self.find_mut(env, self.field_ty(env, field, args), seen, depth + 1)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Whether a where-clause in scope bounds `ty` by `Pure` or a trait with `Pure` as a
    /// supertrait.
    fn bounded(&self, env: TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
        env.param_env.caller_bounds().iter().any(|c| {
            c.as_trait_clause().is_some_and(|tp| {
                self.implied_by(tp.def_id())
                    && self
                        .tcx
                        .erase_and_anonymize_regions(tp.self_ty().skip_binder())
                        == ty
            })
        })
    }
}

/// Whether `tr` holds in `env`, as rustc decides it.
pub fn implements<'tcx>(tcx: TyCtxt<'tcx>, env: TypingEnv<'tcx>, tr: ty::TraitRef<'tcx>) -> bool {
    let (infcx, param_env) = tcx.infer_ctxt().build_with_typing_env(env);
    let tr = tcx.erase_and_anonymize_regions(tr);
    infcx
        .type_implements_trait(tr.def_id, tr.args, param_env)
        .must_apply_modulo_regions()
}
