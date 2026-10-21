//@ rules: BORROW-3
// `Box<dyn Trait>` contents in a signature or field and `dyn Any` hold `Owned` values.
use std::any::Any;
use std::fmt::Display;

pub struct Shown { v: Box<dyn Display> }
pub struct Msg { text: &'static str }

pub fn shown(s: &str) -> Shown { Shown { v: Box::new(s.to_string()) } }
pub fn literal() -> Box<dyn Display> { Box::new("fixed") }
pub fn any_of(s: &str) -> Box<dyn Any> { Box::new(s.len()) }
pub fn msg() -> Box<dyn Any> { Box::new(Msg { text: "static" }) }
pub fn hook(s: &str) -> Box<dyn Fn() -> usize> { let s = s.to_string(); Box::new(move || s.len()) }
