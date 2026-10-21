//@ rules: GLOBAL-6, CAP-2
// `Arc` and `sync::Weak` are created with `&Tasks`; `make_mut` and `upgrade` stay.
use std::sync::Arc;
use std::task::Tasks;

pub fn edit(tasks: &Tasks) -> Option<usize> {
    let mut a = Arc::new(vec![1u32, 2], tasks);
    Arc::make_mut(&mut a).push(3);
    let w = Arc::downgrade(&a, tasks);
    w.upgrade().map(|v| v.len())
}
