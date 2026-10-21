//@ rules: SINK-8
// In the default debug profile, `dbg!` writes to its sink.
pub fn half(x: u32) -> u32 {
    dbg!(x) / 2
}
