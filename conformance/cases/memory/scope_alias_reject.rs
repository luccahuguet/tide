//@ rules: SCOPE-3, BORROW-3
// The borrowed alias originates before the owned call; keep the original diagnostic span.
use std::task::Tasks;
pub fn alias(tasks: &Tasks) {
    let text = String::from("x");
    let s = &text; //~ E0597 FX0203
    let _ = tasks.spawn_isolated(move || s.len());
}
// An unrelated borrow in the same function must not acquire the owned call's code.
pub fn unrelated(tasks: &Tasks) {
    let text = String::from("x");
    let _ = tasks.spawn_isolated(move || text.len());
    let r;
    { let other = String::from("x"); r = &other; } //~ E0597
    let _ = r.len();
}
