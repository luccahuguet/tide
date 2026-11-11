//@ rules: SCOPE-3, BORROW-3
use std::task::Tasks;
// Static aliases meet the obligation; clone borrowed text before moving it into the closure.
pub fn alias(tasks: &Tasks) { let s = "x"; let _ = tasks.spawn_isolated(move || s.len()); }
pub fn copied(tasks: &Tasks) { let text = String::from("x"); let s = &text; let owned = s.to_string(); let _ = tasks.spawn_isolated(move || owned.len()); }
pub fn unrelated(tasks: &Tasks) { let text = String::from("x"); let _ = tasks.spawn_isolated(move || text.len()); let other = String::from("x"); let r = &other; let _ = r.len(); }
