//@ rules: CAP-1
// The same operations as methods on capabilities the caller passes in.
use std::fs::Fs;
use std::time::{Clock, Instant};

pub fn config(fs: &Fs) -> String {
    fs.read_to_string("/etc/app.toml").unwrap_or_default()
}

pub fn started(clock: &Clock) -> Instant {
    clock.now()
}
