//@ rules: REF-0401
// A descendant is not the defining module; aliases do not move ownership.
pub struct Local;
mod child {
    impl super::Local {} //~ TD0401
    impl IntoIterator for &super::Local { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for &mut super::Local { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for Box<super::Local> { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for &mut Box<&super::Local> { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for Box<Box<super::Local>> { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
}
mod owner { pub trait Shape {} pub struct Stone; }
mod elsewhere {
    type Alias = super::owner::Stone;
    impl Alias {} //~ TD0401
    impl super::owner::Shape for super::Local {} //~ TD0401
    impl IntoIterator for &super::owner::Stone { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for &mut super::owner::Stone { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
    impl IntoIterator for Box<super::owner::Stone> { //~ TD0401
        type Item = u8;
        type IntoIter = std::vec::IntoIter<u8>;
        fn into_iter(self) -> Self::IntoIter { vec![].into_iter() }
    }
}
impl owner::Shape for u8 {} //~ TD0401
impl owner::Shape for Vec<Local> {} //~ TD0401

// A foreign trait and a non-local self type leave no legal defining module.
impl From<Local> for u16 { fn from(_: Local) -> u16 { 0 } } //~ TD0401
