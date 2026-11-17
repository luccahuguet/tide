//@ rules: REF-0408
// Rust's disjoint impls still share Tide's one-name-per-type namespace.
pub struct Stone<T>(T);
impl Stone<u8> { pub fn new() {} }
impl Stone<u16> { pub fn new() {} } //~ TD0408
pub struct Other;
impl Other { pub fn value() {} } //~ E0592
impl Other { pub fn value() {} } //~ TD0408
