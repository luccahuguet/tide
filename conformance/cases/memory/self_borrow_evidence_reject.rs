//@ rules: REF-0210
// Returning or storing a view is not a self-borrow when no owner moves into it.
pub struct View<'a> { pub s: &'a str }
pub fn returned<'a>() -> View<'a> { let t = String::from("x"); View { s: &t } } //~ E0515
pub fn pushed(v: &mut Vec<View<'static>>) { let t = String::from("x"); v.push(View { s: &t }); } //~ E0597

pub struct Doc<'a> { pub text: String, pub word: &'a str }
pub fn method<'a>() -> Doc<'a> { let text = String::from("x"); Doc { word: text.as_str(), text } } //~ E0505 E0515 FX0210
pub fn index<'a>() -> Doc<'a> { let text = String::from("x"); Doc { word: &text[..], text } } //~ E0505 E0515 FX0210
pub struct MutDoc<'a> { pub text: String, pub word: &'a mut String }
pub fn mutable<'a>() -> MutDoc<'a> { let mut text = String::from("x"); MutDoc { word: &mut text, text } } //~ E0505 E0515 FX0210
pub struct MutStrDoc<'a> { pub text: String, pub word: &'a mut str }
pub fn mutable_method<'a>() -> MutStrDoc<'a> { let mut text = String::from("x"); MutStrDoc { word: text.as_mut_str(), text } } //~ E0505 E0515 FX0210
pub struct Owner { pub text: String }
pub struct FieldDoc<'a> { pub owner: Owner, pub word: &'a str }
pub fn field<'a>() -> FieldDoc<'a> { let owner = Owner { text: String::from("x") }; FieldDoc { word: &owner.text, owner } } //~ E0505 E0515 FX0210
pub fn field_method<'a>() -> FieldDoc<'a> { let owner = Owner { text: String::from("x") }; FieldDoc { word: owner.text.as_str(), owner } } //~ E0505 E0515 FX0210
pub struct BoxDoc<'a> { pub owner: Box<String>, pub word: &'a str }
pub fn boxed<'a>() -> BoxDoc<'a> { let owner = Box::new(String::from("x")); BoxDoc { word: &*owner, owner } } //~ E0505 E0515 FX0210

// A Copy value is not moved, and a borrowed foreign field is not the moved owner's data.
pub struct Copied<'a> { pub value: u32, pub view: &'a u32 }
pub fn copied<'a>() -> Copied<'a> { let value = 1; Copied { view: &value, value } } //~ E0515
pub struct Mixed { pub text: String, pub stable: &'static str }
pub struct MixedView<'a> { pub owner: Mixed, pub view: &'a [u8], pub other: &'a str }
pub fn foreign<'a>() -> MixedView<'a> { let owner = Mixed { text: String::from("x"), stable: "static" }; let text = String::from("other"); MixedView { view: owner.stable.as_bytes(), other: &text, owner } } //~ E0515
pub struct BoxedMixed { pub text: String, pub stable: Box<&'static str> }
pub struct BoxedView<'a> { pub owner: BoxedMixed, pub view: &'a str, pub other: &'a str }
pub fn boxed_foreign<'a>() -> BoxedView<'a> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; let text = String::from("other"); BoxedView { view: &owner.stable[..], other: &text, owner } } //~ E0515
pub fn coerced_foreign<'a>() -> BoxedView<'a> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; let text = String::from("other"); BoxedView { view: &owner.stable, other: &text, owner } } //~ E0515
pub struct BoxedBytes<'a> { pub owner: BoxedMixed, pub view: &'a [u8], pub other: &'a str }
pub fn boxed_method<'a>() -> BoxedBytes<'a> { let owner = BoxedMixed { text: String::from("x"), stable: Box::new("static") }; let text = String::from("other"); BoxedBytes { view: owner.stable.as_bytes(), other: &text, owner } } //~ E0515
