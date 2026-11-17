//@ rules: REF-0911
// Red team (fx-qlek.5, Opus r2 M2): a `#[path]` that `cfg_attr` produces on an out-of-line module
// is still a path attribute. rustc reads the file while parsing, and the crate is rejected, so no
// artifact is built.
#[cfg_attr(all(), path = "extern_crate_accept.rs")] //~ TD0911
mod hidden;
