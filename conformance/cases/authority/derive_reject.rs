//@ rules: REF-0102
// Derives outside the closed set are errors, even when a caller tries to allow lints.
#![allow(warnings)]
#[derive(Serialize)] //~ error TD0102
pub struct Wire { pub value: u8 }

#[derive(thiserror::Error)] //~ E0433 TD0102
pub struct Failure;
