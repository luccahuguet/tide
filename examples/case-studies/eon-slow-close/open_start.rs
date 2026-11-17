//@ expect: test fails inspect_is_answered_while_a_terminal_starts
//@ reports: cause: Venus's Inspect got no answer within 6s while a terminal started: Timeout
//@ reports: 0 passed; 20 failed (20 seeds)
// eon-pimct, open: fixed.rs's supervisor with terminal starts added as Eon's `edge` branch runs
// them. A start waits inside the request loop for the new terminal's Ready, up to `edge`'s 15 s
// start timeout, so a terminal that never becomes Ready holds Venus's Inspect past its 6 s
// timeout.
// The test is eon-pimct's own falsifier: hold one new terminal's Ready; a read-only Inspect
// must be answered before that start is released or times out.
use std::sync::mpsc::{Receiver, SyncSender};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration};

/// One close's shared bound.
const CLOSE_BOUND: Duration = Duration::from_secs(5);
/// How long a start waits for the new terminal's Ready (`TERMINAL_START_TIMEOUT` on Eon's
/// `edge`, commit `283bf76a`; the v0.1.16 release on `main` waits 5 s).
const START_TIMEOUT: Duration = Duration::from_secs(15);

pub struct Terminal {
    pub id: u32,
    /// How long Orbit takes to confirm this terminal's end once told to stop.
    pub ends_after: Duration,
}

pub enum Request {
    /// Venus's read-only workspace read; the answer is the open terminals' ids.
    Inspect(SyncSender<Vec<u32>>),
    /// Close the tab; the answer is the terminals it kept.
    Close(SyncSender<Vec<u32>>),
    /// Start a terminal, which signals on the `Receiver` once it is Ready; the answer is
    /// whether it started.
    Start(Terminal, Receiver<()>, SyncSender<bool>),
}

/// The supervisor: one task answers every request, in order, until every sender is gone.
pub fn serve(tasks: &Tasks, clock: &Clock, requests: Receiver<Request>, mut tab: Vec<Terminal>) {
    for request in requests {
        match request {
            Request::Inspect(reply) => {
                let _ = reply.send(ids(&tab));
            }
            Request::Close(reply) => {
                tab = close(tasks, clock, tab);
                let _ = reply.send(ids(&tab));
            }
            Request::Start(terminal, ready, reply) => {
                // eon-pimct: the loop waits here for Ready, and answers nothing else meanwhile.
                let started = ready.recv_deadline(clock.now() + START_TIMEOUT).is_ok();
                if started {
                    tab.push(terminal);
                }
                let _ = reply.send(started);
            }
        }
    }
}

fn ids(tab: &[Terminal]) -> Vec<u32> {
    tab.iter().map(|t| t.id).collect()
}

/// Stops every terminal at once and keeps those whose end was not confirmed within the bound.
fn close(tasks: &Tasks, clock: &Clock, tab: Vec<Terminal>) -> Vec<Terminal> {
    let ended = tasks.scope::<_, TaskError, _>(|s| {
        s.set_deadline(clock.now() + CLOSE_BOUND);
        // At the deadline each unfinished stop's wait is cancelled; it answers `false`.
        let stops: Vec<_> = tab
            .iter()
            .map(|terminal| s.spawn(move || Ok(clock.sleep(terminal.ends_after).is_ok())))
            .collect();
        stops
            .into_iter()
            .map(|stop| stop.join())
            .collect::<Result<Vec<bool>, _>>()
    });
    let ended = ended.expect("no stop returns an error, so the scope cannot fail");
    tab.into_iter()
        .zip(ended)
        .filter(|(_, ended)| !ended)
        .map(|(t, _)| t)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;
    use std::sync::mpsc::sync_channel;

    /// Venus gives up on an answer after 6 s.
    const VENUS_TIMEOUT: Duration = Duration::from_secs(6);

    #[sim_test(seeds = 20)]
    fn inspect_is_answered_while_a_terminal_starts(sim: SimRoot) {
        let (tasks, clock) = (&sim.tasks, &sim.clock);
        let (requests, inbox) = sync_channel(16, tasks);
        tasks
            .scope::<_, TaskError, _>(move |s| {
                s.spawn(move || {
                    serve(tasks, clock, inbox, Vec::new());
                    Ok(())
                });
                // The test holds the new terminal's Ready: it never signals.
                let (held, ready) = sync_channel(1, tasks);
                let (started, outcome) = sync_channel(1, tasks);
                let terminal = Terminal {
                    id: 1,
                    ends_after: Duration::from_millis(500),
                };
                requests
                    .send(Request::Start(terminal, ready, started))
                    .unwrap();
                let (reply, answer) = sync_channel(1, tasks);
                let sent = clock.now();
                requests.send(Request::Inspect(reply)).unwrap();
                if let Err(e) = answer.recv_deadline(sent + VENUS_TIMEOUT) {
                    panic!(
                        "Venus's Inspect got no answer within {VENUS_TIMEOUT:?} while a terminal \
                         started: {e:?}"
                    );
                }
                // Release the start: it fails, as a terminal that never became Ready.
                drop(held);
                assert!(!outcome.recv().unwrap());
                drop(requests);
                Ok(())
            })
            .unwrap();
    }
}
