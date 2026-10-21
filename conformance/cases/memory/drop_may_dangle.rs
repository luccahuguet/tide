//@ rules: DROP-2
// A user-written drop-check exemption: `#[may_dangle]` is an unstable feature,
// which the driver never enables (ADMIT-2). A case holds no `unsafe` grant, so
// the `unsafe impl` is also `FX0901`.
pub struct Bag<T> {
    pub items: Vec<T>,
}
// SAFETY: present, so `FX0902` is not expected.
unsafe impl<#[may_dangle] T> Drop for Bag<T> { //~ E0658 FX0901
    fn drop(&mut self) {}
}
