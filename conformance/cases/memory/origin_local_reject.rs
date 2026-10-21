//@ rules: ORIGIN-4
// A loan of a local reaching a signature origin conflicts with the local's storage
// death.
pub fn copy_out(v: &u8) -> &u8 {
    let x = *v;
    &x //~ E0515
}

pub fn owned_str(n: usize) -> &'static str {
    let s = n.to_string();
    s.as_str() //~ E0515
}
