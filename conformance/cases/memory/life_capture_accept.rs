//@ rules: LIFE-2
// `use<..>` on a returned `impl Trait` lists every type parameter in scope and every
// lifetime its bounds name, and may leave out a lifetime it does not capture.
pub fn cloned<'a, T: Clone>(x: &'a T) -> impl Clone + use<T> {
    x.clone()
}

pub fn keep<'a>(x: &'a u8) -> impl Sized + 'a + use<'a> {
    x
}
