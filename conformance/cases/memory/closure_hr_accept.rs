//@ rules: CLOSURE-2
// A closure returning a borrow of its argument takes its signature from an
// expected type, or relates the two origins by a named lifetime, or leaves its
// types to inference.
fn apply<F: Fn(&str) -> &str>(f: F, s: &str) -> usize { f(s).len() }

pub fn first(s: &str) -> usize { apply(|t| &t[..1], s) }

pub fn named<'a>(s: &'a str) -> &'a str {
    let f = |t: &'a str| -> &'a str { t };
    f(s)
}

pub fn inferred(s: &str) -> &str {
    let f = |t| t;
    f(s)
}
