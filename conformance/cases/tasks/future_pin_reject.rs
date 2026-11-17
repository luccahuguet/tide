//@ rules: REF-0601
// These are TD0601 even through core or aliases; task machinery keeps TD0520.
use core::future::Future as Later; //~ TD0601
use core::pin::Pin as Pinned; //~ TD0601
pub fn future(_: impl Later<Output = ()>) {} //~ TD0601
pub fn pin(_: Pinned<&u8>) {} //~ TD0601
pub fn pending() { let _ = core::future::pending::<()>(); } //~ TD0601
pub fn waker(_: &core::task::Waker) {} //~ TD0520
