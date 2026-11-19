// NET-RESOLVE (research/42 §7): the one private resolver that `bind`, `connect` and `get` share.
// The system resolver (`getaddrinfo` through Rust's `ToSocketAddrs`) runs on a helper thread,
// at most POOL at once, while the calling task waits cancellably and, when the call has one,
// until its deadline. On cancellation or deadline the caller returns at once and the lookup is
// abandoned: its thread finishes on its own and its result is dropped.
use super::*;
use crate::task::Signal;
use std::time::Duration;

/// Concurrent lookups (provisional, research/42 §7).
const POOL: usize = 8;
struct Pool {
    busy: usize,
    waiting: Vec<Arc<Signal>>,
}
impl Pool {
    fn release(&mut self) {
        self.busy -= 1;
        // A woken caller can leave without claiming the slot; every live waiter must retry.
        for next in self.waiting.drain(..) {
            next.wake();
        }
    }
}
static SLOTS: Mutex<Pool> = Mutex::new(Pool {
    busy: 0,
    waiting: Vec::new(),
});

#[cfg(test)]
thread_local! {
    /// Lookups this thread started, for NET-HTTP-5's observation of zero resolver calls.
    pub(super) static CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Ends with the deadline's `TimedOut`, the scope's `Cancelled`, or the lookup's result; a
/// lookup with no address is `NotFound`.
pub(super) fn lookup(
    c: &NetContext,
    name: &str,
    port: u16,
    until: Option<Duration>,
    span: Span,
) -> io::Result<Vec<SocketAddr>> {
    let expired = || until.is_some_and(|u| crate::time::monotonic() >= u);
    loop {
        c.check()?;
        if expired() {
            return Err(ErrorKind::TimedOut.into());
        }
        let signal = {
            let mut pool = SLOTS.lock().unwrap();
            if pool.busy < POOL {
                pool.busy += 1;
                break;
            }
            let signal = Arc::new(Signal::new());
            pool.waiting.push(signal.clone());
            signal
        };
        let result = c.wait("net.resolve", 0, span, until, Some(signal.clone()));
        SLOTS.lock().unwrap().waiting.retain(|s| !Arc::ptr_eq(s, &signal));
        result?;
    }
    #[cfg(test)]
    CALLS.with(|n| n.set(n.get() + 1));
    let slot: Arc<Mutex<Option<std::io::Result<Vec<SocketAddr>>>>> = Arc::default();
    let signal = Arc::new(Signal::new());
    let (out, done, name) = (slot.clone(), signal.clone(), name.to_owned());
    let spawned = std::thread::Builder::new()
        .name("tide-resolve".into())
        .spawn(move || {
            let result = std::net::ToSocketAddrs::to_socket_addrs(&(name.as_str(), port))
                .map(|it| it.collect());
            *out.lock().unwrap() = Some(result);
            done.wake();
            SLOTS.lock().unwrap().release();
        });
    if let Err(e) = spawned {
        SLOTS.lock().unwrap().release();
        return Err(io::from_std(e));
    }
    loop {
        c.check()?;
        if expired() {
            return Err(ErrorKind::TimedOut.into());
        }
        if let Some(result) = slot.lock().unwrap().take() {
            let addrs: Vec<SocketAddr> = result.map_err(io::from_std)?;
            return if addrs.is_empty() {
                Err(ErrorKind::NotFound.into())
            } else {
                Ok(addrs)
            };
        }
        c.wait("net.resolve", 0, span, until, Some(signal.clone()))?;
        if slot.lock().unwrap().is_none() && expired() {
            return Err(ErrorKind::TimedOut.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_capacity_survives_cancelled_selected_waiter() {
        let pool = Arc::new(Mutex::new(Pool { busy: POOL, waiting: Vec::new() }));
        let (queued, ready) = std::sync::mpsc::channel();
        let live = pool.clone();
        let waiter = std::thread::spawn(move || {
            let c = Network::new(None).context();
            let signal = Arc::new(Signal::new());
            live.lock().unwrap().waiting.push(signal.clone());
            queued.send(()).unwrap();
            let until = crate::time::monotonic() + Duration::from_secs(1);
            c.wait("net.resolve", 0, std::panic::Location::caller(), Some(until), Some(signal)).unwrap();
            assert!(crate::time::monotonic() < until, "live waiter timed out with an idle slot");
            let mut pool = live.lock().unwrap();
            assert!(pool.busy < POOL);
            pool.busy += 1;
        });
        ready.recv().unwrap();
        // The last queued caller is selected first by the old policy, but has cancelled.
        pool.lock().unwrap().waiting.push(Arc::new(Signal::new()));
        let helper = pool.clone();
        std::thread::spawn(move || helper.lock().unwrap().release()).join().unwrap();
        waiter.join().unwrap();
        assert_eq!(pool.lock().unwrap().busy, POOL);
    }
}
