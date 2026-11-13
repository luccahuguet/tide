//@ rules: REF-0913
mod extension {
    pub trait Size { fn size(&self) -> u32; }
    pub struct S;
    impl Size for S { fn size(&self) -> u32 { 1 } }
}
use extension::Size;
pub fn measure(x: extension::S) -> u32 { x.size() }
