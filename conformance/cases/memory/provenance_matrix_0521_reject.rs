//@ rules: BORROW-3, SCOPE-3
// Matrix rows: shape_route. FX marks are fixed by the evidence policy.
use std::task::Tasks;
pub struct Parts<'a> { pub first: &'a str, pub other: &'static str }
pub fn whole_direct(tasks: &Tasks, text: &str) {  let value = text; let _ = tasks.spawn_isolated(move || (value).len()); } //~ E0521 FX0203
pub fn whole_alias(tasks: &Tasks, text: &str) {  let value = text; let alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521 FX0203
pub fn whole_deferred(tasks: &Tasks, text: &str) {  let value = text; let alias; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521 FX0203
pub fn whole_reassigned(tasks: &Tasks, text: &str) {  let value = text; let mut alias = value; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn borrow_direct(tasks: &Tasks, text: &str) {  let value = text; let _ = tasks.spawn_isolated(move || (&value).len()); } //~ E0521 FX0203
pub fn borrow_alias(tasks: &Tasks, text: &str) {  let value = text; let alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521 E0597 FX0203
pub fn borrow_deferred(tasks: &Tasks, text: &str) {  let value = text; let alias; alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521 E0597 FX0203
pub fn borrow_reassigned(tasks: &Tasks, text: &str) {  let value = text; let mut alias = &value; alias = &value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521 E0597
pub fn field_direct(tasks: &Tasks, text: &str) {  let parts = Parts { first: text, other: "static" }; let _ = tasks.spawn_isolated(move || (parts.first).len()); } //~ E0521
pub fn field_alias(tasks: &Tasks, text: &str) {  let parts = Parts { first: text, other: "static" }; let alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn field_deferred(tasks: &Tasks, text: &str) {  let parts = Parts { first: text, other: "static" }; let alias; alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn field_reassigned(tasks: &Tasks, text: &str) {  let parts = Parts { first: text, other: "static" }; let mut alias = parts.first; alias = parts.first; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn index_direct(tasks: &Tasks, text: &str) {  let parts = [text, "static"]; let _ = tasks.spawn_isolated(move || (parts[0]).len()); } //~ E0521
pub fn index_alias(tasks: &Tasks, text: &str) {  let parts = [text, "static"]; let alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn index_deferred(tasks: &Tasks, text: &str) {  let parts = [text, "static"]; let alias; alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn index_reassigned(tasks: &Tasks, text: &str) {  let parts = [text, "static"]; let mut alias = parts[0]; alias = parts[0]; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn tuple_direct(tasks: &Tasks, text: &str) {  let parts = (text, "static"); let _ = tasks.spawn_isolated(move || (parts.0).len()); } //~ E0521
pub fn tuple_alias(tasks: &Tasks, text: &str) {  let parts = (text, "static"); let alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn tuple_deferred(tasks: &Tasks, text: &str) {  let parts = (text, "static"); let alias; alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn tuple_reassigned(tasks: &Tasks, text: &str) {  let parts = (text, "static"); let mut alias = parts.0; alias = parts.0; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn destructure_direct(tasks: &Tasks, text: &str) {  let (value, _) = (text, "static"); let _ = tasks.spawn_isolated(move || (value).len()); } //~ E0521
pub fn destructure_alias(tasks: &Tasks, text: &str) {  let (value, _) = (text, "static"); let alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn destructure_deferred(tasks: &Tasks, text: &str) {  let (value, _) = (text, "static"); let alias; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
pub fn destructure_reassigned(tasks: &Tasks, text: &str) {  let (value, _) = (text, "static"); let mut alias = value; alias = value; let _ = tasks.spawn_isolated(move || (alias).len()); } //~ E0521
