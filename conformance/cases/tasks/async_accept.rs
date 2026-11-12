//@ rules: REF-0601
// Ordinary functions and closures need no executor or async machinery.
pub fn fetch() -> u8 { 3 }
pub fn caller() -> u8 { let next = || fetch() + 1; next() }
