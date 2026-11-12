//@ rules: ROOT-3
// Building with --test does not exempt ordinary code, including imports and aliases.
use std::sim::SimRoot; //~ FX0504
pub type Simulation = std::sim::SimRoot; //~ FX0504
pub fn helper() { let _ = Root::sim(1); } //~ FX0504
#[cfg(any(test, not(test)))]
pub fn not_test_only() { let _ = Root::sim(2); } //~ FX0504
pub fn typed(_: &SimRoot) {} //~ FX0504
pub fn via_sink() { eprintln!("{:?}", Root::sim(4).elapsed()); } //~ FX0504
