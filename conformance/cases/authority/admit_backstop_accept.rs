//@ rules: ADMIT-1, PURE-1, PAR-7
// The checks after analysis see only instances that can run. A provided method no one calls,
// such as `Iterator::max` for an iterator of function pointers, a method that needs
// `Self: Pure`, or one whose `const` fails for its impl, is not a use; and a generic closure's
// captures are decided at the instance's types, where `T` is `u32`.
use std::par::join;
use std::time::Clock;

pub fn noop() {}

pub struct Calls(pub u8);
impl Iterator for Calls {
    type Item = fn();
    fn next(&mut self) -> Option<fn()> {
        self.0 = self.0.checked_sub(1)?;
        Some(noop)
    }
}

pub fn call_all() {
    for f in Calls(3) {
        f()
    }
}

pub trait Snapshot {
    fn name(&self) -> u8;
    fn snapshot(&self) -> u8
    where
        Self: Pure,
    {
        self.name()
    }
}

pub struct Timed(pub Clock);
impl Snapshot for Timed {
    fn name(&self) -> u8 {
        1
    }
}

pub trait Nonempty {
    const LEN: usize;
    fn first_index(&self) -> usize {
        const { assert!(Self::LEN > 0) };
        0
    }
}

pub struct Nothing;
impl Nonempty for Nothing {
    const LEN: usize = 0;
}

fn paired<T: Pure + Sync>(t: &T) -> (usize, ()) {
    join(|| std::mem::size_of_val(t), || ())
}

pub fn run() -> usize {
    paired(&1u32).0
}
