//@ rules: ROOT-1, ADMIT-1
// `#![no_std]` is not admitted (`admit_no_std`); without the injected std, a `main` still needs
// `#[tide::main]`, and the driver does not take the explicit `std` for Tide's std, so its items
// are not admitted.
#![no_std] //~ FX0520
extern crate std;

fn main() {} //~ FX0512 FX0520
