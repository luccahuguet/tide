//@ rules: REF-0701
// Catching a panic needs task isolation, even through a renamed import.
use std::panic::catch_unwind as recover; //~ E0432 FX0701
pub fn direct() { let _ = std::panic::catch_unwind(|| 4); } //~ E0433 FX0701
pub fn renamed() { let _ = recover(|| 5); } //~ FX0701
