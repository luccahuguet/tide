#[ensures(forall |i: usize| result[i] > 0)]
pub fn f() -> Vec<u8> {
    vec![1]
}
