//@ rules: SEND-3
// A child captures `Send` values and borrows of `Sync` data, returns a `Send`
// result, and sends a `Send` message.
use std::sync::mpsc::sync_channel;
use std::task::{TaskError, Tasks};

pub fn run(tasks: &Tasks, total: &u64) -> Result<u64, TaskError> {
    let name = String::from("job");
    let (tx, rx) = sync_channel::<String>(1, tasks);
    tasks.scope(|s| {
        let job = s.spawn(move || Ok(name.len() as u64 + *total));
        let _ = tx.send(String::from("done"));
        let _ = rx.recv();
        job.join()
    })
}
