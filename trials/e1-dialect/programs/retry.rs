// Task 6: retry -- src/lib.rs
//
// HTTP GET with exponential backoff under one overall deadline.

use std::net::{Net, NetError, Url};
use std::time::{Clock, Duration};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Policy {
    /// At least 1.
    pub attempts: u32,
    /// Doubled after every failed attempt.
    pub first_delay: Duration,
    /// Deadline for the whole call, from its start; bounds every request and sleep.
    pub overall: Duration,
}

#[derive(Debug, Error)]
#[error("GET failed after {attempts} attempt(s): {last}")]
pub struct RetryError {
    pub attempts: u32,
    pub last: NetError,
}

/// GETs `url` until a request succeeds, a non-transient error occurs, the
/// attempts run out, or no time is left before the overall deadline. A
/// cancellation is returned at once.
#[requires(policy.attempts >= 1)]
pub fn fetch_with_retry(net: &Net, clock: &Clock, url: &Url, policy: Policy) -> Result<Vec<u8>, RetryError> {
    let deadline = clock.now() + policy.overall;
    let mut delay = policy.first_delay;
    let mut attempts: u32 = 0;
    loop {
        attempts += 1;
        let last = match net.get(url, deadline) {
            Ok(body) => return Ok(body),
            Err(e) => e,
        };
        if last.is_cancelled() || !last.is_transient() || attempts >= policy.attempts {
            return Err(RetryError { attempts, last });
        }
        // Sleep only if a request can still start before the deadline.
        let wake = match clock.now().checked_add(delay) {
            Some(t) if t < deadline => t,
            _ => return Err(RetryError { attempts, last }),
        };
        if clock.sleep_until(wake).is_err() {
            return Err(RetryError { attempts, last: NetError::Cancelled });
        }
        delay = delay.saturating_mul(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sim::{Reply, SimRoot};

    fn url() -> Url {
        Url::parse("https://api.test/item").unwrap()
    }

    fn policy(overall: Duration) -> Policy {
        Policy { attempts: 5, first_delay: Duration::from_millis(100), overall }
    }

    #[sim_test]
    fn succeeds_on_the_third_attempt(sim: SimRoot) {
        let calls = Rc::new(Cell::new(0u32));
        let seen = Rc::clone(&calls);
        sim.world.http("api.test:443", move |_req| {
            seen.set(seen.get() + 1);
            if seen.get() <= 2 { Reply::status(503) } else { Reply::ok("ok") }
        });
        let body = fetch_with_retry(&sim.net, &sim.clock, &url(), policy(Duration::from_secs(10))).unwrap();
        assert_eq!(body, b"ok".to_vec());
        assert_eq!(calls.get(), 3);
        // Sleeps of 100 ms and 200 ms (first_delay * 3), plus simulated packet time.
        let t = sim.elapsed();
        assert!(t >= Duration::from_millis(300), "elapsed {t:?}");
        assert!(t < Duration::from_millis(900), "elapsed {t:?}");
    }

    #[sim_test]
    fn not_found_fails_after_one_attempt(sim: SimRoot) {
        sim.world.http("api.test:443", |_req| Reply::status(404));
        let err = fetch_with_retry(&sim.net, &sim.clock, &url(), policy(Duration::from_secs(10))).unwrap_err();
        assert_eq!(err.attempts, 1);
        assert!(matches!(err.last, NetError::Status(404)));
    }

    #[sim_test]
    fn hanging_peer_times_out_within_overall(sim: SimRoot) {
        sim.world.http("api.test:443", |_req| Reply::hang());
        let err = fetch_with_retry(&sim.net, &sim.clock, &url(), policy(Duration::from_secs(2))).unwrap_err();
        assert!(matches!(err.last, NetError::Timeout));
        assert_eq!(err.attempts, 1); // the request ran until the overall deadline
        assert!(sim.elapsed() <= Duration::from_millis(2_100), "elapsed {:?}", sim.elapsed());
    }

    #[sim_test]
    fn net_where_everything_times_out(sim: SimRoot) {
        let net = sim.net.drop_all();
        let err = fetch_with_retry(&net, &sim.clock, &url(), policy(Duration::from_secs(2))).unwrap_err();
        assert!(matches!(err.last, NetError::Timeout));
        assert!(err.attempts >= 1 && err.attempts <= 5);
        assert!(sim.elapsed() <= Duration::from_millis(2_100), "elapsed {:?}", sim.elapsed());
    }
}
