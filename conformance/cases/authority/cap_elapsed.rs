//@ rules: CAP-1
// `elapsed` reads the clock, so Tide's `Instant` and `SystemTime` have no `elapsed`; measure
// with a second `now()` from the `Clock` (03 CAP-1).
use std::time::{Clock, Duration, Instant, SystemTime};

pub fn since(clock: &Clock, start: Instant) -> Duration {
    clock.now().duration_since(start)
}

pub fn wall(clock: &Clock) -> SystemTime {
    clock.system_time()
}

pub fn since_ambient(start: Instant) -> Duration {
    start.elapsed() //~ E0599 FX0501
}

pub fn wall_ambient(t: SystemTime) -> bool {
    t.elapsed().is_ok() //~ E0599 FX0501
}
