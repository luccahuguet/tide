//@ rules: REF-0408
// Distinct inherent names and explicitly qualified trait methods are legal.
pub struct Stone<T>(T);
impl Stone<u8> { pub fn new_byte() {} }
impl Stone<u16> { pub fn new_word() {} }
pub trait First { fn new(); }
pub trait Second { fn new(); }
impl First for Stone<u8> { fn new() {} }
impl Second for Stone<u8> { fn new() {} }
