use std::{
    thread,
    time::{Duration, Instant},
};
#[derive(Debug, PartialEq)]
pub struct Report {
    pub calls: u32,
    pub last_ok: Option<u32>,
    pub elapsed: Duration,
}
pub fn watch(interval: Duration, misses: u32, mut probe: impl FnMut() -> bool) -> Report {
    assert!(misses > 0);
    let start = Instant::now();
    let mut next = start;
    let mut bad = 0;
    let mut out = Report {
        calls: 0,
        last_ok: None,
        elapsed: Duration::ZERO,
    };
    loop {
        thread::sleep(next.saturating_duration_since(Instant::now()));
        out.elapsed = start.elapsed();
        if probe() {
            out.last_ok = Some(out.calls);
            bad = 0;
        } else {
            bad += 1;
        }
        out.calls += 1;
        if bad == misses {
            return out;
        }
        if interval.is_zero() {
            next = Instant::now();
        } else {
            next += interval;
            let now = Instant::now();
            while next < now {
                next += interval;
            }
        }
    }
}
