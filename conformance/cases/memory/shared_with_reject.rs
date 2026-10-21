//@ rules: SHARED-1
// Each payload reference has a fresh higher-ranked `σ`, so nothing carrying it
// escapes, into the result or into a captured place.
use std::sync::Shared;

pub fn into_result(cell: &Shared<u64>) {
    let _r = cell.with(|n| n); //~ error
}

pub fn first_of_two(a: &Shared<u64>, b: &Shared<u64>) {
    let _r = Shared::with2(a, b, |x, _y| x); //~ error
}

pub fn into_capture(cell: &Shared<Vec<u64>>) {
    let mut keep: Option<&u64> = None;
    let _ = cell.with(|v| keep = v.first()); //~ E0521
    let _ = keep;
}
