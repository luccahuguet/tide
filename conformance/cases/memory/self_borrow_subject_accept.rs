//@ rules: REF-0210
pub struct Doc<'a> { pub word: &'a str, pub other: &'a str, pub text: String }
pub fn pushed(v: &mut Vec<Doc<'static>>) {
    v.push(Doc { word: "owner", other: "other", text: String::from("owner") });
}
pub fn returned<'a>(text: &'a str, y: &'a str) -> Doc<'a> {
    Doc { word: text, other: y, text: text.to_string() }
}
