//@ rules: BORROW-3
// Borrowing-object lifetime relations and closure-body errors are rustc errors, not TD0203.
pub fn relation<'a, 'b>(s: &'b str) -> Box<dyn Fn() -> usize + 'a> { Box::new(move || s.len()) } //~ error
pub fn internal() -> Box<dyn Fn() -> usize> { Box::new(|| { let r; { let s = String::from("x"); r = &s; } r.len() }) } //~ E0597
pub fn captured_internal(tag: &'static str) -> Box<dyn Fn() -> usize> { Box::new(move || { let r; { let s = String::from("x"); r = &s; } r.len() + tag.len() }) } //~ E0597
fn take(_: Box<dyn Fn() -> usize>, _: &'static str) {}
pub fn other_argument(s: &str) { let text = String::from("x"); take(Box::new(move || text.len()), s); } //~ E0521
pub fn copied_argument(s: &str) { let owned = s.to_string(); take(Box::new(move || owned.len()), s); } //~ E0521
