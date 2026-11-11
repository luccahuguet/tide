//@ rules: SCOPE-3, BORROW-3
use std::task::Tasks;
pub fn once(tasks: &Tasks) {
    let text = String::from("x");
    let s;
    s = text;
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn reassigned(tasks: &Tasks) {
    let mut s = "static";
    s = "also static";
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn paths(tasks: &Tasks, choose: bool) {
    let s;
    if choose { s = "one"; } else { s = "two"; }
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn twice(tasks: &Tasks) {
    let mut s;
    s = "one";
    s = "two";
    let _ = tasks.spawn_isolated(move || s.len());
}
pub fn one_path(tasks: &Tasks, choose: bool) {
    let s;
    if choose { s = "static"; } else { return; }
    let _ = tasks.spawn_isolated(move || s.len());
}
