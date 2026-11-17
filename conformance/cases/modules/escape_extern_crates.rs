//@ rules: REF-0912
// Red team (fx-qlek.5): the sysroot's other crates. `extern crate` is never admitted, so a crate
// cannot load `proc_macro` or name `alloc` beside Tide's std.
extern crate proc_macro; //~ TD0912
extern crate alloc as real; //~ TD0912

pub fn noop() -> real::string::String {
    real::string::String::new()
}
