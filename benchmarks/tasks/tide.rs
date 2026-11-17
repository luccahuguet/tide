// Benchmark pair `tasks`, Tide arm: structured tasks with `Shared` and a channel. Four workers
// split the numbers 1..=count and compute each one's Collatz step count. After every block of
// 16 numbers, a worker adds the block's totals to a `Shared` cell and sends the block's results
// through a channel, so the synchronization grows with `count`. The receiver folds the blocks
// into per-worker summaries. Output depends only on `count`, whatever order blocks arrive in.
// Usage: `tasks [count]`; the default count is the small size the equivalence check runs.
// Its test runs `run` under the simulator on several seeds; the equivalence check compares
// what it prints there with the Rust arm's output.
use std::env::Env;
use std::sync::{LockError, Shared, mpsc};
use std::task::{TaskError, Tasks};

const DEFAULT_COUNT: u64 = 2_000;
const WORKERS: u64 = 4;
const BLOCK: u64 = 16;

#[derive(Debug)]
pub enum RunError {
    Size(String),
    Disconnected,
    Lock(LockError),
    Task(TaskError),
}

impl From<LockError> for RunError {
    fn from(e: LockError) -> Self {
        RunError::Lock(e)
    }
}

impl From<TaskError> for RunError {
    fn from(e: TaskError) -> Self {
        RunError::Task(e)
    }
}

#[derive(Debug, Default)]
pub struct Totals {
    numbers: u64,
    steps: u64,
}

/// One block's results from worker `worker`, or a worker's summary of all its blocks:
/// how many blocks and numbers, and the number with the most steps, `(0, 0)` for none.
#[derive(Debug)]
pub struct Results {
    worker: u64,
    blocks: u64,
    numbers: u64,
    longest: (u64, u64),
}

impl Results {
    pub fn new(worker: u64) -> Self {
        Results {
            worker,
            blocks: 0,
            numbers: 0,
            longest: (0, 0),
        }
    }

    /// Adds `(n, steps)`, keeping the most steps and, among equals, the smallest number, so
    /// the result does not depend on the order of additions.
    pub fn longest(&mut self, (n, steps): (u64, u64)) {
        if steps > self.longest.1 || (steps == self.longest.1 && n < self.longest.0) {
            self.longest = (n, steps);
        }
    }
}

/// The Collatz step count of `n` (at least 1).
pub fn steps(n: u64) -> u64 {
    let (mut n, mut steps) = (n, 0);
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

pub fn count(env: &Env) -> Result<u64, RunError> {
    match env.args().nth(1) {
        Some(arg) => arg.parse().map_err(|_| RunError::Size(arg)),
        None => Ok(DEFAULT_COUNT),
    }
}

pub fn run(env: &Env, tasks: &Tasks) -> Result<(), RunError> {
    let count = count(env)?;
    let totals = Shared::new(Totals::default(), tasks);
    let (tx, rx) = mpsc::channel(tasks);
    let summaries = tasks.scope(|s| {
        for worker in 0..WORKERS {
            let (tx, totals) = (tx.clone(), &totals);
            s.spawn(move || {
                let mut n = worker + 1;
                while n <= count {
                    let (mut block, mut steps_sum) = (Results::new(worker), 0);
                    while n <= count && block.numbers < BLOCK {
                        let k = steps(n);
                        block.numbers += 1;
                        steps_sum += k;
                        block.longest((n, k));
                        n += WORKERS;
                    }
                    let numbers = block.numbers;
                    totals.with(|t| {
                        t.numbers += numbers;
                        t.steps += steps_sum;
                    })?;
                    tx.send(block).map_err(|_| RunError::Disconnected)?;
                }
                Ok(())
            });
        }
        drop(tx);
        let mut summaries: Vec<Results> = (0..WORKERS).map(Results::new).collect();
        for block in rx.iter() {
            let summary = &mut summaries[usize::try_from(block.worker).unwrap()];
            summary.blocks += 1;
            summary.numbers += block.numbers;
            summary.longest(block.longest);
        }
        Ok::<_, RunError>(summaries)
    })?;
    let (numbers, steps) = totals.with(|t| (t.numbers, t.steps))?;
    println!("numbers: {numbers}");
    println!("steps: {steps}");
    for s in &summaries {
        println!(
            "worker {}: {} numbers in {} blocks, longest {} ({} steps)",
            s.worker, s.numbers, s.blocks, s.longest.0, s.longest.1
        );
    }
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), RunError> {
    run(&root.env, &root.tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::Sink;

    /// Runs `run` under the simulator on eight seeds, which pick eight interleavings, and
    /// requires the same standard output from each. It then writes that output to standard
    /// error, outside the simulation, for the equivalence check to compare with the Rust arm.
    #[test]
    fn simulated() {
        let mut outputs = Vec::new();
        for seed in 0..8 {
            let sim = Root::sim(seed);
            sim.world.args(["tasks"]);
            run(&sim.env, &sim.tasks).unwrap();
            let out: String = sim
                .records()
                .into_iter()
                .filter(|r| r.sink == Sink::Out)
                .map(|r| r.message + "\n")
                .collect();
            outputs.push(out);
        }
        assert!(outputs.iter().all(|o| *o == outputs[0]), "{outputs:#?}");
        eprint!("{}", outputs[0]);
    }
}
