//@ rules: BORROW-3, SCOPE-3
// Matrix rows: shape_route. FX marks are fixed by the evidence policy.
pub struct Parts<'a> { pub first: &'a str, pub other: &'static str }
pub fn whole_direct() -> Box<dyn Fn() -> usize> {  let value = "static"; Box::new(|| (&(value)).len()) } //~ E0373 FX0203
pub fn whole_alias() -> Box<dyn Fn() -> usize> {  let value = "static"; let alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn whole_deferred() -> Box<dyn Fn() -> usize> {  let value = "static"; let alias; alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn whole_reassigned() -> Box<dyn Fn() -> usize> {  let value = "static"; let mut alias = value; alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn borrow_direct() -> Box<dyn Fn() -> usize> {  let value = "static"; Box::new(|| (&(&value)).len()) } //~ E0373 FX0203
pub fn borrow_alias() -> Box<dyn Fn() -> usize> {  let value = "static"; let alias = &value; Box::new(|| (&(alias)).len()) } //~ E0373 E0515 FX0203
pub fn borrow_deferred() -> Box<dyn Fn() -> usize> {  let value = "static"; let alias; alias = &value; Box::new(|| (&(alias)).len()) } //~ E0373 E0515 FX0203
pub fn borrow_reassigned() -> Box<dyn Fn() -> usize> {  let value = "static"; let mut alias = &value; alias = &value; Box::new(|| (&(alias)).len()) } //~ E0373 E0515 FX0203
pub fn field_direct() -> Box<dyn Fn() -> usize> {  let parts = Parts { first: "static", other: "static" }; Box::new(|| (&(parts.first)).len()) } //~ E0373 FX0203
pub fn field_alias() -> Box<dyn Fn() -> usize> {  let parts = Parts { first: "static", other: "static" }; let alias = parts.first; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn field_deferred() -> Box<dyn Fn() -> usize> {  let parts = Parts { first: "static", other: "static" }; let alias; alias = parts.first; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn field_reassigned() -> Box<dyn Fn() -> usize> {  let parts = Parts { first: "static", other: "static" }; let mut alias = parts.first; alias = parts.first; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn index_direct() -> Box<dyn Fn() -> usize> {  let parts = ["static", "static"]; Box::new(|| (&(parts[0])).len()) } //~ E0373 FX0203
pub fn index_alias() -> Box<dyn Fn() -> usize> {  let parts = ["static", "static"]; let alias = parts[0]; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn index_deferred() -> Box<dyn Fn() -> usize> {  let parts = ["static", "static"]; let alias; alias = parts[0]; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn index_reassigned() -> Box<dyn Fn() -> usize> {  let parts = ["static", "static"]; let mut alias = parts[0]; alias = parts[0]; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn tuple_direct() -> Box<dyn Fn() -> usize> {  let parts = ("static", "static"); Box::new(|| (&(parts.0)).len()) } //~ E0373 FX0203
pub fn tuple_alias() -> Box<dyn Fn() -> usize> {  let parts = ("static", "static"); let alias = parts.0; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn tuple_deferred() -> Box<dyn Fn() -> usize> {  let parts = ("static", "static"); let alias; alias = parts.0; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn tuple_reassigned() -> Box<dyn Fn() -> usize> {  let parts = ("static", "static"); let mut alias = parts.0; alias = parts.0; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn destructure_direct() -> Box<dyn Fn() -> usize> {  let (value, _) = ("static", "static"); Box::new(|| (&(value)).len()) } //~ E0373 FX0203
pub fn destructure_alias() -> Box<dyn Fn() -> usize> {  let (value, _) = ("static", "static"); let alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn destructure_deferred() -> Box<dyn Fn() -> usize> {  let (value, _) = ("static", "static"); let alias; alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
pub fn destructure_reassigned() -> Box<dyn Fn() -> usize> {  let (value, _) = ("static", "static"); let mut alias = value; alias = value; Box::new(|| (&(alias)).len()) } //~ E0373 FX0203
