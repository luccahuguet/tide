//@ rules: SCOPE-1
// Children borrow places that outlive the `scope` call, one mutably; the
// parent reads the shared one meanwhile and mutates both once the scope returns.
use std::task::{TaskError, Tasks};

pub fn run(tasks: &Tasks) -> Result<u64, TaskError> {
    let mut total = 0u64;
    let mut items = vec![1u64, 2, 3];
    let sum = tasks.scope::<_, TaskError, _>(|s| {
        let a = s.spawn(|| Ok(items.iter().sum::<u64>()));
        let b = s.spawn(|| {
            total += 1;
            Ok(())
        });
        let len = items.len() as u64;
        b.join()?;
        Ok(a.join()? + len)
    })?;
    items.push(4);
    total += sum;
    Ok(total)
}
