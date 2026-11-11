//@ rules: REF-0405
// An ordinary marker and a field express the restriction.
use std::marker::PhantomData;
use std::rc::Rc;
pub trait Shape {}
pub struct Stone(PhantomData<Rc<()>>);
impl Shape for Stone {}
