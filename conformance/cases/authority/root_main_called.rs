//@ rules: ROOT-5
// No code calls `main`, takes it as a value, or imports it, so nothing but the entry point mints
// a real `Root`, a test included.
use crate::main as start; //~ FX0506

#[tide::main]
fn main() {}

pub fn again() {
    let _ = main(); //~ FX0506
    let _f = main; //~ FX0506
    let _ = start(); //~ FX0506
}

#[cfg(test)]
mod tests {
    #[test]
    fn mints() {
        let _ = super::main(); //~ FX0506
    }
}
