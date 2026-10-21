//@ rules: GLOBAL-4, ADMIT-1
// `{:p}` prints an address, so it needs the `unsafe` grant too.
pub fn show(x: &u8) -> String {
    format!("{:p}", x) //~ FX0520
}
