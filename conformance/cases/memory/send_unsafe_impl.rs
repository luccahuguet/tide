//@ rules: SEND-2
// A case has no manifest, so its package holds no `unsafe` grant: an
// `unsafe impl Send` or `Sync` is the checker's error, though rustc accepts it.
use std::rc::Rc;

pub struct Handle {
    pub inner: Rc<u8>,
}
// SAFETY: present, so the missing grant is the only error.
unsafe impl Send for Handle {} //~ FX0901
// SAFETY: present, so the missing grant is the only error.
unsafe impl Sync for Handle {} //~ FX0901
