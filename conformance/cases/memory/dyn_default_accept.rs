//@ rules: DYN-1
// Behind a reference a `dyn` takes the reference's origin; a stated or elided bound borrows; bodies infer.
use std::fmt::Display;

pub struct Ref<'a, T: ?Sized + 'a> { r: &'a T }

pub fn visit(nodes: &[u32], f: &mut dyn FnMut(&u32)) { for n in nodes { f(n) } }
pub fn total(nodes: &[u32]) -> u32 { let mut sum = 0; visit(nodes, &mut |n| sum += *n); sum }
pub fn shown<'a>(s: &'a str) -> Box<dyn Display + 'a> { Box::new(s) }
pub fn shown_elided(s: &str) -> Box<dyn Display + '_> { Box::new(s) }
pub fn owned(s: &str) -> Box<dyn Display> { Box::new(s.to_string()) }
pub fn local(s: &str) -> usize { let f: Box<dyn Fn() -> usize> = Box::new(|| s.len()); f() }
pub fn by_bound<'a>(f: &'a (dyn Fn() + 'a)) -> Ref<'a, dyn Fn()> { Ref { r: f } }
