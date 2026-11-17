//@ rules: REF-0210
// The type is legal; constructing a view into its own owner fails in rustc.
pub struct Doc<'a> { pub text: String, pub word: &'a str }
pub fn make<'a>() -> Doc<'a> {
    let text = String::from("word");
    Doc { word: &text, text } //~ E0505 E0515 TD0210
}
