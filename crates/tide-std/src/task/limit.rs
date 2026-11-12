use super::Signal;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, Weak};

pub(super) struct Limit {
    pub(super) id: usize,
    n: usize,
    state: Mutex<Count>,
}

#[derive(Default)]
struct Count {
    live: usize,
    queue: VecDeque<Arc<Request>>,
}

struct Request {
    // Queues own requests, so backreferences must not keep the counts alive.
    limits: Vec<Weak<Limit>>,
    signal: Arc<Signal>,
}

/// A queued request, then the child's slots. Dropping either passes capacity on.
pub(super) struct Slots {
    limits: Vec<Arc<Limit>>,
    request: Arc<Request>,
}

impl Limit {
    pub(super) fn new(id: usize, n: usize) -> Arc<Self> {
        Arc::new(Self {
            id,
            n,
            state: Mutex::new(Count::default()),
        })
    }

    #[cfg(test)]
    pub(super) fn live(&self) -> usize {
        self.state.lock().unwrap().live
    }

    #[cfg(test)]
    pub(super) fn queued(&self) -> usize {
        self.state.lock().unwrap().queue.len()
    }

    pub(super) fn is_zero(&self) -> bool {
        self.n == 0
    }
}

impl Slots {
    // The runtime lock, when present, is already held. Counts are always locked in
    // derivation order; no task parks while holding even one count lock or slot.
    pub(super) fn queue(limits: &[Arc<Limit>]) -> Self {
        let request = Arc::new(Request {
            limits: limits.iter().map(Arc::downgrade).collect(),
            signal: Arc::new(Signal::new()),
        });
        let mut counts: Vec<_> = limits
            .iter()
            .map(|limit| limit.state.lock().unwrap())
            .collect();
        for count in &mut counts {
            count.queue.push_back(request.clone());
        }
        drop(counts);
        grant(&request);
        Self {
            limits: limits.to_vec(),
            request,
        }
    }

    pub(super) fn signal(&self) -> Arc<Signal> {
        self.request.signal.clone()
    }

    pub(super) fn blocked_at(&self) -> usize {
        self.limits
            .iter()
            .find(|limit| {
                let count = limit.state.lock().unwrap();
                count.live == limit.n
                    || count
                        .queue
                        .front()
                        .is_some_and(|head| !Arc::ptr_eq(head, &self.request))
            })
            .unwrap_or(&self.limits[0])
            .id
    }
}

fn grant(request: &Arc<Request>) {
    let mut pending = VecDeque::from([request.clone()]);
    while let Some(request) = pending.pop_front() {
        let limits: Vec<_> = request.limits.iter().filter_map(Weak::upgrade).collect();
        let mut counts: Vec<_> = limits
            .iter()
            .map(|limit| limit.state.lock().unwrap())
            .collect();
        if counts.iter().zip(&limits).all(|(count, limit)| {
            count.live < limit.n
                && count
                    .queue
                    .front()
                    .is_some_and(|head| Arc::ptr_eq(head, &request))
        }) {
            for count in &mut counts {
                count.queue.pop_front();
                count.live += 1;
            }
            request.signal.wake();
            // Removing a multi-count head may make another count's head eligible too.
            pending.extend(
                counts
                    .iter()
                    .filter_map(|count| count.queue.front().cloned()),
            );
        }
    }
}

impl Drop for Slots {
    fn drop(&mut self) {
        let mut counts: Vec<_> = self
            .limits
            .iter()
            .map(|limit| limit.state.lock().unwrap())
            .collect();
        let granted = self.request.signal.ready.load(Ordering::SeqCst);
        for count in &mut counts {
            if granted {
                count.live -= 1;
            } else {
                count
                    .queue
                    .retain(|request| !Arc::ptr_eq(request, &self.request));
            }
        }
        let next: Vec<_> = counts
            .iter()
            .filter_map(|count| count.queue.front().cloned())
            .collect();
        drop(counts);
        for request in next {
            grant(&request);
        }
    }
}
