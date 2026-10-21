//@ rules: VAR-1, VAR-2, VAR-3
// Invariant positions (`&mut T`'s `T`, cells, `Shared`, `dyn Trait<A>`'s `A`) keep an origin fixed.
use std::cell::Cell;
use std::sync::Shared;

pub trait Sink<T> { fn put(&self, t: T); }
pub struct Slot<'a, T> { r: &'a mut T }

pub fn grow<'a>(x: &'a str) -> &'static str { x } //~ error
pub fn shrink_inner<'a, 'b: 'a>(x: &'a mut &'b str) -> &'a mut &'a str { x } //~ error
pub fn shrink_cell<'a, 'b: 'a>(x: &'a Cell<&'b str>) -> &'a Cell<&'a str> { x } //~ error
pub fn shrink_shared<'a, 'b: 'a>(x: Shared<&'b str>) -> Shared<&'a str> { x } //~ error
pub fn narrow_arg<'a>(f: fn(&'static str)) -> fn(&'a str) { f } //~ error
pub fn shrink_param<'a, 'b: 'a>(x: &'a dyn Sink<&'b str>) -> &'a dyn Sink<&'a str> { x } //~ error
pub fn shrink_slot<'a, 'b: 'a>(x: Slot<'a, &'b str>) -> Slot<'a, &'a str> { x } //~ error
pub fn no_unsize<'a>(x: &'a mut Box<dyn Fn() + 'static>) -> &'a mut Box<dyn Fn() + 'a> { x } //~ error
