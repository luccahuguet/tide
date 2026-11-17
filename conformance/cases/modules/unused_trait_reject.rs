//@ rules: REF-0913
mod extension {
    pub trait Size { fn size(&self) -> u32; }
    pub struct S;
    impl S { pub fn size(&self) -> u32 { 1 } }
}
// The resolver considers this trait, but method resolution uses the inherent method.
#[allow(unused_imports)]
use extension::Size; //~ TD0913
pub fn measure(x: extension::S) -> u32 { x.size() }
