//@ rules: PURE-2
// A missing `Pure` or `Send` never makes two impls disjoint, so both pairs overlap.
use std::net::Net;
use std::rc::Rc;

pub trait Describe {}
impl<T: Pure> Describe for T {}
impl Describe for Net {} //~ E0119 FX0402

pub trait Ship {}
impl<T: Send> Ship for T {}
pub struct Local(Rc<u8>);
impl Ship for Local {} //~ E0119 FX0402
