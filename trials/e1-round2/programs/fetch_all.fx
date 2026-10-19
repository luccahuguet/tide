//! `fetch_all URL...`: fetch every URL, at most 4 at a time, within 10 seconds.

use std::net::{Net, NetError, Url};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration, Instant};

pub const MAX_IN_FLIGHT: usize = 4;
pub const BUDGET: Duration = Duration::from_secs(10);

/// Fetches every URL with at most `MAX_IN_FLIGHT` requests in flight and one
/// overall deadline `BUDGET` from now. Returns one result per URL, in input
/// order: URLs still pending (or never started) when the deadline passes get
/// `Err(NetError::Timeout)`, and the others keep what they got.
pub fn fetch_all(tasks: &Tasks, net: &Net, clock: &Clock, urls: &[Url])
    -> Result<Vec<Result<Vec<u8>, NetError>>, TaskError>
{
    let deadline = clock.now() + BUDGET;
    tasks.limit(MAX_IN_FLIGHT).scope(|s| {
        s.set_deadline(deadline);                       // cancels stragglers; not a failure
        let pending: Vec<_> = urls.iter()
            .map(|url| s.spawn(move || Ok(fetch_one(net, url, deadline))))
            .collect();
        pending.into_iter().map(|task| task.join()).collect()
    })
}

/// Never fails the scope: every error, including the cancellation at the
/// deadline, becomes this URL's result.
fn fetch_one(net: &Net, url: &Url, deadline: Instant) -> Result<Vec<u8>, NetError> {
    match net.get(url, deadline) {
        Err(e) if e.is_cancelled() => Err(NetError::Timeout),   // cut off by the scope deadline
        other => other,
    }
}

fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let urls = root.env.args().skip(1).map(|a| Url::parse(&a)).collect::<Result<Vec<Url>, _>>()?;
    let hosts: Vec<String> = urls.iter()
        .filter_map(|u| Some(format!("{}:{}", u.host_str()?, u.port_or_known_default()?)))
        .collect();
    let net = root.net.only(hosts).no_bind();          // only the hosts named on the command line
    let results = fetch_all(&root.tasks, &net, &root.clock, &urls)?;
    for (url, result) in urls.iter().zip(&results) {
        match result {
            Ok(body) => println!("{url}: {} bytes", body.len()),
            Err(e) => println!("{url}: {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{Reply, SimRoot};

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    /// Half of all network traffic is dropped. Every URL still gets a result,
    /// each result is its own body or a timeout, and the call returns at the
    /// deadline.
    #[sim_test(seeds = 200)]
    fn half_the_packets_dropped(sim: SimRoot) {
        let mut urls = Vec::new();
        for i in 0..10 {
            sim.world.http(&format!("h{i}.test:443"), move |_req| Reply::ok(format!("body {i}")));
            urls.push(url(&format!("https://h{i}.test/")));
        }
        sim.faults.net_drop(0.5);
        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls).unwrap();
        assert_eq!(got.len(), urls.len());
        for (i, result) in got.iter().enumerate() {
            match result {
                Ok(body) => assert_eq!(body, format!("body {i}").as_bytes()),
                Err(e) => assert!(matches!(e, NetError::Timeout), "url {i}: {e}"),
            }
        }
        assert!(sim.elapsed() <= BUDGET + Duration::from_secs(1));   // slack for operation latencies
    }

    /// Four hung requests hold every slot until the deadline, so the URL
    /// queued behind them never starts: partial results, at most 4 in flight.
    #[sim_test]
    fn hung_hosts_hold_every_slot(sim: SimRoot) {
        sim.world.http("fast.test:443", |_req| Reply::ok("ok"));
        sim.world.http("hung.test:443", |_req| Reply::hang());
        let mut urls = vec![url("https://fast.test/")];
        urls.extend((0..4).map(|i| url(&format!("https://hung.test/{i}"))));
        urls.push(url("https://fast.test/late"));
        let got = fetch_all(&sim.tasks, &sim.net, &sim.clock, &urls).unwrap();
        assert!(matches!(&got[0], Ok(body) if body == b"ok"));
        assert!(got[1..].iter().all(|r| matches!(r, Err(NetError::Timeout))));
        assert!(sim.elapsed() >= BUDGET);
    }
}
