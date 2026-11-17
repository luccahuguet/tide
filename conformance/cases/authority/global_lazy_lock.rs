//@ rules: GLOBAL-2, ADMIT-1
// No lazy statics: Rust's `LazyLock` and `OnceLock` are reported as not admitted, not as
// unresolved paths (`OnceCell` stays).
use std::sync::LazyLock; //~ E0432 TD0520
use std::sync::OnceLock; //~ E0432 TD0520
