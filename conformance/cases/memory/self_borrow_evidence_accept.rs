//@ rules: REF-0210
pub struct View<'a> { pub s: &'a str }
pub fn returned(t: &str) -> View<'_> { View { s: t } }
pub fn pushed(v: &mut Vec<View<'static>>) { v.push(View { s: "static" }); }
pub struct Doc<'a> { pub text: String, pub word: &'a str }
pub fn method(text: &str) -> Doc<'_> { Doc { word: text, text: text.to_string() } }
pub fn index(text: &str) -> Doc<'_> { Doc { word: &text[..], text: text.to_string() } }
pub struct MutDoc<'a> { pub text: String, pub word: &'a mut String }
pub fn mutable(text: &mut String) -> MutDoc<'_> { let owned = text.clone(); MutDoc { word: text, text: owned } }
pub struct MutStrDoc<'a> { pub text: String, pub word: &'a mut str }
pub fn mutable_method(text: &mut String) -> MutStrDoc<'_> { let owned = text.clone(); MutStrDoc { word: text.as_mut_str(), text: owned } }
pub struct Owner { pub text: String }
pub struct FieldDoc<'a> { pub owner: Owner, pub word: &'a str }
pub fn field(owner: &Owner) -> FieldDoc<'_> { FieldDoc { word: &owner.text, owner: Owner { text: owner.text.clone() } } }
pub fn field_method(owner: &Owner) -> FieldDoc<'_> { FieldDoc { word: owner.text.as_str(), owner: Owner { text: owner.text.clone() } } }
pub struct BoxDoc<'a> { pub owner: Box<String>, pub word: &'a str }
pub fn boxed(owner: &String) -> BoxDoc<'_> { BoxDoc { word: owner.as_str(), owner: Box::new(owner.clone()) } }
pub struct Copied<'a> { pub value: u32, pub view: &'a u32 }
pub fn copied(value: &u32) -> Copied<'_> { Copied { view: value, value: *value } }
pub struct Mixed { pub text: String, pub stable: &'static str }
pub struct MixedView<'a> { pub owner: Mixed, pub view: &'a [u8], pub other: &'a str }
pub fn foreign(text: &str) -> MixedView<'_> { let owner = Mixed { text: String::from("x"), stable: "static" }; MixedView { view: owner.stable.as_bytes(), other: text, owner } }
pub struct BoxedMixed { pub text: String, pub stable: Box<&'static str> }
pub struct BoxedView<'a> { pub owner: BoxedMixed, pub view: &'a str, pub other: &'a str }
pub fn boxed_foreign(text: &str) -> BoxedView<'_> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; BoxedView { view: &owner.stable[..], other: text, owner } }
pub fn coerced_foreign(text: &str) -> BoxedView<'_> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; BoxedView { view: &owner.stable, other: text, owner } }
pub struct BoxedBytes<'a> { pub owner: BoxedMixed, pub view: &'a [u8], pub other: &'a str }
pub fn boxed_method(text: &str) -> BoxedBytes<'_> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; BoxedBytes { view: owner.stable.as_bytes(), other: text, owner } }
