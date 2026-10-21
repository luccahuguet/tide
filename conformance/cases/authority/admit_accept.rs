//@ rules: ADMIT-1
// Ordinary code over admitted `core` and `alloc` items: collections, `Rc`, `RefCell`,
// `OnceCell`, formatting (through `&mut` references too), iterators, integer methods, float
// constants, `dyn Any`, and derives;
// and items of re-exported std types that do no I/O: path components, `OsStr`, `io::Error`
// from a code, and `ExitCode`'s constants.
use std::cell::{OnceCell, RefCell};
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::Path;
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

pub fn show(n: &mut u32, s: &mut String) -> String {
    format!("{n:?} {s}")
}

pub fn name(p: &Path) -> Option<&str> {
    p.file_name()?.to_str()
}

pub fn not_found() -> Option<i32> {
    io::Error::from_raw_os_error(2).raw_os_error()
}

pub fn precision() -> u32 {
    f64::MANTISSA_DIGITS + f32::RADIX
}

pub fn is_count(v: &dyn std::any::Any) -> bool {
    v.is::<u32>()
}

#[derive(Debug, PartialEq)]
pub enum Shape {
    Dot(i64),
    Line(i64, i64),
}

pub fn status(ok: bool) -> std::process::ExitCode {
    if ok { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}
