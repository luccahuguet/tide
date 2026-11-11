//@ rules: REF-0210
// Keep the owner separate, or retain indices alongside the owned buffer.
pub struct View<'a> { pub word: &'a str }
pub struct Doc { pub text: String, pub word: std::ops::Range<usize> }
pub fn view(text: &str) -> View<'_> { View { word: text } }
pub fn make() -> Doc { Doc { text: String::from("word"), word: 0..4 } }
