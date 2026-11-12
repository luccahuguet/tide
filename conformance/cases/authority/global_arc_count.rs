//@ rules: GLOBAL-6
// `Arc` has no count observers, whose results depend on other tasks.
use std::sync::Arc;

pub fn owners(a: &Arc<u32>) -> usize {
    Arc::strong_count(a) //~ E0599 FX0520
}
