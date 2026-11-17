//@ rules: CAP-3
// `std::sync::raw` needs the package's `unsafe` grant.
pub fn peek(_m: &std::sync::raw::Mutex<u32>) {} //~ TD0520
