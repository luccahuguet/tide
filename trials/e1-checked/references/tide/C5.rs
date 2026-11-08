use std::{
    task::{TaskError, Tasks},
    time::{Clock, Duration},
};
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Done,
    TimedOut,
}
pub fn run_batch(tasks: &Tasks, clock: &Clock, jobs: &[Duration], limit: Duration) -> Vec<Outcome> {
    tasks
        .scope(|s| -> Result<_, TaskError> {
            s.set_deadline(clock.now() + limit);
            let hs: Vec<_> = jobs
                .iter()
                .map(|&job| {
                    s.spawn(move || {
                        Ok(if clock.sleep(job).is_ok() {
                            Outcome::Done
                        } else {
                            Outcome::TimedOut
                        })
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join()).collect()
        })
        .unwrap()
}
