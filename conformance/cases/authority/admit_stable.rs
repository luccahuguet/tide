//@ rules: ADMIT-2
// Stable language features need no gate: let chains (Rust 2024).
pub fn small(a: Option<u32>) -> bool {
    if let Some(x) = a && x < 10 { true } else { false }
}
