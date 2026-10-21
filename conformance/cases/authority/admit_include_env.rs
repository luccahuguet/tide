//@ rules: ADMIT-1
// Compile-time reads of files and the build environment are not admitted.
pub const SOURCE: &str = core::include_str!("admit_include_env.rs"); //~ FX0520
pub const USER: Option<&str> = core::option_env!("USER"); //~ FX0520
