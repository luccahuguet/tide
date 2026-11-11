//@ rules: REF-0401
// Either exact defining module works; a function body uses its enclosing module.
pub struct Local;
mod owner {
    pub trait Shape {}
    pub struct Stone;
    impl Shape for super::Local {}
    impl Shape for u8 {}
}
pub struct Other;
impl owner::Shape for Other {}
impl Local {}
pub fn scoped() { impl Local { pub fn value() -> u8 { 1 } } }
