//@ rules: ROOT-1, ADMIT-1
// In a library, a crate-root `#[tide::main] fn main` is no entry point: its call to Tide's std's
// runtime entry is code's own, which could mint a `Root` (RUN-1).
#[tide::main] fn main() {} //~ FX0520
