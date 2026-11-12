//@ rules: REF-0601
// These are FX0601 even through core or aliases; task machinery keeps FX0520.
use core::future::Future as Later; //~ FX0601
use core::pin::Pin as Pinned; //~ FX0601
pub fn future(_: impl Later<Output = ()>) {} //~ FX0601
pub fn pin(_: Pinned<&u8>) {} //~ FX0601
pub fn pending() { let _ = core::future::pending::<()>(); } //~ FX0601
pub fn waker(_: &core::task::Waker) {} //~ FX0520
