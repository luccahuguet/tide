//@ rules: LOCK-1, LOCK-3, LOCK-4, LOCK-5, SEND-3
// Scoped workers use cloned handles and pure callbacks to update one cell.
use std::sync::Shared;
use std::task::TaskError;

struct Counter(u64);

#[tide::main]
fn main(root: Root) -> Result<(), TaskError> {
    let counter = Shared::new(Counter(0), &root.tasks);
    root.tasks.scope::<_, TaskError, _>(|scope| {
        scope.cancel();
        for _ in 0..4 {
            let counter = counter.clone();
            drop(scope.spawn(move || {
                let step = 2;
                for _ in 0..100 {
                    counter.with(|n| n.0 += step).unwrap();
                }
                Ok(())
            }));
        }
        Ok(())
    })?;
    assert_eq!(counter.with(|n| n.0).unwrap(), 800);
    Ok(())
}
