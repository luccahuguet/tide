//@ rules: ROOT-1
// `#![no_main]` leaves a binary without the entry point that `#[tide::main]` generates; below the
// crate root, rustc ignores it.
#![no_main] //~ TD0512

mod inert {
    #![no_main]
}

fn main() {}
