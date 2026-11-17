//@ expect: test fails inspect_is_answered_while_a_tab_closes
//@ reports: cause: Venus's Inspect, sent 250ms into the close, got no answer within 6s: Timeout
//@ reports: 0 passed; 20 failed (20 seeds)
// Eon's supervisor before d431fcab (eon-8wh7h), modelled: one task answers every request, and a
// tab close stops its terminals one after another inside the request handler. While the close
// runs, Venus's Inspect waits behind it. A 14-terminal tab whose terminals each take Orbit's
// 500 ms grace holds the loop for 7 s, past Venus's 6 s read timeout.
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Clock, Duration};

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
}

/// The supervisor: one task answers every request, in order, until every sender is gone.
pub fn serve(clock: &Clock, requests: Receiver<Request>, mut tab: Vec<Terminal>) {
    for request in requests {
        match request {
            Request::Inspect(reply) => {
                let _ = reply.send(ids(&tab));
            }
            Request::Close(reply) => {
                tab = close(clock, tab);
                let _ = reply.send(ids(&tab));
            }
        }
    }
}

fn ids(tab: &[Terminal]) -> Vec<u32> {
    tab.iter().map(|t| t.id).collect()
}

/// Stops each terminal in turn, waiting for its end before the next; keeps any it could not
/// confirm.
fn close(clock: &Clock, tab: Vec<Terminal>) -> Vec<Terminal> {
    let mut kept = Vec::new();
    for terminal in tab {
        if clock.sleep(terminal.ends_after).is_err() {
            kept.push(terminal);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;
    use std::sync::mpsc::sync_channel;
    use std::task::TaskError;

    /// Venus reads the workspace every 250 ms and gives up on an answer after 6 s.
    const REFRESH: Duration = Duration::from_millis(250);
    const VENUS_TIMEOUT: Duration = Duration::from_secs(6);

    /// eon-8wh7h's reproduction: 14 terminals, each taking Orbit's 500 ms grace to stop.
    fn slow_tab() -> Vec<Terminal> {
        let grace = Duration::from_millis(500);
        (1..=14)
            .map(|id| Terminal {
                id,
                ends_after: grace,
            })
            .collect()
    }

    /// Closes `tab` while Venus inspects for 8 s, each Inspect under Venus's timeout; returns
    /// the terminals the close kept.
    fn close_while_inspecting(sim: &SimRoot, tab: Vec<Terminal>) -> Vec<u32> {
        let (tasks, clock) = (&sim.tasks, &sim.clock);
        let (requests, inbox) = sync_channel(16, tasks);
        tasks
            .scope::<_, TaskError, _>(move |s| {
                s.spawn(move || {
                    serve(clock, inbox, tab);
                    Ok(())
                });
                let (closed, kept) = sync_channel(1, tasks);
                requests.send(Request::Close(closed)).unwrap();
                let start = clock.now();
                while clock.now() < start + Duration::from_secs(8) {
                    clock.sleep(REFRESH).unwrap();
                    let (reply, answer) = sync_channel(1, tasks);
                    let sent = clock.now();
                    requests.send(Request::Inspect(reply)).unwrap();
                    if let Err(e) = answer.recv_deadline(sent + VENUS_TIMEOUT) {
                        panic!(
                            "Venus's Inspect, sent {:?} into the close, got no answer within \
                             {VENUS_TIMEOUT:?}: {e:?}",
                            sent - start
                        );
                    }
                }
                drop(requests);
                Ok(kept.recv().unwrap())
            })
            .unwrap()
    }

    #[sim_test(seeds = 20)]
    fn inspect_is_answered_while_a_tab_closes(sim: SimRoot) {
        assert_eq!(close_while_inspecting(&sim, slow_tab()), Vec::<u32>::new());
    }
}
