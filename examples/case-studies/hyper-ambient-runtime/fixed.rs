//@ expect: tests pass
// hyper's client pool, modelled with its idle interval passed in, not reached for: the caller
// hands the pool the scope its tasks run in and the `Clock` its interval ticks on. Tide's std has
// no other spawner and no other clock, so the pool reaps idle connections under any runtime: the
// threads runtime in production, and the simulator in this file's test.
//
// The pool is buggy.rs's, with three changes: `tokio::spawn` becomes `bg.spawn`,
// `tokio::time::interval` becomes `clock.sleep`, and `tokio::time::Instant::now()` becomes
// `clock.now()`.
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::task::{Scope, TaskError, Tasks};
use std::time::{Clock, Duration, Instant};

pub enum PoolMsg {
    /// Take an idle connection, the most recently checked in, if there is one.
    Checkout(SyncSender<Option<TcpStream>>),
    /// Return a connection to the pool; it is idle from now.
    Checkin(TcpStream),
    /// The idle interval's tick: drop every connection idle for longer than the timeout.
    Expire,
}

/// A handle to the pool's task.
pub struct Pool {
    requests: SyncSender<PoolMsg>,
}

impl Pool {
    pub fn checkout(&self, tasks: &Tasks) -> Option<TcpStream> {
        let (reply, answer) = sync_channel(1, tasks);
        self.requests.send(PoolMsg::Checkout(reply)).ok()?;
        answer.recv().ok().flatten()
    }

    pub fn checkin(&self, conn: TcpStream) {
        let _ = self.requests.send(PoolMsg::Checkin(conn));
    }
}

/// Starts the pool's task and its idle interval in the caller's scope `bg`, ticking on `clock`.
/// Both run until the caller cancels `bg`.
pub fn start<'s>(
    bg: Scope<'s, '_, TaskError>,
    tasks: &Tasks,
    clock: &'s Clock,
    idle_timeout: Duration,
) -> Pool {
    let (requests, inbox) = sync_channel(16, tasks);
    bg.spawn(move || {
        run(clock, idle_timeout, inbox);
        Ok(())
    });
    let ticks = requests.clone();
    bg.spawn(move || {
        while clock.sleep(idle_timeout).is_ok() {
            if ticks.send(PoolMsg::Expire).is_err() {
                break;
            }
        }
        Ok(())
    });
    Pool { requests }
}

/// The pool's task: owns the idle connections; dropping one closes it.
fn run(clock: &Clock, idle_timeout: Duration, requests: Receiver<PoolMsg>) {
    let mut idle: Vec<(TcpStream, Instant)> = Vec::new();
    for msg in requests {
        match msg {
            PoolMsg::Checkout(reply) => {
                let _ = reply.send(idle.pop().map(|(conn, _)| conn));
            }
            PoolMsg::Checkin(conn) => idle.push((conn, clock.now())),
            PoolMsg::Expire => {
                let now = clock.now();
                idle.retain(|(_, since)| now.saturating_duration_since(*since) <= idle_timeout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::sim::SimRoot;

    /// hyper's default `pool_idle_timeout`.
    const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
    /// The interval ticks once per timeout, so a connection goes at the second tick after its
    /// check-in at the latest; one more second covers the close reaching the server.
    const CLOSED_WITHIN: Duration = Duration::from_secs(181);

    #[sim_test(seeds = 20)]
    fn idle_connections_close_after_the_timeout(sim: SimRoot) {
        let (tasks, clock, net) = (&sim.tasks, &sim.clock, &sim.net);
        let listener = net.bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (closed, eofs) = sync_channel(2, tasks);
        tasks
            .scope::<_, TaskError, _>(|s| {
                // The server: one task per connection reports when the client closed it.
                for _ in 0..2 {
                    let (listener, closed) = (&listener, closed.clone());
                    s.spawn(move || {
                        if let Ok((mut conn, _)) = listener.accept() {
                            let _ = conn.read_to_end(&mut Vec::new());
                            let _ = closed.send(clock.now());
                        }
                        Ok(())
                    });
                }
                let pool = start(s, tasks, clock, IDLE_TIMEOUT);
                let (a, b) = (net.connect(addr).unwrap(), net.connect(addr).unwrap());
                pool.checkin(a);
                let a = pool.checkout(tasks).expect("a fresh idle connection is reused");
                pool.checkin(a);
                pool.checkin(b);
                let checked_in = clock.now();
                for _ in 0..2 {
                    match eofs.recv_deadline(checked_in + CLOSED_WITHIN) {
                        Ok(at) => assert!(
                            at >= checked_in + IDLE_TIMEOUT,
                            "an idle connection closed {:?} after its check-in, before the \
                             timeout",
                            at - checked_in
                        ),
                        Err(e) => panic!(
                            "an idle connection was still open {CLOSED_WITHIN:?} after its \
                             check-in: {e:?}"
                        ),
                    }
                }
                s.cancel();
                Ok(())
            })
            .unwrap();
    }
}
