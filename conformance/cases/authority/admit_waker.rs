//@ rules: ADMIT-1
// `core::task` exists only for async, so `Waker` is not admitted.
pub fn wake(_w: &core::task::Waker) {} //~ TD0520
