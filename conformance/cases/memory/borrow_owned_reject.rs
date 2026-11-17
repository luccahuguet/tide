//@ rules: BORROW-3
// A borrowed value cannot fill an `Owned` position: `Box<dyn Trait>` in a field or `dyn Any`.
use std::any::Any;
use std::fmt::Display;

pub struct Shown { v: Box<dyn Display> }

pub fn shown(s: &str) -> Shown { Shown { v: Box::new(s) } } //~ error
pub fn any_of(s: &str) -> Box<dyn Any> { Box::new(s) } //~ error
pub fn hook(s: &str) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) } //~ error TD0203
