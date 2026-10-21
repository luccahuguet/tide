//@ rules: OPAQUE-1
// A returned `impl Trait` borrows its inputs without annotation; `use<..>` and `+ 'static` capture less.
pub fn evens(xs: &[u32]) -> impl Iterator<Item = u32> { xs.iter().copied().filter(|x| x % 2 == 0) }
pub fn first_evens(xs: &[u32]) -> Vec<u32> { evens(xs).collect() }

pub fn len_of<'a>(xs: &'a [String], key: &str) -> impl Fn() -> usize + use<'a> { let n = key.len(); move || xs.len() + n }
pub fn caller(xs: &[String]) -> usize { let key = String::from("k"); let f = len_of(xs, &key); drop(key); f() }

pub fn owned(xs: &[u32]) -> impl Iterator<Item = u32> + 'static { xs.to_vec().into_iter() }
pub fn outlives() -> usize { let it = { let v = vec![1, 2]; owned(&v) }; it.count() }
