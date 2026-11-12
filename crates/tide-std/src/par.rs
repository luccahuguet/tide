//! Deterministic parallel regions (03 §8). The prelude supplies [`ParSlice`].
//!
//! Scoped helpers share a process-wide core budget and are not tasks. This is a partial
//! scheduler: the work-stealing pool and `par-threads` setting remain planned. Simulation
//! runs the same algorithms inline in sequential order, without scheduling points.
//! Sink buffers are unbounded; losing branches' host panic-hook output is not suppressed.
//! Stop flags are per call; propagation across calls is partial. `join` and nested regions
//! run an already spawned right branch to completion after a left panic. A right branch
//! that never terminates diverges, and
//! whether the region panics or diverges can depend on machine load. Every fork finding
//! a free token spawns an OS thread, so fine-grained recursive `join` is slow until the
//! planned pool. Helper panic metadata is not returned to the property runner: on the
//! thread runtime, scheduling can turn a `requires` discard into a failure.

use crate::marker::Pure;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;

struct Budget {
    workers: usize,
    free: AtomicUsize,
}
impl Budget {
    fn new(workers: usize) -> Self {
        Self {
            workers,
            free: AtomicUsize::new(workers - 1),
        }
    }
    fn take(&self) -> Option<Token<'_>> {
        self.free
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
            .ok()
            .map(|_| Token(self))
    }
}
struct Token<'a>(&'a Budget);
impl Drop for Token<'_> {
    fn drop(&mut self) {
        self.0.free.fetch_add(1, Ordering::Relaxed);
    }
}
fn budget() -> &'static Budget {
    static BUDGET: OnceLock<Budget> = OnceLock::new();
    BUDGET.get_or_init(|| Budget::new(thread::available_parallelism().map_or(1, |n| n.get())))
}

/// Evaluate two authority-free closures; the leftmost panic propagates in the caller.
pub fn join<A, B, RA: Send, RB: Send>(a: A, b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Pure + Send,
    B: FnOnce() -> RB + Pure + Send,
{
    fork(budget(), a, b)
}

// The only OS-thread fork. No helper inherits CURRENT or consumes a Tasks::limit slot.
fn fork<RA: Send, RB: Send>(
    budget: &Budget,
    a: impl FnOnce() -> RA + Send,
    b: impl FnOnce() -> RB + Send,
) -> (RA, RB) {
    let token = if crate::task::is_simulating() {
        None
    } else {
        budget.take()
    };
    let Some(token) = token else {
        return (a(), b());
    };
    // On spawn failure the closure is still here, so resource pressure falls back inline.
    let right = Mutex::new(Some(b));
    thread::scope(|scope| {
        let name = thread::current().name().unwrap_or("par").to_owned();
        let helper = thread::Builder::new().name(name).spawn_scoped(scope, || {
            let _token = token;
            let b = right.lock().unwrap().take().unwrap();
            crate::__sink::capture(b)
        });
        let Ok(helper) = helper else {
            return (a(), right.lock().unwrap().take().unwrap()());
        };
        let (left, records) = crate::__sink::capture(a);
        let (right, right_records) = helper.join().unwrap_or_else(|p| resume_unwind(p));
        crate::__sink::append(records);
        match left {
            Err(p) => {
                // Surviving values drop during the winner's unwind (PANIC-2).
                resume_unwind(p)
            }
            Ok(left) => {
                crate::__sink::append(right_records);
                (left, right.unwrap_or_else(|p| resume_unwind(p)))
            }
        }
    })
}

/// Slice operations available without an import in Tide programs.
pub trait ParSlice<T> {
    fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
    where
        T: Pure + Sync;
    fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync)
    where
        T: Pure + Send;
    /// Stable sort by `Ord`, with a split/merge algorithm independent of worker count.
    fn par_sort(&mut self)
    where
        T: Ord + Pure + Send;
}
impl<T> ParSlice<T> for [T] {
    fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
    where
        T: Pure + Sync,
    {
        map(self, &f, budget())
    }
    fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync)
    where
        T: Pure + Send,
    {
        for_each(self, &f, budget());
    }
    fn par_sort(&mut self)
    where
        T: Ord + Pure + Send,
    {
        sort(self, budget());
    }
}

