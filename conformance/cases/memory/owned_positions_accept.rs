//@ rules: BORROW-3
// A named borrowing-object lifetime permits each shape; moving owned data permits static.
pub fn cast<'a>(s: &'a str) -> Box<dyn Fn() -> usize + 'a> { Box::new(move || s.len()) as Box<dyn Fn() -> usize + 'a> }
pub fn local<'a>(s: &'a str) -> Box<dyn Fn() -> usize + 'a> { let job: Box<dyn Fn() -> usize + 'a> = Box::new(move || s.len()); job }
pub fn array<'a>(s: &'a str) -> [Box<dyn Fn() -> usize + 'a>; 1] { [Box::new(move || s.len())] }
pub fn tuple<'a>(s: &'a str) -> (Box<dyn Fn() -> usize + 'a>,) { (Box::new(move || s.len()),) }
pub fn optional<'a>(s: &'a str) -> Option<Box<dyn Fn() -> usize + 'a>> { Some(Box::new(move || s.len())) }
pub fn owned(s: &str) -> Box<dyn Fn() -> usize> { let s = s.to_string(); Box::new(move || s.len()) }
pub fn named<'a>(s: &'a str) -> Box<dyn Fn() -> usize + 'a> { Box::new(move || s.len()) }
pub struct Borrow<'a> { pub text: &'a str }
impl<'a> Borrow<'a> {
    pub fn callback(&self) -> Box<dyn Fn() -> usize + 'a> { let text = self.text; Box::new(move || text.len()) }
}
pub fn local_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let s = &text; let owned = s.to_string(); Box::new(move || owned.len()) }
pub struct Holder<'a> { pub job: Box<dyn Fn() -> usize + 'a> }
pub fn assign<'a>(h: &mut Holder<'a>, s: &'a str) { h.job = Box::new(move || s.len()); }
pub fn mut_alias() -> Box<dyn Fn() -> usize> { let mut text = String::from("x"); let s = &mut text; let owned = s.to_string(); Box::new(move || owned.len()) }

// A destructured binding denotes a field, not the enclosing enum initializer.
pub enum Inner { Value { n: usize } }
pub enum Outer { Value { inner: Inner } }
fn take_inner(_: Inner) -> usize { 0 }
pub fn destructured() -> usize {
    let Outer::Value { inner } = Outer::Value { inner: Inner::Value { n: 0 } };
    take_inner(inner)
}
