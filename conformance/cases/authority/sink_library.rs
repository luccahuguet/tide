//@ rules: SINK-8
// Library stdout is a warning, including renamed macro imports.
use std::println as announce;

pub fn report(n: u32) {
    print!("items: "); //~ WARN FX0508
    announce!("{n}"); //~ WARN FX0508
}
