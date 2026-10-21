//@ rules: LIFE-4
// Neither rule resolves the output: two input origins and no receiver, or no input
// origin at all.
pub fn longest(a: &str, b: &str) -> &str { //~ E0106 FX0202
    if a.len() > b.len() { a } else { b }
}

pub fn none() -> &u8 { //~ E0106 FX0202
    &0
}
