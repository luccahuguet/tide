//@ rules: BORROW-3
// Supply the borrowing relation, and finish local borrows while their owner is live.
pub fn relation<'a, 'b: 'a>(s: &'b str) -> Box<dyn Fn() -> usize + 'a> { Box::new(move || s.len()) }
pub fn internal() -> Box<dyn Fn() -> usize> { Box::new(|| { let s = String::from("x"); let r = &s; r.len() }) }
pub fn captured_internal(tag: &'static str) -> Box<dyn Fn() -> usize> { Box::new(move || { let s = String::from("x"); let r = &s; r.len() + tag.len() }) }
fn take(_: Box<dyn Fn() -> usize>, _: &'static str) {}
pub fn other_argument(s: &'static str) { let text = String::from("x"); take(Box::new(move || text.len()), s); }
pub fn copied_argument(s: &'static str) { let owned = s.to_string(); take(Box::new(move || owned.len()), s); }
