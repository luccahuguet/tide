//@ rules: SCOPE-1, SEND-3
// Shared task signatures; the driver also runs TASK-2/3 and JOIN-1/2 assertions below.
use std::io;
use std::task::{Cancelled, TaskError, Tasks};

fn cancellation(tasks: &Tasks) -> Result<(), TaskError> {
    tasks.checkpoint().map_err(|_| TaskError::Cancelled)?;
    tasks.yield_now().map_err(|_| TaskError::Cancelled)?;
    let value = tasks.scope(|s| -> Result<_, TaskError> {
        let ready = s.spawn(|| Ok(7));
        s.cancel();
        s.cancel();
        for _ in 0..3 {
            let e: io::Error = tasks.checkpoint().unwrap_err().into();
            assert!(e.is_cancelled());
            assert_eq!(e.kind(), io::ErrorKind::Cancelled);
            let _: Cancelled = tasks.yield_now().unwrap_err();
        }
        assert_eq!(ready.join()?, 7);
        s.spawn(|| {
            assert!(tasks.checkpoint().is_err());
            tasks.scope(|inner| {
                assert!(tasks.yield_now().is_err());
                inner.spawn(|| Ok(9)).join()
            })
        })
        .join()
    })?;
    assert_eq!(value, 9);
    tasks.checkpoint().map_err(|_| TaskError::Cancelled)?;

    let failed: Result<(), TaskError> = tasks.scope(|s| {
        let first = s.spawn(|| Err::<(), _>(TaskError::DeadlineExceeded));
        assert_eq!(first.join(), Err(TaskError::Cancelled));
        // Without observing a checkpoint this is independent, despite cancellation.
        let second = s.spawn(|| Err::<(), _>(TaskError::Cancelled));
        let echo = s.spawn(|| {
            tasks.checkpoint().map_err(|_| TaskError::Cancelled)?;
            Ok(())
        });
        assert_eq!(second.join(), Err(TaskError::Cancelled));
        assert_eq!(echo.join(), Err(TaskError::Cancelled));
        Ok(())
    });
    assert_eq!(failed, Err(TaskError::DeadlineExceeded));
    tasks.yield_now().map_err(|_| TaskError::Cancelled)?;
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), TaskError> {
    cancellation(&root.tasks)?;
    println!("CANCELLATION_RAN");
    Ok(())
}
