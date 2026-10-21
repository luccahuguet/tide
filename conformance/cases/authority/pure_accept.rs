//@ rules: PURE-1
// `Pure` looks through fields and `Rc`; `String`, `Rc`, `RefCell`, closures of `Pure` captures,
// and bounded type parameters are `Pure`, and a companion supertrait gives a `Pure` object.
use std::cell::RefCell;
use std::rc::Rc;

pub trait Value {
    fn get(&self) -> usize;
}
pub trait PureValue: Value + Pure {}
impl<T: Value + Pure + ?Sized> PureValue for T {}

pub struct Plain {
    n: usize,
    tag: String,
    cache: Rc<RefCell<Vec<u8>>>,
}

impl Value for Plain {
    fn get(&self) -> usize {
        self.n + self.tag.len() + self.cache.borrow().len()
    }
}

fn run_pure<T: PureValue>(t: &T) -> usize {
    t.get()
}

fn job<F: Fn() -> usize + Pure>(f: F) -> usize {
    f()
}

fn erase<U: PureValue>(u: &U) -> &dyn PureValue {
    u
}

pub fn uses(p: &Plain) -> usize {
    let obj: &dyn PureValue = p;
    let base: &dyn Value = obj;
    let n = 5usize;
    run_pure(p) + job(move || n + 1) + erase(p).get() + base.get()
}
