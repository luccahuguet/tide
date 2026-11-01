//@ rules: ROOT-1, ADMIT-1
// Without the injected std, a `main` still needs `#[tide::main]`, and the driver does not take
// the explicit `std` for Tide's std, so its items are not admitted.
#![no_std]
extern crate std;

fn main() {} //~ FX0512 FX0520
