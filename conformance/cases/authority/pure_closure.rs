//@ rules: PURE-1
// A closure is `Pure` exactly when its captures are; this one captures `&Fs`.
use std::fs::Fs;

fn job<F: Fn() -> bool + Pure>(f: F) -> bool {
    f()
}

pub fn probe(fs: &Fs) -> bool {
    job(|| fs.read("/etc").is_ok()) //~ TD0517
}
