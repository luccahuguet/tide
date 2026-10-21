//@ rules: SCOPE-2
// `σ` is higher-ranked, so neither the `Scope` nor a `Task` escapes through the result.
use std::task::{TaskError, Tasks};

pub fn leak_scope(tasks: &Tasks) {
    let _s = tasks.scope(|s| Ok::<_, TaskError>(s)); //~ error
}

pub fn leak_task(tasks: &Tasks) {
    let _t = tasks.scope(|s| Ok::<_, TaskError>(s.spawn(|| Ok(1u64)))); //~ error
}
