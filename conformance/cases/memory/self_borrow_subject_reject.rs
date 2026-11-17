//@ rules: REF-0210
// Only the moved owner's and its borrow field's errors receive the label.
pub struct Doc<'a> { pub word: &'a str, pub other: &'a str, pub text: String }
pub fn pushed(v: &mut Vec<Doc<'static>>) {
    let text = String::from("owner");
    let y = String::from("other");
    v.push(Doc {
        word: &text, //~ E0597 TD0210
        other: &y, //~ E0597
        text, //~ E0505 TD0210
    });
}
pub fn returned<'a>() -> Doc<'a> {
    let text = String::from("owner");
    let y = String::from("other");
    Doc { //~ E0515 TD0210
        word: &text,
        other: &y,
        text, //~ E0505 TD0210
    }
}
