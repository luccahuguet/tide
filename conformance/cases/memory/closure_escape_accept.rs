//@ rules: CLOSURE-1
// A borrowing closure is used locally, passed, stored in a borrowing type, or returned where its
// origins are named or captured; an owning `move` closure goes anywhere.
pub struct Row { n: u32 }
pub struct Filter<'p> { pred: Box<dyn Fn(&Row) -> bool + 'p> }
pub struct Holder<F> { f: F }

pub fn local(xs: &[Row], min: u32) -> usize { let big = xs.iter().filter(|r| r.n > min); big.count() }
pub fn over(min: &u32) -> impl Fn(&Row) -> bool { move |r| r.n > *min }
pub fn boxed<'a>(min: &'a u32) -> Box<dyn Fn(&Row) -> bool + 'a> { Box::new(move |r| r.n > *min) }
pub fn filter(min: &u32) -> Filter<'_> { Filter { pred: Box::new(move |r| r.n > *min) } }
pub fn hold(min: &u32) -> Holder<impl Fn(&Row) -> bool> { Holder { f: move |r: &Row| r.n > *min } }
pub fn owned(name: &str) -> Box<dyn Fn() -> usize> { let name = name.to_string(); Box::new(move || name.len()) }
