//@ rules: ROOT-1, ROOT-2, ROOT-4
// `#[tide::main] fn main(root: Root)`; its fields are moved or borrowed out one by one.
use std::fs::Fs;
use std::time::Clock;

fn report(fs: &Fs, clock: &Clock) {
    let _ = (fs.read("/etc").is_ok(), clock.now());
}

#[tide::main]
fn main(root: Root) {
    let fs = root.fs.read_only();
    report(&fs, &root.clock);
}
