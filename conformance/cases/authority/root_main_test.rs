//@ rules: ROOT-5
// A `#[test]` on the binary's `main` keeps `#[tide::main]`, and the test it generates calls that
// `main`, which mints a real `Root` in a test. The call is the toolchain's, but it names `main`.
#[cfg_attr(test, test)] //~ FX0506
#[tide::main]
fn main() {}
