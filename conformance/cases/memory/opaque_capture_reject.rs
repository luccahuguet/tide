//@ rules: OPAQUE-1
// Without `use<..>` the opaque type captures `key`'s origin; `+ 'static` and `use<>` must hold of the hidden type.
pub fn len_of<'a>(xs: &'a [String], key: &str) -> impl Fn() -> usize { let n = key.len(); move || xs.len() + n }
pub fn caller(xs: &[String]) -> usize { let key = String::from("k"); let f = len_of(xs, &key); drop(key); f() } //~ E0505

pub fn evens(xs: &[u32]) -> impl Iterator<Item = u32> + 'static { xs.iter().copied() } //~ error
pub fn narrow<'a>(x: &'a str) -> impl Sized + use<> { x } //~ E0700
