//@ rules: VAR-1, VAR-2, VAR-3
// Covariant positions shrink an origin; `fn` arguments widen; unsizing may shorten a `dyn` behind `&mut`.
use std::rc::Rc;

pub struct View<'a> { s: &'a str }

pub fn shrink_ref<'a, 'b: 'a>(x: &'a &'b str) -> &'a &'a str { x }
pub fn shrink_outer<'a, 'b: 'a, 'c>(x: &'b mut &'c str) -> &'a mut &'c str { x }
pub fn shrink_vec<'a, 'b: 'a>(x: Vec<&'b str>) -> Vec<&'a str> { x }
pub fn shrink_many<'a, 'b: 'a>(x: (Option<Box<&'b str>>, [&'b str; 2], Rc<&'b str>)) -> (Option<Box<&'a str>>, [&'a str; 2], Rc<&'a str>) { x }
pub fn shrink_slice<'a, 'b: 'a>(x: &'a [&'b str]) -> &'a [&'a str] { x }
pub fn shrink_view<'a, 'b: 'a>(x: View<'b>) -> View<'a> { x }
pub fn widen_arg<'a>(f: fn(&'a str)) -> fn(&'static str) { f }
pub fn shrink_obj<'a>(b: Box<dyn Fn() + 'static>) -> Box<dyn Fn() + 'a> { b }
pub fn unsize_mut<'a, F: Fn() + 'static>(f: &'a mut F) -> &'a mut (dyn Fn() + 'a) { f }
pub fn shorten_mut<'a>(f: &'a mut (dyn Fn() + 'static)) -> &'a mut (dyn Fn() + 'a) { f }