fn block_size(len: usize, budget: &Budget) -> usize {
    (len / budget.workers.saturating_mul(8)).max(1)
}
fn map<T: Sync, U: Send>(xs: &[T], f: &(impl Fn(&T) -> U + Sync), budget: &Budget) -> Vec<U> {
    blocks(
        xs.chunks(block_size(xs.len(), budget)),
        xs.len(),
        budget,
        &|chunk: &[T]| chunk.iter().map(f).collect::<Vec<_>>(),
    )
    .into_iter()
    .flatten()
    .collect()
}
fn for_each<T: Send>(xs: &mut [T], f: &(impl Fn(&mut T) + Sync), budget: &Budget) {
    let len = xs.len();
    blocks(
        xs.chunks_mut(block_size(len, budget)),
        len,
        budget,
        &|chunk: &mut [T]| chunk.iter_mut().for_each(f),
    );
}

// Claim under the lock, execute without it. A panic stops only unclaimed (later) blocks.
fn blocks<I, R, F>(input: I, len: usize, budget: &Budget, f: &F) -> Vec<R>
where
    I: Iterator + Send,
    I::Item: Send,
    R: Send,
    F: Fn(I::Item) -> R + Sync,
{
    let input = Mutex::new(input.enumerate());
    let stopped = AtomicBool::new(false);
    let worker = || {
        let mut completed = Vec::new();
        loop {
            let next = {
                let mut input = input.lock().unwrap();
                if stopped.load(Ordering::Acquire) {
                    None
                } else {
                    input.next()
                }
            };
            let Some((index, block)) = next else {
                break;
            };
            let (result, records) = crate::__sink::capture(|| f(block));
            if result.is_err() {
                stopped.store(true, Ordering::Release);
            }
            completed.push((index, result, records));
        }
        completed
    };
    fn workers<R: Send>(n: usize, budget: &Budget, f: &(impl Fn() -> Vec<R> + Sync)) -> Vec<R> {
        if n <= 1 {
            return f();
        }
        let (mut left, right) = fork(
            budget,
            || workers(n / 2, budget, f),
            || workers(n - n / 2, budget, f),
        );
        left.extend(right);
        left
    }
    let n = if crate::task::is_simulating() {
        1
    } else {
        budget.workers.min(len)
    };
    let mut completed = workers(n, budget, &worker);
    completed.sort_unstable_by_key(|(index, ..)| *index);
    let mut values = Vec::new();
    let mut completed = completed.into_iter();
    while let Some((_, result, records)) = completed.next() {
        crate::__sink::append(records);
        match result {
            Ok(value) => values.push(value),
            Err(payload) => {
                // Both earlier values and later outcomes drop during the winner's unwind.
                resume_unwind(payload);
            }
        }
    }
    values
}

fn sort<T: Ord + Send>(xs: &mut [T], budget: &Budget) {
    sort_range(xs, budget, &AtomicUsize::new(usize::MAX), 0);
}

fn sort_range<T: Ord + Send>(xs: &mut [T], budget: &Budget, stop: &AtomicUsize, start: usize) {
    // One-element leaves avoid importing a host sort's changing comparison algorithm.
    if xs.len() <= 1 || start > stop.load(Ordering::Acquire) {
        return;
    }
    let len = xs.len();
    let mid = len / 2;
    let (left, right) = xs.split_at_mut(mid);
    if len >= 4096 {
        fork(
            budget,
            || sort_range(left, budget, stop, start),
            || sort_range(right, budget, stop, start + mid),
        );
    } else {
        sort_range(left, budget, stop, start);
        sort_range(right, budget, stop, start + mid);
    }
    if start > stop.load(Ordering::Acquire) {
        return;
    }
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| merge(xs, mid))) {
        stop.fetch_min(start, Ordering::Release);
        resume_unwind(payload);
    }
}

fn merge<T: Ord>(xs: &mut [T], mid: usize) {
    let mut order = Vec::with_capacity(xs.len());
    let (mut left, mut right) = (0, mid);
    while left < mid && right < xs.len() {
        if xs[left] <= xs[right] {
            order.push(left);
            left += 1;
        } else {
            order.push(right);
            right += 1;
        }
    }
    order.extend(left..mid);
    order.extend(right..xs.len());
    // order[destination] is the old source. Invert, then put each source at its destination.
    let mut destinations = vec![0; xs.len()];
    for (destination, source) in order.into_iter().enumerate() {
        destinations[source] = destination;
    }
    for i in 0..xs.len() {
        while destinations[i] != i {
            let destination = destinations[i];
            xs.swap(i, destination);
            destinations.swap(i, destination);
        }
    }
}

#[cfg(test)]
mod tests;
