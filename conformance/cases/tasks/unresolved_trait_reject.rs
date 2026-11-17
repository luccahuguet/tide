//@ rules: CAP-1, REF-0601
// Trait resolution errors must not erase rule feedback elsewhere in the file.
pub fn ambient() { let _ = std::fs::read("x"); } //~ E0425 TD0501
pub fn bound<F: Missing>(_: F) {} //~ E0405
pub struct S;
impl std::future::Future for S {} //~ E0433 TD0601
pub fn future<F: std::future::Future>(_: F) {} //~ E0433 TD0601
pub fn dynamic(_: &dyn std::future::Future<Output = ()>) {} //~ E0433 TD0601
pub fn opaque(_: impl std::future::Future<Output = ()>) {} //~ E0433 TD0601
use std::future::Future as Absent; //~ E0432 TD0601
pub fn imported(_: impl Absent<Output = ()>) {} //~ TD0601
