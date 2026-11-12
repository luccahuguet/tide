//@ rules: REF-0702
#![allow(unused_must_use)]
// Ignored Results are errors even when rustc's lint is disabled or the error is empty.
pub fn ignored(result: Result<u8, ()>) {
    result; //~ FX0702
    Ok::<(), core::convert::Infallible>(()); //~ FX0702
}
pub fn write(mut text: String) {
    use std::fmt::Write;
    writeln!(text, "entry"); //~ FX0702
}
pub fn blocks() {
    { Ok::<(), ()>(()) }; //~ FX0702
}
