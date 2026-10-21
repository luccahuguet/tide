//@ rules: ORIGIN-3
// A body relies on inclusions between signature origins that a declared bound or the
// well-formedness of a parameter type gives.
use std::collections::BTreeMap;

pub fn pick<'r, 'a: 'r, 'b: 'r>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str {
    if s.is_empty() { v[0] } else { s }
}

pub fn words<'m, 't>(m: &'m BTreeMap<char, Vec<&'t str>>) -> &'m [&'t str] {
    &m[&'a']
}
