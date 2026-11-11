//@ rules: SCOPE-3, BORROW-3
use std::task::Tasks;
pub fn once(tasks: &Tasks) {
    let text = String::from("x");
    let s;
    s = &text; //~ E0597 FX0203
    let _ = tasks.spawn_isolated(move || s.len());
}
// Reassignment has deliberately uncertain provenance: keep rustc's rejection alone.
pub fn reassigned(tasks: &Tasks) {
    let text = String::from("x");
    let mut s = "static";
    s = &text; //~ E0597
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn paths(tasks: &Tasks, choose: bool) {
    let text = String::from("x");
    let s: &str;
    if choose { s = &text; } else { s = "static"; } //~ E0597
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn twice(tasks: &Tasks) {
    let first = String::from("one");
    let second = String::from("two");
    let mut s;
    s = &first; //~ E0597
    s = &second; //~ E0597
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn one_path(tasks: &Tasks, choose: bool) {
    let text = String::from("x");
    let s;
    if choose { s = &text; } else { return; } //~ E0597
    let _ = tasks.spawn_isolated(move || s.len());
}
