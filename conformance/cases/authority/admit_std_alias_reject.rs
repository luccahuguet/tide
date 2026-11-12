//@ rules: ADMIT-1
// Aliases and imported names keep the same FX0520 as the absolute std paths.
use std::sync as s;
use std::sync::LazyLock as L; //~ E0432 FX0520
pub fn module_alias() { let _ = s::LazyLock::new(|| 1u32); } //~ E0433 FX0520
pub fn imported() { let _ = L::new(|| 1u32); } //~ FX0520
