//@ rules: REF-0401
// Either exact defining module works; a function body uses its enclosing module.
pub struct Local;
mod owner {
    pub trait Shape {}
    pub struct Stone;
    impl Shape for super::Local {}
    impl Shape for u8 {}
    impl Shape for &super::Other {}
}
pub struct Other;
impl owner::Shape for Other {}
impl Local {}
pub fn scoped() { impl Local { pub fn value() -> u8 { 1 } } }

// Fundamental wrappers keep the innermost local type's defining module.
mod bag {
    pub struct Bag;
    impl IntoIterator for &Bag {
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for &mut Bag {
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for Box<Bag> {
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for &mut Box<&Bag> {
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for Box<Box<Bag>> {
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
}
