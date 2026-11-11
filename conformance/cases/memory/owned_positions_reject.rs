//@ rules: BORROW-3
// The same owned-position failure survives an explicit object cast, a binding and an aggregate.
pub fn cast(s: &str) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) as Box<dyn Fn() -> usize> } //~ error FX0203
pub fn local(s: &str) -> Box<dyn Fn() -> usize> { let job: Box<dyn Fn() -> usize> = Box::new(move || s.len()); job } //~ error FX0203
pub fn array(s: &str) -> [Box<dyn Fn() -> usize>; 1] { [Box::new(move || s.len())] } //~ error FX0203
pub fn tuple(s: &str) -> (Box<dyn Fn() -> usize>,) { (Box::new(move || s.len()),) } //~ error FX0203
pub fn optional(s: &str) -> Option<Box<dyn Fn() -> usize>> { Some(Box::new(move || s.len())) } //~ error FX0203
pub fn named<'a>(s: &'a str) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) } //~ error FX0203
pub struct Borrow<'a> { pub text: &'a str }
impl<'a> Borrow<'a> {
    pub fn callback(&self) -> Box<dyn Fn() -> usize> { let text = self.text; Box::new(move || text.len()) } //~ error
}
pub fn local_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let s = &text; Box::new(move || s.len()) } //~ E0515 FX0203
pub struct Holder { pub job: Box<dyn Fn() -> usize> }
pub fn assign(h: &mut Holder, s: &str) { h.job = Box::new(move || s.len()); } //~ error FX0203
pub fn mut_alias() -> Box<dyn Fn() -> usize> { let mut text = String::from("x"); let s = &mut text; Box::new(move || s.len()) } //~ E0515 FX0203
