//@ rules: SINK-8
//@ profile: release
// `dbg!` is an error in the release profile.
pub fn half(x: u32) -> u32 {
    dbg!(x) / 2 //~ TD0507
}
