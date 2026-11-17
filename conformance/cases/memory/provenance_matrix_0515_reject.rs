//@ rules: BORROW-3, SCOPE-3
// Matrix rows: shape_route. TD marks are fixed by the evidence policy.
pub struct Parts<'a> { pub first: &'a str, pub other: &'static str }
pub fn whole_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); Box::new(move || (value).len()) } //~ E0515 TD0203
pub fn whole_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let alias = value; Box::new(move || (alias).len()) } //~ E0515 TD0203
pub fn whole_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let alias; alias = value; Box::new(move || (alias).len()) } //~ E0515 TD0203
pub fn whole_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let mut alias = value; alias = value; Box::new(move || (alias).len()) } //~ E0515
pub fn borrow_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); Box::new(move || (&value).len()) } //~ E0515 TD0203
pub fn borrow_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let alias = &value; Box::new(move || (alias).len()) } //~ E0515 TD0203
pub fn borrow_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let alias; alias = &value; Box::new(move || (alias).len()) } //~ E0515 TD0203
pub fn borrow_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let value = text.as_str(); let mut alias = &value; alias = &value; Box::new(move || (alias).len()) } //~ E0515
pub fn field_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; Box::new(move || (parts.first).len()) } //~ E0515
pub fn field_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let alias = parts.first; Box::new(move || (alias).len()) } //~ E0515
pub fn field_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let alias; alias = parts.first; Box::new(move || (alias).len()) } //~ E0515
pub fn field_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let mut alias = parts.first; alias = parts.first; Box::new(move || (alias).len()) } //~ E0515
pub fn index_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = [text.as_str(), "static"]; Box::new(move || (parts[0]).len()) } //~ E0515
pub fn index_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = [text.as_str(), "static"]; let alias = parts[0]; Box::new(move || (alias).len()) } //~ E0515
pub fn index_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = [text.as_str(), "static"]; let alias; alias = parts[0]; Box::new(move || (alias).len()) } //~ E0515
pub fn index_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = [text.as_str(), "static"]; let mut alias = parts[0]; alias = parts[0]; Box::new(move || (alias).len()) } //~ E0515
pub fn tuple_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = (text.as_str(), "static"); Box::new(move || (parts.0).len()) } //~ E0515
pub fn tuple_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = (text.as_str(), "static"); let alias = parts.0; Box::new(move || (alias).len()) } //~ E0515
pub fn tuple_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = (text.as_str(), "static"); let alias; alias = parts.0; Box::new(move || (alias).len()) } //~ E0515
pub fn tuple_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let parts = (text.as_str(), "static"); let mut alias = parts.0; alias = parts.0; Box::new(move || (alias).len()) } //~ E0515
pub fn destructure_direct() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); Box::new(move || (value).len()) } //~ E0515
pub fn destructure_alias() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let alias = value; Box::new(move || (alias).len()) } //~ E0515
pub fn destructure_deferred() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let alias; alias = value; Box::new(move || (alias).len()) } //~ E0515
pub fn destructure_reassigned() -> Box<dyn Fn() -> usize> { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let mut alias = value; alias = value; Box::new(move || (alias).len()) } //~ E0515
