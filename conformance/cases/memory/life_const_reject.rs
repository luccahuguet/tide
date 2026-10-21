//@ rules: LIFE-5
// A static's elided lifetime is `'static`, so it cannot hold a borrow of a local.
pub static BAD: &u32 = {
    let n = 1;
    &n //~ E0597
};
