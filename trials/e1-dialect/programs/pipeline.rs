// Task 5: pipeline -- src/lib.rs
//
// parser -> bounded channel -> summer, each child started by a helper on a
// scope it is given. Children of a passed-down scope borrow only what the
// helper's signature ties to the scope (`lines: &'s [String]`) and own the
// rest (the channel ends).

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::task::{Scope, Task, TaskError, Tasks};

#[derive(Debug, Error, PartialEq)]
pub enum PipeError {
    #[error("line {0} is not a decimal i64")]
    BadLine(usize),
    #[error("sum overflowed i64")]
    Overflow,
    #[error("task failed: {0}")]
    Task(#[from] TaskError),
}

/// Starts a child that parses each line as a decimal `i64` and sends it on
/// `tx`, failing with the 1-based number of the first line that does not
/// parse. The child's value is the number of values sent.
pub fn spawn_parser<'s>(s: Scope<'s, '_, PipeError>, lines: &'s [String], tx: SyncSender<i64>)
    -> Task<'s, usize, PipeError>
{
    s.spawn(move || {
        let mut sent = 0;
        for (i, line) in lines.iter().enumerate() {
            let v: i64 = line.parse().map_err(|_| PipeError::BadLine(i + 1))?;
            if tx.send(v).is_err() {
                // The receiver is gone or the scope is cancelled: either way
                // another failure is the root cause, and the scope reports it.
                break;
            }
            sent += 1;
        }
        Ok(sent)
    })
}

/// Starts a child that receives every value and returns their sum, failing
/// with `Overflow` instead of trapping.
pub fn spawn_summer<'s>(s: Scope<'s, '_, PipeError>, rx: Receiver<i64>) -> Task<'s, i64, PipeError> {
    s.spawn(move || {
        let mut sum: i64 = 0;
        loop {
            match rx.recv() {
                Ok(v) => sum = sum.checked_add(v).ok_or(PipeError::Overflow)?,
                Err(e) if e.is_cancelled() => return Err(TaskError::Cancelled.into()), // an echo
                Err(_) => break, // every sender dropped: the input is complete
            }
        }
        Ok(sum)
    })
}

/// Parses and sums `lines` through a channel of capacity 4; returns
/// `(count, sum)`.
pub fn sum_lines(tasks: &Tasks, lines: &[String]) -> Result<(usize, i64), PipeError> {
    let (tx, rx) = sync_channel(4, tasks);
    tasks.scope(|s| {
        let parser = spawn_parser(s, lines, tx);
        let summer = spawn_summer(s, rx);
        let count = parser.join()?;
        let sum = summer.join()?;
        Ok((count, sum))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    fn owned(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
    }

    #[sim_test]
    fn ten_lines(sim: SimRoot) {
        let lines: Vec<String> = (1..=10).map(|n: i64| n.to_string()).collect();
        assert_eq!(sum_lines(&sim.tasks, &lines), Ok((10, 55)));
    }

    #[sim_test]
    fn empty_input(sim: SimRoot) {
        assert_eq!(sum_lines(&sim.tasks, &[]), Ok((0, 0)));
    }

    #[sim_test]
    fn bad_line_three_is_reported(sim: SimRoot) {
        let lines = owned(&["1", "2", "three", "4", "5", "6", "7", "8"]);
        assert_eq!(sum_lines(&sim.tasks, &lines), Err(PipeError::BadLine(3)));
    }

    #[sim_test]
    fn overflowing_sum_is_an_error(sim: SimRoot) {
        let lines = owned(&["9223372036854775807", "1", "2", "3", "4", "5", "6", "7"]);
        assert_eq!(sum_lines(&sim.tasks, &lines), Err(PipeError::Overflow));
    }

    #[sim_test]
    fn negative_values(sim: SimRoot) {
        // Summed in order: MIN + MAX = -1, then -6, then -3; no partial sum overflows.
        let lines = owned(&["-9223372036854775808", "9223372036854775807", "-5", "+3"]);
        assert_eq!(sum_lines(&sim.tasks, &lines), Ok((4, -3)));
    }
}
