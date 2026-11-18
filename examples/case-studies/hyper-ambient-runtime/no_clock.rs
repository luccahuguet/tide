//@ expect: test fails idle_connections_close_after_the_timeout
//@ reports: an idle connection was still open 181s after its check-in: Timeout
//@ reports: 0 passed; 20 failed (20 seeds)
// fixed.rs's pool with its clock made optional, as hyper-util's `pool_timer` is, and the test
// building it without one. Tide admits this pool: it reaches for nothing it was not handed. But
// with no clock it has no interval and cannot tell when a connection went idle (hyper-util
// falls back to Rust's ambient `Instant::now()`, which Tide rejects), so it never reaps, and
// fixed.rs's test fails. Tide's rule is that a pool cannot reach a clock it was not handed, not
// that every pool reaps.
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

/// Starts the pool's task in the caller's scope `bg` and, given a `clock`, its idle interval.
/// Both run until the caller cancels `bg`.
pub fn start<'s>(
    bg: Scope<'s, '_, TaskError>,
    tasks: &Tasks,
    clock: Option<&'s Clock>,
    idle_timeout: Duration,
) -> Pool {
    let (requests, inbox) = sync_channel(16, tasks);
    bg.spawn(move || {
        run(clock, idle_timeout, inbox);
        Ok(())
    });
    if let Some(clock) = clock {
        let ticks = requests.clone();
        bg.spawn(move || {
            while clock.sleep(idle_timeout).is_ok() {
                if ticks.send(PoolMsg::Expire).is_err() {
                    break;
                }
            }
            Ok(())
        });
    }
    Pool { requests }
}

/// The pool's task: owns the idle connections; dropping one closes it. Without a clock a
/// connection has no idle time, and no `Expire` comes.
fn run(clock: Option<&Clock>, idle_timeout: Duration, requests: Receiver<PoolMsg>) {
    let mut idle: Vec<(TcpStream, Option<Instant>)> = Vec::new();
    for msg in requests {
        match msg {
            PoolMsg::Checkout(reply) => {
                let _ = reply.send(idle.pop().map(|(conn, _)| conn));
            }
            PoolMsg::Checkin(conn) => idle.push((conn, clock.map(Clock::now))),
            PoolMsg::Expire => {
                let now = clock.map(Clock::now);
                idle.retain(|(_, since)| match (now, since) {
                    (Some(now), Some(since)) => {
                        now.saturating_duration_since(*since) <= idle_timeout
                    }
                    _ => true,
                });
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
                // Built as hyper-util's builder builds a pool by default: with no timer.
                let pool = start(s, tasks, None, IDLE_TIMEOUT);
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
