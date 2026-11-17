//@ rules: ROOT-1
// The entry attribute is recognized as Tide's std's macro, not by its spelling: here
// `#[tide::main]` names `#[pure]`, which leaves `main` as written.
mod tide {
    pub use std::prelude::rust_2024::pure as main;
}

#[tide::main]
fn main() { //~ TD0512
    println!("hello");
}
