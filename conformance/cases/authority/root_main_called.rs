//@ rules: ROOT-5
// No code calls `main`, so nothing but the entry point mints a real `Root`.
#[tide::main]
fn main() {}

pub fn again() {
    let _ = main(); //~ FX0506
}
