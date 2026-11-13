//@ rules: REF-0910
// Inactive modules need no file; inline modules load no file.
#[cfg(any())]
mod absent;
mod present { pub struct T; }
pub fn make() -> present::T { present::T }
