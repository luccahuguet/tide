//@ rules: ROOT-3
// Building with --test does not exempt ordinary code, including imports and aliases.
use std::sim::SimRoot; //~ TD0504
pub type Simulation = std::sim::SimRoot; //~ TD0504
pub fn helper() { let _ = Root::sim(1); } //~ TD0504
#[cfg(any(test, not(test)))]
pub fn not_test_only() { let _ = Root::sim(2); } //~ TD0504
pub fn typed(_: &SimRoot) {} //~ TD0504
pub fn via_sink() { eprintln!("{:?}", Root::sim(4).elapsed()); } //~ TD0504
