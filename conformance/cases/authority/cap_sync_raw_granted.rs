//@ rules: CAP-3
//@ grants: unsafe
// Under the `unsafe` grant, `std::sync::raw` is admitted.
pub fn peek(_m: &std::sync::raw::Mutex<u32>) {}
