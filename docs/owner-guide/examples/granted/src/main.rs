// Owner's guide, chapter 1: the same program written the Tide way. `main` holds every
// capability, narrows them, and hands each function only what it needs.
use std::fs::{Fs, Mode};
use std::time::Clock;

// Can read files, and only under the directory `main` chose. Cannot touch the network,
// the clock, or anything else: its signature holds nothing else.
fn load_config(config: &Fs) -> String {
    config.read_to_string("app.toml").unwrap_or_default()
}

// Can read the clock and nothing else.
fn uptime_label(clock: &Clock) -> String {
    format!("started at {:?}", clock.now())
}

// Holds no capability at all, so it can only compute on its argument.
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[tide::main]
fn main(root: Root) {
    // Attenuation: a read-only view of one directory.
    let Ok(config) = root.fs.dir("/etc/app", Mode::Read) else { return };
    let text = load_config(&config);
    println!("{} words; {}", count_words(&text), uptime_label(&root.clock));
}
