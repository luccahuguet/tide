//@ rules: PURE-2
// Impls whose type heads differ are disjoint, whatever their auto traits.
use std::net::Net;

pub trait Describe {}
impl<T: Describe> Describe for Vec<T> {}
impl Describe for Net {}
