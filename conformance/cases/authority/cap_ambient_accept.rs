//@ rules: CAP-1
// World operations go through capabilities; data-only std operations need none.
use std::env::Env;
use std::fs::{Fs, OpenOptions};
use std::time::{Clock, Duration};

pub fn inspect(fs: &Fs, env: &Env, clock: &Clock) {
    let _ = fs.read_to_string("settings");
    let _ = fs.open_with("settings", &OpenOptions::new());
    let _ = env.args();
    let _ = env.var("MODE");
    let start = clock.now();
    let _ = clock.now().duration_since(start);
    let _ = clock.system_time();
    let _ = clock.sleep(Duration::ZERO);
    let _ = std::io::stdout();
    let _ = std::path::Path::new("settings").join("child");
}

pub struct Local;
impl Local {
    pub fn now() -> Self { Self }
    pub fn elapsed(&self) -> u64 { 0 }
}
pub fn local() -> u64 { Local::now().elapsed() }

pub fn thread_rng() -> u64 { 4 }
pub fn draw() -> u64 { thread_rng() }
