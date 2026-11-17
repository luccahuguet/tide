//@ expect: tests pass
// Eon's supervisor after d431fcab (eon-8wh7h), modelled: a close stops its terminals together
// under one shared five-second bound, below Venus's 6 s read timeout, and keeps only the
// terminals whose end it could not confirm. The request loop is unchanged: a close still runs
// inside it, but for at most the bound. The test that fails on buggy.rs passes here.
use std::sync::mpsc::{Receiver, SyncSender};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration};

/// One close's shared bound.
const CLOSE_BOUND: Duration = Duration::from_secs(5);

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
                    serve(tasks, clock, inbox, tab);
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

    /// The fix's own promise, after Eon's test
    /// `a_close_stops_its_terminals_together_and_keeps_only_the_unended`: a terminal whose end
    /// is not confirmed within the bound stays in the tab, and Inspect is still answered.
    #[sim_test(seeds = 20)]
    fn a_close_keeps_only_the_unended(sim: SimRoot) {
        let mut tab = slow_tab();
        tab[3].ends_after = Duration::from_secs(60);
        assert_eq!(close_while_inspecting(&sim, tab), vec![4]);
    }
}
