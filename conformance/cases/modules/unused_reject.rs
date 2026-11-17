//@ rules: REF-0913
#[allow(unused_imports)]
use std::collections::BTreeMap; //~ TD0913
#[allow(unused_imports)]
use std::io::Read; //~ TD0913
mod leaf { pub struct T; }
use leaf::*; //~ TD0913

mod redundant {
    mod a { pub struct T; }
    mod b { pub use super::a::T; }
    use a::*; //~ TD0913
    use b::*;
    pub fn make() -> T { T }
}
