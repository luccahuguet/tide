use std::{
    thread,
    time::{Duration, Instant},
};
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Done,
    TimedOut,
}
pub fn run_batch(jobs: &[Duration], limit: Duration) -> Vec<Outcome> {
    let start = Instant::now();
    thread::scope(|s| {
        let hs: Vec<_> = jobs
            .iter()
            .map(|&job| {
                s.spawn(move || {
                    let finish = start + job.min(limit);
                    thread::sleep(finish.saturating_duration_since(Instant::now()));
                    if job <= limit {
                        Outcome::Done
                    } else {
                        Outcome::TimedOut
                    }
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}
