//@ rules: SINK-8
// Library stdout is a warning, including renamed macro imports.
use std::println as announce;

pub fn report(n: u32) {
    print!("items: "); //~ WARN TD0508
    announce!("{n}"); //~ WARN TD0508
}
