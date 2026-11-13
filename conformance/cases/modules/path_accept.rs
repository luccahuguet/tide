//@ rules: REF-0911
mod present { pub struct T; }
pub fn make() -> present::T { present::T }
