//@ rules: BORROW-3, SCOPE-3
// Matrix rows: shape_route. TD marks are fixed by the evidence policy.
use std::task::Tasks;
pub struct Parts<'a> { pub first: &'a str, pub other: &'static str }
pub fn whole_direct(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let _ = tasks.spawn_isolated(move || (value).len()); } //~ E0597 TD0203
pub fn whole_alias(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597 TD0203
pub fn whole_deferred(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let alias; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597 TD0203
pub fn whole_reassigned(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let mut alias = value; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn borrow_direct(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let _ = tasks.spawn_isolated(move || (&value).len()); } //~ E0597 TD0203
pub fn borrow_alias(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597 TD0203
pub fn borrow_deferred(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let alias; alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597 TD0203
pub fn borrow_reassigned(tasks: &Tasks) { let text = String::from("x"); let value = text.as_str(); let mut alias = &value; alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn field_direct(tasks: &Tasks) { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let _ = tasks.spawn_isolated(move || (parts.first).len()); } //~ E0597
pub fn field_alias(tasks: &Tasks) { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn field_deferred(tasks: &Tasks) { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let alias; alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn field_reassigned(tasks: &Tasks) { let text = String::from("x"); let parts = Parts { first: text.as_str(), other: "static" }; let mut alias = parts.first; alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn index_direct(tasks: &Tasks) { let text = String::from("x"); let parts = [text.as_str(), "static"]; let _ = tasks.spawn_isolated(move || (parts[0]).len()); } //~ E0597
pub fn index_alias(tasks: &Tasks) { let text = String::from("x"); let parts = [text.as_str(), "static"]; let alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn index_deferred(tasks: &Tasks) { let text = String::from("x"); let parts = [text.as_str(), "static"]; let alias; alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn index_reassigned(tasks: &Tasks) { let text = String::from("x"); let parts = [text.as_str(), "static"]; let mut alias = parts[0]; alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn tuple_direct(tasks: &Tasks) { let text = String::from("x"); let parts = (text.as_str(), "static"); let _ = tasks.spawn_isolated(move || (parts.0).len()); } //~ E0597
pub fn tuple_alias(tasks: &Tasks) { let text = String::from("x"); let parts = (text.as_str(), "static"); let alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn tuple_deferred(tasks: &Tasks) { let text = String::from("x"); let parts = (text.as_str(), "static"); let alias; alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn tuple_reassigned(tasks: &Tasks) { let text = String::from("x"); let parts = (text.as_str(), "static"); let mut alias = parts.0; alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn destructure_direct(tasks: &Tasks) { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let _ = tasks.spawn_isolated(move || (value).len()); } //~ E0597
pub fn destructure_alias(tasks: &Tasks) { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn destructure_deferred(tasks: &Tasks) { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let alias; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
pub fn destructure_reassigned(tasks: &Tasks) { let text = String::from("x"); let (value, _) = (text.as_str(), "static"); let mut alias = value; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0597
