// src/main.fx of package `trials/fetch-all`
//
// `fetch_all URL...` fetches every URL with at most 4 requests in flight and
// one overall 10-second deadline, and prints one result per URL in input order.

use std::net::{Net, NetError, Url};
use std::sync::{Poisoned, Shared};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration, Instant};

pub const MAX_IN_FLIGHT: usize = 4;
pub const DEADLINE: Duration = Duration::from_secs(10);
const MAX_ATTEMPTS: u32 = 3;
const FIRST_BACKOFF: Duration = Duration::from_millis(100);

/// The outcome for one URL: the body, or why there is none.
pub type UrlResult = Result<Vec<u8>, NetError>;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error("the shared work queue is poisoned")]
    Poisoned(#[from] Poisoned),
}

/// Authority: tasks, network, clock.
///
/// A pool of at most `MAX_IN_FLIGHT` workers pulls URL indices from a shared
/// counter, so no more than four requests are ever in flight. Workers report
/// per-URL failures as data, so one bad host cancels nothing. The scope
/// deadline cancels whatever is still running at `DEADLINE`.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url]) -> Result<Vec<UrlResult>, FetchError> {
    let deadline = clock.now() + DEADLINE;
    let next_cell = Shared::new(0usize);
    let next = &next_cell;
    let batches = tasks.scope(|s| {
        s.set_deadline(deadline);
        let workers: Vec<_> = (0..MAX_IN_FLIGHT.min(urls.len()))
            .map(|_| s.spawn(move || worker(net, clock, urls, next, deadline)))
            .collect();
        workers.into_iter().map(|w| w.join()).collect::<Result<Vec<_>, _>>()
    })?;

    // Deterministic reassembly: results go back to their input positions.
    let mut slots: Vec<Option<UrlResult>> = urls.iter().map(|_| None).collect();
    for (i, result) in batches.into_iter().flatten() {
        slots[i] = Some(result);
    }
    Ok(slots
        .into_iter()
        .map(|r| r.expect("every index is claimed by exactly one worker"))
        .collect())
}

/// One worker: claim the next index, fetch it, repeat until none are left.
fn worker(net: &Net, clock: &Clock, urls: &[Url], next: &Shared<usize>, deadline: Instant)
    -> Result<Vec<(usize, UrlResult)>, FetchError>
{
    let mut done = Vec::new();
    loop {
        let i = lock (next) |n| {
            let i = *n;
            *n += 1;
            i
        }?;
        let Some(url) = urls.get(i) else {
            return Ok(done);
        };
        done.push((i, fetch_one(net, clock, url, deadline)));
    }
}

/// Retries transient failures with exponential backoff until the deadline.
fn fetch_one(net: &Net, clock: &Clock, url: &Url, deadline: Instant) -> UrlResult {
    let mut backoff = FIRST_BACKOFF;
    let mut attempt = 1;
    loop {
        match net.get(url, deadline) {
            Ok(body) => return Ok(body),
            Err(e) if e.is_cancelled() || matches!(e, NetError::Timeout) || attempt == MAX_ATTEMPTS => {
                return Err(e);
            }
            Err(e) => {
                warn!("{url}: attempt {attempt} failed: {e}; retrying in {backoff:?}");
                if clock.sleep(backoff).is_err() {
                    return Err(NetError::Cancelled); // the deadline passed while backing off
                }
                backoff *= 2;
                attempt += 1;
            }
        }
    }
}

fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let urls = root
        .env
        .args()
        .skip(1)
        .map(|arg| Url::parse(&arg))
        .collect::<Result<Vec<_>, _>>()?;
    let results = fetch_all(&root.tasks, &root.net, &root.clock, &urls)?;
    for (url, result) in urls.iter().zip(&results) {
        match result {
            Ok(body) => println!("{url}: {} bytes", body.len()),
            Err(e) => println!("{url}: error: {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{Faults, Reply};

    fn urls(n: usize) -> Vec<Url> {
        (0..n)
            .map(|i| Url::parse(&format!("https://svc.test/item/{i}")).unwrap())
            .collect()
    }

    #[test]
    fn at_most_four_requests_in_flight() {
        let sim = Root::sim(1);
        sim.world.http("svc.test:443", |_req| Reply::ok("ok").after(Duration::from_secs(1)));
        let start = sim.clock.now();

        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls(8)).unwrap();

        let elapsed = sim.clock.now() - start;
        assert_eq!(got.len(), 8);
        assert!(got.iter().all(|r| matches!(r, Ok(body) if body == b"ok")));
        // Eight one-second requests through four slots take two waves of virtual time;
        // unbounded concurrency would finish in one.
        assert!(elapsed >= Duration::from_secs(2), "elapsed {elapsed:?}");
        assert!(elapsed < Duration::from_secs(3), "elapsed {elapsed:?}");
    }

    #[test]
    fn half_the_packets_dropped(seed: u64) {
        let sim = Root::sim(seed);
        sim.world.faults(Faults { net_drop: 0.5, ..Faults::none() });
        sim.world.http("svc.test:443", |_req| Reply::ok("ok").after(Duration::from_millis(50)));
        let list = urls(12);
        let start = sim.clock.now();

        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &list).unwrap();

        assert_eq!(got.len(), list.len()); // one result per URL, in input order
        assert!(sim.clock.now() - start <= DEADLINE, "overran the overall deadline");
        for result in &got {
            if let Ok(body) = result {
                assert_eq!(body, b"ok"); // lost packets never corrupt a delivered body
            }
        }
        let failed = got.iter().filter(|r| r.is_err()).count();
        info!("seed {seed}: {failed} of {} URLs failed", got.len());
    }

    #[test]
    fn everything_fails_by_the_deadline_when_the_network_is_down() {
        let sim = Root::sim(9);
        let net = sim.net.drop_all();
        let start = sim.clock.now();

        let got = fetch_all(&sim.tasks, &net, &sim.clock, &urls(6)).unwrap();

        assert!(got.iter().all(|r| r.is_err()));
        assert!(sim.clock.now() - start <= DEADLINE);
    }
}
