//@ rules: AFREE-1
// `#[pure]` on a function that takes a capability.
use std::fs::Fs;

#[pure]
pub fn configured(fs: &Fs) -> bool { //~ FX0511
    fs.read("/etc/app.toml").is_ok()
}
