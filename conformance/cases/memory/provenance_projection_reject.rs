//@ rules: BORROW-3
// The captured static element must not acquire its sibling's borrow provenance.
pub fn tuple() -> (Box<dyn Fn() -> usize>, &'static str) {
    let text = String::from("x");
    let (s, other) = ("static", text.as_str());
    (Box::new(move || s.len()), other) //~ E0515
}
pub struct Parts<'a> { pub s: &'static str, pub other: &'a str }
pub fn fields() -> (Box<dyn Fn() -> usize>, &'static str) {
    let text = String::from("x");
    let Parts { s, other } = Parts { s: "static", other: text.as_str() };
    (Box::new(move || s.len()), other) //~ E0515
}
pub fn array() -> (Box<dyn Fn() -> usize>, &'static str) {
    let text = String::from("x");
    let [s, other] = ["static", text.as_str()];
    (Box::new(move || s.len()), other) //~ E0515
}
// Destructuring now makes the complete provenance chain uncertain.
pub fn captured_tuple() -> Box<dyn Fn() -> usize> {
    let text = String::from("x");
    let (s, _) = (text.as_str(), "static");
    Box::new(move || s.len()) //~ E0515
}
pub fn captured_array() -> Box<dyn Fn() -> usize> {
    let text = String::from("x");
    let [s, _] = [text.as_str(), "static"];
    Box::new(move || s.len()) //~ E0515
}
pub struct Borrow<'a> { pub s: &'a str, pub other: &'static str }
pub fn captured_field() -> Box<dyn Fn() -> usize> {
    let text = String::from("x");
    let Borrow { s, .. } = Borrow { s: text.as_str(), other: "static" };
    Box::new(move || s.len()) //~ E0515
}
fn values(text: &str) -> (&'static str, &str) { ("static", text) }
pub fn unknown() -> (Box<dyn Fn() -> usize>, &'static str) {
    let text = String::from("x");
    let (s, other) = values(&text);
    (Box::new(move || s.len()), other) //~ E0515
}
pub fn tuple_rest() -> (Box<dyn Fn() -> usize>, &'static str) { let text = String::from("x"); let (_, .., s, other) = (0, 1, "static", text.as_str()); (Box::new(move || s.len()), other) } //~ E0515
pub fn array_rest() -> (Box<dyn Fn() -> usize>, &'static str) { let text = String::from("x"); let [_, .., s, other] = ["one", "two", "static", text.as_str()]; (Box::new(move || s.len()), other) } //~ E0515
pub fn projected() -> (Box<dyn Fn() -> usize>, &'static str) {
    let text = String::from("x");
    let parts = ("static", text.as_str());
    (Box::new(move || parts.0.len()), parts.1) //~ E0515
}
pub fn projection_alias() -> (Box<dyn Fn() -> usize>, &'static str) { let text = String::from("x"); let parts = ("static", text.as_str()); let s = parts.0; (Box::new(move || s.len()), parts.1) } //~ E0515
pub fn projection_assignment() -> (Box<dyn Fn() -> usize>, &'static str) { let text = String::from("x"); let parts = ("static", text.as_str()); let s; s = parts.0; (Box::new(move || s.len()), parts.1) } //~ E0515
pub fn projection_borrow() -> Box<dyn Fn() -> usize> { let pair = (String::from("x"),); Box::new(|| pair.0.len()) } //~ E0373 TD0203
