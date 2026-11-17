//@ rules: SINK-8, LOCK-4, REF-0103
// A user macro is rejected as TD0101, but its name does not make it a stdout or lock sink.
use std::sync::{LockError, Shared};

macro_rules! println { ($($t:tt)*) => {{ let _ = format_args!($($t)*); }} } //~ TD0101

pub fn report(a: &Shared<u32>) -> Result<(), LockError> {
    a.with(|n| println!("{n}")) //~ TD0103
}
