//@ rules: CAP-1
// A timed wait reads the clock, so Tide's `Receiver` has `recv_deadline`, which takes an
// `Instant` from a `Clock`, and no `recv_timeout` (03 CAP-1, §6.4).
use std::error::Error;
use std::sync::mpsc::Receiver;
use std::time::{Clock, Duration};

pub fn poll(rx: &Receiver<u32>, clock: &Clock) -> Result<u32, Box<dyn Error + Send + Sync>> {
    Ok(rx.recv_deadline(clock.now() + Duration::from_millis(10))?)
}

pub fn poll_ambient(rx: &Receiver<u32>) -> Option<u32> {
    rx.recv_timeout(Duration::from_millis(10)).ok() //~ E0599 TD0501
}
