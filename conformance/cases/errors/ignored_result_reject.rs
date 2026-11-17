//@ rules: REF-0702
#![allow(unused_must_use)]
// Ignored Results are errors even when rustc's lint is disabled or the error is empty.
pub fn ignored(result: Result<u8, ()>) {
    result; //~ TD0702
    Ok::<(), core::convert::Infallible>(()); //~ TD0702
}
pub fn write(mut text: String) {
    use std::fmt::Write;
    writeln!(text, "entry"); //~ TD0702
}
pub fn blocks() {
    { Ok::<(), ()>(()) }; //~ TD0702
}
