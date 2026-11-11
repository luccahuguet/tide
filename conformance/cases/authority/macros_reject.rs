//@ rules: REF-0103
// Helpers outside the fixed list remain errors, including aliases of core macros.
use core::concat as joined;
pub fn forms(a: u8, b: u8) {
    core::debug_assert_eq!(a, b); //~ FX0103
    core::debug_assert_ne!(a, b); //~ FX0103
    let _ = joined!("a", "b"); //~ FX0103
    let _ = format!("{}", r#joined /* still a written call */ !("a", "b")); //~ FX0103
    let _ = stringify!(a); //~ FX0103
    let _ = cfg!(test); //~ FX0103
    let _ = core::line!(); //~ FX0103
    let _ = core::file!(); //~ FX0103
}

#[requires(n > 0)]
pub fn contracted(n: u8) {
    let _ = core::line!(); //~ FX0103
}
