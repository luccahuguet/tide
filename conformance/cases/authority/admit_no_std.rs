//@ rules: ADMIT-1, REF-0912
// Every crate's std is Tide's, which rustc injects and the rules key on. Under `#![no_std]`,
// active here through `cfg_attr`, rustc injects `core`, so the `Pure` rules would be off, and an
// `extern crate std` would load Tide's std only as an ordinary crate.
#![cfg_attr(all(), no_std)] //~ TD0520
extern crate std; //~ TD0912
