//@ rules: ORIGIN-1
// A reference flows into a place whose origin may carry its loans: a declared
// `'a: 'b`, `'static` into any origin, and a local whose loan stays live only while
// the borrowed local does.
pub fn shorten<'a: 'b, 'b>(x: &'a u8) -> &'b u8 {
    x
}

pub fn literal<'a>() -> &'a str {
    "lit"
}

pub fn local() -> u8 {
    let r: &u8;
    let x = 1;
    r = &x;
    *r
}
