//@ expect: rejected TD0601 TD0602 E0433
// hyper 0.14's client pool with its `runtime` feature, modelled. The pool's own task runs in the
// caller's scope, but its idle interval reaches for the ambient tokio runtime, as hyper's
// `spawn_idle_interval` did: it is spawned with `tokio::spawn` (hyper's default executor), ticks
// on `tokio::time::interval`, and stamps idle connections with `tokio::time::Instant::now()`.
// Under another executor that interval panicked ("there is no reactor running"); without the
// feature hyper started no interval at all, and idle connections were never reaped.
//
// Admission rejects the spawn (`TD0602`) and the async block it spawns (`TD0601`). The tokio
// timer has no Tide code: Tide admits no external crate, so `tokio` is an unresolved path
// (`E0433`).
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::task::{Scope, TaskError, Tasks};
use std::time::Duration;

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

/// Starts the pool's task in the caller's scope `bg`, and its idle interval on whichever tokio
/// runtime is ambient.
pub fn start<'s>(bg: Scope<'s, '_, TaskError>, tasks: &Tasks, idle_timeout: Duration) -> Pool {
    let (requests, inbox) = sync_channel(16, tasks);
    bg.spawn(move || {
        run(idle_timeout, inbox);
        Ok(())
    });
    let ticks = requests.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(idle_timeout);
        loop {
            interval.tick().await;
            if ticks.send(PoolMsg::Expire).is_err() {
                break;
            }
        }
    });
    Pool { requests }
}

/// The pool's task: owns the idle connections; dropping one closes it.
fn run(idle_timeout: Duration, requests: Receiver<PoolMsg>) {
    let mut idle: Vec<(TcpStream, tokio::time::Instant)> = Vec::new();
    for msg in requests {
        match msg {
            PoolMsg::Checkout(reply) => {
                let _ = reply.send(idle.pop().map(|(conn, _)| conn));
            }
            PoolMsg::Checkin(conn) => idle.push((conn, tokio::time::Instant::now())),
            PoolMsg::Expire => {
                let now = tokio::time::Instant::now();
                idle.retain(|(_, since)| now.saturating_duration_since(*since) <= idle_timeout);
            }
        }
    }
}
