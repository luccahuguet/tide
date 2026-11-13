//@ rules: REF-0913
use std::collections::BTreeMap; //~ FX0913
mod extension {
    pub trait Size { fn size(&self) -> u32; }
    pub struct S;
    impl S { pub fn size(&self) -> u32 { 1 } }
}
use extension::Size; //~ FX0913
pub fn measure(x: extension::S) -> u32 { x.size() }

// The independent early error must not make a used extension import look unused.
mod used {
    mod extension {
        pub trait Size { fn size(&self) -> u32; }
        pub struct S;
        impl Size for S { fn size(&self) -> u32 { 1 } }
    }
    use extension::Size;
    pub fn measure(x: extension::S) -> u32 { x.size() }
}
