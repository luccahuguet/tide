//@ rules: ORIGIN-1, LOAN-2, ORIGIN-3
// Assigning `&x` to `r` puts `x`'s loan in `r`'s origin, which is live at the later
// use, so `x`'s storage death conflicts with it. Pushing `s` into the vector needs
// `'s: 'a`, which no bound gives.
pub fn outlived() -> u8 {
    let r: &u8;
    {
        let x = 1;
        r = &x; //~ E0597
    }
    *r
}

pub fn store<'a, 's>(v: &mut Vec<&'a str>, s: &'s str) {
    v.push(s); //~ error
}
