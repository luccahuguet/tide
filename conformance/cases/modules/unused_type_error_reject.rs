//@ rules: REF-0913
mod extension { pub trait Size { fn size(&self) -> u32; } }
use extension::Size;
// Genuine method-resolution errors leave trait-import usage incomplete.
pub fn measure() { ().size(); } //~ E0599
