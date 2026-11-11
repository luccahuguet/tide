//@ rules: BORROW-3
// Both returned elements have static provenance.
pub fn tuple() -> (Box<dyn Fn() -> usize>, &'static str) {
    let (s, other) = ("static", "other");
    (Box::new(move || s.len()), other)
}
pub struct Parts<'a> { pub s: &'static str, pub other: &'a str }
pub fn fields() -> (Box<dyn Fn() -> usize>, &'static str) {
    let Parts { s, other } = Parts { s: "static", other: "other" };
    (Box::new(move || s.len()), other)
}
pub fn array() -> (Box<dyn Fn() -> usize>, &'static str) {
    let [s, other] = ["static", "other"];
    (Box::new(move || s.len()), other)
}
pub fn captured_tuple<'a>(text: &'a str) -> Box<dyn Fn() -> usize + 'a> {
    let (s, _) = (text, "static");
    Box::new(move || s.len())
}
pub fn captured_array<'a>(text: &'a str) -> Box<dyn Fn() -> usize + 'a> {
    let [s, _] = [text, "static"];
    Box::new(move || s.len())
}
pub struct Borrow<'a> { pub s: &'a str, pub other: &'static str }
pub fn captured_field<'a>(text: &'a str) -> Box<dyn Fn() -> usize + 'a> {
    let Borrow { s, .. } = Borrow { s: text, other: "static" };
    Box::new(move || s.len())
}
fn values(text: &str) -> (&'static str, &str) { ("static", text) }
pub fn unknown() -> (Box<dyn Fn() -> usize>, &'static str) {
    let (s, other) = values("other");
    (Box::new(move || s.len()), other)
}
pub fn tuple_rest() -> (Box<dyn Fn() -> usize>, &'static str) { let (_, .., s, other) = (0, 1, "static", "other"); (Box::new(move || s.len()), other) }
pub fn array_rest() -> (Box<dyn Fn() -> usize>, &'static str) { let [_, .., s, other] = ["one", "two", "static", "other"]; (Box::new(move || s.len()), other) }
pub fn projected() -> (Box<dyn Fn() -> usize>, &'static str) {
    let parts = ("static", "other");
    (Box::new(move || parts.0.len()), parts.1)
}
pub fn projection_alias() -> (Box<dyn Fn() -> usize>, &'static str) { let parts = ("static", "other"); let s = parts.0; (Box::new(move || s.len()), parts.1) }
pub fn projection_assignment() -> (Box<dyn Fn() -> usize>, &'static str) { let parts = ("static", "other"); let s; s = parts.0; (Box::new(move || s.len()), parts.1) }
pub fn projection_borrow() -> Box<dyn Fn() -> usize> { let pair = (String::from("x"),); Box::new(move || pair.0.len()) }
