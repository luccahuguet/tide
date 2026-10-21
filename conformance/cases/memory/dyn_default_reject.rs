//@ rules: DYN-1
// Outside a body and not directly behind a reference, an unbounded `dyn` is `dyn Trait + 'static`.
use std::fmt::Display;

pub struct Hook { run: Box<dyn Fn()> }

pub fn shown(s: &str) -> Box<dyn Display> { Box::new(s) } //~ error
pub fn hook<'a>(f: &'a dyn Fn()) -> Hook { Hook { run: Box::new(f) } } //~ error
