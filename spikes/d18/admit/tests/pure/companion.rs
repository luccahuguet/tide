// The companion-supertrait spelling of `dyn Value + Pure`, and the points tide-admit checks.
#![forbid(unsafe_code)]
use minifacade::{Fs, Pure, Shared};
use std::cell::RefCell;
use std::rc::Rc;

pub trait Value {
    fn get(&self) -> u32;
}
pub trait PureValue: Value + Pure {}
impl<T: Value + ?Sized> PureValue for T {}

struct Plain {
    n: u32,
    tag: String,
    cache: Rc<RefCell<Vec<u32>>>, // Rc and RefCell are Pure (6.6)
}
impl Value for Plain {
    fn get(&self) -> u32 {
        self.n + self.tag.len() as u32 + self.cache.borrow().len() as u32
    }
}

struct Holder {
    fs: Fs,
}
impl Value for Holder {
    fn get(&self) -> u32 {
        let _ = &self.fs;
        1
    }
}

struct Deep {
    inner: (u8, Vec<Box<Shared<u8>>>),
}
impl Value for Deep {
    fn get(&self) -> u32 {
        self.inner.0 as u32
    }
}

fn sum_pure(xs: &[&dyn PureValue]) -> u32 {
    // upcast to the base trait object still works
    xs.iter().map(|x| { let v: &dyn Value = *x; v.get() }).sum()
}

fn run_pure<T: PureValue>(t: &T) -> u32 {
    t.get()
}

fn job<F: Fn() -> u32 + Pure>(f: F) -> u32 {
    f()
}

trait Job: Pure {
    fn go(&self);
}
struct Bad {
    hits: core::sync::atomic::AtomicU64, // FX0520
}
impl Job for Bad { // FX0510 (impl case)
    fn go(&self) {}
}

fn main() {
    let p = Plain { n: 1, tag: "x".to_string(), cache: Rc::new(RefCell::new(vec![])) };
    let h = Holder { fs: Fs::new_for_test() };
    let d = Deep { inner: (2, vec![Box::new(Shared { value: 3 })]) };
    let ok: &dyn PureValue = &p; // admitted
    let bad: &dyn PureValue = &h; // FX0510 at the unsize coercion
    let bad2: Box<dyn PureValue> = Box::new(d); // FX0510 at the unsize coercion
    let _ = sum_pure(&[ok, bad, &*bad2]);
    let _ = run_pure(&p); // admitted
    let _ = run_pure(&h); // FX0510 at substitution for T: PureValue
    let n = 5u32;
    let _ = job(move || n + 1); // admitted closure
    let _ = job(|| { let _ = &h.fs; 0 }); // FX0510: closure captures &Holder
    let v: &dyn Value = &h; // plain dyn Value: no Pure requirement, admitted
    let _ = v.get();
    Bad { hits: core::sync::atomic::AtomicU64::new(0) }.go(); // FX0510 FX0520
}

// Generic code: an unbounded parameter is not Pure (6.6), a Pure-bounded one is.
fn erase_bounded<U: PureValue>(u: &U) -> &dyn PureValue {
    u // admitted
}
fn erase_unbounded<U: Value>(u: &U) -> &dyn PureValue {
    u // FX0510: U has no Pure bound
}
pub fn generic_uses(p: &dyn Value) -> u32 {
    let _ = p;
    0
}
