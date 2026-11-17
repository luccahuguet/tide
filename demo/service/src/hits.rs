//! A page-hit counter service, the fixed revision.
//!
//! `serve` answers a batch of traffic with one worker task per client. Each worker records a hit
//! for every page view in one shared `Stats` cell, and `serve` returns the hits recorded, which
//! must equal the views served.
use std::sync::{LockError, Shared};
use std::task::{TaskError, Tasks};

/// A batch of traffic: `clients` concurrent clients, each sending `views` page views.
pub struct Load {
    pub clients: u32,
    pub views: u32,
}

/// Why `serve` failed.
#[derive(Debug)]
pub enum ServeError {
    Task(TaskError),
    Lock(LockError),
}

impl From<TaskError> for ServeError {
    fn from(error: TaskError) -> Self {
        Self::Task(error)
    }
}

impl From<LockError> for ServeError {
    fn from(error: LockError) -> Self {
        Self::Lock(error)
    }
}

struct Stats {
    hits: u64,
}

/// Serves `load` with one worker task per client and returns the hits recorded.
pub fn serve(tasks: &Tasks, load: &Load) -> Result<u64, ServeError> {
    let stats = Shared::new(Stats { hits: 0 }, tasks);
    tasks.scope::<_, ServeError, _>(|scope| {
        for _ in 0..load.clients {
            scope.spawn(|| {
                for _ in 0..load.views {
                    record_hit(&stats)?;
                }
                Ok(())
            });
        }
        Ok(())
    })?;
    Ok(stats.with(|s| s.hits)?)
}

/// Records one page view: the read and the write happen in one critical section.
fn record_hit(stats: &Shared<Stats>) -> Result<(), LockError> {
    stats.with(|s| s.hits += 1)
}
