//@ rules: CLAUSE-4, AFREE-2
// Const lowering also snapshots only postcondition uses. Its Copy-only entry
// snapshot rejects &mut independently; admission still names the prophecy rule.
#[ensures(*x == *y)] //~ E0277 FX0518 FX0803
pub const fn shared<'a>(x: &'a mut u32, y: &'a u32) { let _ = x; let _ = y; }
