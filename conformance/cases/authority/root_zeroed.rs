//@ rules: ROOT-4, UNSAFE-1
//@ grants: unsafe
// Under the grant, `mem::zeroed`, `transmute`, and raw-pointer reads still produce only a
// `Pure` type, under any path that names them, so they cannot forge a capability:
// `assume_init` through a `Box` or an `Rc` too, `align_to`, which reinterprets bytes, and
// `Vec::set_len`, which makes raw memory into elements. A raw-pointer read is a deref, a read,
// or a conversion to a reference or an owner. A union's field reinterprets another
// field's bytes, as `transmute` does. Assigning to such a place, or borrowing it raw
// (`&raw const`), directly or through a field or an array index, reads nothing,
// unless reaching it derefs a reference, a `Box`, or a `Deref` impl on the way.
use core::ptr::NonNull;
use std::fs::Fs;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::rc::Rc;

pub fn forge() -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { std::mem::zeroed::<Fs>() } //~ FX0517
}

pub fn forge_core() -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { core::mem::zeroed::<Fs>() } //~ FX0517
}

pub fn forge_uninit() -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { MaybeUninit::<Fs>::zeroed().assume_init() } //~ FX0517
}

pub fn forge_box() -> Box<Fs> {
    // SAFETY: this rejected fixture is never executed.
    unsafe { Box::<Fs>::new_zeroed().assume_init() } //~ FX0517
}

pub fn forge_rc() -> Rc<[Fs]> {
    // SAFETY: this rejected fixture is never executed.
    unsafe { Rc::<[Fs]>::new_uninit_slice(1).assume_init() } //~ FX0517
}

pub fn forge_vec() -> Vec<Fs> {
    let mut v = Vec::with_capacity(1);
    // SAFETY: this rejected fixture is never executed.
    unsafe { v.set_len(1) } //~ FX0517
    v
}

pub fn transmuted() -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { std::mem::transmute::<[u8; std::mem::size_of::<Fs>()], Fs>([0; std::mem::size_of::<Fs>()]) } //~ FX0517
}

pub fn aligned(bytes: &[u8]) -> &[Fs] {
    // SAFETY: this rejected fixture is never executed.
    unsafe { bytes.align_to::<Fs>().1 } //~ FX0517
}

pub fn read(p: *const Fs) -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { core::ptr::read(p) } //~ FX0517
}

pub fn read_method(p: *const Fs) -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { p.read() } //~ FX0517
}

pub fn deref(p: *const Fs) -> &'static Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &*p } //~ FX0517
}

pub fn deref_field(p: *mut (u64, Fs)) -> u64 {
    // SAFETY: this rejected fixture is never executed.
    unsafe { (*p).0 } //~ FX0517
}

pub fn as_ref(p: *const Fs) -> Option<&'static Fs> {
    // SAFETY: this rejected fixture is never executed.
    unsafe { p.as_ref() } //~ FX0517
}

pub fn non_null(mut p: NonNull<Fs>) -> &'static mut Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { p.as_mut() } //~ FX0517
}

pub fn slice(p: *const Fs) -> &'static [Fs] {
    // SAFETY: this rejected fixture is never executed.
    unsafe { std::slice::from_raw_parts(p, 1) } //~ FX0517
}

pub fn boxed(p: *mut Fs) -> Box<Fs> {
    // SAFETY: this rejected fixture is never executed.
    unsafe { Box::from_raw(p) } //~ FX0517
}

pub fn indexed(p: *const [Fs; 1]) -> &'static Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &(*p)[0] } //~ FX0517
}

// Indexing or a field through an implicit deref loads the base first, so it reads.
pub fn indexed_through_reference(p: *const &[Fs; 1]) -> *const Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &raw const (*p)[0] } //~ FX0517
}

pub struct Holder {
    pub fs: Fs,
    pub bytes: [u8; 1],
}

impl std::ops::Deref for Holder {
    type Target = [u8; 1];
    fn deref(&self) -> &[u8; 1] {
        &self.bytes
    }
}

#[allow(dangerous_implicit_autorefs)]
pub fn indexed_through_deref(p: *const Holder) -> *const u8 {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &raw const (*p)[0] } //~ FX0517
}

pub fn field_through_reference(p: *const &(u64, Fs)) -> *const u64 {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &raw const (*p).0 } //~ FX0517
}

pub fn generic<T>(p: *const T) -> &'static T {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &*p } //~ FX0517
}

pub union Pun {
    bits: u64,
    fs: ManuallyDrop<Fs>,
}

pub fn punned(p: &Pun) -> &Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe { &p.fs } //~ FX0517
}

pub fn punned_pattern(p: Pun) -> Fs {
    // SAFETY: this rejected fixture is never executed.
    unsafe {
        let Pun { fs } = p; //~ FX0517
        ManuallyDrop::into_inner(fs)
    }
}

pub fn store(p: &mut Pun, slot: *mut Fs, fs: Fs, other: Fs) {
    p.fs = ManuallyDrop::new(fs);
    // SAFETY: the caller passes a valid pointer.
    unsafe { *slot = other }
}

pub fn field_address(p: *const (u64, Fs)) -> *const u64 {
    // SAFETY: the caller passes a valid pointer.
    unsafe { &raw const (*p).0 }
}

pub fn indexed_address(p: *const [Fs; 1]) -> *const Fs {
    // SAFETY: the caller passes a valid pointer.
    unsafe { &raw const (*p)[0] }
}

pub fn indexed_store(p: *mut [Fs; 1], fs: Fs) {
    // SAFETY: the caller passes a valid pointer.
    unsafe { (*p)[0] = fs }
}

pub fn generic_pure<T: Pure>(p: *const T) -> &'static T {
    // SAFETY: the caller passes a valid pointer.
    unsafe { &*p }
}

pub fn zero() -> u64 {
    // SAFETY: an all-zero `u64` is valid.
    unsafe { std::mem::zeroed::<u64>() }
}

pub fn bits() -> u64 {
    // SAFETY: `i64` and `u64` have the same size, and every bit pattern is valid.
    unsafe { std::mem::transmute::<i64, u64>(-1) }
}

pub fn words(bytes: &[u8]) -> &[u64] {
    // SAFETY: every bit pattern is a valid `u64`.
    unsafe { bytes.align_to::<u64>().1 }
}

// `vec!` builds its buffer uninitialized and moves the values in; it makes nothing.
pub fn both(fs: Fs, other: Fs) -> Vec<Fs> {
    vec![fs, other]
}
