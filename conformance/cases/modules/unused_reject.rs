//@ rules: REF-0913
#[allow(unused_imports)]
use std::collections::BTreeMap; //~ FX0913
#[allow(unused_imports)]
use std::io::Read; //~ FX0913
mod leaf { pub struct T; }
use leaf::*; //~ FX0913

mod redundant {
    mod a { pub struct T; }
    mod b { pub use super::a::T; }
    use a::*; //~ FX0913
    use b::*;
    pub fn make() -> T { T }
}
