use std::{
    task::Cancelled,
    time::{Clock, Duration},
};
#[derive(Debug, PartialEq)]
pub struct Report {
    pub calls: u32,
    pub last_ok: Option<u32>,
    pub elapsed: Duration,
}
pub fn watch(
    clock: &Clock,
    interval: Duration,
    misses: u32,
    mut probe: impl FnMut() -> bool,
) -> Result<Report, Cancelled> {
    assert!(misses > 0);
    let start = clock.now();
    let mut next = start;
    let mut bad = 0;
    let mut out = Report {
        calls: 0,
        last_ok: None,
        elapsed: Duration::ZERO,
    };
    loop {
        clock.sleep_until(next)?;
        // The immediate first call is the start, rather than scheduler overhead.
        out.elapsed = if out.calls == 0 {
            Duration::ZERO
        } else {
            clock.now().duration_since(start)
        };
        if probe() {
            out.last_ok = Some(out.calls);
            bad = 0;
        } else {
            bad += 1;
        }
        out.calls += 1;
        if bad == misses {
            return Ok(out);
        }
        if interval.is_zero() {
            next = clock.now();
        } else {
            next += interval;
            let now = clock.now();
            while next < now {
                next += interval;
            }
        }
    }
}
