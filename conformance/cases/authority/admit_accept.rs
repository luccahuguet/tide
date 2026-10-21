//@ rules: ADMIT-1
// Ordinary code over admitted `core` and `alloc` items: collections, `Rc`, `RefCell`,
// `OnceCell`, formatting, iterators, and integer methods.
use std::cell::{OnceCell, RefCell};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

pub struct Point {
    x: i64,
    y: i64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub fn run(words: &str) -> String {
    let mut v: Vec<u32> = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).collect();
    v.sort();
    let total = v.iter().copied().fold(0u32, |a, b| a.saturating_add(b));
    let mut index = BTreeMap::new();
    for (i, w) in words.split_whitespace().enumerate() {
        index.insert(w.to_uppercase(), i);
    }
    let log = Rc::new(RefCell::new(Vec::<String>::new()));
    log.borrow_mut().push(format!("{}", Point { x: -1, y: 2 }));
    let first: OnceCell<u32> = OnceCell::new();
    let n = *first.get_or_init(|| total.wrapping_mul(3));
    format!("{v:?} {total} {n} {index:?} {:?}", log.borrow())
}
