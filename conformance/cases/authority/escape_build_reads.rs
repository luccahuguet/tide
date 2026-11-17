//@ rules: ADMIT-1
// Red team (fx-qlek.5): compile-time reads of the build environment and of files, beyond
// `admit_include_env`: `env!`, `include_bytes!`, a computed `include_str!` path, and `env!` in
// an attribute. Each is `TD0520` (and `concat!` is `TD0103`); the driver also runs rustc with an environment it sets (RUN-2).
pub const HOME: &str = core::env!("HOME"); //~ TD0520
pub const BYTES: &[u8] = core::include_bytes!("escape_build_reads.rs"); //~ TD0520
pub const JOINED: &str = core::include_str!(core::concat!("escape_build", "_reads.rs")); //~ TD0103 TD0520

#[doc = core::env!("HOME")] //~ TD0520
pub fn documented() {}
