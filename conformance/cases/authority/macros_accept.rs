//@ rules: REF-0103
// Nested helpers emitted by admitted forms are not user-written macros.
use std::io::Write;
use core::assert_eq as equal;
use std::prelude::rust_2024::info as report;
pub fn forms() {
    let a = vec![1u8, 2];
    equal!(a.len(), 2);
    assert_ne!(a[0], a[1]);
    assert!(matches!(a[0], 1));
    debug_assert!(a[0] != a[1]);
    let mut out = Vec::new();
    write!(out, "{}", format!("{}", a[0])).unwrap();
    writeln!(out, "ok").unwrap();
    eprint!("{}", "");
    eprintln!("ok");
    report!("a renamed log sink");
}
